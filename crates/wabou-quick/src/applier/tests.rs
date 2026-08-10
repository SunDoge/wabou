use super::*;

#[test]
fn key_payload_keeps_physical_modifiers_separate_from_primary() {
    let platform_primary = if cfg!(target_os = "macos") {
        Modifiers::META
    } else {
        Modifiers::CONTROL
    };
    let event = wabou_shell::KeyEvent {
        phase: KeyPhase::Down,
        key: "t".into(),
        key_without_modifiers: "t".into(),
        code: "KeyT".into(),
        text: None,
        text_with_all_modifiers: None,
        location: Default::default(),
        modifiers: platform_primary,
        repeat: false,
    };
    let payload: serde_json::Value = serde_json::from_str(&key_event_payload(&event)).unwrap();

    assert_eq!(payload["mods"], platform_primary.bits());
    assert_eq!(payload["primary"], true);

    let mut physical_control = event;
    physical_control.modifiers = if cfg!(target_os = "macos") {
        Modifiers::CONTROL
    } else {
        Modifiers::META
    };
    let payload: serde_json::Value =
        serde_json::from_str(&key_event_payload(&physical_control)).unwrap();
    assert_eq!(payload["mods"], physical_control.modifiers.bits());
    assert_eq!(payload["primary"], false);
}

struct HostActionWidget(Option<wabou_shell::HostAction>);

struct EventHostActionWidget(Option<wabou_shell::HostAction>);

struct UnmountActionWidget(Option<wabou_shell::HostAction>);

struct LifecycleWidget(Arc<std::sync::Mutex<Vec<&'static str>>>);

struct NodeEventWidget(Option<crate::widget::WidgetNodeEvent>);

struct ClipboardReadWidget {
    action: Option<wabou_shell::HostAction>,
    completed: Arc<std::sync::Mutex<Vec<wabou_shell::HostActionResult>>>,
}

struct WheelCaptureWidget(Arc<std::sync::Mutex<Vec<Point>>>);

struct KeyCaptureWidget(Arc<std::sync::Mutex<usize>>);

struct MeasuringWidget([f32; 2]);

struct StyleAwareMeasuringWidget(Arc<std::sync::Mutex<Vec<&'static str>>>);

impl crate::widget::Widget for MeasuringWidget {
    fn measure(&mut self, _tcx: &mut TextContext) -> Option<[f32; 2]> {
        Some(self.0)
    }

    fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
        vello::Scene::new()
    }
}

impl crate::widget::Widget for StyleAwareMeasuringWidget {
    fn style_changed(&mut self, _style: &crate::widget::WidgetStyle) {
        self.0.lock().unwrap().push("style");
    }

    fn measure(&mut self, _tcx: &mut TextContext) -> Option<[f32; 2]> {
        self.0.lock().unwrap().push("measure");
        Some([100.0, 40.0])
    }

    fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
        vello::Scene::new()
    }
}

impl crate::widget::Widget for HostActionWidget {
    fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
        vello::Scene::new()
    }

    fn poll_async(&mut self) -> bool {
        self.0.is_some()
    }

    fn take_host_action(&mut self) -> Option<wabou_shell::HostAction> {
        self.0.take()
    }
}

impl crate::widget::Widget for EventHostActionWidget {
    fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
        vello::Scene::new()
    }

    fn handle_event(&mut self, _event: &UiEvent) -> crate::widget::WidgetEventResult {
        crate::widget::WidgetEventResult::HANDLED
    }

    fn take_host_action(&mut self) -> Option<wabou_shell::HostAction> {
        self.0.take()
    }
}

impl crate::widget::Widget for UnmountActionWidget {
    fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
        vello::Scene::new()
    }

    fn unmount(&mut self) {
        self.0 = Some(wabou_shell::HostAction::SetWindowTitle(None));
    }

    fn take_host_action(&mut self) -> Option<wabou_shell::HostAction> {
        self.0.take()
    }
}

impl crate::widget::Widget for LifecycleWidget {
    fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
        vello::Scene::new()
    }

    fn focus_changed(&mut self, focused: bool) {
        self.0
            .lock()
            .unwrap()
            .push(if focused { "focus-in" } else { "focus-out" });
    }

    fn accepts_focus(&self) -> bool {
        true
    }

    fn handle_event(&mut self, event: &UiEvent) -> crate::widget::WidgetEventResult {
        if matches!(event, UiEvent::Pointer(pointer) if pointer.phase == PointerPhase::Cancel) {
            self.0.lock().unwrap().push("pointer-cancel");
            crate::widget::WidgetEventResult::HANDLED
        } else {
            crate::widget::WidgetEventResult::IGNORED
        }
    }

    fn unmount(&mut self) {
        self.0.lock().unwrap().push("unmount");
    }
}

impl crate::widget::Widget for NodeEventWidget {
    fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
        vello::Scene::new()
    }

    fn poll_async(&mut self) -> bool {
        self.0.is_some()
    }

    fn take_node_event(&mut self) -> Option<crate::widget::WidgetNodeEvent> {
        self.0.take()
    }
}

impl crate::widget::Widget for ClipboardReadWidget {
    fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
        vello::Scene::new()
    }

    fn poll_async(&mut self) -> bool {
        self.action.is_some()
    }

    fn take_host_action(&mut self) -> Option<wabou_shell::HostAction> {
        self.action.take()
    }

    fn complete_host_action(&mut self, result: wabou_shell::HostActionResult) {
        self.completed.lock().unwrap().push(result);
    }
}

impl crate::widget::Widget for WheelCaptureWidget {
    fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
        vello::Scene::new()
    }

    fn handle_event(&mut self, event: &UiEvent) -> crate::widget::WidgetEventResult {
        if let UiEvent::Wheel(wheel) = event {
            self.0.lock().unwrap().push(wheel.position);
            crate::widget::WidgetEventResult::HANDLED
        } else {
            crate::widget::WidgetEventResult::IGNORED
        }
    }
}

impl crate::widget::Widget for KeyCaptureWidget {
    fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
        vello::Scene::new()
    }

    fn handle_event(&mut self, event: &UiEvent) -> crate::widget::WidgetEventResult {
        if matches!(event, UiEvent::Key(_)) {
            *self.0.lock().unwrap() += 1;
            crate::widget::WidgetEventResult::handled_consuming_key_text()
        } else {
            crate::widget::WidgetEventResult::IGNORED
        }
    }

    fn accepts_focus(&self) -> bool {
        true
    }
}

#[test]
fn prevented_keydown_never_reaches_the_focused_widget() {
    let js = JsRuntime::new().expect("runtime");
    js.with(|ctx| {
        ctx.eval::<(), _>(
            r#"
            globalThis.__wabou_dispatch_host_frame = (bytes) => {
                const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
                const eventId = view.getUint32(48, true);
                return { needsTick: true, preventedEventIds: new Uint32Array([eventId]) };
            };
            "#,
        )
    })
    .unwrap();
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: 2,
        tag: div,
        attrs: vec![],
    });
    applier.apply_op(&Op::AddEventListener {
        id: 2,
        event_type: event::KEYDOWN,
    });
    let node = applier.node_store.solid_to_node[&2];
    let received = Arc::new(std::sync::Mutex::new(0));
    applier
        .widget_manager
        .widgets
        .insert(node, Box::new(KeyCaptureWidget(received.clone())));
    applier.input.focused_target = Some(2);

    let response = applier.handle_event(UiEvent::Key(wabou_shell::KeyEvent {
        phase: KeyPhase::Down,
        key: "t".into(),
        key_without_modifiers: "t".into(),
        code: "KeyT".into(),
        text: Some("t".into()),
        text_with_all_modifiers: None,
        location: Default::default(),
        modifiers: if cfg!(target_os = "macos") {
            Modifiers::META
        } else {
            Modifiers::CONTROL
        },
        repeat: false,
    }));

    assert!(response.handled);
    assert!(response.consume_key_text);
    assert_eq!(*received.lock().unwrap(), 0);

    applier
        .js
        .with(|ctx| {
            ctx.eval::<(), _>(
                r#"
                globalThis.__wabou_dispatch_host_frame = () => ({
                    needsTick: true,
                    preventedEventIds: new Uint32Array(),
                });
                "#,
            )
        })
        .unwrap();
    applier.handle_event(UiEvent::Key(wabou_shell::KeyEvent {
        phase: KeyPhase::Down,
        key: "x".into(),
        key_without_modifiers: "x".into(),
        code: "KeyX".into(),
        text: Some("x".into()),
        text_with_all_modifiers: None,
        location: Default::default(),
        modifiers: Modifiers::default(),
        repeat: false,
    }));
    assert_eq!(
        *received.lock().unwrap(),
        1,
        "unprevented keydown must continue to the focused widget"
    );
}

#[test]
fn imperative_focus_uses_the_same_host_focus_state_as_pointer_input() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: 2,
        tag: div,
        attrs: vec![],
    });

    applier.apply_op(&Op::FocusNode { id: 2 });
    assert_eq!(applier.input.focused_target, Some(2));

    applier.apply_op(&Op::FocusNode { id: 999 });
    assert_eq!(
        applier.input.focused_target,
        Some(2),
        "a stale JS handle must not clear valid native focus"
    );
}

#[test]
fn widget_measurements_refresh_intrinsic_layout_before_paint() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: 2,
        tag: div,
        attrs: vec![],
    });
    applier.apply_op(&Op::AppendChild {
        parent: 1,
        child: 2,
    });
    let node = applier.node_store.solid_to_node[&2];
    applier
        .widget_manager
        .widgets
        .insert(node, Box::new(MeasuringWidget([123.0, 45.0])));
    assert_eq!(
        applier.computed_node_snapshot(2).unwrap().intrinsic_size,
        None
    );

    let mut tcx = TextContext::new();
    applier.measure_widgets(&mut tcx);

    assert_eq!(
        applier.computed_node_snapshot(2).unwrap().intrinsic_size,
        Some([123.0, 45.0])
    );
    assert!(applier.invalidation.contains(InvalidationFlags::LAYOUT));
}

#[test]
fn widget_styles_are_delivered_once_before_measurement() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
    applier.widget_manager.widgets.insert(
        applier.node_store.root,
        Box::new(StyleAwareMeasuringWidget(calls.clone())),
    );

    let mut tcx = TextContext::new();
    applier.sync_widget_styles();
    applier.measure_widgets(&mut tcx);
    applier.sync_widget_styles();

    assert_eq!(*calls.lock().unwrap(), ["style", "measure"]);

    let mut paint = applier
        .node_store
        .tree
        .get_node_context(applier.node_store.root)
        .unwrap()
        .clone();
    paint.font_size += 1.0;
    applier
        .node_store
        .tree
        .set_node_context(applier.node_store.root, Some(paint))
        .unwrap();
    applier.sync_widget_styles();

    assert_eq!(*calls.lock().unwrap(), ["style", "measure", "style"]);
}

#[test]
fn wheel_routing_preserves_pointer_position_for_widgets() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let received = Arc::new(std::sync::Mutex::new(Vec::new()));
    applier.widget_manager.widgets.insert(
        applier.node_store.root,
        Box::new(WheelCaptureWidget(received.clone())),
    );
    applier.input.hovered_target = Some(1);
    applier.input.pointer_position = (42.0, 73.0);

    let response = applier.handle_event(UiEvent::Wheel(wabou_shell::WheelEvent {
        position: Point { x: 42.0, y: 73.0 },
        delta_x: 0.0,
        delta_y: -40.0,
        modifiers: Modifiers::default(),
    }));

    assert!(response.handled);
    assert_eq!(
        received.lock().unwrap().as_slice(),
        [Point { x: 42.0, y: 73.0 }]
    );
}

#[test]
fn event_mask_is_compact_and_preserves_protocol_codes() {
    assert_eq!(std::mem::size_of::<EventMask>(), 4);
    let mut mask = EventMask::default();
    mask.insert(event::CLICK);
    mask.insert(event::SCROLL);
    mask.insert(0);
    assert!(mask.contains(event::CLICK));
    assert!(mask.contains(event::SCROLL));
    assert_eq!(
        mask.codes().collect::<Vec<_>>(),
        vec![event::CLICK, event::SCROLL]
    );
    mask.remove(event::CLICK);
    assert!(!mask.contains(event::CLICK));
}

#[test]
fn native_scroll_observations_coalesce_by_target() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    applier
        .input
        .listeners
        .entry(1)
        .or_default()
        .insert(event::SCROLL);
    applier
        .scroll_offsets
        .insert(applier.node_store.root, [0.0, 12.0]);
    applier.queue_scroll_event(applier.node_store.root);
    applier
        .scroll_offsets
        .insert(applier.node_store.root, [0.0, 48.0]);
    applier.queue_scroll_event(applier.node_store.root);

    assert_eq!(applier.pending_scroll_events.len(), 1);
    assert_eq!(applier.pending_scroll_events[&1], [0.0, 48.0]);
}

#[test]
fn widget_host_actions_reach_the_frame_source() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    applier.widget_manager.widgets.insert(
        applier.node_store.root,
        Box::new(HostActionWidget(Some(
            wabou_shell::HostAction::SetWindowTitle(Some("terminal".into())),
        ))),
    );

    assert!(FrameSource::poll_async(&mut applier));
    assert_eq!(
        FrameSource::take_host_action(&mut applier),
        Some(wabou_shell::HostAction::SetWindowTitle(Some(
            "terminal".into()
        )))
    );
    assert_eq!(FrameSource::take_host_action(&mut applier), None);
}

#[test]
fn asynchronous_widget_events_are_routed_to_the_owning_solid_node() {
    let mut applier = interactive_applier();
    applier.apply_op(&Op::AddEventListener {
        id: 2,
        event_type: event::TERMINALEXIT,
    });
    let node = applier.node_store.solid_to_node[&2];
    applier.widget_manager.widgets.insert(
        node,
        Box::new(NodeEventWidget(Some(crate::widget::WidgetNodeEvent::json(
            event::TERMINALEXIT,
            r#"{"reason":"exit"}"#,
        )))),
    );

    assert!(FrameSource::poll_async(&mut applier));
    let dispatched = applier
        .js
        .with(|ctx| {
            ctx.eval::<String, _>(
                "JSON.stringify(globalThis.dispatched[globalThis.dispatched.length - 1])",
            )
        })
        .expect("read widget node event");
    assert_eq!(
        dispatched,
        format!(r#"[2,{},"{{\"reason\":\"exit\"}}"]"#, event::TERMINALEXIT)
    );
}

#[test]
fn widget_event_host_actions_are_available_without_an_async_poll() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    applier.widget_manager.widgets.insert(
        applier.node_store.root,
        Box::new(EventHostActionWidget(Some(
            wabou_shell::HostAction::OpenUrl("https://example.com".into()),
        ))),
    );

    let response = applier
        .handle_widget_event(1, &UiEvent::Focus(true))
        .expect("widget handled event");
    assert!(response.handled);
    assert_eq!(
        FrameSource::take_host_action(&mut applier),
        Some(wabou_shell::HostAction::OpenUrl(
            "https://example.com".into()
        ))
    );
}

#[test]
fn dropping_a_widget_drains_unmount_host_actions_before_routing_is_removed() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: 2,
        tag: div,
        attrs: vec![],
    });
    let node = applier.node_store.solid_to_node[&2];
    applier
        .widget_manager
        .widgets
        .insert(node, Box::new(UnmountActionWidget(None)));

    applier.apply_op(&Op::DropNode { id: 2 });

    assert_eq!(
        FrameSource::take_host_action(&mut applier),
        Some(wabou_shell::HostAction::SetWindowTitle(None))
    );
    assert!(!applier.widget_manager.widgets.contains_key(&node));
}

#[test]
fn dropping_a_focused_captured_widget_releases_input_before_unmount() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: 2,
        tag: div,
        attrs: vec![],
    });
    let node = applier.node_store.solid_to_node[&2];
    let lifecycle = Arc::new(std::sync::Mutex::new(Vec::new()));
    applier
        .widget_manager
        .widgets
        .insert(node, Box::new(LifecycleWidget(lifecycle.clone())));
    applier.input.focused_target = Some(2);
    applier.input.pointer_down_target = Some(2);
    applier.input.pointer_down_position = Some((10.0, 20.0));
    applier.input.pointer_dragged = true;

    applier.apply_op(&Op::DropNode { id: 2 });

    assert_eq!(
        *lifecycle.lock().unwrap(),
        ["pointer-cancel", "focus-out", "unmount"]
    );
    assert_eq!(applier.input.focused_target, None);
    assert_eq!(applier.input.pointer_down_target, None);
    assert_eq!(applier.input.pointer_down_position, None);
    assert!(!applier.input.pointer_dragged);
}

#[test]
fn window_focus_loss_cancels_the_captured_pointer_before_blur() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: 2,
        tag: div,
        attrs: vec![],
    });
    let node = applier.node_store.solid_to_node[&2];
    let lifecycle = Arc::new(std::sync::Mutex::new(Vec::new()));
    applier
        .widget_manager
        .widgets
        .insert(node, Box::new(LifecycleWidget(lifecycle.clone())));
    applier.input.focused_target = Some(2);
    applier.input.pointer_down_target = Some(2);
    applier.input.pointer_down_position = Some((10.0, 20.0));
    applier.input.pointer_position = (15.0, 25.0);
    applier.input.pointer_buttons = 1;
    applier.input.pointer_dragged = true;
    applier.last_text_click = Some((Instant::now(), 2, 15.0, 25.0, 1));

    let blurred = applier.handle_event(UiEvent::Focus(false));

    assert_eq!(*lifecycle.lock().unwrap(), ["pointer-cancel", "focus-out"]);
    assert_eq!(blurred.text_input, Some(false));
    assert_eq!(applier.input.focused_target, Some(2));
    assert!(!applier.input.window_focused);
    assert_eq!(applier.input.pointer_down_target, None);
    assert_eq!(applier.input.pointer_down_position, None);
    assert_eq!(applier.input.pointer_buttons, 0);
    assert!(!applier.input.pointer_dragged);
    assert!(applier.last_text_click.is_none());

    let focused = applier.handle_event(UiEvent::Focus(true));
    assert_eq!(focused.text_input, Some(true));
    assert_eq!(
        *lifecycle.lock().unwrap(),
        ["pointer-cancel", "focus-out", "focus-in"]
    );
    assert_eq!(applier.input.focused_target, Some(2));
    assert!(applier.input.window_focused);

    applier.last_text_click = Some((Instant::now(), 2, 15.0, 25.0, 1));
    applier.handle_event(UiEvent::TextInput("x".into()));
    assert!(
        applier.last_text_click.is_none(),
        "text input must break a native text multi-click sequence"
    );
}

#[test]
fn clipboard_read_completions_route_to_the_requesting_widget() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: 2,
        tag: div,
        attrs: vec![],
    });
    let second_node = applier.node_store.solid_to_node[&2];
    let first_completed = Arc::new(std::sync::Mutex::new(Vec::new()));
    let second_completed = Arc::new(std::sync::Mutex::new(Vec::new()));
    applier.widget_manager.widgets.insert(
        applier.node_store.root,
        Box::new(ClipboardReadWidget {
            action: Some(wabou_shell::HostAction::ReadClipboard { request_id: 7 }),
            completed: first_completed.clone(),
        }),
    );
    applier.widget_manager.widgets.insert(
        second_node,
        Box::new(ClipboardReadWidget {
            action: Some(wabou_shell::HostAction::ReadClipboard { request_id: 7 }),
            completed: second_completed.clone(),
        }),
    );

    assert!(FrameSource::poll_async(&mut applier));
    let mut requests = Vec::new();
    while let Some(wabou_shell::HostAction::ReadClipboard { request_id }) =
        FrameSource::take_host_action(&mut applier)
    {
        requests.push(request_id);
    }
    assert_eq!(requests.len(), 2);
    assert_ne!(requests[0], requests[1]);
    for request_id in requests.into_iter().rev() {
        let (node, _) = applier.widget_manager.host_action_routes[&request_id];
        let text = if node == applier.node_store.root {
            "first"
        } else {
            "second"
        };
        FrameSource::complete_host_action(
            &mut applier,
            wabou_shell::HostActionResult::Clipboard {
                request_id,
                text: Some(text.into()),
            },
        );
    }
    assert_eq!(
        *first_completed.lock().unwrap(),
        vec![wabou_shell::HostActionResult::Clipboard {
            request_id: 7,
            text: Some("first".into()),
        }]
    );
    assert_eq!(
        *second_completed.lock().unwrap(),
        vec![wabou_shell::HostActionResult::Clipboard {
            request_id: 7,
            text: Some("second".into()),
        }]
    );
}
use wabou_shell::{Point, PointerEvent};

fn pointer(phase: PointerPhase, x: f64, y: f64, buttons: u32) -> UiEvent {
    UiEvent::Pointer(PointerEvent {
        phase,
        position: Point { x, y },
        button: Some(PointerButton::Primary),
        buttons,
        modifiers: Modifiers::default(),
    })
}

fn install_host_frame_test_hook(js: &JsRuntime) {
    js.with(|ctx| {
        ctx.eval::<(), _>(
            r#"
      globalThis.dispatched = [];
      globalThis.resizeChanges = [];
      globalThis.__host_got = [];
      globalThis.__wabou_dispatch_host_frame = (u8) => {
        const view = new DataView(u8.buffer, u8.byteOffset, u8.byteLength);
        const dec = new TextDecoder();
        let o = 32;
        const count = view.getUint32(24, true);
        for (let n = 0; n < count; n++) {
          const start = o, kind = view.getUint8(o), len = view.getUint32(o + 4, true);
          o += 8;
          if (kind === 1) {
            const id = view.getUint32(o, true), code = view.getUint8(o + 4);
            const payloadKind = view.getUint8(o + 5); o += 12;
            let payload = "";
            if (payloadKind === 2) {
              const size = view.getUint32(o, true); o += 4;
              payload = dec.decode(u8.subarray(o, o + size));
            }
            globalThis.dispatched.push([id, code, payload]);
          } else if (kind === 2) {
            globalThis.resizeChanges.push([
              view.getUint32(o, true), view.getFloat32(o + 4, true), view.getFloat32(o + 8, true)
            ]);
          } else if (kind === 3) {
            const tl = view.getUint16(o, true); o += 2;
            const topic = dec.decode(u8.subarray(o, o + tl)); o += tl;
            const valueKind = u8[o++]; let payload;
            if (valueKind === 2) { payload = view.getInt32(o, true); }
            else if (valueKind === 4) {
              const size = view.getUint16(o, true); o += 2;
              payload = dec.decode(u8.subarray(o, o + size));
            }
            globalThis.__host_got.push({topic, payload});
          }
          o = start + len;
        }
        return { needsTick: true, preventedEventIds: new Uint32Array() };
      };
    "#,
        )
    })
    .expect("host-frame test hook");
}

fn interactive_applier() -> Applier {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);

    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (button, width, height) = {
        let mut atoms = applier.atoms.borrow_mut();
        (
            atoms.intern("button"),
            atoms.intern("width"),
            atoms.intern("height"),
        )
    };
    applier.apply_op(&Op::CreateElement {
        id: 2,
        tag: button,
        attrs: Vec::new(),
    });
    applier.apply_op(&Op::SetStyle {
        id: 2,
        prop: width,
        value: "100px",
    });
    applier.apply_op(&Op::SetStyle {
        id: 2,
        prop: height,
        value: "50px",
    });
    applier.apply_op(&Op::AppendChild {
        parent: 1,
        child: 2,
    });
    for code in [event::POINTERDOWN, event::POINTERUP, event::CLICK] {
        applier.apply_op(&Op::AddEventListener {
            id: 2,
            event_type: code,
        });
    }
    let mut root_style = applier
        .node_store
        .tree
        .style(applier.node_store.root)
        .expect("root style")
        .clone();
    root_style.size.width = taffy::Dimension::length(800.0);
    root_style.size.height = taffy::Dimension::length(600.0);
    applier
        .node_store
        .tree
        .set_style(applier.node_store.root, root_style)
        .expect("viewport style");
    applier
        .node_store
        .tree
        .compute_layout(
            applier.node_store.root,
            taffy::geometry::Size {
                width: taffy::AvailableSpace::Definite(800.0),
                height: taffy::AvailableSpace::Definite(600.0),
            },
        )
        .expect("layout");
    let mut placed = layout::flatten_with_scroll(
        &applier.node_store.tree,
        applier.node_store.root,
        &applier.scroll_offsets,
    );
    applier.update_scrollbar_visuals(&mut placed);
    applier.rebuild_hit_geometry(&placed);
    applier
}

#[test]
fn tab_order_honors_positive_zero_negative_and_disabled_targets() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    js.with(|ctx| {
        ctx.eval::<(), _>(
            "globalThis.__wabou_tick = () => false; globalThis.__wabou_has_raf = () => false;",
        )
    })
    .unwrap();
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (button, tab_index, disabled, width, height) = {
        let mut atoms = applier.atoms.borrow_mut();
        (
            atoms.intern("button"),
            atoms.intern("tabIndex"),
            atoms.intern("disabled"),
            atoms.intern("width"),
            atoms.intern("height"),
        )
    };
    for id in 2..=6 {
        applier.apply_op(&Op::CreateElement {
            id,
            tag: button,
            attrs: Vec::new(),
        });
        applier.apply_op(&Op::AppendChild {
            parent: 1,
            child: id,
        });
        applier.apply_op(&Op::SetStyle {
            id,
            prop: width,
            value: "100px",
        });
        applier.apply_op(&Op::SetStyle {
            id,
            prop: height,
            value: "20px",
        });
    }
    applier.apply_op(&Op::SetAttribute {
        id: 3,
        name: tab_index,
        value: "2",
    });
    applier.apply_op(&Op::SetAttribute {
        id: 4,
        name: tab_index,
        value: "-1",
    });
    applier.apply_op(&Op::SetAttribute {
        id: 5,
        name: tab_index,
        value: "1",
    });
    applier.apply_op(&Op::SetAttribute {
        id: 6,
        name: disabled,
        value: "",
    });
    let mut tcx = TextContext::new();
    let placed = FrameSource::build_frame(&mut applier, &mut tcx, 800, 600);
    assert!(placed.len() >= 6, "placed node count: {}", placed.len());

    assert_eq!(applier.input.focus_order, [5, 3, 2]);
    assert!(applier.input.focusable_targets.contains(&4));
    assert!(!applier.input.focusable_targets.contains(&6));
    assert_eq!(applier.advance_focus(false), Some(5));
    assert_eq!(applier.advance_focus(false), Some(3));
    assert_eq!(applier.advance_focus(true), Some(5));
}

#[test]
fn inert_isolates_an_entire_subtree_from_input_focus_and_semantics() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (view, button, inert, aria_hidden, width, height) = {
        let mut atoms = applier.atoms.borrow_mut();
        (
            atoms.intern("view"),
            atoms.intern("button"),
            atoms.intern("inert"),
            atoms.intern("aria-hidden"),
            atoms.intern("width"),
            atoms.intern("height"),
        )
    };
    applier.apply_op(&Op::CreateElement {
        id: 2,
        tag: view,
        attrs: Vec::new(),
    });
    applier.apply_op(&Op::CreateElement {
        id: 3,
        tag: button,
        attrs: Vec::new(),
    });
    for id in [2, 3] {
        applier.apply_op(&Op::SetStyle {
            id,
            prop: width,
            value: "100px",
        });
        applier.apply_op(&Op::SetStyle {
            id,
            prop: height,
            value: "50px",
        });
    }
    applier.apply_op(&Op::AppendChild {
        parent: 2,
        child: 3,
    });
    applier.apply_op(&Op::AppendChild {
        parent: 1,
        child: 2,
    });
    applier.rebuild_layout_boxes();
    let mut root_style = applier
        .node_store
        .tree
        .style(applier.node_store.root)
        .unwrap()
        .clone();
    root_style.size.width = taffy::Dimension::length(200.0);
    root_style.size.height = taffy::Dimension::length(200.0);
    applier
        .node_store
        .tree
        .set_style(applier.node_store.root, root_style)
        .unwrap();
    applier
        .node_store
        .tree
        .compute_layout(
            applier.node_store.root,
            taffy::geometry::Size {
                width: taffy::AvailableSpace::Definite(200.0),
                height: taffy::AvailableSpace::Definite(200.0),
            },
        )
        .unwrap();
    let placed = layout::flatten_with_scroll(
        &applier.node_store.tree,
        applier.node_store.root,
        &applier.scroll_offsets,
    );

    applier.rebuild_hit_geometry(&placed);
    applier.rebuild_focus_order(&placed);
    applier.rebuild_semantic_snapshot(&placed);
    assert_eq!(applier.input.hit_test(10.0, 10.0), Some(3));
    assert_eq!(applier.input.focus_order, [3]);
    assert!(
        applier
            .projections
            .semantic_snapshot
            .nodes
            .iter()
            .any(|node| node.id == 3)
    );

    applier.apply_op(&Op::SetAttribute {
        id: 2,
        name: inert,
        value: "",
    });
    applier.rebuild_hit_geometry(&placed);
    applier.rebuild_focus_order(&placed);
    applier.rebuild_semantic_snapshot(&placed);
    assert_eq!(applier.input.hit_test(10.0, 10.0), Some(1));
    assert!(applier.input.focus_order.is_empty());
    assert!(
        applier
            .projections
            .semantic_snapshot
            .nodes
            .iter()
            .all(|node| node.id != 2 && node.id != 3)
    );
    assert!(!applier.handle_semantic_action(SemanticAction::Click { target: 3 }));
    assert!(!applier.handle_semantic_action(SemanticAction::Focus { target: 3 }));

    applier.apply_op(&Op::RemoveAttribute { id: 2, name: inert });
    applier.apply_op(&Op::SetAttribute {
        id: 2,
        name: aria_hidden,
        value: "true",
    });
    applier.rebuild_hit_geometry(&placed);
    applier.rebuild_focus_order(&placed);
    applier.rebuild_semantic_snapshot(&placed);
    assert_eq!(applier.input.hit_test(10.0, 10.0), Some(3));
    assert!(applier.input.focus_order.is_empty());
    assert!(
        applier
            .projections
            .semantic_snapshot
            .nodes
            .iter()
            .all(|node| node.id != 2 && node.id != 3)
    );
}

#[test]
fn focused_widget_can_consume_tab_before_default_focus_traversal() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: 2,
        tag: div,
        attrs: vec![],
    });
    let node = applier.node_store.solid_to_node[&2];
    let received = Arc::new(std::sync::Mutex::new(0));
    applier
        .widget_manager
        .widgets
        .insert(node, Box::new(KeyCaptureWidget(received.clone())));
    applier.input.focused_target = Some(2);
    applier.input.focus_order = vec![2, 3];

    let response = applier.handle_event(UiEvent::Key(wabou_shell::KeyEvent {
        phase: KeyPhase::Down,
        key: "Tab".into(),
        key_without_modifiers: "Tab".into(),
        code: "Tab".into(),
        text: None,
        text_with_all_modifiers: None,
        location: Default::default(),
        modifiers: Modifiers::default(),
        repeat: false,
    }));

    assert!(response.handled);
    assert_eq!(applier.input.focused_target, Some(2));
    assert_eq!(*received.lock().unwrap(), 1);
}

mod overlay_cases;
mod projection_cases;
mod runtime_cases;
mod text_cases;
