use super::*;
use blockui::{Badge, BadgeVariant, IntoView, Separator};
struct Fixture;
impl blockui::Actor for Fixture {
    const STATE: &'static [blockui::StateSpec] = &[];
    const ACTIONS: &'static [blockui::Action] = &[];
    fn view(_: &blockui::Cx<'_>) -> blockui::Result<blockui::View> {
        Ok(blockui::Column {
            padding: 24.0,
            gap: 16.0,
            ..Default::default()
        }
        .key("root")
        .children(vec![
            blockui::Row {
                padding: 0.0,
                gap: 8.0,
                ..Default::default()
            }
            .key("badges")
            .children(
                [
                    BadgeVariant::Default,
                    BadgeVariant::Secondary,
                    BadgeVariant::Outline,
                    BadgeVariant::Destructive,
                ]
                .into_iter()
                .enumerate()
                .map(|(i, variant)| {
                    Badge::new(["Default", "Secondary", "Outline", "Destructive"][i])
                        .key(format!("badge.{i}"))
                        .variant(variant)
                        .into_view()
                })
                .collect(),
            ),
            Separator::horizontal().key("horizontal").into_view(),
            blockui::Row {
                padding: 0.0,
                gap: 12.0,
                ..Default::default()
            }
            .key("row")
            .children(vec![
                blockui::Text::new("Guide").key("guide").into_view(),
                Separator::vertical(24.0).key("vertical").into_view(),
                blockui::Text::new("Updated").key("updated").into_view(),
            ]),
            blockui::Button::label("Continue")
                .key("continue")
                .on_click(|_| Ok(())),
        ]))
    }
}
#[test]
fn labels_are_compact_and_separators_have_one_pixel_thickness() -> Result<(), String> {
    let mut s = NativeSource::without_watch(Actor::for_test::<Fixture>()?)?;
    s.set_semantics_enabled(true);
    let mut tcx = TextContext::new();
    s.build_frame(&mut tcx, 600, 400);
    let h = s.widget_layout_rect(key("horizontal")).unwrap();
    let v = s.widget_layout_rect(key("vertical")).unwrap();
    assert_eq!(h[3] - h[1], 1.0);
    assert_eq!(v[2] - v[0], 1.0);
    assert_eq!(v[3] - v[1], 24.0);
    for i in 0..4 {
        let r = s.widget_layout_rect(key(&format!("badge.{i}"))).unwrap();
        assert!(r[2] - r[0] < 120.0 && r[3] - r[1] < 30.0, "{r:?}");
    }
    s.focus_next(false);
    assert_eq!(s.focus, Some(key("continue")));
    let semantics = s.semantic_snapshot().unwrap();
    for key in [key("horizontal"), key("vertical")] {
        assert!(
            semantics
                .nodes
                .iter()
                .any(|n| n.id == key && n.role == shell::SemanticRole::Separator)
        );
    }
    s.build_frame(&mut tcx, 240, 400);
    let first = s.widget_layout_rect(key("badge.0")).unwrap();
    let last = s.widget_layout_rect(key("badge.3")).unwrap();
    assert!(last[1] > first[1], "badge row wraps in a narrow window");
    for i in 0..4 {
        let r = s.widget_layout_rect(key(&format!("badge.{i}"))).unwrap();
        assert!(r[0] >= 24.0 && r[2] <= 216.0, "{r:?}");
    }
    Ok(())
}
#[test]
#[ignore = "explicit offscreen Vello capture"]
fn label_visuals() -> Result<(), String> {
    for scale in [1, 2] {
        let mut s = NativeSource::without_watch(Actor::for_test::<Fixture>()?)?;
        s.scale = f64::from(scale);
        let mut tcx = TextContext::new();
        let nodes = s.build_frame(&mut tcx, 600, 400);
        let mut scene = shell::Scene::new();
        shell::scene::build_scene_scaled(
            &mut scene,
            &nodes,
            &mut tcx,
            600,
            400,
            s.base_color(),
            f64::from(scale),
        );
        let image =
            shell::renderer::render_to_image(&scene, 600 * scale, 400 * scale, s.base_color())
                .map_err(|e| e.to_string())?;
        if scale == 1 {
            image
                .save("../../reports/native-labels.png")
                .map_err(|e| e.to_string())?;
        }
        let h = s.widget_layout_rect(key("horizontal")).unwrap();
        let pixel = image.get_pixel(
            ((h[0] + 10.0) * scale as f32) as u32,
            ((h[1] + 0.5) * scale as f32) as u32,
        );
        assert!(
            pixel[0] < 240 && pixel[0] > 200,
            "separator must paint: {pixel:?}"
        );
    }
    Ok(())
}

struct DynamicBadgeFixture;
const SHOW_BADGE: blockui::State<bool> = blockui::State::<bool>::new("badge.visible", false);
const TINT: blockui::State<bool> = blockui::State::<bool>::new("badge.tint", false);
const TOGGLE_TINT: blockui::Action =
    blockui::Action::new("badge.tint.toggle", |cx, _| TINT.set(cx, !TINT.get(cx)?));
const TOGGLE_BADGE: blockui::Action = blockui::Action::new("badge.toggle", |cx, _| {
    SHOW_BADGE.set(cx, !SHOW_BADGE.get(cx)?)
});
impl blockui::Actor for DynamicBadgeFixture {
    const STATE: &'static [blockui::StateSpec] = &[SHOW_BADGE.spec(), TINT.spec()];
    const ACTIONS: &'static [blockui::Action] = &[TOGGLE_BADGE, TOGGLE_TINT];
    fn view(cx: &blockui::Cx<'_>) -> blockui::Result<blockui::View> {
        let mut children = vec![
            blockui::Text::new("Stable sibling")
                .key("stable.sibling")
                .into_view()
                .style_reactive(|cx| {
                    Ok(blockui::ViewStyle {
                        text_tone: if TINT.get(cx)? {
                            blockui::TextTone::Error
                        } else {
                            blockui::TextTone::Default
                        },
                        ..Default::default()
                    })
                }),
        ];
        if SHOW_BADGE.get(cx)? {
            children.push(Badge::new("Added badge").key("added.badge").into_view());
        }
        Ok(blockui::Row::default()
            .key("dynamic.badges")
            .children(children))
    }
}

#[test]
fn badge_membership_refreshes_unchanged_row_wrapping() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<DynamicBadgeFixture>()?)?;
    let row = key("dynamic.badges");
    let row_id = source.ids[&row];
    let sibling_id = source.ids[&key("stable.sibling")];
    assert_eq!(
        source.tree.style(row_id).unwrap().flex_wrap,
        FlexWrap::NoWrap
    );
    for expected in [FlexWrap::Wrap, FlexWrap::NoWrap, FlexWrap::Wrap] {
        source.actor.dispatch(Message {
            action: key("badge.toggle"),
            value: 0,
        })?;
        source.sync()?;
        assert_eq!(source.ids[&row], row_id);
        assert_eq!(source.ids[&key("stable.sibling")], sibling_id);
        assert_eq!(source.tree.style(row_id).unwrap().flex_wrap, expected);
        let reconciliations = source.sync_reconciliations;
        let badge_syncs = source.badge_row_syncs;
        source.sync()?;
        assert_eq!(source.sync_reconciliations, reconciliations);
        assert_eq!(source.badge_row_syncs, badge_syncs);
        source.actor.dispatch(Message {
            action: key("badge.tint.toggle"),
            value: 0,
        })?;
        source.sync()?;
        assert_eq!(
            source.badge_row_syncs, badge_syncs,
            "paint-only changes must not rebuild the Badge/Row index"
        );
        assert_eq!(source.tree.style(row_id).unwrap().flex_wrap, expected);
    }
    Ok(())
}
