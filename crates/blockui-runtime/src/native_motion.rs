//! Stable renderer presentation tracks, separate from actor state and layout.
use super::*;
use blockui_motion::{
    DISCLOSURE_DURATION, Easing, HOVER_DURATION, Key as MotionKey, Property, SWITCH_DURATION,
};

#[derive(Clone)]
pub(super) struct HoverPaint {
    background: Option<Color>,
    border: Option<(f32, Color)>,
    outline_width: f32,
    outline_offset: f32,
    outline_color: Option<Color>,
}
impl HoverPaint {
    pub(super) fn capture(paint: &Paint) -> Self {
        Self {
            background: paint.background,
            border: paint.border,
            outline_width: paint.outline_width,
            outline_offset: paint.outline_offset,
            outline_color: paint.outline_color,
        }
    }
}
fn blend(a: Option<Color>, b: Option<Color>, amount: f32) -> Option<Color> {
    if amount <= 0.0 {
        return a;
    }
    if amount >= 1.0 {
        return b;
    }
    match (a, b) {
        (None, None) => None,
        (None, Some(b)) => Some(b.multiply_alpha(amount)),
        (Some(a), None) => Some(a.multiply_alpha(1.0 - amount)),
        (Some(a), Some(b)) => Some(a.lerp_rect(b, amount)),
    }
}
impl<M: NativeModel> NativeSource<M> {
    pub(super) fn sync_motion(&mut self) {
        let now = self
            .motion_manual_time
            .unwrap_or_else(|| self.motion_epoch.elapsed());
        self.reconcile_scroll_motion();
        // Manual test time and the native epoch both remain monotonic.
        let _ = self.motion.advance_to(now.max(self.motion.now()));
        self.motion.retain(|key| {
            self.actor
                .view()
                .bindings
                .get(&key.node)
                .is_some_and(|b| match key.property {
                    Property::Hover => matches!(
                        b.node.kind,
                        BUTTON | CHECKBOX | SELECT | ENTRY | TEXTAREA | SLIDER
                    ),
                    Property::Switch => b.node.semantics.role == semantics::SWITCH,
                    Property::Disclosure => b.node.semantics.expanded != 0,
                    Property::ScrollX | Property::ScrollY => b.node.kind == SCROLL,
                })
        });
        self.sample_scroll_motion();
        self.hover_paints
            .retain(|key, _| self.actor.view().bindings.contains_key(key));
        for (key, binding) in &self.actor.view().bindings {
            let node = binding.node;
            if matches!(
                node.kind,
                BUTTON | CHECKBOX | SELECT | ENTRY | TEXTAREA | SLIDER
            ) {
                let duration = if node.disabled != 0 {
                    Duration::ZERO
                } else {
                    HOVER_DURATION
                };
                self.motion
                    .retarget(
                        MotionKey::new(*key, Property::Hover),
                        f64::from(
                            self.hovered == Some(*key) && self.window_focused && node.disabled == 0,
                        ),
                        duration,
                        Easing::default(),
                    )
                    .unwrap();
            }
            if node.semantics.expanded != 0 {
                self.motion
                    .retarget(
                        MotionKey::new(*key, Property::Disclosure),
                        f64::from(node.semantics.expanded == 2),
                        if node.disabled != 0 {
                            Duration::ZERO
                        } else {
                            DISCLOSURE_DURATION
                        },
                        Easing::default(),
                    )
                    .unwrap();
            }
            if node.semantics.role == semantics::SWITCH {
                let checked = self.actor.widget_boolean(*key).unwrap_or(false);
                self.motion
                    .retarget(
                        MotionKey::new(*key, Property::Switch),
                        f64::from(checked),
                        if node.disabled != 0 {
                            Duration::ZERO
                        } else {
                            SWITCH_DURATION
                        },
                        Easing::default(),
                    )
                    .unwrap();
            }
        }
    }
    pub(super) fn paint_motion(&self, nodes: &mut [PlacedNode]) {
        // Paint fragments only: no State writes, view evaluation or Taffy invalidation.
        let keys: HashMap<_, _> = self.ids.iter().map(|(key, id)| (*id, *key)).collect();
        for node in nodes {
            let Some(key) = keys.get(&node.node_id) else {
                continue;
            };
            if self.keyboard_pressed == Some(*key) || self.pointer_pressed_visual == Some(*key) {
                continue;
            }
            let Some((base, hover)) = self.hover_paints.get(key) else {
                continue;
            };
            let amount = self
                .motion
                .value(MotionKey::new(*key, Property::Hover))
                .unwrap_or(0.0) as f32;
            node.paint.background = blend(base.background, hover.background, amount);
            node.paint.border = match (base.border, hover.border) {
                (Some((width, a)), Some((_, b))) => Some((width, a.lerp_rect(b, amount))),
                (a, _) => a,
            };
            if self.focus != Some(*key) && self.actor.view().bindings[key].node.kind == SLIDER {
                node.paint.outline_width =
                    base.outline_width + (hover.outline_width - base.outline_width) * amount;
                node.paint.outline_offset = hover.outline_offset;
                node.paint.outline_color = blend(base.outline_color, hover.outline_color, amount);
            }
        }
    }
    /// Accessibility/application preference. Existing tracks finish immediately.
    pub fn set_reduced_motion(&mut self, reduced: bool) {
        self.motion.set_reduced_motion(reduced);
        self.sample_scroll_motion();
        if let Some(wake) = self.wake.lock().unwrap().as_ref() {
            wake();
        }
    }
    #[cfg(all(test, feature = "testing"))]
    pub(super) fn use_manual_motion_for_test(&mut self) {
        self.motion_manual_time = Some(self.motion.now());
    }
    #[cfg(all(test, feature = "testing"))]
    pub(super) fn advance_motion_for_test(&mut self, elapsed: Duration) {
        let now = self
            .motion_manual_time
            .expect("configure manual motion before advancing");
        self.motion_manual_time = Some(now.checked_add(elapsed).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn absent_background_fades_alpha_without_darkening_the_hover_color() {
        let color = Color::from_rgb8(241, 245, 251);
        assert_eq!(blend(None, Some(color), 0.0), None);
        assert_eq!(
            blend(None, Some(color), 0.5),
            Some(color.multiply_alpha(0.5))
        );
        assert_eq!(
            blend(Some(color), None, 0.5),
            Some(color.multiply_alpha(0.5))
        );
        assert_eq!(blend(Some(color), None, 1.0), None);
    }
}
