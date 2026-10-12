//! Implementation-local captures. This registry must be disposed before dlclose.
//! It owns no host context; dispatch receives a fresh context for each event.
use crate::{Cx, Error, Result};
use std::{collections::BTreeMap, rc::Rc};

pub type Callback = dyn for<'cx, 'host> Fn(&'cx Cx<'host>, i64) -> Result<()>;

/// Transactional implementation-local callback storage. The host's implementation
/// lifetime hooks commit/abort projection changes and dispose captures before unload.
/// Captures must be 'static data/handles, never a borrowed view or turn context.
///
/// ```compile_fail
/// use blockui::{Cx, __events::Events};
/// fn capture_context(events: &mut Events, cx: &Cx<'_>) {
///     events.insert(1, move |_, _| { let _ = cx.monotonic_ms()?; Ok(()) }).unwrap();
/// }
/// ```
#[derive(Clone)]
enum ReactiveCallback {
    Style(Rc<crate::StyleCompute>),
    Text(Rc<crate::TextCompute>),
    Scope(u64, Rc<crate::ScopeCompute>, Option<Rc<()>>),
}
#[doc(hidden)]
#[derive(Default)]
pub struct Events {
    registry: Registry<Rc<Callback>>,
    scopes: BTreeMap<u64, u64>,
    candidate_scopes: Option<BTreeMap<u64, u64>>,
    reactive: BTreeMap<usize, (u64, ReactiveCallback)>,
    candidate_reactive: Option<BTreeMap<usize, (u64, ReactiveCallback)>>,
    next_reactive: usize,
}
impl Events {
    pub fn begin(&mut self) -> Result<()> {
        if self.registry.candidate.is_none() {
            self.registry.candidate = Some(self.registry.active.clone());
            self.candidate_scopes = Some(self.scopes.clone());
            self.candidate_reactive = Some(self.reactive.clone());
        }
        Ok(())
    }
    fn reset_scope(&mut self, scope: u64) -> Result<()> {
        self.begin()?;
        let owners = self.candidate_scopes.as_mut().unwrap();
        self.registry
            .candidate
            .as_mut()
            .unwrap()
            .retain(|id, _| owners.get(id) != Some(&scope));
        owners.retain(|_, owner| *owner != scope);
        self.candidate_reactive
            .as_mut()
            .unwrap()
            .retain(|_, (owner, _)| *owner != scope);
        Ok(())
    }
    fn insert_scoped(&mut self, scope: u64, id: u64, callback: Rc<Callback>) -> Result<()> {
        self.registry.insert(id, callback)?;
        self.candidate_scopes.as_mut().unwrap().insert(id, scope);
        Ok(())
    }
    fn retain(&mut self, live: &[u64]) {
        if let Some(candidate) = self.registry.candidate.as_mut() {
            let live: std::collections::BTreeSet<_> = live.iter().copied().collect();
            candidate.retain(|id, _| live.contains(id));
            self.candidate_scopes
                .as_mut()
                .unwrap()
                .retain(|id, _| live.contains(id));
        }
    }
    fn insert_style(&mut self, scope: u64, compute: Rc<crate::StyleCompute>) -> Result<usize> {
        self.insert_reactive(scope, ReactiveCallback::Style(compute))
    }
    fn insert_text(&mut self, scope: u64, compute: Rc<crate::TextCompute>) -> Result<usize> {
        self.insert_reactive(scope, ReactiveCallback::Text(compute))
    }
    fn insert_scope(
        &mut self,
        owner: u64,
        id: u64,
        build: Rc<crate::ScopeCompute>,
        source: Option<Rc<()>>,
    ) -> Result<usize> {
        if id == 0 {
            return Err(Error::InvalidView("zero structural scope identity"));
        }
        // Reuse only admitted callbacks with identical immutable builder identity.
        // Candidate registrations never seed the cache: abort leaves active rows intact.
        if let Some(source) = &source
            && let Some((token, callback)) = self.reactive.iter().find_map(
                |(token, (previous_owner, callback))| match callback {
                    ReactiveCallback::Scope(key, _, Some(previous_source))
                        if *previous_owner == owner
                            && *key == id
                            && Rc::ptr_eq(source, previous_source) =>
                    {
                        Some((*token, callback.clone()))
                    }
                    _ => None,
                },
            )
        {
            self.candidate_reactive
                .as_mut()
                .ok_or(Error::InvalidView(
                    "reactive registration outside projection",
                ))?
                .insert(token, (owner, callback));
            return Ok(token);
        }
        self.insert_reactive(owner, ReactiveCallback::Scope(id, build, source))
    }
    fn scope(&self, id: u64) -> Option<Rc<crate::ScopeCompute>> {
        self.candidate_reactive
            .as_ref()
            .unwrap_or(&self.reactive)
            .values()
            .rev()
            .find_map(|(_, callback)| match callback {
                ReactiveCallback::Scope(key, build, _) if *key == id => Some(build.clone()),
                _ => None,
            })
    }
    fn insert_reactive(&mut self, scope: u64, compute: ReactiveCallback) -> Result<usize> {
        let candidate = self.candidate_reactive.as_mut().ok_or(Error::InvalidView(
            "reactive registration outside projection",
        ))?;
        // Cached scopes can coexist with their replacements until the host
        // prunes the candidate against its admitted recipes. Published callbacks
        // remain bounded by two per node (text and style).
        if candidate.len() >= crate::abi::MAX_VIEW_NODES * 4 {
            return Err(Error::InvalidView("reactive registry capacity exceeded"));
        }
        // Never reuse an aborted/retired token within an implementation instance.
        self.next_reactive = self.next_reactive.checked_add(1).ok_or(Error::Overflow)?;
        candidate.insert(self.next_reactive, (scope, compute));
        Ok(self.next_reactive)
    }
    fn retain_reactive(&mut self, live: &[u64]) {
        if let Some(candidate) = &mut self.candidate_reactive {
            let live: std::collections::BTreeSet<_> = live.iter().copied().collect();
            candidate.retain(|token, _| live.contains(&(*token as u64)));
        }
    }
    fn style(&self, token: usize) -> Option<Rc<crate::StyleCompute>> {
        self.candidate_reactive
            .as_ref()
            .unwrap_or(&self.reactive)
            .get(&token)
            .and_then(|(_, compute)| match compute {
                ReactiveCallback::Style(compute) => Some(compute.clone()),
                _ => None,
            })
    }
    fn text(&self, token: usize) -> Option<Rc<crate::TextCompute>> {
        self.candidate_reactive
            .as_ref()
            .unwrap_or(&self.reactive)
            .get(&token)
            .and_then(|(_, compute)| match compute {
                ReactiveCallback::Text(compute) => Some(compute.clone()),
                _ => None,
            })
    }
    pub fn insert(
        &mut self,
        id: u64,
        callback: impl for<'cx, 'host> Fn(&'cx Cx<'host>, i64) -> Result<()> + 'static,
    ) -> Result<()> {
        self.insert_scoped(0, id, Rc::new(callback))
    }
    pub fn commit(&mut self) -> Result<()> {
        self.registry.commit()?;
        self.scopes = self.candidate_scopes.take().unwrap();
        self.reactive = self.candidate_reactive.take().unwrap();
        Ok(())
    }
    pub fn abort(&mut self) {
        self.registry.abort();
        self.candidate_scopes = None;
        self.candidate_reactive = None;
    }
    pub fn callback(&self, id: u64) -> Option<Rc<Callback>> {
        self.registry.active.get(&id).cloned()
    }
}

struct Registry<C> {
    active: BTreeMap<u64, C>,
    candidate: Option<BTreeMap<u64, C>>,
}
impl<C> Default for Registry<C> {
    fn default() -> Self {
        Self {
            active: BTreeMap::new(),
            candidate: None,
        }
    }
}
impl<C> Registry<C> {
    #[cfg(test)]
    fn begin(&mut self) -> Result<()> {
        if self.candidate.is_some() {
            return Err(Error::Application("nested event projection".into()));
        }
        self.candidate = Some(BTreeMap::new());
        Ok(())
    }
    fn insert(&mut self, id: u64, callback: C) -> Result<()> {
        let candidate = self
            .candidate
            .as_mut()
            .ok_or_else(|| Error::Application("event registration outside projection".into()))?;
        if id == 0 || candidate.contains_key(&id) {
            return Err(Error::Application(
                "duplicate or zero event identity".into(),
            ));
        }
        if candidate.len() >= crate::abi::MAX_VIEW_NODES {
            return Err(Error::Application(
                "event registry capacity exceeded".into(),
            ));
        }
        candidate.insert(id, callback);
        Ok(())
    }
    fn commit(&mut self) -> Result<()> {
        let candidate = self
            .candidate
            .take()
            .ok_or_else(|| Error::Application("event commit without projection".into()))?;
        self.active = candidate;
        Ok(())
    }
    fn abort(&mut self) {
        self.candidate = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    struct Capture(Rc<Cell<usize>>);
    impl Drop for Capture {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    #[test]
    fn failed_projection_keeps_live_events_and_releases_candidate_captures() {
        let dropped = Rc::new(Cell::new(0));
        let mut registry = Registry::default();
        registry.begin().unwrap();
        registry.insert(1, Capture(dropped.clone())).unwrap();
        registry.commit().unwrap();
        registry.begin().unwrap();
        registry.insert(1, Capture(dropped.clone())).unwrap();
        assert!(registry.insert(1, Capture(dropped.clone())).is_err());
        assert_eq!(dropped.get(), 1);
        registry.abort();
        assert_eq!(dropped.get(), 2);
        assert!(registry.active.contains_key(&1));
        registry.begin().unwrap();
        registry.insert(2, Capture(dropped.clone())).unwrap();
        registry.commit().unwrap();
        assert_eq!(dropped.get(), 3);
        assert!(!registry.active.contains_key(&1));
        drop(registry);
        assert_eq!(dropped.get(), 4);
    }
    #[test]
    fn text_and_style_tokens_cannot_be_confused_and_are_pruned_together() {
        let mut events = Events::default();
        events.begin().unwrap();
        let text = events
            .insert_text(10, Rc::new(|_| Ok("text".into())))
            .unwrap();
        let style = events
            .insert_style(10, Rc::new(|_| Ok(crate::ViewStyle::default())))
            .unwrap();
        assert_ne!(text, style);
        assert!(events.style(text).is_none());
        assert!(events.text(style).is_none());
        events.retain_reactive(&[text as u64, style as u64]);
        events.commit().unwrap();
        assert!(events.text(text).is_some());
        assert!(events.style(style).is_some());
        events.begin().unwrap();
        events.retain_reactive(&[style as u64]);
        events.commit().unwrap();
        assert!(events.text(text).is_none());
        assert!(events.style(style).is_some());
    }
    #[test]
    fn cached_rows_reuse_admitted_tokens_and_abort_or_prune_transactionally() {
        let mut events = Events::default();
        let source = Rc::new(());
        let build: Rc<crate::ScopeCompute> = Rc::new(|_| Ok(crate::View::empty()));
        let weak = Rc::downgrade(&build);
        events.begin().unwrap();
        let first = events
            .insert_scope(1, 10, build, Some(source.clone()))
            .unwrap();
        events.commit().unwrap();
        events.reset_scope(1).unwrap();
        let same = events
            .insert_scope(
                1,
                10,
                Rc::new(|_| Ok(crate::View::empty())),
                Some(source.clone()),
            )
            .unwrap();
        assert_eq!(same, first);
        let different = events
            .insert_scope(
                1,
                10,
                Rc::new(|_| Ok(crate::View::empty())),
                Some(Rc::new(())),
            )
            .unwrap();
        assert_ne!(different, first);
        let another_owner = events
            .insert_scope(
                2,
                10,
                Rc::new(|_| Ok(crate::View::empty())),
                Some(source.clone()),
            )
            .unwrap();
        assert_ne!(another_owner, first);
        events.abort();
        assert!(Rc::ptr_eq(
            &weak.upgrade().unwrap(),
            &events.scope(10).unwrap()
        ));
        events.begin().unwrap();
        events.retain_reactive(&[]);
        events.commit().unwrap();
        assert!(weak.upgrade().is_none());
        events.begin().unwrap();
        let reinserted = events
            .insert_scope(1, 10, Rc::new(|_| Ok(crate::View::empty())), Some(source))
            .unwrap();
        assert_ne!(reinserted, first);
        events.abort();
    }

    #[test]
    fn structural_captures_keep_their_owner_and_abort_parameter_replacement() {
        let mut events = Events::default();
        events.begin().unwrap();
        let first = events
            .insert_scope(
                0,
                10,
                Rc::new(|_| Ok(crate::Column::default().children(vec![]))),
                None,
            )
            .unwrap();
        let nested = events
            .insert_scope(
                10,
                20,
                Rc::new(|_| Ok(crate::Column::default().children(vec![]))),
                None,
            )
            .unwrap();
        assert!(events.text(first).is_none());
        assert!(events.style(first).is_none());
        events.commit().unwrap();
        let old = events.scope(10).unwrap();
        events.reset_scope(10).unwrap();
        assert!(
            events.scope(10).is_some(),
            "the mounted builder is owned by its parent"
        );
        assert!(
            events.scope(20).is_none(),
            "rebuild retires nested builders"
        );
        let replacement = events
            .insert_scope(
                0,
                10,
                Rc::new(|_| Ok(crate::Row::default().children(vec![]))),
                None,
            )
            .unwrap();
        assert!(replacement > nested);
        assert!(!Rc::ptr_eq(&old, &events.scope(10).unwrap()));
        events.abort();
        assert!(Rc::ptr_eq(&old, &events.scope(10).unwrap()));
        assert!(events.scope(20).is_some());
        events.begin().unwrap();
        events.retain_reactive(&[first as u64]);
        events.commit().unwrap();
        assert!(events.scope(10).is_some());
        assert!(events.scope(20).is_none());
    }
    #[test]
    fn scoped_styles_keep_cached_siblings_and_prune_removed_scopes() {
        let dropped = Rc::new(Cell::new(0));
        let callback = || {
            let capture = Capture(dropped.clone());
            Rc::new(move |_: &Cx<'_>| {
                let _ = &capture;
                Ok(crate::ViewStyle::default())
            }) as Rc<crate::StyleCompute>
        };
        let mut events = Events::default();
        events.begin().unwrap();
        let left = events.insert_style(10, callback()).unwrap();
        let right = events.insert_style(20, callback()).unwrap();
        events.commit().unwrap();
        events.reset_scope(10).unwrap();
        let rejected = events.insert_style(10, callback()).unwrap();
        assert!(events.style(left).is_none());
        assert!(events.style(right).is_some());
        events.abort();
        assert_eq!(dropped.get(), 1);
        assert!(events.style(left).is_some());
        events.reset_scope(10).unwrap();
        let replacement = events.insert_style(10, callback()).unwrap();
        assert!(replacement > rejected);
        events.retain_reactive(&[replacement as u64, right as u64]);
        events.commit().unwrap();
        assert_eq!(dropped.get(), 2);
        assert!(events.style(left).is_none());
        assert!(events.style(right).is_some());
        events.begin().unwrap();
        events.retain_reactive(&[replacement as u64]);
        events.commit().unwrap();
        assert_eq!(dropped.get(), 3);
        assert!(events.style(right).is_none());
        drop(events);
        assert_eq!(dropped.get(), 4);
    }
    #[test]
    fn replacement_reconnects_same_event_identity_to_current_closure() {
        let value = Rc::new(Cell::new(0));
        let mut registry: Registry<Box<dyn Fn()>> = Registry::default();
        registry.begin().unwrap();
        let old = value.clone();
        registry
            .insert(42, Box::new(move || old.set(old.get() + 1)))
            .unwrap();
        registry.commit().unwrap();
        registry.active[&42]();
        registry.begin().unwrap();
        let new = value.clone();
        registry
            .insert(42, Box::new(move || new.set(new.get() + 10)))
            .unwrap();
        registry.commit().unwrap();
        registry.active[&42]();
        assert_eq!(value.get(), 11);
    }
}

thread_local! {
    // No TLS destructor: registering one can keep the owning dylib mapped until
    // thread exit. Loaded::drop explicitly disposes every owner before dlclose.
    static INSTANCES: std::cell::RefCell<std::mem::ManuallyDrop<BTreeMap<u64, Events>>> =
        const { std::cell::RefCell::new(std::mem::ManuallyDrop::new(BTreeMap::new())) };
}
pub(crate) fn lifecycle(owner: u64, operation: u32, ids: &[u64]) -> Result<()> {
    if owner == 0 {
        return Err(Error::InvalidView("zero event owner"));
    }
    INSTANCES.with(|instances| {
        let mut instances = instances.borrow_mut();
        match operation {
            1 => instances.entry(owner).or_default().begin(),
            2 => {
                if let Some(events) = instances.get_mut(&owner)
                    && events.registry.candidate.is_some()
                {
                    events.commit()?;
                }
                Ok(())
            }
            3 => {
                if let Some(events) = instances.get_mut(&owner) {
                    events.abort();
                }
                Ok(())
            }
            4 => {
                instances.remove(&owner);
                Ok(())
            }
            5 if ids.len() == 1 => instances.entry(owner).or_default().reset_scope(ids[0]),
            6 => {
                if let Some(events) = instances.get_mut(&owner) {
                    events.retain(ids);
                }
                Ok(())
            }
            7 => {
                if let Some(events) = instances.get_mut(&owner) {
                    events.retain_reactive(ids);
                }
                Ok(())
            }
            _ => Err(Error::InvalidView("unknown event lifecycle operation")),
        }
    })
}
pub(crate) fn register(owner: u64, scope: u64, id: u64, callback: Rc<Callback>) -> Result<()> {
    INSTANCES.with(|instances| {
        instances
            .borrow_mut()
            .get_mut(&owner)
            .ok_or(Error::InvalidView("event registration outside projection"))?
            .insert_scoped(scope, id, callback)
    })
}
pub(crate) fn lookup(owner: u64, id: u64) -> Option<Rc<Callback>> {
    INSTANCES.with(|instances| instances.borrow().get(&owner)?.callback(id))
}

pub(crate) fn register_style(
    owner: u64,
    scope: u64,
    compute: Rc<crate::StyleCompute>,
) -> Result<usize> {
    INSTANCES.with(|instances| {
        instances
            .borrow_mut()
            .get_mut(&owner)
            .ok_or(Error::InvalidView(
                "reactive registration outside projection",
            ))?
            .insert_style(scope, compute)
    })
}
pub(crate) fn lookup_style(owner: u64, token: usize) -> Option<Rc<crate::StyleCompute>> {
    INSTANCES.with(|instances| instances.borrow().get(&owner)?.style(token))
}

pub(crate) fn register_text(
    owner: u64,
    scope: u64,
    compute: Rc<crate::TextCompute>,
) -> Result<usize> {
    INSTANCES.with(|instances| {
        instances
            .borrow_mut()
            .get_mut(&owner)
            .ok_or(Error::InvalidView("text registration outside projection"))?
            .insert_text(scope, compute)
    })
}
pub(crate) fn lookup_text(owner: u64, token: usize) -> Option<Rc<crate::TextCompute>> {
    INSTANCES.with(|instances| instances.borrow().get(&owner)?.text(token))
}

pub(crate) fn register_scope(
    owner: u64,
    scope: u64,
    id: u64,
    build: Rc<crate::ScopeCompute>,
    source: Option<Rc<()>>,
) -> Result<usize> {
    INSTANCES.with(|instances| {
        instances
            .borrow_mut()
            .get_mut(&owner)
            .ok_or(Error::InvalidView("scope registration outside projection"))?
            .insert_scope(scope, id, build, source)
    })
}
pub(crate) fn lookup_scope(owner: u64, id: u64) -> Option<Rc<crate::ScopeCompute>> {
    INSTANCES.with(|instances| instances.borrow().get(&owner)?.scope(id))
}
