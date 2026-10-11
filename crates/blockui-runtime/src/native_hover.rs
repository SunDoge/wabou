//! Host-owned hover paint and aggregate notification activity observations.
use super::*;
impl<M: NativeModel> NativeSource<M> {
    pub(super) fn refresh_hover(&mut self) -> bool {
        let hit = self.pointer_position.and_then(|point| self.hit(point));
        self.set_hover_from_hit(hit)
    }
    pub(super) fn set_hover_from_hit(&mut self, hit: Option<u64>) -> bool {
        let toast_changed = self.reconcile_toast_activity();
        let tooltip_changed = self.reconcile_tooltip();
        let next = hit
            .filter(|_| {
                self.pointer_position.is_some()
                    && self.window_focused
                    && !self.selects.values().any(|control| control.is_open())
            })
            .filter(|key| {
                self.actor.view().bindings.get(key).is_some_and(|binding| {
                    binding.node.disabled == 0
                        && matches!(
                            binding.node.kind,
                            BUTTON | CHECKBOX | SELECT | ENTRY | TEXTAREA | SLIDER
                        )
                })
            });
        if self.hovered == next {
            return toast_changed || tooltip_changed;
        }
        for key in self.hovered.into_iter().chain(next) {
            self.paint_dirty.insert(key);
        }
        self.hovered = next;
        true
    }
    pub(super) fn apply_hover(&self, key: u64, grid_cell: bool, paint: &mut Paint) {
        let node = &self.actor.view().bindings[&key].node;
        if self.hovered != Some(key) || !self.window_focused || node.disabled != 0 {
            return;
        }
        self.apply_hover_style(key, grid_cell, paint);
    }
    pub(super) fn apply_hover_style(&self, key: u64, grid_cell: bool, paint: &mut Paint) {
        let node = &self.actor.view().bindings[&key].node;
        if node.disabled != 0 {
            return;
        }
        if self.theme_interaction(key, paint, false) {
            return;
        }
        if node.semantics.expanded != 0 {
            paint.background = Some(Color::from_rgb8(246, 247, 249));
            return;
        }
        if node.semantics.role == semantics::TAB {
            paint.background = Some(if node.semantics.selected != 0 {
                Color::WHITE
            } else {
                Color::from_rgb8(232, 238, 246)
            });
            return;
        }
        if node.semantics.role == semantics::RADIO {
            paint.background = Some(Color::from_rgb8(238, 241, 246));
            return;
        }
        match node.kind {
            BUTTON => {
                paint.background = Some(if grid_cell {
                    Color::from_rgb8(241, 245, 251)
                } else {
                    match node.button_variant {
                        1 => Color::from_rgb8(232, 238, 246),
                        2 => Color::from_rgb8(255, 236, 236),
                        _ => Color::from_rgb8(27, 78, 140),
                    }
                });
            }
            CHECKBOX => paint.background = Some(Color::from_rgb8(241, 245, 251)),
            SELECT | ENTRY | TEXTAREA => {
                paint.border = Some((1.0, Color::from_rgb8(116, 134, 157)));
            }
            SLIDER => {
                paint.outline_width = 1.0;
                paint.outline_offset = 2.0;
                paint.outline_color = Some(Color::from_rgb8(170, 183, 205));
                paint.border_radii = CornerRadii::uniform(6.0);
            }
            _ => {}
        }
    }
}
