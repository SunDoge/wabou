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

/// UI-thread callback installed while a projection is materialized.
pub type ProjectedInputSink = Rc<dyn Fn(ProjectedPointerEvent, &mut App)>;
