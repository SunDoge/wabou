//! Lightweight launch SDK. No renderer, QuickJS, windowing or Tokio dependencies.
use serde::{Deserialize, Serialize};
use stabby::libloading::StabbyLibrary;
use std::{
    mem::ManuallyDrop,
    path::{Path, PathBuf},
};

/// Launch contract revision, independently checked from native signature layout.
pub const LAUNCH_VERSION: u32 = 1;
/// ABI-stable launch result; owned errors are consumed before library retirement.
pub type LaunchResult = stabby::result::Result<(), stabby::string::String>;
/// Launch entry checked by stabby's exported signature report.
pub type LaunchEntry = extern "C" fn(stabby::string::String) -> LaunchResult;

/// UI source loaded by the resident runtime.
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum UiSource {
    /// Production/development bundle.
    Bundle { path: PathBuf },
    /// Vite entry; UI updates do not rebuild the host.
    Vite { url: String, entry: String },
}
/// One method published by a capability contract.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Method {
    pub name: String,
    pub has_request: bool,
    /// Keep the generated JSON-envelope client when true.
    pub json: bool,
}
/// Implementation source, independent of UI and resident runtime.
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Implementation {
    Library {
        path: PathBuf,
    },
    Project {
        manifest: PathBuf,
        #[serde(default)]
        watch: Vec<PathBuf>,
        #[serde(default)]
        features: Vec<String>,
        /// Optional isolated Cargo output directory for independent builders.
        #[serde(default, rename = "targetDirectory")]
        target_directory: Option<PathBuf>,
    },
}
/// Declarative capability registration consumed inside the runtime library.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Capability {
    pub name: String,
    pub version: u32,
    pub methods: Vec<Method>,
    pub implementation: Implementation,
}
/// Declarative native widget registration. State remains in the resident node.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeWidget {
    pub tag: String,
    /// Fixed state/protocol schema; incompatible reloads are rejected.
    pub version: u32,
    /// Used only when a native node is first created.
    pub initial_state: serde_json::Value,
    pub implementation: Implementation,
}
/// Capability contract selected from a plugin registry.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginCapability {
    pub name: String,
    pub version: u32,
    pub methods: Vec<Method>,
}
/// Widget state slot selected from a plugin registry.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginWidget {
    pub tag: String,
    pub version: u32,
    pub initial_state: serde_json::Value,
}
/// A single compiled/watched library with several independent host consumers.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Plugin {
    pub name: String,
    pub implementation: Implementation,
    #[serde(default)]
    pub capabilities: Vec<PluginCapability>,
    #[serde(default)]
    pub widgets: Vec<PluginWidget>,
}
/// Small startup configuration. Its JSON schema is separate from the ABI layout.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LaunchConfig {
    pub version: u32,
    pub title: String,
    pub width: u32,
    pub height: u32,
    /// Show the operating system's title bar and window borders.
    #[serde(
        default = "native_decorations",
        skip_serializing_if = "decorations_enabled"
    )]
    pub decorations: bool,
    pub source: UiSource,
    /// Base directory for relative paths in resident resource operations.
    #[serde(default)]
    pub resource_directory: Option<PathBuf>,
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub widgets: Vec<NativeWidget>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plugins: Vec<Plugin>,
}
impl LaunchConfig {
    /// Create a default window configuration for a source.
    pub fn new(source: UiSource) -> Self {
        Self {
            version: LAUNCH_VERSION,
            title: "Wabou".into(),
            width: 800,
            height: 600,
            decorations: true,
            source,
            resource_directory: None,
            capabilities: Vec::new(),
            widgets: Vec::new(),
            plugins: Vec::new(),
        }
    }

    /// Resolve application paths relative to the configuration file's directory.
    pub fn resolve_paths(&mut self, directory: &Path) {
        fn resolve(path: &mut PathBuf, directory: &Path) {
            if path.is_relative() {
                *path = directory.join(&*path);
            }
        }
        match &mut self.resource_directory {
            Some(path) => resolve(path, directory),
            None => self.resource_directory = Some(directory.to_owned()),
        }
        if let UiSource::Bundle { path } = &mut self.source {
            resolve(path, directory);
        }
        for implementation in self
            .capabilities
            .iter_mut()
            .map(|c| &mut c.implementation)
            .chain(self.widgets.iter_mut().map(|w| &mut w.implementation))
            .chain(self.plugins.iter_mut().map(|p| &mut p.implementation))
        {
            match implementation {
                Implementation::Library { path } => resolve(path, directory),
                Implementation::Project {
                    manifest,
                    watch,
                    target_directory,
                    ..
                } => {
                    resolve(manifest, directory);
                    for path in watch {
                        resolve(path, directory);
                    }
                    if let Some(path) = target_directory {
                        resolve(path, directory);
                    }
                }
            }
        }
    }
}
fn native_decorations() -> bool {
    true
}
fn decorations_enabled(value: &bool) -> bool {
    *value
}
/// Resident precompiled runtime. It deliberately remains loaded until process
/// exit: platform/GPU/TLS callbacks may remain after a window event loop returns.
/// Construct one per launcher process, not one per capability replacement.
pub struct RuntimeLibrary {
    library: ManuallyDrop<libloading::Library>,
    _main_thread: std::marker::PhantomData<std::rc::Rc<()>>,
}
impl RuntimeLibrary {
    /// Load trusted native runtime code and check its launch signature.
    ///
    /// # Safety
    /// The library must implement this SDK's launch/lifetime contract. Native
    /// code is not sandboxed. Launch must run on the process's main thread.
    pub unsafe fn load(path: &Path) -> Result<Self, String> {
        let library = unsafe { libloading::Library::new(path) }.map_err(|e| e.to_string())?;
        unsafe { library.get_stabbied::<LaunchEntry>(b"wabou_runtime_launch") }
            .map_err(|e| e.to_string())?;
        Ok(Self {
            library: ManuallyDrop::new(library),
            _main_thread: std::marker::PhantomData,
        })
    }
    /// Enter the runtime's window event loop on the main thread. The stable
    /// runtime consumes all ABI-owned result allocations before returning.
    pub fn run(self, config: &LaunchConfig) -> Result<(), String> {
        let payload = serde_json::to_string(config).map_err(|e| e.to_string())?;
        let entry = unsafe {
            self.library
                .get_stabbied::<LaunchEntry>(b"wabou_runtime_launch")
        }
        .map_err(|e| e.to_string())?;
        let result: Result<(), stabby::string::String> =
            entry(stabby::string::String::from(payload.as_str())).into();
        result.map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn window_decorations_default_to_native_and_custom_chrome_round_trips() {
        let mut config = LaunchConfig::new(UiSource::Bundle {
            path: "bundle.js".into(),
        });
        let native = serde_json::to_value(&config).unwrap();
        assert!(native.get("decorations").is_none());
        assert!(
            serde_json::from_value::<LaunchConfig>(native)
                .unwrap()
                .decorations
        );
        config.decorations = false;
        let custom = serde_json::to_value(&config).unwrap();
        assert_eq!(custom["decorations"], false);
        assert!(
            !serde_json::from_value::<LaunchConfig>(custom)
                .unwrap()
                .decorations
        );
    }
}
