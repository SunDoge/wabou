use super::effect_bridge::decode_effect_payload;
use super::*;

fn set_text_behavior(applier: &mut Applier, id: u32) {
    let id = nk(id);
    applier.apply_op(&Op::SetTextBehavior {
        id,
        flags: crate::protocol::TEXT_BEHAVIOR_AGGREGATE_DIRECT
            | crate::protocol::TEXT_BEHAVIOR_SINGLE_LINE,
    });
}

fn set_focus_order(applier: &mut Applier, id: u32, focus_order: i32) {
    let id = nk(id);
    applier.apply_op(&Op::SetInteractionPolicy {
        id,
        flags: crate::protocol::INTERACTION_POLICY_FOCUSABLE,
        focus_order,
    });
}

fn set_interaction_blocked(applier: &mut Applier, id: u32, blocked: bool) {
    let id = nk(id);
    applier.apply_op(&Op::SetInteractionPolicy {
        id,
        flags: if blocked {
            crate::protocol::INTERACTION_POLICY_BLOCK_SUBTREE
        } else {
            0
        },
        focus_order: 0,
    });
}

fn set_focus_contained(applier: &mut Applier, id: u32) {
    let id = nk(id);
    applier.apply_op(&Op::SetInteractionPolicy {
        id,
        flags: crate::protocol::INTERACTION_POLICY_CONTAIN_FOCUS,
        focus_order: 0,
    });
}

#[test]
fn window_effect_rejects_an_unknown_renderer_instead_of_falling_back() {
    let payload = decode_effect_payload(
        wabou_shell::effect::builtin::WINDOW_CREATE,
        wabou_shell::initial_window_resource_key(0),
        r#"{"renderer":"browser"}"#.to_owned(),
        None,
    );
    assert!(matches!(
        payload,
        wabou_shell::EffectPayload::Invalid { message, .. }
            if message == "unknown renderer backend `browser`"
    ));
}

fn create_element_with_attrs(applier: &mut Applier, id: u32, tag: Atom, attrs: &[(Atom, &str)]) {
    let id = nk(id);
    applier.apply_op(&Op::CreateElement { id, tag });
    for &(name, value) in attrs {
        applier.apply_op(&Op::SetAttribute { id, name, value });
    }
}

#[test]
fn debug_layout_overlay_encodes_visible_scene_geometry() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let view = applier.document.atoms.borrow_mut().intern("view");
    applier.apply_op(&Op::CreateElement {
        id: nk(2),
        tag: view,
    });
    let node_id = applier.document.node_store.solid_to_node[&nk(2)];
    let placed = PlacedNode {
        node_id,
        parent_node_id: None,
        depth: 0,
        rect: [10.0, 12.0, 90.0, 52.0],
        content_origin: [10.0, 12.0],
        content_size: [80.0, 40.0],
        clip: None,
        clip_radius: 0.0,
        clip_depth: None,
        own_clip: None,
        own_clip_radius: 0.0,
        border_widths: [0.0; 4],
        scroll: layout::ScrollMetrics::default(),
        paint: Paint::default(),
    };
    let debug = wabou_devtools::DebugState::shared();
    applier.set_debug_state(debug.clone());

    let mut disabled_scene = Scene::new();
    applier.paint_debug_overlay(
        &mut disabled_scene,
        std::slice::from_ref(&placed),
        &mut TextContext::new(),
        1.0,
    );
    assert!(disabled_scene.commands.is_empty());
    let disabled_paint = debug.read().expect("debug state").overlay_paint();
    assert_eq!(disabled_paint.sequence, 1);
    assert!(!disabled_paint.enabled);
    assert_eq!(disabled_paint.layout_bounds, 0);

    debug
        .write()
        .expect("debug state")
        .set_overlay(wabou_devtools::DebugOverlay {
            layout: true,
            ..Default::default()
        });
    let mut enabled_scene = Scene::new();
    applier.paint_debug_overlay(&mut enabled_scene, &[placed], &mut TextContext::new(), 1.0);
    assert!(
        !enabled_scene.commands.is_empty(),
        "enabled layout diagnostics must encode a visible stroke"
    );
    let enabled_paint = debug.read().expect("debug state").overlay_paint();
    assert_eq!(enabled_paint.sequence, 2);
    assert!(enabled_paint.enabled);
    assert_eq!(enabled_paint.layout_bounds, 1);
    assert_eq!(enabled_paint.clip_bounds, 0);
    assert_eq!(enabled_paint.highlights, 0);

    let output = debug
        .write()
        .expect("debug state")
        .request_screenshot()
        .expect("reserve secure screenshot");
    let (requested_path, mut output_file) = debug
        .write()
        .expect("debug state")
        .take_screenshot_request()
        .expect("take secure screenshot request");
    assert_eq!(requested_path, output);
    wabou_shell::renderer::render_to_png_file(
        &enabled_scene,
        120,
        80,
        Color::WHITE,
        &mut output_file,
        &output,
    )
    .expect("render debug overlay");
    drop(output_file);
    debug
        .write()
        .expect("debug state")
        .complete_screenshot(&requested_path, Ok(output.clone()))
        .expect("complete secure screenshot");
    let pixels = image::open(&output)
        .expect("open debug overlay png")
        .into_rgba8();
    std::fs::remove_file(output).expect("remove debug overlay png");
    let cyan_pixels = pixels
        .pixels()
        .filter(|pixel| pixel[0] < 100 && pixel[1] > 130 && pixel[2] > 180)
        .count();
    let halo_pixels = pixels
        .pixels()
        .filter(|pixel| pixel[0] < 230 && pixel[1] < 230 && pixel[2] < 230)
        .count();
    assert!(
        cyan_pixels > 100,
        "cyan layout stroke must reach pixels; found {cyan_pixels}"
    );
    assert!(
        halo_pixels > 100,
        "contrast halo must reach pixels; found {halo_pixels}"
    );
    assert_eq!(pixels.get_pixel(50, 30).0, [255, 255, 255, 255]);
}

#[test]
fn selected_debug_overlay_distinguishes_border_and_content_boxes() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let view = applier.document.atoms.borrow_mut().intern("view");
    applier.apply_op(&Op::CreateElement {
        id: nk(2),
        tag: view,
    });
    let node_id = applier.document.node_store.solid_to_node[&nk(2)];
    let placed = PlacedNode {
        node_id,
        parent_node_id: None,
        depth: 0,
        rect: [10.0, 20.0, 110.0, 80.0],
        content_origin: [30.0, 35.0],
        content_size: [60.0, 30.0],
        clip: None,
        clip_radius: 0.0,
        clip_depth: None,
        own_clip: None,
        own_clip_radius: 0.0,
        border_widths: [2.0, 3.0, 4.0, 5.0],
        scroll: layout::ScrollMetrics::default(),
        paint: Paint::default(),
    };
    let debug = wabou_devtools::DebugState::shared();
    debug
        .write()
        .expect("debug state")
        .set_overlay(wabou_devtools::DebugOverlay {
            selected_node: Some(nk(2)),
            ..Default::default()
        });
    applier.set_debug_state(debug.clone());

    let mut scene = Scene::new();
    applier.paint_debug_overlay(&mut scene, &[placed], &mut TextContext::new(), 1.0);
    let paint = debug.read().expect("debug state").overlay_paint();
    assert_eq!(paint.highlights, 1);

    let output = std::env::temp_dir().join(format!(
        "wabou-debug-overlay-box-model-{}.png",
        std::process::id()
    ));
    wabou_shell::renderer::render_to_png(&scene, 130, 100, Color::WHITE, &output.to_string_lossy())
        .expect("render selected debug overlay");
    let pixels = image::open(&output)
        .expect("open selected debug overlay png")
        .into_rgba8();
    std::fs::remove_file(output).expect("remove selected debug overlay png");

    let padding = pixels.get_pixel(20, 50).0;
    let content = pixels.get_pixel(50, 50).0;
    assert_ne!(padding, [255, 255, 255, 255]);
    assert_ne!(
        content, padding,
        "content and padding must use distinct tints"
    );
    assert!(
        content[2] > content[0],
        "content box must retain its cyan diagnostic tint: {content:?}"
    );
    let cyan_pixels = pixels
        .pixels()
        .filter(|pixel| pixel[0] < 100 && pixel[1] > 160 && pixel[2] > 180)
        .count();
    let amber_pixels = pixels
        .pixels()
        .filter(|pixel| pixel[0] > 180 && pixel[1] > 80 && pixel[2] < 180)
        .count();
    // Stroke coverage varies between the Vello CPU implementations used by
    // local development and CI. Presence plus the interior tint samples above
    // proves the box-model layers without coupling this test to one
    // rasterizer's antialiasing coverage.
    assert!(
        cyan_pixels > 0,
        "content outline must reach pixels; found {cyan_pixels}"
    );
    assert!(
        amber_pixels > 0,
        "border inset must reach pixels; found {amber_pixels}"
    );
}

#[test]
fn text_layout_defaults_require_an_explicit_js_contract() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let text = applier.document.atoms.borrow_mut().intern("text");
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: text,
    });
    let unconfigured = applier.computed_node_snapshot(NodeKey::new(2, 1)).unwrap();
    assert!(unconfigured.wrap_text);
    assert_ne!(unconfigured.layout.flex_shrink, 0.0);

    set_text_behavior(&mut applier, 2);
    let configured = applier.computed_node_snapshot(NodeKey::new(2, 1)).unwrap();
    assert!(!configured.wrap_text);
    assert_eq!(configured.layout.flex_shrink, 0.0);

    applier.apply_op(&Op::SetTextBehavior {
        id: NodeKey::new(2, 1),
        flags: crate::protocol::TEXT_BEHAVIOR_AGGREGATE_DIRECT,
    });
    applier.apply_op(&Op::SetTextMaxLines {
        id: NodeKey::new(2, 1),
        max_lines: 2,
    });
    let clamped = applier.computed_node_snapshot(NodeKey::new(2, 1)).unwrap();
    assert!(clamped.wrap_text);
    assert_eq!(clamped.text_max_lines, 2);
}

#[test]
fn graphic_sources_are_stored_as_typed_state() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let svg = applier.document.atoms.borrow_mut().intern("svg");
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: svg,
    });

    applier.apply_op(&Op::SetGraphicSource {
        id: NodeKey::new(2, 1),
        kind: crate::protocol::GRAPHIC_SOURCE_SVG,
        source: "<svg viewBox='0 0 1 1'/>",
    });
    let node = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    assert_eq!(
        applier.document.node_store.declared[&node]
            .svg_source
            .as_deref(),
        Some("<svg viewBox='0 0 1 1'/>")
    );

    applier.apply_op(&Op::ClearGraphicSource {
        id: NodeKey::new(2, 1),
        kind: crate::protocol::GRAPHIC_SOURCE_SVG,
    });
    assert!(
        applier.document.node_store.declared[&node]
            .svg_source
            .is_none()
    );
}

#[test]
fn large_sibling_tree_only_reprojects_when_ifc_inputs_change() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let view = applier.document.atoms.borrow_mut().intern("view");
    let mut ops = Vec::with_capacity(8192);
    for id in 2..=4097 {
        ops.push(Op::CreateElement {
            id: nk(id),
            tag: view,
        });
        ops.push(Op::AppendChild {
            parent: NodeKey::new(1, 1),
            child: nk(id),
        });
    }

    applier.apply_frame(&Frame { seq: 1, ops });

    assert_eq!(
        applier.document.node_store.children[&applier.document.node_store.root].len(),
        4096
    );
    assert_eq!(
        applier
            .document
            .node_store
            .tree
            .children(applier.document.node_store.root)
            .unwrap()
            .len(),
        4096
    );
    assert_eq!(applier.document.ifc_projection_count, 1);

    applier.apply_frame(&Frame {
        seq: 2,
        ops: vec![Op::SetTransform2D {
            id: NodeKey::new(2, 1),
            matrix: [1.0, 0.0, 0.0, 1.0, 8.0, 4.0],
        }],
    });
    assert_eq!(applier.document.ifc_projection_count, 1);

    applier.apply_frame(&Frame {
        seq: 3,
        ops: Vec::new(),
    });
    assert_eq!(applier.document.ifc_projection_count, 1);

    let display = applier.document.atoms.borrow_mut().intern("display");
    applier.apply_frame(&Frame {
        seq: 4,
        ops: vec![Op::SetStyle {
            id: NodeKey::new(2, 1),
            prop: display,
            value: "none",
        }],
    });
    assert_eq!(applier.document.ifc_projection_count, 2);
}

#[test]
fn app_directory_effect_uses_host_configuration_only() {
    let directories = wabou_shell::AppDirectories::resolve(
        &wabou_shell::AppDirectoryConfig::new("dev", "Wabou", "Effect Test"),
        "/app/resources",
    )
    .unwrap();
    let configured = decode_effect_payload(
        wabou_shell::effect::builtin::APP_DIRS_RESOLVE,
        wabou_shell::initial_window_resource_key(0),
        "null".into(),
        Some(&directories),
    );
    assert_eq!(
        configured,
        wabou_shell::EffectPayload::AppDirsResolve(directories)
    );

    assert!(matches!(
        decode_effect_payload(
            wabou_shell::effect::builtin::APP_DIRS_RESOLVE,
            wabou_shell::initial_window_resource_key(0),
            r#"{"application":"other"}"#.into(),
            None,
        ),
        wabou_shell::EffectPayload::Invalid { .. }
    ));
}

#[test]
fn window_show_effect_restores_a_logical_window() {
    assert_eq!(
        decode_effect_payload(
            wabou_shell::effect::builtin::WINDOW_SHOW,
            wabou_shell::initial_window_resource_key(0),
            "null".into(),
            None,
        ),
        wabou_shell::EffectPayload::WindowControl {
            window_id: wabou_shell::initial_window_resource_key(0),
            command: wabou_shell::WindowCommand::Show,
        }
    );
}

#[test]
fn application_exit_effect_is_process_scoped_and_payload_free() {
    assert_eq!(
        decode_effect_payload(
            wabou_shell::effect::builtin::APPLICATION_EXIT,
            wabou_shell::initial_window_resource_key(0),
            "null".into(),
            None,
        ),
        wabou_shell::EffectPayload::ApplicationExit
    );
    assert_eq!(
        decode_effect_payload(
            wabou_shell::effect::builtin::APPLICATION_RELAUNCH,
            wabou_shell::initial_window_resource_key(0),
            "null".into(),
            None,
        ),
        wabou_shell::EffectPayload::ApplicationRelaunch
    );
}

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

struct VisibilityLifecycleWidget(Arc<std::sync::Mutex<Vec<&'static str>>>);

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
    fn measure(&mut self, cx: &mut crate::widget::MeasureContext<'_>) -> Option<[f32; 2]> {
        Some(cx.resolve_size(self.0))
    }

    fn paint(&mut self, _cx: &mut wabou_shell::PaintContext<'_>) {}
}

impl crate::widget::Widget for StyleAwareMeasuringWidget {
    fn style_changed(
        &mut self,
        _style: &crate::widget::WidgetStyle,
    ) -> crate::widget::WidgetChanges {
        self.0.lock().unwrap().push("style");
        crate::widget::WidgetChanges::REDRAW
    }

    fn measure(&mut self, cx: &mut crate::widget::MeasureContext<'_>) -> Option<[f32; 2]> {
        self.0.lock().unwrap().push("measure");
        Some(cx.resolve_size([100.0, 40.0]))
    }

    fn paint(&mut self, _cx: &mut wabou_shell::PaintContext<'_>) {}
}

impl crate::widget::Widget for HostActionWidget {
    fn paint(&mut self, _cx: &mut wabou_shell::PaintContext<'_>) {}

    fn poll_async(&mut self) -> bool {
        self.0.is_some()
    }

    fn take_host_action(&mut self) -> Option<wabou_shell::HostAction> {
        self.0.take()
    }
}

impl crate::widget::Widget for EventHostActionWidget {
    fn paint(&mut self, _cx: &mut wabou_shell::PaintContext<'_>) {}

    fn handle_event(&mut self, _event: &UiEvent) -> crate::widget::WidgetEventResult {
        crate::widget::WidgetEventResult::HANDLED
    }

    fn take_host_action(&mut self) -> Option<wabou_shell::HostAction> {
        self.0.take()
    }
}

impl crate::widget::Widget for UnmountActionWidget {
    fn paint(&mut self, _cx: &mut wabou_shell::PaintContext<'_>) {}

    fn unmount(&mut self) {
        self.0 = Some(wabou_shell::HostAction::SetWindowTitle(None));
    }

    fn take_host_action(&mut self) -> Option<wabou_shell::HostAction> {
        self.0.take()
    }
}

impl crate::widget::Widget for LifecycleWidget {
    fn paint(&mut self, _cx: &mut wabou_shell::PaintContext<'_>) {}

    fn focus_changed(&mut self, focused: bool) -> crate::widget::WidgetChanges {
        self.0
            .lock()
            .unwrap()
            .push(if focused { "focus-in" } else { "focus-out" });
        crate::widget::WidgetChanges::REDRAW
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

impl crate::widget::Widget for VisibilityLifecycleWidget {
    fn paint(&mut self, _cx: &mut wabou_shell::PaintContext<'_>) {
        self.0.lock().unwrap().push("paint");
    }

    fn mounted(&mut self) -> crate::widget::WidgetChanges {
        self.0.lock().unwrap().push("mount");
        crate::widget::WidgetChanges::REDRAW
    }

    fn visibility_changed(&mut self, visible: bool) -> crate::widget::WidgetChanges {
        self.0
            .lock()
            .unwrap()
            .push(if visible { "visible" } else { "hidden" });
        crate::widget::WidgetChanges::REDRAW
    }

    fn intrinsic_size(&self) -> Option<[f32; 2]> {
        Some([40.0, 20.0])
    }
}

impl crate::widget::Widget for NodeEventWidget {
    fn paint(&mut self, _cx: &mut wabou_shell::PaintContext<'_>) {}

    fn poll_async(&mut self) -> bool {
        self.0.is_some()
    }

    fn take_node_event(&mut self) -> Option<crate::widget::WidgetNodeEvent> {
        self.0.take()
    }
}

impl crate::widget::Widget for ClipboardReadWidget {
    fn paint(&mut self, _cx: &mut wabou_shell::PaintContext<'_>) {}

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
    fn paint(&mut self, _cx: &mut wabou_shell::PaintContext<'_>) {}

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
    fn paint(&mut self, _cx: &mut wabou_shell::PaintContext<'_>) {}

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
                const eventId = view.getUint32(52, true);
                return { needsTick: true, preventedEventIds: new Uint32Array([eventId]) };
            };
            "#,
        )
    })
    .unwrap();
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.document.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: div,
    });
    applier.apply_op(&Op::AddEventListener {
        id: NodeKey::new(2, 1),
        event_type: event::KEYDOWN,
    });
    let node = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    let received = Arc::new(std::sync::Mutex::new(0));
    applier
        .document
        .widget_manager
        .widgets
        .insert(node, Box::new(KeyCaptureWidget(received.clone())));
    applier.interaction.input.focused_target = Some(NodeKey::new(2, 1));

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
        .runtime
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
    let div = applier.document.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: div,
    });

    applier.apply_op(&Op::FocusNode {
        id: NodeKey::new(2, 1),
    });
    assert_eq!(
        applier.interaction.input.focused_target,
        Some(NodeKey::new(2, 1))
    );

    applier.apply_op(&Op::FocusNode {
        id: NodeKey::new(999, 1),
    });
    assert_eq!(
        applier.interaction.input.focused_target,
        Some(NodeKey::new(2, 1)),
        "a stale JS handle must not clear valid native focus"
    );
}

#[test]
fn widget_measurements_refresh_intrinsic_layout_before_paint() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    js.with(|ctx| {
        ctx.eval::<(), _>(
            "globalThis.__wabou_tick = () => false; globalThis.__wabou_has_raf = () => false;",
        )
    })
    .unwrap();
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.document.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: div,
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });
    let node = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    applier
        .document
        .widget_manager
        .widgets
        .insert(node, Box::new(MeasuringWidget([123.0, 45.0])));
    assert_eq!(
        applier
            .computed_node_snapshot(NodeKey::new(2, 1))
            .unwrap()
            .intrinsic_size,
        None
    );

    let mut tcx = TextContext::new();
    applier.measure_widgets(&mut tcx);

    assert_eq!(
        applier
            .computed_node_snapshot(NodeKey::new(2, 1))
            .unwrap()
            .intrinsic_size,
        Some([123.0, 45.0])
    );
    assert!(
        applier
            .document
            .invalidation
            .contains(InvalidationFlags::LAYOUT)
    );
}

#[test]
fn widget_mount_and_visibility_are_delivered_before_first_paint() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    js.with(|ctx| {
        ctx.eval::<(), _>(
            "globalThis.__wabou_tick = () => false; globalThis.__wabou_has_raf = () => false;",
        )
    })
    .unwrap();
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
    let widget_tag = applier.document.atoms.borrow_mut().intern("input");
    let factory_calls = calls.clone();
    applier.document.widget_manager.factories.insert(
        widget_tag,
        Arc::new(move || Box::new(VisibilityLifecycleWidget(factory_calls.clone()))),
    );
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: widget_tag,
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });

    let mut tcx = TextContext::new();
    let placed = applier.build_frame(&mut tcx, 200, 100);
    assert!(
        placed
            .iter()
            .any(|node| node.node_id
                == applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)])
    );

    assert_eq!(*calls.lock().unwrap(), ["mount", "visible", "paint"]);
}

#[test]
fn widget_styles_are_delivered_once_before_measurement() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
    applier.document.widget_manager.widgets.insert(
        applier.document.node_store.root,
        Box::new(StyleAwareMeasuringWidget(calls.clone())),
    );

    let mut tcx = TextContext::new();
    applier.sync_widget_styles();
    applier.measure_widgets(&mut tcx);
    applier.sync_widget_styles();

    assert_eq!(*calls.lock().unwrap(), ["style", "measure"]);

    let mut paint = applier
        .document
        .node_store
        .tree
        .get_node_context(applier.document.node_store.root)
        .unwrap()
        .clone();
    paint.font_size += 1.0;
    applier
        .document
        .node_store
        .tree
        .set_node_context(applier.document.node_store.root, Some(paint))
        .unwrap();
    applier.sync_widget_styles();

    assert_eq!(*calls.lock().unwrap(), ["style", "measure", "style"]);
}

#[test]
fn wheel_routing_preserves_pointer_position_for_widgets() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let received = Arc::new(std::sync::Mutex::new(Vec::new()));
    applier.document.widget_manager.widgets.insert(
        applier.document.node_store.root,
        Box::new(WheelCaptureWidget(received.clone())),
    );
    applier.interaction.input.hovered_target = Some(NodeKey::new(1, 1));
    applier.interaction.input.pointer_position = (42.0, 73.0);

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
    assert_eq!(std::mem::size_of::<EventMask>(), 8);
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
fn pointer_dispatch_resolves_a_listener_on_the_native_parent_chain() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.document.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: div,
    });
    applier.apply_op(&Op::CreateText {
        id: NodeKey::new(3, 1),
        text: "option",
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(2, 1),
        child: NodeKey::new(3, 1),
    });
    applier.apply_op(&Op::AddEventListener {
        id: NodeKey::new(2, 1),
        event_type: event::POINTERMOVE,
    });

    assert_eq!(
        applier.listener_target_in_chain(nk(3), event::POINTERMOVE),
        Some(NodeKey::new(2, 1))
    );
    assert_eq!(applier.listener_target_in_chain(nk(3), event::CLICK), None);
}

#[test]
fn native_scroll_observations_coalesce_by_target() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    applier
        .interaction
        .input
        .listeners
        .entry(nk(1))
        .or_default()
        .insert(event::SCROLL);
    applier
        .interaction
        .scroll
        .offsets
        .insert(applier.document.node_store.root, [0.0, 12.0]);
    applier.queue_scroll_event(applier.document.node_store.root);
    applier
        .interaction
        .scroll
        .offsets
        .insert(applier.document.node_store.root, [0.0, 48.0]);
    applier.queue_scroll_event(applier.document.node_store.root);

    assert_eq!(applier.interaction.scroll.pending_events.len(), 1);
    assert_eq!(
        applier.interaction.scroll.pending_events[&nk(1)],
        [0.0, 48.0]
    );
}

#[test]
fn widget_host_actions_reach_the_frame_source() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    applier.document.widget_manager.widgets.insert(
        applier.document.node_store.root,
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
        id: NodeKey::new(2, 1),
        event_type: event::TERMINALEXIT,
    });
    let node = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    applier.document.widget_manager.widgets.insert(
        node,
        Box::new(NodeEventWidget(Some(crate::widget::WidgetNodeEvent::json(
            event::TERMINALEXIT,
            r#"{"reason":"exit"}"#,
        )))),
    );

    assert!(FrameSource::poll_async(&mut applier));
    let dispatched = applier
        .runtime
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
    applier.document.widget_manager.widgets.insert(
        applier.document.node_store.root,
        Box::new(EventHostActionWidget(Some(
            wabou_shell::HostAction::OpenUrl("https://example.com".into()),
        ))),
    );

    let response = applier
        .handle_widget_event(nk(1), &UiEvent::Focus(true))
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
    let div = applier.document.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: div,
    });
    let node = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    applier
        .document
        .widget_manager
        .widgets
        .insert(node, Box::new(UnmountActionWidget(None)));

    applier.apply_op(&Op::DropNode {
        id: NodeKey::new(2, 1),
    });

    assert_eq!(
        FrameSource::take_host_action(&mut applier),
        Some(wabou_shell::HostAction::SetWindowTitle(None))
    );
    assert!(!applier.document.widget_manager.widgets.contains_key(&node));
}

#[test]
fn dropping_a_focused_captured_widget_releases_input_before_unmount() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.document.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: div,
    });
    let node = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    let lifecycle = Arc::new(std::sync::Mutex::new(Vec::new()));
    applier
        .document
        .widget_manager
        .widgets
        .insert(node, Box::new(LifecycleWidget(lifecycle.clone())));
    applier.interaction.input.focused_target = Some(NodeKey::new(2, 1));
    applier.interaction.input.pointer_down_target = Some(NodeKey::new(2, 1));
    applier.interaction.input.pointer_down_position = Some((10.0, 20.0));
    applier.interaction.input.pointer_dragged = true;

    applier.apply_op(&Op::DropNode {
        id: NodeKey::new(2, 1),
    });

    assert_eq!(
        *lifecycle.lock().unwrap(),
        ["pointer-cancel", "focus-out", "unmount"]
    );
    assert_eq!(applier.interaction.input.focused_target, None);
    assert_eq!(applier.interaction.input.pointer_down_target, None);
    assert_eq!(applier.interaction.input.pointer_down_position, None);
    assert!(!applier.interaction.input.pointer_dragged);
}

#[test]
fn window_focus_loss_cancels_the_captured_pointer_before_blur() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.document.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: div,
    });
    let node = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    let lifecycle = Arc::new(std::sync::Mutex::new(Vec::new()));
    applier
        .document
        .widget_manager
        .widgets
        .insert(node, Box::new(LifecycleWidget(lifecycle.clone())));
    applier.interaction.input.focused_target = Some(NodeKey::new(2, 1));
    applier.interaction.input.pointer_down_target = Some(NodeKey::new(2, 1));
    applier.interaction.input.pointer_down_position = Some((10.0, 20.0));
    applier.interaction.input.pointer_position = (15.0, 25.0);
    applier.interaction.input.pointer_buttons = 1;
    applier.interaction.input.pointer_dragged = true;
    applier.interaction.text_selection.last_click = Some((Instant::now(), nk(2), 15.0, 25.0, 1));

    let blurred = applier.handle_event(UiEvent::Focus(false));

    assert_eq!(*lifecycle.lock().unwrap(), ["pointer-cancel", "focus-out"]);
    assert_eq!(blurred.text_input, Some(false));
    assert_eq!(
        applier.interaction.input.focused_target,
        Some(NodeKey::new(2, 1))
    );
    assert!(!applier.interaction.input.window_focused);
    assert_eq!(applier.interaction.input.pointer_down_target, None);
    assert_eq!(applier.interaction.input.pointer_down_position, None);
    assert_eq!(applier.interaction.input.pointer_buttons, 0);
    assert!(!applier.interaction.input.pointer_dragged);
    assert!(applier.interaction.text_selection.last_click.is_none());

    let focused = applier.handle_event(UiEvent::Focus(true));
    assert_eq!(focused.text_input, Some(false));
    assert_eq!(
        *lifecycle.lock().unwrap(),
        ["pointer-cancel", "focus-out", "focus-in"]
    );
    assert_eq!(
        applier.interaction.input.focused_target,
        Some(NodeKey::new(2, 1))
    );
    assert!(applier.interaction.input.window_focused);

    applier.interaction.text_selection.last_click = Some((Instant::now(), nk(2), 15.0, 25.0, 1));
    applier.handle_event(UiEvent::TextInput("x".into()));
    assert!(
        applier.interaction.text_selection.last_click.is_none(),
        "text input must break a native text multi-click sequence"
    );
}

#[test]
fn clipboard_read_completions_route_to_the_requesting_widget() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.document.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: div,
    });
    let second_node = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    let first_completed = Arc::new(std::sync::Mutex::new(Vec::new()));
    let second_completed = Arc::new(std::sync::Mutex::new(Vec::new()));
    applier.document.widget_manager.widgets.insert(
        applier.document.node_store.root,
        Box::new(ClipboardReadWidget {
            action: Some(wabou_shell::HostAction::ReadClipboard { request_id: 7 }),
            completed: first_completed.clone(),
        }),
    );
    applier.document.widget_manager.widgets.insert(
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
        let (node, _) = applier.document.widget_manager.host_action_routes[&request_id];
        let text = if node == applier.document.node_store.root {
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
    pointer_with_button(phase, x, y, buttons, PointerButton::Primary)
}

fn pointer_with_button(
    phase: PointerPhase,
    x: f64,
    y: f64,
    buttons: u32,
    button: PointerButton,
) -> UiEvent {
    UiEvent::Pointer(PointerEvent {
        phase,
        position: Point { x, y },
        button: Some(button),
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
            const id = view.getUint32(o, true), code = view.getUint8(o + 8);
            const payloadKind = view.getUint8(o + 9); o += 16;
            let payload = "";
            if (payloadKind === 2) {
              const size = view.getUint32(o, true); o += 4;
              payload = dec.decode(u8.subarray(o, o + size));
            }
            globalThis.dispatched.push([id, code, payload]);
          } else if (kind === 2) {
            globalThis.resizeChanges.push([
              view.getUint32(o, true), view.getFloat32(o + 8, true), view.getFloat32(o + 12, true)
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
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("button"),
            atoms.intern("width"),
            atoms.intern("height"),
        )
    };
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: button,
    });
    applier.apply_op(&Op::SetStyle {
        id: NodeKey::new(2, 1),
        prop: width,
        value: "100px",
    });
    applier.apply_op(&Op::SetStyle {
        id: NodeKey::new(2, 1),
        prop: height,
        value: "50px",
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });
    for code in [
        event::POINTERDOWN,
        event::POINTERMOVE,
        event::POINTERUP,
        event::CLICK,
        event::CONTEXTMENU,
    ] {
        applier.apply_op(&Op::AddEventListener {
            id: NodeKey::new(2, 1),
            event_type: code,
        });
    }
    let mut root_style = applier
        .document
        .node_store
        .tree
        .style(applier.document.node_store.root)
        .expect("root style")
        .clone();
    root_style.size.width = taffy::Dimension::length(800.0);
    root_style.size.height = taffy::Dimension::length(600.0);
    applier
        .document
        .node_store
        .tree
        .set_style(applier.document.node_store.root, root_style)
        .expect("viewport style");
    applier
        .document
        .node_store
        .tree
        .compute_layout(
            applier.document.node_store.root,
            taffy::geometry::Size {
                width: taffy::AvailableSpace::Definite(800.0),
                height: taffy::AvailableSpace::Definite(600.0),
            },
        )
        .expect("layout");
    let mut placed = layout::flatten_with_scroll(
        &applier.document.node_store.tree,
        applier.document.node_store.root,
        &applier.interaction.scroll.offsets,
    );
    applier.update_scrollbar_visuals(&mut placed);
    applier.rebuild_hit_geometry(&placed);
    applier
}

#[test]
fn focus_order_is_explicit_without_inferring_disabled_policy() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    js.with(|ctx| {
        ctx.eval::<(), _>(
            "globalThis.__wabou_tick = () => false; globalThis.__wabou_has_raf = () => false;",
        )
    })
    .unwrap();
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (button, disabled, width, height) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("button"),
            atoms.intern("disabled"),
            atoms.intern("width"),
            atoms.intern("height"),
        )
    };
    for id in 2..=6 {
        applier.apply_op(&Op::CreateElement {
            id: nk(id),
            tag: button,
        });
        applier.apply_op(&Op::AppendChild {
            parent: NodeKey::new(1, 1),
            child: nk(id),
        });
        applier.apply_op(&Op::SetStyle {
            id: nk(id),
            prop: width,
            value: "100px",
        });
        applier.apply_op(&Op::SetStyle {
            id: nk(id),
            prop: height,
            value: "20px",
        });
    }
    set_focus_order(&mut applier, 2, 0);
    set_focus_order(&mut applier, 3, 2);
    set_focus_order(&mut applier, 4, -1);
    set_focus_order(&mut applier, 5, 1);
    set_focus_order(&mut applier, 6, -1);
    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(6, 1),
        name: disabled,
        value: "",
    });
    let mut tcx = TextContext::new();
    let placed = FrameSource::build_frame(&mut applier, &mut tcx, 800, 600);
    assert!(placed.len() >= 6, "placed node count: {}", placed.len());

    assert_eq!(applier.interaction.input.focus_order, [nk(5), nk(3), nk(2)]);
    assert!(applier.interaction.input.focusable_targets.contains(&nk(4)));
    assert!(applier.interaction.input.focusable_targets.contains(&nk(6)));
    assert_eq!(applier.advance_focus(false), Some(NodeKey::new(5, 1)));
    assert_eq!(applier.advance_focus(false), Some(NodeKey::new(3, 1)));
    assert_eq!(applier.advance_focus(true), Some(NodeKey::new(5, 1)));
}

#[test]
fn accessibility_attributes_do_not_create_or_remove_focus_behavior() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    js.with(|ctx| {
        ctx.eval::<(), _>(
            "globalThis.__wabou_tick = () => false; globalThis.__wabou_has_raf = () => false;",
        )
    })
    .unwrap();
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (view, button, role, disabled, aria_disabled, aria_hidden, width, height) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("view"),
            atoms.intern("button"),
            atoms.intern("role"),
            atoms.intern("disabled"),
            atoms.intern("aria-disabled"),
            atoms.intern("aria-hidden"),
            atoms.intern("width"),
            atoms.intern("height"),
        )
    };
    for (id, tag) in [(2, view), (3, view), (4, button), (5, view)] {
        applier.apply_op(&Op::CreateElement { id: nk(id), tag });
        applier.apply_op(&Op::AppendChild {
            parent: NodeKey::new(1, 1),
            child: nk(id),
        });
        applier.apply_op(&Op::SetStyle {
            id: nk(id),
            prop: width,
            value: "100px",
        });
        applier.apply_op(&Op::SetStyle {
            id: nk(id),
            prop: height,
            value: "20px",
        });
    }
    for id in [2, 3, 5] {
        applier.apply_op(&Op::SetAttribute {
            id: nk(id),
            name: role,
            value: if id == 5 { "textbox" } else { "button" },
        });
    }
    set_focus_order(&mut applier, 3, 0);
    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(3, 1),
        name: aria_disabled,
        value: "true",
    });
    set_focus_order(&mut applier, 4, 0);
    // Native behavior props are not accessibility policy. JS must publish
    // semantic state explicitly through the semantic contract.
    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(5, 1),
        name: disabled,
        value: "",
    });
    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(4, 1),
        name: aria_hidden,
        value: "true",
    });
    set_focus_order(&mut applier, 5, -1);

    let mut tcx = TextContext::new();
    let placed = FrameSource::build_frame(&mut applier, &mut tcx, 800, 600);
    applier.rebuild_semantic_snapshot(&placed);

    assert_eq!(applier.interaction.input.focus_order, [nk(3), nk(4)]);
    assert!(!applier.interaction.input.focusable_targets.contains(&nk(2)));
    assert!(applier.interaction.input.focusable_targets.contains(&nk(3)));
    assert!(applier.interaction.input.focusable_targets.contains(&nk(4)));
    assert!(applier.interaction.input.focusable_targets.contains(&nk(5)));
    let semantic = &applier.frame.projections.semantic_snapshot.nodes;
    assert!(
        semantic
            .iter()
            .find(|node| node.id == sk(3))
            .unwrap()
            .disabled
    );
    assert!(
        !semantic
            .iter()
            .find(|node| node.id == sk(5))
            .unwrap()
            .disabled
    );
}

#[test]
fn interaction_blocking_isolates_an_entire_subtree() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (view, button, aria_hidden, width, height) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("view"),
            atoms.intern("button"),
            atoms.intern("aria-hidden"),
            atoms.intern("width"),
            atoms.intern("height"),
        )
    };
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: view,
    });
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(3, 1),
        tag: button,
    });
    set_focus_order(&mut applier, 3, 0);
    for id in [2, 3] {
        applier.apply_op(&Op::SetStyle {
            id: nk(id),
            prop: width,
            value: "100px",
        });
        applier.apply_op(&Op::SetStyle {
            id: nk(id),
            prop: height,
            value: "50px",
        });
    }
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(2, 1),
        child: NodeKey::new(3, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });
    applier.rebuild_layout_boxes();
    let mut root_style = applier
        .document
        .node_store
        .tree
        .style(applier.document.node_store.root)
        .unwrap()
        .clone();
    root_style.size.width = taffy::Dimension::length(200.0);
    root_style.size.height = taffy::Dimension::length(200.0);
    applier
        .document
        .node_store
        .tree
        .set_style(applier.document.node_store.root, root_style)
        .unwrap();
    applier
        .document
        .node_store
        .tree
        .compute_layout(
            applier.document.node_store.root,
            taffy::geometry::Size {
                width: taffy::AvailableSpace::Definite(200.0),
                height: taffy::AvailableSpace::Definite(200.0),
            },
        )
        .unwrap();
    let placed = layout::flatten_with_scroll(
        &applier.document.node_store.tree,
        applier.document.node_store.root,
        &applier.interaction.scroll.offsets,
    );

    applier.rebuild_hit_geometry(&placed);
    applier.rebuild_focus_order(&placed);
    applier.rebuild_semantic_snapshot(&placed);
    assert_eq!(
        applier.interaction.input.hit_test(10.0, 10.0),
        Some(NodeKey::new(3, 1))
    );
    assert_eq!(applier.interaction.input.focus_order, [nk(3)]);
    assert!(
        applier
            .frame
            .projections
            .semantic_snapshot
            .nodes
            .iter()
            .any(|node| node.id == sk(3))
    );

    set_interaction_blocked(&mut applier, 2, true);
    applier.rebuild_hit_geometry(&placed);
    applier.rebuild_focus_order(&placed);
    applier.rebuild_semantic_snapshot(&placed);
    assert_eq!(
        applier.interaction.input.hit_test(10.0, 10.0),
        Some(NodeKey::new(1, 1))
    );
    assert!(applier.interaction.input.focus_order.is_empty());
    assert!(
        applier
            .frame
            .projections
            .semantic_snapshot
            .nodes
            .iter()
            .all(|node| node.id != 2 && node.id != 3)
    );
    assert!(!applier.handle_semantic_action(SemanticAction::Click { target: sk(3) }));
    assert!(!applier.handle_semantic_action(SemanticAction::Focus { target: sk(3) }));

    set_interaction_blocked(&mut applier, 2, false);
    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(2, 1),
        name: aria_hidden,
        value: "true",
    });
    applier.rebuild_hit_geometry(&placed);
    applier.rebuild_focus_order(&placed);
    applier.rebuild_semantic_snapshot(&placed);
    assert_eq!(
        applier.interaction.input.hit_test(10.0, 10.0),
        Some(NodeKey::new(3, 1))
    );
    assert_eq!(applier.interaction.input.focus_order, [nk(3)]);
    assert!(
        applier
            .frame
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
    let div = applier.document.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: div,
    });
    let node = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    let received = Arc::new(std::sync::Mutex::new(0));
    applier
        .document
        .widget_manager
        .widgets
        .insert(node, Box::new(KeyCaptureWidget(received.clone())));
    applier.interaction.input.focused_target = Some(NodeKey::new(2, 1));
    applier.interaction.input.focus_order = vec![nk(2), nk(3)];

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
    assert_eq!(
        applier.interaction.input.focused_target,
        Some(NodeKey::new(2, 1))
    );
    assert_eq!(*received.lock().unwrap(), 1);
}

mod overlay_cases;
mod projection_cases;
mod runtime_cases;
mod text_cases;
fn nk(lo: u32) -> NodeKey {
    NodeKey::new(lo, 1)
}

fn sk(lo: u32) -> u64 {
    nk(lo).into()
}
