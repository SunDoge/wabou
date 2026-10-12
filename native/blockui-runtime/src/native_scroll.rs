//! Discrete wheel smoothing on the stable renderer's frame clock.
//! Pixel gestures remain direct; pending targets contain only owned scalar data.
use super::*;
use blockui_motion::{Easing, Key as MotionKey, Property, SCROLL_DURATION};
const AXES: [Property; 2] = [Property::ScrollX, Property::ScrollY];
impl<M: NativeModel> NativeSource<M> {
    fn stop_scroll_motion(&mut self, key: u64) {
        self.scroll_targets.remove(&key);
        for axis in AXES {
            self.motion.remove(MotionKey::new(key, axis));
        }
    }
    pub(super) fn reconcile_scroll_motion(&mut self) {
        let removed: Vec<_> = self
            .scroll_targets
            .keys()
            .copied()
            .filter(|key| {
                let Some(binding) = self.actor.view().bindings.get(key) else {
                    return true;
                };
                let Some(state) = self.actor.view().renderer.get(key) else {
                    return true;
                };
                let offset = [state.scroll_x, state.scroll];
                // A focus reveal, scrollbar drag, resize clamp or app update supersedes animation.
                binding.node.kind != SCROLL
                    || AXES.iter().enumerate().any(|(i, axis)| {
                        self.motion
                            .value(MotionKey::new(*key, *axis))
                            .is_none_or(|value| {
                                offset[i] != value && offset[i] != f64::from(value as f32)
                            })
                    })
            })
            .collect();
        for key in removed {
            self.stop_scroll_motion(key);
        }
    }
    pub(super) fn sample_scroll_motion(&mut self) {
        let keys: Vec<_> = self.scroll_targets.keys().copied().collect();
        let mut changed = Vec::new();
        for key in keys {
            let Some(state) = self.actor.view_mut().renderer.get_mut(&key) else {
                continue;
            };
            let before = [state.scroll_x, state.scroll];
            let next = AXES.map(|axis| self.motion.value(MotionKey::new(key, axis)).unwrap());
            state.scroll_x = next[0];
            state.scroll = next[1];
            if next != before {
                changed.push(key);
            }
            if next == self.scroll_targets[&key] {
                self.stop_scroll_motion(key);
            }
        }
        for key in changed {
            self.notify_scroll(key);
        }
    }
    pub(super) fn scroll_wheel(&mut self, wheel: &shell::WheelEvent) {
        if !wheel.delta_x.is_finite() || !wheel.delta_y.is_finite() {
            return;
        }
        let Some(mut current) = self
            .hit(wheel.position)
            .filter(|key| !self.editors.contains_key(key))
        else {
            return;
        };
        let smooth = wheel.delta_mode == shell::WheelDeltaMode::Line;
        if smooth {
            self.sync_motion();
        }
        let mut remaining = [wheel.delta_x, wheel.delta_y];
        while let Some(binding) = self.actor.view().bindings.get(&current) {
            let node = binding.node;
            if node.kind == SCROLL {
                let range = self
                    .tree
                    .layout(self.ids[&current])
                    .map_or([0.0; 2], |layout| {
                        [
                            f64::from(layout.scroll_width()),
                            f64::from(layout.scroll_height()),
                        ]
                    });
                let state = &self.actor.view().renderer[&current];
                let actual = [state.scroll_x, state.scroll];
                let mut previous = if smooth {
                    self.scroll_targets.get(&current).copied().unwrap_or(actual)
                } else {
                    actual
                };
                for i in 0..2 {
                    // Reverse immediately instead of first consuming queued movement in the old direction.
                    if remaining[i] * (previous[i] - actual[i]) < 0.0 {
                        previous[i] = actual[i];
                    }
                }
                let target = [
                    (previous[0] + remaining[0]).clamp(0.0, range[0]),
                    (previous[1] + remaining[1]).clamp(0.0, range[1]),
                ];
                remaining[0] -= target[0] - previous[0];
                remaining[1] -= target[1] - previous[1];
                if smooth && target != actual {
                    for (i, axis) in AXES.iter().enumerate() {
                        let key = MotionKey::new(current, *axis);
                        if !self.scroll_targets.contains_key(&current) {
                            self.motion
                                .retarget(key, actual[i], Duration::ZERO, Easing::default())
                                .unwrap();
                        }
                        self.motion
                            .retarget(key, target[i], SCROLL_DURATION, Easing::default())
                            .unwrap();
                    }
                    self.scroll_targets.insert(current, target);
                } else if smooth {
                    self.stop_scroll_motion(current);
                } else {
                    self.stop_scroll_motion(current);
                    let state = self.actor.view_mut().renderer.get_mut(&current).unwrap();
                    state.scroll_x = target[0];
                    state.scroll = target[1];
                    if actual != target {
                        self.notify_scroll(current);
                    }
                }
            }
            if remaining == [0.0; 2] || node.parent == 0 {
                break;
            }
            current = node.parent;
        }
        if smooth {
            self.sample_scroll_motion();
        }
    }
}

#[cfg(all(test, feature = "testing"))]
mod tests {
    use super::*;
    use blockui::{
        Action, Actor as UiActor, Column, Cx, IntoView, ScrollView, StateSpec, Text, View,
    };
    struct SplitPage;
    impl UiActor for SplitPage {
        const STATE: &'static [StateSpec] = &[];
        const ACTIONS: &'static [Action] = &[];
        fn view(_: &Cx<'_>) -> blockui::Result<View> {
            use blockui::{Row, ViewStyle};
            let fill = ViewStyle {
                fill_height: true,
                ..Default::default()
            };
            let area = |name, width| {
                ScrollView::default()
                    .key(name)
                    .children(vec![
                        Column::default().children(vec![blockui::Spacer::new(1200.0).into_view()]),
                    ])
                    .style(ViewStyle { width, ..fill })
            };
            Ok(Column {
                gap: 12.0,
                padding: 0.0,
                ..Default::default()
            }
            .key("split.root")
            .children(vec![
                Text::new("Fixed header").key("split.header").into_view(),
                Row::default()
                    .key("split.body")
                    .children(vec![area("split.left", 160.0), area("split.right", 0.0)])
                    .style(fill),
            ])
            .style(fill))
        }
    }
    #[test]
    fn sibling_scroll_areas_fill_viewport_and_route_wheel_independently() -> Result<(), String> {
        let mut source = NativeSource::without_watch(Actor::for_test::<SplitPage>()?)?;
        source.set_reduced_motion(true);
        let mut tcx = TextContext::new();
        for height in [480, 320, 640] {
            source.build_frame(&mut tcx, 800, height);
            let header = source.widget_layout_rect(key("split.header")).unwrap();
            for (name, other) in [("split.left", "split.right"), ("split.right", "split.left")] {
                let rect = source.widget_layout_rect(key(name)).unwrap();
                assert!((rect[3] - height as f32).abs() < 0.1, "{name}: {rect:?}");
                let before = offset(&source, name);
                let other_before = offset(&source, other);
                source.handle_event(UiEvent::Wheel(shell::WheelEvent {
                    position: shell::Point {
                        x: f64::from(rect[0] + 20.0),
                        y: f64::from(rect[1] + 20.0),
                    },
                    delta_x: 0.0,
                    delta_y: 40.0,
                    delta_mode: shell::WheelDeltaMode::Line,
                    phase: shell::GesturePhase::Changed,
                    modifiers: Default::default(),
                }));
                source.build_frame(&mut tcx, 800, height);
                assert_eq!(offset(&source, name), before + 40.0);
                assert_eq!(offset(&source, other), other_before);
                assert_eq!(
                    source.widget_layout_rect(key("split.header")).unwrap(),
                    header
                );
                assert_eq!(source.widget_layout_rect(key(name)).unwrap(), rect);
            }
        }
        Ok(())
    }
    struct LongPage;
    impl UiActor for LongPage {
        const STATE: &'static [StateSpec] = &[];
        const ACTIONS: &'static [Action] = &[];
        fn view(_: &Cx<'_>) -> blockui::Result<View> {
            Ok(ScrollView {
                height: 160.0,
                ..Default::default()
            }
            .key("scroll.outer")
            .children(vec![
                ScrollView {
                    height: 120.0,
                    ..Default::default()
                }
                .key("scroll.inner")
                .children(vec![Column::default().key("scroll.content").children(
                    vec![
                        Text::new("Scrollable content")
                            .key("scroll.text")
                            .into_view(),
                        Text::new("Second row: smooth scrolling").key("scroll.second").into_view(),
                        blockui::Spacer::new(600.0).key("scroll.inner.spacer").into_view(),
                    ],
                )]),
                Column::default().key("scroll.tail").children(vec![
                    Text::new("Tail").into_view(),
                    blockui::Spacer::new(600.0)
                        .key("scroll.outer.spacer")
                        .into_view(),
                ]),
            ]))
        }
    }
    fn fixture() -> Result<(NativeSource, TextContext), String> {
        let mut source = NativeSource::without_watch(Actor::for_test::<LongPage>()?)?;
        source.use_manual_motion_for_test();
        let mut tcx = TextContext::new();
        source.build_frame(&mut tcx, 400, 160);
        Ok((source, tcx))
    }
    fn wheel(source: &mut NativeSource, delta: f64, mode: shell::WheelDeltaMode) {
        source.handle_event(UiEvent::Wheel(shell::WheelEvent {
            position: shell::Point { x: 20.0, y: 20.0 },
            delta_x: 0.0,
            delta_y: delta,
            delta_mode: mode,
            phase: shell::GesturePhase::Changed,
            modifiers: Default::default(),
        }));
    }
    fn offset(source: &NativeSource, name: &str) -> f64 {
        source.actor.view().renderer[&key(name)].scroll
    }
    #[test]
    fn line_wheel_interpolates_accumulates_reverses_and_goes_idle_without_layout()
    -> Result<(), String> {
        let (mut source, mut tcx) = fixture()?;
        let text_y = source.widget_layout_rect(key("scroll.text")).unwrap()[1];
        wheel(&mut source, 40.0, shell::WheelDeltaMode::Line);
        assert_eq!(offset(&source, "scroll.inner"), 0.0);
        assert_eq!(source.scroll_targets[&key("scroll.inner")][1], 40.0);
        assert!(source.has_anim());
        source.advance_motion_for_test(Duration::from_millis(30));
        source.sync()?;
        let first = offset(&source, "scroll.inner");
        assert!(first > 0.0 && first < 40.0);
        for id in source.ids.values() {
            assert!(!source.tree.dirty(*id).unwrap());
        }
        source.build_frame(&mut tcx, 400, 160);
        assert!(
            (source.widget_layout_rect(key("scroll.text")).unwrap()[1] - (text_y - first as f32))
                .abs()
                < 0.001
        );
        assert_ne!(
            source.hit(shell::Point { x: 20.0, y: 20.0 }),
            Some(key("scroll.text"))
        );
        wheel(&mut source, 40.0, shell::WheelDeltaMode::Line);
        assert_eq!(source.scroll_targets[&key("scroll.inner")][1], 80.0);
        let before_reverse = offset(&source, "scroll.inner");
        wheel(&mut source, -10.0, shell::WheelDeltaMode::Line);
        assert!(
            (source.scroll_targets[&key("scroll.inner")][1] - (before_reverse - 10.0)).abs()
                < 0.001
        );
        source.advance_motion_for_test(Duration::from_millis(30));
        source.build_frame(&mut tcx, 400, 160);
        assert!(offset(&source, "scroll.inner") < before_reverse);
        source.advance_motion_for_test(SCROLL_DURATION);
        source.build_frame(&mut tcx, 400, 160);
        assert!(source.scroll_targets.is_empty());
        assert!(!source.has_anim());
        assert!(!source.motion.is_active());
        Ok(())
    }
    #[test]
    fn nested_targets_bubble_only_unconsumed_distance_and_reduced_motion_snaps()
    -> Result<(), String> {
        let (mut source, mut tcx) = fixture()?;
        let inner = key("scroll.inner");
        let range = f64::from(
            source
                .tree
                .layout(source.ids[&inner])
                .unwrap()
                .scroll_height(),
        );
        assert!(range > 400.0);
        wheel(&mut source, range + 30.0, shell::WheelDeltaMode::Line);
        assert_eq!(source.scroll_targets[&inner][1], range);
        assert_eq!(source.scroll_targets[&key("scroll.outer")][1], 30.0);
        source.advance_motion_for_test(Duration::from_millis(30));
        source.build_frame(&mut tcx, 400, 160);
        assert!(offset(&source, "scroll.inner") > 0.0 && offset(&source, "scroll.inner") < range);
        source.set_reduced_motion(true);
        source.build_frame(&mut tcx, 400, 160);
        assert_eq!(offset(&source, "scroll.inner"), range);
        assert_eq!(offset(&source, "scroll.outer"), 30.0);
        assert!(source.scroll_targets.is_empty());
        assert!(!source.has_anim());
        Ok(())
    }
    #[test]
    fn pixel_gesture_cancels_pending_line_animation_and_nonfinite_deltas_are_ignored()
    -> Result<(), String> {
        let (mut source, mut tcx) = fixture()?;
        wheel(&mut source, 40.0, shell::WheelDeltaMode::Line);
        source.advance_motion_for_test(Duration::from_millis(30));
        source.build_frame(&mut tcx, 400, 160);
        let before = offset(&source, "scroll.inner");
        wheel(&mut source, 5.0, shell::WheelDeltaMode::Pixel);
        assert_eq!(offset(&source, "scroll.inner"), before + 5.0);
        assert!(source.scroll_targets.is_empty());
        wheel(&mut source, f64::NAN, shell::WheelDeltaMode::Line);
        wheel(&mut source, f64::INFINITY, shell::WheelDeltaMode::Pixel);
        assert_eq!(offset(&source, "scroll.inner"), before + 5.0);
        source.advance_motion_for_test(SCROLL_DURATION);
        source.build_frame(&mut tcx, 400, 160);
        assert!((offset(&source, "scroll.inner") - (before + 5.0)).abs() < 0.001);
        Ok(())
    }
    #[test]
    fn programmatic_offset_change_supersedes_pending_wheel_target() -> Result<(), String> {
        let (mut source, mut tcx) = fixture()?;
        wheel(&mut source, 40.0, shell::WheelDeltaMode::Line);
        source
            .actor
            .view_mut()
            .renderer
            .get_mut(&key("scroll.inner"))
            .unwrap()
            .scroll = 200.0;
        source.advance_motion_for_test(Duration::from_millis(30));
        source.build_frame(&mut tcx, 400, 160);
        assert_eq!(offset(&source, "scroll.inner"), 200.0);
        assert!(source.scroll_targets.is_empty());
        Ok(())
    }
    #[test]
    fn removing_scroll_nodes_retires_pending_targets_and_tracks() -> Result<(), String> {
        struct Empty;
        impl UiActor for Empty {
            const STATE: &'static [StateSpec] = &[];
            const ACTIONS: &'static [Action] = &[];
            fn view(_: &Cx<'_>) -> blockui::Result<View> {
                Ok(Column::default()
                    .key("empty.root")
                    .children(vec![Text::new("Empty").into_view()]))
            }
        }
        let (mut source, mut tcx) = fixture()?;
        wheel(&mut source, 40.0, shell::WheelDeltaMode::Line);
        assert!(!source.scroll_targets.is_empty());
        source.actor.replace_for_test::<Empty>()?;
        source.build_frame(&mut tcx, 400, 160);
        assert!(source.scroll_targets.is_empty());
        assert!(source.motion.is_empty());
        assert!(!source.has_anim());
        Ok(())
    }
    #[test]
    #[ignore = "explicit Vello scroll frames at 1x and 2x"]
    fn smooth_scroll_pixels_follow_intermediate_positions() -> Result<(), String> {
        for scale in [1u32, 2] {
            let (mut source, mut tcx) = fixture()?;
            let mut frames = Vec::new();
            wheel(&mut source, 40.0, shell::WheelDeltaMode::Line);
            for (stage, elapsed) in [("start", 0), ("middle", 30), ("end", 120)] {
                source.advance_motion_for_test(Duration::from_millis(elapsed));
                let nodes = source.build_frame(&mut tcx, 400, 160);
                let mut scene = shell::Scene::new();
                shell::scene::build_scene_scaled(
                    &mut scene,
                    &nodes,
                    &mut tcx,
                    400,
                    160,
                    source.base_color(),
                    f64::from(scale),
                );
                let image = shell::renderer::render_to_image(
                    &scene,
                    400 * scale,
                    160 * scale,
                    source.base_color(),
                )
                .map_err(|e| e.to_string())?;
                if scale == 1 {
                    image
                        .save(format!("../../reports/scroll-{stage}.png"))
                        .map_err(|e| e.to_string())?;
                }
                frames.push((offset(&source, "scroll.inner"), image.into_raw()));
            }
            assert_eq!(frames[0].0, 0.0);
            assert!(frames[1].0 > 0.0 && frames[1].0 < 40.0);
            assert_eq!(frames[2].0, 40.0);
            assert_ne!(frames[0].1, frames[1].1);
            assert_ne!(frames[1].1, frames[2].1);
        }
        Ok(())
    }
}
