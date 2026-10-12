//! Pointer-free view composition with host-owned actor/widget identities.
use crate::{ActorId, Runtime, View};
use std::collections::BTreeMap;

/// A snapshot of one actor family. Child roots are mounted at their parent's root.
/// Event identities are scoped by actor; equal local widget keys cannot collide.
pub struct ComposedView {
    pub view: View,
    owners: BTreeMap<u64, (ActorId, u64)>,
}
impl ComposedView {
    #[cfg(feature = "native-window")]
    pub(crate) fn persist_renderer(&self, runtime: &mut Runtime) {
        for (widget, (owner, local)) in &self.owners {
            let Some(actor) = runtime.actor_mut(*owner) else {
                continue;
            };
            if actor.view.renderer_retention == 0 {
                continue;
            }
            if actor
                .view
                .bindings
                .get(local)
                .is_some_and(|binding| binding.node.kind == self.view.bindings[widget].node.kind)
                && let Some(state) = self.view.renderer.get(widget)
            {
                actor.view.renderer.insert(*local, state.clone());
            }
        }
        let owners: std::collections::BTreeSet<_> =
            self.owners.values().map(|(owner, _)| *owner).collect();
        for owner in owners {
            if let Some(actor) = runtime.actor_mut(owner) {
                actor.view.invalidate_cached_focus();
            }
        }
    }
    pub fn widget(&self, actor: ActorId, local: u64) -> Option<u64> {
        self.owners
            .iter()
            .find_map(|(key, owner)| (*owner == (actor, local)).then_some(*key))
    }
    /// Mounts a child's root at a semantic Column or Row key in its owning parent's view.
    /// Validation precedes the descriptor update; identities and actor storage stay intact.
    pub fn mount_child(
        &mut self,
        runtime: &Runtime,
        child: ActorId,
        column: u64,
    ) -> Result<(), String> {
        let parent = runtime.parent(child).ok_or("actor is not an owned child")?;
        let target = self
            .widget(parent, column)
            .ok_or("mount target is outside this view")?;
        self.resolve(runtime, target)?;
        let target_node = runtime
            .actor(parent)
            .unwrap()
            .view
            .bindings
            .get(&column)
            .ok_or("removed mount target")?
            .node;
        if !matches!(target_node.kind, blockui_abi::COLUMN | blockui_abi::ROW)
            || !matches!(
                self.view.bindings[&target].node.kind,
                blockui_abi::COLUMN | blockui_abi::ROW
            )
        {
            return Err("child mount target must be a Column or Row".into());
        }
        let local_root = runtime
            .actor(child)
            .ok_or("disposed child")?
            .view
            .order
            .iter()
            .find(|key| runtime.actor(child).unwrap().view.bindings[key].node.parent == 0)
            .copied()
            .ok_or("child has no root")?;
        let root = self
            .widget(child, local_root)
            .ok_or("child is outside this view")?;
        self.resolve(runtime, root)?;
        self.view.bindings.get_mut(&root).unwrap().node.parent = target;
        Ok(())
    }
    fn resolve(&self, runtime: &Runtime, widget: u64) -> Result<(ActorId, u64), String> {
        let owner = *self.owners.get(&widget).ok_or("unknown composed widget")?;
        if runtime.widget_keys.get(&owner) != Some(&widget) {
            return Err("disposed widget identity".into());
        }
        Ok(owner)
    }
    #[cfg(feature = "native-window")]
    pub(crate) fn boolean_state(&self, runtime: &Runtime, widget: u64) -> Option<bool> {
        let (owner, local) = self.resolve(runtime, widget).ok()?;
        let actor = runtime.actor(owner)?;
        actor.checkbox_value(local)
    }
    #[cfg(feature = "native-window")]
    pub(crate) fn integer_state(&self, runtime: &Runtime, widget: u64) -> Option<i64> {
        let (owner, local) = self.resolve(runtime, widget).ok()?;
        let actor = runtime.actor(owner)?;
        actor.integer_control_value(local)
    }
    #[cfg(feature = "native-window")]
    pub(crate) fn set_slider(
        &self,
        runtime: &mut Runtime,
        widget: u64,
        value: i64,
    ) -> Result<(), String> {
        let (owner, local) = self.resolve(runtime, widget)?;
        runtime
            .actor_mut(owner)
            .ok_or("disposed actor")?
            .edit_slider(local, value)
    }
    #[cfg(feature = "native-window")]
    pub(crate) fn set_select(
        &self,
        runtime: &mut Runtime,
        widget: u64,
        value: i64,
    ) -> Result<(), String> {
        let (owner, local) = self.resolve(runtime, widget)?;
        runtime
            .actor_mut(owner)
            .ok_or("disposed actor")?
            .edit_select(local, value)
    }
    #[cfg(feature = "native-window")]
    pub(crate) fn set_canvas(
        &self,
        runtime: &mut Runtime,
        widget: u64,
        point: blockui_abi::LocalPoint,
        secondary: bool,
    ) -> Result<(), String> {
        let (owner, local) = self.resolve(runtime, widget)?;
        runtime
            .actor_mut(owner)
            .ok_or("disposed actor")?
            .edit_canvas(local, point, secondary)
    }
    #[cfg(feature = "native-window")]
    pub(crate) fn observe_scroll(
        &self,
        runtime: &mut Runtime,
        widget: u64,
        point: blockui_abi::LocalPoint,
    ) -> Result<(), String> {
        let (owner, local) = self.resolve(runtime, widget)?;
        runtime
            .actor_mut(owner)
            .ok_or("disposed actor")?
            .observe_scroll(local, point)
    }
    #[cfg(feature = "native-window")]
    pub(crate) fn observe_viewport(
        &self,
        runtime: &mut Runtime,
        widget: u64,
        size: blockui_abi::LocalPoint,
    ) -> Result<(), String> {
        let (owner, local) = self.resolve(runtime, widget)?;
        runtime
            .actor_mut(owner)
            .ok_or("disposed actor")?
            .observe_viewport(local, size)
    }
    #[cfg(feature = "native-window")]
    pub(crate) fn navigate_grid(
        &self,
        runtime: &mut Runtime,
        widget: u64,
        movement: i64,
    ) -> Result<(), String> {
        let (owner, local) = self.resolve(runtime, widget)?;
        runtime
            .actor_mut(owner)
            .ok_or("disposed actor")?
            .navigate_grid(local, movement)
    }
    #[cfg(feature = "native-window")]
    pub(crate) fn editor_command(
        &self,
        runtime: &mut Runtime,
        widget: u64,
        cancel: bool,
    ) -> Result<(), String> {
        let (owner, local) = self.resolve(runtime, widget)?;
        runtime
            .actor_mut(owner)
            .ok_or("disposed actor")?
            .editor_command(local, cancel)
    }
    #[cfg(feature = "native-window")]
    pub(crate) fn dismiss_dialog(&self, runtime: &mut Runtime, widget: u64) -> Result<(), String> {
        let (owner, local) = self.resolve(runtime, widget)?;
        runtime
            .actor_mut(owner)
            .ok_or("disposed actor")?
            .dismiss_dialog(local)
    }
    #[cfg(feature = "native-window")]
    pub(crate) fn dismiss_popover(&self, runtime: &mut Runtime, widget: u64) -> Result<(), String> {
        let (owner, local) = self.resolve(runtime, widget)?;
        runtime
            .actor_mut(owner)
            .ok_or("disposed actor")?
            .dismiss_popover(local)
    }
    #[cfg(feature = "native-window")]
    pub(crate) fn observe_toast_activity(
        &self,
        runtime: &mut Runtime,
        widget: u64,
        paused: bool,
    ) -> Result<(), String> {
        let (owner, local) = self.resolve(runtime, widget)?;
        runtime
            .actor_mut(owner)
            .ok_or("disposed actor")?
            .observe_toast_activity(local, paused)
    }
    pub fn dispatch(&self, runtime: &mut Runtime, widget: u64) -> Result<(), String> {
        let (actor, local) = self.resolve(runtime, widget)?;
        let actor = runtime.actor_mut(actor).ok_or("disposed actor")?;
        actor.dispatch_widget(local)
    }
    pub fn set_text(&self, runtime: &mut Runtime, widget: u64, value: &str) -> Result<(), String> {
        let (actor, local) = self.resolve(runtime, widget)?;
        runtime
            .actor_mut(actor)
            .ok_or("disposed actor")?
            .edit_text(local, value)
    }
}
impl Runtime {
    pub fn compose(&mut self, root: ActorId) -> Result<ComposedView, String> {
        self.apply_children();
        if self.actor(root).is_none() {
            return Err("unknown view actor".into());
        }
        let mut family = vec![root];
        let mut cursor = 0;
        while cursor < family.len() {
            let parent = family[cursor];
            family.extend(
                self.parents
                    .iter()
                    .filter_map(|(child, owner)| (*owner == parent).then_some(*child)),
            );
            cursor += 1;
        }
        let count: usize = family
            .iter()
            .map(|id| self.actor(*id).unwrap().view.order.len())
            .sum();
        if count > 16_384 {
            return Err("composed view capacity reached".into());
        }
        let mut keys = self.widget_keys.clone();
        // Prune disappeared local nodes without disturbing other live families.
        keys.retain(|(owner, local), _| {
            self.actor(*owner)
                .is_some_and(|actor| actor.view.bindings.contains_key(local))
        });
        let mut next = self.next_widget;
        for id in &family {
            for local in &self.actor(*id).unwrap().view.order {
                if let std::collections::btree_map::Entry::Vacant(entry) = keys.entry((*id, *local))
                {
                    let key = next;
                    next = next.checked_add(1).ok_or("widget identity exhausted")?;
                    entry.insert(key);
                }
            }
        }
        let roots: BTreeMap<_, _> = family
            .iter()
            .map(|id| {
                let actor = self.actor(*id).unwrap();
                let local = actor
                    .view
                    .order
                    .iter()
                    .find(|key| actor.view.bindings[key].node.parent == 0)
                    .copied()
                    .ok_or("actor has no view root")?;
                Ok((*id, keys[&(*id, local)]))
            })
            .collect::<Result<_, String>>()?;
        let mut view = View::default();
        let mut owners = BTreeMap::new();
        for id in family {
            let actor = self.actor(id).unwrap();
            for local in &actor.view.renderer_boundaries {
                if let Some(key) = keys.get(&(id, *local)) {
                    view.renderer_boundaries.insert(*key);
                }
            }
            for local in &actor.view.order {
                let key = keys[&(id, *local)];
                let mut binding = actor.view.bindings[local].clone();
                binding.node.key = key;
                for related in [
                    &mut binding.node.semantics.controls,
                    &mut binding.node.semantics.labelled_by,
                    &mut binding.node.semantics.described_by,
                    &mut binding.node.tooltip_anchor,
                ] {
                    if *related != 0 {
                        *related = keys[&(id, *related)];
                    }
                }
                binding.node.parent = if binding.node.parent != 0 {
                    keys[&(id, binding.node.parent)]
                } else if id != root {
                    let parent = self.parents[&id];
                    let slot = self.children.as_ref().and_then(|registry| {
                        registry
                            .borrow()
                            .slots
                            .iter()
                            .find_map(|((owner, slot), child)| {
                                (*owner == parent && child.id == id).then_some(*slot)
                            })
                    });
                    if let Some(slot) = slot {
                        keys[&(parent, slot)]
                    } else {
                        roots[&parent]
                    }
                } else {
                    0
                };
                view.order.push(key);
                view.bindings.insert(key, binding);
                view.renderer
                    .insert(key, actor.view.renderer[local].clone());
                owners.insert(key, (id, *local));
            }
        }
        self.widget_keys = keys;
        self.next_widget = next;
        Ok(ComposedView { view, owners })
    }
}
