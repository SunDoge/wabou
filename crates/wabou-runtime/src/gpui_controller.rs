//! GPUI-owned retained state for one JavaScript runtime.

use crate::{
    ImageResourceHandle, ImageResourceStore, host_frame::HostEvent, jsrt::HostFrameDisposition,
    protocol::Frame, runtime_session::RuntimeSession,
};
use gpui_shell::{GpuiProjection, ProjectionError};
use wabou_style::stylesheet::{StyleSheet, StylesheetUpdate};

/// Renderer-side state consumed exclusively by the GPUI application runtime.
///
/// This controller intentionally does not know about Taffy, Parley, Vello,
/// winit widgets, or the legacy document. It is the extraction boundary that
/// will remain in `wabou-runtime` after the old behavior oracle moves out.
pub struct GpuiController {
    pub(crate) runtime: RuntimeSession,
    projection: GpuiProjection,
    image_resources: ImageResourceStore,
}

impl GpuiController {
    pub(crate) fn new(runtime: RuntimeSession) -> Self {
        Self {
            runtime,
            projection: GpuiProjection::new(),
            image_resources: ImageResourceStore::default(),
        }
    }
    pub(crate) fn set_image_resources(&mut self, resources: ImageResourceStore) {
        self.image_resources = resources;
    }

    pub(crate) fn apply_frame(&mut self, frame: &Frame<'_>) -> Result<(), ProjectionError> {
        let atoms = self.runtime.atoms.borrow();
        self.projection.apply_ops(frame, &atoms, |source| {
            let (lo, hi) = source.split_once(':')?;
            let handle = ImageResourceHandle {
                lo: lo.parse().ok()?,
                hi: hi.parse().ok()?,
            };
            self.image_resources
                .get(handle)
                .map(|resource| resource.gpui_image())
        })
    }

    pub(crate) fn projection(&self) -> &GpuiProjection {
        &self.projection
    }

    pub(crate) fn projection_mut(&mut self) -> &mut GpuiProjection {
        &mut self.projection
    }

    pub(crate) fn take_stylesheet_update(&mut self) -> Option<StylesheetUpdate> {
        self.runtime.pending_css.as_ref()?.borrow_mut().take()
    }

    pub(crate) fn install_stylesheet(&mut self, sheet: StyleSheet) -> Result<(), String> {
        self.projection.set_stylesheet(sheet)
    }

    pub(crate) fn take_color_theme(&mut self) -> Option<String> {
        self.runtime
            .pending_color_theme
            .as_ref()?
            .borrow_mut()
            .take()
    }

    pub(crate) fn select_color_theme(&mut self, name: &str) -> Result<bool, String> {
        self.projection.set_color_theme(name)
    }

    pub(crate) fn take_color_palette(&mut self) -> Option<Vec<u32>> {
        self.runtime
            .pending_color_palette
            .as_ref()?
            .borrow_mut()
            .take()
    }

    pub(crate) fn install_color_palette(
        &mut self,
        colors: std::collections::HashMap<String, u32>,
    ) -> Result<bool, String> {
        self.projection.set_color_palette(colors)
    }

    pub(crate) fn prepare_js_tick(&mut self) {
        self.runtime.js.take_async_wake();
        self.runtime.js.poll_async_runtime();
    }

    pub(crate) fn tick_js(&mut self) -> rquickjs::Result<Vec<u8>> {
        let (bytes, has_raf) = self.runtime.js.tick()?;
        self.runtime.has_raf = has_raf;
        Ok(bytes)
    }

    pub(crate) fn fail_js_tick(&mut self) {
        self.runtime.has_raf = false;
    }

    pub(crate) fn finish_js_tick(&mut self) {
        self.runtime.js.poll_async_runtime();
    }

    pub(crate) fn install_runtime_wake(&mut self, wake: gpui_shell::WakeCallback) {
        self.runtime.js.set_wake_callback(wake.clone());
        self.runtime.host_message_inbox.set_wake(wake.clone());
        self.runtime.reload.set_wake(wake.clone());
        self.runtime.effect_bridge.set_wake_callback(wake.clone());
        self.runtime.wake_callback = Some(wake);
    }

    pub(crate) fn poll_runtime_session(&mut self) -> bool {
        let was_woken = self.runtime.js.take_async_wake();
        let js_progressed = self.runtime.js.poll_async_runtime();
        was_woken
            || js_progressed
            || self.runtime.host_message_inbox.has_pending()
            || self.runtime.reload.is_pending()
    }

    pub(crate) fn take_runtime_host_action(&mut self) -> Option<gpui_shell::HostAction> {
        self.runtime.pending_host_actions.borrow_mut().pop_front()
    }

    pub(crate) fn take_runtime_effect(&mut self) -> Option<gpui_shell::EffectRequest> {
        self.runtime.effect_bridge.take(&self.runtime.js)
    }

    pub(crate) fn complete_runtime_effect(&mut self, completion: gpui_shell::EffectCompletion) {
        self.runtime
            .effect_bridge
            .complete(&self.runtime.js, completion);
    }

    pub(crate) fn record_protocol_frame(&mut self) {
        self.runtime.protocol_revision = self.runtime.protocol_revision.wrapping_add(1);
    }

    pub(crate) fn dispatch_host_frame_raw(
        &mut self,
        events: &[HostEvent],
    ) -> rquickjs::Result<HostFrameDisposition> {
        self.runtime.js.dispatch_host_frame(events)
    }

    /// Monotonically increasing count of non-empty JS-to-host frames.
    pub fn protocol_revision(&self) -> u64 {
        self.runtime.protocol_revision
    }

    /// Boot the application after host bridges have been installed.
    pub fn boot(&mut self, source: &str) -> rquickjs::Result<()> {
        self.runtime.js.boot(source)
    }

    pub(crate) fn boot_with_source_map(
        &mut self,
        source: &str,
        source_map: Option<&[u8]>,
    ) -> rquickjs::Result<()> {
        self.runtime.js.boot_with_source_map(source, source_map)
    }

    /// Evaluate an additional script in the booted application realm.
    pub fn eval_script(&self, source: &str) -> rquickjs::Result<()> {
        self.runtime.js.eval_script(source)
    }

    /// Evaluate a test script and preserve mapped guest diagnostics.
    pub fn eval_script_diagnostic(&self, source: &str) -> Result<(), String> {
        self.runtime.js.eval_script_diagnostic(source)
    }

    /// Evaluate an expression and return its string value.
    pub fn eval_string(&self, source: &str) -> rquickjs::Result<String> {
        self.runtime.js.eval_string(source)
    }

    #[cfg(feature = "vite")]
    /// Boot an application entry module through Vite.
    pub fn boot_vite(&mut self, entry: &str) -> rquickjs::Result<()> {
        self.runtime.js.boot_vite(entry)
    }

    pub(crate) fn set_effect_trace(&mut self, trace: crate::effect_trace::EffectTrace) {
        self.runtime.effect_bridge.set_trace(trace);
    }

    /// Publish application-private directories to native effects.
    pub fn set_app_directories(&mut self, directories: gpui_shell::AppDirectories) {
        self.runtime.effect_bridge.set_app_directories(directories);
    }

    /// Cloneable producer handle for application Rust-to-JavaScript messages.
    pub fn host_message_handle(&self) -> crate::HostMessageHandle {
        self.runtime.host_message_handle.clone()
    }

    pub(crate) fn host_message_context(
        &self,
        window_key: gpui_shell::WindowResourceKey,
    ) -> crate::HostMessageContext {
        crate::HostMessageContext::new(
            window_key,
            self.host_message_handle(),
            self.runtime.host_message_cancellation.clone(),
            self.runtime.js.tokio_handle(),
            self.runtime.host_tasks.clone(),
        )
    }
}
