//! GPUI-owned retained state for one JavaScript runtime.

use crate::reload::{HmrBatch, HmrDrainResult};
use crate::{
    ImageResourceHandle, ImageResourceStore,
    host_frame::{HostEvent, HostNodeEvent, NodeEventPayload, NumericEventData},
    jsrt::HostFrameDisposition,
    protocol::{Frame, decode_frame},
    runtime_session::RuntimeSession,
};
use gpui_shell::{GpuiProjection, ProjectionError};
use wabou_style::stylesheet::{StyleSheet, StylesheetUpdate};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GpuiFrameTiming {
    pub(crate) js_tick_ms: f64,
    pub(crate) projection_ms: f64,
}

/// Renderer-side state consumed exclusively by the GPUI application runtime.
///
/// This controller intentionally does not know about the retired renderer or
/// its document model. GPUI owns layout, text, painting, widgets, and input;
/// this type only connects the JavaScript runtime to that retained projection.
pub struct GpuiController {
    pub(crate) runtime: RuntimeSession,
    projection: GpuiProjection,
    image_resources: ImageResourceStore,
    next_host_event_id: u32,
    hovered_target: Option<wabou_host_api::NodeKey>,
    pressed_targets: std::collections::BTreeMap<u8, wabou_host_api::NodeKey>,
    pointer_buttons: u32,
    last_primary_click: Option<(std::time::Instant, wabou_host_api::NodeKey, f32, f32)>,
    focused_target: Option<wabou_host_api::NodeKey>,
    last_window_metrics: Option<gpui_shell::WindowMetrics>,
}

impl GpuiController {
    pub(crate) fn new(runtime: RuntimeSession) -> Self {
        Self {
            runtime,
            projection: GpuiProjection::new(),
            image_resources: ImageResourceStore::default(),
            next_host_event_id: 0,
            hovered_target: None,
            pressed_targets: std::collections::BTreeMap::new(),
            pointer_buttons: 0,
            last_primary_click: None,
            focused_target: None,
            last_window_metrics: None,
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

    #[cfg(test)]
    pub(crate) fn projection(&self) -> &GpuiProjection {
        &self.projection
    }

    pub(crate) fn projection_mut(&mut self) -> &mut GpuiProjection {
        &mut self.projection
    }

    pub(crate) fn take_projection_commands(&mut self) -> Vec<gpui_shell::GpuiCommand> {
        self.projection.take_commands()
    }

    pub(crate) fn apply_projection_scroll(&mut self, command: gpui_shell::GpuiCommand) -> bool {
        self.projection
            .apply_scroll_command(command)
            .is_some_and(|event| self.handle_projected_scroll(event).handled)
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

    pub(crate) fn poll_runtime(&mut self) -> bool {
        let progressed = self.poll_runtime_session();
        if self.runtime.host_message_inbox.has_pending() {
            self.drain_application_messages();
            return true;
        }
        progressed
    }

    pub(crate) fn take_runtime_host_action(&mut self) -> Option<gpui_shell::HostAction> {
        self.runtime.pending_host_actions.borrow_mut().pop_front()
    }

    pub(crate) fn complete_runtime_host_action(&mut self, result: gpui_shell::HostActionResult) {
        // Clipboard host actions were introduced for legacy Rust widgets. GPUI
        // native widgets own their platform integration and JS uses effects,
        // so there is no request route to complete in this controller.
        let _ = result;
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

    /// Publish a window-level native file-drag transition through the shared
    /// host-to-JavaScript application-message frame.
    pub(crate) fn dispatch_file_drop(&mut self, event: gpui_shell::FileDropEvent) -> bool {
        let phase = match event.phase {
            gpui_shell::FileDropPhase::Entered => "entered",
            gpui_shell::FileDropPhase::Moved => "moved",
            gpui_shell::FileDropPhase::Left => "left",
            gpui_shell::FileDropPhase::Dropped => "dropped",
        };
        let payload = serde_json::json!({
            "phase": phase,
            "paths": event.paths.into_iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            "position": event.position.map(|position| serde_json::json!({
                "x": position.x,
                "y": position.y,
            })),
        });
        self.dispatch_host_frame(&[HostEvent::Application(crate::HostMessage::str(
            "wabou:file-drop",
            payload.to_string(),
        ))])
        .is_ok()
    }

    /// Publish an authoritative GPUI window snapshot when its observable state
    /// changed since the previous completed frame.
    pub(crate) fn update_window_metrics(&mut self, metrics: gpui_shell::WindowMetrics) -> bool {
        if self.last_window_metrics == Some(metrics) {
            return false;
        }
        let payload = serde_json::json!({
            "windowId": metrics.window_key,
            "logicalWidth": metrics.logical_width,
            "logicalHeight": metrics.logical_height,
            "physicalWidth": metrics.physical_width,
            "physicalHeight": metrics.physical_height,
            "scaleFactor": metrics.scale_factor,
            "maximized": metrics.maximized,
            "focused": metrics.focused,
            "outerX": metrics.outer_x,
            "outerY": metrics.outer_y,
            "occluded": metrics.occluded,
            "colorScheme": metrics.color_scheme.map(|scheme| match scheme {
                gpui_shell::ColorScheme::Light => "light",
                gpui_shell::ColorScheme::Dark => "dark",
            }),
        });
        let published = self
            .dispatch_host_frame(&[HostEvent::Application(crate::HostMessage::str(
                "wabou:window-metrics",
                payload.to_string(),
            ))])
            .is_ok();
        if published {
            self.last_window_metrics = Some(metrics);
        }
        published
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

    pub fn handle_projected_pointer(
        &mut self,
        input: gpui_shell::ProjectedPointerEvent,
    ) -> gpui_shell::EventResponse {
        use gpui_shell::{ProjectedPointerButton as Button, ProjectedPointerPhase as Phase};
        use wabou_protocol::{event, event_data};

        let button = input.button.map_or(0, |button| match button {
            Button::Primary => 0,
            Button::Auxiliary => 1,
            Button::Secondary => 2,
            Button::Other => 3,
        });
        let button_mask = input.button.map_or(0, |button| match button {
            Button::Primary => 1,
            Button::Secondary => 2,
            Button::Auxiliary => 4,
            Button::Other => 8,
        });
        match input.phase {
            Phase::Down => self.pointer_buttons |= button_mask,
            Phase::Up => self.pointer_buttons &= !button_mask,
            Phase::Move => {}
        }

        let mut data = [0.0; event_data::LEN];
        data[event_data::CLIENT_X as usize] = f64::from(input.x);
        data[event_data::CLIENT_Y as usize] = f64::from(input.y);
        data[event_data::OFFSET_X as usize] = f64::from(input.local_x);
        data[event_data::OFFSET_Y as usize] = f64::from(input.local_y);
        data[event_data::BUTTON as usize] = f64::from(button);
        data[event_data::BUTTONS as usize] = f64::from(self.pointer_buttons);
        let mut modifiers = gpui_shell::Modifiers::empty();
        modifiers.set(gpui_shell::Modifiers::SHIFT, input.shift);
        modifiers.set(gpui_shell::Modifiers::CONTROL, input.control);
        modifiers.set(gpui_shell::Modifiers::ALT, input.alt);
        modifiers.set(gpui_shell::Modifiers::META, input.platform);
        data[event_data::MODS as usize] = f64::from(modifiers.bits());
        data[event_data::POINTER_ID_LO as usize] = 1.0;
        data[event_data::POINTER_TYPE as usize] = 0.0;
        data[event_data::PRIMARY as usize] = 1.0;
        for slot in [
            event_data::PRESSURE,
            event_data::TANGENTIAL_PRESSURE,
            event_data::TILT_X,
            event_data::TILT_Y,
            event_data::TWIST,
        ] {
            data[slot as usize] = f64::NAN;
        }

        let mut handled = false;
        if self.hovered_target != Some(input.target) {
            if let Some(previous) = self.hovered_target.take() {
                handled |= self
                    .dispatch_node_numeric(
                        previous,
                        event::POINTEROUT,
                        data,
                        event_data::TWIST as usize + 1,
                        false,
                    )
                    .map(|value| value.0)
                    .unwrap_or(false);
                handled |= self
                    .dispatch_node_numeric(
                        previous,
                        event::POINTERLEAVE,
                        data,
                        event_data::TWIST as usize + 1,
                        false,
                    )
                    .map(|value| value.0)
                    .unwrap_or(false);
            }
            self.hovered_target = Some(input.target);
            handled |= self
                .dispatch_node_numeric(
                    input.target,
                    event::POINTEROVER,
                    data,
                    event_data::TWIST as usize + 1,
                    false,
                )
                .map(|value| value.0)
                .unwrap_or(false);
            handled |= self
                .dispatch_node_numeric(
                    input.target,
                    event::POINTERENTER,
                    data,
                    event_data::TWIST as usize + 1,
                    false,
                )
                .map(|value| value.0)
                .unwrap_or(false);
        }

        match input.phase {
            Phase::Move => {
                handled |= self
                    .dispatch_node_numeric(
                        input.target,
                        event::POINTERMOVE,
                        data,
                        event_data::TWIST as usize + 1,
                        false,
                    )
                    .map(|value| value.0)
                    .unwrap_or(false);
            }
            Phase::Down => {
                self.pressed_targets.insert(button, input.target);
                handled |= self
                    .dispatch_node_numeric(
                        input.target,
                        event::POINTERDOWN,
                        data,
                        event_data::TWIST as usize + 1,
                        false,
                    )
                    .map(|value| value.0)
                    .unwrap_or(false);
            }
            Phase::Up => {
                let pressed = self.pressed_targets.remove(&button);
                let release_target = pressed.unwrap_or(input.target);
                handled |= self
                    .dispatch_node_numeric(
                        release_target,
                        event::POINTERUP,
                        data,
                        event_data::TWIST as usize + 1,
                        false,
                    )
                    .map(|value| value.0)
                    .unwrap_or(false);
                if pressed == Some(input.target) {
                    let activation = if button == 2 {
                        Some(event::CONTEXTMENU)
                    } else if button == 0 {
                        Some(event::CLICK)
                    } else {
                        None
                    };
                    if let Some(code) = activation {
                        handled |= self
                            .dispatch_node_numeric(
                                input.target,
                                code,
                                data,
                                event_data::TWIST as usize + 1,
                                true,
                            )
                            .map(|value| value.0)
                            .unwrap_or(false);
                    }
                    if button == 0 {
                        let now = std::time::Instant::now();
                        let double = self.last_primary_click.is_some_and(|(then, target, x, y)| {
                            target == input.target
                                && now.duration_since(then) <= std::time::Duration::from_millis(400)
                                && (input.x - x).abs() <= 4.0
                                && (input.y - y).abs() <= 4.0
                        });
                        if double {
                            handled |= self
                                .dispatch_node_numeric(
                                    input.target,
                                    event::DBLCLICK,
                                    data,
                                    event_data::TWIST as usize + 1,
                                    true,
                                )
                                .map(|value| value.0)
                                .unwrap_or(false);
                            self.last_primary_click = None;
                        } else {
                            self.last_primary_click = Some((now, input.target, input.x, input.y));
                        }
                    }
                }
            }
        }
        gpui_shell::EventResponse {
            handled,
            request_redraw: handled,
            ..gpui_shell::EventResponse::default()
        }
    }

    pub fn handle_projected_wheel(
        &mut self,
        input: gpui_shell::ProjectedWheelEvent,
    ) -> gpui_shell::EventResponse {
        use wabou_protocol::{event, event_data};

        let mut modifiers = gpui_shell::Modifiers::empty();
        modifiers.set(gpui_shell::Modifiers::SHIFT, input.shift);
        modifiers.set(gpui_shell::Modifiers::CONTROL, input.control);
        modifiers.set(gpui_shell::Modifiers::ALT, input.alt);
        modifiers.set(gpui_shell::Modifiers::META, input.platform);
        let scale = if input.precise {
            1.0
        } else {
            gpui_shell::WHEEL_LINE_DELTA
        };
        let mut data = [0.0; event_data::LEN];
        data[event_data::CLIENT_X as usize] = f64::from(input.x);
        data[event_data::CLIENT_Y as usize] = f64::from(input.y);
        data[event_data::OFFSET_X as usize] = f64::from(input.local_x);
        data[event_data::OFFSET_Y as usize] = f64::from(input.local_y);
        data[event_data::MODS as usize] = f64::from(modifiers.bits());
        data[event_data::DELTA_X as usize] = -f64::from(input.delta_x) * scale;
        data[event_data::DELTA_Y as usize] = -f64::from(input.delta_y) * scale;
        data[event_data::PHASE as usize] = match input.phase {
            gpui_shell::ProjectedWheelPhase::Started => 0.0,
            gpui_shell::ProjectedWheelPhase::Changed => 1.0,
            gpui_shell::ProjectedWheelPhase::Ended => 2.0,
            gpui_shell::ProjectedWheelPhase::Cancelled => 3.0,
        };
        let handled = self
            .dispatch_node_numeric(
                input.target,
                event::WHEEL,
                data,
                event_data::PHASE as usize + 1,
                true,
            )
            .map(|value| value.0)
            .unwrap_or(false);
        gpui_shell::EventResponse {
            handled,
            request_redraw: handled,
            ..gpui_shell::EventResponse::default()
        }
    }

    pub fn handle_projected_scroll(
        &mut self,
        input: gpui_shell::ProjectedScrollEvent,
    ) -> gpui_shell::EventResponse {
        use wabou_protocol::{event, event_data};

        let mut data = [0.0; event_data::LEN];
        data[event_data::SCROLL_X as usize] = f64::from(input.x);
        data[event_data::SCROLL_Y as usize] = f64::from(input.y);
        let handled = self
            .dispatch_node_numeric(
                input.target,
                event::SCROLL,
                data,
                event_data::SCROLL_Y as usize + 1,
                false,
            )
            .map(|value| value.0)
            .unwrap_or(false);
        gpui_shell::EventResponse {
            handled,
            request_redraw: true,
            ..gpui_shell::EventResponse::default()
        }
    }

    pub(crate) fn text_controls(&self) -> Vec<gpui_shell::GpuiTextControl> {
        self.projection.text_controls()
    }

    pub(crate) fn native_widgets(
        &self,
        accepts: impl FnMut(&str) -> bool,
    ) -> Vec<gpui_shell::GpuiNativeWidget> {
        self.projection.native_widgets(accepts)
    }

    pub(crate) fn interactive_element(
        &self,
        input: gpui_shell::ProjectedInputSink,
        focus: gpui_shell::gpui::FocusHandle,
        text_input: gpui_shell::ProjectedTextInputState,
        native: Option<gpui_shell::ProjectedNativeElementFactory>,
    ) -> Result<gpui_shell::ProjectedElement, ProjectionError> {
        self.projection.interactive_tree_element(
            wabou_host_api::NodeKey::ROOT,
            input,
            focus,
            text_input,
            native,
        )
    }

    pub(crate) fn layout_snapshot(&self) -> Vec<gpui_shell::GpuiLayoutNode> {
        self.projection.layout_snapshot()
    }

    pub(crate) fn focused_target(&self) -> Option<wabou_host_api::NodeKey> {
        self.focused_target
    }

    #[cfg(test)]
    pub(crate) fn contains(&self, target: wabou_host_api::NodeKey) -> bool {
        self.projection.contains(target)
    }

    pub(crate) fn text_input_state(&self) -> gpui_shell::ProjectedTextInputState {
        let Some(target) = self.focused_target else {
            return gpui_shell::ProjectedTextInputState::default();
        };
        let Some(control) = self
            .projection
            .text_controls()
            .into_iter()
            .find(|control| control.key == target)
        else {
            return gpui_shell::ProjectedTextInputState::default();
        };
        gpui_shell::ProjectedTextInputState {
            accepts_text: !control.disabled && !control.readonly,
            text: Some(control.value),
            selection: None,
            selection_reversed: false,
            cursor_bounds: None,
        }
    }

    pub(crate) fn commit_text_value(
        &mut self,
        target: wabou_host_api::NodeKey,
        value: &str,
    ) -> bool {
        if self
            .projection
            .update_authored_attribute(target, "value", value)
            .is_err()
        {
            return false;
        }
        let payload = serde_json::json!({ "value": value }).to_string();
        self.dispatch_node_json(target, wabou_protocol::event::INPUT, payload, false)
            .map(|result| result.0)
            .unwrap_or(false)
    }

    pub(crate) fn set_text_focus(
        &mut self,
        target: wabou_host_api::NodeKey,
        focused: bool,
    ) -> bool {
        let next = if focused { Some(target) } else { None };
        if (!focused && self.focused_target != Some(target)) || self.focused_target == next {
            return false;
        }
        let previous = std::mem::replace(&mut self.focused_target, next);
        let mut changed = previous != next;
        if let Some(previous) = previous {
            changed |= self
                .dispatch_node_json(previous, wabou_protocol::event::BLUR, "{}".into(), false)
                .map(|result| result.0)
                .unwrap_or(false);
            changed |= self
                .dispatch_node_json(
                    previous,
                    wabou_protocol::event::FOCUSOUT,
                    "{}".into(),
                    false,
                )
                .map(|result| result.0)
                .unwrap_or(false);
        }
        if let Some(next) = next {
            changed |= self
                .dispatch_node_json(next, wabou_protocol::event::FOCUS, "{}".into(), false)
                .map(|result| result.0)
                .unwrap_or(false);
            changed |= self
                .dispatch_node_json(next, wabou_protocol::event::FOCUSIN, "{}".into(), false)
                .map(|result| result.0)
                .unwrap_or(false);
        }
        changed
    }

    pub(crate) fn handle_input(
        &mut self,
        event: gpui_shell::ProjectedInputEvent,
    ) -> gpui_shell::EventResponse {
        match event {
            gpui_shell::ProjectedInputEvent::Pointer(event) => self.handle_projected_pointer(event),
            gpui_shell::ProjectedInputEvent::Wheel(event) => self.handle_projected_wheel(event),
            gpui_shell::ProjectedInputEvent::Scroll(event) => self.handle_projected_scroll(event),
            gpui_shell::ProjectedInputEvent::Key(event) => self.handle_projected_key(event),
            gpui_shell::ProjectedInputEvent::Ime(event) => self.handle_projected_ime(event),
        }
    }

    pub fn handle_projected_key(
        &mut self,
        input: gpui_shell::ProjectedKeyEvent,
    ) -> gpui_shell::EventResponse {
        let Some(target) = self.focused_target else {
            return gpui_shell::EventResponse::default();
        };
        let mut modifiers = gpui_shell::Modifiers::empty();
        modifiers.set(gpui_shell::Modifiers::SHIFT, input.shift);
        modifiers.set(gpui_shell::Modifiers::CONTROL, input.control);
        modifiers.set(gpui_shell::Modifiers::ALT, input.alt);
        modifiers.set(gpui_shell::Modifiers::META, input.platform);
        let payload = serde_json::json!({
            "key": input.key_char.as_deref().unwrap_or(&input.key),
            "keyWithoutModifiers": input.key,
            "code": input.key,
            "location": 0,
            "mods": modifiers.bits(),
            "primary": modifiers.primary_shortcut(),
            "repeat": input.repeat,
            "synthetic": false,
        })
        .to_string();
        let (event_code, cancellable) = match input.phase {
            gpui_shell::ProjectedKeyPhase::Down => (wabou_protocol::event::KEYDOWN, true),
            gpui_shell::ProjectedKeyPhase::Up => (wabou_protocol::event::KEYUP, false),
        };
        let (mut handled, prevented) = self
            .dispatch_node_json(target, event_code, payload, cancellable)
            .unwrap_or((false, false));
        let accepts_text = self.text_input_state().accepts_text;
        if input.phase == gpui_shell::ProjectedKeyPhase::Down
            && !prevented
            && !accepts_text
            && !input.control
            && !input.platform
            && let Some(text) = input
                .key_char
                .filter(|text| text.chars().any(|character| !character.is_control()))
        {
            handled |= self.dispatch_text_commit(target, text, "keyboard");
        }
        gpui_shell::EventResponse {
            handled,
            request_redraw: handled,
            consume_key_text: prevented,
            text_input: Some(accepts_text),
            clipboard: None,
        }
    }

    pub fn handle_projected_ime(
        &mut self,
        input: gpui_shell::ProjectedImeEvent,
    ) -> gpui_shell::EventResponse {
        let Some(target) = self.focused_target else {
            return gpui_shell::EventResponse::default();
        };
        let handled = match input {
            gpui_shell::ProjectedImeEvent::Commit(text) => {
                self.dispatch_text_commit(target, text, "ime")
            }
            gpui_shell::ProjectedImeEvent::Preedit { text, cursor } => {
                let (cursor_start, cursor_end) = cursor
                    .map(|(start, end)| (Some(start), Some(end)))
                    .unwrap_or((None, None));
                self.dispatch_node_json(
                    target,
                    wabou_protocol::event::IMEPREEDIT,
                    serde_json::json!({
                        "data": text,
                        "cursorStart": cursor_start,
                        "cursorEnd": cursor_end,
                    })
                    .to_string(),
                    false,
                )
                .map(|result| result.0)
                .unwrap_or(false)
            }
        };
        gpui_shell::EventResponse {
            handled,
            request_redraw: handled,
            text_input: Some(self.text_input_state().accepts_text),
            ..gpui_shell::EventResponse::default()
        }
    }

    pub fn dispatch_paste(&mut self, text: String) -> gpui_shell::EventResponse {
        let Some(target) = self.focused_target else {
            return gpui_shell::EventResponse::default();
        };
        let handled = self.dispatch_text_commit(target, text, "paste");
        gpui_shell::EventResponse {
            handled,
            request_redraw: handled,
            ..gpui_shell::EventResponse::default()
        }
    }

    fn dispatch_text_commit(
        &mut self,
        target: wabou_host_api::NodeKey,
        text: String,
        source: &str,
    ) -> bool {
        self.dispatch_node_json(
            target,
            wabou_protocol::event::IMECOMMIT,
            serde_json::json!({ "data": text, "source": source }).to_string(),
            false,
        )
        .map(|result| result.0)
        .unwrap_or(false)
    }

    pub(crate) fn drain_hmr(&mut self) -> HmrDrainResult {
        let Some(batch) = self.runtime.reload.drain() else {
            return HmrDrainResult::Idle;
        };
        let result = self.apply_hmr_batch(batch);
        self.runtime.reload.record_result(result.clone());
        result
    }

    fn apply_hmr_batch(&mut self, batch: HmrBatch) -> HmrDrainResult {
        if let Some(diagnostic) = batch.error {
            tracing::error!(target: "hmr", diagnostic = %diagnostic, "Vite update failed; keeping last-good UI");
            self.dispatch_application_message(crate::host_message::HostMessage::str(
                "wabou:dev-server-error",
                diagnostic.clone(),
            ));
            return HmrDrainResult::Error { diagnostic };
        }
        for path in &batch.css_paths {
            tracing::warn!(
                target: "hmr",
                %path,
                "ignoring native Vite css-update; layout styles use virtual:wabou-stylesheet"
            );
        }
        if batch.full_reload {
            let reason = batch
                .full_reload_reason
                .unwrap_or_else(|| "vite full-reload".into());
            self.perform_full_reload(&reason);
            return HmrDrainResult::FullReload { reason };
        }

        #[cfg(feature = "vite")]
        let mut applied = 0usize;
        #[cfg(not(feature = "vite"))]
        let applied = batch.js_updates.len();
        for update in batch.js_updates {
            #[cfg(feature = "vite")]
            match self.runtime.js.apply_hmr_update(
                &update.path,
                &update.accepted_path,
                update.timestamp,
                update.source,
            ) {
                Ok(true) => applied += 1,
                Ok(false) => {
                    let reason = format!("module declined or missing hot context: {}", update.path);
                    self.perform_full_reload(&reason);
                    return HmrDrainResult::FullReload { reason };
                }
                Err(error) => {
                    let reason = format!("apply_hmr failed for {}: {error:?}", update.path);
                    self.perform_full_reload(&reason);
                    return HmrDrainResult::FullReload { reason };
                }
            }
            #[cfg(not(feature = "vite"))]
            let _ = update;
        }
        if applied > 0 || !batch.css_paths.is_empty() {
            self.dispatch_dev_server_ready();
            HmrDrainResult::Applied {
                js_updates: applied,
            }
        } else {
            HmrDrainResult::Idle
        }
    }

    fn perform_full_reload(&mut self, reason: &str) {
        tracing::warn!(target: "hmr", %reason, "performing in-process GPUI full reload");
        #[cfg(feature = "vite")]
        if let Some(entry) = self.runtime.reload.vite_entry().map(str::to_owned) {
            match self.runtime.js.reboot_vite_entry(&entry) {
                Ok(()) => {
                    self.runtime.has_raf = true;
                    self.dispatch_dev_server_ready();
                }
                Err(error) => tracing::error!(
                    target: "hmr",
                    %entry,
                    ?error,
                    "full reload re-import failed; keeping last-good GPUI tree"
                ),
            }
            return;
        }
        tracing::error!(target: "hmr", %reason, "full reload requested without a Vite entry");
    }

    fn dispatch_dev_server_ready(&mut self) {
        self.dispatch_application_message(crate::host_message::HostMessage::str(
            "wabou:dev-server-ready",
            "{}",
        ));
    }

    fn dispatch_application_message(&mut self, message: crate::host_message::HostMessage) {
        if let Err(error) = self.dispatch_host_frame(&[HostEvent::Application(message)]) {
            tracing::error!(target: "bridge", ?error, "failed to dispatch application message");
        }
    }

    fn drain_application_messages(&mut self) {
        let messages = self.runtime.host_message_inbox.drain_batch();
        if messages.is_empty() {
            return;
        }
        let events = messages
            .into_iter()
            .map(HostEvent::Application)
            .collect::<Vec<_>>();
        if let Err(error) = self.dispatch_host_frame(&events) {
            tracing::error!(target: "host_message", ?error, "failed to dispatch application messages");
        }
    }

    fn drain_projection_updates(&mut self) {
        if let Some(update) = self.take_stylesheet_update() {
            match update {
                StylesheetUpdate::Ir(sheet) => {
                    for diagnostic in &sheet.diagnostics {
                        tracing::warn!(target: "stylesheet", %diagnostic);
                    }
                    if let Err(error) = self.install_stylesheet(sheet) {
                        tracing::error!(target: "stylesheet", %error, "failed to install GPUI stylesheet");
                    }
                }
            }
        }
        if let Some(name) = self.take_color_theme()
            && let Err(error) = self.select_color_theme(&name)
        {
            tracing::warn!(target: "stylesheet", %name, %error, "failed to select GPUI color theme");
        }
        if let Some(colors) = self.take_color_palette()
            && let Err(error) = self.projection.set_ordered_color_palette(colors)
        {
            tracing::warn!(target: "stylesheet", %error, "failed to install GPUI color palette");
        }
    }

    /// Advance one complete Solid flush and publish its retained GPUI tree.
    pub fn advance_frame(&mut self) -> bool {
        self.advance_frame_profiled().0
    }

    pub(crate) fn advance_frame_profiled(&mut self) -> (bool, GpuiFrameTiming) {
        self.prepare_js_tick();
        let _ = self.drain_hmr();
        self.drain_application_messages();
        self.drain_projection_updates();
        let js_started = std::time::Instant::now();
        let bytes = match self.tick_js() {
            Ok(bytes) => bytes,
            Err(error) => {
                let timing = GpuiFrameTiming {
                    js_tick_ms: js_started.elapsed().as_secs_f64() * 1_000.0,
                    projection_ms: 0.0,
                };
                self.fail_js_tick();
                tracing::error!(target: "bridge", ?error, "GPUI JavaScript tick failed");
                return (false, timing);
            }
        };
        let js_tick_ms = js_started.elapsed().as_secs_f64() * 1_000.0;
        let projection_started = std::time::Instant::now();
        if !bytes.is_empty() {
            match decode_frame(&bytes) {
                Ok(frame) => {
                    self.record_protocol_frame();
                    if let Err(error) = self.apply_frame(&frame) {
                        tracing::error!(target: "bridge", ?error, "failed to project GPUI protocol frame");
                    }
                }
                Err(error) => {
                    tracing::error!(target: "bridge", %error, "failed to decode GPUI protocol frame")
                }
            }
        }
        self.finish_js_tick();
        self.drain_projection_updates();
        let changed = self.projection.finish_frame();
        (
            changed,
            GpuiFrameTiming {
                js_tick_ms,
                projection_ms: projection_started.elapsed().as_secs_f64() * 1_000.0,
            },
        )
    }

    pub(crate) fn publish_frame_stats(
        &mut self,
        timing: GpuiFrameTiming,
        build_frame_ms: f64,
        viewport: (u32, u32),
    ) {
        let Some(frame_stats) = &self.runtime.frame_stats else {
            return;
        };
        let node_count = self.projection.node_count();
        let mut frame_stats = frame_stats.borrow_mut();
        if let Some(stats) = frame_stats.as_mut() {
            stats.update_gpui(
                build_frame_ms,
                timing.js_tick_ms,
                timing.projection_ms,
                node_count,
                viewport,
            );
        } else {
            *frame_stats = Some(gpui_shell::FrameStats {
                build_frame_ms,
                js_tick_ms: timing.js_tick_ms,
                scene_ms: timing.projection_ms,
                present_ms: 0.0,
                node_count,
                viewport_w: viewport.0,
                viewport_h: viewport.1,
            });
        }
    }

    pub(crate) fn has_animation(&self) -> bool {
        self.runtime.has_raf
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

    #[cfg(feature = "vite")]
    pub fn reload_handle(&mut self) -> crate::ReloadHandle {
        self.runtime.reload.handle()
    }

    #[cfg(feature = "vite")]
    pub fn set_vite_entry(&mut self, entry: impl Into<String>) {
        self.runtime.reload.set_vite_entry(entry);
    }

    pub(crate) fn set_effect_trace(&mut self, trace: crate::effect_trace::EffectTrace) {
        self.runtime.effect_bridge.set_trace(trace);
    }

    /// Publish application-private directories to native effects.
    pub fn set_app_directories(&mut self, directories: gpui_shell::AppDirectories) {
        self.runtime.effect_bridge.set_app_directories(directories);
    }

    #[cfg(feature = "devtools")]
    pub fn set_debug_state(&mut self, state: wabou_devtools::SharedDebugState) {
        self.runtime.js.set_debug_state(state);
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

    fn install_application_message_probe(js: &JsRuntime) {
        js.eval_script(
            r#"
            globalThis.__host_got = [];
            globalThis.__wabou_dispatch_host_frame = (u8) => {
              const view = new DataView(u8.buffer, u8.byteOffset, u8.byteLength);
              const decoder = new TextDecoder();
              let offset = 32;
              const count = view.getUint32(24, true);
              for (let index = 0; index < count; index++) {
                const start = offset;
                const kind = view.getUint8(offset);
                const length = view.getUint32(offset + 4, true);
                offset += 8;
                if (kind === 3) {
                  const topicLength = view.getUint16(offset, true);
                  offset += 2;
                  const topic = decoder.decode(u8.subarray(offset, offset + topicLength));
                  offset += topicLength;
                  const valueKind = u8[offset++];
                  let payload = null;
                  if (valueKind === 4) {
                    const payloadLength = view.getUint16(offset, true);
                    offset += 2;
                    payload = decoder.decode(u8.subarray(offset, offset + payloadLength));
                  }
                  globalThis.__host_got.push({ topic, payload });
                }
                offset = start + length;
              }
              return { needsTick: false, preventedEventIds: new Uint32Array() };
            };
            "#,
        )
        .expect("install application-message probe");
    }

    #[test]
    fn native_file_drop_reaches_javascript_through_gpui_controller() {
        let js = JsRuntime::new().expect("runtime");
        install_application_message_probe(&js);
        let mut controller = GpuiController::new(RuntimeSession::new(
            js,
            gpui_shell::initial_window_resource_key(0),
        ));

        assert!(controller.dispatch_file_drop(gpui_shell::FileDropEvent {
            phase: gpui_shell::FileDropPhase::Dropped,
            paths: vec!["/tmp/one.yaml".into(), "/tmp/two.torrent".into()],
            position: Some(gpui_shell::Point { x: 24.5, y: 31.0 }),
        }));

        let payload = controller
            .eval_string(
                "globalThis.__host_got.find((value) => value.topic === 'wabou:file-drop').payload",
            )
            .expect("read file-drop payload");
        let payload: serde_json::Value = serde_json::from_str(&payload).expect("file-drop json");
        assert_eq!(payload["phase"], "dropped");
        assert_eq!(payload["paths"][0], "/tmp/one.yaml");
        assert_eq!(payload["paths"][1], "/tmp/two.torrent");
        assert_eq!(payload["position"]["x"], 24.5);
        assert_eq!(payload["position"]["y"], 31.0);
    }

    #[test]
    fn gpui_window_metrics_reach_javascript_once_per_distinct_snapshot() {
        let js = JsRuntime::new().expect("runtime");
        install_application_message_probe(&js);
        let mut controller = GpuiController::new(RuntimeSession::new(
            js,
            gpui_shell::initial_window_resource_key(0),
        ));
        let metrics = gpui_shell::WindowMetrics {
            window_key: gpui_shell::initial_window_resource_key(0),
            logical_width: 800,
            logical_height: 600,
            physical_width: 1600,
            physical_height: 1200,
            scale_factor: 2.0,
            maximized: true,
            focused: true,
            outer_x: Some(120),
            outer_y: Some(80),
            occluded: false,
            color_scheme: Some(gpui_shell::ColorScheme::Dark),
        };

        assert!(controller.update_window_metrics(metrics));
        assert!(!controller.update_window_metrics(metrics));

        let messages = controller
            .eval_string(
                "JSON.stringify(globalThis.__host_got.filter((value) => value.topic === 'wabou:window-metrics'))",
            )
            .expect("read window-metrics messages");
        let messages: serde_json::Value = serde_json::from_str(&messages).expect("message json");
        assert_eq!(messages.as_array().map(Vec::len), Some(1));
        let payload: serde_json::Value =
            serde_json::from_str(messages[0]["payload"].as_str().expect("payload string"))
                .expect("window-metrics payload");
        assert_eq!(payload["windowId"], serde_json::json!({ "lo": 1, "hi": 1 }));
        assert_eq!(payload["logicalWidth"], 800);
        assert_eq!(payload["physicalWidth"], 1600);
        assert_eq!(payload["scaleFactor"], 2.0);
        assert_eq!(payload["focused"], true);
        assert_eq!(payload["colorScheme"], "dark");
    }

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

    #[test]
    fn projected_pointer_sequence_uses_gpui_target_and_local_coordinates() {
        let js = JsRuntime::new().expect("runtime");
        js.eval_script(
            r#"
            globalThis.receivedEvents = [];
            globalThis.__wabou_dispatch_host_frame = (bytes) => {
              const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
              const record = 40;
              const code = view.getUint8(record + 8);
              const payloadKind = view.getUint8(record + 9);
              const values = [];
              if (payloadKind === 1) {
                const count = view.getUint16(record + 10, true);
                for (let index = 0; index < count; index++) {
                  values.push(view.getFloat64(record + 16 + index * 8, true));
                }
              }
              globalThis.receivedEvents.push([code, values]);
              return { needsTick: false, preventedEventIds: new Uint32Array() };
            };
            "#,
        )
        .expect("install host-frame hook");
        let mut controller = GpuiController::new(RuntimeSession::new(
            js,
            gpui_shell::initial_window_resource_key(0),
        ));
        let button = controller.runtime.atoms.borrow_mut().intern("button");
        let target = NodeKey::new(7, 3);
        let listeners = [
            wabou_protocol::event::POINTEROVER,
            wabou_protocol::event::POINTERENTER,
            wabou_protocol::event::POINTERDOWN,
            wabou_protocol::event::POINTERUP,
            wabou_protocol::event::CLICK,
        ];
        let mut ops = vec![Op::CreateElement {
            id: target,
            tag: button,
        }];
        ops.extend(listeners.map(|event_type| Op::AddEventListener {
            id: target,
            event_type,
        }));
        controller
            .apply_frame(&Frame { seq: 1, ops })
            .expect("project listeners");

        let event = |phase| gpui_shell::ProjectedPointerEvent {
            target,
            phase,
            x: 110.0,
            y: 75.0,
            local_x: 10.0,
            local_y: 15.0,
            button: Some(gpui_shell::ProjectedPointerButton::Primary),
            shift: false,
            control: false,
            alt: false,
            platform: false,
        };
        assert!(
            controller
                .handle_projected_pointer(event(gpui_shell::ProjectedPointerPhase::Down))
                .handled
        );
        assert!(
            controller
                .handle_projected_pointer(event(gpui_shell::ProjectedPointerPhase::Up))
                .handled
        );

        let trace = controller
            .eval_string("JSON.stringify(globalThis.receivedEvents)")
            .expect("read pointer trace");
        let trace: serde_json::Value = serde_json::from_str(&trace).expect("parse pointer trace");
        let events = trace.as_array().expect("event array");
        assert_eq!(
            events
                .iter()
                .map(|event| event[0].as_u64().unwrap() as u8)
                .collect::<Vec<_>>(),
            [
                wabou_protocol::event::POINTEROVER,
                wabou_protocol::event::POINTERENTER,
                wabou_protocol::event::POINTERDOWN,
                wabou_protocol::event::POINTERUP,
                wabou_protocol::event::CLICK,
            ]
        );
        assert_eq!(
            events[2][1][wabou_protocol::event_data::OFFSET_X as usize],
            10.0
        );
        assert_eq!(
            events[2][1][wabou_protocol::event_data::OFFSET_Y as usize],
            15.0
        );
    }
}
