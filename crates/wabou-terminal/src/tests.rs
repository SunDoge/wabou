use super::*;

use wabou_shell::{ClipboardRequest, KeyEvent, KeyLocation, Modifiers, Point, PointerEvent};

fn pointer(phase: PointerPhase, x: f64, y: f64, buttons: u32) -> UiEvent {
    pointer_with_modifiers(phase, x, y, buttons, Modifiers::default())
}

fn pointer_with_modifiers(
    phase: PointerPhase,
    x: f64,
    y: f64,
    buttons: u32,
    modifiers: Modifiers,
) -> UiEvent {
    UiEvent::Pointer(PointerEvent {
        phase,
        position: Point { x, y },
        button: Some(PointerButton::Primary),
        buttons,
        modifiers,
    })
}

mod input_cases;
mod lifecycle_cases;
mod selection_cases;
