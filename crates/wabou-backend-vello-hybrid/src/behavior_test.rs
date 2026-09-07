//! Native behavior-test adapter for the Winit host.

use std::sync::Arc;

use runtime_api::test_driver::{NativeTestHost, TestController};

pub(crate) struct WinitBehaviorTest {
    controller: TestController,
    window_keys: Vec<vello_shell::WindowResourceKey>,
}

impl WinitBehaviorTest {
    pub(crate) fn new(
        controller: TestController,
        window_keys: Vec<vello_shell::WindowResourceKey>,
    ) -> Self {
        Self {
            controller,
            window_keys,
        }
    }
}

impl vello_shell::ShellExtension for WinitBehaviorTest {
    fn requires_semantics(&self) -> bool {
        true
    }

    fn initialize(&mut self, wake: vello_shell::WakeCallback) -> Result<(), String> {
        self.controller
            .connect_native_windows(self.window_keys.iter().copied(), wake);
        Ok(())
    }

    fn poll(&mut self, context: &mut vello_shell::ExtensionContext<'_>) {
        let mut host = WinitTestHost { context };
        for window_key in self.window_keys.iter().copied() {
            let _ = self.controller.poll_native_host(window_key, &mut host);
        }
        if self.controller.has_report() {
            host.context.exit();
        }
    }
}

struct WinitTestHost<'a, 'b> {
    context: &'a mut vello_shell::ExtensionContext<'b>,
}

impl NativeTestHost for WinitTestHost<'_, '_> {
    fn semantic_snapshot(
        &mut self,
        window_key: vello_shell::WindowResourceKey,
    ) -> Option<Arc<vello_shell::SemanticSnapshot>> {
        self.context.semantic_snapshot(window_key)
    }

    fn dispatch_event(
        &mut self,
        window_key: vello_shell::WindowResourceKey,
        event: vello_shell::UiEvent,
    ) -> bool {
        self.context.dispatch_event(window_key, event)
    }

    fn dispatch_semantic_action(
        &mut self,
        window_key: vello_shell::WindowResourceKey,
        action: vello_shell::SemanticAction,
    ) -> bool {
        self.context.dispatch_semantic_action(window_key, action)
    }

    fn hide_window(
        &mut self,
        window_key: vello_shell::WindowResourceKey,
        mutable_visibility: bool,
    ) -> bool {
        self.context.hide_window_with_capabilities(
            window_key,
            Some(vello_shell::window_lifecycle::WindowCapabilities { mutable_visibility }),
        )
    }

    fn show_window(&mut self, window_key: vello_shell::WindowResourceKey) -> bool {
        self.context.show_window(window_key)
    }

    fn resize_window(
        &mut self,
        window_key: vello_shell::WindowResourceKey,
        width: u32,
        height: u32,
    ) -> bool {
        self.context.resize_window(window_key, width, height)
    }

    fn window_viewport(&self, window_key: vello_shell::WindowResourceKey) -> Option<(u32, u32)> {
        self.context
            .window_metrics(window_key)
            .map(|metrics| (metrics.logical_width, metrics.logical_height))
    }
}
