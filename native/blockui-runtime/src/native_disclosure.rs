//! Accordion presentation and focus-only navigation, owned by the stable host.
use super::*;
impl<M: NativeModel> NativeSource<M> {
    pub(super) fn navigate_disclosure(&mut self, movement: &str) -> bool {
        let Some(current) = self.focus else {
            return false;
        };
        let Some(next) =
            crate::interaction::disclosure_target(self.actor.view(), current, movement)
        else {
            return false;
        };
        self.set_focus(Some(next));
        true
    }
    pub(super) fn style_disclosure(
        &self,
        binding: &crate::Binding,
        style: &mut Style,
        paint: &mut Paint,
    ) {
        let node = binding.node;
        if node.semantics.expanded != 0 && !semantics::is_anchored_trigger(node.semantics.role) {
            style.size.height = Dimension::auto();
            style.min_size.height = LengthPercentageAuto::length(48.0);
            style.padding = Rect {
                left: LengthPercentage::length(0.0),
                right: LengthPercentage::length(28.0),
                top: LengthPercentage::length(14.0),
                bottom: LengthPercentage::length(14.0),
            };
            style.border = Rect::length(0.0);
            paint.background = None;
            paint.border = None;
            paint.font_size = 14.0;
            paint.wrap_text = true;
            paint.text_color = if node.disabled == 0 {
                Color::from_rgb8(28, 35, 48)
            } else {
                Color::from_rgb8(152, 161, 175)
            };
        } else if node.semantics.role == semantics::DISCLOSURE_PANEL {
            style.padding.bottom = LengthPercentage::length(16.0);
        }
    }
    pub(super) fn paint_disclosure(&self, nodes: &mut [PlacedNode], tcx: &mut TextContext) {
        use shell::PaintScene;
        use vello_common::kurbo::{Affine, Line, Stroke};
        let keys: HashMap<_, _> = self.ids.iter().map(|(key, id)| (*id, *key)).collect();
        for placed in nodes {
            let Some(key) = keys.get(&placed.node_id) else {
                continue;
            };
            let node = self.actor.view().bindings[key].node;
            if semantics::is_anchored_trigger(node.semantics.role)
                || (node.semantics.expanded == 0
                    && node.semantics.role != semantics::ACCORDION_ITEM)
            {
                continue;
            }
            let mut paint = shell::widget::PaintContext::new(
                placed.content_size[0],
                placed.content_size[1],
                self.scale,
                tcx,
            );
            let color = if let Some(p) = self.palette(*key) {
                theme::color(p.text_muted)
            } else if node.disabled == 0 {
                Color::from_rgb8(100, 112, 132)
            } else {
                Color::from_rgb8(175, 183, 194)
            };
            if node.semantics.expanded != 0 && !semantics::is_anchored_trigger(node.semantics.role)
            {
                let x = f64::from(placed.content_size[0]) + 14.0;
                let y = f64::from(placed.content_size[1]) * 0.5;
                let progress = self
                    .motion
                    .value(blockui_motion::Key::new(
                        *key,
                        blockui_motion::Property::Disclosure,
                    ))
                    .unwrap_or(f64::from(node.semantics.expanded == 2));
                let angle = progress * std::f64::consts::FRAC_PI_2;
                let points = [(-2.0, -4.0), (2.0, 0.0), (-2.0, 4.0)].map(|(dx, dy)| {
                    (
                        x + dx * angle.cos() - dy * angle.sin(),
                        y + dx * angle.sin() + dy * angle.cos(),
                    )
                });
                for pair in points.windows(2) {
                    paint.scene_mut().stroke(
                        &Stroke::new(1.5),
                        Affine::IDENTITY,
                        color,
                        None,
                        &Line::new(pair[0], pair[1]),
                    );
                }
            } else {
                let y = f64::from(placed.content_size[1]) - 0.5;
                paint.scene_mut().stroke(
                    &Stroke::new(1.0),
                    Affine::IDENTITY,
                    self.palette(*key)
                        .map_or(Color::from_rgb8(226, 230, 236), |p| theme::color(p.border)),
                    None,
                    &Line::new((0.0, y), (f64::from(placed.content_size[0]), y)),
                );
            }
            placed.paint.widget = Some(Arc::new(paint.finish()));
        }
    }
}
