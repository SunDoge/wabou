//! Pure winit → Wabou event translation helpers.
//!
//! Everything in this module is a side-effect-free mapping from winit types
//! (`TouchPhase`, `PointerSource`, `Key`, `ModifiersState`, …) to the
//! host-agnostic [`crate::source`] vocabulary, plus the frame-wake and
//! custom-decoration geometry decisions that depend only on their arguments.
//! Keeping them free of `App` state makes them directly unit-testable.

use std::time::{Duration, Instant};

use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{ButtonSource, DeviceId, MouseButton, TabletToolData, TouchPhase};
use winit::event::{PointerKind as WinitPointerKind, PointerSource};
use winit::keyboard::{Key, KeyLocation as WinitKeyLocation, ModifiersState};
use winit::raw_window_handle::RawWindowHandle;
use winit::window::{ImeHint, ImePurpose as WinitImePurpose, ResizeDirection};

use crate::source::{
    EventResponse, GesturePhase, KeyLocation, Modifiers, Point, PointerButton, PointerId,
    PointerProperties, PointerType,
};
use crate::window_lifecycle::WindowCapabilities;

pub(super) fn ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

pub(super) const fn gesture_phase(phase: TouchPhase) -> GesturePhase {
    match phase {
        TouchPhase::Started => GesturePhase::Started,
        TouchPhase::Moved => GesturePhase::Changed,
        TouchPhase::Ended => GesturePhase::Ended,
        TouchPhase::Cancelled => GesturePhase::Cancelled,
    }
}

pub(super) fn allowed_external_url(raw: &str) -> Option<url::Url> {
    let url = url::Url::parse(raw).ok()?;
    matches!(url.scheme(), "http" | "https").then_some(url)
}

pub(super) fn update_present_retry(presented: bool, retry_pending: &mut bool) -> bool {
    if presented {
        *retry_pending = false;
        false
    } else if !*retry_pending {
        *retry_pending = true;
        true
    } else {
        false
    }
}

pub(super) fn window_capabilities(handle: Option<RawWindowHandle>) -> WindowCapabilities {
    WindowCapabilities {
        mutable_visibility: !matches!(handle, Some(RawWindowHandle::Wayland(_))),
    }
}

// Undecorated windows have no compositor-drawn border to grab. Keep this in
// logical pixels so the target remains usable at every display scale; the
// outer few pixels are otherwise surprisingly difficult to hit on touchpads.
pub(super) const CUSTOM_DECORATION_RESIZE_BORDER: f64 = 12.0;

pub(super) fn resize_direction_at(
    position: Point,
    width: f64,
    height: f64,
    border: f64,
) -> Option<ResizeDirection> {
    if width <= 0.0
        || height <= 0.0
        || position.x < 0.0
        || position.y < 0.0
        || position.x > width
        || position.y > height
    {
        return None;
    }
    let border = border.max(0.0).min(width * 0.5).min(height * 0.5);
    let west = position.x <= border;
    let east = position.x >= width - border;
    let north = position.y <= border;
    let south = position.y >= height - border;
    match (west, east, north, south) {
        (true, _, true, _) => Some(ResizeDirection::NorthWest),
        (_, true, true, _) => Some(ResizeDirection::NorthEast),
        (true, _, _, true) => Some(ResizeDirection::SouthWest),
        (_, true, _, true) => Some(ResizeDirection::SouthEast),
        (true, _, _, _) => Some(ResizeDirection::West),
        (_, true, _, _) => Some(ResizeDirection::East),
        (_, _, true, _) => Some(ResizeDirection::North),
        (_, _, _, true) => Some(ResizeDirection::South),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FrameWake {
    Idle,
    Redraw,
    Deadline(Instant),
}

pub(super) fn frame_wake(
    has_animation: bool,
    deadline: Option<Instant>,
    now: Instant,
) -> FrameWake {
    if has_animation || deadline.is_some_and(|deadline| deadline <= now) {
        FrameWake::Redraw
    } else {
        deadline.map_or(FrameWake::Idle, FrameWake::Deadline)
    }
}

pub(super) fn scheduled_frame_wake(
    has_animation: bool,
    reported_deadline: Option<Instant>,
    scheduled_deadline: &mut Option<Instant>,
    now: Instant,
) -> FrameWake {
    if has_animation {
        *scheduled_deadline = None;
        return FrameWake::Redraw;
    }
    let Some(reported_deadline) = reported_deadline else {
        *scheduled_deadline = None;
        return FrameWake::Idle;
    };
    let deadline = scheduled_deadline
        .map(|scheduled| scheduled.min(reported_deadline))
        .unwrap_or(reported_deadline);
    let wake = frame_wake(false, Some(deadline), now);
    if wake == FrameWake::Redraw {
        *scheduled_deadline = None;
    } else {
        *scheduled_deadline = Some(deadline);
    }
    wake
}

pub(super) fn pointer_button(button: &ButtonSource) -> PointerButton {
    match button {
        ButtonSource::Mouse(MouseButton::Left) => PointerButton::Primary,
        ButtonSource::Mouse(MouseButton::Middle) => PointerButton::Auxiliary,
        ButtonSource::Mouse(MouseButton::Right) => PointerButton::Secondary,
        ButtonSource::Mouse(MouseButton::Back) => PointerButton::Other(3),
        ButtonSource::Mouse(MouseButton::Forward) => PointerButton::Other(4),
        _ => PointerButton::Primary,
    }
}

pub(super) fn button_mask(button: PointerButton) -> u32 {
    match button {
        PointerButton::Primary => 1,
        PointerButton::Auxiliary => 2,
        PointerButton::Secondary => 4,
        PointerButton::Other(index) if index < u32::BITS as u16 => 1 << index,
        PointerButton::Other(_) => 0,
    }
}

pub(super) fn namespaced_pointer_id(namespace: u32, raw: u64) -> PointerId {
    PointerId {
        lo: raw as u32,
        hi: ((raw >> 32) as u32) ^ namespace.rotate_left(29),
    }
}

pub(super) fn device_pointer_id(namespace: u32, device_id: Option<DeviceId>) -> PointerId {
    namespaced_pointer_id(namespace, device_id.map_or(0, |id| id.into_raw() as u64))
}

fn tablet_properties(
    device_id: Option<DeviceId>,
    primary: bool,
    data: &TabletToolData,
) -> PointerProperties {
    let angle = data.clone().angle();
    let tilt = data.clone().tilt();
    PointerProperties {
        id: device_pointer_id(2, device_id),
        pointer_type: PointerType::Pen,
        primary,
        pressure: data.force.as_ref().map(|force| force.normalized(angle)),
        tangential_pressure: data.tangential_force.map(f64::from),
        tilt_x: tilt.map(|tilt| f64::from(tilt.x)),
        tilt_y: tilt.map(|tilt| f64::from(tilt.y)),
        twist: data.twist.map(f64::from),
    }
}

pub(super) fn pointer_source_properties(
    device_id: Option<DeviceId>,
    primary: bool,
    source: &PointerSource,
) -> PointerProperties {
    match source {
        PointerSource::Mouse => PointerProperties {
            id: namespaced_pointer_id(0, 1),
            pointer_type: PointerType::Mouse,
            primary,
            ..PointerProperties::default()
        },
        PointerSource::Touch { finger_id, force } => PointerProperties {
            id: namespaced_pointer_id(1, finger_id.into_raw() as u64),
            pointer_type: PointerType::Touch,
            primary,
            pressure: force.as_ref().map(|force| force.normalized(None)),
            ..PointerProperties::default()
        },
        PointerSource::TabletTool { data, .. } => tablet_properties(device_id, primary, data),
        PointerSource::Unknown => PointerProperties {
            id: device_pointer_id(3, device_id),
            pointer_type: PointerType::Unknown,
            primary,
            ..PointerProperties::default()
        },
    }
}

pub(super) fn pointer_kind_properties(
    device_id: Option<DeviceId>,
    primary: bool,
    kind: WinitPointerKind,
) -> PointerProperties {
    match kind {
        WinitPointerKind::Mouse => PointerProperties {
            id: namespaced_pointer_id(0, 1),
            pointer_type: PointerType::Mouse,
            primary,
            ..PointerProperties::default()
        },
        WinitPointerKind::Touch(finger_id) => PointerProperties {
            id: namespaced_pointer_id(1, finger_id.into_raw() as u64),
            pointer_type: PointerType::Touch,
            primary,
            ..PointerProperties::default()
        },
        WinitPointerKind::TabletTool(_) => PointerProperties {
            id: device_pointer_id(2, device_id),
            pointer_type: PointerType::Pen,
            primary,
            ..PointerProperties::default()
        },
        WinitPointerKind::Unknown => PointerProperties {
            id: device_pointer_id(3, device_id),
            pointer_type: PointerType::Unknown,
            primary,
            ..PointerProperties::default()
        },
    }
}

pub(super) fn button_source_properties(
    device_id: Option<DeviceId>,
    primary: bool,
    source: &ButtonSource,
) -> PointerProperties {
    match source {
        ButtonSource::Mouse(_) => PointerProperties {
            id: namespaced_pointer_id(0, 1),
            pointer_type: PointerType::Mouse,
            primary,
            ..PointerProperties::default()
        },
        ButtonSource::Touch { finger_id, force } => PointerProperties {
            id: namespaced_pointer_id(1, finger_id.into_raw() as u64),
            pointer_type: PointerType::Touch,
            primary,
            pressure: force.as_ref().map(|force| force.normalized(None)),
            ..PointerProperties::default()
        },
        ButtonSource::TabletTool { data, .. } => tablet_properties(device_id, primary, data),
        ButtonSource::Unknown(_) => PointerProperties {
            id: device_pointer_id(3, device_id),
            pointer_type: PointerType::Unknown,
            primary,
            ..PointerProperties::default()
        },
    }
}

pub(super) fn modifiers(state: ModifiersState) -> Modifiers {
    let mut modifiers = Modifiers::empty();
    modifiers.set(Modifiers::SHIFT, state.shift_key());
    modifiers.set(Modifiers::CONTROL, state.control_key());
    modifiers.set(Modifiers::ALT, state.alt_key());
    modifiers.set(Modifiers::META, state.meta_key());
    modifiers
}

pub(super) fn logical_key_name(key: &Key) -> String {
    match key {
        Key::Named(named) => format!("{named:?}"),
        other => other
            .to_text()
            .map_or_else(|| format!("{other:?}"), str::to_owned),
    }
}

pub(super) fn key_location(location: WinitKeyLocation) -> KeyLocation {
    match location {
        WinitKeyLocation::Standard => KeyLocation::Standard,
        WinitKeyLocation::Left => KeyLocation::Left,
        WinitKeyLocation::Right => KeyLocation::Right,
        WinitKeyLocation::Numpad => KeyLocation::Numpad,
    }
}

pub(super) fn printable_key_text(text: &str) -> Option<String> {
    let text: String = text
        .chars()
        .filter(|character| !character.is_control())
        .collect();
    (!text.is_empty()).then_some(text)
}

pub(super) fn committed_key_text(
    text: Option<&str>,
    text_with_all_modifiers: Option<&str>,
    modifiers: Modifiers,
    ime_composing: bool,
) -> Option<String> {
    // Windows and Linux can continue reporting physical keyboard events
    // while an IME owns the text stream. Only Ime::Commit may insert text
    // until preedit ends, otherwise phonetic input can be inserted twice.
    if ime_composing {
        return None;
    }
    if modifiers.control() && modifiers.alt() && !modifiers.meta() {
        text_with_all_modifiers.and_then(printable_key_text)
    } else if !modifiers.control() && !modifiers.meta() {
        text.and_then(printable_key_text)
    } else {
        None
    }
}

pub(super) fn unconsumed_key_text(
    text: Option<String>,
    response: &EventResponse,
) -> Option<String> {
    text.filter(|_| !response.consume_key_text)
}

pub(super) fn ime_cursor_rect(area: Option<[f64; 4]>) -> (LogicalPosition<f64>, LogicalSize<f64>) {
    let [x0, y0, x1, y1] = area.unwrap_or([0.0, 0.0, 1.0, 1.0]);
    (
        LogicalPosition::new(x0, y0),
        LogicalSize::new((x1 - x0).max(1.0), (y1 - y0).max(1.0)),
    )
}

pub(super) fn ime_hint_and_purpose(state: Option<&crate::ImeState>) -> (ImeHint, WinitImePurpose) {
    let Some(state) = state else {
        return (ImeHint::NONE, WinitImePurpose::Normal);
    };
    let mut hint = ImeHint::NONE;
    if state.multiline {
        hint |= ImeHint::MULTILINE;
    }
    if state.completion {
        hint |= ImeHint::COMPLETION;
    }
    if state.spellcheck {
        hint |= ImeHint::SPELLCHECK;
    }
    let purpose = match state.purpose {
        crate::WidgetImePurpose::Normal => WinitImePurpose::Normal,
        crate::WidgetImePurpose::Password => WinitImePurpose::Password,
        crate::WidgetImePurpose::Terminal => WinitImePurpose::Terminal,
        crate::WidgetImePurpose::Number => WinitImePurpose::Number,
        crate::WidgetImePurpose::Phone => WinitImePurpose::Phone,
        crate::WidgetImePurpose::Url => WinitImePurpose::Url,
        crate::WidgetImePurpose::Email => WinitImePurpose::Email,
        crate::WidgetImePurpose::Pin => WinitImePurpose::Pin,
        crate::WidgetImePurpose::Date => WinitImePurpose::Date,
        crate::WidgetImePurpose::Time => WinitImePurpose::Time,
        crate::WidgetImePurpose::DateTime => WinitImePurpose::DateTime,
    };
    (hint, purpose)
}

pub(super) fn ime_client_changed(
    previous: Option<&crate::ImeState>,
    next: Option<&crate::ImeState>,
) -> bool {
    matches!((previous, next), (Some(previous), Some(next)) if previous.client_id != next.client_id)
}
