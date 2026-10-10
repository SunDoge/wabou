//! Declarative child ownership: stage whole new subtrees before committing a parent.
use crate::{
    Actor, ActorId,
    admission::{Admission, Validator},
};
use blockui_abi::{CHILD, Node};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::PathBuf,
    rc::{Rc, Weak},
};

#[derive(Clone)]
pub(crate) enum ImplementationSource {
    Library(PathBuf),
    #[cfg(feature = "testing")]
    Compiled(fn(&mut Actor) -> Result<(), String>),
}
impl ImplementationSource {
    fn initialize(&self, actor: &mut Actor) -> Result<(), String> {
        match self {
            Self::Library(path) => actor.reload(path),
            #[cfg(feature = "testing")]
            Self::Compiled(initialize) => initialize(actor),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Child {
    pub id: ActorId,
    pub implementation: u64,
}
pub(crate) struct Pending {
    pub additions: Vec<(ActorId, Actor)>,
    pub removals: Vec<ActorId>,
}
pub(crate) struct Registry {
    pub timer_pool: std::sync::Arc<crate::timer::Pool>,
    pub io_pool: std::sync::Arc<crate::io::Pool>,
    pub root: PathBuf,
    pub implementations: BTreeMap<u64, (String, ImplementationSource)>,
    pub slots: BTreeMap<(ActorId, u64), Child>,
    pub live: BTreeSet<ActorId>,
    pub resident: usize,
    pub staged: usize,
    pub pending: VecDeque<Pending>,
    pub accepted: BTreeMap<u64, Rc<Snapshot>>,
}
impl Registry {
    pub fn new(
        root: PathBuf,
        io_pool: std::sync::Arc<crate::io::Pool>,
        timer_pool: std::sync::Arc<crate::timer::Pool>,
    ) -> Self {
        Self {
            timer_pool,
            io_pool,
            root,
            implementations: BTreeMap::new(),
            slots: BTreeMap::new(),
            live: BTreeSet::new(),
            resident: 0,
            staged: 0,
            pending: VecDeque::new(),
            accepted: BTreeMap::new(),
        }
    }
    pub fn retire(&mut self, id: ActorId) {
        self.live.remove(&id);
        self.slots
            .retain(|(owner, _), child| *owner != id && child.id != id);
    }
}
struct Noop;
impl Admission for Noop {
    fn commit(self: Box<Self>) {}
}
struct Plan {
    registry: Rc<RefCell<Registry>>,
    updates: Vec<(ActorId, BTreeMap<u64, Child>)>,
    additions: Vec<(ActorId, Actor)>,
    removals: Vec<ActorId>,
    reserved: usize,
}
impl Drop for Plan {
    fn drop(&mut self) {
        if self.reserved != 0 {
            self.registry.borrow_mut().staged -= self.reserved;
        }
    }
}
impl Admission for Plan {
    fn commit(mut self: Box<Self>) {
        let registry_handle = self.registry.clone();
        let mut registry = registry_handle.borrow_mut();
        registry.staged -= self.reserved;
        self.reserved = 0;
        for (parent, desired) in std::mem::take(&mut self.updates) {
            registry.slots.retain(|(owner, _), _| *owner != parent);
            registry.slots.extend(
                desired
                    .into_iter()
                    .map(|(slot, child)| ((parent, slot), child)),
            );
        }
        for id in &self.removals {
            registry.retire(*id);
        }
        for (_, actor) in &self.additions {
            registry.live.insert(ActorId(actor.id));
        }
        registry.resident += self.additions.len();
        if !self.additions.is_empty() || !self.removals.is_empty() {
            registry.pending.push_back(Pending {
                additions: std::mem::take(&mut self.additions),
                removals: std::mem::take(&mut self.removals),
            });
        }
    }
}
fn next_id(registry: &Registry) -> Result<ActorId, String> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1 << 63);
    loop {
        let id = NEXT
            .try_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |id| id.checked_add(1),
            )
            .map_err(|_| "child identity exhausted")?;
        if !registry.live.contains(&ActorId(id)) {
            return Ok(ActorId(id));
        }
    }
}
fn removed_subtree(registry: &Registry, id: ActorId, output: &mut BTreeSet<ActorId>) {
    if !output.insert(id) {
        return;
    }
    for ((owner, _), child) in &registry.slots {
        if *owner == id {
            removed_subtree(registry, child.id, output);
        }
    }
}
fn stage(
    registry: &Registry,
    weak: &Weak<RefCell<Registry>>,
    parent: ActorId,
    nodes: &[Node],
    plan: &mut Plan,
) -> Result<(), String> {
    let mut desired = BTreeMap::new();
    for node in nodes.iter().filter(|node| node.kind == CHILD) {
        let child = if let Some(child) = registry.slots.get(&(parent, node.key)) {
            if child.implementation != node.action {
                return Err(
                    "changing child implementation identity requires a new child key".into(),
                );
            }
            *child
        } else {
            let source = registry
                .implementations
                .get(&node.action)
                .ok_or("unknown child implementation")?
                .1
                .clone();
            if registry.resident + registry.staged + plan.additions.len() >= 64 {
                return Err("child staging capacity reached".into());
            }
            let id = next_id(registry)?;
            let mut actor = Actor::new(id.0, &registry.root)?;
            actor.io_executor.use_pool(registry.io_pool.clone());
            actor.timer_executor.use_pool(registry.timer_pool.clone());
            // Initial child code may declare descendants; the builder validates those
            // recursively before publishing any of the staged actors.
            actor.validator = Some(Box::new(|_| Ok(Box::new(Noop))));
            source.initialize(&mut actor)?;
            let child_nodes: Vec<_> = actor
                .view
                .order
                .iter()
                .map(|key| actor.view.bindings[key].node)
                .collect();
            actor.turn.child_scope = Some((id.0, weak.clone()));
            actor.validator = Some(validator(weak.clone(), id));
            plan.additions.push((parent, actor));
            stage(registry, weak, id, &child_nodes, plan)?;
            Child {
                id,
                implementation: node.action,
            }
        };
        if desired.insert(node.key, child).is_some() {
            return Err("duplicate child key".into());
        }
    }
    let old: BTreeMap<_, _> = registry
        .slots
        .iter()
        .filter_map(|((owner, slot), child)| (*owner == parent).then_some((*slot, *child)))
        .collect();
    if old != desired {
        let mut removed = BTreeSet::new();
        for (slot, child) in old {
            if !desired.contains_key(&slot) {
                removed_subtree(registry, child.id, &mut removed);
            }
        }
        plan.removals.extend(removed);
        plan.updates.push((parent, desired));
    }
    Ok(())
}
pub(crate) fn validator(weak: Weak<RefCell<Registry>>, parent: ActorId) -> Validator {
    Box::new(move |nodes| {
        let registry = weak.upgrade().ok_or("child runtime has ended")?;
        let mut plan = Plan {
            registry: registry.clone(),
            updates: Vec::new(),
            additions: Vec::new(),
            removals: Vec::new(),
            reserved: 0,
        };
        stage(&registry.borrow(), &weak, parent, nodes, &mut plan)?;
        plan.reserved = plan.additions.len();
        registry.borrow_mut().staged += plan.reserved;
        Ok(Box::new(plan))
    })
}

/// Registry-owned code bytes, independent of any individual actor lifetime.
pub(crate) struct Snapshot(pub PathBuf);
impl Snapshot {
    pub fn new(root: &std::path::Path, source: &std::path::Path) -> Result<Self, String> {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let generation = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let bytes = crate::artifact::read(source)?;
        std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
        let path = root.join(format!("accepted-{}-{generation}.so", std::process::id()));
        crate::artifact::write(&path, &bytes)?;
        Ok(Self(path))
    }
}
impl Drop for Snapshot {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
