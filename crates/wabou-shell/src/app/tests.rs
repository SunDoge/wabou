use super::*;

#[test]
fn frame_wake_unifies_animation_and_deadline_scheduling() {
    let now = Instant::now();
    let future = now + std::time::Duration::from_secs(1);

    assert_eq!(frame_wake(true, Some(future), now), FrameWake::Redraw);
    assert_eq!(frame_wake(false, Some(now), now), FrameWake::Redraw);
    assert_eq!(
        frame_wake(false, Some(future), now),
        FrameWake::Deadline(future)
    );
    assert_eq!(frame_wake(false, None, now), FrameWake::Idle);
}
use crate::layout::PlacedNode;
use crate::text::TextContext;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use winit::keyboard::NamedKey;
use winit::raw_window_handle::WaylandWindowHandle;

struct EventActionSource {
    pending: bool,
    drained: Arc<AtomicUsize>,
}

struct EventRecordingSource(Arc<Mutex<Vec<UiEvent>>>);

impl FrameSource for EventRecordingSource {
    fn build_frame(
        &mut self,
        _tcx: &mut TextContext,
        _width: u32,
        _height: u32,
    ) -> Vec<PlacedNode> {
        Vec::new()
    }

    fn base_color(&self) -> Color {
        Color::BLACK
    }

    fn handle_event(&mut self, event: UiEvent) -> EventResponse {
        self.0.lock().unwrap().push(event);
        EventResponse::handled()
    }
}

#[test]
fn targeted_window_commands_do_not_affect_other_windows() {
    let source =
        || Box::new(EventRecordingSource(Arc::new(Mutex::new(Vec::new())))) as Box<dyn FrameSource>;
    let mut windows = [App::new(source()), App::new(source())];
    windows[0].logical_window_id = 11;
    windows[1].logical_window_id = 22;

    let target = find_window_by_logical_id(windows.iter_mut(), 22).expect("target window");
    apply_window_command(target, WindowCommand::Close);

    assert!(!windows[0].close_requested);
    assert!(windows[1].close_requested);
    assert!(find_window_by_logical_id(windows.iter_mut(), 99).is_none());
}

#[test]
fn wayland_windows_report_that_visibility_requires_surface_recreation() {
    let surface = NonNull::from(&()).cast();
    let handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(surface));
    assert!(!window_capabilities(Some(handle)).mutable_visibility);
    assert!(window_capabilities(None).mutable_visibility);
}

#[test]
fn external_url_actions_only_allow_web_schemes() {
    assert_eq!(
        allowed_external_url("https://example.com/path")
            .as_ref()
            .map(url::Url::scheme),
        Some("https")
    );
    assert_eq!(
        allowed_external_url("http://example.com")
            .as_ref()
            .map(url::Url::scheme),
        Some("http")
    );
    for unsafe_url in [
        "file:///etc/passwd",
        "javascript:alert(1)",
        "mailto:user@example.com",
        "not a url",
    ] {
        assert_eq!(allowed_external_url(unsafe_url), None, "{unsafe_url}");
    }
}

impl FrameSource for EventActionSource {
    fn build_frame(
        &mut self,
        _tcx: &mut TextContext,
        _width: u32,
        _height: u32,
    ) -> Vec<PlacedNode> {
        Vec::new()
    }

    fn base_color(&self) -> Color {
        Color::BLACK
    }

    fn handle_event(&mut self, _event: UiEvent) -> EventResponse {
        self.pending = true;
        EventResponse::handled()
    }

    fn take_host_action(&mut self) -> Option<HostAction> {
        self.pending.then(|| {
            self.pending = false;
            self.drained.fetch_add(1, Ordering::Relaxed);
            HostAction::SetWindowTitle(Some("event action".into()))
        })
    }
}

#[test]
fn named_editing_keys_use_dom_compatible_names() {
    assert_eq!(
        App::logical_key_name(&Key::Named(NamedKey::Backspace)),
        "Backspace"
    );
    assert_eq!(
        App::logical_key_name(&Key::Named(NamedKey::Delete)),
        "Delete"
    );
    assert_eq!(App::logical_key_name(&Key::Named(NamedKey::Enter)), "Enter");
    assert_eq!(App::logical_key_name(&Key::Character(" ".into())), " ");
    assert_eq!(App::logical_key_name(&Key::Character("a".into())), "a");
    assert_eq!(App::printable_key_text("a"), Some("a".into()));
    assert_eq!(App::printable_key_text("\u{8}"), None);
    assert_eq!(App::printable_key_text("\u{7f}"), None);
    assert_eq!(
        App::committed_key_text(Some("q"), Some("@"), Modifiers::CONTROL | Modifiers::ALT,),
        Some("@".into())
    );
    assert_eq!(
        App::committed_key_text(Some("a"), Some("\u{1}"), Modifiers::CONTROL),
        None
    );
    assert_eq!(KeyLocation::Standard.dom_code(), 0);
    assert_eq!(KeyLocation::Numpad.dom_code(), 3);

    let mut response = EventResponse::handled();
    assert_eq!(
        App::unconsumed_key_text(Some("x".into()), &response),
        Some("x".into())
    );
    response.consume_key_text = true;
    assert_eq!(App::unconsumed_key_text(Some("x".into()), &response), None);
}

#[test]
fn failed_present_gets_one_recovery_frame_without_a_busy_loop() {
    let mut pending = false;
    assert!(update_present_retry(false, &mut pending));
    assert!(!update_present_retry(false, &mut pending));
    assert!(!update_present_retry(true, &mut pending));
    assert!(!pending);
    assert!(update_present_retry(false, &mut pending));
}

#[test]
fn ime_cursor_rect_is_logical_and_never_empty() {
    let (position, size) = App::ime_cursor_rect(Some([12.5, 20.0, 12.5, 38.0]));
    assert_eq!(position, winit::dpi::LogicalPosition::new(12.5, 20.0));
    assert_eq!(size, winit::dpi::LogicalSize::new(1.0, 18.0));

    let (position, size) = App::ime_cursor_rect(None);
    assert_eq!(position, winit::dpi::LogicalPosition::new(0.0, 0.0));
    assert_eq!(size, winit::dpi::LogicalSize::new(1.0, 1.0));
}

#[test]
fn dispatch_event_drains_synchronous_host_actions() {
    let drained = Arc::new(AtomicUsize::new(0));
    let mut app = App::new(Box::new(EventActionSource {
        pending: false,
        drained: drained.clone(),
    }));

    assert!(app.dispatch_event(UiEvent::Focus(true)).handled);
    assert_eq!(drained.load(Ordering::Relaxed), 1);
}

#[test]
fn focus_loss_cancels_pressed_pointers_and_resets_physical_modifiers() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut app = App::new(Box::new(EventRecordingSource(events.clone())));
    app.pointer_position = Point { x: 12.0, y: 34.0 };
    app.pointer_buttons = 5;
    app.modifiers = Modifiers::CONTROL | Modifiers::SHIFT;

    app.dispatch_focus_change(false);

    assert_eq!(app.pointer_buttons, 0);
    assert_eq!(app.modifiers, Modifiers::default());
    assert_eq!(
        *events.lock().unwrap(),
        [
            UiEvent::Pointer(PointerEvent {
                phase: PointerPhase::Cancel,
                position: Point { x: 12.0, y: 34.0 },
                button: None,
                buttons: 0,
                modifiers: Modifiers::CONTROL | Modifiers::SHIFT,
            }),
            UiEvent::Focus(false),
        ]
    );
}
