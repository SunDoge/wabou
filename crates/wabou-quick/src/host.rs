//! HostBuilder — Tauri-like entry point for wabou apps.
//!
//! Users register custom widgets (Rust `Widget` impls keyed by tag) +
//! named capabilities (Rust fns callable from JS), then `.run()`. Built-in widgets
//! (Canvas) are pre-registered; users override or add their own.
//!
//! ```ignore
//! use wabou_quick::{HostBuilder, Widget};
//!
//! HostBuilder::new()
//!     .widget("chart", || Box::new(MyChart::new()))
//!     .capability("workspace", |ctx, capability| {
//!         capability.set("readFile", rquickjs::Function::new(ctx, |p: String| {
//!             std::fs::read_to_string(&p).unwrap_or_default()
//!         })?)?;
//!         Ok(())
//!     })
//!     .run()
//!     .unwrap();
//! ```

use snafu::ResultExt;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use vello::peniko::Color;

use crate::applier::Applier;
use crate::jsrt::JsRuntime;
use crate::widget::{PasswordInput, SecretStore, Widget, WidgetFactory, builtin_factories};
use crate::{ShellExtension, WindowOptions, run_windows_with_factory_and_extensions, style};

type CapabilityInstaller = Arc<dyn Fn(&JsRuntime) -> rquickjs::Result<()>>;

enum EffectTraceConfig {
    Record { path: PathBuf, record_all: bool },
    Replay { path: PathBuf },
}

pub struct HostBuilder {
    base_color: Color,
    window: WindowOptions,
    additional_windows: Vec<WindowOptions>,
    widget_factories: HashMap<String, WidgetFactory>,
    capabilities: Vec<CapabilityInstaller>,
    devtools: bool,
    extensions: Vec<Box<dyn ShellExtension>>,
    effect_trace: Option<EffectTraceConfig>,
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
            devtools: cfg!(debug_assertions),
            extensions: Vec::new(),
            effect_trace: None,
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
        self.widget_factories.insert(
            "password-input".into(),
            Arc::new(move || Box::new(PasswordInput::new(secrets.clone()))),
        );
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

    pub fn base_color(mut self, color: Color) -> Self {
        self.base_color = color;
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
        let windows = std::iter::once(self.window.clone())
            .chain(self.additional_windows.iter().cloned())
            .collect::<Vec<_>>();
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .try_init()
            .ok();

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
                applier.boot(bundle.as_deref().unwrap()).context(
                    crate::error::JavaScriptSnafu {
                        operation: "boot JavaScript bundle",
                    },
                )?;
            }
            #[cfg(not(feature = "vite"))]
            applier
                .boot(&bundle)
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
            controller.initialize_headless(1..=sources.len() as u64);
            let deadline = Instant::now() + Duration::from_secs(10);
            let mut text = crate::TextContext::new();
            let mut last_nodes = vec![Vec::new(); sources.len()];
            while !controller.has_report() && Instant::now() < deadline {
                for (index, (source, _)) in sources.iter_mut().enumerate() {
                    source.set_semantics_enabled(true);
                    source.handle_event(wabou_shell::UiEvent::WindowMetrics(
                        crate::WindowMetrics {
                            window_id: index as u64 + 1,
                            logical_width: 1100,
                            logical_height: 720,
                            physical_width: 1100,
                            physical_height: 720,
                            scale_factor: 1.0,
                            maximized: false,
                            focused: true,
                        },
                    ));
                    last_nodes[index] = source.build_frame(&mut text, 1100, 720);
                    controller.poll_headless_source(index as u64 + 1, source.as_mut());
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            if controller.report_passed() == Some(false)
                && let Some(directory) =
                    std::env::var_os("WABOU_TEST_ARTIFACT_DIR").map(PathBuf::from)
                && let Some(nodes) = last_nodes.first()
            {
                std::fs::create_dir_all(&directory).map_err(|error| {
                    crate::Error::TestScenario {
                        message: format!("cannot create failure artifact directory: {error}"),
                    }
                })?;
                let mut scene = vello::Scene::new();
                wabou_shell::scene::build_scene_scaled(
                    &mut scene,
                    nodes,
                    &mut text,
                    1100,
                    720,
                    self.base_color,
                    1.0,
                );
                let output = directory.join("failure.png");
                wabou_shell::renderer::render_to_png(
                    &scene,
                    1100,
                    720,
                    self.base_color,
                    output.to_string_lossy().as_ref(),
                )
                .map_err(|error| crate::Error::TestScenario {
                    message: format!("cannot render failure screenshot: {error:?}"),
                })?;
            }
            return finish_test_report(controller.clone());
        }

        let capabilities = self.capabilities.clone();
        let widget_factories = self.widget_factories.clone();
        let base_color = self.base_color;
        let child_debug_state = debug_state.clone();
        let child_effect_trace = effect_trace.clone();
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

fn finish_test_report(controller: crate::test_driver::TestController) -> crate::Result<()> {
    let report = controller
        .take_report()
        .ok_or_else(|| crate::Error::TestScenario {
            message: "host exited or timed out before the scenario reported a result".into(),
        })?;
    let value = serde_json::from_str::<serde_json::Value>(&report).map_err(|error| {
        crate::Error::TestScenario {
            message: format!("scenario returned invalid JSON: {error}"),
        }
    })?;
    if let Some(directory) = std::env::var_os("WABOU_TEST_ARTIFACT_DIR").map(PathBuf::from) {
        std::fs::create_dir_all(&directory).map_err(|error| crate::Error::TestScenario {
            message: format!(
                "cannot create artifact directory {}: {error}",
                directory.display()
            ),
        })?;
        std::fs::write(directory.join("report.json"), format!("{value:#}\n")).map_err(|error| {
            crate::Error::TestScenario {
                message: format!("cannot write test report: {error}"),
            }
        })?;
        if let Some(trace) = value.get("trace") {
            std::fs::write(directory.join("trace.json"), format!("{trace:#}\n")).map_err(
                |error| crate::Error::TestScenario {
                    message: format!("cannot write action trace: {error}"),
                },
            )?;
        }
    }
    println!("{report}");
    if !value
        .get("passed")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return Err(crate::Error::TestScenario { message: report });
    }
    Ok(())
}

fn load_bundle() -> crate::Result<String> {
    let path = bundle_path()?;
    std::fs::read_to_string(&path).context(crate::error::ReadFileSnafu {
        kind: "JavaScript bundle",
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
    use super::{resource_bundle_candidates, resource_bundle_path};
    use std::path::Path;

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
}
