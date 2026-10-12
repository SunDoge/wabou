use super::*;
use blockui::{Cx, DropdownMenu, IntoView, MenuItem, State, StateSpec};
const OPEN: State<bool> = State::<bool>::new("menu.open", false);
const LAST: State<String> = State::<String>::new("menu.last");
const REVERSE: State<bool> = State::<bool>::new("menu.reverse", false);
const REORDER: blockui::Action =
    blockui::Action::new("menu.reorder", |cx, _| REVERSE.set(cx, !REVERSE.get(cx)?));
struct Fixture;
impl blockui::Actor for Fixture {
    const STATE: &'static [StateSpec] = &[OPEN.spec(), LAST.spec(), REVERSE.spec()];
    const ACTIONS: &'static [blockui::Action] = &[REORDER];
    fn view(cx: &Cx<'_>) -> blockui::Result<blockui::View> {
        let mut disabled = MenuItem::new("disabled", "Unavailable", |_| Ok(()));
        disabled.enabled = false;
        let mut items = vec![
            MenuItem::new("alpha", "Alpha", |cx| LAST.set(cx, "Alpha selected")),
            disabled,
            MenuItem::new("apple", "Apple", |cx| LAST.set(cx, "Apple selected")),
            MenuItem::new("save", "保存 🦀", |cx| LAST.set(cx, "保存しました 🦀")),
        ];
        if REVERSE.get(cx)? {
            items.reverse();
        }
        let second = items.split_off(2);
        let entries = vec![
            blockui::MenuEntry::group("first", "First commands", items),
            blockui::MenuEntry::separator("divider"),
            blockui::MenuEntry::group("second", "More commands", second),
        ];
        Ok(blockui::Column::default().children(vec![
            blockui::Button::label("Before")
                .key("before")
                .on_click(|_| Ok(())),
            DropdownMenu::new(OPEN, "Actions")
                .key("commands")
                .entries(cx, entries)?,
            blockui::Button::label("After")
                .key("after")
                .on_click(|_| Ok(())),
            blockui::Text::new(LAST).key("last").into_view(),
        ]))
    }
}
fn part(name: &str) -> u64 {
    blockui::Key::named("commands").scoped(key(name)).0
}
fn item(name: &str) -> u64 {
    blockui::Key::named("commands")
        .scoped(key("item"))
        .scoped(key(name))
        .0
}
fn press(name: &str) -> UiEvent {
    UiEvent::Key(shell::KeyEvent {
        phase: shell::KeyPhase::Down,
        key: name.into(),
        key_without_modifiers: name.into(),
        code: name.into(),
        text: None,
        text_with_all_modifiers: None,
        location: Default::default(),
        modifiers: Default::default(),
        repeat: false,
        synthetic: false,
    })
}
fn frame(source: &mut NativeSource, tcx: &mut TextContext) {
    source.build_frame(tcx, 360, 400);
}
#[test]
fn native_menu_keyboard_semantics_and_commands() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<Fixture>()?)?;
    source.set_semantics_enabled(true);
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    frame(&mut source, &mut tcx);
    source.set_focus(Some(part("trigger")));
    source.handle_event(press("ArrowDown"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(item("alpha")));
    assert_eq!(
        source
            .tree
            .get_node_context(source.ids[&item("alpha")])
            .unwrap()
            .outline_width,
        2.0
    );
    assert_eq!(
        source
            .tree
            .get_node_context(source.ids[&part("trigger")])
            .unwrap()
            .outline_width,
        0.0
    );
    let anchor = source.widget_layout_rect(part("trigger")).unwrap();
    let panel = source.widget_layout_rect(part("panel")).unwrap();
    assert!(
        (panel[1] - anchor[3] - 8.0).abs() < 1.0,
        "{anchor:?} {panel:?}"
    );
    assert!(panel[0] >= 16.0 && panel[2] <= 344.0 && panel[3] <= 384.0);
    let command = source.widget_layout_rect(item("alpha")).unwrap();
    assert_eq!(
        source.hit(shell::Point {
            x: f64::from((command[0] + command[2]) * 0.5),
            y: f64::from((command[1] + command[3]) * 0.5)
        }),
        Some(item("alpha"))
    );

    source.handle_event(press("ArrowDown"));
    assert_eq!(source.focus, Some(item("apple")));
    source.handle_event(press("End"));
    assert_eq!(source.focus, Some(item("save")));
    source.handle_event(press("ArrowDown"));
    assert_eq!(source.focus, Some(item("alpha")));
    source.handle_event(press("保"));
    assert_eq!(source.focus, Some(item("save")));
    frame(&mut source, &mut tcx);
    let semantics = source.semantic_snapshot().unwrap();
    assert!(
        semantics
            .nodes
            .iter()
            .any(|node| node.role == shell::SemanticRole::Group
                && node.label.as_deref() == Some("First commands"))
    );
    assert!(
        semantics
            .nodes
            .iter()
            .any(|node| node.role == shell::SemanticRole::Group
                && node.label.as_deref() == Some("More commands"))
    );
    assert!(semantics.nodes.iter().any(|node| node.id == part("panel")
        && node.role == shell::SemanticRole::Menu
        && node.label.as_deref() == Some("Actions")));
    assert!(
        semantics.nodes.iter().any(|node| node.id == part("trigger")
            && node.states.popup == Some(shell::SemanticPopup::Menu))
    );
    assert_eq!(
        semantics
            .nodes
            .iter()
            .filter(|node| node.role == shell::SemanticRole::MenuItem)
            .count(),
        4
    );
    source.handle_event(press("Enter"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.actor.text("menu.last"), "保存しました 🦀");
    assert_eq!(source.focus, Some(part("trigger")));
    assert!(!source.ids.contains_key(&item("save")));
    source.handle_event(press("ArrowUp"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(item("save")));
    source.handle_event(press("Escape"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(part("trigger")));
    Ok(())
}
#[test]
fn native_menu_reordering_retains_focus_and_tab_exits() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<Fixture>()?)?;
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    frame(&mut source, &mut tcx);
    source.set_focus(Some(part("trigger")));
    source.handle_event(press("ArrowDown"));
    frame(&mut source, &mut tcx);
    source.handle_event(press("ArrowDown"));
    frame(&mut source, &mut tcx);
    let id = source.ids[&item("apple")];
    source.actor.dispatch(Message {
        action: REORDER.id,
        value: 0,
    })?;
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(item("apple")));
    assert_eq!(source.ids[&item("apple")], id);
    source.actor.replace_for_test::<Fixture>()?;
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(item("apple")));
    assert_eq!(source.ids[&item("apple")], id);
    source.handle_event(press("Tab"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(key("after")));
    assert_eq!(
        source.actor.view().bindings[&part("panel")]
            .node
            .semantics
            .hidden,
        1
    );
    Ok(())
}

#[test]
fn native_menu_shift_tab_outside_and_window_blur_close() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<Fixture>()?)?;
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    frame(&mut source, &mut tcx);
    source.set_focus(Some(part("trigger")));
    source.handle_event(press("Enter"));
    frame(&mut source, &mut tcx);
    let UiEvent::Key(mut event) = press("Tab") else {
        unreachable!()
    };
    event.modifiers = shell::Modifiers::SHIFT;
    source.handle_event(UiEvent::Key(event));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(key("before")));
    source.set_focus(Some(part("trigger")));
    source.handle_event(press("Enter"));
    frame(&mut source, &mut tcx);
    source.handle_event(UiEvent::Focus(false));
    frame(&mut source, &mut tcx);
    assert_eq!(
        source.actor.view().bindings[&part("panel")]
            .node
            .semantics
            .hidden,
        1
    );
    source.handle_event(UiEvent::Focus(true));
    source.set_focus(Some(part("trigger")));
    source.handle_event(press("Enter"));
    frame(&mut source, &mut tcx);
    let outside = source.widget_layout_rect(key("before")).unwrap();
    source.handle_event(UiEvent::Pointer(shell::PointerEvent {
        phase: shell::PointerPhase::Down,
        position: shell::Point {
            x: f64::from(outside[0] + 4.0),
            y: f64::from(outside[1] + 4.0),
        },
        button: Some(shell::PointerButton::Primary),
        buttons: 1,
        modifiers: Default::default(),
        properties: Default::default(),
    }));
    frame(&mut source, &mut tcx);
    assert_eq!(
        source.actor.view().bindings[&part("panel")]
            .node
            .semantics
            .hidden,
        1
    );
    Ok(())
}

#[test]
#[ignore = "explicit native menu offscreen capture"]
fn native_menu_visuals() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<Fixture>()?)?;
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    frame(&mut source, &mut tcx);
    source.set_focus(Some(part("trigger")));
    source.handle_event(press("ArrowDown"));
    let nodes = source.build_frame(&mut tcx, 360, 400);
    let mut scene = shell::Scene::new();
    shell::scene::build_scene_scaled(
        &mut scene,
        &nodes,
        &mut tcx,
        360,
        400,
        source.base_color(),
        1.0,
    );
    shell::renderer::render_to_image(&scene, 360, 400, source.base_color())
        .map_err(|error| error.to_string())?
        .save("../../reports/native-menu.png")
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn native_menu_narrow_viewport_reveals_keyboard_target() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<Fixture>()?)?;
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 240, 180);
    source.set_focus(Some(part("trigger")));
    source.handle_event(press("ArrowDown"));
    source.build_frame(&mut tcx, 240, 180);
    source.handle_event(press("End"));
    source.build_frame(&mut tcx, 240, 180);
    // A later clean frame must retain the revealed offset.
    source.build_frame(&mut tcx, 240, 180);
    let panel = source.widget_layout_rect(part("panel")).unwrap();
    let target = source.widget_layout_rect(item("save")).unwrap();
    assert!(
        panel[0] >= 16.0 && panel[2] <= 224.0 && panel[1] >= 16.0 && panel[3] <= 164.0,
        "{panel:?}"
    );
    assert!(
        target[1] >= panel[1] && target[3] <= panel[3],
        "{panel:?} {target:?}"
    );
    assert_eq!(
        source.hit(shell::Point {
            x: f64::from((target[0] + target[2]) * 0.5),
            y: f64::from((target[1] + target[3]) * 0.5)
        }),
        Some(item("save"))
    );
    Ok(())
}

#[test]
#[ignore = "explicit real Gallery dylib menu replacement and Vello capture"]
fn gallery_menu_dylib_focus_and_visuals() -> Result<(), String> {
    let library = std::path::PathBuf::from(
        std::env::var_os("BLOCKUI_MENU_LIBRARY")
            .ok_or("set BLOCKUI_MENU_LIBRARY to the Gallery dylib")?,
    );
    let directory = std::env::temp_dir().join(format!("blockui-menu-dylib-{}", std::process::id()));
    let mut actor = Actor::new(953, &directory)?;
    actor.reload(&library)?;
    actor.dispatch_widget(key("nav.14"))?;
    let menu = blockui::Key::named("menu.actions");
    let trigger = menu.scoped(key("trigger")).0;
    let command = |name| menu.scoped(key("item")).scoped(key(name)).0;
    let mut source = NativeSource::without_watch(actor)?;
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 1200, 1000);
    source.set_focus(Some(trigger));
    source.handle_event(press("ArrowDown"));
    source.build_frame(&mut tcx, 1200, 1000);
    source.handle_event(press("ArrowDown"));
    source.build_frame(&mut tcx, 1200, 1000);
    assert_eq!(source.focus, Some(command("duplicate")));
    let id = source.ids[&command("duplicate")];
    for _ in 0..8 {
        source.actor.reload(&library)?;
        source.build_frame(&mut tcx, 1200, 1000);
        assert_eq!(source.focus, Some(command("duplicate")));
        assert_eq!(source.ids[&command("duplicate")], id);
        assert_eq!(source.actor.artifact_count(), 1);
        #[cfg(any(windows, target_os = "linux"))]
        assert_eq!(source.actor.mapped_implementation_count()?, 1);
    }
    let nodes = source.build_frame(&mut tcx, 1200, 1000);
    let mut scene = shell::Scene::new();
    shell::scene::build_scene_scaled(
        &mut scene,
        &nodes,
        &mut tcx,
        1200,
        1000,
        source.base_color(),
        1.0,
    );
    shell::renderer::render_to_image(&scene, 1200, 1000, source.base_color())
        .map_err(|error| error.to_string())?
        .save("../../reports/gallery-menu-native.png")
        .map_err(|error| error.to_string())?;
    source.handle_event(press("Enter"));
    source.build_frame(&mut tcx, 1200, 1000);
    assert_eq!(
        source.actor.text("gallery.menu_result"),
        "Project duplicated."
    );
    assert_eq!(source.focus, Some(trigger));
    assert!(!source.actor.boolean("gallery.menu_open"));
    let more = menu.scoped(key("submenu")).scoped(key("menu.more"));
    let more_trigger = more.scoped(key("trigger")).0;
    let more_item = more.scoped(key("item")).scoped(key("count")).0;
    source.handle_event(press("ArrowDown"));
    source.build_frame(&mut tcx, 1200, 1000);
    source.motion_manual_time = Some(Duration::ZERO);
    let anchor = source.widget_layout_rect(more_trigger).unwrap();
    menu_pointer(
        &mut source,
        f64::from(anchor[0] + 12.0),
        f64::from((anchor[1] + anchor[3]) * 0.5),
    );
    assert_eq!(source.menu_hover.deadline(), Some(100));
    source.actor.reload(&library)?;
    source.motion_manual_time = Some(Duration::from_millis(99));
    source.build_frame(&mut tcx, 1200, 1000);
    assert!(!source.actor.boolean("gallery.menu_more"));
    source.motion_manual_time = Some(Duration::from_millis(100));
    source.build_frame(&mut tcx, 1200, 1000);
    assert_eq!(source.focus, Some(more_item));
    let child_id = source.ids[&more_item];
    for _ in 0..8 {
        source.actor.reload(&library)?;
        source.build_frame(&mut tcx, 1200, 1000);
        assert_eq!(source.focus, Some(more_item));
        assert_eq!(source.ids[&more_item], child_id);
        assert_eq!(source.actor.artifact_count(), 1);
        #[cfg(any(windows, target_os = "linux"))]
        assert_eq!(source.actor.mapped_implementation_count()?, 1);
    }
    let nodes = source.build_frame(&mut tcx, 1200, 1000);
    let mut child_scene = shell::Scene::new();
    shell::scene::build_scene_scaled(
        &mut child_scene,
        &nodes,
        &mut tcx,
        1200,
        1000,
        source.base_color(),
        1.0,
    );
    shell::renderer::render_to_image(&child_scene, 1200, 1000, source.base_color())
        .map_err(|error| error.to_string())?
        .save("../../reports/gallery-submenu-native.png")
        .map_err(|error| error.to_string())?;
    source.handle_event(press("Enter"));
    source.build_frame(&mut tcx, 1200, 1000);
    assert_eq!(
        source.actor.text("gallery.menu_result"),
        "3 projects in this workspace."
    );
    assert_eq!(source.focus, Some(trigger));
    assert!(
        !source.actor.boolean("gallery.menu_open") && !source.actor.boolean("gallery.menu_more")
    );
    source.actor.dispatch_widget(trigger)?;
    source
        .actor
        .dispatch_widget(more.scoped(key("trigger")).0)?;
    let export = more.scoped(key("submenu")).scoped(key("menu.export"));
    let export_trigger = export.scoped(key("trigger")).0;
    source.build_frame(&mut tcx, 1200, 1000);
    source.set_focus(Some(export_trigger));
    source.handle_event(press("ArrowRight"));
    source.build_frame(&mut tcx, 1200, 1000);
    let deep_item = export.scoped(key("item")).scoped(key("csv")).0;
    assert_eq!(source.focus, Some(deep_item));
    let deep_id = source.ids[&deep_item];
    for _ in 0..8 {
        source.actor.reload(&library)?;
        source.build_frame(&mut tcx, 1200, 1000);
        assert_eq!(source.focus, Some(deep_item));
        assert_eq!(source.ids[&deep_item], deep_id);
        assert_eq!(source.actor.artifact_count(), 1);
        #[cfg(any(windows, target_os = "linux"))]
        assert_eq!(source.actor.mapped_implementation_count()?, 1);
    }
    source.handle_event(press("Enter"));
    source.build_frame(&mut tcx, 1200, 1000);
    assert_eq!(
        source.actor.text("gallery.menu_result"),
        "CSV: projects,active\n3,2"
    );
    assert_eq!(source.focus, Some(trigger));
    for name in [
        "gallery.menu_open",
        "gallery.menu_more",
        "gallery.menu_export",
    ] {
        assert!(!source.actor.boolean(name));
    }
    let file = blockui::Key::named("workspace.bar")
        .scoped(key("menu"))
        .scoped(key("file"));
    let file_trigger = file.scoped(key("trigger")).0;
    let first = file.scoped(key("item")).scoped(key("new")).0;
    source.set_focus(Some(file_trigger));
    source.handle_event(press("ArrowDown"));
    source.build_frame(&mut tcx, 1200, 1000);
    assert_eq!(source.focus, Some(first));
    let first_id = source.ids[&first];
    for _ in 0..8 {
        source.actor.reload(&library)?;
        source.build_frame(&mut tcx, 1200, 1000);
        assert!(source.actor.boolean("gallery.bar_file"));
        assert_eq!(source.focus, Some(first));
        assert_eq!(source.ids[&first], first_id);
        assert_eq!(source.actor.artifact_count(), 1);
        #[cfg(any(windows, target_os = "linux"))]
        assert_eq!(source.actor.mapped_implementation_count()?, 1);
    }
    source.handle_event(press("ArrowRight"));
    source.build_frame(&mut tcx, 1200, 1000);
    assert!(!source.actor.boolean("gallery.bar_file") && source.actor.boolean("gallery.bar_edit"));
    source.handle_event(press("Enter"));
    source.build_frame(&mut tcx, 1200, 1000);
    assert_eq!(
        source.actor.text("gallery.menu_result"),
        "Project link copied in this example."
    );
    let edit_trigger = blockui::Key::named("workspace.bar")
        .scoped(key("menu"))
        .scoped(key("edit"))
        .scoped(key("trigger"))
        .0;
    assert_eq!(source.focus, Some(edit_trigger));
    assert!(!source.actor.boolean("gallery.bar_edit"));
    let export = file.scoped(key("submenu")).scoped(key("export"));
    let export_trigger = export.scoped(key("trigger")).0;
    let summary = export.scoped(key("item")).scoped(key("summary")).0;
    source.set_focus(Some(file_trigger));
    source.handle_event(press("ArrowDown"));
    source.build_frame(&mut tcx, 1200, 1000);
    source.set_focus(Some(export_trigger));
    source.handle_event(press("ArrowRight"));
    source.build_frame(&mut tcx, 1200, 1000);
    assert_eq!(source.focus, Some(summary));
    let summary_id = source.ids[&summary];
    for _ in 0..8 {
        source.actor.reload(&library)?;
        source.build_frame(&mut tcx, 1200, 1000);
        assert!(
            source.actor.boolean("gallery.bar_file") && source.actor.boolean("gallery.bar_export")
        );
        assert_eq!(source.focus, Some(summary));
        assert_eq!(source.ids[&summary], summary_id);
        assert_eq!(source.actor.artifact_count(), 1);
        #[cfg(any(windows, target_os = "linux"))]
        assert_eq!(source.actor.mapped_implementation_count()?, 1);
    }
    source.handle_event(press("Enter"));
    source.build_frame(&mut tcx, 1200, 1000);
    assert_eq!(
        source.actor.text("gallery.menu_result"),
        "Workspace: 3 projects, 2 active."
    );
    assert_eq!(source.focus, Some(file_trigger));
    assert!(
        !source.actor.boolean("gallery.bar_file") && !source.actor.boolean("gallery.bar_export")
    );
    let context = blockui::Key::named("project.context");
    let context_trigger = context.scoped(key("trigger")).0;
    let context_panel = context.scoped(key("panel")).0;
    let inspect = context.scoped(key("item")).scoped(key("inspect")).0;
    source.set_focus(Some(context_trigger));
    source.pending_reveal = Some(context_trigger);
    source.build_frame(&mut tcx, 1200, 1000);
    let rect = source.widget_layout_rect(context_trigger).unwrap();
    let point = [
        f64::from((rect[0] + rect[2]) / 2.0),
        f64::from((rect[1] + rect[3]) / 2.0),
    ];
    source.handle_event(UiEvent::Pointer(shell::PointerEvent {
        phase: shell::PointerPhase::Down,
        position: shell::Point {
            x: point[0],
            y: point[1],
        },
        button: Some(shell::PointerButton::Secondary),
        buttons: 2,
        modifiers: Default::default(),
        properties: Default::default(),
    }));
    source.build_frame(&mut tcx, 1200, 1000);
    assert_eq!(source.focus, Some(inspect));
    let inspect_id = source.ids[&inspect];
    let origin = source.widget_layout_rect(context_panel).unwrap();
    for _ in 0..8 {
        source.actor.reload(&library)?;
        source.build_frame(&mut tcx, 1200, 1000);
        assert!(source.actor.boolean("gallery.context_open"));
        assert_eq!(source.context_anchor, Some((context_trigger, point)));
        assert_eq!(source.focus, Some(inspect));
        assert_eq!(source.ids[&inspect], inspect_id);
        assert_eq!(source.widget_layout_rect(context_panel).unwrap(), origin);
        assert_eq!(source.actor.artifact_count(), 1);
        #[cfg(any(windows, target_os = "linux"))]
        assert_eq!(source.actor.mapped_implementation_count()?, 1);
    }
    let nodes = source.build_frame(&mut tcx, 1200, 1000);
    let mut scene = shell::Scene::new();
    shell::scene::build_scene_scaled(
        &mut scene,
        &nodes,
        &mut tcx,
        1200,
        1000,
        source.base_color(),
        1.0,
    );
    shell::renderer::render_to_image(&scene, 1200, 1000, source.base_color())
        .map_err(|error| error.to_string())?
        .save("../../reports/gallery-context-native.png")
        .map_err(|error| error.to_string())?;
    source.handle_event(press("Enter"));
    source.build_frame(&mut tcx, 1200, 1000);
    assert_eq!(
        source.actor.text("gallery.menu_result"),
        "Atlas · Active · 4 members"
    );
    assert_eq!(source.focus, Some(context_trigger));
    assert!(!source.actor.boolean("gallery.context_open"));
    assert!(source.context_anchor.is_none());
    for (name, state, details, result) in [
        (
            "project.card.context",
            "gallery.card_context_open",
            "context.card.details",
            "Atlas · Active · 4 members",
        ),
        (
            "project.owner.context",
            "gallery.owner_context_open",
            "context.owner.details",
            "Morgan maintains Atlas.",
        ),
    ] {
        let surface = blockui::Key::named(name);
        let trigger = surface.scoped(key("trigger")).0;
        let panel = surface.scoped(key("panel")).0;
        let inspect = surface.scoped(key("item")).scoped(key("inspect")).0;
        let details = key(details);
        source.set_focus(Some(trigger));
        source.pending_reveal = Some(details);
        source.build_frame(&mut tcx, 1200, 1000);
        let rect = source.widget_layout_rect(details).unwrap();
        let point = [
            f64::from((rect[0] + rect[2]) / 2.0),
            f64::from((rect[1] + rect[3]) / 2.0),
        ];
        assert!((0.0..1200.0).contains(&point[0]) && (0.0..1000.0).contains(&point[1]));
        assert_eq!(source.context_trigger_for(details), Some(trigger));
        source.handle_event(UiEvent::Pointer(shell::PointerEvent {
            phase: shell::PointerPhase::Down,
            position: shell::Point {
                x: point[0],
                y: point[1],
            },
            button: Some(shell::PointerButton::Secondary),
            buttons: 2,
            modifiers: Default::default(),
            properties: Default::default(),
        }));
        source.build_frame(&mut tcx, 1200, 1000);
        assert_eq!(source.focus, Some(inspect));
        let inspect_id = source.ids[&inspect];
        let origin = source.widget_layout_rect(panel).unwrap();
        for _ in 0..8 {
            source.actor.reload(&library)?;
            source.build_frame(&mut tcx, 1200, 1000);
            assert!(source.actor.boolean(state));
            assert_eq!(source.context_anchor, Some((trigger, point)));
            assert_eq!(source.focus, Some(inspect));
            assert_eq!(source.ids[&inspect], inspect_id);
            assert_eq!(source.widget_layout_rect(panel).unwrap(), origin);
            assert_eq!(source.actor.artifact_count(), 1);
            #[cfg(any(windows, target_os = "linux"))]
            assert_eq!(source.actor.mapped_implementation_count()?, 1);
        }
        source.handle_event(press("Enter"));
        source.build_frame(&mut tcx, 1200, 1000);
        assert_eq!(source.actor.text("gallery.menu_result"), result);
        assert!(!source.actor.boolean(state));
        assert_eq!(source.focus, Some(trigger));
        assert!(source.context_anchor.is_none());
    }
    println!(
        "PASS native Gallery menu: 65 real replacements including outer/nested context surface pointer placement/focus and current commands, nested menubar, pending hover and focused third-level submenu retain item focus and Taffy identity, current action closes/restores focus; {} allocated bytes",
        source.actor.artifact_allocated_bytes()?
    );
    drop(source);
    assert_eq!(
        std::fs::read_dir(&directory)
            .map_err(|error| error.to_string())?
            .count(),
        0
    );
    std::fs::remove_dir(directory).map_err(|error| error.to_string())?;
    Ok(())
}

const CHECKED: State<bool> = State::<bool>::new("menu.checked", false);
struct Checkable;
impl blockui::Actor for Checkable {
    const STATE: &'static [StateSpec] = &[OPEN.spec(), CHECKED.spec()];
    const ACTIONS: &'static [blockui::Action] = &[];
    fn view(cx: &Cx<'_>) -> blockui::Result<blockui::View> {
        DropdownMenu::new(OPEN, "Settings").key("commands").items(
            cx,
            vec![
                MenuItem::checkbox("toggle", "Setting", CHECKED),
                MenuItem::new("done", "Done", |_| Ok(())),
            ],
        )
    }
}
#[test]
fn native_menu_checkbox_keeps_focus_and_publishes_checked_semantics() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<Checkable>()?)?;
    source.set_semantics_enabled(true);
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    frame(&mut source, &mut tcx);
    source.set_focus(Some(part("trigger")));
    source.handle_event(press("ArrowDown"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(item("toggle")));
    let id = source.ids[&item("toggle")];
    source.handle_event(press(" "));
    let UiEvent::Key(mut release) = press(" ") else {
        unreachable!()
    };
    release.phase = shell::KeyPhase::Up;
    source.handle_event(UiEvent::Key(release));
    frame(&mut source, &mut tcx);
    assert!(source.actor.boolean("menu.checked"));
    assert_eq!(source.focus, Some(item("toggle")));
    assert_eq!(source.ids[&item("toggle")], id);
    assert_eq!(
        source.tree.get_node_context(id).unwrap().text.as_deref(),
        Some("☑ Setting")
    );
    let semantics = source.semantic_snapshot().unwrap();
    let node = semantics
        .nodes
        .iter()
        .find(|node| node.id == item("toggle"))
        .unwrap();
    assert_eq!(node.role, shell::SemanticRole::MenuItemCheckBox);
    assert_eq!(node.label.as_deref(), Some("Setting"));
    assert_eq!(node.states.checked, Some(shell::SemanticToggleState::On));
    source.handle_event(press("Enter"));
    frame(&mut source, &mut tcx);
    assert!(!source.actor.boolean("menu.checked"));
    source.handle_event(press("ArrowDown"));
    assert_eq!(source.focus, Some(item("done")));
    Ok(())
}

const ORDER: State<i64> = State::<i64>::new("menu.order", 0);
struct RadioMenu;
impl blockui::Actor for RadioMenu {
    const STATE: &'static [StateSpec] = &[OPEN.spec(), ORDER.spec()];
    const ACTIONS: &'static [blockui::Action] = &[];
    fn view(cx: &Cx<'_>) -> blockui::Result<blockui::View> {
        let mut disabled = MenuItem::radio("disabled", "Unavailable", ORDER, 2);
        disabled.enabled = false;
        DropdownMenu::new(OPEN, "Sort").key("commands").items(
            cx,
            vec![
                MenuItem::radio("name", "Name", ORDER, 0),
                disabled,
                MenuItem::radio("updated", "Updated", ORDER, 1),
            ],
        )
    }
}
#[test]
fn native_menu_radio_navigation_focus_and_checked_semantics() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<RadioMenu>()?)?;
    source.set_semantics_enabled(true);
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    frame(&mut source, &mut tcx);
    source.set_focus(Some(part("trigger")));
    source.handle_event(press("ArrowDown"));
    frame(&mut source, &mut tcx);
    source.handle_event(press("ArrowDown"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(item("updated")));
    let id = source.ids[&item("updated")];
    source.handle_event(press("Enter"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.actor.integer("menu.order"), 1);
    assert_eq!(source.focus, Some(item("updated")));
    assert_eq!(source.ids[&item("updated")], id);
    assert_eq!(
        source.tree.get_node_context(id).unwrap().text.as_deref(),
        Some("◉ Updated")
    );
    let semantics = source.semantic_snapshot().unwrap();
    for (key, checked) in [
        (item("name"), shell::SemanticToggleState::Off),
        (item("updated"), shell::SemanticToggleState::On),
    ] {
        let node = semantics.nodes.iter().find(|node| node.id == key).unwrap();
        assert_eq!(node.role, shell::SemanticRole::MenuItemRadio);
        assert_eq!(node.states.checked, Some(checked));
    }
    source.handle_event(press("Enter"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.actor.integer("menu.order"), 1);
    source.handle_event(press("n"));
    assert_eq!(source.focus, Some(item("name")));
    source.handle_event(press("Enter"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.actor.integer("menu.order"), 0);
    Ok(())
}

const EXPORT: State<bool> = State::<bool>::new("menu.export", false);
struct Nested;
impl blockui::Actor for Nested {
    const STATE: &'static [StateSpec] = &[OPEN.spec(), EXPORT.spec(), LAST.spec()];
    const ACTIONS: &'static [blockui::Action] = &[];
    fn view(cx: &Cx<'_>) -> blockui::Result<blockui::View> {
        Ok(blockui::Column::default().children(vec![
            DropdownMenu::new(OPEN, "Actions").key("commands").entries(
                cx,
                vec![
                    blockui::MenuEntry::submenu(
                        DropdownMenu::new(EXPORT, "Export").key("export"),
                        vec![
                            MenuItem::new("csv", "CSV", |cx| LAST.set(cx, "CSV exported")).into(),
                            MenuItem::new("json", "JSON", |cx| LAST.set(cx, "JSON exported"))
                                .into(),
                        ],
                    ),
                    MenuItem::new("done", "Done", |_| Ok(())).into(),
                ],
            )?,
            blockui::Button::label("After")
                .key("after")
                .on_click(|_| Ok(())),
        ]))
    }
}
fn export_key(part: &str) -> u64 {
    blockui::Key::named("commands")
        .scoped(key("submenu"))
        .scoped(key("export"))
        .scoped(key(part))
        .0
}
fn export_item(name: &str) -> u64 {
    blockui::Key(export_key("item")).scoped(key(name)).0
}
#[test]
fn native_submenu_keyboard_returns_to_parent_and_command_closes_tree() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<Nested>()?)?;
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    frame(&mut source, &mut tcx);
    source.set_focus(Some(part("trigger")));
    source.handle_event(press("ArrowDown"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(export_key("trigger")));
    source.handle_event(press("ArrowRight"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(export_item("csv")));
    assert!(source.actor.boolean("menu.open") && source.actor.boolean("menu.export"));
    let parent = source.widget_layout_rect(export_key("trigger")).unwrap();
    let child = source.widget_layout_rect(export_key("panel")).unwrap();
    assert!(parent.iter().all(|coordinate| coordinate.is_finite()));
    assert!(
        child[0] >= 16.0 && child[2] <= 344.0 && child[1] >= 16.0 && child[3] <= 384.0,
        "{child:?}"
    );
    source.handle_event(press("ArrowLeft"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(export_key("trigger")));
    assert!(source.actor.boolean("menu.open") && !source.actor.boolean("menu.export"));
    source.handle_event(press("ArrowRight"));
    frame(&mut source, &mut tcx);
    source.handle_event(press("ArrowDown"));
    source.handle_event(press("Enter"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.actor.text("menu.last"), "JSON exported");
    assert_eq!(source.focus, Some(part("trigger")));
    assert!(!source.actor.boolean("menu.open") && !source.actor.boolean("menu.export"));
    source.handle_event(press("ArrowDown"));
    frame(&mut source, &mut tcx);
    source.handle_event(press("ArrowRight"));
    frame(&mut source, &mut tcx);
    source.handle_event(press("Tab"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(key("after")));
    assert!(!source.actor.boolean("menu.open") && !source.actor.boolean("menu.export"));
    source.set_focus(Some(part("trigger")));
    source.handle_event(press("ArrowDown"));
    frame(&mut source, &mut tcx);
    source.handle_event(press("ArrowRight"));
    frame(&mut source, &mut tcx);
    source.handle_event(UiEvent::Focus(false));
    frame(&mut source, &mut tcx);
    assert!(!source.actor.boolean("menu.open") && !source.actor.boolean("menu.export"));
    Ok(())
}

fn menu_pointer(source: &mut NativeSource, x: f64, y: f64) {
    source.handle_event(UiEvent::Pointer(shell::PointerEvent {
        phase: shell::PointerPhase::Move,
        position: shell::Point { x, y },
        button: None,
        buttons: 0,
        modifiers: Default::default(),
        properties: Default::default(),
    }));
}
#[test]
fn native_submenu_hover_delay_grace_and_keyboard_cancellation() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<Nested>()?)?;
    source.motion_manual_time = Some(Duration::ZERO);
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 900, 600);
    source.set_focus(Some(part("trigger")));
    source.handle_event(press("ArrowDown"));
    source.build_frame(&mut tcx, 900, 600);
    let anchor = source.widget_layout_rect(export_key("trigger")).unwrap();
    let origin = [
        f64::from(anchor[2] - 24.0),
        f64::from((anchor[1] + anchor[3]) * 0.5),
    ];
    menu_pointer(&mut source, origin[0], origin[1]);
    assert!(!source.actor.boolean("menu.export"));
    assert_eq!(source.menu_hover.deadline(), Some(100));
    source.motion_manual_time = Some(Duration::from_millis(99));
    source.build_frame(&mut tcx, 900, 600);
    assert!(!source.actor.boolean("menu.export"));
    source.motion_manual_time = Some(Duration::from_millis(100));
    source.build_frame(&mut tcx, 900, 600);
    assert!(source.actor.boolean("menu.export"));
    menu_pointer(&mut source, origin[0], origin[1]);
    let sibling = source.widget_layout_rect(item("done")).unwrap();
    source.motion_manual_time = Some(Duration::from_millis(110));
    menu_pointer(
        &mut source,
        f64::from(anchor[2] - 4.0),
        f64::from((sibling[1] + sibling[3]) * 0.5),
    );
    assert!(source.actor.boolean("menu.export"));
    assert_ne!(source.focus, Some(item("done")));
    source.motion_manual_time = Some(Duration::from_millis(400));
    source.build_frame(&mut tcx, 900, 600);
    assert!(!source.actor.boolean("menu.export"));
    assert_eq!(source.focus, Some(item("done")));
    source.motion_manual_time = Some(Duration::from_millis(500));
    menu_pointer(&mut source, origin[0], origin[1]);
    source.motion_manual_time = Some(Duration::from_millis(600));
    source.build_frame(&mut tcx, 900, 600);
    assert!(source.actor.boolean("menu.export"));
    let child = source.widget_layout_rect(export_item("json")).unwrap();
    menu_pointer(
        &mut source,
        f64::from(child[0] + 12.0),
        f64::from((child[1] + child[3]) * 0.5),
    );
    assert_eq!(source.focus, Some(export_item("json")));
    assert!(source.actor.boolean("menu.export"));
    source.handle_event(press("Escape"));
    source.build_frame(&mut tcx, 900, 600);
    source.motion_manual_time = Some(Duration::from_millis(1000));
    source.build_frame(&mut tcx, 900, 600);
    assert!(!source.actor.boolean("menu.export"));
    assert_eq!(source.menu_hover.deadline(), None);
    Ok(())
}

const FORMAT: State<bool> = State::<bool>::new("menu.format", false);
struct DeepNested;
impl blockui::Actor for DeepNested {
    const STATE: &'static [StateSpec] = &[OPEN.spec(), EXPORT.spec(), FORMAT.spec(), LAST.spec()];
    const ACTIONS: &'static [blockui::Action] = &[];
    fn view(cx: &Cx<'_>) -> blockui::Result<blockui::View> {
        DropdownMenu::new(OPEN, "Actions").key("commands").entries(
            cx,
            vec![blockui::MenuEntry::submenu(
                DropdownMenu::new(EXPORT, "Export").key("export"),
                vec![blockui::MenuEntry::submenu(
                    DropdownMenu::new(FORMAT, "Format").key("format"),
                    vec![
                        MenuItem::new("csv", "CSV", |cx| LAST.set(cx, "Deep CSV exported")).into(),
                    ],
                )],
            )],
        )
    }
}
fn format_key(part: &str) -> u64 {
    blockui::Key::named("commands")
        .scoped(key("submenu"))
        .scoped(key("export"))
        .scoped(key("submenu"))
        .scoped(key("format"))
        .scoped(key(part))
        .0
}
#[test]
fn native_three_level_menu_recovers_each_parent_and_root_after_replacement() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<DeepNested>()?)?;
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    frame(&mut source, &mut tcx);
    source.set_focus(Some(part("trigger")));
    source.handle_event(press("ArrowDown"));
    frame(&mut source, &mut tcx);
    source.handle_event(press("ArrowRight"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(format_key("trigger")));
    source.handle_event(press("ArrowRight"));
    frame(&mut source, &mut tcx);
    let csv = blockui::Key(format_key("item")).scoped(key("csv")).0;
    assert_eq!(source.focus, Some(csv));
    source.handle_event(press("Escape"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(format_key("trigger")));
    assert!(source.actor.boolean("menu.open") && source.actor.boolean("menu.export"));
    assert!(!source.actor.boolean("menu.format"));
    source.handle_event(press("ArrowLeft"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(export_key("trigger")));
    assert!(source.actor.boolean("menu.open"));
    assert!(!source.actor.boolean("menu.export"));
    for _ in 0..2 {
        source.handle_event(press("ArrowRight"));
        frame(&mut source, &mut tcx);
    }
    source.actor.replace_for_test::<DeepNested>()?;
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(csv));
    source.handle_event(press("Enter"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.actor.text("menu.last"), "Deep CSV exported");
    assert_eq!(source.focus, Some(part("trigger")));
    for name in ["menu.open", "menu.export", "menu.format"] {
        assert!(
            !source.actor.boolean(name),
            "{name} must close with the tree"
        );
    }
    Ok(())
}

const BAR_EDIT: State<bool> = State::<bool>::new("menu.bar_edit", false);
const BAR_DISABLED: State<bool> = State::<bool>::new("menu.bar_disabled", false);
struct NativeBar;
impl blockui::Actor for NativeBar {
    const STATE: &'static [StateSpec] = &[
        OPEN.spec(),
        BAR_EDIT.spec(),
        BAR_DISABLED.spec(),
        LAST.spec(),
    ];
    const ACTIONS: &'static [blockui::Action] = &[];
    fn view(cx: &Cx<'_>) -> blockui::Result<blockui::View> {
        let mut disabled = DropdownMenu::new(BAR_DISABLED, "Unavailable").key("disabled");
        disabled.enabled = false;
        blockui::Menubar::new("bar").menus(
            cx,
            vec![
                (
                    DropdownMenu::new(OPEN, "File").key("file"),
                    vec![MenuItem::new("new", "New", |cx| LAST.set(cx, "Created")).into()],
                ),
                (disabled, vec![]),
                (
                    DropdownMenu::new(BAR_EDIT, "Edit").key("edit"),
                    vec![MenuItem::new("copy", "Copy", |cx| LAST.set(cx, "Copied")).into()],
                ),
            ],
        )
    }
}
fn bar_part(menu: &str, part: &str) -> u64 {
    blockui::Key::named("bar")
        .scoped(key("menu"))
        .scoped(key(menu))
        .scoped(key(part))
        .0
}
#[test]
fn native_menubar_horizontal_switch_skips_disabled_and_preserves_open_session() -> Result<(), String>
{
    let mut source = NativeSource::without_watch(Actor::for_test::<NativeBar>()?)?;
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    frame(&mut source, &mut tcx);
    let file = bar_part("file", "trigger");
    let edit = bar_part("edit", "trigger");
    source.set_focus(Some(file));
    source.handle_event(press("ArrowRight"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(edit));
    assert!(!source.actor.boolean("menu.open") && !source.actor.boolean("menu.bar_edit"));
    source.handle_event(press("ArrowDown"));
    frame(&mut source, &mut tcx);
    let copy = blockui::Key(bar_part("edit", "item")).scoped(key("copy")).0;
    assert_eq!(source.focus, Some(copy));
    source.handle_event(press("ArrowRight"));
    frame(&mut source, &mut tcx);
    let new = blockui::Key(bar_part("file", "item")).scoped(key("new")).0;
    assert_eq!(source.focus, Some(new));
    assert!(source.actor.boolean("menu.open") && !source.actor.boolean("menu.bar_edit"));
    source.handle_event(press("ArrowLeft"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(copy));
    source.handle_event(press("Escape"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(edit));
    assert!(!source.actor.boolean("menu.bar_edit"));
    source.handle_event(press("Home"));
    assert_eq!(source.focus, Some(file));
    source.handle_event(press("End"));
    assert_eq!(source.focus, Some(edit));
    source.handle_event(press("ArrowUp"));
    frame(&mut source, &mut tcx);
    source.handle_event(press("Enter"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.actor.text("menu.last"), "Copied");
    assert_eq!(source.focus, Some(edit));
    Ok(())
}

#[test]
fn native_menubar_hover_switch_requires_open_session_and_keyboard_cancels() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<NativeBar>()?)?;
    source.set_semantics_enabled(true);
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 900, 600);
    let file = bar_part("file", "trigger");
    let edit = bar_part("edit", "trigger");
    let hover = |source: &mut NativeSource, key| {
        let rect = source.widget_layout_rect(key).unwrap();
        menu_pointer(
            source,
            f64::from((rect[0] + rect[2]) / 2.0),
            f64::from((rect[1] + rect[3]) / 2.0),
        );
    };
    hover(&mut source, edit);
    source.build_frame(&mut tcx, 900, 600);
    assert!(!source.actor.boolean("menu.bar_edit"));
    source.actor.dispatch_widget(file)?;
    source.build_frame(&mut tcx, 900, 600);
    hover(&mut source, edit);
    source.build_frame(&mut tcx, 900, 600);
    assert!(source.actor.boolean("menu.bar_edit") && !source.actor.boolean("menu.open"));
    let copy = blockui::Key(bar_part("edit", "item")).scoped(key("copy")).0;
    assert_eq!(source.focus, Some(copy));
    hover(&mut source, bar_part("disabled", "trigger"));
    source.build_frame(&mut tcx, 900, 600);
    assert!(source.actor.boolean("menu.bar_edit") && !source.actor.boolean("menu.bar_disabled"));
    source.handle_event(press("Escape"));
    source.build_frame(&mut tcx, 900, 600);
    assert_eq!(source.focus, Some(edit));
    assert!(!source.actor.boolean("menu.bar_edit"));
    source.build_frame(&mut tcx, 900, 600);
    assert!(!source.actor.boolean("menu.bar_edit"));
    let semantics = source.semantic_snapshot().unwrap();
    assert_eq!(
        semantics
            .nodes
            .iter()
            .find(|node| node.id == file)
            .unwrap()
            .role,
        shell::SemanticRole::MenuItem
    );
    Ok(())
}

struct TabBar;
impl blockui::Actor for TabBar {
    const STATE: &'static [StateSpec] = NativeBar::STATE;
    const ACTIONS: &'static [blockui::Action] = &[];
    fn view(cx: &Cx<'_>) -> blockui::Result<blockui::View> {
        Ok(blockui::Column::default().children(vec![
            blockui::Button::label("Before bar")
                .key("before.bar")
                .on_click(|_| Ok(())),
            NativeBar::view(cx)?,
            blockui::Button::label("After bar")
                .key("after.bar")
                .on_click(|_| Ok(())),
        ]))
    }
}
#[test]
fn native_menubar_has_one_remembered_tab_stop_and_menu_tab_exits_bar() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<TabBar>()?)?;
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    frame(&mut source, &mut tcx);
    let file = bar_part("file", "trigger");
    let edit = bar_part("edit", "trigger");
    source.set_focus(Some(key("before.bar")));
    source.handle_event(press("Tab"));
    assert_eq!(source.focus, Some(file));
    source.handle_event(press("ArrowRight"));
    assert_eq!(source.focus, Some(edit));
    source.handle_event(press("Tab"));
    assert_eq!(source.focus, Some(key("after.bar")));
    source.focus_next(true);
    assert_eq!(source.focus, Some(edit));
    source.actor.replace_for_test::<TabBar>()?;
    frame(&mut source, &mut tcx);
    source.focus_next(true);
    assert_eq!(source.focus, Some(key("before.bar")));
    source.handle_event(press("Tab"));
    assert_eq!(source.focus, Some(edit));
    source.handle_event(press("ArrowDown"));
    frame(&mut source, &mut tcx);
    source.handle_event(press("Tab"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(key("after.bar")));
    assert!(!source.actor.boolean("menu.bar_edit"));
    source.focus_next(true);
    assert_eq!(source.focus, Some(edit));
    Ok(())
}

struct DisabledTabBar;
struct RemovedTabBar;
struct UnmountedTabBar;
fn changed_tab_bar(cx: &Cx<'_>, disabled: bool, removed: bool) -> blockui::Result<blockui::View> {
    let mut edit = DropdownMenu::new(BAR_EDIT, "Edit").key("edit");
    edit.enabled = !disabled;
    let mut menus = vec![(
        DropdownMenu::new(OPEN, "File").key("file"),
        vec![MenuItem::new("new", "New", |_| Ok(())).into()],
    )];
    if !removed {
        menus.push((edit, vec![MenuItem::new("copy", "Copy", |_| Ok(())).into()]));
    }
    Ok(blockui::Column::default().children(vec![
        blockui::Button::label("Before bar")
            .key("before.bar")
            .on_click(|_| Ok(())),
        blockui::Menubar::new("bar").menus(cx, menus)?,
        blockui::Button::label("After bar")
            .key("after.bar")
            .on_click(|_| Ok(())),
    ]))
}
impl blockui::Actor for DisabledTabBar {
    const STATE: &'static [StateSpec] = NativeBar::STATE;
    const ACTIONS: &'static [blockui::Action] = &[];
    fn view(cx: &Cx<'_>) -> blockui::Result<blockui::View> {
        changed_tab_bar(cx, true, false)
    }
}
impl blockui::Actor for RemovedTabBar {
    const STATE: &'static [StateSpec] = NativeBar::STATE;
    const ACTIONS: &'static [blockui::Action] = &[];
    fn view(cx: &Cx<'_>) -> blockui::Result<blockui::View> {
        changed_tab_bar(cx, false, true)
    }
}
impl blockui::Actor for UnmountedTabBar {
    const STATE: &'static [StateSpec] = NativeBar::STATE;
    const ACTIONS: &'static [blockui::Action] = &[];
    fn view(_: &Cx<'_>) -> blockui::Result<blockui::View> {
        Ok(blockui::Column::default().children(vec![
            blockui::Button::label("After bar")
                .key("after.bar")
                .on_click(|_| Ok(())),
        ]))
    }
}
#[test]
fn native_menubar_disabled_removed_and_unmounted_roots_reconcile_tab_memory() -> Result<(), String>
{
    let mut source = NativeSource::without_watch(Actor::for_test::<TabBar>()?)?;
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    frame(&mut source, &mut tcx);
    let file = bar_part("file", "trigger");
    let edit = bar_part("edit", "trigger");
    for _ in 0..8 {
        source.set_focus(Some(edit));
        assert_eq!(source.menubar_stops.len(), 1);
        source.actor.replace_for_test::<DisabledTabBar>()?;
        frame(&mut source, &mut tcx);
        assert!(!source.menubar_stops.values().any(|key| *key == edit));
        source.set_focus(Some(key("before.bar")));
        source.handle_event(press("Tab"));
        assert_eq!(source.focus, Some(file));
        source.actor.replace_for_test::<TabBar>()?;
        frame(&mut source, &mut tcx);
        source.set_focus(Some(edit));
        source.actor.replace_for_test::<RemovedTabBar>()?;
        frame(&mut source, &mut tcx);
        source.set_focus(Some(key("before.bar")));
        source.handle_event(press("Tab"));
        assert_eq!(source.focus, Some(file));
        source.actor.replace_for_test::<UnmountedTabBar>()?;
        frame(&mut source, &mut tcx);
        assert!(source.menubar_stops.is_empty());
        source.actor.replace_for_test::<TabBar>()?;
        frame(&mut source, &mut tcx);
    }
    Ok(())
}

struct NestedBar;
impl blockui::Actor for NestedBar {
    const STATE: &'static [StateSpec] = &[OPEN.spec(), BAR_EDIT.spec(), EXPORT.spec(), LAST.spec()];
    const ACTIONS: &'static [blockui::Action] = &[];
    fn view(cx: &Cx<'_>) -> blockui::Result<blockui::View> {
        blockui::Menubar::new("bar").menus(
            cx,
            vec![
                (
                    DropdownMenu::new(OPEN, "File").key("file"),
                    vec![blockui::MenuEntry::submenu(
                        DropdownMenu::new(EXPORT, "Export").key("export"),
                        vec![
                            MenuItem::new("csv", "CSV", |cx| LAST.set(cx, "Nested bar CSV")).into(),
                        ],
                    )],
                ),
                (
                    DropdownMenu::new(BAR_EDIT, "Edit").key("edit"),
                    vec![MenuItem::new("copy", "Copy", |_| Ok(())).into()],
                ),
            ],
        )
    }
}
#[test]
fn native_menubar_submenu_arrows_take_precedence_and_command_restores_root() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<NestedBar>()?)?;
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    frame(&mut source, &mut tcx);
    let file = bar_part("file", "trigger");
    let child = blockui::Key::named("bar")
        .scoped(key("menu"))
        .scoped(key("file"))
        .scoped(key("submenu"))
        .scoped(key("export"));
    let export_trigger = child.scoped(key("trigger")).0;
    let csv = child.scoped(key("item")).scoped(key("csv")).0;
    source.set_focus(Some(file));
    source.handle_event(press("ArrowDown"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(export_trigger));
    source.handle_event(press("ArrowRight"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(csv));
    assert!(source.actor.boolean("menu.open") && source.actor.boolean("menu.export"));
    assert!(!source.actor.boolean("menu.bar_edit"));
    source.handle_event(press("ArrowLeft"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(export_trigger));
    source.handle_event(press("ArrowLeft"));
    frame(&mut source, &mut tcx);
    assert!(!source.actor.boolean("menu.open") && !source.actor.boolean("menu.export"));
    assert!(source.actor.boolean("menu.bar_edit"));
    source.handle_event(press("ArrowRight"));
    frame(&mut source, &mut tcx);
    source.handle_event(press("ArrowRight"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.focus, Some(csv));
    source.actor.replace_for_test::<NestedBar>()?;
    frame(&mut source, &mut tcx);
    source.handle_event(press("Enter"));
    frame(&mut source, &mut tcx);
    assert_eq!(source.actor.text("menu.last"), "Nested bar CSV");
    assert_eq!(source.focus, Some(file));
    assert!(!source.actor.boolean("menu.open") && !source.actor.boolean("menu.export"));
    Ok(())
}

struct NativeContext;
impl blockui::Actor for NativeContext {
    const STATE: &'static [StateSpec] = &[OPEN.spec(), LAST.spec()];
    const ACTIONS: &'static [blockui::Action] = &[];
    fn view(cx: &Cx<'_>) -> blockui::Result<blockui::View> {
        blockui::ContextMenu::new(OPEN, "Project actions")
            .key("context")
            .items(
                cx,
                vec![
                    MenuItem::new("copy", "Copy project", |cx| LAST.set(cx, "Context copied")),
                    MenuItem::new("archive", "Archive project", |_| Ok(())),
                ],
            )
    }
}
#[test]
fn native_context_secondary_pointer_reposition_keyboard_and_focus_recovery() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<NativeContext>()?)?;
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 900, 600);
    let root = blockui::Key::named("context");
    let trigger = root.scoped(key("trigger")).0;
    let panel = root.scoped(key("panel")).0;
    let copy = root.scoped(key("item")).scoped(key("copy")).0;
    let rect = source.widget_layout_rect(trigger).unwrap();
    let activate = |source: &mut NativeSource, point: [f64; 2]| {
        source.handle_event(UiEvent::Pointer(shell::PointerEvent {
            phase: shell::PointerPhase::Down,
            position: shell::Point {
                x: point[0],
                y: point[1],
            },
            button: Some(shell::PointerButton::Secondary),
            buttons: 2,
            modifiers: Default::default(),
            properties: Default::default(),
        }));
    };
    let point = [
        f64::from(rect[2] - 4.0),
        f64::from((rect[1] + rect[3]) / 2.0),
    ];
    activate(&mut source, point);
    source.build_frame(&mut tcx, 900, 600);
    assert!(source.actor.boolean("menu.open"));
    assert_eq!(source.focus, Some(copy));
    let panel_rect = source.widget_layout_rect(panel).unwrap();
    assert!(panel_rect[0] >= 16.0 && panel_rect[2] <= 884.0);
    assert!((f64::from(panel_rect[1]) - point[1]).abs() < 0.1);
    let point = [f64::from(rect[0] + 4.0), point[1]];
    activate(&mut source, point);
    source.build_frame(&mut tcx, 900, 600);
    assert!(source.actor.boolean("menu.open"));
    assert!((source.widget_layout_rect(panel).unwrap()[0] - 16.0).abs() < 0.1);
    source.handle_event(press("Escape"));
    source.build_frame(&mut tcx, 900, 600);
    assert_eq!(source.focus, Some(trigger));
    assert!(source.context_anchor.is_none());
    let mut event = press("F10");
    if let UiEvent::Key(key) = &mut event {
        key.modifiers = shell::Modifiers::SHIFT;
    }
    source.handle_event(event);
    source.build_frame(&mut tcx, 900, 600);
    assert_eq!(source.focus, Some(copy));
    assert!(source.context_anchor.is_none());
    source.actor.replace_for_test::<NativeContext>()?;
    source.build_frame(&mut tcx, 900, 600);
    source.handle_event(press("Enter"));
    source.build_frame(&mut tcx, 900, 600);
    assert_eq!(source.actor.text("menu.last"), "Context copied");
    assert_eq!(source.focus, Some(trigger));
    source.handle_event(press("ContextMenu"));
    source.build_frame(&mut tcx, 900, 600);
    assert!(source.actor.boolean("menu.open"));
    Ok(())
}

struct NativeContextSurface;
impl blockui::Actor for NativeContextSurface {
    const STATE: &'static [StateSpec] = NativeContext::STATE;
    const ACTIONS: &'static [blockui::Action] = &[];
    fn view(cx: &Cx<'_>) -> blockui::Result<blockui::View> {
        blockui::ContextMenu::new(OPEN, "Project card")
            .key("surface")
            .surface(
                cx,
                blockui::Column::default().children(vec![
                    blockui::Text::new("Atlas project")
                        .key("surface.title")
                        .into_view(),
                    blockui::Button::label("Open project")
                        .key("surface.open")
                        .on_click(|cx| LAST.set(cx, "Child opened")),
                ]),
                vec![
                    MenuItem::new("inspect", "Inspect", |cx| LAST.set(cx, "Surface inspected"))
                        .into(),
                ],
            )
    }
}
#[test]
fn native_context_surface_routes_descendants_and_keeps_primary_child_actions() -> Result<(), String>
{
    let mut source = NativeSource::without_watch(Actor::for_test::<NativeContextSurface>()?)?;
    source.set_semantics_enabled(true);
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 900, 600);
    let surface = blockui::Key::named("surface").scoped(key("trigger")).0;
    source.focus_next(false);
    assert_eq!(source.focus, Some(surface));
    let pointer = |source: &mut NativeSource, id, button, phase| {
        let rect = source.widget_layout_rect(id).unwrap();
        source.handle_event(UiEvent::Pointer(shell::PointerEvent {
            phase,
            position: shell::Point {
                x: f64::from((rect[0] + rect[2]) / 2.0),
                y: f64::from((rect[1] + rect[3]) / 2.0),
            },
            button: Some(button),
            buttons: 0,
            modifiers: Default::default(),
            properties: Default::default(),
        }));
    };
    pointer(
        &mut source,
        key("surface.title"),
        shell::PointerButton::Secondary,
        shell::PointerPhase::Down,
    );
    source.build_frame(&mut tcx, 900, 600);
    assert!(source.actor.boolean("menu.open"));
    let inspect = blockui::Key::named("surface")
        .scoped(key("item"))
        .scoped(key("inspect"))
        .0;
    assert_eq!(source.focus, Some(inspect));
    let panel = blockui::Key::named("surface").scoped(key("panel")).0;
    assert_eq!(
        source
            .semantic_snapshot()
            .unwrap()
            .nodes
            .iter()
            .find(|node| node.id == panel)
            .unwrap()
            .label
            .as_deref(),
        Some("Project card")
    );
    source.handle_event(press("Escape"));
    source.build_frame(&mut tcx, 900, 600);
    assert_eq!(source.focus, Some(surface));
    for phase in [shell::PointerPhase::Down, shell::PointerPhase::Up] {
        pointer(
            &mut source,
            key("surface.open"),
            shell::PointerButton::Primary,
            phase,
        );
    }
    source.build_frame(&mut tcx, 900, 600);
    assert_eq!(source.actor.text("menu.last"), "Child opened");
    assert!(!source.actor.boolean("menu.open"));
    source.set_focus(Some(key("surface.open")));
    source.handle_event(press("ContextMenu"));
    source.build_frame(&mut tcx, 900, 600);
    assert_eq!(source.focus, Some(inspect));
    source.handle_event(press("Enter"));
    source.build_frame(&mut tcx, 900, 600);
    assert_eq!(source.actor.text("menu.last"), "Surface inspected");
    assert_eq!(source.focus, Some(surface));
    Ok(())
}

struct NestedContextSurface;
struct DisabledNestedContextSurface;
fn nested_context_surface(cx: &Cx<'_>, enabled: bool) -> blockui::Result<blockui::View> {
    let mut inner = blockui::ContextMenu::new(EXPORT, "Owner").key("inner");
    inner.enabled = enabled;
    blockui::ContextMenu::new(OPEN, "Project")
        .key("outer")
        .surface(
            cx,
            inner.surface(
                cx,
                blockui::Text::new("Morgan").key("nested.owner").into_view(),
                vec![MenuItem::new("inspect", "Inspect owner", |_| Ok(())).into()],
            )?,
            vec![MenuItem::new("inspect", "Inspect project", |_| Ok(())).into()],
        )
}
impl blockui::Actor for NestedContextSurface {
    const STATE: &'static [StateSpec] = &[OPEN.spec(), EXPORT.spec()];
    const ACTIONS: &'static [blockui::Action] = &[];
    fn view(cx: &Cx<'_>) -> blockui::Result<blockui::View> {
        nested_context_surface(cx, true)
    }
}
impl blockui::Actor for DisabledNestedContextSurface {
    const STATE: &'static [StateSpec] = NestedContextSurface::STATE;
    const ACTIONS: &'static [blockui::Action] = &[];
    fn view(cx: &Cx<'_>) -> blockui::Result<blockui::View> {
        nested_context_surface(cx, false)
    }
}
#[test]
fn native_nested_context_surface_stops_at_disabled_targets_and_menu_planes() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<NestedContextSurface>()?)?;
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 900, 600);
    let inner = blockui::Key::named("inner");
    let trigger = inner.scoped(key("trigger")).0;
    assert_eq!(
        source.context_trigger_for(key("nested.owner")),
        Some(trigger)
    );
    source.set_focus(Some(trigger));
    source.handle_event(press("ContextMenu"));
    source.build_frame(&mut tcx, 900, 600);
    assert!(source.actor.boolean("menu.export") && !source.actor.boolean("menu.open"));
    let command = inner.scoped(key("item")).scoped(key("inspect")).0;
    assert_eq!(source.context_trigger_for(command), None);
    source.handle_event(press("ContextMenu"));
    source.build_frame(&mut tcx, 900, 600);
    assert!(!source.actor.boolean("menu.open"));
    source.actor.dismiss_popover(inner.scoped(key("panel")).0)?;
    source
        .actor
        .replace_for_test::<DisabledNestedContextSurface>()?;
    source.build_frame(&mut tcx, 900, 600);
    assert_eq!(source.context_trigger_for(key("nested.owner")), None);
    source.set_focus(Some(key("nested.owner")));
    source.handle_event(press("ContextMenu"));
    source.build_frame(&mut tcx, 900, 600);
    assert!(!source.actor.boolean("menu.export") && !source.actor.boolean("menu.open"));
    Ok(())
}

#[test]
fn destructive_menu_keeps_compact_geometry_and_red_host_paint() -> Result<(), String> {
    struct Destructive;
    impl blockui::Actor for Destructive {
        const STATE: &'static [StateSpec] = &[OPEN.spec(), LAST.spec()];
        const ACTIONS: &'static [blockui::Action] = &[];
        fn view(cx: &Cx<'_>) -> blockui::Result<blockui::View> {
            let mut disabled =
                MenuItem::new("disabled", "Disabled delete", |_| Ok(())).destructive(true);
            disabled.enabled = false;
            DropdownMenu::new(OPEN, "Actions").key("commands").items(
                cx,
                vec![
                    MenuItem::new("normal", "Keep project", |_| Ok(())),
                    MenuItem::new("delete", "Delete project", |cx| LAST.set(cx, "Deleted"))
                        .destructive(true),
                    disabled,
                ],
            )
        }
    }
    let mut source = NativeSource::without_watch(Actor::for_test::<Destructive>()?)?;
    let mut tcx = TextContext::new();
    source.actor.dispatch_widget(part("trigger"))?;
    source.build_frame(&mut tcx, 480, 400);
    let normal = source
        .tree
        .get_node_context(source.ids[&item("normal")])
        .unwrap();
    let danger = source
        .tree
        .get_node_context(source.ids[&item("delete")])
        .unwrap();
    assert_eq!(danger.text_color, Color::from_rgb8(166, 32, 40));
    assert_eq!(danger.background, normal.background);
    assert!(danger.border.is_none());
    assert_eq!(
        source.widget_layout_rect(item("normal")).unwrap()[3]
            - source.widget_layout_rect(item("normal")).unwrap()[1],
        source.widget_layout_rect(item("delete")).unwrap()[3]
            - source.widget_layout_rect(item("delete")).unwrap()[1]
    );
    let disabled = source
        .tree
        .get_node_context(source.ids[&item("disabled")])
        .unwrap();
    assert_eq!(disabled.text_color, Color::from_rgb8(109, 119, 130));
    let mut hover = danger.clone();
    source.apply_hover_style(item("delete"), false, &mut hover);
    assert_eq!(hover.background, Some(Color::from_rgb8(255, 236, 236)));
    assert_eq!(hover.text_color, danger.text_color);
    source.set_focus(Some(item("delete")));
    source.handle_event(press("Enter"));
    source.build_frame(&mut tcx, 480, 400);
    assert_eq!(source.actor.text("menu.last"), "Deleted");
    assert!(!source.actor.boolean("menu.open"));
    assert_eq!(source.focus, Some(part("trigger")));
    Ok(())
}
