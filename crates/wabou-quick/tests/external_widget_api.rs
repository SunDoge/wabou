use std::collections::VecDeque;
use std::time::Instant;

use wabou_quick::widget_api::*;

struct ExternalWidget {
    wake: Option<WakeCallback>,
    actions: VecDeque<HostAction>,
    events: VecDeque<WidgetNodeEvent>,
}

impl ExternalWidget {
    fn new() -> Self {
        Self {
            wake: None,
            actions: VecDeque::new(),
            events: VecDeque::new(),
        }
    }
}

impl Widget for ExternalWidget {
    fn paint(&mut self, _cx: &mut PaintContext<'_>) {}

    fn handle_event(&mut self, _event: &UiEvent) -> WidgetEventResult {
        WidgetEventResult::HANDLED
    }

    fn style_changed(&mut self, _style: &WidgetStyle) {}

    fn animation_deadline(&self) -> Option<Instant> {
        None
    }

    fn set_wake_callback(&mut self, wake: WakeCallback) {
        self.wake = Some(wake);
    }

    fn take_host_action(&mut self) -> Option<HostAction> {
        self.actions.pop_front()
    }

    fn complete_host_action(&mut self, _result: HostActionResult) {}

    fn take_node_event(&mut self) -> Option<WidgetNodeEvent> {
        self.events.pop_front()
    }
}

#[test]
fn external_widget_can_use_the_complete_public_sdk() {
    let _clipboard = ClipboardRequest::Read;
    let _changes = WidgetChanges::HANDLED | WidgetChanges::REDRAW;
    let _pointer = UiEvent::Pointer(PointerEvent {
        phase: PointerPhase::Move,
        position: Point { x: 1.0, y: 2.0 },
        button: Some(PointerButton::Primary),
        buttons: 1,
        modifiers: Modifiers::empty(),
    });
    let _window_action = HostAction::CreateWindow {
        window_id: 2,
        options: WindowOptions::default(),
    };
    let _factory: WidgetFactory = std::sync::Arc::new(|| Box::new(ExternalWidget::new()));
    let _host = HostBuilder::new().widget("external-widget", || Box::new(ExternalWidget::new()));
}
