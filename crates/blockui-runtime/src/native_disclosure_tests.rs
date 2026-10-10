use super::*;
use blockui::{
    Accordion, AccordionItem, Action, Collapsible, Column, Cx, Key, State, StateSpec, Text,
    TextInput, View,
};
const SINGLE: State<i64> = State::<i64>::new("disclosure.single", 0);
const MULTIPLE: State<Vec<i64>> = State::<Vec<i64>>::new("disclosure.multiple");
const OPEN: State<bool> = State::<bool>::new("disclosure.open", false);
const DRAFT: State<String> = State::<String>::new("disclosure.draft");
const CLOSE: Action = Action::new("disclosure.close", |cx, _| SINGLE.set(cx, Accordion::NONE));
struct Disclosures;
impl blockui::Actor for Disclosures {
    const STATE: &'static [StateSpec] =
        &[SINGLE.spec(), MULTIPLE.spec(), OPEN.spec(), DRAFT.spec()];
    const ACTIONS: &'static [Action] = &[CLOSE];
    fn view(cx: &Cx<'_>) -> blockui::Result<View> {
        Ok(Column {
            gap: 16.0,
            padding: 24.0,
            ..Default::default()
        }
        .children(vec![
            Accordion::single(SINGLE)
                .collapsible(true)
                .key("single")
                .children(
                    cx,
                    vec![
                        AccordionItem::new(0, "Account", TextInput::new(DRAFT).key("draft")),
                        AccordionItem::new(1, "Disabled", Text::new("Unavailable")).disabled(),
                        AccordionItem::new(
                            2,
                            "Security",
                            Text::new("Security settings").key("security"),
                        ),
                    ],
                )?,
            Accordion::multiple(MULTIPLE).key("multiple").children(
                cx,
                vec![
                    AccordionItem::new(0, "First", Text::new("First details")),
                    AccordionItem::new(1, "Second", Text::new("Second details")),
                ],
            )?,
            Collapsible::new("Advanced options", OPEN)
                .key("advanced")
                .children(cx, Text::new("Tracing and diagnostics").key("details"))?,
        ]))
    }
}
fn trigger(group: &str, value: u64) -> u64 {
    Key::named(group)
        .scoped(value)
        .scoped(blockui_abi::key("trigger"))
        .0
}
fn fixture() -> Result<(NativeSource, TextContext), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<Disclosures>()?)?;
    source.set_semantics_enabled(true);
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 480, 900);
    Ok((source, tcx))
}
fn key_event(key: &str) -> UiEvent {
    UiEvent::Key(shell::KeyEvent {
        key: key.into(),
        key_without_modifiers: key.into(),
        code: key.into(),
        text: None,
        text_with_all_modifiers: None,
        location: Default::default(),
        phase: shell::KeyPhase::Down,
        modifiers: Default::default(),
        repeat: false,
        synthetic: false,
    })
}
#[test]
fn arrows_move_focus_without_toggling_and_activation_has_exact_semantics() -> Result<(), String> {
    let (mut source, mut tcx) = fixture()?;
    let account = trigger("single", 0);
    let security = trigger("single", 2);
    source.set_focus(Some(account));
    source.handle_event(key_event("ArrowDown"));
    assert_eq!(source.focus, Some(security));
    assert_eq!(source.actor.integer("disclosure.single"), 0);
    source.handle_event(key_event("Enter"));
    source.build_frame(&mut tcx, 480, 900);
    assert_eq!(source.actor.integer("disclosure.single"), 2);
    let snapshot = source.semantic_snapshot().unwrap();
    let trigger = snapshot
        .nodes
        .iter()
        .find(|node| node.id == security)
        .unwrap();
    assert_eq!(trigger.states.expanded, Some(true));
    assert_eq!(trigger.role, shell::SemanticRole::Button);
    let panel = source.actor.view().bindings[&security]
        .node
        .semantics
        .controls;
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|node| node.id == panel)
            .unwrap()
            .role,
        shell::SemanticRole::Region
    );
    let closed = source.actor.view().bindings[&account]
        .node
        .semantics
        .controls;
    assert!(!snapshot.nodes.iter().any(|node| node.id == closed));
    source.handle_event(key_event("ArrowDown"));
    assert_eq!(source.focus, Some(account));
    source.handle_event(key_event("End"));
    assert_eq!(source.focus, Some(security));
    source.handle_event(key_event("Home"));
    assert_eq!(source.focus, Some(account));
    source.focus_next(false);
    assert_eq!(
        source.focus,
        Some(security),
        "Tab visits every enabled header and skips disabled ones"
    );
    Ok(())
}
#[test]
fn programmatic_close_restores_focus_and_reopen_preserves_draft() -> Result<(), String> {
    let (mut source, mut tcx) = fixture()?;
    source.actor.edit_text(key("draft"), "Retained 🦀")?;
    source.set_focus(Some(key("draft")));
    source.actor.tell(CLOSE, 0)?;
    source.actor.tick()?;
    source.build_frame(&mut tcx, 480, 900);
    assert_eq!(source.focus, Some(trigger("single", 0)));
    assert!(!source.ids.contains_key(&key("draft")));
    source.handle_event(key_event("Enter"));
    source.build_frame(&mut tcx, 480, 900);
    assert_eq!(source.actor.text("disclosure.draft"), "Retained 🦀");
    assert!(source.ids.contains_key(&key("draft")));
    Ok(())
}

#[test]
#[ignore = "explicit Vello accordion separators and chevrons at 1x and 2x"]
fn disclosure_visuals() -> Result<(), String> {
    for scale in [1, 2] {
        let (mut source, mut tcx) = fixture()?;
        source.scale = scale as f64;
        let nodes = source.build_frame(&mut tcx, 480, 900);
        let item = nodes
            .iter()
            .find(|node| node.node_id == source.ids[&Key::named("single").scoped(0).0])
            .unwrap();
        let header = nodes
            .iter()
            .find(|node| node.node_id == source.ids[&trigger("single", 0)])
            .unwrap();
        let mut scene = shell::Scene::new();
        shell::scene::build_scene_scaled(
            &mut scene,
            &nodes,
            &mut tcx,
            480,
            900,
            source.base_color(),
            scale as f64,
        );
        let image =
            shell::renderer::render_to_image(&scene, 480 * scale, 900 * scale, source.base_color())
                .map_err(|e| e.to_string())?;
        let line = image.get_pixel(
            ((item.rect[0] + item.rect[2]) * 0.5 * scale as f32) as u32,
            ((item.rect[3] - 1.0) * scale as f32) as u32,
        );
        for (channel, expected) in [226u8, 230, 236].iter().enumerate() {
            assert!(
                line[channel].abs_diff(*expected) <= 2,
                "separator at scale {scale}: {line:?}"
            );
        }
        let x = header.content_origin[0] + header.content_size[0] + 14.0;
        let y = header.content_origin[1] + header.content_size[1] * 0.5;
        let mut ink = 0;
        for px in ((x - 5.0) * scale as f32) as u32..((x + 5.0) * scale as f32) as u32 {
            for py in ((y - 4.0) * scale as f32) as u32..((y + 4.0) * scale as f32) as u32 {
                if image.get_pixel(px, py)[0] < 180 {
                    ink += 1;
                }
            }
        }
        assert!(
            ink >= 8 * scale,
            "missing expanded chevron at scale {scale}"
        );
        if scale == 1 {
            std::fs::create_dir_all("../../reports").map_err(|e| e.to_string())?;
            image
                .save("../../reports/native-accordion.png")
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[test]
fn nested_accordion_keys_stay_in_their_group_and_outer_close_recovers_focus() -> Result<(), String>
{
    const INNER: State<i64> = State::<i64>::new("disclosure.inner", 0);
    struct Nested;
    impl blockui::Actor for Nested {
        const STATE: &'static [StateSpec] = &[SINGLE.spec(), INNER.spec(), DRAFT.spec()];
        const ACTIONS: &'static [Action] = &[CLOSE];
        fn view(cx: &Cx<'_>) -> blockui::Result<View> {
            Ok(Column::default().children(vec![
                Accordion::single(SINGLE)
                    .collapsible(true)
                    .key("outer")
                    .children(
                        cx,
                        vec![
                            AccordionItem::new(
                                0,
                                "Outer first",
                                Accordion::single(INNER).key("inner").children(
                                    cx,
                                    vec![
                                        AccordionItem::new(
                                            0,
                                            "Inner first",
                                            TextInput::new(DRAFT).key("nested.input"),
                                        ),
                                        AccordionItem::new(
                                            1,
                                            "Inner last",
                                            Text::new("Inner details"),
                                        ),
                                    ],
                                )?,
                            ),
                            AccordionItem::new(2, "Outer last", Text::new("Outer details")),
                        ],
                    )?,
            ]))
        }
    }
    let mut source = NativeSource::without_watch(Actor::for_test::<Nested>()?)?;
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 480, 900);
    source.set_semantics_enabled(true);
    assert!(
        source
            .semantic_snapshot()
            .unwrap()
            .nodes
            .iter()
            .find(|node| node.id == trigger("inner", 0))
            .unwrap()
            .disabled
    );
    source.set_focus(Some(trigger("inner", 0)));
    source.handle_event(key_event("End"));
    assert_eq!(source.focus, Some(trigger("inner", 1)));
    source.handle_event(key_event("ArrowDown"));
    assert_eq!(source.focus, Some(trigger("inner", 0)));
    assert_eq!(source.actor.integer("disclosure.inner"), 0);
    source.set_focus(Some(key("nested.input")));
    source.actor.tell(CLOSE, 0)?;
    source.actor.tick()?;
    source.build_frame(&mut tcx, 480, 900);
    assert_eq!(source.focus, Some(trigger("outer", 0)));
    Ok(())
}

#[test]
fn chevron_motion_uses_host_time_without_recomputing_state_or_layout() -> Result<(), String> {
    use blockui_motion::{Key as MotionKey, Property};
    let (mut source, mut tcx) = fixture()?;
    source.use_manual_motion_for_test();
    let header = trigger("single", 0);
    let track = MotionKey::new(header, Property::Disclosure);
    assert_eq!(source.motion.value(track), Some(1.0));
    source.actor.dispatch_widget(header)?;
    source.build_frame(&mut tcx, 480, 900);
    let evaluations = source.actor.view().bindings[&header].evaluations;
    source.advance_motion_for_test(Duration::from_millis(60));
    source.sync()?;
    assert_eq!(source.motion.value(track), Some(0.125));
    assert_eq!(
        source.actor.view().bindings[&header].evaluations,
        evaluations
    );
    for id in source.ids.values() {
        assert!(!source.tree.dirty(*id).unwrap());
    }
    source.actor.dispatch_widget(header)?;
    source.sync()?;
    assert_eq!(source.motion.value(track), Some(0.125));
    source.advance_motion_for_test(Duration::from_millis(60));
    source.sync()?;
    assert_eq!(source.motion.value(track), Some(0.890625));
    source.set_reduced_motion(true);
    assert_eq!(source.motion.value(track), Some(1.0));
    assert!(!source.has_anim());
    Ok(())
}

#[test]
fn long_headers_wrap_on_narrow_windows_and_keep_their_widget_identity() -> Result<(), String> {
    struct Narrow;
    impl blockui::Actor for Narrow {
        const STATE: &'static [StateSpec] = &[SINGLE.spec()];
        const ACTIONS: &'static [Action] = &[];
        fn view(cx: &Cx<'_>) -> blockui::Result<View> {
            Ok(Column { padding: 24.0, gap: 0.0, ..Default::default() }.children(vec![
                Accordion::single(SINGLE).key("narrow").children(cx, vec![
                    AccordionItem::new(0, "Can I keep a very long heading readable when the application window becomes narrow?", Text::new("Details")),
                ])?,
            ]))
        }
    }
    let mut source = NativeSource::without_watch(Actor::for_test::<Narrow>()?)?;
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 480, 900);
    let header = trigger("narrow", 0);
    let id = source.ids[&header];
    let wide = source.widget_layout_rect(header).unwrap();
    let nodes = source.build_frame(&mut tcx, 240, 900);
    let narrow = source.widget_layout_rect(header).unwrap();
    assert!(
        narrow[3] - narrow[1] > wide[3] - wide[1],
        "long header must gain height when wrapping"
    );
    assert!(narrow[2] <= 240.0);
    assert_eq!(source.ids[&header], id);
    let placed = nodes.iter().find(|node| node.node_id == id).unwrap();
    assert!(placed.content_origin[0] + placed.content_size[0] + 18.0 <= narrow[2]);
    Ok(())
}
