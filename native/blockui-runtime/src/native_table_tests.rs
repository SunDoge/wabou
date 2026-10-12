use super::*;
use blockui::ActionToken;
use blockui::{Column, IntoView, Table, TableRow, Text, TextInput};
const DRAFT: blockui::State<String> =
    blockui::State::<String>::with_default("table.draft", "Draft 🦀");
const REVERSE: blockui::State<bool> = blockui::State::<bool>::new("table.reverse", false);
const FIXED: blockui::State<bool> = blockui::State::<bool>::new("table.fixed", false);
const RESIZE: blockui::Action = blockui::Action::new("table.resize", |cx, _| FIXED.set(cx, true));
const SORT: blockui::Action =
    blockui::Action::new("table.sort", |cx, _| REVERSE.set(cx, !REVERSE.get(cx)?));
struct Fixture;
impl blockui::Actor for Fixture {
    const STATE: &'static [blockui::StateSpec] = &[DRAFT.spec(), REVERSE.spec(), FIXED.spec()];
    const ACTIONS: &'static [blockui::Action] = &[SORT, RESIZE];
    fn view(cx: &blockui::Cx<'_>) -> blockui::Result<blockui::View> {
        let mut rows = vec![
            TableRow::new(
                "one",
                vec![
                    Text::new("INV001").into_view(),
                    TextInput::new(DRAFT).key("draft").into_view(),
                ],
            ),
            TableRow::new(
                "two",
                vec![
                    Text::new("INV002").into_view(),
                    Text::new("Paid").into_view(),
                ],
            ),
        ];
        if REVERSE.get(cx)? {
            rows.reverse();
        }
        Ok(Column {
            padding: 16.0,
            ..Default::default()
        }
        .key("root")
        .children(vec![
            Table::new(
                "Invoice drafts",
                vec![
                    Text::new("Invoice").into_view(),
                    Text::new("Status").into_view(),
                ],
            )
            .key("table")
            .sorted_column(
                0,
                if REVERSE.get(cx)? {
                    blockui::TableSortDirection::Descending
                } else {
                    blockui::TableSortDirection::Ascending
                },
            )
            .column_widths(if FIXED.get(cx)? {
                vec![180.0, 180.0]
            } else {
                vec![104.0, 0.0]
            })
            .rows(rows)?,
        ]))
    }
}
#[test]
fn table_semantics_and_reordering_preserve_native_editor() -> Result<(), String> {
    let mut s = NativeSource::without_watch(Actor::for_test::<Fixture>()?)?;
    s.set_semantics_enabled(true);
    let mut tcx = TextContext::new();
    s.build_frame(&mut tcx, 360, 400);
    s.set_focus(Some(key("draft")));
    s.actor.edit_text(key("draft"), "Retained 🦀")?;
    s.build_frame(&mut tcx, 360, 400);
    let id = s.ids[&key("draft")];
    let before = s.widget_layout_rect(key("draft")).unwrap();
    let editor = std::ptr::from_ref(&*s.editors[&key("draft")]);
    s.actor.dispatch(Message {
        action: SORT.encode(0).0,
        value: 0,
    })?;
    s.build_frame(&mut tcx, 360, 400);
    assert_eq!(s.ids[&key("draft")], id);
    assert_eq!(std::ptr::from_ref(&*s.editors[&key("draft")]), editor);
    assert_eq!(s.focus, Some(key("draft")));
    assert_eq!(s.actor.text("table.draft"), "Retained 🦀");
    assert!(s.widget_layout_rect(key("draft")).unwrap()[1] > before[1]);
    let snapshot = s.semantic_snapshot().unwrap();
    let header = blockui::Key::named("table")
        .scoped(blockui::abi::key("header"))
        .scoped(0);
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|node| node.id == header.0)
            .unwrap()
            .states
            .sort_direction,
        Some(shell::SemanticSortDirection::Descending)
    );
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|node| node.id == header.0)
            .unwrap()
            .label
            .as_deref(),
        Some("Invoice")
    );

    for (role, count) in [
        (shell::SemanticRole::Table, 1),
        (shell::SemanticRole::Row, 3),
        (shell::SemanticRole::ColumnHeader, 2),
        (shell::SemanticRole::Cell, 4),
    ] {
        assert_eq!(
            snapshot.nodes.iter().filter(|n| n.role == role).count(),
            count
        );
    }
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|n| n.role == shell::SemanticRole::Table)
            .unwrap()
            .label
            .as_deref(),
        Some("Invoice drafts")
    );
    s.set_focus(None);
    s.focus_next(false);
    assert_eq!(s.focus, Some(key("draft")));
    s.focus_next(false);
    assert_eq!(s.focus, Some(key("draft")));
    for n in &s.placed {
        assert!(n.rect.iter().all(|n| n.is_finite()));
        assert!(n.rect[0] >= 0.0 && n.rect[2] <= 360.0, "{:?}", n.rect);
    }
    Ok(())
}
#[test]
#[ignore = "explicit offscreen Vello table capture"]
fn table_visuals() -> Result<(), String> {
    let directory =
        std::env::temp_dir().join(format!("blockui-table-capture-{}", std::process::id()));
    let gallery = std::env::var_os("BLOCKUI_TABLE_LIBRARY");
    let actor = if let Some(path) = &gallery {
        let mut actor = Actor::new(952, &directory)?;
        actor.reload(&std::path::PathBuf::from(path))?;
        actor.dispatch_widget(key("nav.13"))?;
        actor
    } else {
        Actor::for_test::<Fixture>()?
    };
    let (width, height, output) = if gallery.is_some() {
        (1200, 1200, "../../reports/gallery-table-native.png")
    } else {
        (360, 260, "../../reports/native-table.png")
    };
    let mut s = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    let nodes = s.build_frame(&mut tcx, width, height);
    let mut scene = shell::Scene::new();
    shell::scene::build_scene_scaled(
        &mut scene,
        &nodes,
        &mut tcx,
        width,
        height,
        s.base_color(),
        1.0,
    );
    shell::renderer::render_to_image(&scene, width, height, s.base_color())
        .map_err(|e| e.to_string())?
        .save(output)
        .map_err(|e| e.to_string())?;
    drop(s);
    if directory.exists() {
        assert_eq!(
            std::fs::read_dir(&directory)
                .map_err(|e| e.to_string())?
                .count(),
            0
        );
        std::fs::remove_dir(directory).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[test]
fn column_width_changes_align_rows_and_shrink_without_replacing_editors() -> Result<(), String> {
    let mut s = NativeSource::without_watch(Actor::for_test::<Fixture>()?)?;
    let mut tcx = TextContext::new();
    s.build_frame(&mut tcx, 360, 400);
    let header = blockui::Key::named("table").scoped(key("header"));
    let record = blockui::Key::named("table")
        .scoped(key("record"))
        .scoped(key("one"));
    let widths = |s: &NativeSource| {
        [header, record].map(|row| {
            [0, 1].map(|column| {
                let rect = s.widget_layout_rect(row.scoped(column).0).unwrap();
                rect[2] - rect[0]
            })
        })
    };
    assert_eq!(widths(&s), [[104.0, 224.0]; 2]);
    s.set_focus(Some(key("draft")));
    let editor = std::ptr::from_ref(&*s.editors[&key("draft")]);
    s.actor.dispatch(Message {
        action: RESIZE.encode(0).0,
        value: 0,
    })?;
    s.build_frame(&mut tcx, 360, 400);
    assert_eq!(widths(&s), [[164.0, 164.0]; 2]);
    s.build_frame(&mut tcx, 600, 400);
    assert_eq!(widths(&s), [[180.0, 180.0]; 2]);
    assert_eq!(std::ptr::from_ref(&*s.editors[&key("draft")]), editor);
    assert_eq!(s.focus, Some(key("draft")));
    Ok(())
}

#[test]
fn compact_checkbox_uses_visible_invoice_name() -> Result<(), String> {
    const SELECTED: blockui::State<bool> = blockui::State::<bool>::new("invoice.selected", false);
    const CHANGE: blockui::BoolAction =
        blockui::BoolAction::new("invoice.change", |cx, value| SELECTED.set(cx, value));
    struct SelectionFixture;
    impl blockui::Actor for SelectionFixture {
        const STATE: &'static [blockui::StateSpec] = &[SELECTED.spec()];
        const ACTIONS: &'static [blockui::Action] = &[CHANGE.spec()];
        fn view(_: &blockui::Cx<'_>) -> blockui::Result<blockui::View> {
            Table::new("Invoices", vec![Text::new("Invoice").into_view()]).rows(vec![
                TableRow::new(
                    "one",
                    vec![
                        blockui::Row {
                            gap: 0.0,
                            ..Default::default()
                        }
                        .children(vec![
                            blockui::Checkbox::new("", SELECTED, CHANGE)
                                .key("select")
                                .into_view()
                                .labelled_by("invoice")
                                .style(blockui::ViewStyle {
                                    width: 28.0,
                                    ..Default::default()
                                }),
                            Text::new("INV001 🦀").key("invoice").into_view(),
                        ]),
                    ],
                ),
            ])
        }
    }
    let mut source = NativeSource::without_watch(Actor::for_test::<SelectionFixture>()?)?;
    source.set_semantics_enabled(true);
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 360, 260);
    let snapshot = source.semantic_snapshot().unwrap();
    let node = snapshot
        .nodes
        .iter()
        .find(|node| node.id == key("select"))
        .unwrap();
    assert_eq!(node.role, shell::SemanticRole::CheckBox);
    assert_eq!(node.label.as_deref(), Some("INV001 🦀"));
    source.set_focus(None);
    source.focus_next(false);
    assert_eq!(source.focus, Some(key("select")));
    source.actor.dispatch_widget(key("select"))?;
    source.build_frame(&mut tcx, 360, 260);
    assert_eq!(source.actor.checkbox_value(key("select")), Some(true));
    assert_eq!(source.focus, Some(key("select")));
    Ok(())
}

#[test]
fn pagination_native_keyboard_identity_and_narrow_bounds() -> Result<(), String> {
    const PAGE: blockui::State<i64> = blockui::State::<i64>::new("pager.page", 0);
    struct Pager;
    impl blockui::Actor for Pager {
        const STATE: &'static [blockui::StateSpec] = &[PAGE.spec()];
        const ACTIONS: &'static [blockui::Action] = &[];
        fn view(cx: &blockui::Cx<'_>) -> blockui::Result<blockui::View> {
            blockui::Pagination::new(PAGE, 3).key("pager").view(cx)
        }
    }
    let mut source = NativeSource::without_watch(Actor::for_test::<Pager>()?)?;
    let mut tcx = TextContext::new();
    source.set_semantics_enabled(true);
    source.build_frame(&mut tcx, 240, 180);
    let next = blockui::Key::named("pager").scoped(key("next")).0;
    let previous = blockui::Key::named("pager").scoped(key("previous")).0;
    let id = source.ids[&next];
    for item in [next, previous] {
        let rect = source.widget_layout_rect(item).unwrap();
        assert!(rect[0] >= 0.0 && rect[2] <= 240.0 && rect[3] <= 180.0);
        assert!(rect[2] > rect[0] && rect[3] > rect[1]);
    }
    source.set_focus(Some(next));
    source.handle_event(UiEvent::Key(shell::KeyEvent {
        phase: shell::KeyPhase::Down,
        key: "Enter".into(),
        key_without_modifiers: "Enter".into(),
        code: "Enter".into(),
        text: None,
        text_with_all_modifiers: None,
        location: Default::default(),
        modifiers: Default::default(),
        repeat: false,
        synthetic: false,
    }));
    source.build_frame(&mut tcx, 240, 180);
    assert_eq!(source.actor.integer("pager.page"), 1);
    assert_eq!(source.focus, Some(next));
    assert_eq!(source.ids[&next], id);
    source.actor.dispatch_widget(next)?;
    source.build_frame(&mut tcx, 240, 180);
    assert_eq!(source.actor.view().bindings[&next].node.disabled, 1);
    assert_eq!(source.ids[&next], id);
    assert_eq!(
        source.focus,
        Some(previous),
        "disabled Next recovers locally"
    );
    Ok(())
}

#[test]
fn numbered_pagination_native_jump_keeps_focus_and_bounded_layout() -> Result<(), String> {
    const PAGE: blockui::State<i64> = blockui::State::<i64>::new("numbered.page", 0);
    struct Pager;
    impl blockui::Actor for Pager {
        const STATE: &'static [blockui::StateSpec] = &[PAGE.spec()];
        const ACTIONS: &'static [blockui::Action] = &[];
        fn view(cx: &blockui::Cx<'_>) -> blockui::Result<blockui::View> {
            blockui::Pagination::new(PAGE, 20)
                .key("numbered")
                .numbered(true)
                .view(cx)
        }
    }
    let mut source = NativeSource::without_watch(Actor::for_test::<Pager>()?)?;
    let mut tcx = TextContext::new();
    source.set_semantics_enabled(true);
    source.build_frame(&mut tcx, 240, 180);
    let numbers = blockui::Key::named("numbered").scoped(key("numbers"));
    let last = numbers.scoped(19).0;
    let id = source.ids[&last];
    source.set_focus(Some(last));
    source.handle_event(UiEvent::Key(shell::KeyEvent {
        phase: shell::KeyPhase::Down,
        key: "Enter".into(),
        key_without_modifiers: "Enter".into(),
        code: "Enter".into(),
        text: None,
        text_with_all_modifiers: None,
        location: Default::default(),
        modifiers: Default::default(),
        repeat: false,
        synthetic: false,
    }));
    source.build_frame(&mut tcx, 240, 180);
    assert_eq!(source.actor.integer("numbered.page"), 19);
    assert_eq!(source.focus, Some(last));
    assert_eq!(source.ids[&last], id);
    assert_eq!(source.actor.view().bindings[&last].node.button_variant, 0);
    let snapshot = source.semantic_snapshot().unwrap();
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .filter(|node| node.states.current.is_some())
            .count(),
        1
    );
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|node| node.id == last)
            .unwrap()
            .states
            .current,
        Some(shell::SemanticCurrent::Page)
    );
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|node| node.id == numbers.scoped(0).0)
            .unwrap()
            .states
            .current,
        None
    );
    assert!(source.actor.dispatch_widget(numbers.scoped(1).0).is_err());
    source.actor.dispatch_widget(numbers.scoped(18).0)?;
    source.build_frame(&mut tcx, 240, 180);
    let next = blockui::Key::named("numbered").scoped(key("next")).0;
    source.set_focus(Some(next));
    source.actor.dispatch_widget(next)?;
    source.build_frame(&mut tcx, 240, 180);
    assert_eq!(
        source.focus,
        Some(last),
        "disabled numbered Next returns to the current page"
    );
    source.actor.dispatch_widget(numbers.scoped(0).0)?;
    source.build_frame(&mut tcx, 240, 180);
    let snapshot = source.semantic_snapshot().unwrap();
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|node| node.id == last)
            .unwrap()
            .states
            .current,
        None
    );
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|node| node.id == numbers.scoped(0).0)
            .unwrap()
            .states
            .current,
        Some(shell::SemanticCurrent::Page)
    );

    for binding in source
        .actor
        .view()
        .bindings
        .values()
        .filter(|binding| binding.node.parent == numbers.0)
    {
        let rect = source.widget_layout_rect(binding.node.key).unwrap();
        assert!(rect[0] >= 0.0 && rect[2] <= 240.0 && rect[3] <= 180.0);
    }
    Ok(())
}
