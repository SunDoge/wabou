//! Typed transient records over durable host-owned text storage.
use crate::{Cx, Error, Result, State, TextMap};
use std::marker::PhantomData;

/// Application-owned canonical codec. FORMAT identifies the stored format,
/// independent of implementation versions. Changing it requires migration.
/// In `#[actor]`, use `#[state(codec = Document)]` on the collection's
/// `State<Vec<String>>` field. This generates a per-collection format marker;
/// default migration rejects changes to an existing collection's codec even
/// when the view does not read it. Supply explicit migration for format changes.
/// Alternatively tie format changes to schema admission with a separate marker:
/// `#[state(key = Document::FORMAT)] codec: State<i64>`. Keep the collection's
/// own key unchanged so migration can read its previous entries. Use distinct,
/// namespaced format identities for independent codecs in the same actor.
/// Handwritten codecs define FORMAT explicitly. With the `macros` feature,
/// `#[derive(Record)]` generates canonical codecs for named String/i64/u64/f64/bool
/// fields, plus Option of those types, and fingerprints field names/types/order
/// automatically. None and Some(empty string) have distinct encodings. Its namespace
/// includes the module and struct name, so moving/renaming the type also changes
/// FORMAT. Declare the codec attribute or a schema marker as above; field changes
/// still need explicit migration, and compatible reload does not recode storage.
/// Add `#[record(name = "app.document")]` to decouple storage identity from
/// module/type renames. Keep this name stable and distinct for independent codecs.
/// `#[record(previous = OldDocument)]` generates RecordUpgrade<OldDocument>:
/// same-name fields move from the old value; mark added fields with
/// `#[record(default)]` or `#[record(default = expression)]`. Declared State<T>
/// default migration invokes it automatically, as does #[state(codec = T)] for
/// RecordMap collections. Handwritten Actor implementations call it explicitly.
/// Rust checks missing/type changes.
/// `Vec<T>` implements Record when T does, allowing `State<Vec<Row>>` directly.
/// Its format includes the element schema even when empty; an explicitly declared
/// previous element schema enables automatic per-element upgrades. Explicit
/// migrations can call `Vec::<NewRow>::upgrade(previous_rows)` when NewRow
/// implements RecordUpgrade<OldRow>; a failed element conversion rejects the
/// whole conversion before publishing collection state. The whole list
/// is one reactive slot, with at most 4096 entries and a 1 MiB canonical envelope.
/// Reads decode and edits encode the collection; compatible reload retains bytes.
/// Float fields encode exact IEEE bits, preserving signed zero and NaN payloads.
/// Native handles, references and plugin callbacks must not be encoded.
pub trait Record: Sized {
    const FORMAT: &'static str;
    /// Explicit predecessor identity, also checked for empty collections.
    const PREVIOUS_FORMAT: Option<&'static str> = None;
    fn encode(&self) -> Result<String>;
    fn decode(payload: &str) -> Result<Self>;
    /// Upgrade a canonical previous envelope during default declared-state
    /// migration. Derive generates this only for an explicitly named previous
    /// record; handwritten codecs may implement their own Rust conversion.
    fn upgrade_encoded(_encoded: &str) -> Result<Self> {
        Err(Error::Application(
            "Record codec changed; explicit migration is required".into(),
        ))
    }
}
/// Current-code conversion from a decoded previous format. Generated upgrades
/// move same-name fields and evaluate explicitly declared defaults. Call during
/// actor migration before publishing through RecordMap::set; no Rust value or
/// conversion callback is retained by the runtime.
pub trait RecordUpgrade<Previous>: Record {
    fn upgrade(previous: Previous) -> Result<Self>;
}
/// Records live as bounded owned text in the runtime; T exists only during a
/// current-code call. Compatible reload never serializes or recreates storage.
/// Reads decode records; edits encode them. `get` tracks one whole row; `field`
/// tracks only the selected field in that row. Collection enumeration tracks the slot.
pub struct RecordMap<T> {
    storage: TextMap,
    marker: PhantomData<T>,
}
impl<T: Record> RecordMap<T> {
    pub const fn new(state: State<Vec<String>>) -> Self {
        Self {
            storage: TextMap::new(state),
            marker: PhantomData,
        }
    }
    fn encode(record: &T) -> Result<String> {
        crate::record_codec::encode_record(record)
    }
    fn decode(encoded: &str) -> Result<T> {
        crate::record_codec::decode_record(encoded)
    }
    pub fn get(&self, cx: &Cx<'_>, key: &str) -> Result<Option<T>> {
        self.storage
            .get(cx, key)?
            .map(|text| Self::decode(&text))
            .transpose()
    }
    /// Create a context-free text/editor source for one existing collection row.
    /// A visible binding to a missing row rejects projection; remove its view
    /// when removing the row. Edits go through the supplied current text action.
    ///
    /// This example requires the `macros` feature for `#[derive(Record)]`.
    /// ```
    /// # #[cfg(feature = "macros")]
    /// # fn main() {
    /// use blockui::{Record, RecordMap, RecordMapText, State, Text, TextInput,
    ///     TextArea, TextAction, Cx, Result, Error};
    /// #[derive(Record)] struct Document { title: String }
    /// fn rename(cx: &Cx<'_>, text: String) -> Result<()> {
    ///     let documents = RecordMap::<Document>::new(State::<Vec<String>>::new("documents"));
    ///     if !documents.edit_field(cx, "42", Document::title_field(), |title| {
    ///         *title = text; Ok(())
    ///     })? { return Err(Error::Application("Document removed".into())); }
    ///     Ok(())
    /// }
    /// let documents = RecordMap::<Document>::new(State::<Vec<String>>::new("documents"));
    /// let source = documents.text("42", Document::title_field());
    /// let action = TextAction::new("rename", rename);
    /// let input: TextInput<RecordMapText> = TextInput::field(source.clone(), action);
    /// let area = TextArea::field(source.clone(), action);
    /// let preview = Text::new(source);
    /// # }
    /// # #[cfg(not(feature = "macros"))]
    /// # fn main() {}
    /// ```
    /// ```compile_fail
    /// use blockui::{Record, RecordMap, State};
    /// #[derive(Record)] struct Document { count: i64 }
    /// RecordMap::<Document>::new(State::<Vec<String>>::new("documents"))
    ///     .text("42", Document::count_field());
    /// ```
    pub fn text(
        &self,
        key: impl Into<String>,
        field: crate::RecordField<T, String>,
    ) -> crate::RecordMapText {
        crate::RecordMapText {
            field: crate::RecordText {
                slot: self.storage.state.key(),
                format: T::FORMAT,
                index: field.index,
                identity: field.identity,
                list: None,
            },
            key: key.into(),
        }
    }
    /// Read one generated typed field. Absence is a tracked dependency, so a
    /// later insertion invalidates it. Sibling fields and other rows do not.
    /// The host validates record framing; the current plugin decodes the field.
    ///
    /// This example requires the `macros` feature for `#[derive(Record)]`.
    /// ```
    /// # #[cfg(feature = "macros")]
    /// # fn main() {
    /// use blockui::{Record, RecordMap, State, Cx, Result};
    /// #[derive(Record)]
    /// struct Document { title: String, saved: bool }
    /// fn title(cx: &Cx<'_>) -> Result<Option<String>> {
    ///     let documents = RecordMap::<Document>::new(State::<Vec<String>>::new("documents"));
    ///     documents.field(cx, "42", Document::title_field())
    /// }
    /// fn rename(cx: &Cx<'_>) -> Result<bool> {
    ///     let documents = RecordMap::<Document>::new(State::<Vec<String>>::new("documents"));
    ///     documents.edit_field(cx, "42", Document::title_field(), |title| {
    ///         *title = "Updated".into();
    ///         Ok(())
    ///     })
    /// }
    /// # }
    /// # #[cfg(not(feature = "macros"))]
    /// # fn main() {}
    /// ```
    /// A field from a different record cannot be applied to this collection:
    /// ```compile_fail
    /// use blockui::{Record, RecordMap, State, Cx, Result};
    /// #[derive(Record)] struct Document { title: String }
    /// #[derive(Record)] struct User { title: String }
    /// fn wrong(cx: &Cx<'_>) -> Result<Option<String>> {
    ///     RecordMap::<Document>::new(State::<Vec<String>>::new("documents"))
    ///         .field(cx, "42", User::title_field())
    /// }
    /// ```
    pub fn field<V>(
        &self,
        cx: &Cx<'_>,
        key: &str,
        field: crate::RecordField<T, V>,
    ) -> Result<Option<V>> {
        let read = cx.api.read_record_map_field;
        let length = unsafe {
            read(
                cx.api.context,
                self.storage.state.key(),
                crate::abi::Bytes::new(key),
                crate::abi::Bytes::new(T::FORMAT),
                field.index,
                std::ptr::null_mut(),
                0,
            )
        };
        if length == crate::abi::MAP_ABSENT {
            return Ok(None);
        }
        if !(0..=1024 * 1024).contains(&length) {
            return Err(Error::Host(crate::abi::ERROR));
        }
        let mut bytes = vec![0; length as usize];
        let written = unsafe {
            read(
                cx.api.context,
                self.storage.state.key(),
                crate::abi::Bytes::new(key),
                crate::abi::Bytes::new(T::FORMAT),
                field.index,
                bytes.as_mut_ptr(),
                bytes.len(),
            )
        };
        if written != length {
            return Err(Error::Host(crate::abi::ERROR));
        }
        let text = std::str::from_utf8(&bytes).map_err(|_| Error::Host(crate::abi::ERROR))?;
        (field.decode)(text).map(Some)
    }
    /// Edit one field through the whole record codec and the existing keyed-edit
    /// conflict/rollback rules. Missing rows return false without calling edit.
    pub fn edit_field<V>(
        &self,
        cx: &Cx<'_>,
        key: &str,
        field: crate::RecordField<T, V>,
        edit: impl FnOnce(&mut V) -> Result<()>,
    ) -> Result<bool> {
        self.edit(cx, key, |record| edit((field.project_mut)(record)))
    }
    pub fn insert(&self, cx: &Cx<'_>, key: &str, record: &T) -> Result<()> {
        let encoded = Self::encode(record)?;
        Self::decode(&encoded)?;
        self.storage.insert(cx, key, &encoded)?;
        Ok(())
    }
    /// Edit one existing record using ordinary Rust fields. Missing keys return
    /// false without invoking the callback. A callback or codec error publishes
    /// no record change. The callback and decoded value never leave this call.
    /// Other actor writes performed by the callback follow the enclosing turn's
    /// transaction rules; propagate errors to roll those writes back too.
    /// If the callback changes/removes this key, returns an error instead of
    /// overwriting its newer value. Writes to other keys are preserved. This
    /// compares stored values, not epochs: restoring the exact original bytes
    /// does not cause a conflict.
    pub fn edit(
        &self,
        cx: &Cx<'_>,
        key: &str,
        edit: impl FnOnce(&mut T) -> Result<()>,
    ) -> Result<bool> {
        let Some(original) = self.storage.get(cx, key)? else {
            return Ok(false);
        };
        let mut record = Self::decode(&original)?;
        edit(&mut record)?;
        if self.storage.get(cx, key)?.as_deref() != Some(original.as_str()) {
            return Err(Error::Application("Record changed during edit".into()));
        }
        self.insert(cx, key, &record)?;
        Ok(true)
    }
    /// Decode once into a temporary Rust collection and publish once on
    /// success. Neither callback nor T values survive the current call.
    pub fn update(
        &self,
        cx: &Cx<'_>,
        edit: impl FnOnce(&mut std::collections::BTreeMap<String, T>) -> Result<()>,
    ) -> Result<()> {
        self.storage.update(cx, |stored| {
            let mut records = Self::decode_update(stored)?;
            edit(&mut records)?;
            *stored = Self::encode_update(records)?;
            Ok(())
        })
    }
    fn decode_update(
        stored: &std::collections::BTreeMap<String, String>,
    ) -> Result<std::collections::BTreeMap<String, T>> {
        stored
            .iter()
            .map(|(key, encoded)| Ok((key.clone(), Self::decode(encoded)?)))
            .collect()
    }
    fn encode_update(
        records: std::collections::BTreeMap<String, T>,
    ) -> Result<std::collections::BTreeMap<String, String>> {
        let mut encoded = std::collections::BTreeMap::new();
        for (key, record) in records {
            let value = Self::encode(&record)?;
            Self::decode(&value)?;
            encoded.insert(key, value);
        }
        Ok(encoded)
    }
    /// Replace all records without decoding their previous format. Use this
    /// after reading an old codec's previous_entries in explicit migration.
    pub fn set(&self, cx: &Cx<'_>, records: std::collections::BTreeMap<String, T>) -> Result<()> {
        let mut encoded = std::collections::BTreeMap::new();
        for (key, record) in records {
            let value = Self::encode(&record)?;
            Self::decode(&value)?;
            encoded.insert(key, value);
        }
        self.storage.update(cx, |stored| {
            *stored = encoded;
            Ok(())
        })
    }
    pub fn remove(&self, cx: &Cx<'_>, key: &str) -> Result<()> {
        self.storage.remove(cx, key)?;
        Ok(())
    }
    /// Enumerate canonical business keys without decoding the record values.
    /// A Computed<Vec<String>> over these keys separates list membership from
    /// individual row fields; field edits can then update only their consumers.
    pub fn keys(&self, cx: &Cx<'_>) -> Result<Vec<String>> {
        self.storage.keys(cx)
    }
    pub fn entries(&self, cx: &Cx<'_>) -> Result<Vec<(String, T)>> {
        self.storage
            .entries(cx)?
            .into_iter()
            .map(|(key, text)| Ok((key, Self::decode(&text)?)))
            .collect()
    }
    pub fn previous_entries(&self, cx: &Cx<'_>) -> Result<Vec<(String, T)>> {
        self.storage
            .previous_entries(cx)?
            .into_iter()
            .map(|(key, text)| Ok((key, Self::decode(&text)?)))
            .collect()
    }
    /// Upgrade every previous-format record during actor migration. Previous
    /// reads are rejected by the host outside migration. Decode/conversion must
    /// all succeed before publishing; the enclosing migration transaction also
    /// rolls back subsequent failures. Compatible reload does not invoke this.
    pub fn upgrade_previous<P: Record>(&self, cx: &Cx<'_>) -> Result<()>
    where
        T: RecordUpgrade<P>,
    {
        let records = self
            .storage
            .previous_entries(cx)?
            .into_iter()
            .map(|(key, text)| Ok((key, T::upgrade(RecordMap::<P>::decode(&text)?)?)))
            .collect::<Result<std::collections::BTreeMap<_, _>>>()?;
        self.set(cx, records)
    }
    #[doc(hidden)]
    pub fn __upgrade_previous(&self, cx: &Cx<'_>) -> Result<()> {
        if T::PREVIOUS_FORMAT.is_none() {
            return Err(Error::Application(
                "Record codec changed; explicit migration is required".into(),
            ));
        }
        let records = self
            .storage
            .previous_entries(cx)?
            .into_iter()
            .map(|(key, text)| Ok((key, T::upgrade_encoded(&text)?)))
            .collect::<Result<std::collections::BTreeMap<_, _>>>()?;
        self.set(cx, records)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Number(i64);
    impl Record for Number {
        const FORMAT: &'static str = "number.v1";
        fn encode(&self) -> Result<String> {
            Ok(self.0.to_string())
        }
        fn decode(payload: &str) -> Result<Self> {
            payload
                .parse()
                .map(Self)
                .map_err(|_| Error::Application("Invalid number".into()))
        }
    }
    #[test]
    fn format_and_canonical_payload_are_checked() {
        assert_eq!(RecordMap::<Number>::decode("number.v1\n42").unwrap().0, 42);
        for bad in ["42", "number.v2\n42", "number.v1\n042", "number.v1\nx"] {
            assert!(RecordMap::<Number>::decode(bad).is_err());
        }
        assert_eq!(
            RecordMap::<Number>::encode(&Number(-1)).unwrap(),
            "number.v1\n-1"
        );
    }
}
