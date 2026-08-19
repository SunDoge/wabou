//! HostBuilder — Tauri-like entry point for wabou apps.
//!
//! Users register custom widgets (Rust `Widget` impls keyed by tag) +
//! named capabilities (Rust fns callable from JS), then `.run()`. Built-in widgets
//! (Canvas) are pre-registered; users override or add their own.
//!
//! ```ignore
//! use serde::{Deserialize, Serialize};
//! use wabou_bindgen::{JsonCapabilityContract, JsonMethod};
//! use wabou_runtime::{HostBuilder, Widget};
//!
//! #[derive(Deserialize)]
//! struct ReadFileRequest {
//!     path: String,
//! }
//!
//! #[derive(Serialize)]
//! struct ReadFileResponse {
//!     contents: String,
//! }
//!
//! const WORKSPACE: JsonCapabilityContract = JsonCapabilityContract::new("workspace", 1);
//!
//! HostBuilder::new()
//!     .widget("chart", || Box::new(MyChart::new()))
//!     .json_capability(WORKSPACE, |capability| {
//!         capability.method(JsonMethod::new("readFile"), |request: ReadFileRequest| async move {
//!             let contents = std::fs::read_to_string(request.path)
//!                 .map_err(|error| error.to_string())?;
//!             Ok::<_, String>(ReadFileResponse { contents })
//!         })
//!     })
//!     .run()
//!     .unwrap();
//! ```

use snafu::ResultExt;
use std::collections::HashMap;
use std::fmt::Display;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use vello::peniko::Color;

use serde::Serialize;
use serde::de::DeserializeOwned;
use wabou_bindgen::{JsonCapabilityContract, JsonCapabilityErrorCode, JsonMethod};

use crate::HostMessageContext;
use crate::applier::Applier;
use crate::asset_cache::ResourceCache;
use crate::jsrt::JsRuntime;
use crate::{ShellExtension, WindowOptions, run_windows_with_factory_and_extensions, style};
use wabou_shell::{Widget, WidgetFactory};
use wabou_widgets::{SecretStore, builtin_factories, password_input_factory};

type CapabilityInstaller = Arc<dyn Fn(&JsRuntime) -> rquickjs::Result<()>>;
type HostMessageProducer = Arc<dyn Fn(HostMessageContext) + Send + Sync>;
type WindowSource = (Box<dyn crate::FrameSource>, WindowOptions);

/// A namespaced host capability that exposes structured asynchronous methods.
///
/// Generated Wabou clients JSON-encode their single request value. This adapter
/// owns the matching decode, Promise installation, and result envelope so
/// applications do not duplicate transport glue around ordinary Rust async
/// functions.
pub struct JsonCapability<'js> {
    ctx: rquickjs::Ctx<'js>,
    object: rquickjs::Object<'js>,
}

impl<'js> JsonCapability<'js> {
    /// Install one async method accepting a JSON-encoded request and returning
    /// Wabou's `{ ok, value | error }` envelope.
    pub fn method<Request, Response, Error, Handler, HandlerFuture>(
        &self,
        method: JsonMethod<Request, Response>,
        handler: Handler,
    ) -> rquickjs::Result<()>
    where
        Request: DeserializeOwned + 'static,
        Response: Serialize + 'static,
        Error: Display + 'static,
        Handler: Fn(Request) -> HandlerFuture + Clone + rquickjs::markers::ParallelSend + 'static,
        HandlerFuture: Future<Output = Result<Response, Error>> + 'static,
    {
        if !wabou_bindgen::is_contract_identifier(method.name()) {
            return Err(rquickjs::Exception::throw_type(
                &self.ctx,
                &format!(
                    "invalid JSON capability method identifier `{}`",
                    method.name()
                ),
            ));
        }
        if self.object.contains_key(method.name())? {
            return Err(rquickjs::Exception::throw_type(
                &self.ctx,
                &format!("duplicate JSON capability method `{}`", method.name()),
            ));
        }
        if method.has_request() {
            let function = rquickjs::Function::new(
                self.ctx.clone(),
                rquickjs::prelude::Async(move |raw: String| {
                    let handler = handler.clone();
                    async move { invoke_json_method(&raw, handler).await }
                }),
            )?;
            self.object.set(method.name(), function)
        } else {
            let function = rquickjs::Function::new(
                self.ctx.clone(),
                rquickjs::prelude::Async(move || {
                    let handler = handler.clone();
                    async move { invoke_json_method("null", handler).await }
                }),
            )?;
            self.object.set(method.name(), function)
        }
    }
}

async fn invoke_json_method<Request, Response, Error, Handler, HandlerFuture>(
    raw: &str,
    handler: Handler,
) -> String
where
    Request: DeserializeOwned,
    Response: Serialize,
    Error: Display,
    Handler: Fn(Request) -> HandlerFuture,
    HandlerFuture: Future<Output = Result<Response, Error>>,
{
    let request = match serde_json::from_str(raw) {
        Ok(request) => request,
        Err(error) => {
            return json_capability_error(
                JsonCapabilityErrorCode::InvalidRequest,
                format!("invalid capability request: {error}"),
            );
        }
    };
    match handler(request).await {
        Ok(value) => match serde_json::to_value(value) {
            Ok(value) => serde_json::json!({ "ok": true, "value": value }).to_string(),
            Err(error) => json_capability_error(
                JsonCapabilityErrorCode::ResponseEncodingFailure,
                format!("cannot encode capability response: {error}"),
            ),
        },
        Err(error) => {
            json_capability_error(JsonCapabilityErrorCode::HandlerFailure, error.to_string())
        }
    }
}

fn json_capability_error(code: JsonCapabilityErrorCode, message: String) -> String {
    serde_json::json!({
        "ok": false,
        "error": {
            "code": code.as_str(),
            "message": message,
        },
    })
    .to_string()
}

struct ResourceCacheShutdownGuard {
    cache: Arc<ResourceCache>,
}

impl ResourceCacheShutdownGuard {
    fn new(cache: Arc<ResourceCache>) -> Self {
        Self { cache }
    }
}

impl Drop for ResourceCacheShutdownGuard {
    fn drop(&mut self) {
        if let Err(error) = self.cache.shutdown() {
            tracing::warn!(%error, "failed to gracefully close persistent asset cache");
        }
    }
}

#[cfg(feature = "profiling")]
fn init_tracing() -> Option<tracing_chrome::FlushGuard> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let Some(path) = std::env::var_os("WABOU_PROFILE_TRACE") else {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .try_init()
            .ok();
        return None;
    };
    let file = match std::fs::File::create(&path) {
        Ok(file) => file,
        Err(error) => {
            eprintln!(
                "failed to create Wabou profile trace {}: {error}",
                Path::new(&path).display()
            );
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .try_init()
                .ok();
            return None;
        }
    };
    init_profile_tracing(file, env_filter)
}

#[cfg(feature = "profiling")]
fn init_profile_tracing(
    file: std::fs::File,
    env_filter: tracing_subscriber::EnvFilter,
) -> Option<tracing_chrome::FlushGuard> {
    use tracing_subscriber::filter::filter_fn;
    use tracing_subscriber::prelude::*;

    let (chrome, guard) = tracing_chrome::ChromeLayerBuilder::new()
        .writer(file)
        .include_args(true)
        .include_locations(false)
        .build();
    let initialized = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(env_filter))
        .with(chrome.with_filter(filter_fn(|metadata| metadata.target() == "wabou::perf")))
        .try_init()
        .is_ok();
    initialized.then_some(guard)
}

enum EffectTraceConfig {
    Record { path: PathBuf, record_all: bool },
    Replay { path: PathBuf },
}

/// Application-facing builder for windows, widgets, capabilities, and tooling.
pub struct HostBuilder {
    base_color: Color,
    window: WindowOptions,
    additional_windows: Vec<WindowOptions>,
    widget_factories: HashMap<String, WidgetFactory>,
    capabilities: Vec<CapabilityInstaller>,
    host_message_producers: Vec<HostMessageProducer>,
    devtools: bool,
    extensions: Vec<Box<dyn ShellExtension>>,
    effect_trace: Option<EffectTraceConfig>,
    app_directory_config: Option<wabou_shell::AppDirectoryConfig>,
}

impl Default for HostBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HostBuilder {
    /// Create a builder with built-in widgets pre-registered (`"canvas"` →
    /// `Canvas`). In development the CLI supplies a Vite URL; packaged apps
    /// load `resources/bundle.js` next to the executable.
    pub fn new() -> Self {
        Self {
            base_color: style::parse_color("#0f172a")
                .unwrap_or_else(|| Color::from_rgb8(0x0f, 0x17, 0x2a)),
            window: WindowOptions::default(),
            additional_windows: Vec::new(),
            widget_factories: builtin_factories(),
            capabilities: Vec::new(),
            host_message_producers: Vec::new(),
            devtools: cfg!(debug_assertions),
            extensions: Vec::new(),
            effect_trace: None,
            app_directory_config: None,
        }
    }

    /// Register or override a widget factory for `tag`. When the SolidJS app
    /// creates an element with this tag (`<chart />`), the factory is called
    /// to produce a fresh `Box<dyn Widget>`.
    pub fn widget(
        mut self,
        tag: impl Into<String>,
        factory: impl Fn() -> Box<dyn Widget> + 'static,
    ) -> Self {
        self.widget_factories.insert(tag.into(), Arc::new(factory));
        self
    }

    /// Register the framework password widget backed by a Rust-only secret store.
    pub fn password_inputs(mut self, secrets: SecretStore) -> Self {
        self.widget_factories
            .insert("password-input".into(), password_input_factory(secrets));
        self
    }

    /// Mount a namespaced capability object before the app bundle boots.
    /// The closure may install synchronous functions or `rquickjs::Async`
    /// functions, which QuickJS exposes as native Promises.
    pub fn capability<F>(mut self, name: impl Into<String>, mount: F) -> Self
    where
        F: for<'js> Fn(rquickjs::Ctx<'js>, rquickjs::Object<'js>) -> rquickjs::Result<()>
            + rquickjs::markers::ParallelSend
            + Send
            + Sync
            + 'static,
    {
        let name = name.into();
        let mount = Arc::new(mount);
        self.capabilities.push(Arc::new(move |js| {
            let mount = mount.clone();
            js.mount_capability(&name, move |ctx, capability| mount(ctx, capability))
        }));
        self
    }

    /// Mount structured async methods without exposing QuickJS transport
    /// details to application code.
    pub fn json_capability<F>(self, contract: JsonCapabilityContract, mount: F) -> Self
    where
        F: for<'js> Fn(JsonCapability<'js>) -> rquickjs::Result<()>
            + rquickjs::markers::ParallelSend
            + Send
            + Sync
            + 'static,
    {
        self.capability(contract.name(), move |ctx, object| {
            object.set("__wabouCapabilityVersion", contract.version())?;
            mount(JsonCapability { ctx, object })
        })
    }

    /// Register a producer for application-level Rust → JavaScript messages.
    ///
    /// The callback runs once for every native window after that window's
    /// bounded message queue is created and before its JavaScript bundle boots.
    /// Background tasks may retain the cloneable context and emit from any
    /// thread. JavaScript receives values through `hostMessages.subscribe()`.
    /// The context is cancelled when its window is dropped.
    ///
    /// Producers should use `context.window_id()` to avoid duplicate global
    /// streams when an application creates additional windows.
    pub fn host_message_producer<F>(mut self, producer: F) -> Self
    where
        F: Fn(HostMessageContext) + Send + Sync + 'static,
    {
        self.host_message_producers.push(Arc::new(producer));
        self
    }

    /// Configure the primary native window.
    pub fn window(mut self, options: WindowOptions) -> Self {
        self.window = options;
        self
    }

    /// Create another independent native window running the same application
    /// bundle, widget registry and host capabilities.
    pub fn additional_window(mut self, options: WindowOptions) -> Self {
        self.additional_windows.push(options);
        self
    }

    /// Set the viewport clear color behind the retained root.
    pub fn base_color(mut self, color: Color) -> Self {
        self.base_color = color;
        self
    }

    /// Configure the stable identity used by app-private storage directories.
    pub fn app_directories(
        self,
        qualifier: impl Into<String>,
        organization: impl Into<String>,
        application: impl Into<String>,
    ) -> Self {
        self.app_directory_config(wabou_shell::AppDirectoryConfig::new(
            qualifier,
            organization,
            application,
        ))
    }

    /// Set an already constructed stable application directory identity.
    pub fn app_directory_config(mut self, config: wabou_shell::AppDirectoryConfig) -> Self {
        self.app_directory_config = Some(config);
        self
    }

    /// Enable or disable the local, read-only DevTools socket. It defaults to
    /// enabled in debug builds and is absent from release builds unless opted in.
    pub fn devtools(mut self, enabled: bool) -> Self {
        self.devtools = enabled;
        self
    }

    /// Install a native integration that shares Wabou's platform event loop.
    pub fn extension(mut self, extension: impl ShellExtension + 'static) -> Self {
        self.extensions.push(Box::new(extension));
        self
    }

    /// Record replay-safe native effects to `path` when the application exits.
    /// Clipboard and third-party payloads are excluded because they may contain secrets.
    pub fn record_effects(mut self, path: impl Into<PathBuf>) -> Self {
        self.effect_trace = Some(EffectTraceConfig::Record {
            path: path.into(),
            record_all: false,
        });
        self
    }

    /// Record every native effect. This can persist clipboard or extension secrets.
    pub fn record_all_effects(mut self, path: impl Into<PathBuf>) -> Self {
        self.effect_trace = Some(EffectTraceConfig::Record {
            path: path.into(),
            record_all: true,
        });
        self
    }

    /// Replay recorded effects without invoking their native implementations.
    /// Operations absent from the tape continue to execute live.
    pub fn replay_effects(mut self, path: impl Into<PathBuf>) -> Self {
        self.effect_trace = Some(EffectTraceConfig::Replay { path: path.into() });
        self
    }

    /// Build the JsRuntime + Applier + run the winit event loop.
    pub fn run(mut self) -> crate::Result<()> {
        let test_script = std::env::var_os("WABOU_TEST_SCRIPT")
            .map(PathBuf::from)
            .map(|path| {
                std::fs::read_to_string(&path).context(crate::error::ReadFileSnafu {
                    kind: "test scenario bundle",
                    path,
                })
            })
            .transpose()?;
        let test_controller = test_script
            .as_ref()
            .map(|_| crate::test_driver::TestController::default());
        let headless_test = test_controller.is_some()
            && std::env::var("WABOU_TEST_HEADLESS").is_ok_and(|value| value != "0");
        if let Some(controller) = &test_controller {
            let capability_controller = controller.clone();
            self.capabilities
                .push(Arc::new(move |js| capability_controller.mount(js)));
            if !headless_test {
                self.extensions
                    .push(Box::new(crate::test_driver::TestDriver::new(
                        controller.clone(),
                    )));
            }
        }
        let trace_path = self.effect_trace.as_ref().map(|config| match config {
            EffectTraceConfig::Record { path, .. } | EffectTraceConfig::Replay { path } => {
                path.clone()
            }
        });
        let effect_trace = match &self.effect_trace {
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
        let app_directories = self
            .app_directory_config
            .as_ref()
            .map(|config| {
                let resource = resource_directory()?;
                wabou_shell::AppDirectories::resolve(config, resource).ok_or_else(|| {
                    crate::Error::AppDirectories {
                        application: "configured application".into(),
                    }
                })
            })
            .transpose()?;
        let windows = std::iter::once(self.window.clone())
            .chain(self.additional_windows.iter().cloned())
            .collect::<Vec<_>>();
        #[cfg(feature = "profiling")]
        let _profile_guard = init_tracing();
        #[cfg(not(feature = "profiling"))]
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .try_init()
            .ok();

        let asset_cache = Arc::new(if let Some(directories) = &app_directories {
            ResourceCache::with_disk(&directories.cache_dir).unwrap_or_else(|error| {
                tracing::warn!(%error, "failed to enable persistent asset cache");
                ResourceCache::memory_only()
            })
        } else {
            ResourceCache::memory_only()
        });
        // Declared immediately after the cache so it runs on successful exit
        // and on every later `?` path, before the cache-owned runtime is dropped.
        let _asset_cache_shutdown = ResourceCacheShutdownGuard::new(asset_cache.clone());

        #[cfg(feature = "vite")]
        let vite = std::env::var("WABOU_VITE_URL").ok().map(|url| {
            let entry =
                std::env::var("WABOU_VITE_ENTRY").unwrap_or_else(|_| "src/index.tsx".to_string());
            (url, entry)
        });

        let mut devtools_server = None;
        let debug_state = self.devtools.then(wabou_devtools::DebugState::shared);
        if self.devtools {
            let path = wabou_devtools::socket_path();
            devtools_server = Some(
                wabou_devtools::serve(debug_state.as_ref().unwrap().clone(), path.clone())
                    .context(crate::error::DevtoolsSnafu)?,
            );
            tracing::info!(target: "devtools", socket = %path.display(), "Wabou DevTools listening");
        }

        #[cfg(not(feature = "vite"))]
        let bundle = load_bundle()?;
        #[cfg(feature = "vite")]
        let bundle = vite.is_none().then(load_bundle).transpose()?;
        #[cfg(not(feature = "vite"))]
        let bundle_source_map = load_bundle_source_map()?;
        #[cfg(feature = "vite")]
        let bundle_source_map = if vite.is_none() {
            load_bundle_source_map()?
        } else {
            None
        };
        #[cfg(feature = "vite")]
        let mut hmr_clients = Vec::new();
        let mut sources = Vec::with_capacity(windows.len());
        for (index, options) in windows.into_iter().enumerate() {
            #[cfg(feature = "vite")]
            let js = if let Some((url, _)) = vite.as_ref() {
                JsRuntime::new_vite(url).context(crate::error::JavaScriptSnafu {
                    operation: "create Vite JavaScript runtime",
                })?
            } else {
                JsRuntime::new().context(crate::error::JavaScriptSnafu {
                    operation: "create JavaScript runtime",
                })?
            };
            #[cfg(not(feature = "vite"))]
            let js = JsRuntime::new().context(crate::error::JavaScriptSnafu {
                operation: "create JavaScript runtime",
            })?;
            for capability in &self.capabilities {
                capability(&js).context(crate::error::JavaScriptSnafu {
                    operation: "mount JavaScript capability",
                })?;
            }
            let mut applier = Applier::from_runtime_with_factories_and_window(
                js,
                self.widget_factories.clone(),
                self.base_color,
                index as u64 + 1,
            );
            install_host_message_producers(
                &self.host_message_producers,
                index as u64 + 1,
                &applier,
            );
            applier.set_asset_cache(asset_cache.clone());
            if let Some(directories) = &app_directories {
                applier.set_app_directories(directories.clone());
            }
            if let Some(trace) = &effect_trace {
                applier.set_effect_trace(trace.clone());
            }
            #[cfg(feature = "vite")]
            if let Some((_, entry)) = vite.as_ref() {
                applier
                    .boot_vite(entry)
                    .context(crate::error::JavaScriptSnafu {
                        operation: "boot Vite entry module",
                    })?;
            } else {
                applier
                    .boot_with_source_map(bundle.as_deref().unwrap(), bundle_source_map.as_deref())
                    .context(crate::error::JavaScriptSnafu {
                        operation: "boot JavaScript bundle",
                    })?;
            }
            #[cfg(not(feature = "vite"))]
            applier
                .boot_with_source_map(&bundle, bundle_source_map.as_deref())
                .context(crate::error::JavaScriptSnafu {
                    operation: "boot JavaScript bundle",
                })?;
            if index == 0
                && let Some(script) = &test_script
            {
                applier
                    .eval_script(script)
                    .context(crate::error::JavaScriptSnafu {
                        operation: "evaluate test scenario",
                    })?;
            }
            if let Some(state) = &debug_state {
                applier.set_debug_state(state.clone());
            }
            #[cfg(feature = "vite")]
            if let Some((url, entry)) = vite.as_ref() {
                applier.set_vite_entry(entry);
                hmr_clients.push(
                    crate::start_hmr_client(url, applier.reload_handle())
                        .context(crate::error::ViteSnafu)?,
                );
            }
            sources.push((Box::new(applier) as Box<dyn crate::FrameSource>, options));
        }

        if headless_test && let Some(controller) = &test_controller {
            return run_headless_test(controller, &mut sources, self.base_color);
        }

        let capabilities = self.capabilities.clone();
        let host_message_producers = self.host_message_producers.clone();
        let widget_factories = self.widget_factories.clone();
        let base_color = self.base_color;
        let child_debug_state = debug_state.clone();
        let child_effect_trace = effect_trace.clone();
        let child_app_directories = app_directories.clone();
        let child_asset_cache = asset_cache.clone();
        #[cfg(feature = "vite")]
        let child_vite = vite.clone();
        #[cfg(feature = "vite")]
        let child_bundle = bundle.clone();
        #[cfg(not(feature = "vite"))]
        let child_bundle = bundle.clone();
        #[cfg(feature = "vite")]
        let child_hmr_clients = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        #[cfg(feature = "vite")]
        let child_hmr_store = child_hmr_clients.clone();
        #[allow(clippy::arc_with_non_send_sync)] // winit invokes this only on its event thread.
        let factory: crate::FrameSourceFactory = Arc::new(move |window_id, _options| {
            #[cfg(feature = "vite")]
            let js = if let Some((url, _)) = child_vite.as_ref() {
                JsRuntime::new_vite(url).map_err(|error| format!("{error:?}"))?
            } else {
                JsRuntime::new().map_err(|error| format!("{error:?}"))?
            };
            #[cfg(not(feature = "vite"))]
            let js = JsRuntime::new().map_err(|error| format!("{error:?}"))?;
            for capability in &capabilities {
                capability(&js).map_err(|error| format!("{error:?}"))?;
            }
            let mut applier = Applier::from_runtime_with_factories_and_window(
                js,
                widget_factories.clone(),
                base_color,
                window_id,
            );
            install_host_message_producers(&host_message_producers, window_id, &applier);
            applier.set_asset_cache(child_asset_cache.clone());
            if let Some(directories) = &child_app_directories {
                applier.set_app_directories(directories.clone());
            }
            if let Some(trace) = &child_effect_trace {
                applier.set_effect_trace(trace.clone());
            }
            #[cfg(feature = "vite")]
            if let Some((_, entry)) = child_vite.as_ref() {
                applier
                    .boot_vite(entry)
                    .map_err(|error| format!("{error:?}"))?;
            } else {
                applier
                    .boot(child_bundle.as_deref().unwrap())
                    .map_err(|error| format!("{error:?}"))?;
            }
            #[cfg(not(feature = "vite"))]
            applier
                .boot(&child_bundle)
                .map_err(|error| format!("{error:?}"))?;
            if let Some(state) = &child_debug_state {
                applier.set_debug_state(state.clone());
            }
            #[cfg(feature = "vite")]
            if let Some((url, entry)) = child_vite.as_ref() {
                applier.set_vite_entry(entry);
                let client = crate::start_hmr_client(url, applier.reload_handle())
                    .map_err(|error| error.to_string())?;
                child_hmr_store.lock().unwrap().push(client);
            }
            Ok(Box::new(applier))
        });

        let run_result =
            run_windows_with_factory_and_extensions(sources, Some(factory), self.extensions)
                .context(crate::error::ShellSnafu);
        let trace_result =
            if recording_effects && let (Some(trace), Some(path)) = (&effect_trace, trace_path) {
                trace
                    .write(&path)
                    .map_err(|message| crate::Error::EffectTrace { message })
            } else {
                Ok(())
            };
        run_result?;
        trace_result?;
        if let Some(controller) = test_controller {
            finish_test_report(controller)?;
        }
        #[cfg(feature = "vite")]
        drop(hmr_clients);
        #[cfg(feature = "vite")]
        drop(child_hmr_clients);
        drop(devtools_server);
        Ok(())
    }
}

fn install_host_message_producers(
    producers: &[HostMessageProducer],
    window_id: u64,
    applier: &Applier,
) {
    for producer in producers {
        producer(applier.host_message_context(window_id));
    }
}

fn run_headless_test(
    controller: &crate::test_driver::TestController,
    sources: &mut [WindowSource],
    base_color: Color,
) -> crate::Result<()> {
    const WIDTH: u32 = 1100;
    const HEIGHT: u32 = 720;

    controller.initialize_headless(1..=sources.len() as u64);
    // JavaScript reports individual test timeouts with the test name. This is
    // only a final safety net for a broken runtime or runner that cannot
    // produce a report at all.
    // The longest JavaScript test/replay budget is 60 seconds. Keep the host
    // deadline later so JavaScript can serialize its named failure first.
    let deadline = Instant::now() + Duration::from_secs(65);
    let mut text = crate::TextContext::new();
    let mut last_nodes = vec![Vec::new(); sources.len()];
    while !controller.has_report() && Instant::now() < deadline {
        for (index, (source, _)) in sources.iter_mut().enumerate() {
            let window_id = index as u64 + 1;
            source.set_semantics_enabled(true);
            source.handle_event(wabou_shell::UiEvent::WindowMetrics(crate::WindowMetrics {
                window_id,
                logical_width: WIDTH,
                logical_height: HEIGHT,
                physical_width: WIDTH,
                physical_height: HEIGHT,
                scale_factor: 1.0,
                maximized: false,
                focused: true,
            }));
            last_nodes[index] = source.build_frame(&mut text, WIDTH, HEIGHT);
            controller.poll_headless_source(window_id, source.as_mut());
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    if controller.report_passed() == Some(false) {
        render_headless_failure(&last_nodes, &mut text, base_color)?;
    }
    finish_test_report(controller.clone())
}

fn render_headless_failure(
    last_nodes: &[Vec<wabou_shell::layout::PlacedNode>],
    text: &mut crate::TextContext,
    base_color: Color,
) -> crate::Result<()> {
    if !std::env::var("WABOU_TEST_FAILURE_SCREENSHOT").is_ok_and(|value| value != "0") {
        return Ok(());
    }
    let Some(directory) = std::env::var_os("WABOU_TEST_ARTIFACT_DIR").map(PathBuf::from) else {
        return Ok(());
    };
    let Some(nodes) = last_nodes.first() else {
        return Ok(());
    };
    std::fs::create_dir_all(&directory).map_err(|error| crate::Error::TestScenario {
        message: format!("cannot create failure artifact directory: {error}"),
    })?;
    let mut scene = vello::Scene::new();
    wabou_shell::scene::build_scene_scaled(&mut scene, nodes, text, 1100, 720, base_color, 1.0);
    let output = directory.join("failure.png");
    wabou_shell::renderer::render_to_png(
        &scene,
        1100,
        720,
        base_color,
        output.to_string_lossy().as_ref(),
    )
    .map_err(|error| crate::Error::TestScenario {
        message: format!("cannot render failure screenshot: {error:?}"),
    })
}

fn finish_test_report(controller: crate::test_driver::TestController) -> crate::Result<()> {
    let report = controller
        .take_report()
        .ok_or_else(|| crate::Error::TestScenario {
            message: "host exited or timed out before the scenario reported a result".into(),
        })?;
    let mut value = serde_json::from_str::<serde_json::Value>(&report).map_err(|error| {
        crate::Error::TestScenario {
            message: format!("scenario returned invalid JSON: {error}"),
        }
    })?;
    let version = value
        .get("version")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| crate::Error::TestScenario {
            message: "scenario returned an unversioned test report".into(),
        })?;
    if version != 1 {
        return Err(crate::Error::TestScenario {
            message: format!("unsupported test report version {version}; expected 1"),
        });
    }
    let headless = std::env::var("WABOU_TEST_HEADLESS").is_ok_and(|value| value != "0");
    value
        .as_object_mut()
        .ok_or_else(|| crate::Error::TestScenario {
            message: "scenario returned a non-object test report".into(),
        })?
        .insert("environment".into(), test_environment(headless));
    let passed = value
        .get("passed")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let artifact_directory = std::env::var_os("WABOU_TEST_ARTIFACT_DIR").map(PathBuf::from);
    if let Some(directory) = artifact_directory.as_deref() {
        std::fs::create_dir_all(directory).map_err(|error| crate::Error::TestScenario {
            message: format!(
                "cannot create artifact directory {}: {error}",
                directory.display()
            ),
        })?;
        write_test_artifact(directory, "report.json", &format!("{value:#}\n"))?;
        if let Some(trace) = value.get("trace") {
            let artifact = test_trace_artifact(trace);
            write_test_artifact(directory, "trace.json", &format!("{artifact:#}\n"))?;
        }
        if !passed {
            let semantics = controller.semantic_artifact();
            write_test_artifact(directory, "semantics.json", &format!("{semantics:#}\n"))?;
        }
    }
    let summary = test_report_summary(&value, artifact_directory.as_deref());
    if !passed {
        return Err(crate::Error::TestScenario { message: summary });
    }
    println!("{summary}");
    Ok(())
}

fn write_test_artifact(directory: &Path, name: &str, contents: &str) -> crate::Result<()> {
    let destination = directory.join(name);
    let temporary = directory.join(format!("{name}.tmp"));
    std::fs::write(&temporary, contents).map_err(|error| crate::Error::TestScenario {
        message: format!(
            "cannot write temporary test artifact {}: {error}",
            temporary.display()
        ),
    })?;
    if let Err(error) = std::fs::rename(&temporary, &destination) {
        let _ = std::fs::remove_file(&temporary);
        return Err(crate::Error::TestScenario {
            message: format!(
                "cannot publish test artifact {}: {error}",
                destination.display()
            ),
        });
    }
    Ok(())
}

fn test_trace_artifact(trace: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "version": 1,
        "actions": trace,
    })
}

fn test_environment(headless: bool) -> serde_json::Value {
    serde_json::json!({
        "backend": if headless { "deterministic" } else { "native" },
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "wabouVersion": env!("CARGO_PKG_VERSION"),
    })
}

fn test_report_summary(value: &serde_json::Value, artifact_directory: Option<&Path>) -> String {
    let tests = value
        .get("tests")
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let passed = tests
        .iter()
        .filter(|test| test.get("passed").and_then(serde_json::Value::as_bool) == Some(true))
        .count();
    let failed = tests.len().saturating_sub(passed);
    let actions = value
        .get("trace")
        .and_then(serde_json::Value::as_array)
        .map_or(0, Vec::len);
    let duration_ms = tests
        .iter()
        .filter_map(|test| test.get("durationMs").and_then(serde_json::Value::as_f64))
        .sum::<f64>();
    let mut summary = format!(
        "test result: {}. {passed} passed; {failed} failed; {actions} actions; {duration_ms:.1}ms",
        if failed == 0 { "ok" } else { "FAILED" }
    );
    for test in tests
        .iter()
        .filter(|test| test.get("passed").and_then(serde_json::Value::as_bool) != Some(true))
    {
        let name = test
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("test");
        summary.push_str(&format!("\n\n---- {name} ----"));
        if let Some(error) = test.get("error").and_then(serde_json::Value::as_str) {
            summary.push('\n');
            summary.push_str(error);
        }
    }
    if failed > 0
        && let Some(environment) = value.get("environment")
    {
        let backend = environment
            .get("backend")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let os = environment
            .get("os")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let arch = environment
            .get("arch")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let version = environment
            .get("wabouVersion")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        summary.push_str(&format!(
            "\nenvironment: {backend}; {os}/{arch}; wabou {version}"
        ));
    }
    if let Some(directory) = artifact_directory {
        summary.push_str(&format!("\nartifacts: {}", directory.display()));
    }
    summary
}

fn load_bundle() -> crate::Result<String> {
    let path = bundle_path()?;
    std::fs::read_to_string(&path).context(crate::error::ReadFileSnafu {
        kind: "JavaScript bundle",
        path,
    })
}

fn load_bundle_source_map() -> crate::Result<Option<Vec<u8>>> {
    let path = bundle_path()?.with_extension("js.map");
    if !path.is_file() {
        return Ok(None);
    }
    std::fs::read(&path)
        .map(Some)
        .context(crate::error::ReadFileSnafu {
            kind: "JavaScript source map",
            path,
        })
}

fn bundle_path() -> crate::Result<PathBuf> {
    if let Some(path) = std::env::var_os("WABOU_BUNDLE_PATH") {
        return Ok(PathBuf::from(path));
    }
    let executable = std::env::current_exe().context(crate::error::ReadFileSnafu {
        kind: "current executable path",
        path: PathBuf::from("<current executable>"),
    })?;
    Ok(resource_bundle_candidates(&executable)
        .into_iter()
        .find(|path| path.is_file())
        .unwrap_or_else(|| resource_bundle_path(&executable)))
}

fn resource_directory() -> crate::Result<PathBuf> {
    let bundle = bundle_path()?;
    Ok(bundle.parent().unwrap_or_else(|| Path::new(".")).to_owned())
}

fn resource_bundle_path(executable: &Path) -> PathBuf {
    executable
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("resources/bundle.js")
}

fn resource_bundle_candidates(executable: &Path) -> Vec<PathBuf> {
    let adjacent = resource_bundle_path(executable);
    let Some(directory) = executable.parent() else {
        return vec![adjacent];
    };
    let mut candidates = vec![adjacent];

    // cargo-packager places Debian resources under
    // /usr/lib/<binary>/resources rather than next to /usr/bin/<binary>.
    if directory.file_name().and_then(|name| name.to_str()) == Some("bin")
        && let (Some(prefix), Some(binary)) = (directory.parent(), executable.file_stem())
    {
        candidates.push(prefix.join("lib").join(binary).join("resources/bundle.js"));
    }

    // A macOS .app keeps executables and resources in sibling directories.
    if directory.file_name().and_then(|name| name.to_str()) == Some("MacOS")
        && let Some(contents) = directory.parent()
    {
        candidates.push(contents.join("Resources/resources/bundle.js"));
        candidates.push(contents.join("Resources/bundle.js"));
    }
    candidates
}

#[cfg(test)]
mod tests {
    use super::{
        HostBuilder, JsonCapability, JsonCapabilityContract, JsonMethod,
        install_host_message_producers, invoke_json_method, resource_bundle_candidates,
        resource_bundle_path, test_environment, test_report_summary, test_trace_artifact,
        write_test_artifact,
    };
    use crate::host_message::{HostMessagePayload, host_message_channel};
    use crate::{Applier, HostMessageContext, JsRuntime};
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    #[derive(serde::Deserialize)]
    struct JsonRequest {
        value: u32,
    }

    #[derive(serde::Serialize)]
    struct JsonResponse {
        doubled: u32,
    }

    #[test]
    fn json_capability_encodes_successful_results() {
        let response = futures_lite::future::block_on(invoke_json_method(
            r#"{"value":21}"#,
            |request: JsonRequest| async move {
                Ok::<_, String>(JsonResponse {
                    doubled: request.value * 2,
                })
            },
        ));

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&response).unwrap(),
            serde_json::json!({ "ok": true, "value": { "doubled": 42 } })
        );
    }

    #[test]
    fn json_capability_encodes_handler_errors() {
        let response = futures_lite::future::block_on(invoke_json_method(
            r#"{"value":7}"#,
            |_request: JsonRequest| async move {
                Err::<JsonResponse, _>("palette unavailable".to_owned())
            },
        ));

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&response).unwrap(),
            serde_json::json!({
                "ok": false,
                "error": {
                    "code": "handlerFailure",
                    "message": "palette unavailable",
                },
            })
        );
    }

    #[test]
    fn json_capability_rejects_malformed_requests_before_calling_the_handler() {
        let response = futures_lite::future::block_on(invoke_json_method(
            r#"{"value":"wrong"}"#,
            |_request: JsonRequest| async move {
                panic!("malformed requests must not reach the handler");
                #[allow(unreachable_code)]
                Ok::<_, String>(JsonResponse { doubled: 0 })
            },
        ));
        let response = serde_json::from_str::<serde_json::Value>(&response).unwrap();

        assert_eq!(response["ok"], false);
        assert_eq!(response["error"]["code"], "invalidRequest");
        assert!(
            response["error"]["message"]
                .as_str()
                .unwrap()
                .starts_with("invalid capability request:")
        );
    }

    #[test]
    fn json_capability_classifies_response_encoding_failures() {
        struct InvalidResponse;

        impl serde::Serialize for InvalidResponse {
            fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                Err(serde::ser::Error::custom("fixture cannot be encoded"))
            }
        }

        let response = futures_lite::future::block_on(invoke_json_method(
            r#"{"value":7}"#,
            |_request: JsonRequest| async move { Ok::<_, String>(InvalidResponse) },
        ));
        let response = serde_json::from_str::<serde_json::Value>(&response).unwrap();

        assert_eq!(response["ok"], false);
        assert_eq!(response["error"]["code"], "responseEncodingFailure");
        assert_eq!(
            response["error"]["message"],
            "cannot encode capability response: fixture cannot be encoded"
        );
    }

    #[test]
    fn json_capability_mounts_structured_and_empty_requests_in_quickjs() {
        let runtime = JsRuntime::new().expect("runtime");
        runtime
            .mount_capability("typed", |ctx, object| {
                let capability = JsonCapability { ctx, object };
                capability.method(
                    JsonMethod::new("double"),
                    |request: JsonRequest| async move {
                        Ok::<_, String>(JsonResponse {
                            doubled: request.value * 2,
                        })
                    },
                )?;
                capability.method(JsonMethod::no_request("ping"), |(): ()| async move {
                    Ok::<_, String>("pong")
                })
            })
            .expect("mount JSON capability");
        runtime
            .with(|ctx| {
                ctx.eval::<(), _>(
                    r#"
                    globalThis.capabilityResult = undefined;
                    Promise.all([
                      __wabou_capabilities.typed.double(JSON.stringify({ value: 9 })),
                      __wabou_capabilities.typed.ping(),
                    ]).then(values => globalThis.capabilityResult = JSON.stringify(values.map(JSON.parse)));
                    "#,
                )
            })
            .expect("invoke JSON capability");
        runtime.poll_async_runtime();
        let result = runtime
            .with(|ctx| ctx.eval::<Option<String>, _>("globalThis.capabilityResult"))
            .expect("read JSON capability result")
            .expect("capability promise settled");

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&result).unwrap(),
            serde_json::json!([
                { "ok": true, "value": { "doubled": 18 } },
                { "ok": true, "value": "pong" },
            ])
        );
    }

    #[test]
    fn json_capability_publishes_the_shared_abi_version() {
        const CONTRACT: JsonCapabilityContract = JsonCapabilityContract::new("versioned", 7);
        let host = HostBuilder::new().json_capability(CONTRACT, |_capability| Ok(()));
        let runtime = JsRuntime::new().expect("runtime");

        for install in host.capabilities {
            install(&runtime).expect("install capability");
        }

        let version = runtime
            .with(|ctx| {
                ctx.eval::<u32, _>(
                    "globalThis.__wabou_capabilities.versioned.__wabouCapabilityVersion",
                )
            })
            .expect("read capability version");
        assert_eq!(version, CONTRACT.version());
    }

    #[test]
    fn json_capability_rejects_duplicate_method_registration() {
        let runtime = JsRuntime::new().expect("runtime");
        let result = runtime.mount_capability("typed", |ctx, object| {
            let capability = JsonCapability { ctx, object };
            capability.method(JsonMethod::no_request("ping"), |(): ()| async move {
                Ok::<_, String>("first")
            })?;
            capability.method(JsonMethod::no_request("ping"), |(): ()| async move {
                Ok::<_, String>("second")
            })
        });

        assert!(result.is_err());
    }

    #[test]
    fn json_capability_uses_the_shared_method_identifier_rules() {
        let runtime = JsRuntime::new().expect("runtime");
        let result = runtime.mount_capability("typed", |ctx, object| {
            JsonCapability { ctx, object }
                .method(JsonMethod::no_request("1ping"), |(): ()| async move {
                    Ok::<_, String>("pong")
                })
        });

        assert!(result.is_err());
    }

    #[cfg(feature = "profiling")]
    #[test]
    fn profiling_writes_only_opted_in_perf_spans_without_source_locations() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("trace.json");
        let file = std::fs::File::create(&path).unwrap();
        let guard = super::init_profile_tracing(file, tracing_subscriber::EnvFilter::new("info"))
            .expect("test owns the process-global tracing subscriber");
        {
            let span = tracing::trace_span!(
                target: "wabou::perf",
                "quick.test",
                nodes = 3_u64,
            );
            let _guard = span.enter();
            tracing::trace!(target: "unrelated", secret = "must-not-appear", "private");
        }
        drop(guard);
        let trace = std::fs::read_to_string(path).unwrap();
        assert!(trace.contains("quick.test"));
        assert!(trace.contains("nodes"));
        assert!(!trace.contains("must-not-appear"));
        assert!(!trace.contains("host.rs"));
    }

    #[test]
    fn packaged_bundle_is_resolved_next_to_the_executable() {
        assert_eq!(
            resource_bundle_path(Path::new("/opt/demo/demo")),
            Path::new("/opt/demo/resources/bundle.js")
        );
    }

    #[test]
    fn native_packages_expose_platform_resource_candidates() {
        assert_eq!(
            resource_bundle_candidates(Path::new("/usr/bin/warden-desktop")),
            [
                Path::new("/usr/bin/resources/bundle.js").to_path_buf(),
                Path::new("/usr/lib/warden-desktop/resources/bundle.js").to_path_buf(),
            ]
        );
        assert_eq!(
            resource_bundle_candidates(Path::new(
                "/Applications/Warden.app/Contents/MacOS/warden-desktop"
            )),
            [
                Path::new("/Applications/Warden.app/Contents/MacOS/resources/bundle.js")
                    .to_path_buf(),
                Path::new("/Applications/Warden.app/Contents/Resources/resources/bundle.js")
                    .to_path_buf(),
                Path::new("/Applications/Warden.app/Contents/Resources/bundle.js").to_path_buf(),
            ]
        );
    }

    #[test]
    fn test_summary_keeps_diagnostics_but_omits_action_payloads() {
        let report = serde_json::json!({
            "environment": {
                "backend": "deterministic",
                "os": "linux",
                "arch": "x86_64",
                "wabouVersion": "0.1.0-test",
            },
            "passed": false,
            "tests": [
                {
                    "name": "opens dialog",
                    "passed": true,
                    "durationMs": 12.25,
                },
                {
                    "name": "submits form",
                    "passed": false,
                    "durationMs": 7.75,
                    "error": "expected Save to be enabled\n    at form.tsx:42:3",
                },
            ],
            "trace": [
                { "action": "clickByRole", "label": "private action payload" },
                { "action": "inputByRole", "input": { "text": "secret" } },
            ],
        });

        let summary = test_report_summary(&report, Some(Path::new("/tmp/artifacts")));
        assert!(summary.contains("test result: FAILED. 1 passed; 1 failed; 2 actions; 20.0ms"));
        assert!(summary.contains("---- submits form ----"));
        assert!(summary.contains("at form.tsx:42:3"));
        assert!(summary.contains("environment: deterministic; linux/x86_64; wabou 0.1.0-test"));
        assert!(summary.contains("artifacts: /tmp/artifacts"));
        assert!(!summary.contains("private action payload"));
        assert!(!summary.contains("secret"));
    }

    #[test]
    fn standalone_test_trace_has_an_explicit_schema_version() {
        let trace = serde_json::json!([{ "action": "showWindow", "windowId": 1 }]);
        assert_eq!(
            test_trace_artifact(&trace),
            serde_json::json!({
                "version": 1,
                "actions": trace,
            })
        );
    }

    #[test]
    fn test_artifacts_are_published_without_leaving_the_temporary_file() {
        let directory = tempfile::tempdir().unwrap();
        write_test_artifact(directory.path(), "report.json", "{\"passed\":true}\n").unwrap();
        assert_eq!(
            std::fs::read_to_string(directory.path().join("report.json")).unwrap(),
            "{\"passed\":true}\n"
        );
        assert!(!directory.path().join("report.json.tmp").exists());
    }

    #[test]
    fn test_environment_distinguishes_deterministic_and_native_runs() {
        let deterministic = test_environment(true);
        let native = test_environment(false);
        assert_eq!(deterministic["backend"], "deterministic");
        assert_eq!(native["backend"], "native");
        assert_eq!(deterministic["os"], std::env::consts::OS);
        assert_eq!(deterministic["arch"], std::env::consts::ARCH);
        assert_eq!(deterministic["wabouVersion"], env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn application_message_producers_receive_each_window_handle() {
        let observed = Arc::new(Mutex::new(Vec::new()));
        let producer_observed = observed.clone();
        let builder = HostBuilder::new().host_message_producer(move |context| {
            producer_observed.lock().unwrap().push(context.window_id());
            context
                .messages()
                .emit_i32("ready", context.window_id() as i32)
                .unwrap();
        });
        let (handle, inbox) = host_message_channel(4);
        let cancellation = tokio_util::sync::CancellationToken::new();
        let runtime = tokio::runtime::Runtime::new().unwrap();

        for producer in &builder.host_message_producers {
            producer(crate::HostMessageContext::new(
                7,
                handle.clone(),
                cancellation.clone(),
                runtime.handle().clone(),
            ));
        }

        assert_eq!(*observed.lock().unwrap(), [7]);
        let messages = inbox.drain_batch();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].topic, "ready");
        assert_eq!(messages[0].payload, HostMessagePayload::I32(7));
    }

    #[test]
    fn producer_context_is_cancelled_when_window_source_drops() {
        let observed = Arc::new(Mutex::new(None::<HostMessageContext>));
        let producer_observed = observed.clone();
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (stopped_tx, stopped_rx) = std::sync::mpsc::channel();
        let builder = HostBuilder::new().host_message_producer(move |context| {
            *producer_observed.lock().unwrap() = Some(context.clone());
            let task_context = context.clone();
            let started_tx = started_tx.clone();
            let stopped_tx = stopped_tx.clone();
            context.spawn(async move {
                started_tx.send(()).unwrap();
                task_context.cancelled().await;
                stopped_tx.send(()).unwrap();
            });
        });
        let runtime = JsRuntime::new().unwrap();
        let applier =
            Applier::from_runtime(runtime, vello::peniko::Color::from_rgb8(0x00, 0x00, 0x00));
        install_host_message_producers(&builder.host_message_producers, 3, &applier);
        started_rx
            .recv_timeout(std::time::Duration::from_secs(1))
            .unwrap();
        let context = observed.lock().unwrap().clone().unwrap();
        assert_eq!(context.window_id(), 3);
        assert!(!context.is_cancelled());

        drop(applier);

        stopped_rx
            .recv_timeout(std::time::Duration::from_secs(1))
            .unwrap();
        assert!(context.is_cancelled());
    }
}
