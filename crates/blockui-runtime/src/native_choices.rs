//! Host-owned choice navigation and visuals. No implementation callbacks retained.
use super::*;
impl<M: NativeModel> NativeSource<M> {
    pub(super) fn navigate_choice(&mut self, movement: &str) -> bool {
        let Some(current) = self.focus else {
            return false;
        };
        let Some(next) = crate::interaction::choice_target(self.actor.view(), current, movement)
        else {
            return false;
        };
        if let Err(error) = self.actor.dispatch_widget(next).and_then(|_| self.sync()) {
            self.diagnostic = error;
        } else {
            self.set_focus(Some(next));
        }
        true
    }
    pub(super) fn style_choice(
        &self,
        binding: &crate::Binding,
        style: &mut Style,
        paint: &mut Paint,
    ) {
        match binding.node.semantics.role {
            semantics::TAB_LIST => {
                paint.background = Some(Color::from_rgb8(238, 241, 246));
                paint.border_radii = CornerRadii::uniform(8.0);
            }
            semantics::TAB => {
                style.size.height = Dimension::length(36.0);
                style.padding = Rect::length(8.0);
                style.border = Rect::length(0.0);
                paint.font_size = 14.0;
                paint.background = (binding.node.semantics.selected != 0).then_some(Color::WHITE);
                paint.border = None;
                paint.border_radii = CornerRadii::uniform(6.0);
                paint.text_color = if binding.node.disabled != 0 {
                    Color::from_rgb8(152, 161, 175)
                } else if binding.node.semantics.selected != 0 {
                    Color::from_rgb8(28, 35, 48)
                } else {
                    Color::from_rgb8(100, 112, 132)
                };
            }
            semantics::SWITCH => {
                style.padding.left = LengthPercentage::length(54.0);
                paint.text = Some(Arc::from(
                    binding
                        .text
                        .strip_prefix("☑ ")
                        .or_else(|| binding.text.strip_prefix("☐ "))
                        .unwrap_or(&binding.text),
                ));
            }
            semantics::RADIO => {
                style.padding.left = LengthPercentage::length(30.0);
                paint.background = None;
                paint.border = None;
                paint.text_color = Color::from_rgb8(28, 35, 48);
                if binding.node.disabled != 0 {
                    paint.text_color = Color::from_rgb8(109, 119, 130);
                }
            }
            _ => {}
        }
    }
    pub(super) fn paint_choices(&self, nodes: &mut [PlacedNode], tcx: &mut TextContext) {
        use shell::PaintScene;
        use vello_common::{
            kurbo::{Affine, Circle, Rect, Stroke},
            peniko::Fill,
        };
        let keys: HashMap<_, _> = self.ids.iter().map(|(key, id)| (*id, *key)).collect();
        for placed in nodes {
            let Some(key) = keys.get(&placed.node_id) else {
                continue;
            };
            let node = self.actor.view().bindings[key].node;
            if !matches!(node.semantics.role, semantics::SWITCH | semantics::RADIO) {
                continue;
            }
            let mut paint = shell::widget::PaintContext::new(
                placed.content_size[0],
                placed.content_size[1],
                self.scale,
                tcx,
            );
            let y = f64::from(placed.content_size[1]) * 0.5;
            let palette = self.palette(*key);
            let accent = if let Some(p) = palette {
                theme::color(if node.disabled != 0 {
                    p.text_muted
                } else {
                    p.accent
                })
            } else if node.disabled != 0 {
                Color::from_rgb8(180, 186, 195)
            } else {
                Color::from_rgb8(31, 88, 155)
            };
            if node.semantics.role == semantics::SWITCH {
                let amount = self
                    .motion
                    .value(blockui_motion::Key::new(
                        *key,
                        blockui_motion::Property::Switch,
                    ))
                    .unwrap_or(f64::from(self.actor.widget_boolean(*key).unwrap_or(false)));
                paint.scene_mut().fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    palette
                        .map_or(Color::from_rgb8(210, 216, 225), |p| theme::color(p.border))
                        .lerp_rect(accent, amount as f32),
                    None,
                    &Rect::new(-48.0, y - 10.0, -12.0, y + 10.0).to_rounded_rect(10.0),
                );
                paint.scene_mut().fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    palette.map_or(Color::WHITE, |p| theme::color(p.on_accent)),
                    None,
                    &Circle::new((-38.0 + 16.0 * amount, y), 7.0),
                );
            } else {
                let circle = Circle::new((-17.0, y), 7.0);
                paint.scene_mut().stroke(
                    &Stroke::new(1.5),
                    Affine::IDENTITY,
                    accent,
                    None,
                    &circle,
                );
                if node.semantics.selected != 0 {
                    paint.scene_mut().fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        accent,
                        None,
                        &Circle::new((-17.0, y), 3.5),
                    );
                }
            }
            placed.paint.widget = Some(Arc::new(paint.finish()));
        }
    }
}
