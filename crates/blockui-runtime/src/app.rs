//! Native application entry point. All runtime and renderer ownership stays in the host.
use crate::{
    ActorId, Runtime,
    native::{NativeSource, ReloadConfig},
    native_model::NativeFamily,
};
use std::path::{Path, PathBuf};
pub use wabou_shell_vello::WindowOptions;

/// Run an already mounted native model on the platform GUI thread.
pub fn run_source<M: crate::native_model::NativeModel + 'static>(
    source: NativeSource<M>,
    options: WindowOptions,
) -> Result<(), String> {
    wabou_shell_vello::run_window_with_options(Box::new(source), options)
        .map_err(|error| error.to_string())
}

/// Configure a root actor and its named child implementations, then enter Winit.
/// Root admission is deferred until all child implementations have been registered.
pub struct App {
    services: crate::application::Services,
    runtime: Runtime,
    root: PathBuf,
    options: WindowOptions,
    root_watch: Option<ReloadConfig>,
    renderer_retention: usize,
    implementation_watches: Vec<(String, ReloadConfig)>,
    initial_requests: Vec<(u64, String)>,
}
impl App {
    /// `artifacts` is an application-owned directory for immutable dylib generations.
    pub fn new(root: impl AsRef<Path>, artifacts: impl AsRef<Path>) -> Self {
        Self {
            services: Default::default(),
            runtime: Runtime::new(artifacts.as_ref()),
            root: root.as_ref().to_owned(),
            options: WindowOptions::new(),
            root_watch: None,
            renderer_retention: 0,
            implementation_watches: Vec::new(),
            initial_requests: Vec::new(),
        }
    }
    /// Business objects and handlers stay in this executable across UI reloads.
    pub fn services(mut self, services: crate::application::Services) -> Self {
        self.services = services;
        self
    }
    pub fn window(mut self, options: WindowOptions) -> Self {
        self.options = options;
        self
    }
    /// Submit an owned text request after root/children admission and before the
    /// first native frame. Errors abort assembly; requests are never replayed on
    /// reload. This is suitable for an initial canonical route from CLI arguments.
    pub fn initial_request(mut self, action: u64, text: impl Into<String>) -> Result<Self, String> {
        let text = text.into();
        if action == 0
            || self.initial_requests.len() >= 64
            || text.len() > blockui_abi::MAX_MESSAGE_TEXT_BYTES
            || self
                .initial_requests
                .iter()
                .map(|(_, text)| text.len())
                .sum::<usize>()
                + text.len()
                > blockui_abi::MAX_TEXT_LIST_BYTES
        {
            return Err("invalid initial request or startup request budget exceeded".into());
        }
        self.initial_requests.push((action, text));
        Ok(self)
    }
    /// Retain at most this many removed root-actor renderer records. Zero is
    /// the default. Native widgets are rebuilt; actor state is independent.
    /// Child actors retain their own explicit policies.
    pub fn renderer_retention(mut self, capacity: usize) -> Result<Self, String> {
        if capacity > blockui_abi::MAX_VIEW_NODES {
            return Err("renderer retention exceeds node budget".into());
        }
        self.renderer_retention = capacity;
        Ok(self)
    }
    /// Register a named implementation before admitting the root's declared children.
    pub fn implementation(mut self, name: &str, library: impl AsRef<Path>) -> Result<Self, String> {
        self.runtime
            .register_implementation(name, library.as_ref())?;
        Ok(self)
    }
    /// Replace only the root implementation after successful development builds.
    pub fn watch(mut self, config: ReloadConfig) -> Self {
        self.root_watch = Some(config);
        self
    }
    /// Share a build/watch stream across children declaring the named implementation.
    pub fn watch_implementation(
        mut self,
        name: &str,
        config: ReloadConfig,
    ) -> Result<Self, String> {
        if name.is_empty()
            || self.implementation_watches.len() >= 64
            || self
                .implementation_watches
                .iter()
                .any(|(existing, _)| blockui_abi::key(existing) == blockui_abi::key(name))
        {
            return Err("duplicate implementation watcher or capacity reached".into());
        }
        self.implementation_watches.push((name.to_owned(), config));
        Ok(self)
    }
    /// Assemble without opening a window, for custom host loops and native verification.
    /// Consuming ownership ensures the same runtime becomes the retained renderer model.
    pub fn into_window(mut self) -> Result<(NativeSource<NativeFamily>, WindowOptions), String> {
        // Validate the entire configuration before admitting actors or starting workers.
        // A misspelled name otherwise leaves an idle watcher that can never publish.
        for (name, _) in &self.implementation_watches {
            if !self.runtime.has_implementation(name) {
                return Err(format!("watched implementation {name:?} is not registered"));
            }
        }
        let root = ActorId(1);
        self.runtime.spawn(root, &self.root)?;
        self.runtime
            .actor_mut(root)
            .unwrap()
            .set_services(self.services);
        for (action, text) in self.initial_requests {
            self.runtime
                .actor_mut(root)
                .unwrap()
                .dispatch_text_request(action, text)
                .map_err(|error| format!("initial request rejected: {error}"))?;
            // Publish child/bus commands through the stable runtime before
            // admitting the next initial turn. Void actions need no reply future.
            let faults = self.runtime.tick();
            if let Some(fault) = faults.first() {
                return Err(format!("initial request failed: {}", fault.error));
            }
        }
        self.runtime
            .actor_mut(root)
            .unwrap()
            .view
            .set_renderer_retention(self.renderer_retention)?;
        let family = NativeFamily::new(self.runtime, root)?;
        let mut source = if let Some(config) = self.root_watch {
            NativeSource::new(family, config)?
        } else {
            NativeSource::without_watch(family)?
        };
        for (name, config) in self.implementation_watches {
            source.watch_implementation(&name, config)?;
        }
        Ok((source, self.options))
    }
    /// Run on the operating system's GUI thread. Compiler errors keep the live actor running.
    pub fn run(self) -> Result<(), String> {
        let (source, options) = self.into_window()?;
        wabou_shell_vello::run_window_with_options(Box::new(source), options)
            .map_err(|error| error.to_string())
    }
    /// Assemble independent actor runtimes for one native event loop.
    /// Give each application its own artifact directory: actor IDs are local to
    /// each runtime. If admission fails, all already assembled sources are dropped.
    pub fn into_windows(
        apps: impl IntoIterator<Item = Self>,
    ) -> Result<Vec<(NativeSource<NativeFamily>, WindowOptions)>, String> {
        apps.into_iter().map(Self::into_window).collect()
    }
    /// Run independent windows on the operating system's GUI thread.
    /// Closing a window disposes its runtime; the other windows continue running.
    pub fn run_windows(apps: impl IntoIterator<Item = Self>) -> Result<(), String> {
        let windows = Self::into_windows(apps)?
            .into_iter()
            .map(|(source, options)| {
                (
                    Box::new(source) as Box<dyn wabou_shell_vello::FrameSource>,
                    options,
                )
            })
            .collect();
        wabou_shell_vello::run_windows(windows)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

/// Render a native capture through the same stable host application assembly.
pub fn run_capture(
    source: NativeSource<NativeFamily>,
    options: WindowOptions,
) -> Result<(), String> {
    wabou_shell_vello::run_window_with_options(Box::new(source.capture()), options)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initial_requests_are_bounded_before_actor_admission() {
        let mut app = App::new("not-loaded.so", ".runtime");
        for _ in 0..64 {
            app = app.initial_request(1, "").unwrap();
        }
        assert!(app.initial_request(1, "").is_err());
        assert!(
            App::new("not-loaded.so", ".runtime")
                .initial_request(1, "x".repeat(blockui_abi::MAX_MESSAGE_TEXT_BYTES + 1))
                .is_err()
        );
        let mut app = App::new("not-loaded.so", ".runtime");
        let chunk = blockui_abi::MAX_MESSAGE_TEXT_BYTES.min(blockui_abi::MAX_TEXT_LIST_BYTES);
        for _ in 0..blockui_abi::MAX_TEXT_LIST_BYTES / chunk {
            app = app.initial_request(1, "x".repeat(chunk)).unwrap();
        }
        assert!(app.initial_request(1, "x").is_err());
    }
}
