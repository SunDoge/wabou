//! Structural identities live only with the loaded implementation's function table.
use super::*;
#[derive(Default)]
struct Collection {
    ids: BTreeSet<u64>,
    error: Option<String>,
}
unsafe extern "C" fn emit(ctx: *mut c_void, id: u64) -> i32 {
    let collection = unsafe { &mut *ctx.cast::<Collection>() };
    if collection.error.is_some() {
        return ERROR;
    }
    let error = if id == 0 {
        Some("zero scope identity".into())
    } else if collection.ids.contains(&id) {
        Some(format!("duplicate scope identity {id:#018x}"))
    } else if collection.ids.len() >= 256 {
        Some(format!("scope limit 256 exceeded at identity {id:#018x}"))
    } else {
        None
    };
    if let Some(error) = error {
        collection.error = Some(error);
        return ERROR;
    }
    collection.ids.insert(id);
    OK
}
pub(super) fn registry(loaded: &Loaded) -> Result<BTreeSet<u64>, String> {
    let mut collection = Collection::default();
    let status =
        unsafe { (loaded.table().scope_ids)(std::ptr::from_mut(&mut collection).cast(), emit) };
    if let Some(error) = collection.error {
        return Err(format!("invalid structural scope registry: {error}"));
    }
    if status != OK {
        return Err(format!(
            "structural scope registry callback failed (status {status})"
        ));
    }
    Ok(collection.ids)
}

/// Host-owned dependencies of one mounted structural builder; no plugin pointers.
#[derive(Clone, Debug, Default)]
pub struct ScopeDependencies {
    /// Successful builder executions published within this mount/implementation.
    /// Cached reuse preserves the count; replacement or remount can reset it.
    pub evaluations: u64,
    /// Opaque implementation token; changes invalidate captured parameters.
    pub capture_token: Option<usize>,
    pub dependencies: BTreeSet<Dependency>,
    pub versions: BTreeMap<Dependency, u64>,
}

/// Resolve mounted markers through the current DLL, retaining only host-owned data.
pub(super) fn expand(
    loaded: &Loaded,
    turn: &mut Turn,
    collection: &mut bindings::Collection,
) -> Result<BTreeMap<u64, ScopeDependencies>, String> {
    expand_preserving(loaded, turn, collection, &BTreeSet::new())
}

pub(super) fn expand_preserving(
    loaded: &Loaded,
    turn: &mut Turn,
    collection: &mut bindings::Collection,
    preserved: &BTreeSet<u64>,
) -> Result<BTreeMap<u64, ScopeDependencies>, String> {
    expand_cached(loaded, turn, collection, preserved, None)
}

/// Only reuse projections within the same loaded implementation, after flush
/// has refreshed structural dependency versions. Reload must pass no cache.
pub(super) fn expand_cached(
    loaded: &Loaded,
    turn: &mut Turn,
    collection: &mut bindings::Collection,
    preserved: &BTreeSet<u64>,
    previous: Option<&View>,
) -> Result<BTreeMap<u64, ScopeDependencies>, String> {
    let previous = previous.filter(|view| !view.scope_dependencies.is_empty());
    let mut previous_children: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    let mut previous_positions = BTreeMap::new();
    if let Some(view) = previous {
        for (position, key) in view.order.iter().enumerate() {
            previous_children
                .entry(view.bindings[key].node.parent)
                .or_default()
                .push(*key);
            previous_positions.insert(*key, position);
        }
    }
    let mut scope_dependencies = BTreeMap::new();
    let mut expanded = preserved.clone();
    loop {
        let Some((recipe_index, id)) =
            collection
                .recipes
                .iter()
                .enumerate()
                .find_map(|(index, (_, recipe))| {
                    if let Some(id) = recipe.scope_id()
                        && !expanded.contains(&id)
                    {
                        return Some((index, id));
                    }
                    None
                })
        else {
            return Ok(scope_dependencies);
        };
        let capture_token = collection.recipes[recipe_index].1.scope_token();
        let declared = loaded.scopes.borrow().contains(&id);
        if capture_token.is_some() && declared {
            return Err("captured scope conflicts with declared scope".into());
        }
        if capture_token.is_none() && !declared {
            return Err(format!(
                "unregistered mounted scope {id:#018x}; register it in Actor::SCOPES or use Scope::reactive"
            ));
        }
        if expanded.len() >= blockui_abi::MAX_VIEW_NODES {
            return Err("structural scope expansion limit exceeded".into());
        }
        let mount = collection.recipes[recipe_index].0;
        let index = collection
            .nodes
            .iter()
            .position(|node| node.key == mount)
            .ok_or("scope marker has no node")?;
        let marker = collection.nodes[index];
        // The builder owns the real root. Styling its placeholder would have
        // ambiguous precedence and must not discard either callback silently.
        if collection
            .recipes
            .iter()
            .any(|(key, recipe)| *key == mount && matches!(recipe, bindings::Recipe::Style(_)))
        {
            return Err("style the scope builder root, not its mount marker".into());
        }
        if marker.kind != COLUMN
            || mount != id
            || collection.nodes.iter().any(|node| node.parent == mount)
        {
            return Err("scope marker must be an empty column with its registered identity".into());
        }
        if let Some(view) = previous
            && view
                .scope_dependencies
                .get(&id)
                .is_some_and(|scope| scope.capture_token == capture_token)
        {
            let mut descendants = BTreeSet::new();
            let mut pending = vec![id];
            while let Some(key) = pending.pop() {
                if descendants.insert(key)
                    && let Some(children) = previous_children.get(&key)
                {
                    pending.extend(children);
                }
            }
            let cached: BTreeMap<_, _> = view
                .scope_dependencies
                .iter()
                .filter(|(key, _)| descendants.contains(key))
                .map(|(key, value)| (*key, value.clone()))
                .collect();
            if cached
                .values()
                .all(|scope| graph::versions(turn, &scope.dependencies) == scope.versions)
            {
                if cached.keys().any(|key| expanded.contains(key)) {
                    return Err("recursive or repeated scope mount".into());
                }
                let mut positions: Vec<_> = descendants
                    .iter()
                    .filter_map(|key| {
                        previous_positions
                            .get(key)
                            .map(|position| (*position, *key))
                    })
                    .collect();
                positions.sort_unstable();
                let mut nodes: Vec<_> = positions
                    .into_iter()
                    .map(|(_, key)| view.bindings[&key].node)
                    .collect();
                if collection.nodes.len() - 1 + nodes.len() > blockui_abi::MAX_VIEW_NODES {
                    return Err("expanded view exceeds node budget".into());
                }
                let root = nodes
                    .iter_mut()
                    .find(|node| node.key == id)
                    .ok_or("cached scope has no root")?;
                root.parent = marker.parent;
                root.binding = marker.binding;
                collection.nodes.splice(index..=index, nodes);
                if let Some(recipe) = loaded
                    .bindings
                    .borrow()
                    .get(&id)
                    .and_then(bindings::Recipe::scoped_recipe)
                {
                    collection.recipes[recipe_index].1 = recipe.clone();
                }
                // The marker carries structural identity, while the cached root
                // also has a live, independently tracked style observer.
                if !matches!(
                    collection.recipes[recipe_index].1,
                    bindings::Recipe::Styled(..)
                ) && let Some(bindings::Recipe::Styled(_, token)) =
                    loaded.bindings.borrow().get(&id)
                {
                    collection
                        .recipes
                        .push((id, bindings::Recipe::Style(*token)));
                }
                collection.recipes.extend(
                    loaded
                        .bindings
                        .borrow()
                        .iter()
                        .filter(|(key, _)| descendants.contains(key) && **key != id)
                        .map(|(key, recipe)| (*key, recipe.clone())),
                );
                for scope in cached.values() {
                    turn.reads.extend(scope.dependencies.iter().copied());
                }
                expanded.extend(cached.keys().copied());
                scope_dependencies.extend(cached);
                continue;
            }
        }
        let mut subtree = bindings::Collection::default();
        let previous_reads = std::mem::take(&mut turn.reads);
        let code = with_api(turn, loaded.table, |api| unsafe {
            (loaded.table().scope_view)(
                api,
                id,
                std::ptr::from_mut(&mut subtree).cast(),
                super::emit,
                bindings::emit_recipe,
            )
        });
        let dependencies = std::mem::replace(&mut turn.reads, previous_reads);
        let versions = graph::versions(turn, &dependencies);
        turn.reads.extend(dependencies.iter().copied());
        scope_dependencies.insert(
            id,
            ScopeDependencies {
                evaluations: previous
                    .and_then(|view| view.scope_dependencies.get(&id))
                    .map_or(1, |scope| scope.evaluations.saturating_add(1)),
                capture_token,
                dependencies,
                versions,
            },
        );
        if code != OK || turn.graph.fault.is_some() || subtree.error.is_some() {
            let reason = subtree
                .error
                .or_else(|| turn.graph.fault.clone())
                .or_else(|| turn.handler_error.clone())
                .unwrap_or_else(|| "scope projection failed".into());
            return Err(format!("scope {id:#018x} projection failed: {reason}"));
        }
        let root = subtree.nodes.first().ok_or_else(|| {
            format!("invalid scope projection root {id:#018x}: no root node was emitted")
        })?;
        if !matches!(root.kind, COLUMN | ROW | DIALOG | SCROLL) {
            return Err(format!(
                "invalid scope projection root {id:#018x}: node kind {} is not a container; return a Column, Row, Dialog or Scroll root, wrapping leaf widgets in its children",
                root.kind
            ));
        }
        if root.key != id || root.parent != 0 {
            return Err(format!(
                "invalid scope projection root {id:#018x}: expected root key {id:#018x} and parent 0, got key {:#018x} and parent {:#018x}",
                root.key, root.parent
            ));
        }
        if subtree.recipes.iter().any(|(_, recipe)| {
            recipe
                .scope_id()
                .is_some_and(|child| child == id || expanded.contains(&child))
        }) {
            return Err("recursive or repeated scope mount".into());
        }
        if collection.nodes.len() - 1 + subtree.nodes.len() > blockui_abi::MAX_VIEW_NODES {
            return Err("expanded view exceeds node budget".into());
        }
        subtree.nodes[0].parent = marker.parent;
        subtree.nodes[0].binding = marker.binding;
        // Structure and root content are independent: a dialog name or scroll
        // caption must survive both initial expansion and cached refresh.
        let root_recipe = subtree
            .recipes
            .iter()
            .position(|(key, recipe)| *key == id && !matches!(recipe, bindings::Recipe::Style(_)))
            .ok_or("scope root has no content recipe")?;
        let content = subtree.recipes.remove(root_recipe).1;
        let marker_recipe = collection.recipes[recipe_index].1.scope_marker().clone();
        collection.recipes[recipe_index].1 =
            bindings::Recipe::Scoped(Box::new(marker_recipe), Box::new(content));
        collection.nodes.splice(index..=index, subtree.nodes);
        collection.recipes.extend(subtree.recipes);
        expanded.insert(id);
    }
}

/// Stage replacements for topmost dirty scopes, retaining unrelated projected nodes.
pub(super) fn refresh(
    loaded: &Loaded,
    turn: &mut Turn,
    view: &View,
    dirty: &BTreeSet<u64>,
) -> Result<Projection, String> {
    let mut collection = bindings::Collection {
        nodes: view
            .order
            .iter()
            .map(|key| view.bindings[key].node)
            .collect(),
        recipes: loaded
            .bindings
            .borrow()
            .iter()
            .map(|(key, recipe)| (*key, recipe.clone()))
            .collect(),
        error: None,
    };
    let mut parents = BTreeMap::new();
    let mut children: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    for node in &collection.nodes {
        parents.insert(node.key, node.parent);
        children.entry(node.parent).or_default().push(node.key);
    }
    // The published tree is already validated. Walk ancestry rather than computing
    // a complete descendant set for every dirty (potentially nested) scope.
    let mut roots = BTreeSet::new();
    for id in dirty {
        let mut parent = *parents.get(id).ok_or("dirty scope has no mount")?;
        while parent != 0 && !dirty.contains(&parent) {
            parent = parents[&parent];
        }
        if parent == 0 {
            roots.insert(*id);
        }
    }
    let mut removed = BTreeSet::new();
    let mut pending: Vec<_> = roots.iter().copied().collect();
    while let Some(id) = pending.pop() {
        removed.insert(id);
        if let Some(descendants) = children.get(&id) {
            pending.extend(descendants);
        }
    }
    collection.nodes = collection
        .nodes
        .into_iter()
        .filter_map(|mut node| {
            if roots.contains(&node.key) {
                node.kind = COLUMN;
                Some(node)
            } else if !removed.contains(&node.key) {
                Some(node)
            } else {
                None
            }
        })
        .collect();
    collection.recipes.retain(|(key, _)| !removed.contains(key));
    collection.recipes.extend(
        roots
            .into_iter()
            .map(|id| (id, loaded.bindings.borrow()[&id].scope_marker().clone())),
    );
    let mut scopes = view.scope_dependencies.clone();
    scopes.retain(|id, _| !removed.contains(id));
    let preserved = scopes.keys().copied().collect();
    turn.read_only = true;
    turn.reads.clear();
    let expanded = expand_cached(loaded, turn, &mut collection, &preserved, Some(view));
    turn.read_only = false;
    scopes.extend(expanded?);
    let mut dependencies = view.root_dependencies.clone();
    for scope in scopes.values() {
        dependencies.extend(scope.dependencies.iter().copied());
    }
    let retained = collection
        .nodes
        .iter()
        .map(|node| node.key)
        .filter(|key| !removed.contains(key))
        .collect();
    super::project_collection(
        loaded,
        turn,
        collection,
        dependencies,
        view.root_dependencies.clone(),
        scopes,
        Some(ProjectionCache {
            bindings: &view.bindings,
            retained: Some(&retained),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_rejects_zero_duplicate_and_overflow_even_if_sender_ignores_error() {
        let mut collection = Collection::default();
        assert_eq!(
            unsafe { emit(std::ptr::from_mut(&mut collection).cast(), 0) },
            ERROR
        );
        assert_eq!(collection.error.as_deref(), Some("zero scope identity"));
        assert_eq!(
            unsafe { emit(std::ptr::from_mut(&mut collection).cast(), 1) },
            ERROR
        );
        assert!(collection.ids.is_empty());
        let mut collection = Collection::default();
        for id in 1..=256 {
            assert_eq!(
                unsafe { emit(std::ptr::from_mut(&mut collection).cast(), id) },
                OK
            );
        }
        assert_eq!(
            unsafe { emit(std::ptr::from_mut(&mut collection).cast(), 257) },
            ERROR
        );
        assert_eq!(collection.ids.len(), 256);
        assert_eq!(
            collection.error.as_deref(),
            Some("scope limit 256 exceeded at identity 0x0000000000000101")
        );
        let mut collection = Collection::default();
        assert_eq!(
            unsafe { emit(std::ptr::from_mut(&mut collection).cast(), 1) },
            OK
        );
        assert_eq!(
            unsafe { emit(std::ptr::from_mut(&mut collection).cast(), 1) },
            ERROR
        );
        assert_eq!(
            collection.error.as_deref(),
            Some("duplicate scope identity 0x0000000000000001")
        );
    }
    #[test]
    fn exported_scope_projects_owned_nodes_and_rejects_unknown_identity() {
        use blockui::{
            Action, Actor as Definition, Column, Cx, IntoView, Scope, StateSpec, Text, View,
        };
        struct Model;
        impl Definition for Model {
            const STATE: &'static [StateSpec] = &[];
            const ACTIONS: &'static [Action] = &[];
            const SCOPES: &'static [Scope] = &[Scope::new("panel", |_| {
                Ok(Column::default().children(vec![Text::new("Owned text").into_view()]))
            })];
            fn view(_: &Cx<'_>) -> blockui::Result<View> {
                Ok(Column::default().children(vec![]))
            }
        }
        let table = &blockui::Export::<Model>::TABLE;
        let mut turn = Turn {
            read_only: true,
            ..Default::default()
        };
        let mut collection = bindings::Collection::default();
        let status = with_api(&mut turn, table, |api| unsafe {
            (table.scope_view)(
                api,
                key("panel"),
                std::ptr::from_mut(&mut collection).cast(),
                super::super::emit,
                bindings::emit_recipe,
            )
        });
        assert_eq!(status, OK);
        assert_eq!(collection.nodes.len(), 2);
        assert_eq!(collection.nodes[0].key, key("panel"));
        assert_eq!(collection.nodes[1].parent, key("panel"));
        assert!(
            matches!(&collection.recipes[1].1, bindings::Recipe::Static(text) if text.as_ref() == "Owned text")
        );
        let mut rejected = bindings::Collection::default();
        let status = with_api(&mut turn, table, |api| unsafe {
            (table.scope_view)(
                api,
                key("unknown"),
                std::ptr::from_mut(&mut rejected).cast(),
                super::super::emit,
                bindings::emit_recipe,
            )
        });
        assert_eq!(status, ERROR);
        assert!(rejected.nodes.is_empty());
    }
}
