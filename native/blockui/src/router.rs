//! Typed navigation whose position and history belong to runtime state.
//!
//! Use [`Router`] for a finite set of destinations with stable integer IDs.
//! Use [`PathRouter`] when destinations carry parameters. Both are temporary
//! handles over [`State`] slots; recreating the handle does not recreate history.
//! Store the slots on the actor and construct the handle when needed.
//!
//! ```
//! use blockui::{PathRouter, RoutePath, State};
//!
//! #[derive(Clone, Default, PartialEq, Eq)]
//! enum Destination {
//!     #[default]
//!     Home,
//!     Document(u64),
//! }
//! impl RoutePath for Destination {
//!     fn to_path(&self) -> String {
//!         match self {
//!             Self::Home => "/".into(),
//!             Self::Document(id) => format!("/documents/{id}"),
//!         }
//!     }
//!     fn from_path(path: &str) -> Option<Self> {
//!         if path == "/" { return Some(Self::Home); }
//!         let id = path.strip_prefix("/documents/")?.parse().ok()?;
//!         Some(Self::Document(id))
//!     }
//! }
//! fn router(history: State<Vec<String>>) -> PathRouter<Destination> {
//!     PathRouter::new(history)
//! }
//! ```
//!
//! Invoke `push`, `replace`, `back`, `forward`, or `navigate` inside an actor
//! action. `navigate` evaluates a synchronous guard using the current code.
//! To await user confirmation, store the pending request in actor state, render
//! a dialog, and navigate from its later confirmation action. Never retain a
//! guard callback or a route object in the host across implementation swaps.
//!
//! Route encodings are a persistence contract: renaming paths, reusing integer
//! IDs, or removing destinations requires explicit migration of current and
//! historical entries, including pending confirmation state. Validation rejects
//! noncanonical paths such as `/documents/0042` even if `from_path` parses them.
//! [`PathRouter::push_path`] accepts a canonical application path; URL schemes,
//! decoding and operating-system delivery belong to the host.
//!
//! Key each page's `Outlet` by its business identity (for example document ID).
//! Actor drafts remain in runtime slots; native widget focus and selection
//! restoration require the host's explicit, bounded renderer-retention policy.
//! Router history alone does not persist document contents or save them to disk.
use crate::{Cx, Error, Result, State};

/// A navigation request evaluated by the currently running implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Navigation<R> {
    Push(R),
    Replace(R),
    Back,
    Forward,
}
impl<R: RoutePath> Navigation<R> {
    /// Encode a request as data for a later confirmation turn. No callback or
    /// Rust route value needs to survive implementation replacement.
    pub fn to_pending(&self) -> Result<String> {
        match self {
            Self::Push(route) => PathRouter::<R>::encode(route),
            Self::Replace(route) => Ok(format!("replace:{}", PathRouter::<R>::encode(route)?)),
            Self::Back => Ok("back".into()),
            Self::Forward => Ok("forward".into()),
        }
    }
    /// Decode empty state or a canonical pending request using current code.
    /// Legacy push paths and back/forward tokens keep their representation.
    pub fn from_pending(request: &str) -> Result<Option<Self>> {
        match request {
            "" => Ok(None),
            "back" => Ok(Some(Self::Back)),
            "forward" => Ok(Some(Self::Forward)),
            path => {
                if let Some(path) = path.strip_prefix("replace:") {
                    Ok(Some(Self::Replace(PathRouter::<R>::parse_path(path)?)))
                } else {
                    Ok(Some(Self::Push(PathRouter::<R>::parse_path(path)?)))
                }
            }
        }
    }
}

/// Stable route IDs are part of the application schema. Never reuse an ID for
/// a different destination; removed routes must be migrated explicitly.
pub trait Route: Copy + Eq {
    /// Each destination must have a unique ID that round-trips through `from_id`.
    fn id(self) -> i64;
    fn from_id(id: i64) -> Option<Self>;
}

/// A navigation token, not an owner of pages or plugin callbacks.
/// History is bounded to 128 entries in each direction. Navigation is committed
/// in the enclosing actor transaction, so rejected actions restore all slots.
pub struct Router<R> {
    current: State<i64>,
    back: State<Vec<i64>>,
    forward: State<Vec<i64>>,
    marker: std::marker::PhantomData<R>,
}
impl<R: Route> Router<R> {
    /// Check a synchronous guard before changing route state or history.
    /// Returns false for denied or unavailable navigation. The guard is called
    /// once for an available request and is never retained. Guard side effects
    /// belong to the enclosing actor transaction; false does not undo them.
    /// Changing this router's state inside the guard returns an error; propagate
    /// it from the action to roll back the transaction.
    /// For asynchronous confirmation, store the pending request as actor state
    /// and issue navigation in a later action after confirmation.
    pub fn navigate(
        &self,
        cx: &Cx<'_>,
        request: Navigation<R>,
        guard: impl FnOnce(&R, &R) -> Result<bool>,
    ) -> Result<bool> {
        let (current, back, forward) = self.snapshot(cx)?;
        let target = match &request {
            Navigation::Push(route) | Navigation::Replace(route) => {
                Self::validate(*route)?;
                *route
            }
            Navigation::Back | Navigation::Forward => {
                let history = if matches!(request, Navigation::Back) {
                    &back
                } else {
                    &forward
                };
                let Some(&id) = history.last() else {
                    return Ok(false);
                };
                Self::decode(id)?
            }
        };
        let allowed = guard(&current, &target)?;
        if self.current.get(cx)? != current.id()
            || self.back.get(cx)? != back
            || self.forward.get(cx)? != forward
        {
            return Err(Error::Application(
                "Navigation guard changed router state".into(),
            ));
        }
        if !allowed {
            return Ok(false);
        }
        match request {
            Navigation::Push(route) => self.push(cx, route)?,
            Navigation::Replace(route) => self.replace(cx, route)?,
            Navigation::Back => return self.back(cx),
            Navigation::Forward => return self.forward(cx),
        }
        Ok(true)
    }
    pub const fn new(current: State<i64>, back: State<Vec<i64>>, forward: State<Vec<i64>>) -> Self {
        Self {
            current,
            back,
            forward,
            marker: std::marker::PhantomData,
        }
    }
    pub fn current(&self, cx: &Cx<'_>) -> Result<R> {
        self.snapshot(cx).map(|(current, _, _)| current)
    }
    fn snapshot(&self, cx: &Cx<'_>) -> Result<(R, Vec<i64>, Vec<i64>)> {
        let back = self.history(cx, self.back)?;
        let forward = self.history(cx, self.forward)?;
        let current = Self::decode(self.current.get(cx)?)?;
        Ok((current, back, forward))
    }
    fn validate_history(history: &[i64]) -> Result<()> {
        if history.len() > 128 {
            return Err(Error::Application(
                "Route history exceeds 128 entries".into(),
            ));
        }
        for &id in history {
            Self::decode(id)?;
        }
        Ok(())
    }
    fn history(&self, cx: &Cx<'_>, slot: State<Vec<i64>>) -> Result<Vec<i64>> {
        let history = slot.get(cx)?;
        Self::validate_history(&history)?;
        Ok(history)
    }
    fn decode(id: i64) -> Result<R> {
        R::from_id(id)
            .filter(|route| route.id() == id)
            .ok_or_else(|| Error::Application(format!("Unknown or noncanonical route {id}")))
    }
    fn validate(route: R) -> Result<()> {
        if Self::decode(route.id())? != route {
            return Err(Error::Application("Ambiguous route ID".into()));
        }
        Ok(())
    }
    fn append(history: &mut Vec<i64>, id: i64) {
        if history.len() >= 128 {
            history.remove(0);
        }
        history.push(id);
    }
    pub fn push(&self, cx: &Cx<'_>, route: R) -> Result<()> {
        Self::validate(route)?;
        let (current, mut back, _) = self.snapshot(cx)?;
        if current == route {
            return Ok(());
        }
        Self::append(&mut back, current.id());
        self.back.set(cx, &back)?;
        self.forward.set(cx, &[])?;
        self.current.set(cx, route.id())
    }
    pub fn replace(&self, cx: &Cx<'_>, route: R) -> Result<()> {
        Self::validate(route)?;
        self.current(cx)?;
        self.current.set(cx, route.id())
    }
    pub fn can_back(&self, cx: &Cx<'_>) -> Result<bool> {
        let (_, back, _) = self.snapshot(cx)?;
        Ok(!back.is_empty())
    }
    pub fn can_forward(&self, cx: &Cx<'_>) -> Result<bool> {
        let (_, _, forward) = self.snapshot(cx)?;
        Ok(!forward.is_empty())
    }
    pub fn back(&self, cx: &Cx<'_>) -> Result<bool> {
        self.travel(cx, false)
    }
    pub fn forward(&self, cx: &Cx<'_>) -> Result<bool> {
        self.travel(cx, true)
    }
    fn travel(&self, cx: &Cx<'_>, forward: bool) -> Result<bool> {
        let (current, back, ahead) = self.snapshot(cx)?;
        let (from, to, mut source, mut destination) = if forward {
            (self.forward, self.back, ahead, back)
        } else {
            (self.back, self.forward, back, ahead)
        };
        let Some(id) = source.pop() else {
            return Ok(false);
        };
        Self::append(&mut destination, current.id());
        from.set(cx, &source)?;
        to.set(cx, &destination)?;
        self.current.set(cx, id)?;
        Ok(true)
    }
}

/// An owned, version-stable route representation (for example `/documents/42`).
/// Parsing must reject removed destinations and invalid parameters. The default
/// destination is used before the first navigation; changing it is a migration.
pub trait RoutePath: Clone + Eq + Default {
    /// Durable format identity for `#[state(route = ...)]`. The derive generates
    /// this from path declarations; handwritten implementations must supply it.
    const FORMAT: &'static str = "";
    fn to_path(&self) -> String;
    fn from_path(path: &str) -> Option<Self>;
}

#[doc(hidden)]
pub fn __validate_pending_route<R: RoutePath>(request: &str) -> Result<()> {
    Navigation::<R>::from_pending(request).map(|_| ())
}

/// Parameterized navigation stored in one runtime text-list slot.
/// Its first entry is the history cursor, followed by canonical route paths.
/// No Rust route object crosses the ABI. History contains at most 129 entries.
pub struct PathRouter<R> {
    state: State<Vec<String>>,
    marker: std::marker::PhantomData<R>,
}
impl<R: RoutePath> PathRouter<R> {
    /// Guard navigation before writing its history. See `Router::navigate` for
    /// guard lifetime, transaction and asynchronous confirmation semantics.
    pub fn navigate(
        &self,
        cx: &Cx<'_>,
        request: Navigation<R>,
        guard: impl FnOnce(&R, &R) -> Result<bool>,
    ) -> Result<bool> {
        let snapshot = self.state.get(cx)?;
        let (cursor, entries) = self.read(cx)?;
        let current = Self::parse_path(&entries[cursor])?;
        let target = match &request {
            Navigation::Push(route) | Navigation::Replace(route) => {
                Self::encode(route)?;
                route.clone()
            }
            Navigation::Back | Navigation::Forward => {
                let next = if matches!(request, Navigation::Back) {
                    cursor.checked_sub(1)
                } else {
                    cursor.checked_add(1).filter(|i| *i < entries.len())
                };
                let Some(next) = next else { return Ok(false) };
                Self::parse_path(&entries[next])?
            }
        };
        let allowed = guard(&current, &target)?;
        if self.state.get(cx)? != snapshot {
            return Err(Error::Application(
                "Navigation guard changed router state".into(),
            ));
        }
        if !allowed {
            return Ok(false);
        }
        match request {
            Navigation::Push(route) => self.push(cx, route)?,
            Navigation::Replace(route) => self.replace(cx, route)?,
            Navigation::Back => return self.back(cx),
            Navigation::Forward => return self.forward(cx),
        }
        Ok(true)
    }
    pub const fn new(state: State<Vec<String>>) -> Self {
        Self {
            state,
            marker: std::marker::PhantomData,
        }
    }
    fn encode(route: &R) -> Result<String> {
        let path = route.to_path();
        if path.len() > 4096 || R::from_path(&path).as_ref() != Some(route) {
            return Err(Error::Application("Invalid route representation".into()));
        }
        Ok(path)
    }
    /// Decode a canonical application path without reading or changing state.
    /// Use this for pending navigation and external requests before passing the
    /// typed destination to `navigate`. The returned value belongs to the current
    /// implementation; persist its canonical path, not the Rust value.
    pub fn parse_path(path: &str) -> Result<R> {
        if path.len() > 4096 {
            return Err(Error::Application("Route path exceeds 4096 bytes".into()));
        }
        let route =
            R::from_path(path).ok_or_else(|| Error::Application("Unknown route path".into()))?;
        if Self::encode(&route)? != path {
            return Err(Error::Application("Noncanonical route path".into()));
        }
        Ok(route)
    }
    /// Navigate from a canonical external path. Parsing/validation happens before
    /// writes; history and actor transaction behavior are identical to `push`.
    /// URL schemes, percent decoding and OS deep-link delivery belong to the host.
    pub fn push_path(&self, cx: &Cx<'_>, path: &str) -> Result<()> {
        self.push(cx, Self::parse_path(path)?)
    }
    fn read(&self, cx: &Cx<'_>) -> Result<(usize, Vec<String>)> {
        let mut stored = self.state.get(cx)?;
        if stored.is_empty() {
            return Ok((0, vec![Self::encode(&R::default())?]));
        }
        let encoded_cursor = stored.remove(0);
        let cursor = encoded_cursor
            .parse::<usize>()
            .map_err(|_| Error::Application("Invalid route cursor".into()))?;
        if cursor.to_string() != encoded_cursor
            || stored.is_empty()
            || stored.len() > 129
            || cursor >= stored.len()
            || stored.iter().any(|p| Self::parse_path(p).is_err())
        {
            return Err(Error::Application(
                "Invalid route history; migrate removed routes".into(),
            ));
        }
        Ok((cursor, stored))
    }
    fn write(&self, cx: &Cx<'_>, cursor: usize, mut entries: Vec<String>) -> Result<()> {
        entries.insert(0, cursor.to_string());
        self.state.set(cx, &entries)
    }
    pub fn current(&self, cx: &Cx<'_>) -> Result<R> {
        let (cursor, entries) = self.read(cx)?;
        R::from_path(&entries[cursor]).ok_or_else(|| Error::Application("Invalid route".into()))
    }
    pub fn push(&self, cx: &Cx<'_>, route: R) -> Result<()> {
        let path = Self::encode(&route)?;
        let (cursor, mut entries) = self.read(cx)?;
        if R::from_path(&entries[cursor]).as_ref() == Some(&route) {
            return Ok(());
        }
        entries.truncate(cursor + 1);
        entries.push(path);
        if entries.len() > 129 {
            entries.remove(0);
        }
        self.write(cx, entries.len() - 1, entries)
    }
    pub fn replace(&self, cx: &Cx<'_>, route: R) -> Result<()> {
        let path = Self::encode(&route)?;
        let (cursor, mut entries) = self.read(cx)?;
        entries[cursor] = path;
        self.write(cx, cursor, entries)
    }
    pub fn can_back(&self, cx: &Cx<'_>) -> Result<bool> {
        Ok(self.read(cx)?.0 > 0)
    }
    pub fn can_forward(&self, cx: &Cx<'_>) -> Result<bool> {
        let (cursor, entries) = self.read(cx)?;
        Ok(cursor + 1 < entries.len())
    }
    pub fn back(&self, cx: &Cx<'_>) -> Result<bool> {
        self.travel(cx, false)
    }
    pub fn forward(&self, cx: &Cx<'_>) -> Result<bool> {
        self.travel(cx, true)
    }
    fn travel(&self, cx: &Cx<'_>, forward: bool) -> Result<bool> {
        let (cursor, entries) = self.read(cx)?;
        let next = if forward {
            cursor.checked_add(1).filter(|i| *i < entries.len())
        } else {
            cursor.checked_sub(1)
        };
        let Some(next) = next else {
            return Ok(false);
        };
        self.write(cx, next, entries)?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Default, PartialEq, Eq, Debug)]
    enum Parameterized {
        #[default]
        Home,
        Document(u64),
    }
    impl RoutePath for Parameterized {
        fn to_path(&self) -> String {
            match self {
                Self::Home => "/".into(),
                Self::Document(id) => format!("/documents/{id}"),
            }
        }
        fn from_path(path: &str) -> Option<Self> {
            if path == "/" {
                return Some(Self::Home);
            }
            Some(Self::Document(
                path.strip_prefix("/documents/")?.parse().ok()?,
            ))
        }
    }
    #[test]
    fn public_path_parser_requires_canonical_bounded_round_trip() {
        assert_eq!(
            PathRouter::<Parameterized>::parse_path("/").unwrap(),
            Parameterized::Home
        );
        assert_eq!(
            PathRouter::<Parameterized>::parse_path("/documents/42").unwrap(),
            Parameterized::Document(42)
        );
        for path in [
            "/documents/0042",
            "/documents/+42",
            "/documents/-1",
            "/missing",
            "/documents/42/",
        ] {
            assert!(
                PathRouter::<Parameterized>::parse_path(path).is_err(),
                "{path}"
            );
        }
        assert!(PathRouter::<Parameterized>::parse_path(&"x".repeat(4097)).is_err());
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Destination {
        Home,
        Alias,
    }
    impl Route for Destination {
        fn id(self) -> i64 {
            0
        }
        fn from_id(id: i64) -> Option<Self> {
            match id {
                0 | 1 => Some(Self::Home),
                _ => None,
            }
        }
    }

    #[test]
    fn rejects_alias_ids_and_destinations_before_navigation_writes() {
        assert!(Router::<Destination>::validate(Destination::Home).is_ok());
        assert!(Router::<Destination>::validate(Destination::Alias).is_err());
        assert!(Router::<Destination>::decode(1).is_err());
        assert!(Router::<Destination>::decode(2).is_err());
    }

    #[test]
    fn rejects_removed_routes_anywhere_in_history_and_oversized_history() {
        assert!(Router::<Destination>::validate_history(&[0; 128]).is_ok());
        assert!(Router::<Destination>::validate_history(&[0; 129]).is_err());
        assert!(Router::<Destination>::validate_history(&[2, 0]).is_err());
        assert!(Router::<Destination>::validate_history(&[0, 1]).is_err());
    }
}
