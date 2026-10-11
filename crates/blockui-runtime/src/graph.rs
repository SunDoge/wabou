//! Host-owned derived values and edges. No callback or library pointer lives here.
use super::*;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dependency {
    State(u64),
    Computed(u64),
    MapEntry(u64),
}
#[derive(Clone, Debug)]
struct Memo {
    kind: u32,
    value: Option<Arc<Value>>,
    dependencies: BTreeMap<Dependency, u64>,
    dirty: bool,
    revision: u64,
    evaluations: u64,
}
#[derive(Clone, Default)]
pub(super) struct Graph {
    pub(super) map_reads: crate::keyed_map::Cache,
    memos: BTreeMap<u64, Memo>,
    // Bounded execution order also provides actionable cycle diagnostics.
    active: Vec<u64>,
    dependents: BTreeMap<Dependency, BTreeSet<u64>>,
    pub fault: Option<String>,
}
unsafe extern "C" fn descriptor(ctx: *mut c_void, spec: ComputedSpec) -> i32 {
    let graph = unsafe { &mut *ctx.cast::<Graph>() };
    if spec.key == 0
        || !matches!(spec.kind, I64 | F64 | BOOLEAN | TEXT | I64_LIST | TEXT_LIST)
        || graph.memos.len() >= 256
        || graph.memos.contains_key(&spec.key)
    {
        return ERROR;
    }
    graph.memos.insert(
        spec.key,
        Memo {
            kind: spec.kind,
            value: None,
            dependencies: BTreeMap::new(),
            dirty: true,
            revision: 0,
            evaluations: 0,
        },
    );
    OK
}
impl Graph {
    pub fn load(loaded: &Loaded) -> Result<Self, String> {
        let mut graph = Self::default();
        if unsafe { (loaded.table().computed_specs)((&mut graph as *mut Self).cast(), descriptor) }
            != OK
        {
            return Err("invalid computed registry".into());
        }
        Ok(graph)
    }
    pub fn affected(&self, sources: impl IntoIterator<Item = Dependency>) -> BTreeSet<Dependency> {
        let mut queue: VecDeque<_> = sources.into_iter().collect();
        let mut seen = BTreeSet::new();
        while let Some(dependency) = queue.pop_front() {
            if !seen.insert(dependency) {
                continue;
            }
            if let Some(keys) = self.dependents.get(&dependency) {
                queue.extend(keys.iter().map(|key| Dependency::Computed(*key)));
            }
        }
        seen
    }
    pub fn invalidate(&mut self, source: Dependency) {
        for dependency in self.affected([source]) {
            if let Dependency::Computed(key) = dependency {
                self.memos.get_mut(&key).unwrap().dirty = true;
            }
        }
    }
    fn replace_dependencies(&mut self, key: u64, dependencies: BTreeMap<Dependency, u64>) {
        let old = std::mem::replace(
            &mut self.memos.get_mut(&key).unwrap().dependencies,
            dependencies,
        );
        for dependency in old.keys() {
            if let Some(keys) = self.dependents.get_mut(dependency) {
                keys.remove(&key);
                if keys.is_empty() {
                    self.dependents.remove(dependency);
                }
            }
        }
        for dependency in self.memos[&key].dependencies.keys() {
            self.dependents.entry(*dependency).or_default().insert(key);
        }
    }
    pub(super) fn check_map_reads(
        &self,
        published: impl IntoIterator<Item = Dependency>,
    ) -> Result<(), String> {
        let live = published
            .into_iter()
            .chain(
                self.memos
                    .values()
                    .flat_map(|memo| memo.dependencies.keys().copied()),
            )
            .filter_map(|dependency| match dependency {
                Dependency::MapEntry(id) => Some(id),
                _ => None,
            })
            .collect();
        self.map_reads.validate_live(&live)
    }
    pub(super) fn retain_map_reads(&mut self, published: impl IntoIterator<Item = Dependency>) {
        // Validation has already checked published identities. With no cached
        // reads there is nothing to retire, so do not traverse the whole view.
        if self.map_reads.is_empty() {
            return;
        }
        let dependencies = published.into_iter().chain(
            self.memos
                .values()
                .flat_map(|memo| memo.dependencies.keys().copied()),
        );
        let live = dependencies
            .filter_map(|dependency| match dependency {
                Dependency::MapEntry(id) => Some(id),
                _ => None,
            })
            .collect();
        self.map_reads.retain(&live);
    }
    pub fn evaluations(&self, key: u64) -> u64 {
        self.memos.get(&key).map_or(0, |m| m.evaluations)
    }
    pub fn count(&self) -> usize {
        self.memos.len()
    }
    pub fn value(&self, key: u64) -> Option<&Value> {
        self.memos.get(&key)?.value.as_deref()
    }
    pub fn dependencies(&self, key: u64) -> Option<BTreeSet<Dependency>> {
        Some(self.memos.get(&key)?.dependencies.keys().copied().collect())
    }
}
pub(super) fn fail(context: &Context, message: &str) -> String {
    let turn = unsafe { &mut *context.turn };
    turn.graph.fault = Some(message.into());
    message.into()
}
pub(super) fn revision(turn: &Turn, dependency: Dependency) -> u64 {
    match dependency {
        Dependency::MapEntry(id) => turn.graph.map_reads.revision(id),
        Dependency::State(key) => turn.revisions.get(&key).copied().unwrap_or(0),
        Dependency::Computed(key) => turn.graph.memos.get(&key).map_or(0, |m| m.revision),
    }
}
pub(super) fn versions(
    turn: &Turn,
    dependencies: &BTreeSet<Dependency>,
) -> BTreeMap<Dependency, u64> {
    dependencies
        .iter()
        .map(|d| (*d, revision(turn, *d)))
        .collect()
}
unsafe extern "C" fn output(ctx: *mut c_void, output: Output) -> i32 {
    let slot = unsafe { &mut *ctx.cast::<Option<Value>>() };
    if slot.is_some() {
        return ERROR;
    }
    let value = match output.kind {
        F64 => Value::Float(f64::from_bits(output.integer as u64)),
        I64 => Value::Integer(output.integer),
        BOOLEAN if matches!(output.integer, 0 | 1) => Value::Boolean(output.integer == 1),
        I64_LIST if output.text.len.is_multiple_of(8) => {
            match unsafe { list_state::copy(output.text.ptr.cast(), output.text.len / 8) } {
                Ok(value) => Value::IntegerList(value),
                Err(_) => return ERROR,
            }
        }
        TEXT_LIST
            if output.integer == 0
                && output.text.len.is_multiple_of(std::mem::size_of::<Bytes>()) =>
        {
            match unsafe {
                text_list_state::copy(
                    output.text.ptr.cast(),
                    output.text.len / std::mem::size_of::<Bytes>(),
                )
            } {
                Ok(value) => Value::TextList(value),
                Err(_) => return ERROR,
            }
        }
        TEXT => match unsafe { copy_bytes(output.text) } {
            Ok(v) => Value::Text(v),
            Err(_) => return ERROR,
        },
        _ => return ERROR,
    };
    *slot = Some(value);
    OK
}
/// All mutable host borrows end before calling into plugin code or recursing.
pub(super) fn evaluate(context: &Context, key: u64) -> Result<Arc<Value>, String> {
    let cached = {
        let turn = unsafe { &mut *context.turn };
        if let Some(fault) = &turn.graph.fault {
            return Err(fault.clone());
        }
        let Some(memo) = turn.graph.memos.get(&key) else {
            return Err(fail(
                context,
                &format!(
                    "unregistered computed {key:#018x}; register its spec in Actor::COMPUTEDS or use #[computed]"
                ),
            ));
        };
        if !memo.dirty {
            return memo
                .value
                .clone()
                .ok_or_else(|| "missing memo value".into());
        }
        if turn.graph.active.contains(&key) || turn.graph.active.len() >= 64 {
            let reason = if turn.graph.active.contains(&key) {
                "cyclic computation"
            } else {
                "computation depth limit exceeded (64)"
            };
            let chain = turn
                .graph
                .active
                .iter()
                .copied()
                .chain([key])
                .map(|identity| format!("{identity:#018x}"))
                .collect::<Vec<_>>()
                .join(" -> ");
            return Err(fail(context, &format!("{reason}: {chain}")));
        }
        let cached = memo.clone();
        turn.graph.active.push(key);
        cached
    };
    let result = (|| {
        for dependency in cached.dependencies.keys() {
            if let Dependency::Computed(key) = dependency {
                evaluate(context, *key)?;
            }
        }
        let unchanged = {
            let turn = unsafe { &*context.turn };
            cached.value.is_some()
                && cached
                    .dependencies
                    .iter()
                    .all(|(d, v)| revision(turn, *d) == *v)
        };
        if unchanged {
            unsafe { &mut *context.turn }
                .graph
                .memos
                .get_mut(&key)
                .unwrap()
                .dirty = false;
            return Ok(cached.value.clone().unwrap());
        }
        let (outer_reads, was_read_only) = {
            let turn = unsafe { &mut *context.turn };
            let reads = std::mem::take(&mut turn.reads);
            let read_only = turn.read_only;
            turn.read_only = true;
            (reads, read_only)
        };
        let mut value = None;
        let status = unsafe {
            ((*context.table).compute)(
                &context.api(),
                key,
                (&mut value as *mut Option<Value>).cast(),
                output,
            )
        };
        let (dependencies, fault) = {
            let turn = unsafe { &mut *context.turn };
            let reads = std::mem::replace(&mut turn.reads, outer_reads);
            turn.read_only = was_read_only;
            (versions(turn, &reads), turn.graph.fault.clone())
        };
        if let Some(fault) = fault {
            return Err(fault);
        }
        if status != OK {
            let reason = unsafe { &*context.turn }
                .handler_error
                .clone()
                .unwrap_or_else(|| "computed callback failed".into());
            return Err(fail(
                context,
                &format!("computed {key:#018x} evaluation failed: {reason}"),
            ));
        }
        let value = value.ok_or_else(|| {
            fail(
                context,
                &format!("computed {key:#018x} callback omitted its value"),
            )
        })?;
        if !matches!(
            (cached.kind, &value),
            (F64, Value::Float(_))
                | (I64, Value::Integer(_))
                | (BOOLEAN, Value::Boolean(_))
                | (TEXT, Value::Text(_))
                | (I64_LIST, Value::IntegerList(_))
                | (TEXT_LIST, Value::TextList(_))
        ) {
            return Err(fail(
                context,
                &format!("computed {key:#018x} result type mismatch"),
            ));
        }
        let equal = cached.value.as_deref() == Some(&value);
        let value = if equal {
            cached.value.unwrap()
        } else {
            Arc::new(value)
        };
        let turn = unsafe { &mut *context.turn };
        turn.graph.replace_dependencies(key, dependencies);
        let memo = turn.graph.memos.get_mut(&key).unwrap();
        memo.value = Some(value.clone());
        memo.dirty = false;
        memo.evaluations += 1;
        if !equal {
            memo.revision += 1;
        }
        Ok(value)
    })();
    let active = unsafe { &mut *context.turn }.graph.active.pop();
    debug_assert_eq!(active, Some(key));
    result
}
pub(super) fn eager(loaded: &Loaded, turn: &mut Turn) -> Result<(), String> {
    turn.graph.fault = None;
    turn.handler_error = None;
    let keys: Vec<_> = turn.graph.memos.keys().copied().collect();
    let context = Context {
        turn,
        table: loaded.table,
    };
    for key in keys {
        evaluate(&context, key)?;
    }
    Ok(())
}
pub(super) fn refreshed(
    loaded: &Loaded,
    turn: &mut Turn,
    dependencies: &BTreeSet<Dependency>,
) -> Result<BTreeMap<Dependency, u64>, String> {
    turn.graph.fault = None;
    turn.handler_error = None;
    let context = Context {
        turn,
        table: loaded.table,
    };
    for dependency in dependencies {
        if let Dependency::Computed(key) = dependency {
            evaluate(&context, *key)?;
        }
    }
    Ok(versions(turn, dependencies))
}
pub(super) unsafe extern "C" fn read_integer(ctx: *mut c_void, key: u64, out: *mut i64) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let Ok(value) = evaluate(context, key) else {
        return ERROR;
    };
    unsafe { &mut *context.turn }
        .reads
        .insert(Dependency::Computed(key));
    if let Value::Boolean(value) = &*value
        && !out.is_null()
    {
        unsafe {
            *out = i64::from(*value);
        }
        return OK;
    }
    if let Value::Integer(value) = &*value
        && !out.is_null()
    {
        unsafe { *out = *value };
        return OK;
    }
    fail(context, "computed integer read type mismatch");
    ERROR
}
pub(super) unsafe extern "C" fn read_string(
    ctx: *mut c_void,
    key: u64,
    out: *mut u8,
    capacity: usize,
) -> isize {
    let context = unsafe { &*ctx.cast::<Context>() };
    let Ok(value) = evaluate(context, key) else {
        return -1;
    };
    unsafe { &mut *context.turn }
        .reads
        .insert(Dependency::Computed(key));
    if let Value::Text(value) = &*value {
        if out.is_null() {
            return value.len() as isize;
        }
        if capacity >= value.len() {
            unsafe { std::ptr::copy_nonoverlapping(value.as_ptr(), out, value.len()) };
            return value.len() as isize;
        }
    }
    fail(context, "computed text read type mismatch or short buffer");
    -1
}

pub(super) unsafe extern "C" fn read_float(ctx: *mut c_void, key: u64, out: *mut f64) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let Ok(value) = evaluate(context, key) else {
        return ERROR;
    };
    if let Value::Float(value) = &*value
        && !out.is_null()
    {
        unsafe {
            *out = *value;
        }
        unsafe { &mut *context.turn }
            .reads
            .insert(Dependency::Computed(key));
        return OK;
    }
    fail(context, "computed float read type mismatch or null output");
    ERROR
}

pub(super) unsafe extern "C" fn read_list(
    ctx: *mut c_void,
    key: u64,
    out: *mut i64,
    capacity: usize,
) -> isize {
    let context = unsafe { &*ctx.cast::<Context>() };
    let Ok(value) = evaluate(context, key) else {
        return ERROR as isize;
    };
    unsafe { &mut *context.turn }
        .reads
        .insert(Dependency::Computed(key));
    if let Value::IntegerList(value) = &*value {
        let result = unsafe { list_state::output(value, out, capacity) };
        if result >= 0 {
            return result;
        }
    }
    fail(
        context,
        "computed list read type mismatch or invalid buffer",
    );
    ERROR as isize
}

pub(super) unsafe extern "C" fn read_list_range(
    ctx: *mut c_void,
    key: u64,
    start: usize,
    end: usize,
    out: *mut i64,
    capacity: usize,
) -> isize {
    let context = unsafe { &*ctx.cast::<Context>() };
    let Ok(value) = evaluate(context, key) else {
        return ERROR as isize;
    };
    unsafe { &mut *context.turn }
        .reads
        .insert(Dependency::Computed(key));
    if let Value::IntegerList(value) = &*value {
        let result = unsafe { list_state::output_range(value, start, end, out, capacity) };
        if result >= 0 {
            return result;
        }
    }
    fail(
        context,
        "computed list range type mismatch or invalid range/buffer",
    );
    ERROR as isize
}
pub(super) unsafe extern "C" fn read_text_list(
    ctx: *mut c_void,
    key: u64,
    index: usize,
    out: *mut u8,
    capacity: usize,
) -> isize {
    let context = unsafe { &*ctx.cast::<Context>() };
    let Ok(value) = evaluate(context, key) else {
        return ERROR as isize;
    };
    if let Value::TextList(value) = &*value {
        let result = unsafe { text_list_state::output(value, index, out, capacity) };
        if result >= 0 {
            unsafe { &mut *context.turn }
                .reads
                .insert(Dependency::Computed(key));
            return result;
        }
    }
    fail(
        context,
        "computed text list read type mismatch or invalid buffer",
    );
    ERROR as isize
}

#[cfg(test)]
mod index_tests {
    #[test]
    fn computed_list_output_copies_values_and_rejects_malformed_buffers() {
        let mut slot: Option<Value> = None;
        let mut source = vec![i64::MIN, 0, i64::MAX];
        let valid = Output {
            kind: I64_LIST,
            integer: 0,
            text: Bytes {
                ptr: source.as_ptr().cast(),
                len: source.len() * 8,
            },
        };
        assert_eq!(
            unsafe { output(std::ptr::from_mut(&mut slot).cast(), valid) },
            OK
        );
        source[0] = 12;
        drop(source);
        assert_eq!(slot, Some(Value::IntegerList(vec![i64::MIN, 0, i64::MAX])));
        let empty = Output {
            kind: I64_LIST,
            integer: 0,
            text: Bytes {
                ptr: std::ptr::null(),
                len: 0,
            },
        };
        assert_eq!(
            unsafe { output(std::ptr::from_mut(&mut slot).cast(), empty) },
            ERROR
        );
        assert_eq!(slot, Some(Value::IntegerList(vec![i64::MIN, 0, i64::MAX])));
        let mut empty_slot: Option<Value> = None;
        assert_eq!(
            unsafe { output(std::ptr::from_mut(&mut empty_slot).cast(), empty) },
            OK
        );
        assert_eq!(empty_slot, Some(Value::IntegerList(vec![])));
        let aligned = [0_i64];
        let misaligned = unsafe { aligned.as_ptr().cast::<u8>().add(1) };
        for (ptr, len) in [
            (std::ptr::null(), 8),
            (misaligned, 8),
            (aligned.as_ptr().cast(), 7),
            (std::ptr::null(), (MAX_LIST_ITEMS + 1) * 8),
        ] {
            let mut rejected: Option<Value> = None;
            let malformed = Output {
                kind: I64_LIST,
                integer: 0,
                text: Bytes { ptr, len },
            };
            assert_eq!(
                unsafe { output(std::ptr::from_mut(&mut rejected).cast(), malformed) },
                ERROR
            );
            assert!(rejected.is_none());
        }
    }

    #[test]
    fn text_list_output_owns_unicode_and_rejects_bad_descriptors() {
        let strings = ["标题😀".to_owned(), "a\0b".to_owned()];
        let descriptors: Vec<_> = strings.iter().map(|s| Bytes::new(s)).collect();
        let mut slot = None;
        let valid = Output {
            kind: TEXT_LIST,
            integer: 0,
            text: Bytes {
                ptr: descriptors.as_ptr().cast(),
                len: std::mem::size_of_val(descriptors.as_slice()),
            },
        };
        assert_eq!(
            unsafe { output(std::ptr::from_mut(&mut slot).cast(), valid) },
            OK
        );
        drop(descriptors);
        drop(strings);
        assert_eq!(
            slot,
            Some(Value::TextList(vec!["标题😀".into(), "a\0b".into()]))
        );
        for len in [
            1,
            std::mem::size_of::<Bytes>(),
            (MAX_LIST_ITEMS + 1) * std::mem::size_of::<Bytes>(),
        ] {
            let mut rejected: Option<Value> = None;
            let invalid = Output {
                kind: TEXT_LIST,
                integer: 0,
                text: Bytes {
                    ptr: std::ptr::null(),
                    len,
                },
            };
            assert_eq!(
                unsafe { output(std::ptr::from_mut(&mut rejected).cast(), invalid) },
                ERROR
            );
            assert!(rejected.is_none());
        }
    }

    use super::*;

    fn memo() -> Memo {
        Memo {
            kind: I64,
            value: None,
            dependencies: BTreeMap::new(),
            dirty: false,
            revision: 0,
            evaluations: 0,
        }
    }

    #[test]
    fn dynamic_dependency_replacement_prunes_old_edges_and_preserves_shared_readers() {
        let mut graph = Graph::default();
        for key in 1..=4 {
            graph.memos.insert(key, memo());
        }
        let a = Dependency::State(10);
        let b = Dependency::State(20);
        graph.replace_dependencies(1, BTreeMap::from([(a, 0)]));
        graph.replace_dependencies(2, BTreeMap::from([(a, 0)]));
        graph.replace_dependencies(3, BTreeMap::from([(Dependency::Computed(1), 0)]));
        graph.replace_dependencies(1, BTreeMap::from([(b, 0)]));
        graph.invalidate(a);
        assert!(!graph.memos[&1].dirty);
        assert!(graph.memos[&2].dirty);
        assert!(!graph.memos[&3].dirty);
        assert!(!graph.memos[&4].dirty);
        graph.invalidate(b);
        assert!(graph.memos[&1].dirty);
        assert!(graph.memos[&3].dirty);
        graph.replace_dependencies(2, BTreeMap::new());
        assert!(!graph.dependents.contains_key(&a));
        // Transaction snapshots own the index; changing a clone cannot prune live edges.
        let mut candidate = graph.clone();
        candidate.replace_dependencies(1, BTreeMap::new());
        assert!(!candidate.dependents.contains_key(&b));
        assert_eq!(graph.dependents[&b], BTreeSet::from([1]));
    }
    #[test]
    fn keyed_read_retention_skips_empty_cache_and_preserves_memo_dependencies() {
        use std::cell::Cell;
        let mut graph = Graph::default();
        let visits = Cell::new(0);
        graph.retain_map_reads((0..1000).map(|id| {
            visits.set(visits.get() + 1);
            Dependency::State(id)
        }));
        assert_eq!(visits.get(), 0, "empty cache must not scan bindings");
        // Admission still checks invalid identities even with an empty cache.
        assert!(graph.check_map_reads([Dependency::MapEntry(123)]).is_err());
        let visible = graph.map_reads.track(10, "visible", Some("one")).unwrap();
        let memo_read = graph.map_reads.track(10, "memo", Some("two")).unwrap();
        let retired = graph.map_reads.track(10, "retired", Some("three")).unwrap();
        graph.memos.insert(1, memo());
        graph.replace_dependencies(1, BTreeMap::from([(Dependency::MapEntry(memo_read), 0)]));
        graph.retain_map_reads([Dependency::MapEntry(visible)]);
        assert!(
            graph
                .check_map_reads([Dependency::MapEntry(visible)])
                .is_ok()
        );
        assert!(
            graph
                .check_map_reads([Dependency::MapEntry(retired)])
                .is_err()
        );
        graph.replace_dependencies(1, BTreeMap::new());
        graph.retain_map_reads([]);
        assert!(graph.map_reads.is_empty());
        let next = graph.map_reads.track(10, "visible", Some("new")).unwrap();
        assert!(
            next > retired,
            "retired read identities must never be reused"
        );
        graph.retain_map_reads([Dependency::MapEntry(next)]);
        assert!(graph.check_map_reads([Dependency::MapEntry(next)]).is_ok());
        assert!(
            graph
                .check_map_reads([Dependency::MapEntry(visible)])
                .is_err()
        );
    }
}
