//! Application host for the Winit + Taffy + Vello Hybrid backend.

#[cfg(feature = "vite")]
use std::cell::RefCell;
use std::{
    collections::HashMap,
    path::PathBuf,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use vello_common::peniko::Color;
use wabou_bindgen::{CapabilityContract, JsonMethod};

use crate::{Applier, ImageResourceStore, JsRuntime, JsRuntimeOptions};
use vello_shell::{FrameSource, Widget, WidgetFactory, WindowOptions};

type CapabilityInstaller = Arc<dyn Fn(&JsRuntime) -> rquickjs::Result<()>>;
type HostMessageProducer = Arc<dyn Fn(crate::HostMessageContext) + Send + Sync>;

struct HostServicesGuard(Vec<Arc<dyn runtime_api::HostService>>);

impl HostServicesGuard {
    fn finish(mut self) -> crate::Result<()> {
        let mut failures = Vec::new();
        while let Some(service) = self.0.pop() {
            if let Err(error) = service.shutdown() {
                failures.push(format!("`{}`: {error}", service.name()));
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(crate::Error::HostServiceShutdown {
                message: failures.join("; "),
            })
        }
    }
}

impl Drop for HostServicesGuard {
    fn drop(&mut self) {
        while let Some(service) = self.0.pop() {
            if let Err(error) = service.shutdown() {
                tracing::warn!(service = service.name(), %error, "failed to shut down host service");
            }
        }
    }
}

fn start_services(
    services: &[(Arc<dyn runtime_api::HostService>, bool)],
    context: &runtime_api::HostServiceContext,
) -> crate::Result<HostServicesGuard> {
    let mut started_services = HostServicesGuard(Vec::with_capacity(services.len()));
    for (service, required) in services {
        let started = Instant::now();
        match service.start(context) {
            Ok(()) => started_services.0.push(service.clone()),
            Err(message) if *required => {
                return Err(crate::Error::HostService {
                    name: service.name(),
                    message,
                });
            }
            Err(message) => {
                tracing::warn!(service = service.name(), %message, "recoverable host service failed to start");
            }
        }
        if started.elapsed() >= Duration::from_secs(1) {
            tracing::warn!(
                service = service.name(),
                elapsed_ms = started.elapsed().as_millis(),
                "host service startup was slow"
            );
        }
    }
    Ok(started_services)
}

#[derive(Clone)]
enum ApplicationSource {
    Bundle {
        code: String,
        source_map: Option<Vec<u8>>,
    },
    #[cfg(feature = "vite")]
    Vite { url: String, entry: String },
}

#[derive(Clone)]
struct RuntimeSourceConfig {
    source: ApplicationSource,
    js_runtime_options: JsRuntimeOptions,
    capabilities: Vec<CapabilityInstaller>,
    host_message_producers: Vec<HostMessageProducer>,
    widget_factories: HashMap<String, WidgetFactory>,
    base_color: Color,
    image_resources: ImageResourceStore,
    effect_trace: Option<crate::effect_trace::EffectTrace>,
    app_directories: Option<vello_shell::AppDirectories>,
    #[cfg(feature = "devtools")]
    debug_state: Option<wabou_devtools::SharedDebugState>,
    #[cfg(feature = "vite")]
    hmr_clients: Rc<RefCell<Vec<runtime_api::HmrClient>>>,
}

impl RuntimeSourceConfig {
    fn create(
        &self,
        window_key: vello_shell::WindowResourceKey,
        options: &WindowOptions,
    ) -> crate::Result<Applier> {
        #[cfg(feature = "vite")]
        let js = match &self.source {
            ApplicationSource::Bundle { .. } => {
                JsRuntime::new_with_options(self.js_runtime_options)
            }
            ApplicationSource::Vite { url, .. } => {
                JsRuntime::new_vite_with_options(url, self.js_runtime_options)
            }
        }
        .context(crate::error::JavaScriptSnafu {
            operation: "create JavaScript runtime",
        })?;
        #[cfg(not(feature = "vite"))]
        let js = JsRuntime::new_with_options(self.js_runtime_options).context(
            crate::error::JavaScriptSnafu {
                operation: "create JavaScript runtime",
            },
        )?;
        for capability in &self.capabilities {
            capability(&js).context(crate::error::JavaScriptSnafu {
                operation: "mount JavaScript capability",
            })?;
        }
        let serialized_window_options =
            serde_json::to_string(options).expect("WindowOptions must remain serializable");
        js.with(|ctx| {
            ctx.globals()
                .set("__wabou_window_options_json", serialized_window_options)
        })
        .context(crate::error::JavaScriptSnafu {
            operation: "install native window creation options",
        })?;
        let mut controller = Applier::from_runtime_with_factories_and_window(
            js,
            self.widget_factories.clone(),
            self.base_color,
            window_key,
        );
        controller.set_image_resource_store(self.image_resources.clone());
        if let Some(trace) = &self.effect_trace {
            controller.set_effect_trace(trace.clone());
        }
        #[cfg(feature = "devtools")]
        if let Some(state) = &self.debug_state {
            controller.set_debug_state(state.clone());
        }
        if let Some(directories) = &self.app_directories {
            controller.set_app_directories(directories.clone());
        }
        let message_context = controller.host_message_context(window_key);
        for producer in &self.host_message_producers {
            producer(message_context.clone());
        }
        match &self.source {
            ApplicationSource::Bundle { code, source_map } => {
                controller
                    .boot_with_source_map(code, source_map.as_deref())
                    .context(crate::error::JavaScriptSnafu {
                        operation: "boot JavaScript bundle",
                    })?;
            }
            #[cfg(feature = "vite")]
            ApplicationSource::Vite { url, entry } => {
                controller.set_vite_entry(entry);
                controller
                    .boot_vite(entry)
                    .context(crate::error::JavaScriptSnafu {
                        operation: "boot Vite entry module",
                    })?;
                let reload = controller.reload_handle();
                self.hmr_clients.borrow_mut().push(
                    runtime_api::start_hmr_bridge(url, move |event| {
                        reload
                            .send(match event {
                                runtime_api::ViteHmrEvent::Update {
                                    path,
                                    accepted_path,
                                    timestamp,
                                    source,
                                } => crate::reload::ReloadMsg::HmrUpdate {
                                    path,
                                    accepted_path,
                                    timestamp,
                                    source,
                                },
                                runtime_api::ViteHmrEvent::CssUpdate { path } => {
                                    crate::reload::ReloadMsg::CssUpdate { path }
                                }
                                runtime_api::ViteHmrEvent::FullReload => {
                                    crate::reload::ReloadMsg::FullReload
                                }
                                runtime_api::ViteHmrEvent::Error { diagnostic } => {
                                    crate::reload::ReloadMsg::Error { diagnostic }
                                }
                            })
                            .is_ok()
                    })
                    .context(crate::error::ViteSnafu)?,
                );
            }
        }
        Ok(controller)
    }
}

enum EffectTraceConfig {
    Record { path: PathBuf, record_all: bool },
    Replay { path: PathBuf },
}

const IMAGE_RESOURCES: CapabilityContract = CapabilityContract::new("imageResources", 1);
const CREATE_FILE_IMAGE: JsonMethod<CreateFileImageRequest, ImageResourceDescriptor> =
    JsonMethod::new("createFile");
const CREATE_NETWORK_IMAGE: JsonMethod<CreateNetworkImageRequest, ImageResourceDescriptor> =
    JsonMethod::new("createNetwork");
const RELEASE_IMAGE: JsonMethod<crate::ImageResourceHandle, bool> = JsonMethod::new("release");

#[derive(Deserialize)]
struct CreateFileImageRequest {
    path: String,
}

#[derive(Deserialize)]
struct CreateNetworkImageRequest {
    url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageResourceDescriptor {
    handle: crate::ImageResourceHandle,
    width: u32,
    height: u32,
}

/// Builder for the independent Winit + Taffy + Vello Hybrid application path.
///
/// It consumes the same JavaScript bundle and binary mutation protocol as the
/// GPUI host. The distinct type keeps backend-specific native widgets explicit
/// while the second backend is brought to feature parity.
pub struct VelloHybridHostBuilder {
    window: WindowOptions,
    additional_windows: Vec<WindowOptions>,
    base_color: Color,
    widget_factories: HashMap<String, WidgetFactory>,
    capabilities: Vec<CapabilityInstaller>,
    host_message_producers: Vec<HostMessageProducer>,
    services: Vec<(Arc<dyn runtime_api::HostService>, bool)>,
    app_directory_config: Option<vello_shell::AppDirectoryConfig>,
    persisted_window_size: Option<String>,
    kv_enabled: bool,
    devtools: bool,
    shell_extensions: Vec<Box<dyn vello_shell::ShellExtension>>,
    effect_trace: Option<EffectTraceConfig>,
    image_resources: ImageResourceStore,
    js_runtime_options: JsRuntimeOptions,
}

impl Default for VelloHybridHostBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl VelloHybridHostBuilder {
    /// Construct a host with the standard Vello Hybrid native-widget registry.
    pub fn new() -> Self {
        Self::with_image_resources(ImageResourceStore::default())
    }

    /// Construct a host whose Rust code shares ownership of image resources.
    pub fn with_image_resources(image_resources: ImageResourceStore) -> Self {
        let builder = Self {
            window: WindowOptions::default(),
            additional_windows: Vec::new(),
            base_color: Color::from_rgb8(0x0f, 0x17, 0x2a),
            widget_factories: wabou_widgets_vello::builtin_factories(),
            capabilities: Vec::new(),
            host_message_producers: Vec::new(),
            services: Vec::new(),
            app_directory_config: None,
            persisted_window_size: None,
            kv_enabled: false,
            devtools: cfg!(all(debug_assertions, feature = "devtools")),
            shell_extensions: Vec::new(),
            effect_trace: None,
            image_resources: image_resources.clone(),
            js_runtime_options: JsRuntimeOptions::default(),
        };
        let mounted = image_resources;
        builder.capability(IMAGE_RESOURCES, move |capability| {
            let files = mounted.clone();
            capability.json_method(CREATE_FILE_IMAGE, move |request: CreateFileImageRequest| {
                let resources = files.clone();
                async move {
                    let loader = resources.clone();
                    let handle =
                        tokio::task::spawn_blocking(move || loader.create_file(request.path))
                            .await
                            .map_err(|error| error.to_string())??;
                    let (width, height) = resources
                        .get(handle)
                        .ok_or_else(|| "created image resource did not resolve".to_owned())?
                        .dimensions();
                    Ok::<_, String>(ImageResourceDescriptor {
                        handle,
                        width,
                        height,
                    })
                }
            })?;
            let network = mounted.clone();
            capability.json_method(
                CREATE_NETWORK_IMAGE,
                move |request: CreateNetworkImageRequest| {
                    let resources = network.clone();
                    async move {
                        let handle = resources.create_network(&request.url).await?;
                        let (width, height) = resources
                            .get(handle)
                            .ok_or_else(|| "created image resource did not resolve".to_owned())?
                            .dimensions();
                        Ok::<_, String>(ImageResourceDescriptor {
                            handle,
                            width,
                            height,
                        })
                    }
                },
            )?;
            let release = mounted.clone();
            capability.json_method(RELEASE_IMAGE, move |handle: crate::ImageResourceHandle| {
                let resources = release.clone();
                async move { Ok::<_, String>(resources.remove(handle)) }
            })
        })
    }

    /// Configure the primary native window.
    pub fn window(mut self, options: WindowOptions) -> Self {
        self.window = options;
        self
    }

    /// Add another native window running an independent copy of the bundle.
    pub fn additional_window(mut self, options: WindowOptions) -> Self {
        self.additional_windows.push(options);
        self
    }

    /// Register or replace a Winit native widget.
    pub fn widget(
        mut self,
        tag: impl Into<String>,
        factory: impl Fn() -> Box<dyn Widget> + 'static,
    ) -> Self {
        self.widget_factories.insert(tag.into(), Arc::new(factory));
        self
    }

    /// Register secure native password inputs backed by one Rust-only store.
    ///
    /// Values entered into `<PasswordInput>` never cross into QuickJS. Native
    /// capability code can atomically take a value from the matching `secret`
    /// slot through the returned store clone.
    pub fn password_inputs(mut self, secrets: crate::VelloHybridSecretStore) -> Self {
        self.widget_factories.insert(
            "password-input".into(),
            wabou_widgets_vello::password_input_factory(secrets),
        );
        self
    }

    /// Mount one versioned application capability into every window runtime.
    ///
    /// This is API-compatible with the default GPUI host: direct structured
    /// methods and opt-in JSON methods use the same generated contracts.
    pub fn capability<F>(mut self, contract: CapabilityContract, mount: F) -> Self
    where
        F: for<'js> Fn(runtime_api::NativeCapability<'js>) -> rquickjs::Result<()>
            + rquickjs::markers::ParallelSend
            + Send
            + Sync
            + 'static,
    {
        let name = contract.name().to_owned();
        let version = contract.version();
        let mount = Arc::new(mount);
        self.capabilities.push(Arc::new(move |js| {
            let mount = mount.clone();
            js.mount_capability(&name, move |ctx, object| {
                object.set("__wabouCapabilityVersion", version)?;
                mount(runtime_api::NativeCapability::from_runtime_parts(
                    ctx, object,
                ))
            })
        }));
        self
    }

    /// Register a Rust-to-JavaScript producer for every native window.
    pub fn host_message_producer<F>(mut self, producer: F) -> Self
    where
        F: Fn(crate::HostMessageContext) + Send + Sync + 'static,
    {
        self.host_message_producers.push(Arc::new(producer));
        self
    }

    /// Connect the same window-addressable router used by the default GPUI host.
    pub fn host_message_router(mut self, router: runtime_api::HostMessageRouter) -> Self {
        self.host_message_producers
            .push(Arc::new(move |context| router.attach(context)));
        self
    }

    /// Own a required background service for the complete Winit host lifetime.
    pub fn service(mut self, service: impl runtime_api::HostService + 'static) -> Self {
        self.services.push((Arc::new(service), true));
        self
    }

    /// Own a background service whose startup failure leaves the UI available.
    pub fn recoverable_service(mut self, service: impl runtime_api::HostService + 'static) -> Self {
        self.services.push((Arc::new(service), false));
        self
    }

    /// Configure stable application-private platform directories.
    pub fn app_directories(
        self,
        qualifier: impl Into<String>,
        organization: impl Into<String>,
        application: impl Into<String>,
    ) -> Self {
        self.app_directory_config(vello_shell::AppDirectoryConfig::new(
            qualifier,
            organization,
            application,
        ))
    }

    /// Set an already constructed application directory identity.
    pub fn app_directory_config(mut self, config: vello_shell::AppDirectoryConfig) -> Self {
        self.app_directory_config = Some(config);
        self
    }

    /// Restore and persist the primary window's normal logical size.
    pub fn persist_window_size(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        assert!(
            !key.is_empty()
                && key
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')),
            "window persistence key must contain only ASCII letters, numbers, '-' or '_'"
        );
        self.persisted_window_size = Some(key);
        self
    }

    /// Enable the built-in SQLite-backed hierarchical KV capability.
    pub fn kv(mut self) -> Self {
        self.kv_enabled = true;
        self
    }

    /// Enable or disable the local read-only DevTools transport.
    pub fn devtools(mut self, enabled: bool) -> Self {
        self.devtools = enabled;
        self
    }

    /// Install a Winit event-loop extension for tray icons or platform integration.
    pub fn shell_extension(
        mut self,
        extension: impl vello_shell::ShellExtension + 'static,
    ) -> Self {
        self.shell_extensions.push(Box::new(extension));
        self
    }

    /// Record replay-safe native effects when the application exits.
    pub fn record_effects(mut self, path: impl Into<PathBuf>) -> Self {
        self.effect_trace = Some(EffectTraceConfig::Record {
            path: path.into(),
            record_all: false,
        });
        self
    }

    /// Record every native effect, including potentially sensitive payloads.
    pub fn record_all_effects(mut self, path: impl Into<PathBuf>) -> Self {
        self.effect_trace = Some(EffectTraceConfig::Record {
            path: path.into(),
            record_all: true,
        });
        self
    }

    /// Replay effects found in a previously recorded tape.
    pub fn replay_effects(mut self, path: impl Into<PathBuf>) -> Self {
        self.effect_trace = Some(EffectTraceConfig::Replay { path: path.into() });
        self
    }

    /// Set the physical surface clear color.
    pub fn base_color(mut self, rgba: [u8; 4]) -> Self {
        self.base_color = Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]);
        self
    }

    /// Configure the maximum native stack available to every QuickJS runtime.
    pub fn quickjs_stack_size(mut self, bytes: usize) -> Self {
        self.js_runtime_options = self.js_runtime_options.max_stack_size(bytes);
        self
    }

    /// Boot one runtime per window and enter the Winit event loop.
    pub fn run(self) -> crate::Result<()> {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .try_init()
            .ok();

        let test_script = std::env::var_os("WABOU_TEST_SCRIPT")
            .map(PathBuf::from)
            .map(|path| {
                std::fs::read_to_string(&path).context(crate::error::ReadFileSnafu {
                    kind: "test scenario bundle",
                    path,
                })
            })
            .transpose()?;
        let trace_path = self.effect_trace.as_ref().map(|config| match config {
            EffectTraceConfig::Record { path, .. } | EffectTraceConfig::Replay { path } => {
                path.clone()
            }
        });
        let mut effect_trace = match &self.effect_trace {
            Some(EffectTraceConfig::Record { record_all, .. }) => {
                Some(crate::effect_trace::EffectTrace::record(*record_all))
            }
            Some(EffectTraceConfig::Replay { path }) => Some(
                crate::effect_trace::EffectTrace::replay(path)
                    .map_err(|message| crate::Error::EffectTrace { message })?,
            ),
            None => None,
        };
        let recording_effects = matches!(self.effect_trace, Some(EffectTraceConfig::Record { .. }));
        if test_script.is_some() && effect_trace.is_none() {
            effect_trace = Some(crate::effect_trace::EffectTrace::fixtures());
        }
        let test_controller = test_script.as_ref().map(|_| {
            runtime_api::test_driver::TestController::new(
                effect_trace
                    .clone()
                    .expect("behavior tests always install an effect fixture bridge"),
            )
        });

        #[cfg(feature = "devtools")]
        let (_devtools_server, debug_state) = {
            let state = self.devtools.then(wabou_devtools::DebugState::shared);
            let server = if let Some(state) = &state {
                let path = wabou_devtools::socket_path();
                let server = wabou_devtools::serve(state.clone(), path.clone())
                    .context(crate::error::DevtoolsSnafu)?;
                tracing::info!(target: "devtools", socket = %path.display(), "Wabou DevTools listening");
                Some(server)
            } else {
                None
            };
            (server, state)
        };
        #[cfg(not(feature = "devtools"))]
        if self.devtools {
            tracing::warn!(
                "VelloHybridHostBuilder::devtools(true) requires the `wabou/devtools` feature"
            );
        }

        #[cfg(feature = "vite")]
        let source = if let Ok(url) = std::env::var("WABOU_VITE_URL") {
            ApplicationSource::Vite {
                url,
                entry: std::env::var("WABOU_VITE_ENTRY")
                    .unwrap_or_else(|_| "src/index.tsx".to_owned()),
            }
        } else {
            ApplicationSource::Bundle {
                code: crate::bundle::load()?,
                source_map: crate::bundle::load_source_map()?,
            }
        };
        #[cfg(not(feature = "vite"))]
        let source = ApplicationSource::Bundle {
            code: crate::bundle::load()?,
            source_map: crate::bundle::load_source_map()?,
        };
        let app_directories = self
            .app_directory_config
            .as_ref()
            .map(|config| {
                let resource = crate::bundle::resource_directory()?;
                vello_shell::AppDirectories::resolve(config, resource).ok_or_else(|| {
                    crate::Error::AppDirectories {
                        application: "configured application".to_owned(),
                    }
                })
            })
            .transpose()?;
        let mut capabilities = self.capabilities;
        if let Some(controller) = &test_controller {
            let controller = controller.clone();
            capabilities.push(Arc::new(move |js| {
                let controller = controller.clone();
                js.mount_capability("test", move |ctx, object| {
                    controller.mount_object(ctx, object)
                })
            }));
        }
        if self.kv_enabled {
            let directories = app_directories
                .as_ref()
                .ok_or(crate::Error::MissingArgument {
                    argument: "VelloHybridHostBuilder::app_directories before VelloHybridHostBuilder::kv",
                })?;
            let path = directories
                .storage_namespace("kv")
                .expect("framework KV namespace is valid")
                .join("default.sqlite3");
            capabilities.push(Arc::new(move |js| {
                let path = path.clone();
                js.mount_capability("kv", move |ctx, object| {
                    object.set("__wabouCapabilityVersion", 2_u16)?;
                    runtime_api::mount_kv_methods(
                        runtime_api::NativeCapability::from_runtime_parts(ctx, object),
                        path.clone(),
                    )
                })
            }));
        }
        let headless = test_controller.is_some()
            && std::env::var("WABOU_TEST_HEADLESS").is_ok_and(|value| value != "0");
        let service_context = runtime_api::HostServiceContext::for_alternate_host(
            app_directories.clone(),
            test_controller.is_some(),
            headless,
        );
        let services = start_services(&self.services, &service_context)?;

        let mut windows = std::iter::once(self.window)
            .chain(self.additional_windows)
            .collect::<Vec<_>>();
        let mut extensions = self.shell_extensions;
        if let Some(controller) = &test_controller {
            let window_keys = (0..windows.len())
                .map(vello_shell::initial_window_resource_key)
                .collect();
            extensions.push(Box::new(crate::behavior_test::WinitBehaviorTest::new(
                controller.clone(),
                window_keys,
            )));
        }
        if let (Some(key), Some(directories)) = (
            self.persisted_window_size.as_ref(),
            app_directories.as_ref(),
        ) {
            let path = directories
                .local_data_dir
                .join("window-state")
                .join(format!("{key}.json"));
            extensions.push(Box::new(vello_shell::WindowSizePersistence::restore(
                path,
                vello_shell::initial_window_resource_key(0),
                &mut windows[0],
            )));
        } else if let Some(key) = &self.persisted_window_size {
            tracing::warn!(key, "window size persistence requires app_directories");
        }
        let runtime_sources = RuntimeSourceConfig {
            source,
            js_runtime_options: self.js_runtime_options,
            capabilities,
            host_message_producers: self.host_message_producers,
            widget_factories: self.widget_factories,
            base_color: self.base_color,
            image_resources: self.image_resources,
            effect_trace: effect_trace.clone(),
            app_directories,
            #[cfg(feature = "devtools")]
            debug_state: debug_state.clone(),
            #[cfg(feature = "vite")]
            hmr_clients: Rc::new(RefCell::new(Vec::new())),
        };
        let mut sources = Vec::with_capacity(windows.len());
        for (index, options) in windows.into_iter().enumerate() {
            let controller = runtime_sources
                .create(vello_shell::initial_window_resource_key(index), &options)?;
            if index == 0
                && let Some(script) = &test_script
            {
                controller
                    .eval_script_diagnostic(script)
                    .map_err(|message| crate::Error::TestScenario {
                        message: format!("failed to evaluate Winit test scenario: {message}"),
                    })?;
            }
            sources.push((Box::new(controller) as Box<dyn FrameSource>, options));
        }
        let dynamic_sources = runtime_sources.clone();
        let factory: vello_shell::FrameSourceFactory = Rc::new(move |window_key, options| {
            dynamic_sources
                .create(window_key, options)
                .map(|controller| Box::new(controller) as Box<dyn FrameSource>)
                .map_err(|error| error.to_string())
        });
        if headless {
            crate::headless_test::run(
                test_controller
                    .as_ref()
                    .expect("headless test requires a controller"),
                &mut sources,
                self.base_color,
                #[cfg(feature = "devtools")]
                debug_state.as_ref(),
            )?;
        } else {
            vello_shell::run_windows_with_factory_and_extensions(
                sources,
                Some(factory),
                extensions,
            )
            .context(crate::error::WinitShellSnafu)?;
        }
        if recording_effects && let (Some(trace), Some(path)) = (&effect_trace, trace_path) {
            trace
                .write(&path)
                .map_err(|message| crate::Error::EffectTrace { message })?;
        }
        if let Some(controller) = test_controller {
            runtime_api::test_report::finish_test_report(controller).map_err(|error| {
                crate::Error::TestScenario {
                    message: error.to_string(),
                }
            })?;
        }
        services.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use wabou_bindgen::HostMethod;

    #[derive(Deserialize)]
    struct DoubleRequest {
        value: u32,
    }

    #[derive(Serialize)]
    struct DoubleResponse {
        value: u32,
    }

    #[test]
    fn typed_capabilities_mount_before_the_winit_bundle_boots() {
        const CONTRACT: CapabilityContract = CapabilityContract::new("winitTest", 3);
        const DOUBLE: HostMethod<DoubleRequest, DoubleResponse> = HostMethod::new("double");
        let builder = VelloHybridHostBuilder::new().capability(CONTRACT, |capability| {
            capability.sync_method(DOUBLE, |request: DoubleRequest| {
                Ok::<_, String>(DoubleResponse {
                    value: request.value * 2,
                })
            })
        });
        let runtime = JsRuntime::new().expect("runtime");
        for capability in &builder.capabilities {
            capability(&runtime).expect("mount capability");
        }
        let result = runtime
            .with(|ctx| {
                ctx.eval::<String, _>(
                    "JSON.stringify([__wabou_capabilities.winitTest.__wabouCapabilityVersion, __wabou_capabilities.winitTest.double({ value: 21 }).value])",
                )
            })
            .expect("invoke capability");
        assert_eq!(result, "[3,42]");
    }

    #[test]
    fn password_inputs_register_the_public_secure_editor_tag() {
        let secrets = crate::VelloHybridSecretStore::default();
        let builder = VelloHybridHostBuilder::new().password_inputs(secrets.clone());
        let mut widget = builder.widget_factories["password-input"]();

        assert!(
            widget
                .handle_event(&vello_shell::UiEvent::TextInput("secret".into()))
                .is_handled()
        );
        assert_eq!(secrets.take("default").as_str(), "secret");
    }

    #[test]
    fn runtime_source_factory_boots_dynamic_windows_with_shared_contracts() {
        const CONTRACT: CapabilityContract = CapabilityContract::new("winitTest", 3);
        const DOUBLE: HostMethod<DoubleRequest, DoubleResponse> = HostMethod::new("double");
        let builder = VelloHybridHostBuilder::new().capability(CONTRACT, |capability| {
            capability.sync_method(DOUBLE, |request: DoubleRequest| {
                Ok::<_, String>(DoubleResponse {
                    value: request.value * 2,
                })
            })
        });
        let source = RuntimeSourceConfig {
            source: ApplicationSource::Bundle {
                code: r#"
                    globalThis.bootResult = [
                      JSON.parse(__wabou_window_options_json).title,
                      __wabou_capabilities.winitTest.double({ value: 21 }).value,
                    ];
                "#
                .to_owned(),
                source_map: None,
            },
            js_runtime_options: builder.js_runtime_options,
            capabilities: builder.capabilities,
            host_message_producers: builder.host_message_producers,
            widget_factories: builder.widget_factories,
            base_color: builder.base_color,
            image_resources: builder.image_resources,
            effect_trace: None,
            app_directories: None,
            #[cfg(feature = "devtools")]
            debug_state: None,
            #[cfg(feature = "vite")]
            hmr_clients: Rc::new(RefCell::new(Vec::new())),
        };
        let child = source
            .create(
                vello_shell::WindowResourceKey::from_parts(9, 1).unwrap(),
                &WindowOptions::new().title("Dynamic child"),
            )
            .expect("create dynamic window runtime");

        assert_eq!(
            child
                .eval_string("JSON.stringify(globalThis.bootResult)")
                .unwrap(),
            r#"["Dynamic child",42]"#
        );
    }

    #[test]
    fn public_host_message_router_attaches_to_winit_runtime() {
        let router = runtime_api::HostMessageRouter::new();
        let builder = VelloHybridHostBuilder::new().host_message_router(router.clone());
        let controller = Applier::from_runtime(JsRuntime::new().expect("runtime"), Color::BLACK);
        let window_key = vello_shell::initial_window_resource_key(0);
        let context = controller.host_message_context(window_key);
        for producer in &builder.host_message_producers {
            producer(context.clone());
        }

        router
            .send_to(window_key, runtime_api::HostMessage::str("ready", "winit"))
            .expect("public router sends through Winit queue");
    }

    struct TestService {
        starts: Arc<AtomicUsize>,
        stops: Arc<AtomicUsize>,
        fail: bool,
    }

    impl runtime_api::HostService for TestService {
        fn name(&self) -> &'static str {
            "winit-test"
        }

        fn start(&self, _context: &runtime_api::HostServiceContext) -> Result<(), String> {
            self.starts.fetch_add(1, Ordering::Relaxed);
            if self.fail {
                Err("expected failure".to_owned())
            } else {
                Ok(())
            }
        }

        fn shutdown(&self) -> Result<(), String> {
            self.stops.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }

    #[test]
    fn services_surround_the_event_loop_and_recoverable_failures_do_not_stop_startup() {
        let starts = Arc::new(AtomicUsize::new(0));
        let stops = Arc::new(AtomicUsize::new(0));
        let services: Vec<(Arc<dyn runtime_api::HostService>, bool)> = vec![
            (
                Arc::new(TestService {
                    starts: starts.clone(),
                    stops: stops.clone(),
                    fail: true,
                }),
                false,
            ),
            (
                Arc::new(TestService {
                    starts: starts.clone(),
                    stops: stops.clone(),
                    fail: false,
                }),
                true,
            ),
        ];
        let context = runtime_api::HostServiceContext::for_alternate_host(None, false, false);
        let guard = start_services(&services, &context).expect("start services");
        assert_eq!(starts.load(Ordering::Relaxed), 2);
        guard.finish().expect("stop services");
        assert_eq!(stops.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn required_service_failure_stops_services_that_started_before_it() {
        let starts = Arc::new(AtomicUsize::new(0));
        let stops = Arc::new(AtomicUsize::new(0));
        let services: Vec<(Arc<dyn runtime_api::HostService>, bool)> = vec![
            (
                Arc::new(TestService {
                    starts: starts.clone(),
                    stops: stops.clone(),
                    fail: false,
                }),
                true,
            ),
            (
                Arc::new(TestService {
                    starts: starts.clone(),
                    stops: stops.clone(),
                    fail: true,
                }),
                true,
            ),
        ];
        let context = runtime_api::HostServiceContext::for_alternate_host(None, false, false);
        assert!(start_services(&services, &context).is_err());
        assert_eq!(starts.load(Ordering::Relaxed), 2);
        assert_eq!(stops.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn shared_kv_contract_runs_inside_the_winit_quickjs_runtime() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let runtime = JsRuntime::new().expect("runtime");
        let path = directory.path().join("winit-kv.sqlite3");
        runtime
            .mount_capability("kv", move |ctx, object| {
                object.set("__wabouCapabilityVersion", 2_u16)?;
                runtime_api::mount_kv_methods(
                    runtime_api::NativeCapability::from_runtime_parts(ctx, object),
                    path.clone(),
                )
            })
            .expect("mount KV capability");
        let result = runtime
            .eval_promise_json(
                r#"(() => {
                  const key = [{ type: "string", value: "backend" }];
                  return __wabou_capabilities.kv
                    .set({ key, value: { name: "vello-hybrid" } })
                    .then(() => __wabou_capabilities.kv.get({ key }));
                })()"#,
                Duration::from_secs(2),
            )
            .expect("KV promise settled");
        let result: serde_json::Value = serde_json::from_str(&result).expect("KV response");
        assert_eq!(result["value"]["name"], "vello-hybrid");
    }
}
