use super::*;
use crate::interaction::tooltip::{Input, Trigger};
impl<M: NativeModel> NativeSource<M> {
    fn tooltip_time(&self) -> Duration {
        self.motion_manual_time
            .unwrap_or_else(|| self.motion_epoch.elapsed())
    }
    fn tooltip_eligible(&self, hint: u64) -> bool {
        self.window_focused
            && !self.selects.values().any(|s| s.is_open())
            && crate::interaction::tooltip::eligible(self.actor.view(), hint, self.modal)
    }

    fn tooltip_input(&self) -> Input {
        let trigger = |key: Option<u64>| {
            key.and_then(|key| {
                let node = self.actor.view().bindings.get(&key)?.node;
                let hint = node.semantics.described_by;
                if !self.tooltip_eligible(hint) {
                    return None;
                }
                Some(Trigger {
                    key: hint,
                    delay: Duration::from_millis(u64::from(
                        self.actor.view().bindings[&hint].node.tooltip_delay_ms,
                    )),
                })
            })
        };
        let hover = self.pointer_position.and_then(|p| self.hit(p));
        let content = self.tooltip.visible().filter(|key| {
            self.pointer_position.is_some_and(|p| {
                self.ids.get(key).is_some_and(|id| {
                    self.placed.iter().any(|n| {
                        n.node_id == *id
                            && p.x >= f64::from(n.rect[0])
                            && p.x <= f64::from(n.rect[2])
                            && p.y >= f64::from(n.rect[1])
                            && p.y <= f64::from(n.rect[3])
                    })
                })
            })
        });
        Input {
            hovered: trigger(hover),
            focused: trigger(self.focus),
            content,
        }
    }
    pub(super) fn reconcile_tooltip(&mut self) -> bool {
        let available = self.window_focused && !self.selects.values().any(|s| s.is_open());
        let view = self.actor.view();
        let modal = self.modal;
        let changed = self.tooltip.reconcile(|hint| {
            available && crate::interaction::tooltip::eligible(view, hint, modal)
        });
        let input = self.tooltip_input();
        self.tooltip.observe(input, self.tooltip_time()) || changed
    }
    pub(super) fn dismiss_tooltip(&mut self) {
        self.tooltip
            .dismiss(self.tooltip_input(), self.tooltip_time());
    }
    pub(super) fn dismiss_tooltip_at(&mut self, hit: Option<u64>) {
        let mut input = self.tooltip_input();
        // Touch has no hover position. Include the pressed target before focus
        // moves, so pointer-induced focus cannot reopen the activated hint.
        if let Some(hint) = hit
            .and_then(|key| self.actor.view().bindings.get(&key))
            .map(|binding| binding.node.semantics.described_by)
            .filter(|hint| self.tooltip_eligible(*hint))
        {
            input.hovered = Some(Trigger {
                key: hint,
                delay: Duration::ZERO,
            });
        }
        self.tooltip.dismiss(input, self.tooltip_time());
    }
    pub(super) fn append_tooltip(
        &mut self,
        nodes: &mut Vec<PlacedNode>,
        tcx: &mut TextContext,
        width: u32,
        height: u32,
    ) {
        let Some(key) = self.tooltip.visible() else {
            return;
        };
        // No drawable port remains inside the viewport margin.
        if width <= 32 || height <= 32 {
            return;
        }
        let binding = self.actor.view().bindings[&key].node;
        let Some(anchor) = nodes
            .iter()
            .find(|n| self.ids.get(&binding.tooltip_anchor) == Some(&n.node_id))
        else {
            return;
        };
        let anchor = anchor.rect;
        let id = self.ids[&key];
        let mut style = self.tree.style(id).unwrap().clone();
        style.size.width = Dimension::length(
            binding
                .presentation
                .width
                .min((width as f32 - 32.0).max(0.0)),
        );
        if self.tree.style(id).unwrap() != &style {
            self.tree.set_style(id, style).unwrap();
        }
        let mut hint = shell::layout::compute_and_walk_with_scroll(
            &mut self.tree,
            id,
            width as f32,
            height as f32,
            tcx,
            &HashMap::new(),
        );
        let Some(root) = hint.first() else {
            return;
        };
        let natural_height = root.rect[3] - root.rect[1];
        let viewport_height = (height as f32 - 32.0).max(0.0);
        // Prefer the room on either side of the trigger, so a tall hint does
        // not cover it when a usable vertical placement is available.
        let vertical_room = (anchor[1] - 24.0)
            .max(height as f32 - anchor[3] - 24.0)
            .min(viewport_height);
        let max_height = if binding.overlay_position / 3 < 2 && vertical_room >= 40.0 {
            vertical_room
        } else {
            viewport_height
        };
        let visible_height = natural_height.min(max_height);
        let range = (natural_height - visible_height).max(0.0);
        let state = self.actor.view_mut().renderer.get_mut(&key).unwrap();
        state.scroll = state.scroll.clamp(0.0, f64::from(range));
        let offset = state.scroll as f32;
        let origin = toast::popover_origin(
            anchor,
            [root.rect[2] - root.rect[0], visible_height],
            [width as f32, height as f32],
            binding.overlay_position,
        );
        let delta = [origin[0] - root.rect[0], origin[1] - root.rect[1]];
        for node in &mut hint {
            node.rect[0] += delta[0];
            node.rect[2] += delta[0];
            node.rect[1] += delta[1];
            node.rect[3] = node.rect[1] + visible_height;
            node.content_origin[0] += delta[0];
            node.content_origin[1] += delta[1] - offset;
            node.parent_node_id = None;
            node.depth = 0;
            node.clip = None;
            node.clip_depth = None;
            node.own_clip = Some(node.rect);
            node.own_clip_radii = node.paint.border_radii;
            node.own_clip_radius = node.paint.border_radii.max();
            node.scroll.port = node.rect;
            node.scroll.scrollable = [false, range > 0.0];
            node.scroll.range = [0.0, range];
            node.scroll.offset = [0.0, offset];
            node.scroll.opacity = if range > 0.0 { 1.0 } else { 0.0 };
        }
        nodes.extend(hint);
    }
}

impl<M: NativeModel> NativeSource<M> {
    pub(super) fn scroll_tooltip(&mut self, wheel: &shell::WheelEvent) -> bool {
        let Some(key) = self.tooltip.visible() else {
            return false;
        };
        let Some(node) = self.placed.iter().find(|node| {
            self.ids.get(&key) == Some(&node.node_id)
                && wheel.position.x >= f64::from(node.rect[0])
                && wheel.position.x <= f64::from(node.rect[2])
                && wheel.position.y >= f64::from(node.rect[1])
                && wheel.position.y <= f64::from(node.rect[3])
        }) else {
            return false;
        };
        // Consume wheel events over the overlay even at its bounds: scrolling
        // the page behind it could move the trigger away from its description.
        if wheel.delta_y.is_finite() {
            let state = self.actor.view_mut().renderer.get_mut(&key).unwrap();
            state.scroll =
                (state.scroll + wheel.delta_y).clamp(0.0, f64::from(node.scroll.range[1]));
        }
        true
    }
}
