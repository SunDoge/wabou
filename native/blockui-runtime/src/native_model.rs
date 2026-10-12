//! Native rendering models share the same retained Taffy/widget implementation.
use crate::{Actor, ActorId, ComposedView, Message, Runtime, View};
use std::{collections::BTreeMap, path::Path, sync::Arc};

pub trait NativeModel {
    /// Snapshot opt-in renderer state before a turn can remove its projection.
    fn persist_renderer(&mut self) {}
    /// Resolve a root-local authored key to the mounted native identity.
    fn root_widget(&self, local: u64) -> Option<u64> {
        self.view().bindings.contains_key(&local).then_some(local)
    }
    fn root_actor(&self) -> &Actor;
    fn root_actor_mut(&mut self) -> &mut Actor;
    fn lifecycle(&self, id: ActorId) -> Option<u64> {
        (self.root_actor().id == id.0).then(|| self.root_actor().lifecycle())
    }
    fn reload_target(&mut self, id: ActorId, path: &Path) -> Result<(), String> {
        if self.lifecycle(id).is_none() {
            return Err("unknown reload target".into());
        }
        self.reload(path)
    }
    fn reload_targets(&mut self, ids: &[ActorId], path: &Path) -> Result<(), String> {
        if ids.len() != 1 {
            return Err("model does not support grouped replacement".into());
        }
        self.reload_target(ids[0], path)
    }
    fn declared_children(&self) -> Vec<(ActorId, u64)> {
        Vec::new()
    }
    fn view(&self) -> &View;
    fn view_mut(&mut self) -> &mut View;
    fn reload(&mut self, path: &Path) -> Result<(), String> {
        self.root_actor_mut().reload(path)
    }
    /// Earliest runtime service deadline, independent of editor animations.
    fn async_deadline(&self) -> Option<std::time::Instant> {
        None
    }
    fn tick(&mut self) -> Result<(), String> {
        self.root_actor_mut().tick()
    }
    fn set_wake(&mut self, wake: Arc<dyn Fn() + Send + Sync>) {
        self.root_actor_mut().set_wake(wake);
    }
    fn widget_boolean(&self, key: u64) -> Option<bool> {
        self.root_actor().checkbox_value(key)
    }
    fn widget_integer(&self, key: u64) -> Option<i64> {
        self.root_actor().integer_control_value(key)
    }
    fn observe_toast_activity(&mut self, key: u64, paused: bool) -> Result<(), String> {
        self.root_actor_mut().observe_toast_activity(key, paused)
    }
    fn observe_scroll(&mut self, key: u64, point: blockui_abi::LocalPoint) -> Result<(), String> {
        self.root_actor_mut().observe_scroll(key, point)
    }
    fn observe_viewport(&mut self, key: u64, size: blockui_abi::LocalPoint) -> Result<(), String> {
        self.root_actor_mut().observe_viewport(key, size)
    }
    fn navigate_grid(&mut self, key: u64, movement: i64) -> Result<(), String> {
        self.root_actor_mut().navigate_grid(key, movement)
    }
    fn editor_command(&mut self, key: u64, cancel: bool) -> Result<(), String> {
        self.root_actor_mut().editor_command(key, cancel)
    }
    fn dismiss_dialog(&mut self, key: u64) -> Result<(), String> {
        self.root_actor_mut().dismiss_dialog(key)
    }
    fn dismiss_popover(&mut self, key: u64) -> Result<(), String> {
        self.root_actor_mut().dismiss_popover(key)
    }
    fn set_widget_canvas(
        &mut self,
        key: u64,
        point: blockui_abi::LocalPoint,
        secondary: bool,
    ) -> Result<(), String> {
        self.root_actor_mut().edit_canvas(key, point, secondary)
    }
    fn set_widget_select(&mut self, key: u64, value: i64) -> Result<(), String> {
        self.root_actor_mut().edit_select(key, value)
    }
    fn set_widget_slider(&mut self, key: u64, value: i64) -> Result<(), String> {
        self.root_actor_mut().edit_slider(key, value)
    }
    fn dispatch_widget(&mut self, key: u64) -> Result<(), String> {
        self.root_actor_mut().dispatch_widget(key)
    }
    fn set_widget_text(&mut self, key: u64, value: &str) -> Result<(), String> {
        self.root_actor_mut().edit_text(key, value)
    }
}
impl NativeModel for Actor {
    fn persist_renderer(&mut self) {
        self.view.invalidate_cached_focus();
    }
    fn root_actor(&self) -> &Actor {
        self
    }
    fn root_actor_mut(&mut self) -> &mut Actor {
        self
    }
    fn view(&self) -> &View {
        &self.view
    }
    fn view_mut(&mut self) -> &mut View {
        &mut self.view
    }
}

/// Runtime-owned actor family projected into one native tree. No actor is copied
/// or transferred into the renderer; only view descriptors and strings are copied.
pub struct NativeFamily {
    runtime: Runtime,
    root: ActorId,
    projection: ComposedView,
    mounts: BTreeMap<ActorId, (u64, u64)>,
}
impl NativeFamily {
    pub fn new(mut runtime: Runtime, root: ActorId) -> Result<Self, String> {
        let projection = runtime.compose(root)?;
        Ok(Self {
            runtime,
            root,
            projection,
            mounts: BTreeMap::new(),
        })
    }
    fn notify(&mut self) {
        crate::wake::notify(&mut self.root_actor_mut().wake);
    }
    /// Queue a one-way host message; the current implementation handles it on tick.
    /// Admission never lends actor state to the caller or patches the view synchronously.
    pub fn tell(&mut self, target: ActorId, message: Message) -> Result<(), String> {
        self.runtime.tell(target, message)
    }
    /// Cancel a host reply wait; already executed effects cannot be reversed.
    pub fn cancel_request(&mut self, id: crate::request::RequestId) -> bool {
        let cancelled = self.runtime.cancel_request(id);
        if cancelled {
            self.notify();
        }
        cancelled
    }
    /// Queue a host request without exposing a mutable runtime/state borrow.
    pub fn ask(
        &mut self,
        target: ActorId,
        message: Message,
        timeout: std::time::Duration,
    ) -> Result<crate::request::Reply, String> {
        let reply = self.runtime.ask(target, message, timeout)?;
        self.notify();
        Ok(reply)
    }
    /// Submit a stable-host future without exposing mutable actor/runtime state.
    pub fn spawn_async<F>(
        &mut self,
        target: ActorId,
        id: u64,
        future: F,
        on_panic: Message,
        policy: crate::AsyncReloadPolicy,
    ) -> Result<(), String>
    where
        F: std::future::Future<Output = Message> + Send + 'static,
    {
        self.runtime
            .spawn_async(target, id, future, on_panic, policy)
    }
    pub fn cancel_task(&mut self, target: ActorId, id: u64) -> Result<bool, String> {
        self.runtime.cancel_task(target, id)
    }
    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }
    pub fn widget(&self, actor: ActorId, local: u64) -> Option<u64> {
        self.projection.widget(actor, local)
    }
    /// Moves a child view without recreating its actor or native widgets.
    /// A disappearing target falls back to the parent's root until it returns.
    pub fn mount_child(&mut self, child: ActorId, column: u64) -> Result<(), String> {
        self.projection.mount_child(&self.runtime, child, column)?;
        let local_root = self
            .runtime
            .actor(child)
            .unwrap()
            .view
            .order
            .iter()
            .find(|key| {
                self.runtime.actor(child).unwrap().view.bindings[key]
                    .node
                    .parent
                    == 0
            })
            .copied()
            .unwrap();
        let identity = self.projection.widget(child, local_root).unwrap();
        self.mounts.insert(child, (identity, column));
        self.notify();
        Ok(())
    }
    fn refresh(&mut self) -> Result<(), String> {
        let mut next = self.runtime.compose(self.root)?;
        self.mounts.retain(|child, (identity, column)| {
            let Some(actor) = self.runtime.actor(*child) else {
                return false;
            };
            let Some(local_root) = actor
                .view
                .order
                .iter()
                .find(|key| actor.view.bindings[key].node.parent == 0)
            else {
                return false;
            };
            if next.widget(*child, *local_root) != Some(*identity) {
                return false;
            }
            // Keep the requested semantic location during temporary structural edits.
            // Invalid/absent targets use the default parent root already in `next`.
            let _ = next.mount_child(&self.runtime, *child, *column);
            true
        });
        for (key, state) in &mut next.view.renderer {
            if self
                .projection
                .view
                .bindings
                .get(key)
                .is_some_and(|old| old.node.kind == next.view.bindings[key].node.kind)
                && let Some(previous) = self.projection.view.renderer.get(key)
            {
                *state = previous.clone();
            }
        }
        self.projection = next;
        Ok(())
    }
    pub fn reload_actor(&mut self, id: ActorId, path: &Path) -> Result<(), String> {
        self.runtime.reload(id, path)?;
        self.refresh()?;
        self.notify();
        Ok(())
    }
    pub fn reconcile_children(
        &mut self,
        parent: ActorId,
        desired: &[(ActorId, &Path)],
    ) -> Result<(), String> {
        self.runtime.reconcile_children(parent, desired)?;
        self.refresh()?;
        self.notify();
        Ok(())
    }
}
impl NativeModel for NativeFamily {
    fn persist_renderer(&mut self) {
        self.projection.persist_renderer(&mut self.runtime);
    }
    fn root_widget(&self, local: u64) -> Option<u64> {
        self.projection.widget(self.root, local)
    }
    fn declared_children(&self) -> Vec<(ActorId, u64)> {
        self.runtime
            .declared_children()
            .into_iter()
            .filter(|(id, _)| {
                let mut current = *id;
                while let Some(parent) = self.runtime.parent(current) {
                    if parent == self.root {
                        return true;
                    }
                    current = parent;
                }
                false
            })
            .collect()
    }
    fn root_actor(&self) -> &Actor {
        self.runtime.actor(self.root).unwrap()
    }
    fn root_actor_mut(&mut self) -> &mut Actor {
        self.runtime.actor_mut(self.root).unwrap()
    }
    fn lifecycle(&self, id: ActorId) -> Option<u64> {
        self.runtime.actor(id).map(Actor::lifecycle)
    }
    fn reload_target(&mut self, id: ActorId, path: &Path) -> Result<(), String> {
        self.reload_actor(id, path)
    }
    fn reload_targets(&mut self, ids: &[ActorId], path: &Path) -> Result<(), String> {
        self.runtime.reload_group(ids, path)?;
        self.refresh()?;
        self.notify();
        Ok(())
    }

    fn view(&self) -> &View {
        &self.projection.view
    }
    fn view_mut(&mut self) -> &mut View {
        &mut self.projection.view
    }
    fn reload(&mut self, path: &Path) -> Result<(), String> {
        self.reload_actor(self.root, path)
    }
    fn async_deadline(&self) -> Option<std::time::Instant> {
        self.runtime.next_request_deadline()
    }
    fn tick(&mut self) -> Result<(), String> {
        let faults = self.runtime.tick();
        self.refresh()?;
        if faults.is_empty() {
            Ok(())
        } else {
            Err(faults
                .into_iter()
                .map(|f| format!("actor {}: {}", f.actor.0, f.error))
                .collect::<Vec<_>>()
                .join("; "))
        }
    }
    fn set_wake(&mut self, wake: Arc<dyn Fn() + Send + Sync>) {
        self.runtime.set_wake(wake);
    }
    fn widget_boolean(&self, key: u64) -> Option<bool> {
        self.projection.boolean_state(&self.runtime, key)
    }
    fn widget_integer(&self, key: u64) -> Option<i64> {
        self.projection.integer_state(&self.runtime, key)
    }
    fn observe_scroll(&mut self, key: u64, point: blockui_abi::LocalPoint) -> Result<(), String> {
        self.projection
            .observe_scroll(&mut self.runtime, key, point)?;
        self.refresh()
    }
    fn observe_viewport(&mut self, key: u64, size: blockui_abi::LocalPoint) -> Result<(), String> {
        self.projection
            .observe_viewport(&mut self.runtime, key, size)?;
        self.refresh()
    }
    fn navigate_grid(&mut self, key: u64, movement: i64) -> Result<(), String> {
        self.projection
            .navigate_grid(&mut self.runtime, key, movement)?;
        self.refresh()
    }
    fn editor_command(&mut self, key: u64, cancel: bool) -> Result<(), String> {
        self.projection
            .editor_command(&mut self.runtime, key, cancel)?;
        self.refresh()
    }
    fn dismiss_dialog(&mut self, key: u64) -> Result<(), String> {
        self.projection.dismiss_dialog(&mut self.runtime, key)?;
        self.refresh()
    }
    fn dismiss_popover(&mut self, key: u64) -> Result<(), String> {
        self.projection.dismiss_popover(&mut self.runtime, key)?;
        self.refresh()
    }
    fn set_widget_canvas(
        &mut self,
        key: u64,
        point: blockui_abi::LocalPoint,
        secondary: bool,
    ) -> Result<(), String> {
        self.projection
            .set_canvas(&mut self.runtime, key, point, secondary)?;
        self.refresh()
    }
    fn set_widget_select(&mut self, key: u64, value: i64) -> Result<(), String> {
        self.projection.set_select(&mut self.runtime, key, value)?;
        self.refresh()
    }
    fn set_widget_slider(&mut self, key: u64, value: i64) -> Result<(), String> {
        self.projection.set_slider(&mut self.runtime, key, value)?;
        self.refresh()
    }
    fn observe_toast_activity(&mut self, key: u64, paused: bool) -> Result<(), String> {
        self.projection
            .observe_toast_activity(&mut self.runtime, key, paused)?;
        self.refresh()
    }
    fn dispatch_widget(&mut self, key: u64) -> Result<(), String> {
        self.projection.dispatch(&mut self.runtime, key)?;
        self.refresh()
    }
    fn set_widget_text(&mut self, key: u64, value: &str) -> Result<(), String> {
        self.projection.set_text(&mut self.runtime, key, value)?;
        self.refresh()
    }
}
