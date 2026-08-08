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

use vello::peniko::Color;

use crate::applier::Applier;
use crate::jsrt::JsRuntime;
use crate::widget::{PasswordInput, SecretStore, Widget, WidgetFactory, builtin_factories};
use crate::{WindowOptions, run_windows_with_factory, style};

type CapabilityInstaller = Arc<dyn Fn(&JsRuntime) -> rquickjs::Result<()>>;

pub struct HostBuilder {
    base_color: Color,
    window: WindowOptions,
    additional_windows: Vec<WindowOptions>,
    widget_factories: HashMap<String, WidgetFactory>,
    capabilities: Vec<CapabilityInstaller>,
    devtools: bool,
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

    /// Build the JsRuntime + Applier + run the winit event loop.
    pub fn run(self) -> crate::Result<()> {
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

        let capabilities = self.capabilities.clone();
        let widget_factories = self.widget_factories.clone();
        let base_color = self.base_color;
        let child_debug_state = debug_state.clone();
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

        run_windows_with_factory(sources, Some(factory)).context(crate::error::ShellSnafu)?;
        #[cfg(feature = "vite")]
        drop(hmr_clients);
        #[cfg(feature = "vite")]
        drop(child_hmr_clients);
        drop(devtools_server);
        Ok(())
    }
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
        candidates.push(
            prefix
                .join("lib")
                .join(binary)
                .join("resources/bundle.js"),
        );
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
