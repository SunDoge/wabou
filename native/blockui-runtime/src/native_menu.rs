//! Stable-host command-menu focus and keyboard handling.
use super::*;
use crate::interaction::menu::{Item, Movement};
impl<M: NativeModel> NativeSource<M> {
    pub(super) fn context_trigger_for(&self, mut key: u64) -> Option<u64> {
        while let Some(binding) = self.actor.view().bindings.get(&key) {
            if semantics::is_anchored_panel(binding.node.semantics.role) {
                return None;
            }
            if semantics::is_context_trigger(binding.node.semantics.role) {
                return (binding.node.disabled == 0).then_some(key);
            }
            key = binding.node.parent;
        }
        None
    }
    pub(super) fn open_context_menu(&mut self, trigger: u64, point: Option<[f64; 2]>) {
        let panel = self.actor.view().bindings[&trigger].node.semantics.controls;
        if let Some(active) = self.active_popover() {
            let root = self.menu_root(active);
            if root != panel
                && let Err(error) = self.actor.dismiss_popover(root)
            {
                self.diagnostic = error;
                return;
            }
        }
        self.context_anchor = point.map(|point| (trigger, point));
        self.set_focus(Some(trigger));
        if let Err(error) = self.actor.dispatch_widget(trigger) {
            self.context_anchor = None;
            self.diagnostic = error;
            return;
        }
        let items = self.menu_items(panel);
        let target = self.menus.entry(panel).or_default().open(&items, false);
        self.set_focus(target.or(Some(panel)));
        self.pending_reveal = target;
    }
    pub(super) fn menubar_focus_owner(&self, mut key: u64) -> Option<(u64, u64)> {
        let view = self.actor.view();
        while let Some(binding) = view.bindings.get(&key) {
            let parent = binding.node.parent;
            if view
                .bindings
                .get(&parent)
                .is_some_and(|node| node.node.semantics.role == semantics::MENUBAR)
            {
                let trigger = view.order.iter().copied().find(|id| {
                    let node = view.bindings[id].node;
                    node.parent == key && node.semantics.role == semantics::MENU_TRIGGER
                })?;
                return Some((parent, trigger));
            }
            key = parent;
        }
        None
    }
    pub(super) fn poll_menu_hover(&mut self) {
        use crate::interaction::menu_hover::{Branch, Target};
        let Some(point) = self
            .pointer_position
            .filter(|_| self.window_focused && self.menu_pointer_active)
        else {
            self.menu_hover.reset();
            return;
        };
        if let Some(hovered) = self.hit(point) {
            let view = self.actor.view();
            let node = view.bindings[&hovered].node;
            let bar = view
                .bindings
                .get(&node.parent)
                .map(|parent| parent.node.parent);
            if node.semantics.role == semantics::MENU_TRIGGER
                && node.disabled == 0
                && let Some(bar) = bar.filter(|bar| {
                    view.bindings
                        .get(bar)
                        .is_some_and(|binding| binding.node.semantics.role == semantics::MENUBAR)
                })
            {
                use crate::interaction::menubar::{self, Input};
                let items: Vec<_> = view
                    .order
                    .iter()
                    .filter_map(|key| {
                        let node = view.bindings[key].node;
                        (node.semantics.role == semantics::MENU_TRIGGER
                            && view
                                .bindings
                                .get(&node.parent)
                                .is_some_and(|parent| parent.node.parent == bar))
                        .then_some(menubar::Item {
                            key: *key,
                            enabled: node.disabled == 0,
                        })
                    })
                    .collect();
                let open = items
                    .iter()
                    .find(|item| view.bindings[&item.key].node.semantics.expanded == 2)
                    .map(|item| item.key);
                if let Some(target) =
                    menubar::resolve(&items, hovered, open, Input::Hover(hovered)).open
                {
                    self.menu_hover.reset();
                    self.set_focus(Some(target));
                    if let Err(error) = self.actor.dispatch_widget(target) {
                        self.diagnostic = error;
                    }
                    return;
                }
            }
        }
        let target = self.hit(point).and_then(|key| {
            let node = self.actor.view().bindings.get(&key)?.node;
            (semantics::is_menu_item(node.semantics.role)
                && node.disabled == 0
                && self.popover_owner(key).is_some())
            .then_some(Target {
                key,
                opens_submenu: node.semantics.role == semantics::SUBMENU_TRIGGER
                    && node.semantics.expanded != 2,
            })
        });
        let branch = self.active_popover().and_then(|panel| {
            let node = self.actor.view().bindings[&panel].node;
            let trigger = node.semantics.labelled_by;
            if self.actor.view().bindings[&trigger].node.semantics.role
                != semantics::SUBMENU_TRIGGER
            {
                return None;
            }
            Some(Branch {
                trigger,
                anchor: self.widget_layout_rect(trigger)?.map(f64::from),
                panel: self.widget_layout_rect(panel)?.map(f64::from),
            })
        });
        let now = self
            .motion_manual_time
            .unwrap_or_else(|| self.motion_epoch.elapsed())
            .as_millis()
            .min(u128::from(u64::MAX)) as u64;
        let decision = self
            .menu_hover
            .observe([point.x, point.y], target, branch, now);
        if let Some(focus) = decision.focus {
            if let Some(branch) = branch
                && focus != branch.trigger
                && self.popover_owner(focus) == self.popover_owner(branch.trigger)
            {
                let panel = self.actor.view().bindings[&branch.trigger]
                    .node
                    .semantics
                    .controls;
                if let Err(error) = self.actor.dismiss_popover(panel) {
                    self.diagnostic = error;
                }
            }
            self.set_focus(Some(focus));
        }
        if let Some(open) = decision.open
            && self
                .actor
                .view()
                .bindings
                .get(&open)
                .is_some_and(|binding| {
                    binding.node.semantics.role == semantics::SUBMENU_TRIGGER
                        && binding.node.semantics.expanded != 2
                        && binding.node.disabled == 0
                })
            && let Err(error) = self.actor.dispatch_widget(open)
        {
            self.diagnostic = error;
        }
    }
    pub(super) fn menu_root(&self, mut panel: u64) -> u64 {
        while let Some(parent) = self.popover_owner(self.actor.view().bindings[&panel].node.parent)
        {
            if self.actor.view().bindings[&parent].node.semantics.role != semantics::MENU_PANEL {
                break;
            }
            panel = parent;
        }
        panel
    }
    pub(super) fn menu_items(&self, panel: u64) -> Vec<Item> {
        self.actor
            .view()
            .order
            .iter()
            .filter_map(|key| {
                let binding = &self.actor.view().bindings[key];
                let node = binding.node;
                (self.popover_owner(*key) == Some(panel)
                    && semantics::is_menu_item(node.semantics.role))
                .then(|| Item {
                    key: *key,
                    label: binding.text.clone(),
                    enabled: node.disabled == 0,
                })
            })
            .collect()
    }
    pub(super) fn reconcile_menus(&mut self) {
        let view = self.actor.view();
        if self.context_anchor.is_some_and(|(trigger, _)| {
            !view.bindings.get(&trigger).is_some_and(|binding| {
                semantics::is_context_trigger(binding.node.semantics.role)
                    && binding.node.semantics.expanded == 2
            })
        }) {
            self.context_anchor = None;
        }

        self.menubar_stops.retain(|bar, trigger| {
            view.bindings
                .get(bar)
                .is_some_and(|binding| binding.node.semantics.role == semantics::MENUBAR)
                && view.bindings.get(trigger).is_some_and(|binding| {
                    binding.node.semantics.role == semantics::MENU_TRIGGER
                        && binding.node.disabled == 0
                        && view
                            .bindings
                            .get(&binding.node.parent)
                            .is_some_and(|parent| parent.node.parent == *bar)
                })
        });
        self.menus.retain(|panel, _| {
            self.actor
                .view()
                .bindings
                .get(panel)
                .is_some_and(|binding| {
                    binding.node.semantics.role == semantics::MENU_PANEL
                        && binding.node.semantics.hidden == 0
                })
        });
        let owners: Vec<_> = self.menus.keys().copied().collect();
        for panel in owners {
            let items = self.menu_items(panel);
            let focused = self
                .focus
                .filter(|key| items.iter().any(|item| item.key == *key));
            let controller = self.menus.get_mut(&panel).unwrap();
            if let Some(key) = focused
                && controller.active() != Some(key)
            {
                controller.focus(&items, key);
            }
            let previous = controller.active();
            let active = controller.reconcile(&items);
            if previous != active && (previous.is_some() || self.focus == Some(panel)) {
                self.set_focus(active.or(Some(panel)));
                self.pending_reveal = active;
            }
        }
    }
    pub(super) fn navigate_menu(&mut self, key: &shell::KeyEvent) -> bool {
        let Some(focus) = self.focus else {
            return false;
        };
        let Some(binding) = self.actor.view().bindings.get(&focus) else {
            return false;
        };
        let node = binding.node;
        if node.disabled != 0 {
            return false;
        }
        if node.semantics.role == semantics::SUBMENU_TRIGGER && key.key == "ArrowRight" {
            if !key.repeat
                && node.semantics.expanded != 2
                && let Err(error) = self.actor.dispatch_widget(focus)
            {
                self.diagnostic = error;
            }
            if !key.repeat && node.semantics.expanded == 2 {
                let panel = node.semantics.controls;
                let items = self.menu_items(panel);
                let target = self.menus.entry(panel).or_default().open(&items, false);
                self.set_focus(target.or(Some(panel)));
                self.pending_reveal = target;
            }
            return true;
        }
        if key.key == "ArrowLeft"
            && let Some(panel) = self.popover_owner(focus)
        {
            let anchor = self.actor.view().bindings[&panel]
                .node
                .semantics
                .labelled_by;
            if self.actor.view().bindings[&anchor].node.semantics.role == semantics::SUBMENU_TRIGGER
            {
                if !key.repeat
                    && let Err(error) = self.actor.dismiss_popover(panel)
                {
                    self.diagnostic = error;
                }
                return true;
            }
        }
        let root_trigger = if node.semantics.role == semantics::MENU_TRIGGER {
            Some(focus)
        } else {
            self.popover_owner(focus).map(|panel| {
                self.actor.view().bindings[&self.menu_root(panel)]
                    .node
                    .semantics
                    .labelled_by
            })
        };
        if let Some(root_trigger) = root_trigger {
            let view = self.actor.view();
            let root = view.bindings[&root_trigger].node;
            let bar = view
                .bindings
                .get(&root.parent)
                .map(|binding| binding.node.parent);
            if let Some(bar) = bar.filter(|bar| {
                view.bindings
                    .get(bar)
                    .is_some_and(|binding| binding.node.semantics.role == semantics::MENUBAR)
            }) {
                use crate::interaction::menubar::{self, Input};
                let input = match key.key.as_str() {
                    "ArrowLeft" => Some(Input::Previous),
                    "ArrowRight" => Some(Input::Next),
                    "Home" if focus == root_trigger => Some(Input::First),
                    "End" if focus == root_trigger => Some(Input::Last),
                    _ => None,
                };
                if let Some(input) = input {
                    if key.repeat {
                        return true;
                    }
                    let items: Vec<_> = view
                        .order
                        .iter()
                        .filter_map(|id| {
                            let node = view.bindings[id].node;
                            (node.semantics.role == semantics::MENU_TRIGGER
                                && view
                                    .bindings
                                    .get(&node.parent)
                                    .is_some_and(|parent| parent.node.parent == bar))
                            .then_some(menubar::Item {
                                key: *id,
                                enabled: node.disabled == 0,
                            })
                        })
                        .collect();
                    let open = items
                        .iter()
                        .find(|item| view.bindings[&item.key].node.semantics.expanded == 2)
                        .map(|item| item.key);
                    let decision = menubar::resolve(&items, root_trigger, open, input);
                    if let Some(target) = decision.focus {
                        self.set_focus(Some(target));
                    }
                    if let Some(target) = decision.open {
                        let node = self.actor.view().bindings[&target].node;
                        if node.semantics.expanded != 2 {
                            if let Err(error) = self.actor.dispatch_widget(target) {
                                self.diagnostic = error;
                            }
                        } else {
                            let panel = node.semantics.controls;
                            let items = self.menu_items(panel);
                            let focus = self.menus.entry(panel).or_default().open(&items, false);
                            self.set_focus(focus.or(Some(panel)));
                            self.pending_reveal = focus;
                        }
                    }
                    return true;
                }
            }
        }
        if matches!(
            node.semantics.role,
            semantics::MENU_TRIGGER | semantics::CONTEXT_TRIGGER | semantics::CONTEXT_SURFACE
        ) && matches!(key.key.as_str(), "ArrowUp" | "ArrowDown")
        {
            if key.repeat {
                return true;
            }
            let panel = node.semantics.controls;
            if node.semantics.expanded != 2 {
                self.menu_open_last = (key.key == "ArrowUp").then_some(panel);
                if let Err(error) = self.actor.dispatch_widget(focus) {
                    self.menu_open_last = None;
                    self.diagnostic = error;
                }
            } else {
                let items = self.menu_items(panel);
                let target = self
                    .menus
                    .entry(panel)
                    .or_default()
                    .open(&items, key.key == "ArrowUp");
                self.set_focus(target.or(Some(panel)));
                self.pending_reveal = target;
            }
            return true;
        }
        let Some(panel) = self.popover_owner(focus).filter(|panel| {
            self.actor.view().bindings[panel].node.semantics.role == semantics::MENU_PANEL
        }) else {
            return false;
        };
        let movement = match key.key.as_str() {
            "ArrowDown" => Some(Movement::Next),
            "ArrowUp" => Some(Movement::Previous),
            "Home" => Some(Movement::First),
            "End" => Some(Movement::Last),
            _ => None,
        };
        let text = key.text.as_deref().unwrap_or(&key.key);
        let printable = !matches!(key.key.as_str(), "Enter" | " " | "Space")
            && !text.is_empty()
            && !text.chars().any(char::is_control)
            && (key.text.is_some() || text.chars().count() == 1);
        if movement.is_none() && !printable {
            return false;
        }
        let items = self.menu_items(panel);
        let now = self
            .motion_manual_time
            .unwrap_or_else(|| self.motion_epoch.elapsed())
            .as_millis()
            .min(u128::from(u64::MAX)) as u64;
        let controller = self.menus.entry(panel).or_default();
        if controller.active() != Some(focus) {
            controller.focus(&items, focus);
        }
        let target = if let Some(movement) = movement {
            controller.navigate(&items, movement)
        } else {
            controller.typeahead(&items, text, now)
        };
        self.set_focus(target.or(Some(panel)));
        self.pending_reveal = target;
        true
    }
}
