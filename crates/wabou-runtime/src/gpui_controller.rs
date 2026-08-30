//! GPUI-owned retained state for one JavaScript runtime.

use crate::{
    ImageResourceHandle, ImageResourceStore,
    host_frame::{HostEvent, HostNodeEvent, NodeEventPayload, NumericEventData},
    jsrt::HostFrameDisposition,
    protocol::{Frame, decode_frame},
    runtime_session::RuntimeSession,
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
    next_host_event_id: u32,
}

impl GpuiController {
    pub(crate) fn new(runtime: RuntimeSession) -> Self {
        Self {
            runtime,
            projection: GpuiProjection::new(),
            image_resources: ImageResourceStore::default(),
            next_host_event_id: 0,
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

    /// Deliver one atomic Host→JS event batch and immediately project any
    /// Solid writes returned by that synchronous dispatch.
    pub fn dispatch_host_frame(
        &mut self,
        events: &[HostEvent],
    ) -> rquickjs::Result<HostFrameDisposition> {
        let mut disposition = self.dispatch_host_frame_raw(events)?;
        if !disposition.protocol_frame.is_empty() {
            let frame =
                decode_frame(&disposition.protocol_frame).map_err(|_| rquickjs::Error::Unknown)?;
            self.record_protocol_frame();
            self.apply_frame(&frame)
                .map_err(|_| rquickjs::Error::Unknown)?;
            disposition.needs_tick |= self.projection.finish_frame();
        }
        Ok(disposition)
    }

    pub fn dispatch_node_json(
        &mut self,
        target: wabou_host_api::NodeKey,
        event_code: u8,
        payload: String,
        cancellable: bool,
    ) -> rquickjs::Result<(bool, bool)> {
        if !self.projection.has_listener_in_chain(target, event_code) {
            return Ok((false, false));
        }
        let event_id = self.next_event_id(cancellable);
        let disposition = self.dispatch_host_frame(&[HostEvent::Node(HostNodeEvent {
            target,
            event_code,
            event_id,
            cancellable,
            payload: NodeEventPayload::Json(payload),
        })])?;
        Ok((true, disposition.is_prevented(event_id)))
    }

    pub fn dispatch_node_numeric(
        &mut self,
        target: wabou_host_api::NodeKey,
        event_code: u8,
        values: [f64; wabou_protocol::event_data::LEN],
        value_count: usize,
        cancellable: bool,
    ) -> rquickjs::Result<(bool, bool)> {
        if !self.projection.has_listener_in_chain(target, event_code) {
            return Ok((false, false));
        }
        let event_id = self.next_event_id(cancellable);
        let disposition = self.dispatch_host_frame(&[HostEvent::Node(HostNodeEvent {
            target,
            event_code,
            event_id,
            cancellable,
            payload: NodeEventPayload::Numeric(NumericEventData::prefix(values, value_count)),
        })])?;
        Ok((true, disposition.is_prevented(event_id)))
    }

    fn next_event_id(&mut self, cancellable: bool) -> u32 {
        if !cancellable {
            return 0;
        }
        self.next_host_event_id = self.next_host_event_id.wrapping_add(1).max(1);
        self.next_host_event_id
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{JsRuntime, protocol::Op};
    use gpui_shell::NodeKey;

    #[test]
    fn projected_listener_dispatches_and_preserves_cancellation() {
        let js = JsRuntime::new().expect("runtime");
        js.eval_script(
            r#"
            globalThis.receivedCodes = [];
            globalThis.__wabou_dispatch_host_frame = (bytes) => {
              const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
              const record = 40;
              const code = view.getUint8(record + 8);
              const eventId = view.getUint32(record + 12, true);
              globalThis.receivedCodes.push(code);
              return {
                needsTick: false,
                preventedEventIds: new Uint32Array([eventId]),
              };
            };
            "#,
        )
        .expect("install host-frame hook");
        let mut controller = GpuiController::new(RuntimeSession::new(
            js,
            gpui_shell::initial_window_resource_key(0),
        ));
        let button = controller.runtime.atoms.borrow_mut().intern("button");
        let target = NodeKey::new(2, 1);
        controller
            .apply_frame(&Frame {
                seq: 1,
                ops: vec![
                    Op::CreateElement {
                        id: target,
                        tag: button,
                    },
                    Op::AddEventListener {
                        id: target,
                        event_type: wabou_protocol::event::CLICK,
                    },
                ],
            })
            .expect("project listener");

        assert_eq!(
            controller
                .dispatch_node_json(target, wabou_protocol::event::CLICK, "{}".into(), true,)
                .expect("dispatch click"),
            (true, true)
        );
        assert_eq!(
            controller
                .eval_string("JSON.stringify(globalThis.receivedCodes)")
                .expect("read event trace"),
            format!("[{}]", wabou_protocol::event::CLICK)
        );
    }

    #[test]
    fn event_without_projected_listener_does_not_cross_into_javascript() {
        let js = JsRuntime::new().expect("runtime");
        js.eval_script(
            "globalThis.__wabou_dispatch_host_frame = () => { throw new Error('unexpected'); };",
        )
        .expect("install rejecting hook");
        let mut controller = GpuiController::new(RuntimeSession::new(
            js,
            gpui_shell::initial_window_resource_key(0),
        ));
        assert_eq!(
            controller
                .dispatch_node_json(
                    NodeKey::new(2, 1),
                    wabou_protocol::event::CLICK,
                    "{}".into(),
                    true,
                )
                .expect("ignore absent listener"),
            (false, false)
        );
    }
}
