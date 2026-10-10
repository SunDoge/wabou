//! Host-owned actor collection. Messages contain data, never implementation callbacks.
use crate::{Actor, Message};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use std::{
    cell::RefCell,
    collections::{BTreeSet, VecDeque},
    ffi::c_void,
    rc::Rc,
};
#[derive(Default)]
pub(crate) struct Router {
    ids: BTreeSet<ActorId>,
    lifecycles: BTreeMap<ActorId, u64>,
    pub(crate) inflight: usize,
    next_request: u64,
    pub(crate) queue: VecDeque<Outgoing>,
    cancellations: BTreeSet<(ActorId, u64, u64)>,
}
impl Router {
    pub(crate) fn cancelled(&self, token: (ActorId, u64, u64)) -> bool {
        self.cancellations.contains(&token)
    }
    pub(crate) fn publish(&mut self, outbox: Vec<Outgoing>) {
        for route in &outbox {
            match route {
                Outgoing::Ask(_) => self.inflight += 1,
                Outgoing::Cancel {
                    source,
                    lifecycle,
                    id,
                } => {
                    self.cancellations.insert((*source, *lifecycle, *id));
                }
                Outgoing::Tell(_, _) => {}
            }
        }
        self.queue.extend(outbox);
    }
}

// Cancellation must remain admissible while ordinary routes are saturated.
// Both classes are independently bounded; publication cannot grow the queue
// beyond 256 ordinary routes plus 256 transactional control commands.
fn route_capacity_exhausted(router: &Router, outbox: &[Outgoing], controls: bool) -> bool {
    if router.queue.len() + outbox.len() < 256 {
        return false;
    }
    router
        .queue
        .iter()
        .chain(outbox)
        .filter(|route| matches!(route, Outgoing::Cancel { .. }) == controls)
        .count()
        >= 256
}

pub(crate) unsafe extern "C" fn child_actor(ctx: *mut c_void, key: u64, out: *mut u64) -> i32 {
    let context = unsafe { &*ctx.cast::<crate::Context>() };
    let turn = unsafe { &*context.turn };
    if turn.read_only || turn.journal.is_none() || key == 0 || out.is_null() {
        return blockui_abi::ERROR;
    }
    let id = if let Some((owner, registry)) = &turn.child_scope {
        let Some(registry) = registry.upgrade() else {
            return blockui_abi::ERROR;
        };
        registry
            .borrow()
            .slots
            .get(&(ActorId(*owner), key))
            .map_or(0, |child| child.id.0)
    } else {
        0
    };
    unsafe {
        *out = id;
    }
    blockui_abi::OK
}
pub(crate) unsafe extern "C" fn send_actor(
    ctx: *mut c_void,
    target: u64,
    action: u64,
    value: i64,
) -> i32 {
    let context = unsafe { &*ctx.cast::<crate::Context>() };
    let turn = unsafe { &mut *context.turn };
    if turn.read_only || turn.journal.is_none() || action == 0 {
        return blockui_abi::ERROR;
    }
    let Some(router) = &turn.router else {
        return blockui_abi::ERROR;
    };
    let router = router.borrow();
    if !router.ids.contains(&ActorId(target))
        || route_capacity_exhausted(&router, &turn.outbox, false)
    {
        return blockui_abi::ERROR;
    }
    turn.outbox.push(Outgoing::Tell(
        ActorId(target),
        Message { action, value }.into(),
    ));
    blockui_abi::OK
}

pub(crate) unsafe extern "C" fn send_actor_text(
    ctx: *mut c_void,
    target: u64,
    action: u64,
    bytes: blockui_abi::Bytes,
) -> i32 {
    if bytes.len > blockui_abi::MAX_MESSAGE_TEXT_BYTES {
        return blockui_abi::ERROR;
    }
    let context = unsafe { &*ctx.cast::<crate::Context>() };
    let turn = unsafe { &mut *context.turn };
    if turn.read_only || turn.journal.is_none() || action == 0 {
        return blockui_abi::ERROR;
    }
    let Some(router) = &turn.router else {
        return blockui_abi::ERROR;
    };
    let router = router.borrow();
    if !router.ids.contains(&ActorId(target))
        || route_capacity_exhausted(&router, &turn.outbox, false)
    {
        return blockui_abi::ERROR;
    }
    let Ok(text) = (unsafe { crate::copy_bytes(bytes) }) else {
        return blockui_abi::ERROR;
    };
    let mut envelope: crate::Envelope = Message { action, value: 0 }.into();
    envelope.message_text = Some(text);
    turn.outbox.push(Outgoing::Tell(ActorId(target), envelope));
    blockui_abi::OK
}

pub(crate) enum Outgoing {
    Tell(ActorId, crate::Envelope),
    Ask(SdkRequest),
    Cancel {
        source: ActorId,
        lifecycle: u64,
        id: u64,
    },
}
impl Outgoing {
    fn target(&self) -> ActorId {
        match self {
            Self::Tell(target, _) => *target,
            Self::Ask(request) => request.target,
            Self::Cancel { source, .. } => *source,
        }
    }
}
pub(crate) struct SdkRequest {
    id: u64,
    source: ActorId,
    lifecycle: u64,
    target: ActorId,
    target_lifecycle: u64,
    message: Message,
    message_text: Option<String>,
    success: u64,
    failure: u64,
    deadline: std::time::Instant,
}
enum SdkReply {
    Scalar(crate::request::Reply),
    Text(crate::request::TextReply),
}
enum ReplyPayload {
    Scalar(i64),
    Text(String),
}
impl SdkReply {
    fn id(&self) -> crate::request::RequestId {
        match self {
            Self::Scalar(reply) => reply.id(),
            Self::Text(reply) => reply.id(),
        }
    }
    fn failure_detail(&self) -> Option<String> {
        match self {
            Self::Scalar(reply) => reply.failure_detail(),
            Self::Text(reply) => reply.failure_detail(),
        }
    }
}
impl std::future::Future for SdkReply {
    type Output = Result<ReplyPayload, crate::request::RequestError>;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.get_mut() {
            Self::Scalar(reply) => std::pin::Pin::new(reply)
                .poll(cx)
                .map(|result| result.map(ReplyPayload::Scalar)),
            Self::Text(reply) => std::pin::Pin::new(reply)
                .poll(cx)
                .map(|result| result.map(ReplyPayload::Text)),
        }
    }
}
struct SdkWait {
    request: SdkRequest,
    reply: Option<SdkReply>,
    completion: Option<Message>,
    message_text: Option<String>,
    failure_detail: Option<String>,
}
pub(crate) unsafe extern "C" fn ask_actor(
    ctx: *mut c_void,
    target: u64,
    action: u64,
    value: i64,
    success: u64,
    failure: u64,
    timeout_ms: u64,
    out: *mut u64,
) -> i32 {
    unsafe {
        stage_ask(
            ctx, target, action, value, None, success, failure, timeout_ms, out,
        )
    }
}

pub(crate) unsafe extern "C" fn ask_actor_text(
    ctx: *mut c_void,
    target: u64,
    action: u64,
    bytes: blockui_abi::Bytes,
    success: u64,
    failure: u64,
    timeout_ms: u64,
    out: *mut u64,
) -> i32 {
    if bytes.len > blockui_abi::MAX_MESSAGE_TEXT_BYTES {
        return blockui_abi::ERROR;
    }
    let Ok(text) = (unsafe { crate::copy_bytes(bytes) }) else {
        return blockui_abi::ERROR;
    };
    unsafe {
        stage_ask(
            ctx,
            target,
            action,
            0,
            Some(text),
            success,
            failure,
            timeout_ms,
            out,
        )
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn stage_ask(
    ctx: *mut c_void,
    target: u64,
    action: u64,
    value: i64,
    message_text: Option<String>,
    success: u64,
    failure: u64,
    timeout_ms: u64,
    out: *mut u64,
) -> i32 {
    let context = unsafe { &*ctx.cast::<crate::Context>() };
    let turn = unsafe { &mut *context.turn };
    if turn.read_only
        || turn.journal.is_none()
        || action == 0
        || success == 0
        || failure == 0
        || out.is_null()
        || timeout_ms == 0
        || timeout_ms > 86_400_000
    {
        return blockui_abi::ERROR;
    }
    let (Some(router), Some((source, lifecycle))) = (&turn.router, turn.owner) else {
        return blockui_abi::ERROR;
    };
    let mut router = router.borrow_mut();
    if !router.ids.contains(&ActorId(target))
        || router.inflight
            + turn
                .outbox
                .iter()
                .filter(|route| matches!(route, Outgoing::Ask(_)))
                .count()
            >= 256
        || route_capacity_exhausted(&router, &turn.outbox, false)
    {
        return blockui_abi::ERROR;
    }
    let Some(deadline) =
        std::time::Instant::now().checked_add(std::time::Duration::from_millis(timeout_ms))
    else {
        return blockui_abi::ERROR;
    };
    let Some(id) = router.next_request.checked_add(1) else {
        return blockui_abi::ERROR;
    };
    router.next_request = id;
    unsafe {
        *out = id;
    }
    turn.outbox.push(Outgoing::Ask(SdkRequest {
        id,
        source,
        lifecycle,
        target: ActorId(target),
        target_lifecycle: router.lifecycles[&ActorId(target)],
        message: Message { action, value },
        message_text,
        success,
        failure,
        deadline,
    }));
    blockui_abi::OK
}

pub(crate) unsafe extern "C" fn cancel_request(ctx: *mut c_void, id: u64) -> i32 {
    let context = unsafe { &*ctx.cast::<crate::Context>() };
    let turn = unsafe { &mut *context.turn };
    if turn.read_only || turn.journal.is_none() || id == 0 {
        return blockui_abi::ERROR;
    }
    let (Some(router), Some((source, lifecycle))) = (&turn.router, turn.owner) else {
        return blockui_abi::ERROR;
    };
    if route_capacity_exhausted(&router.borrow(), &turn.outbox, true) {
        return blockui_abi::ERROR;
    }
    turn.outbox.push(Outgoing::Cancel {
        source,
        lifecycle,
        id,
    });
    blockui_abi::OK
}

/// Stable identity within one runtime. Zero is reserved for an absent actor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActorId(pub u64);

#[derive(Debug)]
pub struct ActorFault {
    pub actor: ActorId,
    pub error: String,
}

/// Drives independent actors on the host thread, with bounded admission and turns.
/// Replacement is per actor; queued messages resolve its current implementation.
pub struct Runtime {
    timer_pool: std::sync::Arc<crate::timer::Pool>,
    io_pool: std::sync::Arc<crate::io::Pool>,
    network_pool: crate::network_task::Pool,
    future_executor: crate::async_task::Executor,
    sdk_waits: Vec<SdkWait>,
    root: PathBuf,
    pub(crate) children: Option<Rc<RefCell<crate::children::Registry>>>,
    applying_children: bool,
    router: Rc<RefCell<Router>>,
    requests: Rc<RefCell<crate::request::RequestTracker>>,
    actors: BTreeMap<ActorId, Actor>,
    pub(crate) parents: BTreeMap<ActorId, ActorId>,
    pub(crate) widget_keys: BTreeMap<(ActorId, u64), u64>,
    pub(crate) next_widget: u64,
    wake: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
}
impl Runtime {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            timer_pool: std::sync::Arc::default(),
            io_pool: std::sync::Arc::default(),
            network_pool: crate::network_task::Pool::default(),
            future_executor: crate::async_task::Executor::default(),
            sdk_waits: Vec::new(),
            root: root.into(),
            children: None,
            applying_children: false,
            router: Rc::default(),
            requests: Rc::default(),
            actors: BTreeMap::new(),
            parents: BTreeMap::new(),
            widget_keys: BTreeMap::new(),
            next_widget: 1,
            wake: None,
        }
    }
    /// Registers data-only implementation identities before actors are started.
    pub fn register_implementation(&mut self, name: &str, path: &Path) -> Result<(), String> {
        self.register_source(
            name,
            crate::children::ImplementationSource::Library(path.to_owned()),
        )
    }
    /// Register a compiled-in child definition before spawning actors. Its callbacks
    /// belong to the test executable and cannot be dynamically unloaded.
    #[cfg(feature = "testing")]
    pub fn register_for_test<A: blockui::Actor>(&mut self, name: &str) -> Result<(), String> {
        self.register_source(
            name,
            crate::children::ImplementationSource::Compiled(Actor::replace_for_test::<A>),
        )
    }
    fn register_source(
        &mut self,
        name: &str,
        source: crate::children::ImplementationSource,
    ) -> Result<(), String> {
        if name.is_empty() || !self.actors.is_empty() {
            return Err("register named implementations before spawning actors".into());
        }
        let registry = self.children.get_or_insert_with(|| {
            Rc::new(RefCell::new(crate::children::Registry::new(
                self.root.clone(),
                self.io_pool.clone(),
                self.timer_pool.clone(),
            )))
        });
        let mut registry = registry.borrow_mut();
        let key = blockui_abi::key(name);
        if registry.implementations.len() >= 64 || registry.implementations.contains_key(&key) {
            return Err("duplicate implementation identity or registry capacity reached".into());
        }
        registry
            .implementations
            .insert(key, (name.to_owned(), source));
        Ok(())
    }
    #[cfg(feature = "native-window")]
    pub(crate) fn has_implementation(&self, name: &str) -> bool {
        self.children.as_ref().is_some_and(|registry| {
            registry
                .borrow()
                .implementations
                .contains_key(&blockui_abi::key(name))
        })
    }
    pub fn child(&self, parent: ActorId, key: u64) -> Option<ActorId> {
        self.children
            .as_ref()?
            .borrow()
            .slots
            .get(&(parent, key))
            .map(|child| child.id)
    }
    /// Data-only identities for currently admitted declarative child lifecycles.
    pub fn declared_children(&self) -> Vec<(ActorId, u64)> {
        self.children.as_ref().map_or_else(Vec::new, |registry| {
            registry
                .borrow()
                .slots
                .values()
                .filter(|child| self.actors.contains_key(&child.id))
                .map(|child| (child.id, child.implementation))
                .collect()
        })
    }
    fn prepare_actor(&self, id: ActorId) -> Result<Actor, String> {
        if let Some(registry) = &self.children
            && registry.borrow().resident >= 64
        {
            return Err("actor staging capacity reached".into());
        }
        let mut actor = Actor::new(id.0, &self.root)?;
        actor.io_executor.use_pool(self.io_pool.clone());
        actor.network_pool = self.network_pool.clone();
        actor.timer_executor.use_pool(self.timer_pool.clone());
        actor.future_executor = self.future_executor.sharing(actor.task_wake.clone());
        if let Some(registry) = &self.children {
            actor.turn.child_scope = Some((id.0, Rc::downgrade(registry)));
            actor.validator = Some(crate::children::validator(Rc::downgrade(registry), id));
        }
        actor.turn.router = Some(self.router.clone());
        if let Some(wake) = &self.wake {
            actor.set_wake(wake.clone());
        }
        Ok(actor)
    }
    pub(crate) fn apply_children(&mut self) {
        if self.applying_children {
            return;
        }
        self.applying_children = true;
        loop {
            let pending = self
                .children
                .as_ref()
                .and_then(|registry| registry.borrow_mut().pending.pop_front());
            let Some(pending) = pending else {
                break;
            };
            for (parent, mut actor) in pending.additions {
                actor.future_executor = self.future_executor.sharing(actor.task_wake.clone());
                actor.network_pool = self.network_pool.clone();
                let id = ActorId(actor.id);
                actor.turn.router = Some(self.router.clone());
                if let Some(wake) = &self.wake {
                    actor.set_wake(wake.clone());
                }
                self.router.borrow_mut().ids.insert(id);
                self.router
                    .borrow_mut()
                    .lifecycles
                    .insert(id, actor.lifecycle());
                self.parents.insert(id, parent);
                self.actors.insert(id, actor);
            }
            for id in pending.removals {
                self.remove(id);
            }
            crate::wake::notify(&mut self.wake);
        }
        self.applying_children = false;
    }
    pub fn spawn(&mut self, id: ActorId, implementation: &Path) -> Result<(), String> {
        self.spawn_using(id, |actor| actor.reload(implementation))
    }
    /// Register a compiled-in SDK definition with the production message router.
    #[cfg(feature = "testing")]
    pub fn spawn_for_test<A: blockui::Actor>(&mut self, id: ActorId) -> Result<(), String> {
        self.spawn_using(id, |actor| actor.replace_for_test::<A>())
    }
    #[cfg(feature = "testing")]
    pub fn replace_for_test<A: blockui::Actor>(&mut self, id: ActorId) -> Result<(), String> {
        self.actors
            .get_mut(&id)
            .ok_or("unknown actor")?
            .replace_for_test::<A>()?;
        self.apply_children();
        Ok(())
    }
    fn spawn_using(
        &mut self,
        id: ActorId,
        initialize: impl FnOnce(&mut Actor) -> Result<(), String>,
    ) -> Result<(), String> {
        if id.0 == 0 || self.actors.contains_key(&id) {
            return Err("invalid or occupied actor identity".into());
        }
        if self.actors.len() >= 64 {
            return Err("actor capacity reached".into());
        }
        let mut actor = self.prepare_actor(id)?;
        if let Some(registry) = &self.children {
            let mut registry = registry.borrow_mut();
            registry.resident += 1;
            registry.live.insert(id);
        }
        if let Err(error) = initialize(&mut actor) {
            if let Some(registry) = &self.children {
                let mut registry = registry.borrow_mut();
                registry.resident -= 1;
                registry.retire(id);
            }
            return Err(error);
        }
        self.router.borrow_mut().ids.insert(id);
        self.router
            .borrow_mut()
            .lifecycles
            .insert(id, actor.lifecycle());
        self.actors.insert(id, actor);
        self.apply_children();
        Ok(())
    }
    /// Creates an owned child. Ownership survives replacement of either implementation.
    pub fn spawn_child(
        &mut self,
        parent: ActorId,
        id: ActorId,
        implementation: &Path,
    ) -> Result<(), String> {
        if !self.actors.contains_key(&parent) {
            return Err("unknown parent actor".into());
        }
        self.spawn(id, implementation)?;
        self.parents.insert(id, parent);
        Ok(())
    }
    /// Reconciles direct children by stable IDs. Existing children are retained;
    /// their implementation changes use explicit `reload`, independently of membership.
    /// All new implementations are validated before any existing child is removed.
    pub fn reconcile_children(
        &mut self,
        parent: ActorId,
        desired: &[(ActorId, &Path)],
    ) -> Result<(), String> {
        if self.children.is_some() {
            return Err("declarative child membership is owned by the view".into());
        }
        if !self.actors.contains_key(&parent) {
            return Err("unknown parent actor".into());
        }
        let mut ids = BTreeSet::new();
        let mut additions = Vec::new();
        for (id, path) in desired {
            if id.0 == 0 || !ids.insert(*id) {
                return Err("invalid or duplicate child identity".into());
            }
            if self.actors.contains_key(id) {
                if self.parent(*id) != Some(parent) {
                    return Err("child identity belongs to another owner".into());
                }
            } else {
                additions.push((*id, *path));
            }
        }
        // Bound peak worker/library/disk use, including both old and staged actors.
        if self.actors.len() + additions.len() > 64 {
            return Err("actor staging capacity reached".into());
        }
        let mut staged = Vec::new();
        for (id, path) in additions {
            let mut actor = self.prepare_actor(id)?;
            actor.reload(path)?;
            staged.push((id, actor));
        }
        let removed: Vec<_> = self
            .parents
            .iter()
            .filter_map(|(id, owner)| (*owner == parent && !ids.contains(id)).then_some(*id))
            .collect();
        for id in removed {
            self.remove(id);
        }
        for (id, actor) in staged {
            self.router.borrow_mut().ids.insert(id);
            self.router
                .borrow_mut()
                .lifecycles
                .insert(id, actor.lifecycle());
            self.parents.insert(id, parent);
            self.actors.insert(id, actor);
        }
        Ok(())
    }
    pub fn parent(&self, id: ActorId) -> Option<ActorId> {
        self.parents.get(&id).copied()
    }
    /// Host callback only: installed on both existing and subsequently created actors.
    pub fn set_wake(&mut self, wake: std::sync::Arc<dyn Fn() + Send + Sync>) {
        for actor in self.actors.values_mut() {
            actor.set_wake(wake.clone());
        }
        self.wake = Some(wake);
    }
    pub(crate) fn actor_mut(&mut self, id: ActorId) -> Option<&mut Actor> {
        self.actors.get_mut(&id)
    }
    pub fn actor(&self, id: ActorId) -> Option<&Actor> {
        self.actors.get(&id)
    }
    pub fn reload(&mut self, id: ActorId, implementation: &Path) -> Result<(), String> {
        self.reload_group(&[id], implementation)
    }
    /// Prepare every target before any state/code/view or lifecycle change commits.
    pub fn reload_group(&mut self, ids: &[ActorId], implementation: &Path) -> Result<(), String> {
        let unique = self.validate_group_targets(ids)?;
        let snapshot =
            std::rc::Rc::new(crate::children::Snapshot::new(&self.root, implementation)?);
        self.replace_group_using(
            ids,
            unique,
            crate::children::ImplementationSource::Library(snapshot.0.clone()),
            Some(snapshot.clone()),
            |actor| actor.prepare_reload(&snapshot.0),
        )
    }
    /// Atomically replace compiled-in definitions and update registered child sources.
    #[cfg(feature = "testing")]
    pub fn replace_group_for_test<A: blockui::Actor>(
        &mut self,
        ids: &[ActorId],
    ) -> Result<(), String> {
        let unique = self.validate_group_targets(ids)?;
        self.replace_group_using(
            ids,
            unique,
            crate::children::ImplementationSource::Compiled(Actor::replace_for_test::<A>),
            None,
            Actor::prepare_for_test::<A>,
        )
    }
    fn validate_group_targets(
        &self,
        ids: &[ActorId],
    ) -> Result<std::collections::BTreeSet<ActorId>, String> {
        if ids.is_empty() {
            return Err("reload group must have targets".into());
        }
        let unique: std::collections::BTreeSet<_> = ids.iter().copied().collect();
        if unique.len() != ids.len() || ids.iter().any(|id| !self.actors.contains_key(id)) {
            return Err("duplicate or unknown reload target".into());
        }
        Ok(unique)
    }
    fn replace_group_using(
        &mut self,
        ids: &[ActorId],
        unique: std::collections::BTreeSet<ActorId>,
        source: crate::children::ImplementationSource,
        snapshot: Option<std::rc::Rc<crate::children::Snapshot>>,
        prepare: impl Fn(&mut Actor) -> Result<crate::PreparedReload, String>,
    ) -> Result<(), String> {
        let identities: std::collections::BTreeSet<_> =
            self.children
                .as_ref()
                .map_or_else(Default::default, |registry| {
                    registry
                        .borrow()
                        .slots
                        .values()
                        .filter(|child| unique.contains(&child.id))
                        .map(|child| child.implementation)
                        .collect()
                });
        let mut old_sources = Vec::new();
        if let Some(registry) = &self.children {
            let mut registry = registry.borrow_mut();
            for identity in &identities {
                let entry = registry.implementations.get_mut(identity).unwrap();
                old_sources.push((*identity, std::mem::replace(&mut entry.1, source.clone())));
            }
        }
        let mut prepared = Vec::new();
        for id in ids {
            match prepare(self.actors.get_mut(id).unwrap()) {
                Ok(candidate) => prepared.push((*id, candidate)),
                Err(error) => {
                    if let Some(registry) = &self.children {
                        let mut registry = registry.borrow_mut();
                        for (identity, path) in old_sources {
                            registry.implementations.get_mut(&identity).unwrap().1 = path;
                        }
                    }
                    return Err(format!("actor {}: {error}", id.0));
                }
            }
        }
        // All fallible plugin work has completed. No turns or rendering occur
        // between these commits; admissions hold bounded staged children.
        for (id, candidate) in prepared {
            self.actors.get_mut(&id).unwrap().commit_reload(candidate);
        }
        if !identities.is_empty() {
            // A group normally has one named implementation. Sharing ownership
            // also makes explicit heterogeneous groups use one immutable snapshot.
            let mut registry = self.children.as_ref().unwrap().borrow_mut();
            for identity in identities {
                if let Some(snapshot) = &snapshot {
                    registry.accepted.insert(identity, snapshot.clone());
                } else {
                    registry.accepted.remove(&identity);
                }
            }
        }
        self.apply_children();
        Ok(())
    }
    /// Submit a stable-host future to an actor without borrowing its state.
    /// The future and its destructor must belong to the host executable.
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
        self.actors
            .get_mut(&target)
            .ok_or("unknown actor")?
            .spawn_async_with_policy(id, future, on_panic, policy)
    }
    /// Cancel runtime-owned work; an absent task is an idempotent false result.
    pub fn cancel_task(&mut self, target: ActorId, id: u64) -> Result<bool, String> {
        Ok(self
            .actors
            .get_mut(&target)
            .ok_or("unknown actor")?
            .cancel_task(id))
    }
    /// Queue a scalar request in mailbox order and return a stable-host reply future.
    /// The runtime must continue ticking while the caller awaits; no plugin future
    /// or mutable actor borrow may be held across the wait.
    pub fn ask(
        &mut self,
        target: ActorId,
        message: Message,
        timeout: std::time::Duration,
    ) -> Result<crate::request::Reply, String> {
        if message.action == 0
            || timeout.is_zero()
            || timeout > std::time::Duration::from_secs(86_400)
        {
            return Err("invalid request action or timeout (maximum one day)".into());
        }
        let deadline = std::time::Instant::now()
            .checked_add(timeout)
            .ok_or("request deadline out of range")?;
        self.ask_until(target, message, deadline)
    }
    fn ask_until(
        &mut self,
        target: ActorId,
        message: Message,
        deadline: std::time::Instant,
    ) -> Result<crate::request::Reply, String> {
        let actor = self.actors.get_mut(&target).ok_or("unknown actor")?;
        if actor.mailbox.len() >= 256 {
            return Err("mailbox full".into());
        }
        let (id, reply) = self
            .requests
            .borrow_mut()
            .admit(target, actor.lifecycle(), deadline)?;
        actor.mailbox.push_back(crate::Envelope {
            request_error: None,
            message_text: None,
            text_request: false,
            sdk_request: None,
            message,
            request: Some((id, self.requests.clone())),
            timer_ack: None,
            io_result: None,
        });
        crate::wake::notify(&mut actor.wake);
        Ok(reply)
    }
    /// Queue a bounded owned UTF-8 request. The text reply future belongs to host code.
    pub fn ask_text(
        &mut self,
        target: ActorId,
        action: u64,
        text: String,
        timeout: std::time::Duration,
    ) -> Result<crate::request::TextReply, String> {
        if action == 0
            || text.len() > blockui_abi::MAX_MESSAGE_TEXT_BYTES
            || timeout.is_zero()
            || timeout > std::time::Duration::from_secs(86_400)
        {
            return Err("invalid text request action, payload or timeout (maximum one day)".into());
        }
        let deadline = std::time::Instant::now()
            .checked_add(timeout)
            .ok_or("request deadline out of range")?;
        self.ask_text_until(target, action, text, deadline)
    }
    fn ask_text_until(
        &mut self,
        target: ActorId,
        action: u64,
        text: String,
        deadline: std::time::Instant,
    ) -> Result<crate::request::TextReply, String> {
        let actor = self.actors.get_mut(&target).ok_or("unknown actor")?;
        if actor.mailbox.len() >= 256 {
            return Err("mailbox full".into());
        }
        let (id, reply) =
            self.requests
                .borrow_mut()
                .admit_text(target, actor.lifecycle(), deadline)?;
        let mut envelope: crate::Envelope = Message { action, value: 0 }.into();
        envelope.message_text = Some(text);
        envelope.text_request = true;
        envelope.request = Some((id, self.requests.clone()));
        actor.mailbox.push_back(envelope);
        crate::wake::notify(&mut actor.wake);
        Ok(reply)
    }
    pub fn cancel_request(&mut self, id: crate::request::RequestId) -> bool {
        self.requests.borrow_mut().cancel(id)
    }
    /// Earliest pending request deadline, for host event-loop wake scheduling.
    pub fn next_request_deadline(&self) -> Option<std::time::Instant> {
        self.requests
            .borrow()
            .next_deadline()
            .into_iter()
            .chain(
                self.sdk_waits
                    .iter()
                    .filter(|wait| wait.completion.is_none())
                    .map(|wait| wait.request.deadline),
            )
            .chain(
                self.router
                    .borrow()
                    .queue
                    .iter()
                    .filter_map(|route| match route {
                        Outgoing::Ask(request) => Some(request.deadline),
                        _ => None,
                    }),
            )
            .min()
    }
    /// Admit one owned message to the recipient mailbox. Processing happens on tick.
    pub fn tell(&mut self, target: ActorId, message: Message) -> Result<(), String> {
        if message.action == 0 {
            return Err("invalid action identity".into());
        }
        let actor = self.actors.get_mut(&target).ok_or("unknown actor")?;
        actor.enqueue(message)?;
        crate::wake::notify(&mut actor.wake);
        Ok(())
    }
    /// One failing actor does not prevent the remaining actors from making progress.
    pub fn tick(&mut self) -> Vec<ActorFault> {
        self.requests.borrow_mut().expire(std::time::Instant::now());
        self.apply_children();
        self.deliver();
        let ids: Vec<_> = self.actors.keys().copied().collect();
        let mut faults = Vec::new();
        for id in ids {
            if let Some(actor) = self.actors.get_mut(&id)
                && let Err(error) = actor.tick()
            {
                faults.push(ActorFault { actor: id, error });
            }
            self.apply_children();
        }
        self.deliver();
        self.deliver_requests();
        faults
    }

    fn deliver(&mut self) {
        self.deliver_routes(false);
    }
    fn deliver_routes(&mut self, requests: bool) {
        let (routes, cancelled) = {
            let mut router = self.router.borrow_mut();
            (
                std::mem::take(&mut router.queue),
                std::mem::take(&mut router.cancellations),
            )
        };
        for wait in &mut self.sdk_waits {
            if cancelled.contains(&(wait.request.source, wait.request.lifecycle, wait.request.id)) {
                if let Some(reply) = wait.reply.take() {
                    self.requests.borrow_mut().cancel(reply.id());
                }
                wait.failure_detail = None;
                wait.message_text = None;
                wait.completion = Some(Message {
                    action: wait.request.failure,
                    value: crate::request::RequestError::Cancelled.code(),
                });
            }
        }
        let mut blocked = BTreeSet::new();
        for route in routes {
            if matches!(route, Outgoing::Cancel { .. }) {
                continue;
            }
            if let Outgoing::Ask(request) = &route
                && cancelled.contains(&(request.source, request.lifecycle, request.id))
            {
                self.sdk_waits.push(SdkWait {
                    message_text: None,
                    failure_detail: None,
                    completion: Some(Message {
                        action: request.failure,
                        value: crate::request::RequestError::Cancelled.code(),
                    }),
                    reply: None,
                    request: match route {
                        Outgoing::Ask(request) => request,
                        _ => unreachable!(),
                    },
                });
                continue;
            }
            let target = route.target();
            // Expired/retired requests must leave even a blocked target's queue;
            // otherwise an already-due deadline can make the idle host spin.
            let terminal = match &route {
                Outgoing::Ask(request) => {
                    std::time::Instant::now() >= request.deadline
                        || !self
                            .actors
                            .get(&request.source)
                            .is_some_and(|actor| actor.lifecycle() == request.lifecycle)
                        || !self
                            .actors
                            .get(&target)
                            .is_some_and(|actor| actor.lifecycle() == request.target_lifecycle)
                }
                _ => false,
            };
            if blocked.contains(&target) && !terminal {
                self.router.borrow_mut().queue.push_back(route);
                continue;
            }
            match route {
                Outgoing::Cancel { .. } => unreachable!(),
                Outgoing::Tell(id, message) => {
                    if let Some(actor) = self.actors.get_mut(&id) {
                        if let Some(message) = actor.enqueue_envelope(message) {
                            blocked.insert(id);
                            self.router
                                .borrow_mut()
                                .queue
                                .push_back(Outgoing::Tell(id, message));
                        } else {
                            crate::wake::notify(&mut actor.wake);
                        }
                    }
                }
                Outgoing::Ask(mut request) => {
                    if !requests && !terminal {
                        blocked.insert(target);
                        self.router
                            .borrow_mut()
                            .queue
                            .push_back(Outgoing::Ask(request));
                        continue;
                    }
                    let live_source = self
                        .actors
                        .get(&request.source)
                        .is_some_and(|actor| actor.lifecycle() == request.lifecycle);
                    if !live_source {
                        self.router.borrow_mut().inflight -= 1;
                        continue;
                    }
                    let completion = if std::time::Instant::now() >= request.deadline {
                        Some(Message {
                            action: request.failure,
                            value: crate::request::RequestError::Timeout.code(),
                        })
                    } else if !self
                        .actors
                        .get(&target)
                        .is_some_and(|actor| actor.lifecycle() == request.target_lifecycle)
                    {
                        Some(Message {
                            action: request.failure,
                            value: crate::request::RequestError::TargetTerminated.code(),
                        })
                    } else {
                        None
                    };
                    let reply = if completion.is_none() {
                        match if let Some(text) = &request.message_text {
                            self.ask_text_until(
                                target,
                                request.message.action,
                                text.clone(),
                                request.deadline,
                            )
                            .map(SdkReply::Text)
                        } else {
                            self.ask_until(target, request.message, request.deadline)
                                .map(SdkReply::Scalar)
                        } {
                            Ok(reply) => {
                                // Input is now owned by the target envelope; don't
                                // keep a second payload for the duration of the wait.
                                request.message_text = None;
                                // ask_until just appended this host-owned envelope. Mark it
                                // before any actor turn can consume it; no plugin pointer escapes.
                                let envelope = self
                                    .actors
                                    .get_mut(&target)
                                    .unwrap()
                                    .mailbox
                                    .back_mut()
                                    .unwrap();
                                debug_assert_eq!(envelope.request.as_ref().unwrap().0, reply.id());
                                envelope.sdk_request =
                                    Some((request.source, request.lifecycle, request.id));
                                Some(reply)
                            }
                            Err(_) => {
                                blocked.insert(target);
                                self.router
                                    .borrow_mut()
                                    .queue
                                    .push_back(Outgoing::Ask(request));
                                continue;
                            }
                        }
                    } else {
                        None
                    };
                    self.sdk_waits.push(SdkWait {
                        message_text: None,
                        failure_detail: None,
                        request,
                        reply,
                        completion,
                    });
                }
            }
        }
    }
    fn deliver_requests(&mut self) {
        use std::{
            future::Future,
            pin::Pin,
            task::{Context, Poll, Waker},
        };
        self.deliver_routes(true);
        let waits = std::mem::take(&mut self.sdk_waits);
        for mut wait in waits {
            let live = self
                .actors
                .get(&wait.request.source)
                .is_some_and(|actor| actor.lifecycle() == wait.request.lifecycle);
            if !live {
                if let Some(reply) = &wait.reply {
                    self.requests.borrow_mut().cancel(reply.id());
                }
                self.router.borrow_mut().inflight -= 1;
                continue;
            }
            if let Some(reply) = &mut wait.reply
                && let Poll::Ready(result) =
                    Pin::new(&mut *reply).poll(&mut Context::from_waker(Waker::noop()))
            {
                wait.failure_detail = if result.is_err() {
                    reply.failure_detail()
                } else {
                    None
                };
                wait.completion = Some(match result {
                    Ok(ReplyPayload::Scalar(value)) => Message {
                        action: wait.request.success,
                        value,
                    },
                    Ok(ReplyPayload::Text(text)) => {
                        wait.message_text = Some(text);
                        Message {
                            action: wait.request.success,
                            value: 0,
                        }
                    }
                    Err(error) => Message {
                        action: wait.request.failure,
                        value: error.code(),
                    },
                });
                wait.reply = None;
            }
            if let Some(message) = wait.completion {
                let actor = self.actors.get_mut(&wait.request.source).unwrap();
                let mut envelope: crate::Envelope = message.into();
                envelope.message_text = wait.message_text.take();
                envelope.request_error = wait.failure_detail.take();
                if let Some(envelope) = actor.enqueue_envelope(envelope) {
                    wait.message_text = envelope.message_text;
                    wait.failure_detail = envelope.request_error;
                } else {
                    crate::wake::notify(&mut actor.wake);
                    self.router.borrow_mut().inflight -= 1;
                    continue;
                }
            }
            self.sdk_waits.push(wait);
        }
    }
    /// Explicit termination disposes descendants before their parent.
    /// Failure of an actor turn does not implicitly terminate its children.
    pub fn remove(&mut self, id: ActorId) -> bool {
        self.apply_children();
        if !self.actors.contains_key(&id) {
            return false;
        }
        let children: Vec<_> = self
            .parents
            .iter()
            .filter_map(|(child, parent)| (*parent == id).then_some(*child))
            .collect();
        for child in children {
            self.remove(child);
        }
        self.parents.remove(&id);
        self.widget_keys.retain(|(owner, _), _| *owner != id);
        self.router.borrow_mut().ids.remove(&id);
        self.router.borrow_mut().lifecycles.remove(&id);
        self.router
            .borrow_mut()
            .queue
            .retain(|route| !matches!(route, Outgoing::Tell(target, _) if *target == id));
        if let Some(registry) = &self.children {
            let mut registry = registry.borrow_mut();
            registry.resident -= 1;
            registry.retire(id);
        }
        // Caller termination abandons its waits immediately, before queued target turns.
        let mut retired = 0;
        self.sdk_waits.retain(|wait| {
            if wait.request.source == id {
                if let Some(reply) = &wait.reply {
                    self.requests.borrow_mut().cancel(reply.id());
                }
                retired += 1;
                false
            } else {
                true
            }
        });
        {
            let mut router = self.router.borrow_mut();
            router.queue.retain(|route| {
                if matches!(route, Outgoing::Ask(request) if request.source == id) {
                    retired += 1;
                    false
                } else {
                    !matches!(route, Outgoing::Cancel { source, .. } if *source == id)
                }
            });
            router.cancellations.retain(|(source, _, _)| *source != id);
            router.inflight -= retired;
        }
        let lifecycle = self.actors[&id].lifecycle();
        self.requests.borrow_mut().terminate(id, lifecycle);
        self.actors.remove(&id);
        true
    }
    pub fn len(&self) -> usize {
        self.actors.len()
    }
    pub fn is_empty(&self) -> bool {
        self.actors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires counter DLLs; run scripts/verify_requests.py"]
    fn real_sdk_ask_text_retains_owned_completion_under_backpressure_and_both_reloads() {
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let next = std::env::var_os("ACTOR_TEST_REPLACEMENT").unwrap();
        let root =
            std::env::temp_dir().join(format!("blockui-text-backpressure-{}", std::process::id()));
        {
            let mut runtime = Runtime::new(&root);
            let source = ActorId(1);
            let target = ActorId(2);
            runtime.spawn(source, Path::new(&base)).unwrap();
            runtime.spawn(target, Path::new(&base)).unwrap();
            let slot = runtime.actor(source).unwrap().slot_address("notes");
            runtime
                .actor_mut(source)
                .unwrap()
                .set_text(blockui_abi::key("notes"), "資料 😀")
                .unwrap();
            runtime
                .actor_mut(source)
                .unwrap()
                .dispatch_request(Message {
                    action: blockui_abi::key("ask_text"),
                    value: 2,
                })
                .unwrap();
            runtime
                .actor_mut(source)
                .unwrap()
                .set_text(blockui_abi::key("notes"), "changed after send")
                .unwrap();
            runtime.reload(target, Path::new(&next)).unwrap();
            runtime.deliver_requests();
            assert!(runtime.actor_mut(target).unwrap().tick().is_ok());
            for _ in 0..256 {
                runtime
                    .actor_mut(source)
                    .unwrap()
                    .enqueue(Message {
                        action: blockui_abi::key("increment"),
                        value: 0,
                    })
                    .unwrap();
            }
            runtime.deliver_requests();
            assert_eq!(runtime.sdk_waits.len(), 1);
            assert_eq!(
                runtime.sdk_waits[0].message_text.as_deref(),
                Some("v2:資料 😀")
            );
            runtime.reload(source, Path::new(&next)).unwrap();
            for _ in 0..10 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(
                runtime.actor(source).unwrap().text("notes"),
                "v2:v2:資料 😀"
            );
            assert_eq!(runtime.actor(source).unwrap().slot_address("notes"), slot);
            assert!(runtime.sdk_waits.is_empty());
            assert_eq!(runtime.router.borrow().inflight, 0);
        }
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    #[ignore = "requires counter DLLs; run scripts/verify_requests.py"]
    fn real_sdk_ask_text_transaction_bounds_cancel_timeout_and_lifecycles() {
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let root = std::env::temp_dir().join(format!("blockui-text-policy-{}", std::process::id()));
        for case in [
            "empty", "maximum", "rollback", "oversize", "fail", "cancel", "timeout", "source",
            "target",
        ] {
            {
                let mut runtime = Runtime::new(&root);
                let source = ActorId(1);
                let target = ActorId(2);
                runtime.spawn(source, Path::new(&base)).unwrap();
                runtime.spawn(target, Path::new(&base)).unwrap();
                let text = match case {
                    "empty" => String::new(),
                    "maximum" => "x".repeat(blockui_abi::MAX_MESSAGE_TEXT_BYTES),
                    "oversize" => "x".repeat(blockui_abi::MAX_MESSAGE_TEXT_BYTES + 1),
                    "fail" | "timeout" => case.into(),
                    _ => "資料 😀".into(),
                };
                runtime
                    .actor_mut(source)
                    .unwrap()
                    .set_text(blockui_abi::key("notes"), &text)
                    .unwrap();
                let request = runtime
                    .actor_mut(source)
                    .unwrap()
                    .dispatch_request(Message {
                        action: blockui_abi::key("ask_text"),
                        value: if case == "rollback" { -2 } else { 2 },
                    });
                if matches!(case, "rollback" | "oversize") {
                    assert!(request.is_err());
                    assert!(runtime.router.borrow().queue.is_empty());
                    assert_eq!(runtime.router.borrow().inflight, 0);
                    assert!(runtime.tick().is_empty());
                    assert_eq!(runtime.actor(target).unwrap().integer("count"), 0);
                    continue;
                }
                let request = request.unwrap().unwrap();
                match case {
                    "cancel" => runtime
                        .actor_mut(source)
                        .unwrap()
                        .dispatch(Message {
                            action: blockui_abi::key("cancel_request"),
                            value: request,
                        })
                        .unwrap(),
                    "timeout" => std::thread::sleep(std::time::Duration::from_millis(5)),
                    "source" => {
                        assert!(runtime.remove(source));
                    }
                    "target" => {
                        assert!(runtime.remove(target));
                    }
                    _ => {}
                }
                let faults: Vec<_> = (0..4).flat_map(|_| runtime.tick()).collect();
                assert_eq!(faults.len(), usize::from(case == "fail"), "{case}");
                assert!(runtime.sdk_waits.is_empty(), "{case}");
                assert_eq!(runtime.router.borrow().inflight, 0, "{case}");
                if let Some(actor) = runtime.actor(source) {
                    let error = match case {
                        "fail" => Some(crate::request::RequestError::HandlerFailed),
                        "cancel" => Some(crate::request::RequestError::Cancelled),
                        "timeout" => Some(crate::request::RequestError::Timeout),
                        "target" => Some(crate::request::RequestError::TargetTerminated),
                        _ => None,
                    };
                    if let Some(error) = error {
                        assert_eq!(actor.integer("task_result"), -error.code(), "{case}");
                    }
                    assert_eq!(actor.text("notes"), text, "{case}");
                }
                if !matches!(case, "empty" | "maximum")
                    && let Some(actor) = runtime.actor(target)
                {
                    assert_eq!(actor.integer("count"), 0, "{case}");
                }
            }
        }
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn text_send_validates_before_copy_and_obeys_turn_and_route_capacity() {
        let router = Rc::new(RefCell::new(Router::default()));
        router.borrow_mut().ids.insert(ActorId(2));
        let mut turn = crate::Turn {
            router: Some(router.clone()),
            journal: Some(BTreeMap::new()),
            ..crate::Turn::default()
        };
        let context = crate::Context {
            turn: &mut turn,
            table: std::ptr::null(),
        };
        let ctx = (&context as *const crate::Context).cast_mut().cast();
        for bytes in [
            blockui_abi::Bytes {
                ptr: std::ptr::null(),
                len: blockui_abi::MAX_MESSAGE_TEXT_BYTES + 1,
            },
            blockui_abi::Bytes {
                ptr: std::ptr::null(),
                len: 1,
            },
            blockui_abi::Bytes {
                ptr: [0xff].as_ptr(),
                len: 1,
            },
        ] {
            assert_eq!(
                unsafe { send_actor_text(ctx, 2, 7, bytes) },
                blockui_abi::ERROR
            );
        }
        assert!(turn.outbox.is_empty());
        let mut text = String::from("資料 😀");
        assert_eq!(
            unsafe { send_actor_text(ctx, 2, 7, blockui_abi::Bytes::new(&text)) },
            blockui_abi::OK
        );
        text.clear();
        let Outgoing::Tell(_, envelope) = &turn.outbox[0] else {
            panic!("tell")
        };
        assert_eq!(envelope.message_text.as_deref(), Some("資料 😀"));
        assert_eq!(
            unsafe { send_actor_text(ctx, 3, 7, blockui_abi::Bytes::new("")) },
            blockui_abi::ERROR
        );
        assert_eq!(
            unsafe { send_actor_text(ctx, 2, 0, blockui_abi::Bytes::new("")) },
            blockui_abi::ERROR
        );
        turn.read_only = true;
        assert_eq!(
            unsafe { send_actor_text(ctx, 2, 7, blockui_abi::Bytes::new("")) },
            blockui_abi::ERROR
        );
        turn.read_only = false;
        turn.journal = None;
        assert_eq!(
            unsafe { send_actor_text(ctx, 2, 7, blockui_abi::Bytes::new("")) },
            blockui_abi::ERROR
        );
        turn.journal = Some(BTreeMap::new());
        for _ in 1..256 {
            assert_eq!(
                unsafe {
                    send_actor_text(
                        ctx,
                        2,
                        7,
                        blockui_abi::Bytes {
                            ptr: std::ptr::null(),
                            len: 0,
                        },
                    )
                },
                blockui_abi::OK
            );
        }
        assert_eq!(
            unsafe { send_actor_text(ctx, 2, 7, blockui_abi::Bytes::new("")) },
            blockui_abi::ERROR
        );
        assert_eq!(turn.outbox.len(), 256);
        assert!(turn.changed.is_empty());
    }

    #[test]
    #[ignore = "requires counter DLLs; run scripts/verify_requests.py"]
    fn real_host_text_request_commits_reloads_and_rolls_back_failed_replies() {
        use std::{
            future::Future,
            pin::Pin,
            task::{Context, Poll, Waker},
        };
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let next = std::env::var_os("ACTOR_TEST_REPLACEMENT").unwrap();
        let root =
            std::env::temp_dir().join(format!("actor-host-text-query-{}", std::process::id()));
        let timeout = std::time::Duration::from_secs(30);
        let action = blockui_abi::key("text_query");
        let target = ActorId(1);
        let poll = |reply: &mut crate::request::TextReply| {
            Pin::new(reply).poll(&mut Context::from_waker(Waker::noop()))
        };
        {
            let mut runtime = Runtime::new(&root);
            runtime.spawn(target, Path::new(&base)).unwrap();
            let address = runtime.actor(target).unwrap().slot_address("count");
            let mut reply = runtime
                .ask_text(target, action, "資料 😀".into(), timeout)
                .unwrap();
            assert_eq!(poll(&mut reply), Poll::Pending);
            runtime.reload(target, Path::new(&next)).unwrap();
            assert!(runtime.tick().is_empty());
            assert_eq!(
                runtime.actor(target).unwrap().slot_address("count"),
                address
            );
            assert_eq!(runtime.actor(target).unwrap().integer("count"), 10);
            runtime.reload(target, Path::new(&base)).unwrap();
            assert_eq!(poll(&mut reply), Poll::Ready(Ok("v2:資料 😀".into())));
            for text in [
                String::new(),
                "x".repeat(blockui_abi::MAX_MESSAGE_TEXT_BYTES),
            ] {
                let mut reply = runtime
                    .ask_text(target, action, text.clone(), timeout)
                    .unwrap();
                assert!(runtime.tick().is_empty());
                assert_eq!(poll(&mut reply), Poll::Ready(Ok(text)));
            }
            let before = runtime.actor(target).unwrap().integer("count");
            for text in ["fail", "duplicate", "wrong-kind", "oversize", "panic"] {
                let mut reply = runtime
                    .ask_text(target, action, text.into(), timeout)
                    .unwrap();
                let faults = runtime.tick();
                assert_eq!(faults.len(), 1, "{text}");
                assert_eq!(
                    poll(&mut reply),
                    Poll::Ready(Err(crate::request::RequestError::HandlerFailed))
                );
                assert!(reply.failure_detail().is_some());
                let actor = runtime.actor(target).unwrap();
                assert_eq!(actor.integer("count"), before, "{text}");
                assert!(actor.turn.message_text.is_none());
                assert!(actor.turn.request_text_reply.is_none());
            }
            assert!(
                runtime
                    .ask_text(
                        target,
                        action,
                        "x".repeat(blockui_abi::MAX_MESSAGE_TEXT_BYTES + 1),
                        timeout
                    )
                    .is_err()
            );
            assert!(runtime.ask_text(target, 0, "".into(), timeout).is_err());
            assert!(
                runtime
                    .ask_text(target, action, "".into(), std::time::Duration::ZERO)
                    .is_err()
            );
            let actor = runtime.actors.get_mut(&target).unwrap();
            let mut tell: crate::Envelope = Message { action, value: 0 }.into();
            tell.message_text = Some("tell cannot reply".into());
            assert!(actor.enqueue_envelope(tell).is_none());
            assert!(actor.tick().is_err());
            assert_eq!(actor.integer("count"), before);
            assert!(actor.turn.request_text_reply.is_none());
            let mut wrong = runtime
                .ask(target, Message { action, value: 0 }, timeout)
                .unwrap();
            assert_eq!(runtime.tick().len(), 1);
            assert_eq!(
                Pin::new(&mut wrong).poll(&mut Context::from_waker(Waker::noop())),
                Poll::Ready(Err(crate::request::RequestError::HandlerFailed))
            );
            let mut cancelled = runtime
                .ask_text(target, action, "cancelled".into(), timeout)
                .unwrap();
            assert!(runtime.cancel_request(cancelled.id()));
            assert!(runtime.tick().is_empty());
            assert_eq!(
                poll(&mut cancelled),
                Poll::Ready(Err(crate::request::RequestError::Cancelled))
            );
            assert_eq!(runtime.actor(target).unwrap().integer("count"), before);
            let mut terminated = runtime
                .ask_text(target, action, "removed".into(), timeout)
                .unwrap();
            assert!(runtime.remove(target));
            runtime.spawn(target, Path::new(&next)).unwrap();
            assert!(runtime.tick().is_empty());
            assert_eq!(
                poll(&mut terminated),
                Poll::Ready(Err(crate::request::RequestError::TargetTerminated))
            );
            let mut stopped = runtime
                .ask_text(target, action, "stopped".into(), timeout)
                .unwrap();
            drop(runtime);
            assert_eq!(
                poll(&mut stopped),
                Poll::Ready(Err(crate::request::RequestError::RuntimeStopped))
            );
            assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
        }
        std::fs::remove_dir(root).unwrap();
    }

    #[test]
    #[ignore = "requires counter DLLs; run scripts/verify_requests.py"]
    fn real_sdk_text_tell_obeys_sender_and_recipient_termination() {
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let next = std::env::var_os("ACTOR_TEST_REPLACEMENT").unwrap();
        let root =
            std::env::temp_dir().join(format!("actor-text-lifecycle-{}", std::process::id()));
        {
            let mut runtime = Runtime::new(&root);
            let source = ActorId(1);
            let target = ActorId(2);
            runtime.spawn(source, Path::new(&base)).unwrap();
            runtime.spawn(target, Path::new(&base)).unwrap();
            let send = Message {
                action: blockui_abi::key("send_text"),
                value: target.0 as i64,
            };
            let caller = runtime.actors.get_mut(&source).unwrap();
            caller
                .set_text(blockui_abi::key("notes"), "sender gone 資料")
                .unwrap();
            caller.dispatch(send).unwrap();
            assert_eq!(runtime.router.borrow().queue.len(), 1);
            // Committed tells belong to the recipient; unloading the sender cannot cancel them.
            assert!(runtime.remove(source));
            runtime.reload(target, Path::new(&next)).unwrap();
            assert!(runtime.tick().is_empty());
            assert_eq!(
                runtime.actor(target).unwrap().text("notes"),
                "v2:sender gone 資料"
            );
            assert!(runtime.router.borrow().queue.is_empty());
            runtime.spawn(source, Path::new(&base)).unwrap();
            let caller = runtime.actors.get_mut(&source).unwrap();
            caller
                .set_text(blockui_abi::key("notes"), "retired route")
                .unwrap();
            caller.dispatch(send).unwrap();
            let recipient = runtime.actors.get_mut(&target).unwrap();
            for _ in 0..256 {
                recipient
                    .enqueue(Message {
                        action: blockui_abi::key("increment"),
                        value: 0,
                    })
                    .unwrap();
            }
            runtime.deliver();
            assert_eq!(runtime.router.borrow().queue.len(), 1);
            let previous_lifecycle = runtime.actor(target).unwrap().lifecycle();
            assert!(runtime.remove(target));
            assert!(runtime.router.borrow().queue.is_empty());
            runtime.spawn(target, Path::new(&next)).unwrap();
            assert_ne!(
                runtime.actor(target).unwrap().lifecycle(),
                previous_lifecycle
            );
            assert!(runtime.tick().is_empty());
            assert_eq!(runtime.actor(target).unwrap().text("notes"), "");
            assert_eq!(runtime.actor(target).unwrap().integer("count"), 0);
            // Also discard payloads already admitted to the old lifecycle's mailbox.
            runtime
                .actors
                .get_mut(&source)
                .unwrap()
                .dispatch(send)
                .unwrap();
            runtime.deliver();
            assert_eq!(runtime.actor(target).unwrap().mailbox.len(), 1);
            assert!(runtime.remove(target));
            runtime.spawn(target, Path::new(&next)).unwrap();
            assert!(runtime.tick().is_empty());
            assert_eq!(runtime.actor(target).unwrap().text("notes"), "");
            // New messages intentionally address the current actor under the reused ID.
            let caller = runtime.actors.get_mut(&source).unwrap();
            caller
                .set_text(blockui_abi::key("notes"), "new lifecycle 😀")
                .unwrap();
            caller.dispatch(send).unwrap();
            assert!(runtime.tick().is_empty());
            assert_eq!(
                runtime.actor(target).unwrap().text("notes"),
                "v2:new lifecycle 😀"
            );
            assert!(runtime.remove(source));
            assert!(runtime.remove(target));
            assert!(runtime.router.borrow().queue.is_empty());
            assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
        }
        std::fs::remove_dir(root).unwrap();
    }

    #[test]
    #[ignore = "requires counter DLLs; run scripts/verify_requests.py"]
    fn real_sdk_text_tell_is_owned_transactional_bounded_and_uses_current_handler() {
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let next = std::env::var_os("ACTOR_TEST_REPLACEMENT").unwrap();
        let root = std::env::temp_dir().join(format!("actor-text-tell-{}", std::process::id()));
        {
            let mut runtime = Runtime::new(&root);
            runtime.spawn(ActorId(1), Path::new(&base)).unwrap();
            runtime.spawn(ActorId(2), Path::new(&base)).unwrap();
            let send = blockui_abi::key("send_text");
            let caller = runtime.actors.get_mut(&ActorId(1)).unwrap();
            caller
                .set_text(blockui_abi::key("notes"), "資料 😀")
                .unwrap();
            assert!(
                caller
                    .dispatch(Message {
                        action: send,
                        value: -2
                    })
                    .is_err()
            );
            assert!(runtime.router.borrow().queue.is_empty());
            runtime
                .actors
                .get_mut(&ActorId(1))
                .unwrap()
                .dispatch(Message {
                    action: send,
                    value: 2,
                })
                .unwrap();
            let target = runtime.actors.get_mut(&ActorId(2)).unwrap();
            let storage = target.slot_address("notes");
            for _ in 0..256 {
                target
                    .enqueue(Message {
                        action: blockui_abi::key("increment"),
                        value: 0,
                    })
                    .unwrap();
            }
            runtime.deliver();
            assert_eq!(runtime.router.borrow().queue.len(), 1);
            runtime.reload(ActorId(1), Path::new(&next)).unwrap();
            runtime.reload(ActorId(2), Path::new(&next)).unwrap();
            for _ in 0..10 {
                assert!(runtime.tick().is_empty());
            }
            let target = runtime.actor(ActorId(2)).unwrap();
            assert_eq!(target.text("notes"), "v2:資料 😀");
            assert_eq!(target.slot_address("notes"), storage);
            assert!(target.turn.message_text.is_none());
            assert!(runtime.router.borrow().queue.is_empty());
            for text in [
                String::new(),
                "x".repeat(blockui_abi::MAX_MESSAGE_TEXT_BYTES),
            ] {
                let caller = runtime.actors.get_mut(&ActorId(1)).unwrap();
                caller.set_text(blockui_abi::key("notes"), &text).unwrap();
                caller
                    .dispatch(Message {
                        action: send,
                        value: 2,
                    })
                    .unwrap();
                assert!(runtime.tick().is_empty());
                assert!(runtime.tick().is_empty());
                assert_eq!(
                    runtime.actor(ActorId(2)).unwrap().text("notes"),
                    format!("v2:{text}")
                );
            }
            let caller = runtime.actors.get_mut(&ActorId(1)).unwrap();
            caller
                .set_text(
                    blockui_abi::key("notes"),
                    &"x".repeat(blockui_abi::MAX_MESSAGE_TEXT_BYTES + 1),
                )
                .unwrap();
            assert!(
                caller
                    .dispatch(Message {
                        action: send,
                        value: 2
                    })
                    .is_err()
            );
            assert!(runtime.router.borrow().queue.is_empty());
            let target = runtime.actors.get_mut(&ActorId(2)).unwrap();
            let before = target.text("notes").to_owned();
            let mut rejected: crate::Envelope = Message {
                action: blockui_abi::key("text_message"),
                value: 0,
            }
            .into();
            rejected.message_text = Some("reject".into());
            assert!(target.enqueue_envelope(rejected).is_none());
            assert!(target.tick().unwrap_err().contains("text message rejected"));
            assert_eq!(target.text("notes"), before);
            assert!(target.turn.message_text.is_none());
            target
                .enqueue(Message {
                    action: blockui_abi::key("text_message"),
                    value: 0,
                })
                .unwrap();
            assert!(target.tick().is_err()); // scalar is not empty text
            target
                .enqueue(Message {
                    action: blockui_abi::key("increment"),
                    value: 0,
                })
                .unwrap();
            target.tick().unwrap(); // rejected text does not leak to the next turn
            let before = target.integer("count");
            let mut wrong: crate::Envelope = Message {
                action: blockui_abi::key("increment"),
                value: 0,
            }
            .into();
            wrong.message_text = Some(String::new());
            assert!(target.enqueue_envelope(wrong).is_none());
            assert!(target.tick().is_err());
            assert_eq!(target.integer("count"), before);
            assert!(target.turn.message_text.is_none());
        }
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn owned_route_envelopes_keep_metadata_and_order_under_backpressure() {
        let root =
            std::env::temp_dir().join(format!("actor-envelope-pressure-{}", std::process::id()));
        let id = ActorId(1);
        let mut runtime = Runtime::new(&root);
        let mut actor = Actor::new(id.0, &root).unwrap();
        for value in 0..256 {
            actor.enqueue(Message { action: 7, value }).unwrap();
        }
        runtime.actors.insert(id, actor);
        let detail = String::from("資料 😀");
        let allocation = detail.as_ptr();
        let mut first: crate::Envelope = Message {
            action: 8,
            value: 256,
        }
        .into();
        first.request_error = Some(detail);
        runtime
            .router
            .borrow_mut()
            .queue
            .push_back(Outgoing::Tell(id, first));
        runtime.router.borrow_mut().queue.push_back(Outgoing::Tell(
            id,
            Message {
                action: 9,
                value: 257,
            }
            .into(),
        ));
        for _ in 0..3 {
            runtime.deliver();
            assert_eq!(runtime.router.borrow().queue.len(), 2);
            let router = runtime.router.borrow();
            let Outgoing::Tell(_, first) = &router.queue[0] else {
                panic!("expected tell")
            };
            assert_eq!(first.request_error.as_ref().unwrap().as_ptr(), allocation);
            assert_eq!(first.message.value, 256);
        }
        let actor = runtime.actors.get_mut(&id).unwrap();
        actor.mailbox.pop_front();
        runtime.deliver();
        assert_eq!(runtime.router.borrow().queue.len(), 1);
        let actor = runtime.actors.get_mut(&id).unwrap();
        let first = actor.mailbox.back().unwrap();
        assert_eq!(first.message.value, 256);
        assert_eq!(first.request_error.as_deref(), Some("資料 😀"));
        assert_eq!(first.request_error.as_ref().unwrap().as_ptr(), allocation);
        actor.mailbox.pop_front();
        runtime.deliver();
        let actor = runtime.actors.get(&id).unwrap();
        assert_eq!(actor.mailbox.back().unwrap().message.value, 257);
        assert!(runtime.router.borrow().queue.is_empty());
        drop(runtime);
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    #[ignore = "requires counter DLLs; run scripts/verify_requests.py"]
    fn real_sdk_ask_error_detail_survives_full_mailbox_and_both_replacements() {
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let next = std::env::var_os("ACTOR_TEST_REPLACEMENT").unwrap();
        let root =
            std::env::temp_dir().join(format!("blockui-detail-pressure-{}", std::process::id()));
        {
            let mut runtime = Runtime::new(&root);
            runtime.spawn(ActorId(1), Path::new(&base)).unwrap();
            runtime.spawn(ActorId(2), Path::new(&base)).unwrap();
            let caller_storage = runtime.actor(ActorId(1)).unwrap().state_address();
            runtime
                .actors
                .get_mut(&ActorId(1))
                .unwrap()
                .dispatch(Message {
                    action: blockui_abi::key("ask_handler_fail"),
                    value: 2,
                })
                .unwrap();
            runtime.deliver_requests();
            for _ in 0..256 {
                runtime
                    .actors
                    .get_mut(&ActorId(1))
                    .unwrap()
                    .enqueue(Message {
                        action: blockui_abi::key("increment"),
                        value: 0,
                    })
                    .unwrap();
            }
            assert!(runtime.actors.get_mut(&ActorId(2)).unwrap().tick().is_err());
            runtime.deliver_requests();
            assert_eq!(runtime.sdk_waits.len(), 1);
            assert!(runtime.sdk_waits[0].reply.is_none());
            assert_eq!(
                runtime.sdk_waits[0].failure_detail.as_deref(),
                Some("unknown actor action")
            );
            assert_eq!(runtime.actor(ActorId(1)).unwrap().mailbox.len(), 256);
            runtime.reload(ActorId(1), Path::new(&next)).unwrap();
            runtime.reload(ActorId(2), Path::new(&next)).unwrap();
            for _ in 0..10 {
                assert!(runtime.tick().is_empty());
            }
            let caller = runtime.actor(ActorId(1)).unwrap();
            assert_eq!(caller.state_address(), caller_storage);
            assert_eq!(caller.integer("count"), 2560);
            assert_eq!(caller.integer("task_result"), -5);
            assert!(caller.turn.request_error.is_none());
            assert!(caller.mailbox.is_empty());
            assert!(runtime.sdk_waits.is_empty());
            assert_eq!(runtime.router.borrow().inflight, 0);
            assert_eq!(runtime.actor(ActorId(2)).unwrap().integer("count"), 0);
        }
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn host_task_api_rejects_unknown_actor_without_polling_future() {
        let mut runtime = Runtime::new("unused-task-admission-root");
        let polled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = polled.clone();
        let message = Message {
            action: 7,
            value: 0,
        };
        assert!(
            runtime
                .spawn_async(
                    ActorId(99),
                    1,
                    std::future::poll_fn(move |_| {
                        flag.store(true, std::sync::atomic::Ordering::Relaxed);
                        std::task::Poll::Ready(message)
                    }),
                    message,
                    crate::AsyncReloadPolicy::Preserve
                )
                .unwrap_err()
                .contains("unknown actor")
        );
        assert!(!polled.load(std::sync::atomic::Ordering::Relaxed));
        assert!(
            runtime
                .cancel_task(ActorId(99), 1)
                .unwrap_err()
                .contains("unknown actor")
        );
        assert!(runtime.actors.is_empty());
    }
    #[test]
    #[ignore = "requires built counter dylibs; run scripts/verify_requests.py"]
    fn real_sdk_ask_cancel_controls_release_on_source_termination() {
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let root =
            std::env::temp_dir().join(format!("blockui-cancel-retire-{}", std::process::id()));
        let message = |name, value| Message {
            action: blockui_abi::key(name),
            value,
        };
        {
            let mut runtime = Runtime::new(&root);
            let source = ActorId(1);
            let peer = ActorId(2);
            runtime.spawn(source, Path::new(&base)).unwrap();
            runtime.spawn(peer, Path::new(&base)).unwrap();
            let mut tokens = Vec::new();
            for _ in 0..256 {
                tokens.push(
                    runtime
                        .actors
                        .get_mut(&source)
                        .unwrap()
                        .dispatch_request(message("ask_token", 2))
                        .unwrap()
                        .unwrap(),
                );
            }
            for token in &tokens {
                runtime
                    .actors
                    .get_mut(&source)
                    .unwrap()
                    .dispatch(message("cancel_request", *token))
                    .unwrap();
            }
            assert_eq!(runtime.router.borrow().cancellations.len(), 256);
            assert!(runtime.remove(source));
            assert!(
                runtime.router.borrow().queue.is_empty(),
                "terminated source must release its control queue immediately"
            );
            assert!(runtime.router.borrow().cancellations.is_empty());
            assert_eq!(runtime.router.borrow().inflight, 0);
            // Reusing the actor ID creates a different lifecycle. Old request IDs
            // cannot cancel a new request, even before any intervening runtime tick.
            runtime.spawn(source, Path::new(&base)).unwrap();
            let fresh = runtime
                .actors
                .get_mut(&source)
                .unwrap()
                .dispatch_request(message("ask_token", 2))
                .unwrap()
                .unwrap();
            assert!(!tokens.contains(&fresh));
            runtime
                .actors
                .get_mut(&source)
                .unwrap()
                .dispatch(message("cancel_request", tokens[0]))
                .unwrap();
            for _ in 0..4 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(runtime.actor(peer).unwrap().integer("count"), 1);
            assert_eq!(runtime.actor(source).unwrap().integer("task_result"), 1);
            assert!(runtime.router.borrow().cancellations.is_empty());
            assert_eq!(runtime.router.borrow().inflight, 0);
        }
        assert!(std::fs::read_dir(&root).unwrap().next().is_none());
        std::fs::remove_dir(root).unwrap();
    }

    #[test]
    #[ignore = "requires built counter dylibs; run scripts/verify_requests.py"]
    fn real_sdk_ask_cancel_from_mailbox_prevents_later_target_turn() {
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let root =
            std::env::temp_dir().join(format!("blockui-cancel-mailbox-{}", std::process::id()));
        let message = |name, value| Message {
            action: blockui_abi::key(name),
            value,
        };
        for target in [ActorId(2), ActorId(1)] {
            let mut runtime = Runtime::new(&root);
            let source = ActorId(1);
            runtime.spawn(source, Path::new(&base)).unwrap();
            if target != source {
                runtime.spawn(target, Path::new(&base)).unwrap();
            }
            let token = runtime
                .actors
                .get_mut(&source)
                .unwrap()
                .dispatch_request(message("ask_token", target.0 as i64))
                .unwrap()
                .unwrap();
            if target == source {
                // Same actor: cancel is ahead of its own request in the mailbox.
                runtime
                    .tell(source, message("cancel_request", token))
                    .unwrap();
                runtime.deliver_requests();
            } else {
                runtime.deliver_requests();
                runtime
                    .tell(source, message("cancel_request", token))
                    .unwrap();
            }
            for _ in 0..4 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(
                runtime.actor(target).unwrap().integer("count"),
                0,
                "committed mailbox cancellation must prevent a later request turn"
            );
            assert_eq!(runtime.actor(source).unwrap().integer("task_result"), -4);
            assert_eq!(runtime.router.borrow().inflight, 0);
            assert!(runtime.sdk_waits.is_empty());
        }
        assert!(std::fs::read_dir(&root).unwrap().next().is_none());
        std::fs::remove_dir(root).unwrap();
    }

    #[test]
    #[ignore = "requires built counter dylibs; run scripts/verify_requests.py"]
    fn real_sdk_ask_cancel_remains_available_when_data_queue_is_full() {
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let next = std::env::var_os("ACTOR_TEST_REPLACEMENT").unwrap();
        let root = std::env::temp_dir().join(format!("blockui-cancel-full-{}", std::process::id()));
        let message = |name, value| Message {
            action: blockui_abi::key(name),
            value,
        };
        {
            let mut runtime = Runtime::new(&root);
            let source = ActorId(1);
            let target = ActorId(2);
            runtime.spawn(source, Path::new(&base)).unwrap();
            runtime.spawn(target, Path::new(&base)).unwrap();
            let storage = runtime.actor(source).unwrap().state_address();
            let mut tokens = Vec::new();
            for _ in 0..256 {
                tokens.push(
                    runtime
                        .actors
                        .get_mut(&source)
                        .unwrap()
                        .dispatch_request(message("ask_token", 2))
                        .unwrap()
                        .unwrap(),
                );
            }
            assert_eq!(runtime.router.borrow().queue.len(), 256);
            assert!(
                runtime
                    .actors
                    .get_mut(&source)
                    .unwrap()
                    .dispatch_request(message("ask_token", 2))
                    .is_err()
            );
            for token in &tokens {
                runtime
                    .actors
                    .get_mut(&source)
                    .unwrap()
                    .dispatch(message("cancel_request", *token))
                    .unwrap();
            }
            assert_eq!(runtime.router.borrow().queue.len(), 512);
            assert_eq!(runtime.router.borrow().cancellations.len(), 256);
            // Cancellation itself is bounded; the overflow command rolls back.
            assert!(
                runtime
                    .actors
                    .get_mut(&source)
                    .unwrap()
                    .dispatch(message("cancel_request", tokens[0]))
                    .is_err()
            );
            assert_eq!(runtime.router.borrow().queue.len(), 512);
            assert_eq!(runtime.router.borrow().cancellations.len(), 256);
            runtime.reload(source, Path::new(&next)).unwrap();
            for _ in 0..10 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(runtime.actor(target).unwrap().integer("count"), 0);
            assert_eq!(runtime.actor(source).unwrap().integer("task_result"), -4);
            assert_eq!(runtime.actor(source).unwrap().state_address(), storage);
            assert_eq!(runtime.router.borrow().inflight, 0);
            assert!(runtime.router.borrow().queue.is_empty());
            assert!(runtime.router.borrow().cancellations.is_empty());
            assert!(runtime.sdk_waits.is_empty());
            runtime
                .actors
                .get_mut(&source)
                .unwrap()
                .dispatch_request(message("ask_token", 2))
                .unwrap();
            for _ in 0..4 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(runtime.actor(target).unwrap().integer("count"), 1);
            assert_eq!(runtime.actor(source).unwrap().integer("task_result"), 1001);
        }
        assert!(std::fs::read_dir(&root).unwrap().next().is_none());
        std::fs::remove_dir(root).unwrap();
    }

    #[test]
    #[ignore = "requires built counter dylibs; run scripts/verify_requests.py"]
    fn real_sdk_ask_cancel_commits_with_turn_and_survives_reload() {
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let next = std::env::var_os("ACTOR_TEST_REPLACEMENT").unwrap();
        let root = std::env::temp_dir().join(format!("blockui-cancel-{}", std::process::id()));
        let message = |name, value| Message {
            action: blockui_abi::key(name),
            value,
        };
        {
            let mut runtime = Runtime::new(&root);
            for id in 1..=3 {
                runtime.spawn(ActorId(id), Path::new(&base)).unwrap();
            }
            let source = ActorId(1);
            let target = ActorId(2);
            runtime
                .actors
                .get_mut(&source)
                .unwrap()
                .dispatch(message("ask_cancel", 2))
                .unwrap();
            for _ in 0..4 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(runtime.actor(target).unwrap().integer("count"), 0);
            assert_eq!(runtime.actor(source).unwrap().integer("task_result"), -4);
            assert_eq!(runtime.router.borrow().inflight, 0);
            // Cancellation of a request already queued to the target still prevents its turn.
            let token = runtime
                .actors
                .get_mut(&source)
                .unwrap()
                .dispatch_request(message("ask_token", 2))
                .unwrap()
                .unwrap();
            runtime.deliver_requests();
            assert_eq!(runtime.sdk_waits.len(), 1);
            let storage = runtime.actor(source).unwrap().state_address();
            runtime.reload(source, Path::new(&next)).unwrap();
            runtime
                .actors
                .get_mut(&source)
                .unwrap()
                .dispatch(message("cancel_request", token))
                .unwrap();
            for _ in 0..4 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(runtime.actor(target).unwrap().integer("count"), 0);
            assert_eq!(runtime.actor(source).unwrap().integer("task_result"), -4);
            assert_eq!(runtime.actor(source).unwrap().state_address(), storage);
            // A failed cancelling turn has no effect; another actor cannot cancel this token.
            let token = runtime
                .actors
                .get_mut(&source)
                .unwrap()
                .dispatch_request(message("ask_token", 2))
                .unwrap()
                .unwrap();
            runtime.deliver_requests();
            assert!(
                runtime
                    .actors
                    .get_mut(&source)
                    .unwrap()
                    .dispatch(message("cancel_request_fail", token))
                    .is_err()
            );
            assert!(
                runtime.router.borrow().cancellations.is_empty(),
                "failed turns must not publish cancellation identities"
            );
            runtime
                .actors
                .get_mut(&ActorId(3))
                .unwrap()
                .dispatch(message("cancel_request", token))
                .unwrap();
            assert!(!runtime.router.borrow().cancelled((
                source,
                runtime.actor(source).unwrap().lifecycle(),
                token as u64
            )));
            assert!(runtime.router.borrow().cancelled((
                ActorId(3),
                runtime.actor(ActorId(3)).unwrap().lifecycle(),
                token as u64
            )));
            for _ in 0..4 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(runtime.actor(target).unwrap().integer("count"), 1);
            assert_eq!(runtime.actor(source).unwrap().integer("task_result"), 1001);
            assert_eq!(runtime.router.borrow().inflight, 0);
            assert!(runtime.sdk_waits.is_empty());
            // A committed target effect remains, while an undelivered completion
            // can be replaced by cancellation under source mailbox backpressure.
            let token = runtime
                .actors
                .get_mut(&source)
                .unwrap()
                .dispatch_request(message("ask_token", 2))
                .unwrap()
                .unwrap();
            runtime.deliver_requests();
            runtime.actors.get_mut(&target).unwrap().tick().unwrap();
            assert_eq!(runtime.actor(target).unwrap().integer("count"), 2);
            for _ in 0..256 {
                runtime
                    .actors
                    .get_mut(&source)
                    .unwrap()
                    .enqueue(message("increment", 0))
                    .unwrap();
            }
            runtime.deliver_requests();
            assert_eq!(runtime.sdk_waits.len(), 1);
            assert_eq!(runtime.sdk_waits[0].completion.unwrap().value, 2);
            runtime
                .actors
                .get_mut(&source)
                .unwrap()
                .dispatch(message("cancel_request", token))
                .unwrap();
            runtime.deliver();
            assert_eq!(
                runtime.sdk_waits[0].completion.unwrap().value,
                crate::RequestError::Cancelled.code()
            );
            runtime.reload(source, Path::new(&base)).unwrap();
            for _ in 0..10 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(runtime.actor(source).unwrap().integer("task_result"), -4);
            assert_eq!(runtime.actor(target).unwrap().integer("count"), 2);
            assert!(runtime.sdk_waits.is_empty());
            assert_eq!(runtime.router.borrow().inflight, 0);
            assert_eq!(runtime.actor(source).unwrap().state_address(), storage);
            // Once completion enters the mailbox, a later cancel cannot rewrite it.
            let token = runtime
                .actors
                .get_mut(&source)
                .unwrap()
                .dispatch_request(message("ask_token", 2))
                .unwrap()
                .unwrap();
            runtime.deliver_requests();
            runtime.actors.get_mut(&target).unwrap().tick().unwrap();
            runtime.deliver_requests();
            assert!(runtime.sdk_waits.is_empty());
            runtime
                .actors
                .get_mut(&source)
                .unwrap()
                .dispatch(message("cancel_request", token))
                .unwrap();
            for _ in 0..3 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(runtime.actor(source).unwrap().integer("task_result"), 3);
            assert_eq!(runtime.actor(target).unwrap().integer("count"), 3);
            assert_eq!(runtime.router.borrow().inflight, 0);
        }
        assert!(std::fs::read_dir(&root).unwrap().next().is_none());
        std::fs::remove_dir(root).unwrap();
    }

    #[test]
    #[ignore = "requires built counter dylibs; run scripts/verify_requests.py"]
    fn real_sdk_ask_waits_for_shared_tracker_capacity_without_reordering_or_recreating_state() {
        use std::time::Duration;
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let next = std::env::var_os("ACTOR_TEST_REPLACEMENT").unwrap();
        let root =
            std::env::temp_dir().join(format!("blockui-tracker-capacity-{}", std::process::id()));
        let message = |name, value| Message {
            action: blockui_abi::key(name),
            value,
        };
        {
            let mut runtime = Runtime::new(&root);
            for id in 1..=4 {
                runtime.spawn(ActorId(id), Path::new(&base)).unwrap();
            }
            let mut pending = Vec::new();
            // These ordinary actions commit without a reply, retaining all tracker slots.
            for _ in 0..256 {
                pending.push(
                    runtime
                        .ask(ActorId(3), message("increment", 0), Duration::from_secs(30))
                        .unwrap(),
                );
            }
            for _ in 0..8 {
                runtime.actors.get_mut(&ActorId(3)).unwrap().tick().unwrap();
            }
            assert!(runtime.actor(ActorId(3)).unwrap().mailbox.is_empty());
            assert!(
                runtime
                    .ask(
                        ActorId(3),
                        message("request_count", 0),
                        Duration::from_secs(30)
                    )
                    .err()
                    .unwrap()
                    .contains("pending request limit")
            );
            let storage = runtime.actor(ActorId(1)).unwrap().state_address();
            let slot = runtime
                .actor(ActorId(1))
                .unwrap()
                .slot_address("task_result");
            runtime
                .actors
                .get_mut(&ActorId(1))
                .unwrap()
                .dispatch(message("ask_order", 2))
                .unwrap();
            runtime
                .actors
                .get_mut(&ActorId(1))
                .unwrap()
                .dispatch(message("forward", 4))
                .unwrap();
            runtime.deliver_requests();
            assert_eq!(runtime.router.borrow().queue.len(), 2);
            assert!(
                runtime.actor(ActorId(2)).unwrap().mailbox.is_empty(),
                "the following tell must remain behind a request blocked by tracker capacity"
            );
            runtime.actors.get_mut(&ActorId(4)).unwrap().tick().unwrap();
            assert_eq!(
                runtime.actor(ActorId(4)).unwrap().integer("task_result"),
                99
            );
            runtime.reload(ActorId(1), Path::new(&next)).unwrap();
            runtime.reload(ActorId(2), Path::new(&next)).unwrap();
            assert!(runtime.cancel_request(pending[0].id()));
            runtime.deliver_requests();
            assert!(runtime.router.borrow().queue.is_empty());
            assert!(runtime.router.borrow().cancellations.is_empty());
            assert_eq!(runtime.actor(ActorId(2)).unwrap().mailbox.len(), 2);
            for _ in 0..4 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(runtime.actor(ActorId(2)).unwrap().integer("count"), 20);
            assert_eq!(
                runtime.actor(ActorId(1)).unwrap().integer("task_result"),
                1010,
                "queued request/reply must execute current code in sender order after capacity opens"
            );
            assert_eq!(runtime.actor(ActorId(1)).unwrap().state_address(), storage);
            assert_eq!(
                runtime
                    .actor(ActorId(1))
                    .unwrap()
                    .slot_address("task_result"),
                slot
            );
            assert_eq!(runtime.router.borrow().inflight, 0);
            assert!(runtime.sdk_waits.is_empty());
            assert!(runtime.remove(ActorId(3)));
            assert_eq!(runtime.next_request_deadline(), None);
            drop(pending);
            for id in [1, 2, 4] {
                assert_eq!(runtime.actor(ActorId(id)).unwrap().artifact_count(), 1);
            }
        }
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    #[ignore = "requires built counter dylibs; run scripts/verify_requests.py"]
    fn real_sdk_ask_and_tell_keep_sender_order() {
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let root = std::env::temp_dir().join(format!("blockui-order-{}", std::process::id()));
        {
            let mut runtime = Runtime::new(&root);
            runtime.spawn(ActorId(1), Path::new(&base)).unwrap();
            runtime.spawn(ActorId(2), Path::new(&base)).unwrap();
            runtime
                .actors
                .get_mut(&ActorId(1))
                .unwrap()
                .dispatch(Message {
                    action: blockui_abi::key("ask_order"),
                    value: 2,
                })
                .unwrap();
            for _ in 0..4 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(
                runtime.actor(ActorId(1)).unwrap().integer("task_result"),
                1,
                "ask must execute before the tell following it in the sender turn"
            );
            assert_eq!(runtime.actor(ActorId(2)).unwrap().integer("count"), 2);
            runtime
                .actors
                .get_mut(&ActorId(1))
                .unwrap()
                .dispatch(Message {
                    action: blockui_abi::key("ask_order"),
                    value: -2,
                })
                .unwrap();
            for _ in 0..4 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(runtime.actor(ActorId(1)).unwrap().integer("task_result"), 4);
            assert_eq!(runtime.actor(ActorId(2)).unwrap().integer("count"), 4);

            runtime.spawn(ActorId(3), Path::new(&base)).unwrap();
            for _ in 0..256 {
                runtime
                    .tell(
                        ActorId(2),
                        Message {
                            action: blockui_abi::key("task_done"),
                            value: 7,
                        },
                    )
                    .unwrap();
            }
            runtime
                .actors
                .get_mut(&ActorId(1))
                .unwrap()
                .dispatch(Message {
                    action: blockui_abi::key("ask_order"),
                    value: 2,
                })
                .unwrap();
            runtime
                .actors
                .get_mut(&ActorId(1))
                .unwrap()
                .dispatch(Message {
                    action: blockui_abi::key("forward"),
                    value: 3,
                })
                .unwrap();
            runtime.deliver_requests();
            assert_eq!(runtime.router.borrow().queue.len(), 2);
            runtime.actors.get_mut(&ActorId(3)).unwrap().tick().unwrap();
            assert_eq!(
                runtime.actor(ActorId(3)).unwrap().integer("task_result"),
                99,
                "one blocked target must not block another target's delivery"
            );
            runtime.actors.get_mut(&ActorId(2)).unwrap().tick().unwrap();
            runtime.deliver();
            assert_eq!(
                runtime.router.borrow().queue.len(),
                2,
                "a following tell must not overtake a deferred ask when space opens"
            );
            for _ in 0..10 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(runtime.actor(ActorId(1)).unwrap().integer("task_result"), 5);
            assert_eq!(runtime.actor(ActorId(2)).unwrap().integer("count"), 6);

            for _ in 0..256 {
                runtime
                    .tell(
                        ActorId(2),
                        Message {
                            action: blockui_abi::key("task_done"),
                            value: 7,
                        },
                    )
                    .unwrap();
            }
            runtime
                .actors
                .get_mut(&ActorId(1))
                .unwrap()
                .dispatch(Message {
                    action: blockui_abi::key("ask"),
                    value: 2,
                })
                .unwrap();
            runtime
                .actors
                .get_mut(&ActorId(1))
                .unwrap()
                .dispatch(Message {
                    action: blockui_abi::key("ask_timeout"),
                    value: 2,
                })
                .unwrap();
            runtime.deliver_requests();
            std::thread::sleep(std::time::Duration::from_millis(5));
            runtime.deliver_requests();
            runtime.actors.get_mut(&ActorId(1)).unwrap().tick().unwrap();
            assert_eq!(
                runtime.actor(ActorId(1)).unwrap().integer("task_result"),
                -1
            );
            assert!(
                runtime.next_request_deadline().unwrap() > std::time::Instant::now(),
                "expired request behind a blocked request must not leave a due deadline"
            );
            assert_eq!(runtime.router.borrow().queue.len(), 1);
            assert!(runtime.remove(ActorId(1)));
            assert_eq!(runtime.next_request_deadline(), None);
            assert_eq!(runtime.actor(ActorId(2)).unwrap().integer("count"), 6);
        }
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    #[ignore = "requires built counter dylibs; run scripts/verify_requests.py"]
    fn real_sdk_ask_transacts_survives_reload_and_obeys_lifecycle_capacity_and_deadlines() {
        use std::time::Duration;
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let next = std::env::var_os("ACTOR_TEST_REPLACEMENT").unwrap();
        let root = std::env::temp_dir().join(format!("blockui-ask-{}", std::process::id()));
        let message = |name, value| Message {
            action: blockui_abi::key(name),
            value,
        };
        {
            let mut runtime = Runtime::new(&root);
            runtime.spawn(ActorId(1), Path::new(&base)).unwrap();
            runtime.spawn(ActorId(2), Path::new(&base)).unwrap();
            for code in [i64::MIN, -1, 0, 6, i64::MAX] {
                assert!(
                    runtime
                        .actors
                        .get_mut(&ActorId(1))
                        .unwrap()
                        .dispatch(message("ask_error", code))
                        .is_err()
                );
                assert_eq!(
                    runtime.actor(ActorId(1)).unwrap().integer("task_result"),
                    0,
                    "unknown error codes must not invoke the application handler"
                );
            }
            let wakes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let wake_count = wakes.clone();
            runtime.set_wake(std::sync::Arc::new(move || {
                wake_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }));
            wakes.store(0, std::sync::atomic::Ordering::Relaxed); // Exclude callback-install backlog wakes.
            let count = runtime.actor(ActorId(2)).unwrap().integer("count");
            let storage = runtime.actor(ActorId(1)).unwrap().state_address();
            let slot = runtime
                .actor(ActorId(1))
                .unwrap()
                .slot_address("task_result");
            assert!(
                runtime
                    .actors
                    .get_mut(&ActorId(1))
                    .unwrap()
                    .dispatch(message("ask_fail", 2))
                    .is_err()
            );
            assert_eq!(
                wakes.load(std::sync::atomic::Ordering::Relaxed),
                0,
                "failed sender turn must not wake or publish a request"
            );
            assert_eq!(runtime.router.borrow().inflight, 0);
            assert!(runtime.router.borrow().queue.is_empty());
            assert!(runtime.router.borrow().cancellations.is_empty());
            runtime
                .actors
                .get_mut(&ActorId(1))
                .unwrap()
                .dispatch(message("ask", 2))
                .unwrap();
            assert_eq!(
                wakes.load(std::sync::atomic::Ordering::Relaxed),
                1,
                "committed request must wake an idle host before its deadline"
            );
            assert_eq!(runtime.actor(ActorId(2)).unwrap().integer("count"), count);
            assert!(runtime.tick().is_empty()); // Admit target envelope; neither action has run.
            runtime.reload(ActorId(1), Path::new(&next)).unwrap();
            runtime.reload(ActorId(2), Path::new(&next)).unwrap();
            for _ in 0..3 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(
                runtime.actor(ActorId(2)).unwrap().integer("count"),
                count + 10
            );
            assert_eq!(
                runtime.actor(ActorId(1)).unwrap().integer("task_result"),
                count + 1010
            );
            assert_eq!(runtime.actor(ActorId(1)).unwrap().state_address(), storage);
            assert_eq!(
                runtime
                    .actor(ActorId(1))
                    .unwrap()
                    .slot_address("task_result"),
                slot
            );
            assert_eq!(runtime.router.borrow().inflight, 0);

            runtime
                .actors
                .get_mut(&ActorId(1))
                .unwrap()
                .dispatch(message("ask_handler_fail", 2))
                .unwrap();
            assert!(runtime.tick().is_empty());
            assert_eq!(runtime.tick().len(), 1);
            let completion = runtime.actors[&ActorId(1)]
                .mailbox
                .iter()
                .find(|entry| entry.message.action == blockui_abi::key("ask_detail_error"))
                .unwrap();
            assert_eq!(
                completion.request_error.as_deref(),
                Some("unknown actor action")
            );
            runtime.reload(ActorId(1), Path::new(&base)).unwrap();
            runtime.reload(ActorId(2), Path::new(&base)).unwrap();
            runtime.reload(ActorId(1), Path::new(&next)).unwrap();
            runtime.reload(ActorId(2), Path::new(&next)).unwrap();
            assert!(runtime.tick().is_empty());
            assert_eq!(
                runtime.actor(ActorId(1)).unwrap().integer("task_result"),
                -5
            );
            assert_eq!(
                runtime.actor(ActorId(2)).unwrap().integer("count"),
                count + 10
            );
            runtime
                .actors
                .get_mut(&ActorId(1))
                .unwrap()
                .dispatch(message("ask_timeout", 2))
                .unwrap();
            assert!(runtime.next_request_deadline().is_some());
            std::thread::sleep(Duration::from_millis(5));
            for _ in 0..2 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(
                runtime.actor(ActorId(1)).unwrap().integer("task_result"),
                -1
            );
            assert_eq!(
                runtime.actor(ActorId(2)).unwrap().integer("count"),
                count + 10
            );

            runtime
                .actors
                .get_mut(&ActorId(1))
                .unwrap()
                .dispatch(message("ask", 2))
                .unwrap();
            assert!(runtime.remove(ActorId(2)));
            runtime.spawn(ActorId(2), Path::new(&base)).unwrap();
            for _ in 0..3 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(
                runtime.actor(ActorId(1)).unwrap().integer("task_result"),
                -2
            );
            assert_eq!(runtime.actor(ActorId(2)).unwrap().integer("count"), 0);
            runtime
                .actors
                .get_mut(&ActorId(1))
                .unwrap()
                .dispatch(message("ask", 2))
                .unwrap();
            assert!(runtime.tick().is_empty());
            assert!(runtime.remove(ActorId(1)));
            runtime.spawn(ActorId(1), Path::new(&base)).unwrap();
            for _ in 0..3 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(runtime.actor(ActorId(1)).unwrap().integer("task_result"), 0);
            assert_eq!(runtime.router.borrow().inflight, 0);

            // Target and caller mailbox backpressure retain requests/results without duplication.
            runtime
                .actors
                .get_mut(&ActorId(1))
                .unwrap()
                .dispatch(message("ask", 2))
                .unwrap();
            for _ in 0..256 {
                runtime.tell(ActorId(2), message("task_done", 7)).unwrap();
            }
            runtime.deliver_requests();
            assert!(runtime.sdk_waits.is_empty());
            assert_eq!(runtime.router.borrow().queue.len(), 1);
            assert_eq!(runtime.router.borrow().inflight, 1);
            runtime.actors.get_mut(&ActorId(2)).unwrap().tick().unwrap();
            runtime.deliver_requests();
            assert!(runtime.sdk_waits[0].reply.is_some());
            for _ in 0..8 {
                runtime.actors.get_mut(&ActorId(2)).unwrap().tick().unwrap();
            }
            for _ in 0..256 {
                runtime.tell(ActorId(1), message("task_done", 7)).unwrap();
            }
            runtime.deliver_requests();
            assert!(runtime.sdk_waits[0].completion.is_some());
            assert_eq!(runtime.router.borrow().inflight, 1);
            assert_eq!(runtime.actor(ActorId(1)).unwrap().mailbox.len(), 256);
            for _ in 0..10 {
                assert!(runtime.tick().is_empty());
            }
            assert_eq!(runtime.actor(ActorId(1)).unwrap().integer("task_result"), 1);
            assert_eq!(runtime.actor(ActorId(2)).unwrap().integer("count"), 1);
            assert_eq!(runtime.router.borrow().inflight, 0);

            for _ in 0..256 {
                runtime
                    .actors
                    .get_mut(&ActorId(1))
                    .unwrap()
                    .dispatch(message("ask", 2))
                    .unwrap();
            }
            assert!(
                runtime
                    .actors
                    .get_mut(&ActorId(1))
                    .unwrap()
                    .dispatch(message("ask", 2))
                    .is_err()
            );
            assert_eq!(runtime.router.borrow().inflight, 256);
            assert!(runtime.remove(ActorId(1)));
            assert!(runtime.tick().is_empty());
            assert_eq!(runtime.router.borrow().inflight, 0);
            assert!(runtime.sdk_waits.is_empty());
        }
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    #[ignore = "requires built counter dylibs; run scripts/verify_create.py"]
    fn real_runtime_shared_io_survives_reload_removal_and_identity_reuse() {
        use std::time::{Duration, Instant};
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let next = std::env::var_os("ACTOR_TEST_REPLACEMENT").unwrap();
        let root = std::env::temp_dir().join(format!("actor-shared-io-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        {
            let mut runtime = Runtime::new(root.join("snapshots"));
            for id in 1..=3 {
                runtime.spawn(ActorId(id), Path::new(&base)).unwrap();
                assert!(std::ptr::eq(
                    runtime.io_pool.as_ref(),
                    &*runtime.actor(ActorId(id)).unwrap().io_executor,
                ));
                let path = root.join(format!("document-{id}"));
                let actor = runtime.actor_mut(ActorId(id)).unwrap();
                actor
                    .set_text(blockui_abi::key("notes"), path.to_str().unwrap())
                    .unwrap();
                actor
                    .dispatch(Message {
                        action: blockui_abi::key("create_file"),
                        value: 0,
                    })
                    .unwrap();
                assert_eq!(actor.integer("task_result"), 0);
                assert_eq!(actor.io_worker_count(), 2);
            }
            let actor = runtime.actor(ActorId(1)).unwrap();
            let storage = actor.state_address();
            let slot = actor.slot_address("task_result");
            // Publish all files but deliberately do not deliver any actor completions.
            let deadline = Instant::now() + Duration::from_secs(3);
            while (1..=3).any(|id| !root.join(format!("document-{id}")).exists()) {
                assert!(
                    Instant::now() < deadline,
                    "shared I/O did not publish files"
                );
                std::thread::sleep(Duration::from_millis(2));
            }
            runtime.reload(ActorId(1), Path::new(&next)).unwrap();
            assert_eq!(runtime.actor(ActorId(1)).unwrap().integer("task_result"), 0);
            runtime.remove(ActorId(2));
            runtime.spawn(ActorId(2), Path::new(&base)).unwrap();
            let deadline = Instant::now() + Duration::from_secs(3);
            while runtime.actor(ActorId(1)).unwrap().integer("task_result") == 0
                || runtime.actor(ActorId(3)).unwrap().integer("task_result") == 0
            {
                assert!(runtime.tick().is_empty());
                assert!(Instant::now() < deadline, "shared I/O completion timeout");
                std::thread::sleep(Duration::from_millis(2));
            }
            let actor = runtime.actor(ActorId(1)).unwrap();
            assert_eq!(actor.integer("task_result"), 1001); // New DLL callback.
            assert_eq!(actor.state_address(), storage);
            assert_eq!(actor.slot_address("task_result"), slot);
            assert_eq!(actor.io_result_count(), 0);
            assert!(actor.tasks.is_empty());
            assert_eq!(runtime.actor(ActorId(3)).unwrap().integer("task_result"), 1);
            assert_eq!(runtime.actor(ActorId(2)).unwrap().integer("task_result"), 0);
            for id in 1..=3 {
                assert_eq!(runtime.actor(ActorId(id)).unwrap().io_worker_count(), 2);
                assert_eq!(
                    std::fs::read(root.join(format!("document-{id}"))).unwrap(),
                    "新文档😀".as_bytes()
                );
            }
        }
        assert_eq!(
            std::fs::read_dir(root.join("snapshots")).unwrap().count(),
            0
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[ignore = "requires built counter dylibs; run scripts/verify_requests.py"]
    fn real_sdk_ask_boolean_input_reloads_and_rejects_invalid_wire_values() {
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let next = std::env::var_os("ACTOR_TEST_REPLACEMENT").unwrap();
        let root = std::env::temp_dir().join(format!("actor-bool-request-{}", std::process::id()));
        {
            let mut runtime = Runtime::new(&root);
            runtime.spawn(ActorId(1), Path::new(&base)).unwrap();
            runtime.spawn(ActorId(2), Path::new(&base)).unwrap();
            let storage = runtime.actor(ActorId(1)).unwrap().state_address();
            for input in [1, 0] {
                runtime
                    .tell(
                        ActorId(1),
                        Message {
                            action: blockui_abi::key("ask_boolean"),
                            value: input,
                        },
                    )
                    .unwrap();
                assert!(runtime.tick().is_empty());
                runtime.reload(ActorId(1), Path::new(&next)).unwrap();
                runtime.reload(ActorId(2), Path::new(&next)).unwrap();
                for _ in 0..4 {
                    assert!(runtime.tick().is_empty());
                }
                assert_eq!(
                    runtime.actor(ActorId(2)).unwrap().integer("count"),
                    10 + input
                );
                assert_eq!(
                    runtime.actor(ActorId(1)).unwrap().integer("task_result"),
                    1000 + input
                );
                assert_eq!(runtime.actor(ActorId(1)).unwrap().state_address(), storage);
            }
            for invalid in [2, -1, i64::MAX] {
                runtime
                    .tell(
                        ActorId(2),
                        Message {
                            action: blockui_abi::key("boolean_query"),
                            value: invalid,
                        },
                    )
                    .unwrap();
                assert_eq!(runtime.tick().len(), 1);
                assert_eq!(runtime.actor(ActorId(2)).unwrap().integer("count"), 10);
            }
            runtime
                .tell(
                    ActorId(2),
                    Message {
                        action: blockui_abi::key("boolean_query"),
                        value: 1,
                    },
                )
                .unwrap();
            assert_eq!(runtime.tick().len(), 1); // A tell has no reply scope; attempted write rolls back.
            assert_eq!(runtime.actor(ActorId(2)).unwrap().integer("count"), 10);
        }
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    #[ignore = "requires built counter dylibs; run scripts/verify_requests.py"]
    fn real_runtime_shares_timer_worker_across_reload_and_removal() {
        use std::time::{Duration, Instant};
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let next = std::env::var_os("ACTOR_TEST_REPLACEMENT").unwrap();
        let root = std::env::temp_dir().join(format!("actor-shared-timer-{}", std::process::id()));
        {
            let mut runtime = Runtime::new(&root);
            let message = Message {
                action: blockui_abi::key("increment"),
                value: 0,
            };
            for id in 1..=64 {
                runtime.spawn(ActorId(id), Path::new(&base)).unwrap();
                runtime
                    .actor_mut(ActorId(id))
                    .unwrap()
                    .delay(1, Duration::from_secs(86_400), message)
                    .unwrap();
            }
            let worker = runtime.actor(ActorId(1)).unwrap().tasks[&1].worker_id();
            for id in 1..=64 {
                assert_eq!(
                    runtime.actor(ActorId(id)).unwrap().tasks[&1].worker_id(),
                    worker
                );
            }
            let storage = runtime.actor(ActorId(1)).unwrap().state_address();
            runtime.reload(ActorId(1), Path::new(&next)).unwrap();
            assert_eq!(
                runtime.actor(ActorId(1)).unwrap().tasks[&1].worker_id(),
                worker
            );
            assert_eq!(runtime.actor(ActorId(1)).unwrap().state_address(), storage);
            runtime.remove(ActorId(2));
            runtime.spawn(ActorId(2), Path::new(&next)).unwrap();
            runtime
                .actor_mut(ActorId(2))
                .unwrap()
                .delay(1, Duration::from_millis(5), message)
                .unwrap();
            assert_eq!(
                runtime.actor(ActorId(2)).unwrap().tasks[&1].worker_id(),
                worker
            );
            let deadline = Instant::now() + Duration::from_secs(2);
            while runtime.actor(ActorId(2)).unwrap().integer("count") == 0 {
                assert!(runtime.tick().is_empty());
                assert!(Instant::now() < deadline, "shared timer did not complete");
                std::thread::sleep(Duration::from_millis(2));
            }
            assert_eq!(runtime.actor(ActorId(2)).unwrap().integer("count"), 10);
            assert_eq!(runtime.actor(ActorId(1)).unwrap().integer("count"), 0);
            assert_eq!(
                runtime.actor(ActorId(3)).unwrap().tasks[&1].worker_id(),
                worker
            );
        }
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    #[ignore = "requires built counter dylibs; run scripts/verify_requests.py"]
    fn real_runtime_shares_future_worker_across_reload_and_actor_removal() {
        use std::{
            sync::{Arc, Mutex},
            task::{Poll, Waker},
            time::{Duration, Instant},
        };
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let next = std::env::var_os("ACTOR_TEST_REPLACEMENT").unwrap();
        let root = std::env::temp_dir().join(format!("actor-shared-future-{}", std::process::id()));
        {
            let mut runtime = Runtime::new(&root);
            runtime.spawn(ActorId(1), Path::new(&base)).unwrap();
            runtime.spawn(ActorId(2), Path::new(&base)).unwrap();
            let message = Message {
                action: blockui_abi::key("increment"),
                value: 0,
            };
            runtime
                .spawn_async(
                    ActorId(1),
                    7,
                    std::future::pending(),
                    message,
                    crate::AsyncReloadPolicy::Preserve,
                )
                .unwrap();
            let gate = Arc::new(Mutex::new((false, None::<Waker>)));
            let pending = gate.clone();
            runtime
                .spawn_async(
                    ActorId(2),
                    7,
                    std::future::poll_fn(move |cx| {
                        let mut gate = pending.lock().unwrap();
                        if gate.0 {
                            Poll::Ready(message)
                        } else {
                            gate.1 = Some(cx.waker().clone());
                            Poll::Pending
                        }
                    }),
                    message,
                    crate::AsyncReloadPolicy::Preserve,
                )
                .unwrap();
            let first = runtime.actor(ActorId(1)).unwrap().tasks[&7].worker_id();
            assert_eq!(
                first,
                runtime.actor(ActorId(2)).unwrap().tasks[&7].worker_id()
            );
            let deadline = Instant::now() + Duration::from_secs(5);
            while gate.lock().unwrap().1.is_none() {
                assert!(Instant::now() < deadline);
                std::thread::yield_now();
            }
            let actor = runtime.actor(ActorId(2)).unwrap();
            let storage = actor.state_address();
            let slot = actor.slot_address("count");
            let count = actor.integer("count");
            // Reject a candidate while the host future is parked. Rejection must
            // not cancel it, replace its executor, or disturb actor storage.
            let broken = root.join("invalid-candidate.so");
            std::fs::write(&broken, b"not a shared library").unwrap();
            assert!(runtime.reload(ActorId(2), &broken).is_err());
            std::fs::remove_file(&broken).unwrap();
            let retained = runtime.actor(ActorId(2)).unwrap();
            assert_eq!(retained.state_address(), storage);
            assert_eq!(retained.slot_address("count"), slot);
            assert_eq!(retained.integer("count"), count);
            assert_eq!(retained.tasks[&7].worker_id(), first);
            assert_eq!(retained.artifact_count(), 1);
            assert!(gate.lock().unwrap().1.is_some());
            runtime.reload(ActorId(2), Path::new(&next)).unwrap();
            assert!(runtime.remove(ActorId(1)));
            assert_eq!(
                first,
                runtime.actor(ActorId(2)).unwrap().tasks[&7].worker_id()
            );
            let waker = {
                let mut gate = gate.lock().unwrap();
                gate.0 = true;
                gate.1.take().unwrap()
            };
            waker.wake();
            while runtime.actor(ActorId(2)).unwrap().integer("count") == count {
                assert!(Instant::now() < deadline);
                assert!(runtime.tick().is_empty());
                std::thread::yield_now();
            }
            let actor = runtime.actor(ActorId(2)).unwrap();
            assert_eq!(actor.integer("count"), count + 10);
            assert_eq!(actor.state_address(), storage);
            assert_eq!(actor.slot_address("count"), slot);
            assert!(actor.tasks.is_empty());
            assert_eq!(actor.artifact_count(), 1);
            assert!(runtime.remove(ActorId(2)));
            runtime.spawn(ActorId(1), Path::new(&base)).unwrap();
            runtime
                .spawn_async(
                    ActorId(1),
                    7,
                    std::future::pending(),
                    message,
                    crate::AsyncReloadPolicy::Preserve,
                )
                .unwrap();
            assert_eq!(
                first,
                runtime.actor(ActorId(1)).unwrap().tasks[&7].worker_id()
            );
            for id in 2..=64 {
                runtime.spawn(ActorId(id), Path::new(&base)).unwrap();
                runtime
                    .spawn_async(
                        ActorId(id),
                        7,
                        std::future::pending(),
                        message,
                        crate::AsyncReloadPolicy::Preserve,
                    )
                    .unwrap();
                assert_eq!(
                    first,
                    runtime.actor(ActorId(id)).unwrap().tasks[&7].worker_id()
                );
            }
            assert_eq!(runtime.len(), 64);
        }
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    #[ignore = "requires built counter dylibs; run scripts/verify_requests.py"]
    fn real_runtime_retains_waiters_on_reload_and_terminates_old_lifecycles() {
        use crate::request::RequestError;
        use std::{
            future::Future,
            pin::Pin,
            task::{Context, Poll, Waker},
            time::{Duration, Instant},
        };
        fn poll(reply: &mut crate::request::Reply) -> Poll<Result<i64, RequestError>> {
            Pin::new(reply).poll(&mut Context::from_waker(Waker::noop()))
        }
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let next = std::env::var_os("ACTOR_TEST_REPLACEMENT").unwrap();
        let root =
            std::env::temp_dir().join(format!("actor-request-lifecycle-{}", std::process::id()));
        {
            let mut queued = Runtime::new(root.join("queued"));
            let target = ActorId(8);
            queued.spawn(target, Path::new(&base)).unwrap();
            let message = |name| Message {
                action: blockui_abi::key(name),
                value: 0,
            };
            queued.tell(target, message("increment")).unwrap();
            let mut reply = queued
                .ask(target, message("request_count"), Duration::from_secs(30))
                .unwrap();
            assert_eq!(poll(&mut reply), Poll::Pending);
            queued.reload(target, Path::new(&next)).unwrap();
            assert!(queued.tick().is_empty());
            assert_eq!(poll(&mut reply), Poll::Ready(Ok(20)));
            assert_eq!(queued.actor(target).unwrap().integer("count"), 20);
            let mut cancelled = queued
                .ask(target, message("request_count"), Duration::from_secs(30))
                .unwrap();
            assert!(queued.cancel_request(cancelled.id()));
            assert_eq!(
                poll(&mut cancelled),
                Poll::Ready(Err(RequestError::Cancelled))
            );
            let abandoned = queued
                .ask(target, message("request_count"), Duration::from_secs(30))
                .unwrap();
            drop(abandoned);
            assert!(queued.tick().is_empty());
            assert_eq!(queued.actor(target).unwrap().integer("count"), 20);
            let mut failed = queued
                .ask(target, message("request_fail"), Duration::from_secs(30))
                .unwrap();
            assert_eq!(queued.tick().len(), 1);
            assert_eq!(
                poll(&mut failed),
                Poll::Ready(Err(RequestError::HandlerFailed))
            );
            assert_eq!(
                failed.failure_detail().as_deref(),
                Some("unknown actor action")
            );
            queued.reload(target, Path::new(&base)).unwrap();
            assert_eq!(
                failed.failure_detail().as_deref(),
                Some("unknown actor action")
            );
            assert_eq!(queued.actor(target).unwrap().integer("count"), 20);
            let mut expired = queued
                .ask(target, message("request_count"), Duration::from_millis(1))
                .unwrap();
            std::thread::sleep(Duration::from_millis(2));
            assert!(queued.tick().is_empty());
            assert_eq!(poll(&mut expired), Poll::Ready(Err(RequestError::Timeout)));
            assert_eq!(queued.actor(target).unwrap().integer("count"), 20);
            let awaited = queued
                .ask(target, message("request_count"), Duration::from_secs(30))
                .unwrap();
            queued
                .spawn_async(
                    target,
                    90,
                    async move {
                        Message {
                            action: blockui_abi::key("task_done"),
                            value: awaited.await.unwrap(),
                        }
                    },
                    Message {
                        action: blockui_abi::key("task_done"),
                        value: -1,
                    },
                    crate::AsyncReloadPolicy::Preserve,
                )
                .unwrap();
            // The runtime-owned future is awaiting a still-queued request while
            // both implementation generations are replaced before target delivery.
            let storage = queued.actor(target).unwrap().state_address();
            let result_slot = queued.actor(target).unwrap().slot_address("task_result");
            queued.reload(target, Path::new(&base)).unwrap();
            queued.reload(target, Path::new(&next)).unwrap();
            assert_eq!(queued.actor(target).unwrap().state_address(), storage);
            assert_eq!(
                queued.actor(target).unwrap().slot_address("task_result"),
                result_slot
            );
            let finish = Instant::now() + Duration::from_secs(2);
            while queued.actor(target).unwrap().integer("task_result") != 30 {
                assert!(Instant::now() < finish, "host ask await timed out");
                assert!(queued.tick().is_empty());
                std::thread::sleep(Duration::from_millis(1));
            }
            assert_eq!(queued.next_request_deadline(), None);
            for _ in 0..256 {
                queued.tell(target, message("increment")).unwrap();
            }
            assert!(
                queued
                    .ask(target, message("request_count"), Duration::from_secs(30))
                    .is_err()
            );
            assert_eq!(queued.next_request_deadline(), None);
            for _ in 0..8 {
                assert!(queued.tick().is_empty());
            }
            let mut stopped = queued
                .ask(target, message("request_count"), Duration::from_secs(30))
                .unwrap();
            drop(queued);
            assert_eq!(
                poll(&mut stopped),
                Poll::Ready(Err(RequestError::RuntimeStopped))
            );
        }
        let mut runtime = Runtime::new(&root);
        let id = ActorId(1);
        runtime.spawn(id, Path::new(&base)).unwrap();
        let lifecycle = runtime.actor(id).unwrap().lifecycle();
        let slot = runtime.actor(id).unwrap().slot_address("count");
        // Reply staging does not publish when the handler fails or replies twice.
        let actor = runtime.actor_mut(id).unwrap();
        let message = |name| Message {
            action: blockui_abi::key(name),
            value: 0,
        };
        assert!(actor.dispatch(message("request_count")).is_err());
        assert_eq!(actor.integer("count"), 0);
        assert!(actor.dispatch_request(message("request_fail")).is_err());
        assert!(
            actor
                .dispatch_request(message("request_duplicate"))
                .is_err()
        );
        assert_eq!(actor.integer("count"), 0);
        actor.validator = Some(Box::new(|_| Err("reject request view admission".into())));
        assert!(actor.dispatch_request(message("request_count")).is_err());
        assert_eq!(actor.integer("count"), 0);
        actor.validator = None;
        assert_eq!(
            actor.dispatch_request(message("request_count")).unwrap(),
            Some(1)
        );
        assert_eq!(actor.dispatch_request(message("increment")).unwrap(), None);
        assert_eq!(actor.integer("count"), 2);
        let deadline = Instant::now() + Duration::from_secs(30);
        let (request, mut reply) = runtime
            .requests
            .borrow_mut()
            .admit(id, lifecycle, deadline)
            .unwrap();
        assert!(runtime.reload(id, &root.join("missing.so")).is_err());
        assert_eq!(poll(&mut reply), Poll::Pending);
        runtime.reload(id, Path::new(&next)).unwrap();
        assert_eq!(runtime.actor(id).unwrap().lifecycle(), lifecycle);
        assert_eq!(runtime.actor(id).unwrap().slot_address("count"), slot);
        assert_eq!(poll(&mut reply), Poll::Pending);
        assert!(
            runtime
                .requests
                .borrow_mut()
                .resolve(request, id, lifecycle, 42, Instant::now())
        );
        assert_eq!(poll(&mut reply), Poll::Ready(Ok(42)));
        let (old, mut terminated) = runtime
            .requests
            .borrow_mut()
            .admit(id, lifecycle, deadline)
            .unwrap();
        assert!(runtime.remove(id));
        assert_eq!(
            poll(&mut terminated),
            Poll::Ready(Err(RequestError::TargetTerminated))
        );
        runtime.spawn(id, Path::new(&next)).unwrap();
        let new_lifecycle = runtime.actor(id).unwrap().lifecycle();
        assert_ne!(new_lifecycle, lifecycle);
        assert!(
            !runtime
                .requests
                .borrow_mut()
                .resolve(old, id, new_lifecycle, 99, Instant::now())
        );
        let (_, mut timed_out) = runtime
            .requests
            .borrow_mut()
            .admit(id, new_lifecycle, Instant::now())
            .unwrap();
        assert!(runtime.tick().is_empty());
        assert_eq!(
            poll(&mut timed_out),
            Poll::Ready(Err(RequestError::Timeout))
        );
        let (_, mut stopped) = runtime
            .requests
            .borrow_mut()
            .admit(id, new_lifecycle, deadline)
            .unwrap();
        drop(runtime);
        assert_eq!(
            poll(&mut stopped),
            Poll::Ready(Err(RequestError::RuntimeStopped))
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[ignore = "requires a built counter dylib; see docs/multi-actor.md"]
    fn real_actors_route_reload_isolate_faults_and_dispose() {
        let library = std::env::var_os("ACTOR_TEST_LIBRARY")
            .expect("build counter and set ACTOR_TEST_LIBRARY");
        let root = std::env::temp_dir().join(format!("actor-system-test-{}", std::process::id()));
        let mut runtime = Runtime::new(&root);
        let a = ActorId(1);
        let b = ActorId(2);
        runtime.spawn(a, Path::new(&library)).unwrap();
        runtime.spawn(b, Path::new(&library)).unwrap();
        assert!(runtime.spawn(a, Path::new(&library)).is_err());
        runtime
            .tell(
                a,
                Message {
                    action: blockui_abi::key("forward"),
                    value: 2,
                },
            )
            .unwrap();
        assert!(runtime.tick().is_empty());
        runtime.reload(b, Path::new(&library)).unwrap();
        assert!(runtime.tick().is_empty());
        assert_eq!(runtime.actor(b).unwrap().integer("task_result"), 99);
        runtime
            .tell(
                a,
                Message {
                    action: blockui_abi::key("forward_fail"),
                    value: 2,
                },
            )
            .unwrap();
        assert_eq!(runtime.tick().len(), 1);
        assert!(runtime.tick().is_empty());
        assert_eq!(runtime.actor(b).unwrap().integer("task_result"), 99);
        runtime
            .tell(
                a,
                Message {
                    action: blockui_abi::key("forward"),
                    value: 777,
                },
            )
            .unwrap();
        assert_eq!(runtime.tick().len(), 1);

        let address = runtime.actor(b).unwrap().slot_address("count");
        let message = Message {
            action: blockui_abi::key("increment"),
            value: 0,
        };
        runtime.tell(b, message).unwrap();
        runtime.reload(b, Path::new(&library)).unwrap();
        assert_eq!(runtime.actor(b).unwrap().slot_address("count"), address);
        assert!(runtime.tick().is_empty());
        assert_eq!(runtime.actor(a).unwrap().integer("count"), 0);
        assert_eq!(runtime.actor(b).unwrap().integer("count"), 1);
        runtime
            .tell(
                a,
                Message {
                    action: u64::MAX,
                    value: 0,
                },
            )
            .unwrap();
        runtime.tell(b, message).unwrap();
        let faults = runtime.tick();
        assert_eq!(faults.len(), 1);
        assert_eq!(faults[0].actor, a);
        assert_eq!(runtime.actor(b).unwrap().integer("count"), 2);
        for _ in 0..256 {
            runtime.tell(b, message).unwrap();
        }
        assert!(runtime.tell(b, message).is_err());
        assert!(runtime.remove(b));
        assert!(runtime.tell(b, message).is_err());
        runtime.spawn(b, Path::new(&library)).unwrap();
        assert!(runtime.tick().is_empty());
        assert_eq!(runtime.actor(b).unwrap().integer("count"), 0);
        let child = ActorId(3);
        let grandchild = ActorId(4);
        let wake_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let count = wake_count.clone();
        runtime.set_wake(std::sync::Arc::new(move || {
            count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }));
        runtime.spawn_child(a, child, Path::new(&library)).unwrap();
        runtime
            .spawn_child(child, grandchild, Path::new(&library))
            .unwrap();
        assert_eq!(runtime.parent(grandchild), Some(child));
        assert!(
            runtime
                .spawn_child(ActorId(999), ActorId(5), Path::new(&library))
                .is_err()
        );
        assert!(
            runtime
                .spawn_child(a, ActorId(5), Path::new("/missing/actor.so"))
                .is_err()
        );
        assert!(runtime.actor(ActorId(5)).is_none());
        // Child admission also wakes the window; measure only these two tells.
        let wakes_before_tell = wake_count.load(std::sync::atomic::Ordering::SeqCst);
        runtime.tell(child, message).unwrap();
        runtime.tell(b, message).unwrap();
        assert_eq!(
            wake_count.load(std::sync::atomic::Ordering::SeqCst),
            wakes_before_tell + 2
        );
        let child_address = runtime.actor(child).unwrap().slot_address("count");
        let replacement = ActorId(6);
        let library_path = Path::new(&library);
        assert!(
            runtime
                .reconcile_children(
                    a,
                    &[
                        (replacement, library_path),
                        (ActorId(7), Path::new("/missing/actor.so"))
                    ]
                )
                .is_err()
        );
        assert!(runtime.actor(replacement).is_none());
        assert!(runtime.actor(child).is_some());
        assert!(runtime.actor(grandchild).is_some());
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 4);
        assert!(
            runtime
                .reconcile_children(a, &[(child, library_path), (child, library_path)])
                .is_err()
        );
        assert!(runtime.reconcile_children(a, &[(b, library_path)]).is_err());
        runtime
            .reconcile_children(a, &[(replacement, library_path), (child, library_path)])
            .unwrap();
        runtime
            .reconcile_children(a, &[(child, library_path), (replacement, library_path)])
            .unwrap();
        assert_eq!(
            runtime.actor(child).unwrap().slot_address("count"),
            child_address
        );
        assert!(runtime.actor(grandchild).is_some());
        runtime
            .reconcile_children(a, &[(child, library_path)])
            .unwrap();
        assert!(runtime.actor(replacement).is_none());

        runtime.reload(a, Path::new(&library)).unwrap();
        assert_eq!(
            runtime.actor(child).unwrap().slot_address("count"),
            child_address
        );
        assert!(runtime.tick().is_empty());
        assert_eq!(runtime.actor(child).unwrap().integer("count"), 1);
        // Runtime wake installation also reaches host tasks, including future children.
        let (wake_tx, wake_rx) = std::sync::mpsc::channel();
        runtime.set_wake(std::sync::Arc::new(move || {
            let _ = wake_tx.send(());
        }));
        runtime
            .actors
            .get_mut(&grandchild)
            .unwrap()
            .delay(1, std::time::Duration::from_millis(1), message)
            .unwrap();
        // Installing a wake callback may notify already-pending work. A wake is
        // a hint to poll, not proof that this particular timer has completed.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while runtime.actor(grandchild).unwrap().integer("count") == 0 {
            wake_rx
                .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
                .unwrap();
            assert!(runtime.tick().is_empty());
        }
        assert_eq!(runtime.actor(grandchild).unwrap().integer("count"), 1);
        // Removal also discards committed routes that haven't reached a mailbox yet.
        runtime
            .router
            .borrow_mut()
            .queue
            .push_back(Outgoing::Tell(grandchild, message.into()));
        assert!(runtime.remove(a));
        assert!(runtime.actor(child).is_none());
        assert!(runtime.actor(grandchild).is_none());
        assert!(runtime.router.borrow().queue.is_empty());
        assert_eq!(runtime.len(), 1);
        assert!(runtime.actor(b).is_some());
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
        runtime.spawn(a, Path::new(&library)).unwrap();
        assert_eq!(runtime.parent(a), None);
        runtime.spawn_child(a, child, library_path).unwrap();
        let snapshot = runtime.compose(a).unwrap();
        let old_input = snapshot
            .widget(child, blockui_abi::key("counter.notes"))
            .unwrap();
        runtime.remove(child);
        runtime.spawn_child(a, child, library_path).unwrap();
        runtime.compose(a).unwrap();
        assert!(snapshot.set_text(&mut runtime, old_input, "stale").is_err());
        assert_eq!(runtime.actor(child).unwrap().text("notes"), "");

        drop(runtime);
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
        std::fs::remove_dir(root).unwrap();
    }
}
