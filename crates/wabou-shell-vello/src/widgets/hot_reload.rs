//! Native widget adapter over the existing versioned capability loader.
//! The resident object owns its JSON state, focus, config and last valid drawing.
//! Each callback pins a library generation; malformed/failed callbacks cannot
//! replace state or drawing. This first protocol deliberately excludes arbitrary
//! Rust objects, GPU handles, IME composition and asynchronous widget callbacks.
use crate::PaintScene;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;
use vello_common::{
    kurbo::{Affine, Rect},
    peniko::{Color, Fill},
};
use wabou_capability_sdk::widget::{Draw, Event, Inputs, Request, Update};
use wabou_runtime::capability_module::CapabilityModule;
use wabou_shell::{
    MeasureContext, PaintContext, UiEvent, WakeCallback, Widget, WidgetChanges, WidgetEventResult,
};

/// Runtime-owned widget instance; construct once per stable native node.
pub struct ReloadableWidget {
    module: CapabilityModule,
    active: Option<CapabilityModule>,
    state: Value,
    config: Value,
    size: [f32; 2],
    focused: bool,
    drawing: Vec<Draw>,
    drawing_dirty: bool,
    measured: Option<[f32; 2]>,
    generation: Option<u64>,
    error: Option<String>,
    wake: Option<WakeCallback>,
}
impl ReloadableWidget {
    /// Initial state is allocated only when the node is created, never on reload.
    pub fn new(module: CapabilityModule, state: Value) -> Self {
        let active = module.snapshot().ok();
        let generation = active.as_ref().and_then(|m| m.generation().ok().flatten());
        Self {
            module,
            active,
            state,
            config: Value::Null,
            size: [0.0; 2],
            focused: false,
            drawing: Vec::new(),
            drawing_dirty: true,
            measured: None,
            generation,
            error: None,
            wake: None,
        }
    }
    /// Borrow durable state without crossing the dynamic-library boundary.
    pub fn state(&self) -> &Value {
        &self.state
    }
    /// Resident focus survives implementation replacement.
    pub fn focused(&self) -> bool {
        self.focused
    }
    /// Most recent callback failure; successful callbacks clear it.
    pub fn last_error(&self) -> Option<&str> {
        self.error.as_deref()
    }
    /// Last validated drawing, also retained if new code fails to paint.
    pub fn drawing(&self) -> &[Draw] {
        &self.drawing
    }
    fn record_error(&mut self, error: Option<String>) {
        if self.error != error
            && let Some(error) = &error
        {
            tracing::warn!(%error, "native widget callback failed; resident state/drawing retained");
        }
        self.error = error;
    }
    fn call<T: DeserializeOwned>(&self, method: &str, event: Option<Event>) -> Result<T, String> {
        let request = Request {
            inputs: Inputs {
                state: self.state.clone(),
                config: self.config.clone(),
                size: self.size,
                focused: self.focused,
            },
            event,
        };
        let request = serde_json::to_vec(&request).map_err(|e| e.to_string())?;
        let mut invocation = self
            .active
            .as_ref()
            .unwrap_or(&self.module)
            .start(method, &request)?;
        let bytes = invocation
            .poll()?
            .ok_or("widget callback must complete synchronously")?;
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())
    }
    /// Invoke the current paint implementation without a native window.
    pub fn refresh_drawing(&mut self, size: [f32; 2]) -> Result<(), String> {
        self.size = size;
        let result = self
            .call::<Vec<Draw>>("widget.paint", None)
            .and_then(|drawing| {
                validate_drawing(&drawing)?;
                self.drawing = drawing;
                self.drawing_dirty = false;
                Ok(())
            });
        self.record_error(result.as_ref().err().cloned());
        result
    }
    /// Apply one event transaction. An error leaves the state slot unchanged.
    pub fn dispatch(&mut self, event: Event) -> Result<bool, String> {
        let result = self
            .call::<Update>("widget.event", Some(event))
            .and_then(|update| {
                if !update.handled && update.state != self.state {
                    return Err("unhandled widget event must not change state".into());
                }
                self.drawing_dirty |= update.handled;
                self.state = update.state;
                Ok(update.handled)
            });
        self.record_error(result.as_ref().err().cloned());
        result
    }
}
fn validate_drawing(drawing: &[Draw]) -> Result<(), String> {
    if drawing.len() > 4096 {
        return Err("widget drawing exceeds 4096 commands".into());
    }
    for command in drawing {
        let valid = match command {
            Draw::Rect { rect, .. } => {
                rect.iter().all(|v| v.is_finite())
                    && rect[2] >= 0.0
                    && rect[3] >= 0.0
                    && (rect[0] + rect[2]).is_finite()
                    && (rect[1] + rect[3]).is_finite()
            }
            Draw::Text {
                origin,
                text,
                font_size,
                ..
            } => {
                origin.iter().all(|v| v.is_finite())
                    && font_size.is_finite()
                    && *font_size > 0.0
                    && *font_size <= 4096.0
                    && text.len() <= 65536
            }
        };
        if !valid {
            return Err("invalid widget drawing command".into());
        }
    }
    Ok(())
}
impl Widget for ReloadableWidget {
    fn mounted(&mut self) -> WidgetChanges {
        self.poll_async_changes()
    }
    fn measure(&mut self, cx: &mut MeasureContext<'_>) -> Option<[f32; 2]> {
        match self.call::<Option<[f32; 2]>>("widget.measure", None) {
            Ok(size) if size.is_none_or(|s| s.iter().all(|v| v.is_finite() && *v >= 0.0)) => {
                self.measured = size
            }
            Ok(_) => self.error = Some("invalid widget measurement".into()),
            Err(error) => self.error = Some(error),
        }
        self.measured.map(|size| cx.resolve_size(size))
    }
    fn paint(&mut self, cx: &mut PaintContext<'_>) {
        if self.drawing_dirty || self.size != cx.size() {
            let _ = self.refresh_drawing(cx.size());
        }
        for command in &self.drawing {
            match command {
                Draw::Rect {
                    rect: [x, y, w, h],
                    color,
                } => cx.scene_mut().fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    Color::from_rgba8(color[0], color[1], color[2], color[3]),
                    None,
                    &Rect::new(*x, *y, x + w, y + h),
                ),
                Draw::Text {
                    origin,
                    text,
                    font_size,
                    color,
                } => {
                    let layout = wabou_shell::text::layout_text_styled(
                        cx.text(),
                        Arc::from(text.as_str()),
                        *font_size,
                        400.0,
                        false,
                        None,
                        wabou_shell::style::TextAlign::Start,
                        *color,
                        Arc::from([]),
                        None,
                        None,
                    );
                    cx.draw_text_layout(&layout, *origin);
                }
            }
        }
    }
    fn config_changed(&mut self, json: &str) -> Result<WidgetChanges, String> {
        self.config = serde_json::from_str(json).map_err(|e| e.to_string())?;
        self.drawing_dirty = true;
        Ok(WidgetChanges::REDRAW | WidgetChanges::MEASURE | WidgetChanges::LAYOUT)
    }
    fn config_removed(&mut self) -> WidgetChanges {
        self.config = Value::Null;
        self.drawing_dirty = true;
        WidgetChanges::REDRAW | WidgetChanges::MEASURE | WidgetChanges::LAYOUT
    }
    fn handle_event(&mut self, event: &UiEvent) -> WidgetEventResult {
        use wabou_shell::{PointerButton, PointerPhase};
        let event = match event {
            UiEvent::Pointer(p)
                if p.button == Some(PointerButton::Primary)
                    && matches!(p.phase, PointerPhase::Down | PointerPhase::Up) =>
            {
                Event::Pointer {
                    down: p.phase == PointerPhase::Down,
                    position: [p.position.x, p.position.y],
                }
            }
            UiEvent::Pointer(p) => {
                use wabou_capability_sdk::widget::PointerMotionPhase;
                match p.phase {
                    PointerPhase::Down | PointerPhase::Up => Event::PointerButton {
                        down: p.phase == PointerPhase::Down,
                        button: match p.button {
                            Some(PointerButton::Auxiliary) => 1,
                            Some(PointerButton::Secondary) => 2,
                            Some(PointerButton::Other(n)) => u32::from(n) + 3,
                            _ => return WidgetEventResult::IGNORED,
                        },
                        position: [p.position.x, p.position.y],
                        buttons: p.buttons,
                        modifiers: p.modifiers.bits(),
                    },
                    phase => Event::PointerMotion {
                        phase: match phase {
                            PointerPhase::Enter => PointerMotionPhase::Enter,
                            PointerPhase::Move => PointerMotionPhase::Move,
                            PointerPhase::Cancel => PointerMotionPhase::Cancel,
                            PointerPhase::Leave => PointerMotionPhase::Leave,
                            _ => unreachable!(),
                        },
                        position: [p.position.x, p.position.y],
                        buttons: p.buttons,
                        modifiers: p.modifiers.bits(),
                    },
                }
            }
            UiEvent::Wheel(w) => Event::Wheel {
                position: [w.position.x, w.position.y],
                delta: [w.delta_x, w.delta_y],
                modifiers: w.modifiers.bits(),
            },
            UiEvent::Key(k) => Event::Key {
                down: k.phase == wabou_shell::KeyPhase::Down,
                key: k.key.clone(),
                code: k.code.clone(),
                repeat: k.repeat,
                modifiers: k.modifiers.bits(),
            },
            UiEvent::TextInput(text) => Event::TextInput { text: text.clone() },
            UiEvent::Paste(text) => Event::Paste { text: text.clone() },
            UiEvent::Ime(event) => {
                use wabou_capability_sdk::widget::Ime;
                use wabou_shell::ImeEvent;
                Event::Ime {
                    event: match event {
                        ImeEvent::Enabled => Ime::Enabled,
                        ImeEvent::Disabled => Ime::Disabled,
                        ImeEvent::Preedit { text, cursor } => Ime::Preedit {
                            text: text.clone(),
                            cursor: *cursor,
                        },
                        ImeEvent::Commit(text) => Ime::Commit { text: text.clone() },
                        ImeEvent::DeleteSurrounding {
                            before_bytes,
                            after_bytes,
                        } => Ime::DeleteSurrounding {
                            before_bytes: *before_bytes,
                            after_bytes: *after_bytes,
                        },
                    },
                }
            }
            _ => return WidgetEventResult::IGNORED,
        };
        match self.dispatch(event) {
            Ok(true) => WidgetEventResult::handled_with(
                WidgetChanges::HANDLED
                    | WidgetChanges::REDRAW
                    | WidgetChanges::MEASURE
                    | WidgetChanges::LAYOUT,
            ),
            _ => WidgetEventResult::IGNORED,
        }
    }
    fn accepts_focus(&self) -> bool {
        true
    }
    fn accepts_text_input(&self) -> bool {
        self.call::<bool>("widget.textInput", None).unwrap_or(false)
    }
    fn focus_changed(&mut self, focused: bool) -> WidgetChanges {
        self.focused = focused;
        self.drawing_dirty = true;
        let _ = self.dispatch(Event::Focus { focused });
        WidgetChanges::REDRAW
    }
    fn set_wake_callback(&mut self, wake: WakeCallback) {
        self.module.subscribe_replacement(&wake);
        self.wake = Some(wake);
    }
    fn poll_async_changes(&mut self) -> WidgetChanges {
        if self.poll_async() {
            WidgetChanges::REDRAW | WidgetChanges::MEASURE | WidgetChanges::LAYOUT
        } else {
            WidgetChanges::empty()
        }
    }
    fn poll_async(&mut self) -> bool {
        let next = self.module.generation().ok().flatten();
        if next == self.generation {
            return false;
        }
        match self.module.snapshot() {
            Ok(active) => {
                self.generation = active.generation().ok().flatten();
                self.active = Some(active);
                self.drawing_dirty = true;
                true
            }
            Err(error) => {
                self.error = Some(error);
                false
            }
        }
    }
}
#[cfg(test)]
mod tests;
