//! Keyed text values in one runtime-owned slot; no Rust map crosses the ABI.
use crate::{Cx, Error, Result, State, abi};

/// A bounded dictionary backed by a text-list state slot. The schema encoding is
/// sorted alternating key/value strings. Keys are nonempty exact UTF-8 strings;
/// no case folding or Unicode normalization is implicit. At most 2048 entries
/// and 1 MiB aggregate UTF-8 are accepted. Compatible reload keeps host storage.
/// get() reads one value and tracks that key, including absence. Enumeration and
/// edits still copy the bounded collection. Typed codecs build on RecordMap.
#[derive(Clone, Copy)]
pub struct TextMap {
    pub(crate) state: State<Vec<String>>,
}
impl TextMap {
    pub const fn new(state: State<Vec<String>>) -> Self {
        Self { state }
    }
    fn validate(entries: &[String]) -> Result<()> {
        if !entries.len().is_multiple_of(2)
            || entries.len() > abi::MAX_LIST_ITEMS
            || entries.iter().map(String::len).sum::<usize>() > abi::MAX_TEXT_LIST_BYTES
        {
            return Err(Error::Application(
                "Invalid text map size or encoding".into(),
            ));
        }
        let mut previous: Option<&str> = None;
        for pair in entries.as_chunks::<2>().0 {
            let key = pair[0].as_str();
            if key.is_empty() || previous.is_some_and(|old| old >= key) {
                return Err(Error::Application(
                    "Text map keys must be nonempty, unique and sorted".into(),
                ));
            }
            previous = Some(key);
        }
        Ok(())
    }
    fn position(entries: &[String], key: &str) -> std::result::Result<usize, usize> {
        let mut low = 0;
        let mut high = entries.len() / 2;
        while low < high {
            let mid = low + (high - low) / 2;
            match entries[mid * 2].as_str().cmp(key) {
                std::cmp::Ordering::Less => low = mid + 1,
                std::cmp::Ordering::Greater => high = mid,
                std::cmp::Ordering::Equal => return Ok(mid * 2),
            }
        }
        Err(low * 2)
    }
    /// Edit a temporary ordinary Rust map and publish once on success. The
    /// callback runs synchronously and is never retained across replacement.
    /// Errors publish no batch changes; writes made directly by the callback
    /// still follow the enclosing actor transaction. Propagate errors to reject
    /// those writes too. Changing this collection during the callback returns
    /// an error instead of overwriting newer values. Comparison uses values,
    /// so restoring the exact original collection does not conflict.
    pub fn update(
        &self,
        cx: &Cx<'_>,
        edit: impl FnOnce(&mut std::collections::BTreeMap<String, String>) -> Result<()>,
    ) -> Result<()> {
        let original = self.state.get(cx)?;
        let mut map: std::collections::BTreeMap<_, _> = Self::decode_entries(original.clone())?
            .into_iter()
            .collect();
        edit(&mut map)?;
        self.publish_update(cx, original, map)
    }
    fn publish_update(
        &self,
        cx: &Cx<'_>,
        original: Vec<String>,
        map: std::collections::BTreeMap<String, String>,
    ) -> Result<()> {
        if self.state.get(cx)? != original {
            return Err(Error::Application("Text map changed during update".into()));
        }
        if map.len() > abi::MAX_LIST_ITEMS / 2 {
            return Err(Error::Application("Text map exceeds entry limit".into()));
        }
        let bytes = map.iter().try_fold(0_usize, |total, (key, value)| {
            if key.is_empty() {
                return None;
            }
            total.checked_add(key.len())?.checked_add(value.len())
        });
        if bytes.is_none_or(|bytes| bytes > abi::MAX_TEXT_LIST_BYTES) {
            return Err(Error::Application(
                "Text map exceeds key or byte limits".into(),
            ));
        }
        let mut entries = Vec::with_capacity(map.len() * 2);
        for (key, value) in map {
            entries.push(key);
            entries.push(value);
        }
        self.state.set(cx, &entries)
    }
    /// Copy entries in canonical key order for keyed view projections. Strings
    /// move from the host read buffer into pairs without another string clone.
    pub fn entries(&self, cx: &Cx<'_>) -> Result<Vec<(String, String)>> {
        Self::decode_entries(self.state.get(cx)?)
    }
    /// Return keys in canonical order without returning values. This reads the
    /// bounded collection; use a Computed<Vec<String>> to suppress structural
    /// invalidation when values change but membership stays the same.
    pub fn keys(&self, cx: &Cx<'_>) -> Result<Vec<String>> {
        let entries = self.state.get(cx)?;
        Self::validate(&entries)?;
        Ok(entries.into_iter().step_by(2).collect())
    }
    /// Read canonical old entries during an explicit schema migration. The
    /// underlying state token may name an old/renamed slot. Outside migration,
    /// this follows State::previous's error behavior.
    pub fn previous_entries(&self, cx: &Cx<'_>) -> Result<Vec<(String, String)>> {
        Self::decode_entries(self.state.previous(cx)?)
    }
    fn decode_entries(entries: Vec<String>) -> Result<Vec<(String, String)>> {
        Self::validate(&entries)?;
        let mut pairs = Vec::with_capacity(entries.len() / 2);
        let mut strings = entries.into_iter();
        while let Some(key) = strings.next() {
            let value = strings.next().expect("validated paired text map storage");
            pairs.push((key, value));
        }
        Ok(pairs)
    }
    pub fn get(&self, cx: &Cx<'_>, key: &str) -> Result<Option<String>> {
        if key.is_empty() {
            return Err(Error::Application("Text map key must be nonempty".into()));
        }
        let read = cx.api.read_text_map;
        let length = unsafe {
            read(
                cx.api.context,
                self.state.key(),
                abi::Bytes::new(key),
                std::ptr::null_mut(),
                0,
            )
        };
        if length == abi::MAP_ABSENT {
            return Ok(None);
        }
        if length < 0 || length as usize > abi::MAX_TEXT_LIST_BYTES {
            return Err(Error::Host(abi::ERROR));
        }
        let mut bytes = vec![0; length as usize];
        let copied = unsafe {
            read(
                cx.api.context,
                self.state.key(),
                abi::Bytes::new(key),
                bytes.as_mut_ptr(),
                bytes.len(),
            )
        };
        if copied != length {
            return Err(Error::Host(abi::ERROR));
        }
        Ok(Some(
            String::from_utf8(bytes).map_err(|_| Error::Host(abi::ERROR))?,
        ))
    }
    /// Returns the previous value. Failed publication leaves the slot unchanged;
    /// propagate errors to roll back any other writes in the actor turn.
    pub fn insert(&self, cx: &Cx<'_>, key: &str, value: &str) -> Result<Option<String>> {
        if key.is_empty()
            || key
                .len()
                .checked_add(value.len())
                .is_none_or(|bytes| bytes > abi::MAX_TEXT_LIST_BYTES)
        {
            return Err(Error::Application("Text map entry exceeds limits".into()));
        }
        let mut entries = self.state.get(cx)?;
        Self::validate(&entries)?;
        let previous = match Self::position(&entries, key) {
            Ok(index) => Some(std::mem::replace(&mut entries[index + 1], value.into())),
            Err(index) => {
                entries.splice(index..index, [key.into(), value.into()]);
                None
            }
        };
        Self::validate(&entries)?;
        self.state.set(cx, &entries)?;
        Ok(previous)
    }
    pub fn remove(&self, cx: &Cx<'_>, key: &str) -> Result<Option<String>> {
        let mut entries = self.state.get(cx)?;
        Self::validate(&entries)?;
        let Ok(index) = Self::position(&entries, key) else {
            return Ok(None);
        };
        let value = entries.remove(index + 1);
        entries.remove(index);
        self.state.set(cx, &entries)?;
        Ok(Some(value))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| (*s).into()).collect()
    }
    #[test]
    fn canonical_storage_and_lookup() {
        let entries = strings(&["a", "", "标题", "🦀"]);
        assert!(TextMap::validate(&entries).is_ok());
        assert_eq!(TextMap::position(&entries, "a"), Ok(0));
        assert_eq!(TextMap::position(&entries, "标题"), Ok(2));
        assert_eq!(TextMap::position(&entries, "b"), Err(2));
        for invalid in [
            &["a"][..],
            &["", "value"],
            &["a", "1", "a", "2"],
            &["b", "1", "a", "2"],
        ] {
            assert!(TextMap::validate(&strings(invalid)).is_err());
        }
        assert!(
            TextMap::validate(&strings(&["a", &"x".repeat(abi::MAX_TEXT_LIST_BYTES)])).is_err()
        );
    }
}
