//! Shared resident widget adapter for directly linked implementations.
//! The resident object owns its JSON state, focus, config and last valid drawing.
//! Malformed or failed callbacks cannot replace state or drawing.
use crate::PaintScene;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use vello_common::{
    kurbo::{Affine, Circle, Line, Rect, Stroke},
    peniko::{Color, Fill},
};
use wabou_shell::{
    MeasureContext, PaintContext, UiEvent, WakeCallback, Widget, WidgetChanges, WidgetEventResult,
};
use wabou_shell_api::widget_implementation::{
    WidgetDraw as Draw, WidgetEvent as Event, WidgetEventReply as EventReply, WidgetImplementation,
    WidgetInputs as Inputs, WidgetInvalidation as EventInvalidation,
    WidgetPresentation as Presentation, WidgetUpdate as Update,
};

/// Runtime-owned widget instance; construct once per stable native node.
pub struct WidgetAdapter {
    implementation: Implementation,
    inputs: Inputs,
    drawing: Vec<Draw>,
    drawing_dirty: bool,
    measured: Option<[f32; 2]>,
    error: Option<String>,
    wake: Option<WakeCallback>,
    presentation: Presentation,
    resources: HashMap<ResourceSpec, ResidentResource>,
    prepared: Vec<Option<ResidentResource>>,
    resource_pool: Arc<super::HostResourcePool>,
    event_invalidation: EventInvalidation,
}
#[derive(Clone, PartialEq, Eq, Hash)]
enum ResourceSpec {
    Image {
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    },
    Svg(String),
    Shader {
        source: String,
        module: bool,
        instance: usize,
    },
}
impl ResourceSpec {
    fn from_draw(draw: &Draw, instance: usize) -> Option<Self> {
        match draw {
            Draw::Image {
                width,
                height,
                rgba,
                ..
            } => Some(Self::Image {
                width: *width,
                height: *height,
                rgba: rgba.clone(),
            }),
            Draw::Svg { source, .. } => Some(Self::Svg(source.clone())),
            Draw::Shader { source, module, .. } => Some(Self::Shader {
                source: source.clone(),
                module: *module,
                instance,
            }),
            _ => None,
        }
    }
    fn prepare(&self) -> Result<ResidentResource, String> {
        match self {
            Self::Image {
                width,
                height,
                rgba,
            } => crate::WidgetRasterImage::from_rgba8(*width, *height, rgba.clone())
                .map(ResidentResource::Image)
                .map_err(|error| error.to_string()),
            Self::Svg(source) => crate::svg::SvgImage::parse(source)
                .map(ResidentResource::Svg)
                .map_err(|error| error.to_string()),
            Self::Shader { source, module, .. } => {
                let source = if *module {
                    crate::ShaderEffectSource::new_module(source)
                } else {
                    crate::ShaderEffectSource::new(source)
                }?;
                Ok(ResidentResource::Shader {
                    source,
                    id: crate::ShaderEffectId::new(),
                })
            }
        }
    }
}
#[derive(Clone)]
enum ResidentResource {
    Image(crate::WidgetRasterImage),
    Svg(crate::svg::SvgImage),
    Shader {
        source: crate::ShaderEffectSource,
        id: crate::ShaderEffectId,
    },
}
struct StaticCallbacks {
    measure: fn(&Inputs) -> Result<Option<[f32; 2]>, String>,
    paint: fn(&Inputs) -> Result<Vec<Draw>, String>,
    event: fn(&Inputs, Event) -> Result<Update, String>,
    presentation: fn(&Inputs) -> Result<Presentation, String>,
    event_invalidation: EventInvalidation,
}
enum Implementation {
    Static(StaticCallbacks),
}
fn invoke_static<T>(callback: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback))
        .unwrap_or_else(|_| Err("capability module panicked".into()))
}
impl Implementation {
    fn measure(&self, inputs: &Inputs) -> Result<Option<[f32; 2]>, String> {
        match self {
            Self::Static(callbacks) => invoke_static(|| (callbacks.measure)(inputs)),
        }
    }
    fn paint(&self, inputs: &Inputs) -> Result<Vec<Draw>, String> {
        match self {
            Self::Static(callbacks) => invoke_static(|| (callbacks.paint)(inputs)),
        }
    }
    fn event(&self, inputs: &Inputs, event: Event) -> Result<EventReply, String> {
        match self {
            Self::Static(callbacks) => {
                invoke_static(|| (callbacks.event)(inputs, event)).map(|update| EventReply {
                    update,
                    presentation: None,
                    invalidation: callbacks.event_invalidation,
                })
            }
        }
    }
    fn presentation(&self, inputs: &Inputs) -> Result<Presentation, String> {
        match self {
            Self::Static(callbacks) => invoke_static(|| (callbacks.presentation)(inputs)),
        }
    }
}
impl WidgetAdapter {
    /// Register a statically linked implementation.
    pub fn from_implementation<W: WidgetImplementation>(state: Value) -> Self {
        Self::with_implementation(
            Implementation::Static(StaticCallbacks {
                measure: W::measure,
                paint: W::paint,
                event: W::event,
                presentation: W::presentation,
                event_invalidation: W::EVENT_INVALIDATION,
            }),
            state,
        )
    }
    fn with_implementation(implementation: Implementation, state: Value) -> Self {
        let mut widget = Self {
            implementation,
            inputs: Inputs {
                state,
                config: Value::Null,
                size: [0.0; 2],
                focused: false,
            },
            drawing: Vec::new(),
            drawing_dirty: true,
            measured: None,
            error: None,
            wake: None,
            presentation: Presentation::default(),
            resources: HashMap::new(),
            prepared: Vec::new(),
            resource_pool: Arc::new(super::HostResourcePool::default()),
            event_invalidation: EventInvalidation::Layout,
        };
        let _ = widget.refresh_presentation();
        widget
    }
    /// Use the application's resident resource pool. Cached frames pin their
    /// pixel buffers; future image handles are resolved against this pool only.
    pub fn with_resource_pool(mut self, pool: Arc<super::HostResourcePool>) -> Self {
        self.resource_pool = pool;
        self.drawing_dirty = true;
        self
    }
    /// Borrow durable state without crossing the dynamic-library boundary.
    pub fn state(&self) -> &Value {
        &self.inputs.state
    }
    /// Resident focus survives implementation replacement.
    pub fn focused(&self) -> bool {
        self.inputs.focused
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
    fn inputs(&self) -> &Inputs {
        &self.inputs
    }
    fn presentation_changes(&self, previous: &Presentation) -> WidgetChanges {
        let mut changes = WidgetChanges::empty();
        if self.presentation.value != previous.value {
            changes |= WidgetChanges::VALUE;
        }
        if self.presentation.selection != previous.selection {
            changes |= WidgetChanges::SELECTION;
        }
        if self.presentation.accessibility != previous.accessibility {
            changes |= WidgetChanges::SEMANTICS;
        }
        changes
    }
    fn refresh_presentation(&mut self) -> Result<(), String> {
        let result = self
            .implementation
            .presentation(self.inputs())
            .and_then(|presentation| {
                validate_presentation(&presentation)?;
                self.presentation = presentation;
                Ok(())
            });
        self.record_error(result.as_ref().err().cloned());
        result
    }
    /// Invoke the current paint implementation without a native window.
    pub fn refresh_drawing(&mut self, size: [f32; 2]) -> Result<(), String> {
        self.inputs.size = size;
        let result = self
            .implementation
            .paint(self.inputs())
            .and_then(|drawing| {
                validate_drawing(&drawing)?;
                let presentation = self.implementation.presentation(self.inputs())?;
                validate_presentation(&presentation)?;
                let mut resources = HashMap::new();
                let mut prepared = Vec::with_capacity(drawing.len());
                let mut shader_instances = HashMap::new();
                for command in &drawing {
                    let instance = if let Draw::Shader { source, module, .. } = command {
                        let count = shader_instances
                            .entry((source.as_str(), *module))
                            .or_insert(0);
                        let instance = *count;
                        *count += 1;
                        instance
                    } else {
                        0
                    };
                    let resource = if let Draw::ImageRef { image, .. } = command {
                        Some(ResidentResource::Image(self.resource_pool.image(*image)?))
                    } else if let Some(spec) = ResourceSpec::from_draw(command, instance) {
                        let resource =
                            match resources.get(&spec).or_else(|| self.resources.get(&spec)) {
                                Some(resource) => resource.clone(),
                                None => spec.prepare()?,
                            };
                        resources.insert(spec, resource.clone());
                        Some(resource)
                    } else {
                        None
                    };
                    prepared.push(resource);
                }
                self.presentation = presentation;
                self.resources = resources;
                self.prepared = prepared;
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
            .implementation
            .event(self.inputs(), event)
            .and_then(|reply| {
                let EventReply {
                    update,
                    presentation,
                    invalidation,
                } = reply;
                if !update.handled && update.state != self.inputs.state {
                    return Err("unhandled widget event must not change state".into());
                }
                if let Some(presentation) = presentation {
                    validate_presentation(&presentation)?;
                    self.inputs.state = update.state;
                    self.presentation = presentation;
                    self.drawing_dirty |= update.handled;
                    self.event_invalidation = invalidation;
                    return Ok(update.handled);
                }
                let previous = std::mem::replace(&mut self.inputs.state, update.state);
                let result =
                    self.implementation
                        .presentation(self.inputs())
                        .and_then(|presentation| {
                            validate_presentation(&presentation)?;
                            Ok(presentation)
                        });
                match result {
                    Ok(presentation) => {
                        self.drawing_dirty |= update.handled;
                        self.event_invalidation = invalidation;
                        self.presentation = presentation;
                        Ok(update.handled)
                    }
                    Err(error) => {
                        self.inputs.state = previous;
                        Err(error)
                    }
                }
            });
        self.record_error(result.as_ref().err().cloned());
        result
    }
}
fn validate_presentation(presentation: &Presentation) -> Result<(), String> {
    if let Some(ime) = &presentation.ime
        && (!ime.cursor_area.iter().all(|value| value.is_finite())
            || ime.cursor_area[2] < 0.0
            || ime.cursor_area[3] < 0.0
            || !ime
                .surrounding_text
                .is_char_boundary(ime.surrounding_cursor)
            || !ime
                .surrounding_text
                .is_char_boundary(ime.surrounding_anchor)
            || ime.selection_utf16.start > ime.selection_utf16.end
            || ime
                .marked_range_utf16
                .as_ref()
                .is_some_and(|range| range.start > range.end))
    {
        return Err("invalid native widget IME snapshot".into());
    }
    Ok(())
}
fn valid_rect(rect: &[f64; 4]) -> bool {
    rect.iter().all(|v| v.is_finite())
        && rect[2] >= 0.0
        && rect[3] >= 0.0
        && (rect[0] + rect[2]).is_finite()
        && (rect[1] + rect[3]).is_finite()
}
fn validate_drawing(drawing: &[Draw]) -> Result<(), String> {
    if drawing.len() > 4096 {
        return Err("widget drawing exceeds 4096 commands".into());
    }
    let mut resource_bytes = 0usize;
    for command in drawing {
        resource_bytes = resource_bytes.saturating_add(match command {
            Draw::Image { rgba, .. } => rgba.len(),
            Draw::Svg { source, .. } | Draw::Shader { source, .. } => source.len(),
            _ => 0,
        });
        if resource_bytes > 32 * 1024 * 1024 {
            return Err("widget resource data exceeds 32 MiB".into());
        }
        let valid = match command {
            Draw::Rect { rect, .. } => {
                rect.iter().all(|v| v.is_finite())
                    && rect[2] >= 0.0
                    && rect[3] >= 0.0
                    && (rect[0] + rect[2]).is_finite()
                    && (rect[1] + rect[3]).is_finite()
            }
            Draw::Circle { center, radius, .. } => {
                center.iter().all(|v| v.is_finite()) && radius.is_finite() && *radius >= 0.0
            }
            Draw::Line {
                from, to, width, ..
            } => from.iter().chain(to).all(|v| v.is_finite()) && width.is_finite() && *width >= 0.0,
            Draw::Image {
                rect,
                width,
                height,
                rgba,
            } => {
                valid_rect(rect)
                    && *width > 0
                    && *height > 0
                    && rgba.len() <= 4 * 1024 * 1024
                    && (u64::from(*width) * u64::from(*height)).checked_mul(4)
                        == Some(rgba.len() as u64)
            }
            Draw::ImageRef { rect, .. } => valid_rect(rect),
            Draw::Svg { rect, source } => valid_rect(rect) && source.len() <= 1024 * 1024,
            Draw::Shader {
                source,
                time,
                values,
                ..
            } => {
                source.len() <= 1024 * 1024
                    && time.is_finite()
                    && values.len() <= 256
                    && values.iter().all(|v| v.is_finite())
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
impl Widget for WidgetAdapter {
    fn mounted(&mut self) -> WidgetChanges {
        self.poll_async_changes()
    }
    fn measure(&mut self, cx: &mut MeasureContext<'_>) -> Option<[f32; 2]> {
        match self.implementation.measure(self.inputs()) {
            Ok(size) if size.is_none_or(|s| s.iter().all(|v| v.is_finite() && *v >= 0.0)) => {
                self.measured = size
            }
            Ok(_) => self.error = Some("invalid widget measurement".into()),
            Err(error) => self.error = Some(error),
        }
        self.measured.map(|size| cx.resolve_size(size))
    }
    fn paint(&mut self, cx: &mut PaintContext<'_>) {
        if self.drawing_dirty || self.inputs.size != cx.size() {
            let _ = self.refresh_drawing(cx.size());
        }
        for (index, command) in self.drawing.iter().enumerate() {
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
                Draw::Circle {
                    center,
                    radius,
                    color,
                } => cx.scene_mut().fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    Color::from_rgba8(color[0], color[1], color[2], color[3]),
                    None,
                    &Circle::new((center[0], center[1]), *radius),
                ),
                Draw::Line {
                    from,
                    to,
                    width,
                    color,
                } => cx.scene_mut().stroke(
                    &Stroke::new(*width),
                    Affine::IDENTITY,
                    Color::from_rgba8(color[0], color[1], color[2], color[3]),
                    None,
                    &Line::new((from[0], from[1]), (to[0], to[1])),
                ),
                Draw::Image { rect, .. } | Draw::ImageRef { rect, .. } => {
                    if let Some(ResidentResource::Image(image)) = &self.prepared[index] {
                        cx.draw_raster_image_in(image, *rect);
                    }
                }
                Draw::Svg {
                    rect: [x, y, width, height],
                    ..
                } => {
                    if let Some(ResidentResource::Svg(svg)) = &self.prepared[index] {
                        let size = svg.size();
                        let transform = Affine::translate((*x, *y))
                            * Affine::scale_non_uniform(
                                *width / f64::from(size[0]),
                                *height / f64::from(size[1]),
                            );
                        cx.scene_mut().draw_svg(svg.document().clone(), transform);
                    }
                }
                Draw::Shader { time, values, .. } => {
                    if let Some(ResidentResource::Shader { id, source }) = &self.prepared[index] {
                        cx.draw_shader_effect(*id, source.clone(), *time, values.clone().into());
                    }
                }
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
        let config = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let previous_config = std::mem::replace(&mut self.inputs.config, config);
        let result = self
            .implementation
            .presentation(self.inputs())
            .and_then(|presentation| {
                validate_presentation(&presentation)?;
                Ok(presentation)
            });
        let presentation = match result {
            Ok(presentation) => presentation,
            Err(error) => {
                self.inputs.config = previous_config;
                return Err(error);
            }
        };
        let previous = std::mem::replace(&mut self.presentation, presentation);
        self.drawing_dirty = true;
        Ok(WidgetChanges::REDRAW
            | WidgetChanges::MEASURE
            | WidgetChanges::LAYOUT
            | self.presentation_changes(&previous))
    }
    fn config_removed(&mut self) -> WidgetChanges {
        let previous = self.presentation.clone();
        self.inputs.config = Value::Null;
        let _ = self.refresh_presentation();
        self.drawing_dirty = true;
        WidgetChanges::REDRAW
            | WidgetChanges::MEASURE
            | WidgetChanges::LAYOUT
            | self.presentation_changes(&previous)
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
                use wabou_shell_api::widget_implementation::PointerMotionPhase;
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
                use wabou_shell::ImeEvent;
                use wabou_shell_api::widget_implementation::WidgetIme as Ime;
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
        let previous = self.presentation.clone();
        match self.dispatch(event) {
            Ok(true) => {
                let mut changes = WidgetChanges::REDRAW | self.presentation_changes(&previous);
                if self.event_invalidation == EventInvalidation::Layout {
                    changes |= WidgetChanges::MEASURE | WidgetChanges::LAYOUT;
                }
                WidgetEventResult::handled_with(changes)
            }
            _ => WidgetEventResult::IGNORED,
        }
    }
    fn accepts_focus(&self) -> bool {
        self.presentation.focusable
    }
    fn accepts_text_input(&self) -> bool {
        self.presentation.text_input
    }
    fn current_value(&self) -> Option<&str> {
        self.presentation.value.as_deref()
    }
    fn text_selection(&self) -> Option<wabou_shell::WidgetTextSelection> {
        self.presentation.selection.clone()
    }
    fn accessibility(&self) -> wabou_shell::WidgetAccessibility {
        self.presentation.accessibility.clone()
    }
    fn ime_state(&self) -> Option<wabou_shell::WidgetImeState> {
        self.presentation.ime.clone()
    }
    fn ime_cursor_area(&self) -> Option<[f32; 4]> {
        self.presentation.ime.as_ref().map(|ime| ime.cursor_area)
    }
    fn focus_changed(&mut self, focused: bool) -> WidgetChanges {
        let previous = self.presentation.clone();
        self.inputs.focused = focused;
        self.drawing_dirty = true;
        let _ = self.dispatch(Event::Focus { focused });
        WidgetChanges::REDRAW | WidgetChanges::SEMANTICS | self.presentation_changes(&previous)
    }
    fn set_wake_callback(&mut self, wake: WakeCallback) {
        self.wake = Some(wake);
    }
    fn poll_async_changes(&mut self) -> WidgetChanges {
        let previous = self.presentation.clone();
        if self.poll_async() {
            WidgetChanges::REDRAW
                | WidgetChanges::MEASURE
                | WidgetChanges::LAYOUT
                | self.presentation_changes(&previous)
        } else {
            WidgetChanges::empty()
        }
    }
    fn poll_async(&mut self) -> bool {
        false
    }
}

#[cfg(test)]
mod static_tests {
    use super::*;
    use serde_json::json;

    struct Counter;
    impl WidgetImplementation for Counter {
        fn measure(_: &Inputs) -> Result<Option<[f32; 2]>, String> {
            Ok(Some([120.0, 40.0]))
        }
        fn paint(inputs: &Inputs) -> Result<Vec<Draw>, String> {
            let count = inputs.state.as_u64().ok_or("expected count")?;
            if inputs.config.get("invalidDrawing").and_then(Value::as_bool) == Some(true) {
                return Ok(vec![Draw::Rect {
                    rect: [0.0, 0.0, -1.0, 5.0],
                    color: [0; 4],
                }]);
            }
            Ok(vec![Draw::Text {
                origin: [0.0, 0.0],
                text: count.to_string(),
                font_size: 14.0,
                color: [255; 4],
            }])
        }
        fn event(inputs: &Inputs, event: Event) -> Result<Update, String> {
            let count = inputs.state.as_u64().ok_or("expected count")?;
            match event {
                Event::Pointer { down: true, .. } => Ok(Update {
                    state: json!(count + 1),
                    handled: true,
                }),
                Event::Key { .. } => Ok(Update {
                    state: json!(count + 1),
                    handled: false,
                }),
                _ => Ok(Update {
                    state: inputs.state.clone(),
                    handled: false,
                }),
            }
        }
    }
    #[test]
    fn directly_linked_implementation_has_resident_identity_and_transactional_errors() {
        let mut widget = WidgetAdapter::from_implementation::<Counter>(json!(40));
        let address = widget.state() as *const Value;
        widget.focus_changed(true);
        widget
            .dispatch(Event::Pointer {
                down: true,
                position: [1.0, 1.0],
            })
            .unwrap();
        assert_eq!(widget.state(), &json!(41));
        assert_eq!(widget.state() as *const Value, address);
        widget.refresh_drawing([120.0, 40.0]).unwrap();
        let drawing = widget.drawing().to_vec();
        assert!(
            widget
                .dispatch(Event::Key {
                    down: true,
                    key: "Enter".into(),
                    code: "Enter".into(),
                    repeat: false,
                    modifiers: 0
                })
                .is_err()
        );
        assert_eq!(widget.state(), &json!(41));
        widget.config_changed(r#"{"invalidDrawing":true}"#).unwrap();
        assert!(widget.refresh_drawing([120.0, 40.0]).is_err());
        assert_eq!(widget.drawing(), drawing);
        assert!(widget.focused());
        assert!(!widget.poll_async());
    }
}
