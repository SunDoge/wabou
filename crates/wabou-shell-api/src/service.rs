//! Application service lifecycle shared by every native backend.

use crate::AppDirectories;

/// Application-owned resource that must surround every native window and JavaScript runtime.
pub trait HostService: Send + Sync {
    /// Stable name used in diagnostics.
    fn name(&self) -> &'static str;

    /// Start the service before Wabou creates JavaScript runtimes.
    fn start(&self, context: &HostServiceContext) -> Result<(), String>;

    /// Stop the service after the native event loop finishes.
    fn shutdown(&self) -> Result<(), String>;
}

/// Read-only environment available while a host-owned service starts.
#[derive(Clone, Debug)]
pub struct HostServiceContext {
    app_directories: Option<AppDirectories>,
    behavior_test: bool,
    headless: bool,
}

impl HostServiceContext {
    /// Construct service context for an alternate Wabou application host.
    #[doc(hidden)]
    pub fn for_alternate_host(
        app_directories: Option<AppDirectories>,
        behavior_test: bool,
        headless: bool,
    ) -> Self {
        Self {
            app_directories,
            behavior_test,
            headless,
        }
    }

    /// Return the application directories resolved by the host, when configured.
    pub fn app_directories(&self) -> Option<&AppDirectories> {
        self.app_directories.as_ref()
    }

    /// Return whether `wabou test` controls this host run.
    pub fn is_behavior_test(&self) -> bool {
        self.behavior_test
    }

    /// Return whether the deterministic headless shell is active.
    pub fn is_headless(&self) -> bool {
        self.headless
    }
}
