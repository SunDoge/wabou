//! Typed input emitted by GPUI hit targets toward the Wabou runtime.

use std::rc::Rc;

use gpui::App;

use crate::NodeKey;

/// Pointer phase delivered by a projected GPUI element.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectedPointerPhase {
    /// The pointer moved over the target.
    Move,
    /// A pointer button was pressed.
    Down,
    /// A pointer button was released.
    Up,
}

/// Pointer button independent of GPUI and the guest event model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectedPointerButton {
    /// Primary/left button.
    Primary,
    /// Auxiliary/middle button.
    Auxiliary,
    /// Secondary/right button.
    Secondary,
    /// Navigation or otherwise backend-specific button.
    Other,
}

/// One pointer transition with an explicit retained target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectedPointerEvent {
    /// Deepest projected node selected by GPUI hit testing.
    pub target: NodeKey,
    /// Native transition phase.
    pub phase: ProjectedPointerPhase,
    /// Window-logical horizontal coordinate.
    pub x: f32,
    /// Window-logical vertical coordinate.
    pub y: f32,
    /// Button changed by this transition.
    pub button: Option<ProjectedPointerButton>,
    /// Whether Shift is held.
    pub shift: bool,
    /// Whether Control is held.
    pub control: bool,
    /// Whether Alt/Option is held.
    pub alt: bool,
    /// Whether Command/Super/Windows is held.
    pub platform: bool,
}

/// Native wheel phase independent of GPUI's platform representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectedWheelPhase {
    /// A precise gesture started.
    Started,
    /// The wheel or gesture changed.
    Changed,
    /// A precise gesture ended.
    Ended,
    /// The platform cancelled the gesture.
    Cancelled,
}

/// Wheel transition targeted by GPUI hit testing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectedWheelEvent {
    /// Deepest projected node selected by GPUI hit testing.
    pub target: NodeKey,
    /// Window-logical horizontal coordinate.
    pub x: f32,
    /// Window-logical vertical coordinate.
    pub y: f32,
    /// Horizontal delta in source units.
    pub delta_x: f32,
    /// Vertical delta in source units.
    pub delta_y: f32,
    /// Whether the source units are precise logical pixels rather than lines.
    pub precise: bool,
    /// Native gesture lifecycle.
    pub phase: ProjectedWheelPhase,
    /// Whether Shift is held.
    pub shift: bool,
    /// Whether Control is held.
    pub control: bool,
    /// Whether Alt/Option is held.
    pub alt: bool,
    /// Whether Command/Super/Windows is held.
    pub platform: bool,
}

/// Input transition emitted from one retained GPUI hit target.
#[derive(Clone, Debug, PartialEq)]
pub enum ProjectedInputEvent {
    /// Pointer movement or button transition.
    Pointer(ProjectedPointerEvent),
    /// Wheel or trackpad transition.
    Wheel(ProjectedWheelEvent),
    /// Keyboard transition delivered through the GPUI root focus handle.
    Key(ProjectedKeyEvent),
}

/// Keyboard phase independent of GPUI's event types.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectedKeyPhase {
    /// A key was pressed or repeated.
    Down,
    /// A key was released.
    Up,
}

/// Keyboard transition normalized at the GPUI shell boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedKeyEvent {
    /// Press or release phase.
    pub phase: ProjectedKeyPhase,
    /// GPUI's layout-independent key identity.
    pub key: String,
    /// Character produced by the active layout, when available.
    pub key_char: Option<String>,
    /// Whether this is an automatic repeat.
    pub repeat: bool,
    /// Whether Shift is held.
    pub shift: bool,
    /// Whether Control is held.
    pub control: bool,
    /// Whether Alt/Option is held.
    pub alt: bool,
    /// Whether Command/Super/Windows is held.
    pub platform: bool,
}

/// UI-thread callback installed while a projection is materialized.
pub type ProjectedInputSink = Rc<dyn Fn(ProjectedInputEvent, &mut App)>;
