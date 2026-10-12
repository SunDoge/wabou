use super::*;
use blockui::{
    Card, Cx, IntoView, State, StateSpec, Styled, Text, TextInput, TextTone, View, ViewStyle,
};
const LARGE: State<bool> = State::<bool>::new("native.style.large", false);
const DRAFT: State<String> = State::<String>::new("native.style.draft");
struct StyledApp;
impl blockui::Actor for StyledApp {
    const STATE: &'static [StateSpec] = &[LARGE.spec(), DRAFT.spec()];
    const ACTIONS: &'static [blockui::Action] =
        &[blockui::Action::new("native.style.toggle", |cx, _| {
            LARGE.set(cx, !LARGE.get(cx)?)
        })];
    fn view(_: &Cx<'_>) -> blockui::Result<View> {
        Ok(Card::default().children(vec![
            Text::new("Reactive native style")
                .key("native.style.label")
                .style_reactive(|cx| {
                    let large = LARGE.get(cx)?;
                    Ok(ViewStyle {
                        font_size: if large { 32.0 } else { 14.0 },
                        text_tone: if large {
                            TextTone::Error
                        } else {
                            TextTone::Muted
                        },
                        ..Default::default()
                    })
                }),
            TextInput::new(DRAFT).key("native.style.editor").into_view(),
        ]))
    }
}
#[test]
fn reactive_style_updates_native_layout_without_replacing_editor() -> Result<(), String> {
    let actor = Actor::for_test::<StyledApp>()?;
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    let label = key("native.style.label");
    let editor = key("native.style.editor");
    let before = source.build_frame(&mut tcx, 640, 240);
    let panel = before
        .iter()
        .find(|node| node.parent_node_id.is_none())
        .unwrap();
    assert_eq!(panel.border_widths, [1.0; 4]);
    assert_eq!(
        panel.paint.border,
        Some((1.0, Color::from_rgb8(226, 230, 236)))
    );
    let node_id = source.ids[&label];
    let small = before.iter().find(|node| node.node_id == node_id).unwrap();
    assert_eq!(small.paint.font_size, 14.0);
    let old_height = small.rect[3] - small.rect[1];
    source.set_focus(Some(editor));
    let editor_id = source.ids[&editor];
    let editor_paint = &before
        .iter()
        .find(|node| node.node_id == editor_id)
        .unwrap()
        .paint;
    assert_eq!(editor_paint.border_radii, CornerRadii::uniform(6.0));
    let evaluations = source.actor.view.bindings[&editor].evaluations;
    source.actor.dispatch(Message {
        action: key("native.style.toggle"),
        value: 0,
    })?;
    assert!(!source.layout_children_changed());
    let after = source.build_frame(&mut tcx, 640, 240);
    assert_eq!(source.ids[&label], node_id);
    assert_eq!(source.ids[&editor], editor_id);
    assert_eq!(source.focus, Some(editor));
    assert_eq!(source.actor.view.bindings[&editor].evaluations, evaluations);
    let large = after.iter().find(|node| node.node_id == node_id).unwrap();
    assert_eq!(large.paint.font_size, 32.0);
    assert!(large.rect[3] - large.rect[1] > old_height);
    assert_ne!(small.paint.text_color, large.paint.text_color);
    Ok(())
}
#[test]
#[ignore = "explicit isolated Vello reactive style pixels"]
fn reactive_style_pixels() -> Result<(), String> {
    let actor = Actor::for_test::<StyledApp>()?;
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    let mut images = Vec::new();
    for index in 0..2 {
        if index == 1 {
            source.actor.dispatch(Message {
                action: key("native.style.toggle"),
                value: 0,
            })?;
        }
        let nodes = source.build_frame(&mut tcx, 640, 240);
        let mut scene = shell::Scene::new();
        shell::scene::build_scene_scaled(
            &mut scene,
            &nodes,
            &mut tcx,
            640,
            240,
            source.base_color(),
            1.0,
        );
        let image = shell::renderer::render_to_image(&scene, 640, 240, source.base_color())
            .map_err(|error| error.to_string())?;
        let rect = source
            .widget_layout_rect(key("native.style.label"))
            .unwrap();
        let mut red_ink = 0;
        for y in rect[1] as u32..rect[3] as u32 {
            for x in rect[0] as u32..rect[2] as u32 {
                let pixel = image.get_pixel(x, y);
                if i16::from(pixel[0]) - i16::from(pixel[1]) > 40
                    && i16::from(pixel[0]) - i16::from(pixel[2]) > 40
                {
                    red_ink += 1;
                }
            }
        }
        if index == 0 {
            assert_eq!(red_ink, 0);
        } else {
            assert!(red_ink > 30);
        }
        images.push(image);
    }
    assert_ne!(images[0], images[1]);
    Ok(())
}

const SCOPE_WIDE: State<bool> = State::<bool>::new("native.scope.wide", false);
static SCOPE_BUILDS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
const STYLED_SCOPE: blockui::Scope = blockui::Scope::new("native.scope.panel", |_| {
    SCOPE_BUILDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Ok(blockui::Column {
        padding: 0.0,
        ..Default::default()
    }
    .children(vec![
        TextInput::new(DRAFT).key("native.scope.editor").into_view(),
    ])
    .style_reactive(|cx| {
        Ok(ViewStyle {
            width: if SCOPE_WIDE.get(cx)? { 280.0 } else { 180.0 },
            surface: blockui::Surface::Panel,
            radius: 8.0,
            ..Default::default()
        })
    }))
});
struct ScopedStyleApp;
impl blockui::Actor for ScopedStyleApp {
    const STATE: &'static [StateSpec] = &[SCOPE_WIDE.spec(), DRAFT.spec()];
    const ACTIONS: &'static [blockui::Action] =
        &[blockui::Action::new("native.scope.toggle", |cx, _| {
            SCOPE_WIDE.set(cx, !SCOPE_WIDE.get(cx)?)
        })];
    const SCOPES: &'static [blockui::Scope] = &[STYLED_SCOPE];
    fn view(_: &Cx<'_>) -> blockui::Result<View> {
        Ok(blockui::Column::default().children(vec![STYLED_SCOPE.into_view()]))
    }
}
#[test]
fn scope_root_style_reaches_native_layout_and_keeps_editor_and_focus() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<ScopedStyleApp>()?)?;
    let mut tcx = TextContext::new();
    let editor = key("native.scope.editor");
    let initial = source.build_frame(&mut tcx, 640, 240);
    let panel_id = source.ids[&STYLED_SCOPE.id()];
    let editor_id = source.ids[&editor];
    let panel = initial.iter().find(|n| n.node_id == panel_id).unwrap();
    assert!((panel.rect[2] - panel.rect[0] - 180.0).abs() < 0.01);
    assert!(panel.paint.background.is_some());
    source.set_focus(Some(editor));
    let builds = SCOPE_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
    let evaluations = source.actor.view.bindings[&editor].evaluations;
    source.actor.dispatch(Message {
        action: key("native.scope.toggle"),
        value: 0,
    })?;
    assert!(!source.layout_children_changed());
    let next = source.build_frame(&mut tcx, 640, 240);
    let panel = next.iter().find(|n| n.node_id == panel_id).unwrap();
    assert!((panel.rect[2] - panel.rect[0] - 280.0).abs() < 0.01);
    assert_eq!(source.ids[&editor], editor_id);
    assert_eq!(source.focus, Some(editor));
    assert_eq!(source.actor.view.bindings[&editor].evaluations, evaluations);
    assert_eq!(
        SCOPE_BUILDS.load(std::sync::atomic::Ordering::Relaxed),
        builds
    );
    Ok(())
}

#[test]
fn layout_children_invalidation_covers_structure_and_scroll_text() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<StyledApp>()?)?;
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 640, 240);
    assert!(!source.layout_children_changed());
    let label = key("native.style.label");
    let original = source.actor.view.bindings[&label].node;
    source
        .actor
        .view
        .bindings
        .get_mut(&label)
        .unwrap()
        .node
        .parent = 0;
    assert!(source.layout_children_changed());
    source.actor.view.bindings.get_mut(&label).unwrap().node = original;
    source.actor.view.order.reverse();
    assert!(source.layout_children_changed());
    source.actor.view.order.reverse();
    source
        .actor
        .view
        .bindings
        .get_mut(&label)
        .unwrap()
        .node
        .semantics
        .role = semantics::POPOVER_PANEL;
    assert!(source.layout_children_changed());
    source.actor.view.bindings.get_mut(&label).unwrap().node = original;
    source
        .actor
        .view
        .bindings
        .get_mut(&label)
        .unwrap()
        .text
        .clear();
    assert!(!source.layout_children_changed());
    // Isolate the scroll leaf transition with matching applied/current kinds.
    source
        .actor
        .view
        .bindings
        .get_mut(&label)
        .unwrap()
        .node
        .kind = SCROLL;
    assert!(source.layout_children_changed());
    source.applied.get_mut(&label).unwrap().0.kind = SCROLL;
    assert!(source.layout_children_changed());
    source.applied.get_mut(&label).unwrap().1.clear();
    assert!(!source.layout_children_changed());
    source.actor.view.bindings.get_mut(&label).unwrap().text = "scroll content".into();
    assert!(source.layout_children_changed());
    source.applied.get_mut(&label).unwrap().1 = "previous content".into();
    assert!(!source.layout_children_changed());
    source.actor.view.bindings.remove(&label);
    assert!(source.layout_children_changed());
    Ok(())
}

struct ScrollTopologyApp;
impl blockui::Actor for ScrollTopologyApp {
    const STATE: &'static [StateSpec] = &[DRAFT.spec(), LARGE.spec()];
    const ACTIONS: &'static [blockui::Action] = &[
        blockui::Action::new("native.scroll.text", |cx, value| {
            DRAFT.set(
                cx,
                match value {
                    0 => "",
                    1 => "First caption",
                    _ => "Second caption",
                },
            )
        }),
        blockui::Action::new("native.scroll.style", |cx, _| {
            LARGE.set(cx, !LARGE.get(cx)?)
        }),
    ];
    fn view(_: &Cx<'_>) -> blockui::Result<View> {
        Ok(blockui::ScrollView {
            key: blockui::Key::named("native.scroll.container"),
            text: DRAFT.into(),
            ..Default::default()
        }
        .children(vec![
            TextInput::new(DRAFT)
                .key("native.scroll.editor")
                .into_view(),
        ])
        .style_reactive(|cx| {
            Ok(ViewStyle {
                width: if LARGE.get(cx)? { 400.0 } else { 300.0 },
                ..Default::default()
            })
        }))
    }
}
#[test]
fn retained_scroll_children_survive_text_and_style_updates() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<ScrollTopologyApp>()?)?;
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 640, 240);
    let scroll = key("native.scroll.container");
    let editor = key("native.scroll.editor");
    let scroll_id = source.ids[&scroll];
    let editor_id = source.ids[&editor];
    let text_id = source.scroll_children[&scroll];
    source.set_focus(Some(editor));
    for (value, topology_changes) in [(1, true), (2, false), (0, true)] {
        source.actor.dispatch(Message {
            action: key("native.scroll.text"),
            value,
        })?;
        assert_eq!(source.layout_children_changed(), topology_changes);
        source.build_frame(&mut tcx, 640, 240);
        let expected = if value == 0 {
            vec![editor_id]
        } else {
            vec![text_id, editor_id]
        };
        assert_eq!(source.tree.children(scroll_id).unwrap(), expected);
        source.actor.dispatch(Message {
            action: key("native.scroll.style"),
            value: 0,
        })?;
        assert!(!source.layout_children_changed());
        source.build_frame(&mut tcx, 640, 240);
        assert_eq!(source.tree.children(scroll_id).unwrap(), expected);
        assert_eq!(source.ids[&editor], editor_id);
        assert_eq!(source.focus, Some(editor));
    }
    Ok(())
}

struct CapturedScopeApp;
impl blockui::Actor for CapturedScopeApp {
    const STATE: &'static [StateSpec] = &[LARGE.spec(), SCOPE_WIDE.spec(), DRAFT.spec()];
    const ACTIONS: &'static [blockui::Action] = &[
        blockui::Action::new("native.captured.structure", |cx, _| {
            SCOPE_WIDE.set(cx, !SCOPE_WIDE.get(cx)?)
        }),
        blockui::Action::new("native.captured.style", |cx, _| {
            LARGE.set(cx, !LARGE.get(cx)?)
        }),
    ];
    fn view(_: &Cx<'_>) -> blockui::Result<View> {
        let title = String::from("Captured panel");
        Ok(
            blockui::Column::default().children(vec![blockui::Scope::reactive(
                "native.captured",
                move |cx| {
                    let mut children = vec![TextInput::new(DRAFT).key("editor").into_view()];
                    if SCOPE_WIDE.get(cx)? {
                        children.push(Text::new(title.clone()).key("extra").into_view());
                    }
                    Ok(blockui::Column::default()
                        .children(children)
                        .style_reactive(|cx| {
                            Ok(ViewStyle {
                                width: if LARGE.get(cx)? { 280.0 } else { 180.0 },
                                ..Default::default()
                            })
                        }))
                },
            )]),
        )
    }
}
#[test]
fn captured_scope_structure_retains_native_editor_focus_and_selection() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<CapturedScopeApp>()?)?;
    let mut tcx = TextContext::new();
    let root = blockui::Key::named("native.captured");
    let editor = root.scoped(key("editor")).0;
    let extra = root.scoped(key("extra")).0;
    source.build_frame(&mut tcx, 640, 240);
    source.set_focus(Some(editor));
    source
        .actor
        .set_text(key("native.style.draft"), "draft survives")?;
    source.sync()?;
    source
        .editors
        .get_mut(&editor)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":4}}"#)
        .unwrap();
    source.build_frame(&mut tcx, 640, 240);
    let editor_id = source.ids[&editor];
    let editor_address = std::ptr::from_ref(source.editors[&editor].as_ref());
    let selection = source.editors[&editor].text_selection();
    for action in [
        "native.captured.structure",
        "native.captured.style",
        "native.captured.structure",
    ] {
        // Persist edits through the same stable-host path as normal events.
        source.persist_native_renderer();
        source.actor.dispatch(Message {
            action: key(action),
            value: 0,
        })?;
        source.build_frame(&mut tcx, 640, 240);
        assert_eq!(source.ids[&editor], editor_id);
        assert_eq!(
            std::ptr::from_ref(source.editors[&editor].as_ref()),
            editor_address
        );
        assert_eq!(
            source.editors[&editor].current_value(),
            Some("draft survives")
        );
        assert_eq!(source.focus, Some(editor));
        assert_eq!(source.editors[&editor].text_selection(), selection);
    }
    assert!(!source.ids.contains_key(&extra));
    let rect = source.widget_layout_rect(root.0).unwrap();
    assert!((rect[2] - rect[0] - 280.0).abs() < 0.01);
    Ok(())
}

#[test]
fn idle_frames_skip_reconciliation_but_reactive_changes_resume_it() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<StyledApp>()?)?;
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 640, 240);
    let editor = key("native.style.editor");
    source.set_focus(Some(editor));
    source.build_frame(&mut tcx, 640, 240);
    let reconciliations = source.sync_reconciliations;
    let ids = source.ids.clone();
    for _ in 0..5 {
        source.build_frame(&mut tcx, 640, 240);
    }
    assert_eq!(source.sync_reconciliations, reconciliations);
    assert_eq!(source.ids, ids);
    assert_eq!(source.focus, Some(editor));
    source
        .actor
        .tell(<StyledApp as blockui::Actor>::ACTIONS[0], 0)?;
    assert!(source.poll_async());
    source.build_frame(&mut tcx, 640, 240);
    assert!(source.sync_reconciliations > reconciliations);
    assert_eq!(source.ids, ids);
    assert_eq!(source.focus, Some(editor));
    let reconciliations = source.sync_reconciliations;
    source.build_frame(&mut tcx, 640, 240);
    assert_eq!(source.sync_reconciliations, reconciliations);
    Ok(())
}

#[test]
fn dialog_scope_preserves_native_identity_and_closed_layout() -> Result<(), String> {
    const OPEN: State<bool> = State::<bool>::new("scope-dialog.open", false);
    const SHOW: blockui::Action =
        blockui::Action::new("scope-dialog.show", |cx, _| OPEN.set(cx, true));
    const CLOSE: blockui::Action =
        blockui::Action::new("scope-dialog.close", |cx, _| OPEN.set(cx, false));
    const MODAL: blockui::Scope = blockui::Scope::new("scope-dialog.modal", |cx| {
        if !OPEN.get(cx)? {
            return Ok(View::empty());
        }
        Ok(blockui::Dialog::new("Scoped dialog", CLOSE).children(vec![
            TextInput::new(DRAFT).key("scope-dialog.editor").into_view(),
        ]))
    });
    struct App;
    impl blockui::Actor for App {
        const STATE: &'static [StateSpec] = &[OPEN.spec(), DRAFT.spec()];
        const ACTIONS: &'static [blockui::Action] = &[SHOW, CLOSE];
        const SCOPES: &'static [blockui::Scope] = &[MODAL];
        fn view(_: &Cx<'_>) -> blockui::Result<View> {
            Ok(blockui::Column::default()
                .children(vec![
                    blockui::Button::new("Open", SHOW)
                        .key("scope-dialog.opener")
                        .into_view(),
                    MODAL.into_view(),
                    Text::new("Footer").key("scope-dialog.footer").into_view(),
                ])
                .style(blockui::ViewStyle {
                    theme: blockui::ThemeMode::Dark,
                    ..Default::default()
                }))
        }
    }
    let mut source = NativeSource::without_watch(Actor::for_test::<App>()?)?;
    let mut tcx = TextContext::new();
    let closed = source.build_frame(&mut tcx, 640, 480);
    let footer_id = source.ids[&key("scope-dialog.footer")];
    let footer_rect = closed
        .iter()
        .find(|node| node.node_id == footer_id)
        .unwrap()
        .rect;
    let modal = key("scope-dialog.modal");
    assert!(source.modal.is_none());
    assert!(!closed.iter().any(|node| node.node_id == source.ids[&modal]));
    source.actor.dispatch_widget(key("scope-dialog.opener"))?;
    let opened = source.build_frame(&mut tcx, 640, 480);
    assert_eq!(
        opened
            .iter()
            .find(|node| node.node_id == source.ids[&modal])
            .unwrap()
            .paint
            .background,
        Some(theme::color(source.palette(modal).unwrap().panel))
    );
    assert_eq!(source.modal, Some(modal));
    assert_eq!(source.actor.view.bindings[&modal].text, "Scoped dialog");
    let editor = key("scope-dialog.editor");
    let editor_id = source.ids[&editor];
    assert_eq!(source.focus, Some(editor));
    source.actor.edit_text(editor, "Retained native draft")?;
    source.actor.replace_for_test::<App>()?;
    source.build_frame(&mut tcx, 640, 480);
    assert_eq!(source.ids[&editor], editor_id);
    assert_eq!(source.focus, Some(editor));
    source.actor.tell(CLOSE, 0)?;
    assert!(source.poll_async());
    let after = source.build_frame(&mut tcx, 640, 480);
    assert!(source.modal.is_none());
    assert!(!source.editors.contains_key(&editor));
    assert_eq!(
        after
            .iter()
            .find(|node| node.node_id == footer_id)
            .unwrap()
            .rect,
        footer_rect
    );
    Ok(())
}

struct PaintOnlyApp;
const PAINT_MODE: State<i64> = State::<i64>::new("paint.mode", 0);
impl blockui::Actor for PaintOnlyApp {
    const STATE: &'static [StateSpec] = &[PAINT_MODE.spec()];
    const ACTIONS: &'static [blockui::Action] =
        &[blockui::Action::new("paint.change", |cx, mode| {
            PAINT_MODE.set(cx, mode)
        })];
    fn view(_: &Cx<'_>) -> blockui::Result<View> {
        Ok(Card::default().key("paint.root").children(vec![
            Text::new("Retained measurement")
                .key("paint.label")
                .style_reactive(|cx| {
                    let mode = PAINT_MODE.get(cx)?;
                    Ok(ViewStyle {
                        text_tone: if mode == 0 {
                            TextTone::Muted
                        } else {
                            TextTone::Error
                        },
                        font_size: if mode == 2 { 32.0 } else { 14.0 },
                        ..Default::default()
                    })
                }),
        ]))
    }
}

#[test]
fn reactive_color_preserves_clean_layout_but_font_size_invalidates_it() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<PaintOnlyApp>()?)?;
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 640, 240);
    let root = source.ids[&key("paint.root")];
    let label = source.ids[&key("paint.label")];
    let old_color = source.tree.get_node_context(label).unwrap().text_color;
    assert!(!source.tree.dirty(root).unwrap());
    source.actor.dispatch(Message {
        action: key("paint.change"),
        value: 1,
    })?;
    source.sync()?;
    assert_ne!(
        source.tree.get_node_context(label).unwrap().text_color,
        old_color
    );
    assert!(
        !source.tree.dirty(label).unwrap(),
        "color must preserve measurement cache"
    );
    assert!(
        !source.tree.dirty(root).unwrap(),
        "color must preserve ancestor layout"
    );
    source.actor.dispatch(Message {
        action: key("paint.change"),
        value: 2,
    })?;
    source.sync()?;
    assert!(source.tree.dirty(label).unwrap());
    assert!(source.tree.dirty(root).unwrap());
    let nodes = source.build_frame(&mut tcx, 640, 240);
    assert_eq!(
        nodes
            .iter()
            .find(|node| node.node_id == label)
            .unwrap()
            .paint
            .font_size,
        32.0
    );
    Ok(())
}
