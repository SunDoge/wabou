//! Host-owned keyed read dependencies. IDs are interned, never hash-derived.
use crate::{Context, Dependency, Value};
use blockui_abi::{Bytes, ERROR, MAX_LIST_ITEMS, MAX_TEXT_LIST_BYTES};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::c_void,
    sync::Arc,
};
type RecordSelector = (Arc<str>, usize);
type ReadIdentity = (u64, Arc<str>, Option<RecordSelector>);

#[derive(Clone)]
struct Entry {
    slot: u64,
    key: Arc<str>,
    field: Option<RecordSelector>,
    value: Result<Option<Arc<str>>, ()>,
    revision: u64,
}
#[derive(Clone, Default)]
pub(super) struct Cache {
    next_id: u64,
    entries: BTreeMap<u64, Entry>,
    identities: BTreeMap<ReadIdentity, u64>,
}
fn valid(values: &[String]) -> bool {
    values.len().is_multiple_of(2)
        && values.len() <= MAX_LIST_ITEMS
        && values.iter().map(String::len).sum::<usize>() <= MAX_TEXT_LIST_BYTES
        && values
            .as_chunks::<2>()
            .0
            .iter()
            .all(|pair| !pair[0].is_empty())
        && values
            .as_chunks::<2>()
            .0
            .windows(2)
            .all(|pair| pair[0][0] < pair[1][0])
}
fn lookup<'a>(values: &'a [String], key: &str) -> Option<&'a str> {
    let pairs = values.as_chunks::<2>().0;
    pairs
        .binary_search_by(|pair| pair[0].as_str().cmp(key))
        .ok()
        .map(|index| pairs[index][1].as_str())
}
impl Cache {
    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    fn bytes(&self) -> usize {
        self.entries
            .values()
            .map(|entry| {
                entry.key.len()
                    + entry.field.as_ref().map_or(0, |(format, _)| format.len())
                    + entry
                        .value
                        .as_ref()
                        .ok()
                        .and_then(Option::as_ref)
                        .map_or(0, |value| value.len())
            })
            .sum()
    }
    pub(super) fn track(&mut self, slot: u64, key: &str, value: Option<&str>) -> Result<u64, ()> {
        self.track_field(slot, key, None, value)
    }
    fn track_field(
        &mut self,
        slot: u64,
        key: &str,
        field: Option<RecordSelector>,
        value: Option<&str>,
    ) -> Result<u64, ()> {
        if let Some(id) = self.identities.get(&(slot, Arc::from(key), field.clone())) {
            return Ok(*id);
        }
        if self.entries.len() >= 2 * MAX_LIST_ITEMS
            || self
                .bytes()
                .saturating_add(key.len())
                .saturating_add(field.as_ref().map_or(0, |(format, _)| format.len()))
                .saturating_add(value.map_or(0, str::len))
                > 8 * MAX_TEXT_LIST_BYTES
        {
            return Err(());
        }
        let id = self.next_id.checked_add(1).ok_or(())?;
        self.next_id = id;
        let key: Arc<str> = Arc::from(key);
        self.identities
            .insert((slot, key.clone(), field.clone()), id);
        self.entries.insert(
            id,
            Entry {
                slot,
                key,
                field,
                value: Ok(value.map(Arc::from)),
                revision: 0,
            },
        );
        Ok(id)
    }
    pub(super) fn validate_live(&self, live: &BTreeSet<u64>) -> Result<(), String> {
        if live.iter().any(|id| !self.entries.contains_key(id)) {
            return Err("Published keyed dependency is missing".into());
        }
        let bytes: usize = live
            .iter()
            .filter_map(|id| self.entries.get(id))
            .map(|entry| {
                entry.key.len()
                    + entry.field.as_ref().map_or(0, |(format, _)| format.len())
                    + entry
                        .value
                        .as_ref()
                        .ok()
                        .and_then(Option::as_ref)
                        .map_or(0, |value| value.len())
            })
            .sum();
        if live.len() > MAX_LIST_ITEMS || bytes > 4 * MAX_TEXT_LIST_BYTES {
            return Err("Published keyed dependencies exceed cache budget".into());
        }
        Ok(())
    }
    /// Only published dependencies survive turn completion. IDs are never reused.
    pub(super) fn retain(&mut self, live: &BTreeSet<u64>) {
        self.entries.retain(|id, _| live.contains(id));
        self.identities.retain(|_, id| live.contains(id));
    }
    pub(super) fn revision(&self, id: u64) -> u64 {
        self.entries.get(&id).map_or(0, |entry| entry.revision)
    }
    pub(super) fn sources(&self, slots: &BTreeSet<u64>) -> Vec<Dependency> {
        self.entries
            .iter()
            .filter(|(_, entry)| slots.contains(&entry.slot))
            .map(|(id, _)| Dependency::MapEntry(*id))
            .collect()
    }
    /// Stage cache updates so rejected writes leave both values and versions intact.
    pub(super) fn refresh(&mut self, slot: u64, values: &[String]) -> Result<Vec<Dependency>, ()> {
        let canonical = valid(values);
        self.refresh_with(slot, |entry| {
            if canonical {
                let record = lookup(values, &entry.key);
                match (&entry.field, record) {
                    (Some((format, index)), Some(record)) => {
                        crate::record_fields::lookup(record, &format!("{format}\n{index}"))
                            .map(Some)
                    }
                    _ => Ok(record),
                }
            } else {
                Err(())
            }
        })
    }
    pub(super) fn refresh_record(
        &mut self,
        slot: u64,
        encoded: &str,
    ) -> Result<Vec<Dependency>, ()> {
        self.refresh_with(slot, |entry| {
            crate::record_fields::lookup(encoded, &entry.key).map(Some)
        })
    }
    fn refresh_with<'a>(
        &mut self,
        slot: u64,
        lookup: impl Fn(&Entry) -> Result<Option<&'a str>, ()>,
    ) -> Result<Vec<Dependency>, ()> {
        if !self.entries.values().any(|entry| entry.slot == slot) {
            return Ok(Vec::new());
        }
        let mut next = self.clone();
        let mut changed = Vec::new();
        for (id, entry) in &mut next.entries {
            if entry.slot != slot {
                continue;
            }
            let borrowed = lookup(entry);
            let unchanged = match (&entry.value, &borrowed) {
                (Ok(old), Ok(new)) => old.as_deref() == *new,
                (Err(()), Err(())) => true,
                _ => false,
            };
            if unchanged {
                continue;
            }
            let value = borrowed.map(|value| value.map(Arc::from));
            if value != entry.value {
                entry.value = value;
                entry.revision = entry.revision.checked_add(1).ok_or(())?;
                changed.push(Dependency::MapEntry(*id));
            }
        }
        if next.bytes() > 8 * MAX_TEXT_LIST_BYTES {
            return Err(());
        }
        *self = next;
        Ok(changed)
    }
}
pub(super) unsafe extern "C" fn read(
    ctx: *mut c_void,
    slot: u64,
    key: Bytes,
    out: *mut u8,
    capacity: usize,
) -> isize {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    let Ok(key) = (unsafe { crate::copy_bytes(key) }) else {
        return ERROR as isize;
    };
    if key.is_empty() {
        return ERROR as isize;
    }
    let Some(Value::TextList(values)) = turn.storage.slots.get(&slot).map(Box::as_ref) else {
        return ERROR as isize;
    };
    if !valid(values) {
        return ERROR as isize;
    }
    let value = lookup(values, &key);
    let Ok(id) = turn.graph.map_reads.track(slot, &key, value) else {
        return ERROR as isize;
    };
    turn.reads.insert(Dependency::MapEntry(id));
    let Some(value) = value else {
        return blockui_abi::MAP_ABSENT;
    };
    if out.is_null() {
        return value.len() as isize;
    }
    if capacity < value.len() {
        return ERROR as isize;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(value.as_ptr(), out, value.len());
    }
    value.len() as isize
}

/// Field identity is a separate cache key from the raw whole-row identity; even
/// adversarial key strings cannot alias a generated selector. No decoder escapes.
pub(super) unsafe extern "C" fn read_record_field(
    ctx: *mut c_void,
    slot: u64,
    key: Bytes,
    format: Bytes,
    index: usize,
    out: *mut u8,
    capacity: usize,
) -> isize {
    if format.len == 0 || format.len > 128 {
        return ERROR as isize;
    }
    let Ok(format) = (unsafe { crate::copy_bytes(format) }) else {
        return ERROR as isize;
    };
    let Ok(key) = (unsafe { crate::copy_bytes(key) }) else {
        return ERROR as isize;
    };
    if key.is_empty() || format.contains('\n') {
        return ERROR as isize;
    }
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    let Ok(value) = selected_field(turn, slot, &key, &format, index) else {
        return ERROR as isize;
    };
    let Some(value) = value else {
        return blockui_abi::MAP_ABSENT;
    };
    if out.is_null() {
        return value.len() as isize;
    }
    if capacity < value.len() {
        return ERROR as isize;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(value.as_ptr(), out, value.len());
    }
    value.len() as isize
}

pub(super) fn selected_field<'a>(
    turn: &'a mut crate::Turn,
    slot: u64,
    key: &str,
    format: &str,
    index: usize,
) -> Result<Option<&'a str>, ()> {
    let Some(Value::TextList(values)) = turn.storage.slots.get(&slot).map(Box::as_ref) else {
        return Err(());
    };
    if !valid(values) {
        return Err(());
    }
    let value = lookup(values, key)
        .map(|encoded| crate::record_fields::lookup(encoded, &format!("{format}\n{index}")))
        .transpose()?;
    let id =
        turn.graph
            .map_reads
            .track_field(slot, key, Some((Arc::from(format), index)), value)?;
    turn.reads.insert(Dependency::MapEntry(id));
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn row_and_field_identities_do_not_alias_and_refresh_independently() {
        let mut cache = Cache::default();
        let whole = cache
            .track(1, "key\nschema\n0", Some("schema\n1\nx1\ny"))
            .unwrap();
        let name = cache
            .track_field(
                1,
                "key\nschema\n0",
                Some((Arc::from("schema"), 0)),
                Some("x"),
            )
            .unwrap();
        let other = cache
            .track_field(
                1,
                "key\nschema\n0",
                Some((Arc::from("schema"), 1)),
                Some("y"),
            )
            .unwrap();
        assert_ne!(whole, name);
        assert_ne!(name, other);
        let changed = cache
            .refresh(1, &["key\nschema\n0".into(), "schema\n1\nx1\nz".into()])
            .unwrap();
        assert_eq!(
            changed,
            [Dependency::MapEntry(whole), Dependency::MapEntry(other)]
        );
        assert_eq!(cache.revision(name), 0);
        let changed = cache.refresh(1, &[]).unwrap();
        assert!(changed.contains(&Dependency::MapEntry(name)));
        assert_eq!(cache.entries[&name].value, Ok(None));
        assert_eq!(
            cache
                .refresh(1, &["key\nschema\n0".into(), "broken".into()])
                .unwrap()
                .len(),
            3
        );
        assert!(cache.entries[&name].value.is_err());
    }
    #[test]
    fn field_abi_rejects_bad_framing_and_small_output_without_writing() {
        let mut turn = crate::Turn::default();
        turn.storage.slots.insert(
            1,
            Box::new(Value::TextList(vec![
                "a\n🦀".into(),
                "schema\n4\n🦀0\n".into(),
            ])),
        );
        let context = Context {
            turn: &mut turn,
            table: std::ptr::null(),
        };
        let ctx = (&context as *const Context).cast_mut().cast();
        let mut output = [0x55; 4];
        let read = |key: &str, format: &str, index, out, capacity| unsafe {
            read_record_field(
                ctx,
                1,
                Bytes::new(key),
                Bytes::new(format),
                index,
                out,
                capacity,
            )
        };
        assert_eq!(read("a\n🦀", "schema", 0, std::ptr::null_mut(), 0), 4);
        assert_eq!(
            read("a\n🦀", "schema", 0, output.as_mut_ptr(), 3),
            ERROR as isize
        );
        assert_eq!(output, [0x55; 4]);
        assert_eq!(read("a\n🦀", "schema", 0, output.as_mut_ptr(), 4), 4);
        assert_eq!(&output, "🦀".as_bytes());
        assert_eq!(read("a\n🦀", "schema", 1, std::ptr::null_mut(), 0), 0);
        assert_eq!(
            read("missing", "schema", 0, std::ptr::null_mut(), 0),
            blockui_abi::MAP_ABSENT
        );
        for format in ["", "wrong", "schema\n"] {
            assert_eq!(
                read("a\n🦀", format, 0, output.as_mut_ptr(), 4),
                ERROR as isize
            );
        }
        assert_eq!(
            read("", "schema", 0, output.as_mut_ptr(), 4),
            ERROR as isize
        );
        assert_eq!(
            read("a\n🦀", "schema", usize::MAX, output.as_mut_ptr(), 4),
            ERROR as isize
        );
        turn.storage.slots.insert(
            1,
            Box::new(Value::TextList(vec![
                "a\n🦀".into(),
                "schema\n4\n🦀bad-tail".into(),
            ])),
        );
        assert_eq!(
            read("a\n🦀", "schema", 0, output.as_mut_ptr(), 4),
            ERROR as isize
        );
        assert_eq!(&output, "🦀".as_bytes());
    }
    #[test]
    fn published_budget_counts_only_live_entries_and_checks_missing_ids() {
        let mut cache = Cache::default();
        let mut live = BTreeSet::new();
        for index in 0..MAX_LIST_ITEMS + 1 {
            live.insert(cache.track(1, &index.to_string(), None).unwrap());
        }
        assert!(cache.validate_live(&live).is_err());
        live.pop_last();
        assert!(cache.validate_live(&live).is_ok());
        cache.retain(&live);
        assert!(cache.validate_live(&BTreeSet::from([u64::MAX])).is_err());

        let mut cache = Cache::default();
        let value = "x".repeat(MAX_TEXT_LIST_BYTES - 1);
        let mut live = BTreeSet::new();
        for slot in 0..4 {
            live.insert(cache.track(slot, "k", Some(&value)).unwrap());
        }
        assert!(cache.validate_live(&live).is_ok());
        let extra = cache.track(4, "k", None).unwrap();
        live.insert(extra);
        assert!(cache.validate_live(&live).is_err());
        live.remove(&extra);
        assert!(cache.validate_live(&live).is_ok());
    }
    #[test]
    fn cache_identity_and_revisions_follow_keys_not_positions_or_slots() {
        let mut cache = Cache::default();
        let a = cache.track(1, "key", Some("value")).unwrap();
        let b = cache.track(2, "key", Some("other")).unwrap();
        assert_ne!(a, b);
        assert_eq!(cache.track(1, "key", Some("value")).unwrap(), a);
        assert!(
            cache
                .refresh(
                    1,
                    &["earlier".into(), "x".into(), "key".into(), "value".into()]
                )
                .unwrap()
                .is_empty()
        );
        assert_eq!(cache.refresh(1, &[]).unwrap(), [Dependency::MapEntry(a)]);
        assert_eq!(cache.revision(a), 1);
        assert_eq!(cache.revision(b), 0);
        assert_eq!(
            cache.refresh(1, &["key".into(), "new".into()]).unwrap(),
            [Dependency::MapEntry(a)]
        );
        assert_eq!(cache.revision(a), 2);
        assert_eq!(
            cache.refresh(1, &["broken".into()]).unwrap(),
            [Dependency::MapEntry(a)]
        );
    }
    #[test]
    fn retired_identities_are_reclaimed_without_reusing_dependency_ids() {
        let mut cache = Cache::default();
        let old = cache.track(1, "old", Some("old value")).unwrap();
        let retained = cache.track(1, "live", Some("live value")).unwrap();
        cache.retain(&BTreeSet::from([retained]));
        assert_eq!(cache.entries.len(), 1);
        assert_eq!(cache.identities.len(), 1);
        assert_eq!(
            cache.track(1, "live", Some("live value")).unwrap(),
            retained
        );
        let new = cache.track(1, "new", None).unwrap();
        assert!(new > retained && new != old);
        cache.retain(&BTreeSet::new());
        assert!(cache.entries.is_empty());
        assert!(cache.identities.is_empty());
    }
    #[test]
    fn cache_count_and_byte_budgets_fail_without_partial_revision_changes() {
        let mut cache = Cache::default();
        for index in 0..2 * MAX_LIST_ITEMS {
            cache.track(1, &index.to_string(), None).unwrap();
        }
        assert!(cache.track(1, "overflow", None).is_err());
        assert_eq!(cache.entries.len(), 2 * MAX_LIST_ITEMS);
        let mut cache = Cache::default();
        let original_length = 8 * MAX_TEXT_LIST_BYTES / 9 - 8;
        for slot in 0..9 {
            cache
                .track(slot, "key", Some(&"x".repeat(original_length)))
                .unwrap();
        }
        let revision = cache.revision(1);
        assert!(
            cache
                .refresh(0, &["key".into(), "x".repeat(MAX_TEXT_LIST_BYTES - 3)])
                .is_err()
        );
        assert_eq!(cache.revision(1), revision);
        assert_eq!(
            cache.entries[&1]
                .value
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap()
                .len(),
            original_length
        );
    }
}
