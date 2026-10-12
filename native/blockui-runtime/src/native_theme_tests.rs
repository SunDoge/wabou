use super::*;
use blockui::{
    Button, Card, Column, Cx, Density, IntoView, State, StateSpec, Text, TextInput, ThemeMode,
    View, ViewStyle,
};
use blockui_abi::theme::Palette;
const MODE: State<i64> = State::<i64>::new("theme.mode", 1);
const DENSE: State<bool> = State::<bool>::new("theme.dense", true);
const DRAFT: State<String> = State::<String>::with_default("theme.draft", "中文 🦀 draft");
const MODE_ACTION: blockui::Action =
    blockui::Action::new("theme.set", |cx, value| MODE.set(cx, value));
const DENSITY_ACTION: blockui::Action =
    blockui::Action::new("theme.density", |cx, value| DENSE.set(cx, value != 0));
struct Themed;
impl blockui::Actor for Themed {
    const STATE: &'static [StateSpec] = &[MODE.spec(), DENSE.spec(), DRAFT.spec()];
    const ACTIONS: &'static [blockui::Action] = &[MODE_ACTION, DENSITY_ACTION];
    fn view(_: &Cx<'_>) -> blockui::Result<View> {
        Ok(Column::default()
            .key("theme.root")
            .children(vec![
                Text::new("Stable label").key("theme.label").into_view(),
                TextInput::new(DRAFT).key("theme.editor").into_view(),
                Button::new("Action", MODE_ACTION)
                    .key("theme.button")
                    .into_view(),
                Card::default()
                    .key("theme.nested")
                    .children(vec![
                        Button::new("Nested", MODE_ACTION)
                            .key("theme.nested.button")
                            .into_view(),
                    ])
                    .style(ViewStyle {
                        theme: ThemeMode::Light,
                        density: Density::Comfortable,
                        surface: blockui::Surface::Panel,
                        ..Default::default()
                    }),
            ])
            .style_reactive(|cx| {
                Ok(ViewStyle {
                    theme: match MODE.get(cx)? {
                        1 => ThemeMode::Light,
                        2 => ThemeMode::Dark,
                        3 => ThemeMode::System,
                        _ => ThemeMode::Inherit,
                    },
                    density: if DENSE.get(cx)? {
                        Density::Compact
                    } else {
                        Density::Comfortable
                    },
                    ..Default::default()
                })
            }))
    }
}
fn source() -> Result<NativeSource, String> {
    NativeSource::without_watch(Actor::for_test::<Themed>()?)
}
#[test]
fn sorted_preflight_rejects_same_size_cache_key_mismatch() -> Result<(), String> {
    let mut source = source()?;
    let previous = source.applied.remove(&key("theme.label")).unwrap();
    source
        .applied
        .insert(key("unexpected.cached.key"), previous);
    assert!(source.projection_needs_sync());
    assert_eq!(source.projection_changes(), (true, true));
    Ok(())
}
#[test]
fn theme_table_reuses_idle_and_text_updates_but_tracks_reparenting() -> Result<(), String> {
    let mut source = source()?;
    let rebuilds = source.theme_rebuilds;
    source.sync()?;
    assert_eq!(source.theme_rebuilds, rebuilds);
    source
        .actor
        .edit_text(key("theme.editor"), "Changed draft")?;
    source.sync()?;
    assert_eq!(source.theme_rebuilds, rebuilds);
    let nested = key("theme.nested.button");
    let root = key("theme.root");
    assert_ne!(source.themes[&nested], source.themes[&root]);
    source
        .actor
        .view
        .bindings
        .get_mut(&nested)
        .unwrap()
        .node
        .parent = root;
    source.sync()?;
    assert_eq!(source.theme_rebuilds, rebuilds + 1);
    assert_eq!(source.themes[&nested], source.themes[&root]);
    Ok(())
}
#[test]
fn theme_switch_preserves_editor_focus_selection_and_binding_work() -> Result<(), String> {
    let mut source = source()?;
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 720, 480);
    let editor = key("theme.editor");
    let button = key("theme.button");
    let label = key("theme.label");
    let nested = key("theme.nested.button");
    let editor_id = source.ids[&editor];
    let instance = &*source.editors[&editor] as *const _;
    source.set_focus(Some(editor));
    source
        .editors
        .get_mut(&editor)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":5}}"#)?;
    source.build_frame(&mut tcx, 720, 480);
    let selection = source.editors[&editor].text_selection();
    let evaluations = source.actor.view.bindings[&label].evaluations;
    let version = source.actor.state_address();
    source.actor.dispatch(Message {
        action: MODE_ACTION.id,
        value: 2,
    })?;
    source.sync()?;
    for id in source.ids.values() {
        assert!(
            !source.tree.dirty(*id).unwrap(),
            "palette change dirtied layout"
        );
    }
    let dark = source.build_frame(&mut tcx, 720, 480);
    assert_eq!(source.ids[&editor], editor_id);
    assert_eq!(&*source.editors[&editor] as *const _, instance);
    assert_eq!(source.editors[&editor].text_selection(), selection);
    assert_eq!(source.focus, Some(editor));
    assert_eq!(source.actor.state_address(), version);
    assert_eq!(source.actor.text("theme.draft"), "中文 🦀 draft");
    assert_eq!(source.actor.view.bindings[&label].evaluations, evaluations);
    assert_eq!(
        source.actor.view.bindings[&key("theme.root")].style_evaluations,
        2
    );
    assert!(
        !source
            .actor
            .view
            .root_dependencies
            .contains(&crate::Dependency::State(MODE.key()))
    );
    let paint = |k| {
        &dark
            .iter()
            .find(|n| n.node_id == source.ids[&k])
            .unwrap()
            .paint
    };
    assert_eq!(
        paint(button).background,
        Some(theme::color(Palette::DARK.accent))
    );
    assert_eq!(
        paint(nested).background,
        Some(theme::color(Palette::LIGHT.accent))
    );
    let rect = |k| {
        dark.iter()
            .find(|n| n.node_id == source.ids[&k])
            .unwrap()
            .rect
    };
    assert_eq!(rect(button)[3] - rect(button)[1], 30.0);
    assert_eq!(rect(nested)[3] - rect(nested)[1], 36.0);
    source.actor.dispatch(Message {
        action: DENSITY_ACTION.id,
        value: 0,
    })?;
    source.build_frame(&mut tcx, 720, 480);
    assert_eq!(source.focus, Some(editor));
    assert_eq!(source.editors[&editor].text_selection(), selection);
    Ok(())
}
#[test]
fn system_preference_is_host_owned_and_independent_between_windows() -> Result<(), String> {
    let mut a = source()?;
    let mut b = source()?;
    let mut tcx = TextContext::new();
    for source in [&mut a, &mut b] {
        source.actor.dispatch(Message {
            action: MODE_ACTION.id,
            value: 3,
        })?;
    }
    a.handle_event(UiEvent::WindowMetrics(shell::WindowMetrics {
        color_scheme: Some(shell::ColorScheme::Dark),
        ..Default::default()
    }));
    b.handle_event(UiEvent::WindowMetrics(shell::WindowMetrics {
        color_scheme: Some(shell::ColorScheme::Light),
        ..Default::default()
    }));
    a.build_frame(&mut tcx, 720, 480);
    b.build_frame(&mut tcx, 720, 480);
    assert_eq!(a.palette(key("theme.button")), Some(Palette::DARK));
    assert_eq!(b.palette(key("theme.button")), Some(Palette::LIGHT));
    let evaluations = a.actor.view.bindings[&key("theme.root")].style_evaluations;
    a.handle_event(UiEvent::WindowMetrics(shell::WindowMetrics {
        color_scheme: Some(shell::ColorScheme::Light),
        ..Default::default()
    }));
    a.build_frame(&mut tcx, 720, 480);
    assert_eq!(a.palette(key("theme.button")), Some(Palette::LIGHT));
    assert_eq!(
        a.actor.view.bindings[&key("theme.root")].style_evaluations,
        evaluations
    );
    Ok(())
}

struct ThemedScroll;
impl blockui::Actor for ThemedScroll {
    const STATE: &'static [StateSpec] = &[MODE.spec(), DRAFT.spec()];
    const ACTIONS: &'static [blockui::Action] = &[MODE_ACTION];
    fn view(_: &Cx<'_>) -> blockui::Result<View> {
        Ok(Column::default()
            .children(vec![
                blockui::ScrollView {
                    text: DRAFT.into(),
                    height: 100.0,
                    ..Default::default()
                }
                .key("theme.scroll")
                .children(vec![
                    TextInput::new(DRAFT).key("theme.scroll.editor").into_view(),
                ]),
                Column::default()
                    .key("theme.scroll.panel")
                    .children(vec![])
                    .style(ViewStyle {
                        surface: blockui::Surface::Panel,
                        ..Default::default()
                    }),
            ])
            .style_reactive(|cx| {
                Ok(ViewStyle {
                    theme: if MODE.get(cx)? == 2 {
                        ThemeMode::Dark
                    } else {
                        ThemeMode::Light
                    },
                    ..Default::default()
                })
            }))
    }
}
#[test]
fn scroll_surfaces_and_captions_follow_palette_without_dirtying_layout() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<ThemedScroll>()?)?;
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 720, 480);
    let scroll = key("theme.scroll");
    let panel = key("theme.scroll.panel");
    let editor = key("theme.scroll.editor");
    let editor_id = source.ids[&editor];
    source.set_focus(Some(editor));
    for (mode, palette) in [(2, Palette::DARK), (1, Palette::LIGHT)] {
        source.actor.dispatch(Message {
            action: MODE_ACTION.id,
            value: mode,
        })?;
        source.sync()?;
        assert_eq!(
            source
                .tree
                .get_node_context(source.ids[&scroll])
                .unwrap()
                .background,
            Some(theme::color(palette.canvas))
        );
        assert_eq!(
            source
                .tree
                .get_node_context(source.ids[&panel])
                .unwrap()
                .background,
            Some(theme::color(palette.panel))
        );
        let text = source.scroll_children[&scroll];
        assert_eq!(
            source.tree.get_node_context(text).unwrap().text_color,
            theme::color(palette.text)
        );
        assert!(
            !source.tree.dirty(text).unwrap(),
            "caption palette dirtied measurement"
        );
        for id in source.ids.values() {
            assert!(!source.tree.dirty(*id).unwrap());
        }
        source.build_frame(&mut tcx, 720, 480);
        assert_eq!(source.ids[&editor], editor_id);
        assert_eq!(source.focus, Some(editor));
    }
    Ok(())
}
