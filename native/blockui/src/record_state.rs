use crate::{BoundState, Cx, Key, Record, Result, State, StateSpec, abi, record_codec};
use std::marker::PhantomData;

/// A generated typed projection. Host state owns the record; only selected field
/// bytes become a reactive dependency. Descriptors stay inside application code.
///
/// This example requires the `macros` feature for `#[derive(Record)]`.
/// ```
/// # #[cfg(feature = "macros")]
/// # fn main() {
/// use blockui::{Record, State, Cx, Result};
/// #[derive(Record, Default)]
/// struct Document { body: String, edits: i64 }
/// fn body(state: State<Document>, cx: &Cx<'_>) -> Result<String> {
///     Document::body_field().get(state, cx)
/// }
/// fn bound_body(state: State<Document>, cx: &Cx<'_>) -> Result<String> {
///     state.bind(cx)?.field(Document::body_field())
/// }
/// fn edit(state: State<Document>, cx: &Cx<'_>) -> Result<()> {
///     Document::body_field().set(state, cx, "Updated".into())?;
///     state.bind(cx)?.update_field(Document::edits_field(), |edits| {
///         *edits += 1;
///         Ok(())
///     })
/// }
/// # }
/// # #[cfg(not(feature = "macros"))]
/// # fn main() {}
/// ```
/// Field mutation borrows are temporary and cannot escape the edit:
/// ```compile_fail
/// use blockui::{Record, State, Cx, Result};
/// #[derive(Record, Default)]
/// struct Document { body: String }
/// fn escape(state: State<Document>, cx: &Cx<'_>) -> Result<()> {
///     let mut escaped = None;
///     Document::body_field().update(state, cx, |body| {
///         escaped = Some(body);
///         Ok(())
///     })
/// }
/// ```
pub struct RecordField<R, T> {
    pub(crate) index: usize,
    pub(crate) identity: u64,
    pub(crate) decode: fn(&str) -> Result<T>,
    pub(crate) project_mut: fn(&mut R) -> &mut T,
    marker: PhantomData<fn() -> R>,
}
impl<R: Record, T> RecordField<R, T> {
    #[doc(hidden)]
    pub fn __new(
        index: usize,
        identity: u64,
        decode: fn(&str) -> Result<T>,
        project_mut: fn(&mut R) -> &mut T,
    ) -> Self {
        Self {
            index,
            identity,
            decode,
            project_mut,
            marker: PhantomData,
        }
    }
    /// Read one field of the record with this integer business identity.
    /// Tracks only selected field bytes, independent of list position. Missing
    /// or duplicate matching identities reject the read. No context or decoder escapes.
    /// This example requires the `macros` feature for `#[derive(Record)]`.
    /// ```
    /// # #[cfg(feature = "macros")]
    /// # fn main() {
    /// use blockui::{Record, State, Cx, Result};
    /// #[derive(Record, Default)] struct Row { id: i64, title: String }
    /// fn title(rows: State<Vec<Row>>, cx: &Cx<'_>) -> Result<String> {
    ///     Row::title_field().get_from_list(rows, cx, Row::id_field(), 42)
    /// }
    /// # }
    /// # #[cfg(not(feature = "macros"))]
    /// # fn main() {}
    /// ```
    pub fn get_from_list(
        self,
        state: State<Vec<R>>,
        cx: &Cx<'_>,
        identity: RecordField<R, i64>,
        key: i64,
    ) -> Result<T> {
        let mut selector = "\n@record-list1\n".to_owned();
        record_codec::append(&mut selector, &Vec::<R>::FORMAT.to_owned())?;
        record_codec::append(&mut selector, &R::FORMAT.to_owned())?;
        record_codec::append(&mut selector, &identity.index.to_string())?;
        record_codec::append(&mut selector, &key)?;
        self.read_selected(state.key(), cx, &selector)
    }
    pub fn get(self, state: State<R>, cx: &Cx<'_>) -> Result<T> {
        self.read_selected(state.key(), cx, R::FORMAT)
    }
    fn read_selected(self, slot: u64, cx: &Cx<'_>, format: &str) -> Result<T> {
        let read = cx.api.read_record_field;
        let length = unsafe {
            read(
                cx.api.context,
                slot,
                abi::Bytes::new(format),
                self.index,
                std::ptr::null_mut(),
                0,
            )
        };
        if !(0..=1024 * 1024).contains(&length) {
            return Err(crate::Error::Host(abi::ERROR));
        }
        let mut bytes = vec![0; length as usize];
        let written = unsafe {
            read(
                cx.api.context,
                slot,
                abi::Bytes::new(format),
                self.index,
                bytes.as_mut_ptr(),
                bytes.len(),
            )
        };
        if written != length {
            return Err(crate::Error::Host(abi::ERROR));
        }
        let text = std::str::from_utf8(&bytes).map_err(|_| crate::Error::Host(abi::ERROR))?;
        (self.decode)(text)
    }
    /// Replace one field through the parent record's validation and transaction.
    pub fn set(self, state: State<R>, cx: &Cx<'_>, value: T) -> Result<()> {
        self.update(state, cx, |field| {
            *field = value;
            Ok(())
        })
    }
    /// Edit a transient field; publish only after both the closure and codec succeed.
    /// Same-parent reentrant writes are rejected by the parent record update.
    pub fn update(
        self,
        state: State<R>,
        cx: &Cx<'_>,
        update: impl FnOnce(&mut T) -> Result<()>,
    ) -> Result<()> {
        state.update(cx, |record| update((self.project_mut)(record)))
    }
}
/// Context-free text projection. The host copies only data from this descriptor.
///
/// This example requires the `macros` feature for `#[derive(Record)]`.
/// ```
/// # #[cfg(feature = "macros")]
/// # fn main() {
/// use blockui::{Record, State, RecordText, Text, TextInput, TextAction, Cx, Result};
/// #[derive(Record, Default)]
/// struct Document { body: String }
/// let document = State::<Document>::new("document");
/// let source = document.text(Document::body_field());
/// let label = Text::new(source);
/// let input = TextInput::field(source, TextAction::new("edit", |_, _| Ok(())));
/// fn from_bound(cx: &Cx<'_>) -> Result<RecordText> {
///     let document = State::<Document>::new("document").bind(cx)?;
///     Ok(document.text(Document::body_field())) // no context lifetime retained
/// }
/// # }
/// # #[cfg(not(feature = "macros"))]
/// # fn main() {}
/// ```
///
/// ```compile_fail
/// use blockui::{Record, State};
/// #[derive(Record, Default)]
/// struct Document { count: i64 }
/// State::<Document>::new("document").text(Document::count_field());
/// ```
#[derive(Clone, Copy)]
pub struct RecordText {
    pub(crate) slot: u64,
    pub(crate) format: &'static str,
    pub(crate) index: usize,
    pub(crate) identity: u64,
    pub(crate) list: Option<RecordListSelector>,
}
#[derive(Clone, Copy)]
pub(crate) struct RecordListSelector {
    format: &'static str,
    identity_index: usize,
    identity_name: u64,
    key: i64,
}
/// Text from a record selected by an integer business identity. The selector is
/// copied into the host; no decoder or context is retained across code replacement.
#[derive(Clone, Copy)]
pub struct RecordListText(pub(crate) RecordText);
impl<R: Record> RecordField<R, String> {
    /// Context-free text source selected by an integer business identity.
    /// This example requires the `macros` feature for `#[derive(Record)]`.
    /// ```
    /// # #[cfg(feature = "macros")]
    /// # fn main() {
    /// use blockui::{Record, State, Text, TextInput, TextAction};
    /// #[derive(Record, Default)] struct Row { id: i64, title: String }
    /// let rows = State::<Vec<Row>>::new("rows");
    /// let source = Row::title_field().text_from_list(rows, Row::id_field(), 42);
    /// let caption = Text::new(source);
    /// let editor = TextInput::field(source, TextAction::new("edit", |_, _| Ok(())));
    /// # }
    /// # #[cfg(not(feature = "macros"))]
    /// # fn main() {}
    /// ```
    pub fn text_from_list(
        self,
        state: State<Vec<R>>,
        identity: RecordField<R, i64>,
        key: i64,
    ) -> RecordListText {
        RecordListText(RecordText {
            slot: state.key(),
            format: Vec::<R>::FORMAT,
            index: self.index,
            identity: self.identity,
            list: Some(RecordListSelector {
                format: R::FORMAT,
                identity_index: identity.index,
                identity_name: identity.identity,
                key,
            }),
        })
    }
}
impl<R: Record> RecordField<R, bool> {
    /// A controlled boolean source selected by a stable integer record identity.
    /// Reordering the list does not change which record supplies the value.
    pub fn boolean_from_list(
        self,
        state: State<Vec<R>>,
        identity: RecordField<R, i64>,
        key: i64,
    ) -> RecordBoolean {
        RecordBoolean(RecordText {
            slot: state.key(),
            format: Vec::<R>::FORMAT,
            index: self.index,
            identity: self.identity,
            list: Some(RecordListSelector {
                format: R::FORMAT,
                identity_index: identity.index,
                identity_name: identity.identity,
                key,
            }),
        })
    }
}
impl RecordText {
    pub(crate) fn wire(self, label: &str) -> Result<String> {
        let Some(list) = self.list else {
            return Ok(format!("{}\n{}\n{label}", self.format, self.index));
        };
        if list.format.is_empty() || list.format.len() > 128 || list.format.contains('\n') {
            return Err(crate::Error::InvalidView(
                "invalid record list element format",
            ));
        }
        let mut selector = "\n@record-list1\n".to_owned();
        record_codec::append(&mut selector, &self.format.to_owned())?;
        record_codec::append(&mut selector, &list.format.to_owned())?;
        record_codec::append(&mut selector, &list.identity_index.to_string())?;
        record_codec::append(&mut selector, &list.key)?;
        selector.push_str(&format!("\n{}", self.index));
        Ok(format!(
            "\n@record-list-binding1\n{}\n{selector}{label}",
            selector.len()
        ))
    }
    pub(crate) fn identity_key(self, namespace: &str) -> Key {
        let identity = Key::named(namespace)
            .scoped(self.slot)
            .scoped(self.identity);
        match self.list {
            Some(list) => identity.scoped(list.identity_name).scoped(list.key as u64),
            None => identity,
        }
    }
}
mod scalar_sealed {
    pub trait Sealed {}
    impl Sealed for i64 {}
    impl Sealed for u64 {}
    impl Sealed for f64 {}
    impl Sealed for bool {}
}
/// Primitive record values with host-provided default text formatting.
pub trait RecordDisplayValue: scalar_sealed::Sealed {
    #[doc(hidden)]
    const KIND: usize;
}
impl RecordDisplayValue for i64 {
    const KIND: usize = abi::RECORD_DISPLAY_I64;
}
impl RecordDisplayValue for u64 {
    const KIND: usize = abi::RECORD_DISPLAY_U64;
}
impl RecordDisplayValue for f64 {
    const KIND: usize = abi::RECORD_DISPLAY_F64;
}
impl RecordDisplayValue for bool {
    const KIND: usize = abi::RECORD_DISPLAY_BOOL;
}
/// Read-only default-formatted scalar projection; no context or callback retained.
#[derive(Clone, Copy)]
pub struct RecordScalarText {
    pub(crate) value: RecordText,
    pub(crate) kind: usize,
}

/// Context-free boolean record source for a controlled checkbox.
#[derive(Clone, Copy)]
pub struct RecordBoolean(pub(crate) RecordText);
/// Context-free integer record source for controlled numeric widgets.
#[derive(Clone, Copy)]
pub struct RecordInteger(pub(crate) RecordText);
/// Context-free floating-point record source.
#[derive(Clone, Copy)]
pub struct RecordFloat(pub(crate) RecordText);

impl<T: Record> State<T> {
    /// Project an f64 field without retaining a context or decoder.
    /// ```compile_fail
    /// use blockui::{Record, State};
    /// #[derive(Record, Default)]
    /// #[record(name = "float.Settings")]
    /// struct Settings { volume: i64 }
    /// State::<Settings>::new("settings").float(Settings::volume_field());
    /// ```
    pub fn float(self, field: RecordField<T, f64>) -> RecordFloat {
        RecordFloat(RecordText {
            slot: self.key(),
            format: T::FORMAT,
            index: field.index,
            identity: field.identity,
            list: None,
        })
    }

    /// Project an i64 field without retaining a context or decoder.
    /// ```compile_fail
    /// use blockui::{Record, State};
    /// #[derive(Record, Default)]
    /// #[record(name = "integer.Settings")]
    /// struct Settings { enabled: bool }
    /// State::<Settings>::new("settings").integer(Settings::enabled_field());
    /// ```
    pub fn integer(self, field: RecordField<T, i64>) -> RecordInteger {
        RecordInteger(RecordText {
            slot: self.key(),
            format: T::FORMAT,
            index: field.index,
            identity: field.identity,
            list: None,
        })
    }

    /// Project a boolean field without retaining a context or decoder.
    /// ```compile_fail
    /// use blockui::{Record, State};
    /// #[derive(Record, Default)]
    /// #[record(name = "checkbox.Counter")]
    /// struct Counter { count: i64 }
    /// State::<Counter>::new("counter").boolean(Counter::count_field());
    /// ```
    pub fn boolean(self, field: RecordField<T, bool>) -> RecordBoolean {
        RecordBoolean(RecordText {
            slot: self.key(),
            format: T::FORMAT,
            index: field.index,
            identity: field.identity,
            list: None,
        })
    }

    /// Default text formatting for an i64, u64, f64 or bool record field.
    /// Formatting runs in the host; custom formatting belongs in a computed.
    ///
    /// This example requires the `macros` feature for `#[derive(Record)]`.
    /// ```
    /// # #[cfg(feature = "macros")]
    /// # fn main() {
    /// use blockui::{Record, State, Text};
    /// #[derive(Record, Default)]
    /// #[record(name = "display.Counter")]
    /// struct Counter { count: i64 }
    /// let state = State::<Counter>::new("counter");
    /// let label = Text::new(state.display(Counter::count_field()));
    /// # }
    /// # #[cfg(not(feature = "macros"))]
    /// # fn main() {}
    /// ```
    ///
    /// Scalar sources are read-only and cannot drive a text editor:
    /// ```compile_fail
    /// use blockui::{Record, State, TextInput};
    /// #[derive(Record, Default)]
    /// #[record(name = "display.Counter")]
    /// struct Counter { count: i64 }
    /// let state = State::<Counter>::new("counter");
    /// let editor = TextInput::new(state.display(Counter::count_field()));
    /// ```
    pub fn display<V: RecordDisplayValue>(self, field: RecordField<T, V>) -> RecordScalarText {
        RecordScalarText {
            value: RecordText {
                slot: self.key(),
                format: T::FORMAT,
                index: field.index,
                identity: field.identity,
                list: None,
            },
            kind: V::KIND,
        }
    }
    /// A context-free text source for labels and controlled text editors.
    pub fn text(self, field: RecordField<T, String>) -> RecordText {
        RecordText {
            slot: self.key(),
            format: T::FORMAT,
            index: field.index,
            identity: field.identity,
            list: None,
        }
    }
    /// A typed record slot backed by a host-owned canonical text envelope.
    /// Initialize it during cold migration before the first read. T itself is
    /// transient: no Rust allocation or destructor crosses the library boundary.
    pub const fn new(name: &'static str) -> Self {
        Self {
            spec: abi::SlotSpec::text_default(name, ""),
            marker: PhantomData,
        }
    }
    /// Both the value and its field-scoped format marker belong in actor STATE.
    /// A compatible implementation replacement leaves the encoded buffer intact.
    pub const fn schema(self) -> [StateSpec; 2] {
        assert!(
            !T::FORMAT.is_empty(),
            "record state requires a nonempty FORMAT"
        );
        [
            self.spec(),
            State::<i64>::new(T::FORMAT, 0)
                .scoped(Key(self.key()))
                .spec(),
        ]
    }
    fn storage(self) -> State<String> {
        State {
            spec: self.spec,
            marker: PhantomData,
        }
    }
    #[doc(hidden)]
    pub fn __initialize(self, cx: &Cx<'_>, default: impl FnOnce() -> T) -> Result<()> {
        let length =
            unsafe { (cx.api.read_text)(cx.api.context, self.key(), std::ptr::null_mut(), 0) };
        if length < 0 {
            return Err(crate::Error::Host(abi::ERROR));
        }
        if length == 0 && self.__previous_len(cx).is_err() {
            self.set(cx, &default())?;
        }
        Ok(())
    }
    #[doc(hidden)]
    pub fn __previous_len(self, cx: &Cx<'_>) -> Result<usize> {
        let length =
            unsafe { (cx.api.previous_text)(cx.api.context, self.key(), std::ptr::null_mut(), 0) };
        if length >= 0 {
            return Ok(length as usize);
        }
        // Distinguish a new field from a previous primitive field. A changed kind
        // also needs explicit migration, rather than silently accepting a default.
        let mut integer = 0;
        let mut float = 0.0;
        let exists = unsafe {
            (cx.api.previous_i64)(cx.api.context, self.key(), &mut integer) == abi::OK
                || (cx.api.previous_f64)(cx.api.context, self.key(), &mut float) == abi::OK
                || (cx.api.previous_list)(cx.api.context, self.key(), std::ptr::null_mut(), 0) >= 0
                || (cx.api.previous_text_list)(
                    cx.api.context,
                    self.key(),
                    usize::MAX,
                    std::ptr::null_mut(),
                    0,
                ) >= 0
        };
        if exists {
            Ok(0)
        } else {
            Err(crate::Error::Host(abi::ERROR))
        }
    }
    pub fn get(self, cx: &Cx<'_>) -> Result<T> {
        record_codec::decode_record(&self.storage().get(cx)?)
    }
    pub fn set(self, cx: &Cx<'_>, value: &T) -> Result<()> {
        let encoded = record_codec::encode_record(value)?;
        record_codec::decode_record::<T>(&encoded)?;
        self.storage().set(cx, &encoded)
    }
    /// Decode a previous record format inside explicit migration.
    pub fn previous<P: Record>(self, cx: &Cx<'_>) -> Result<P> {
        record_codec::decode_record(&self.storage().previous(cx)?)
    }
    #[doc(hidden)]
    pub fn __upgrade_previous(self, cx: &Cx<'_>) -> Result<()> {
        let encoded = self.storage().previous(cx)?;
        self.set(cx, &T::upgrade_encoded(&encoded)?)
    }
    /// Publish an edited temporary only after the closure and codec succeed.
    /// Reject a callback that changes this slot through another descriptor rather
    /// than overwriting its newer value. Other slot writes are allowed. This
    /// compares canonical bytes: restoring the original value is not a conflict.
    /// Propagate the error to roll back all writes in the enclosing actor turn.
    pub fn update(self, cx: &Cx<'_>, update: impl FnOnce(&mut T) -> Result<()>) -> Result<()> {
        let original = self.storage().get(cx)?;
        let mut value = record_codec::decode_record(&original)?;
        update(&mut value)?;
        if self.storage().get(cx)? != original {
            return Err(crate::Error::Application(
                "Record changed during update".into(),
            ));
        }
        self.set(cx, &value)
    }
}
impl<T: Record> BoundState<'_, '_, T> {
    /// Context-free floating-point source from this bound record.
    pub fn float(&self, field: RecordField<T, f64>) -> RecordFloat {
        self.state.float(field)
    }

    /// Context-free integer source from this bound record.
    pub fn integer(&self, field: RecordField<T, i64>) -> RecordInteger {
        self.state.integer(field)
    }

    /// Context-free boolean source from this bound record.
    pub fn boolean(&self, field: RecordField<T, bool>) -> RecordBoolean {
        self.state.boolean(field)
    }
    pub fn display<V: RecordDisplayValue>(&self, field: RecordField<T, V>) -> RecordScalarText {
        self.state.display(field)
    }
    pub fn set_field<V>(&self, field: RecordField<T, V>, value: V) -> Result<()> {
        field.set(self.state, self.cx, value)
    }
    pub fn update_field<V>(
        &self,
        field: RecordField<T, V>,
        update: impl FnOnce(&mut V) -> Result<()>,
    ) -> Result<()> {
        field.update(self.state, self.cx, update)
    }
    /// Copy only the descriptor; no turn context escapes into the returned source.
    pub fn text(&self, field: RecordField<T, String>) -> RecordText {
        self.state.text(field)
    }
    pub fn field<V>(&self, field: RecordField<T, V>) -> Result<V> {
        field.get(self.state, self.cx)
    }
    pub fn get(&self) -> Result<T> {
        self.state.get(self.cx)
    }
    pub fn set(&self, value: &T) -> Result<()> {
        self.state.set(self.cx, value)
    }
    pub fn update(&self, update: impl FnOnce(&mut T) -> Result<()>) -> Result<()> {
        self.state.update(self.cx, update)
    }
    pub fn previous<P: Record>(&self) -> Result<P> {
        self.state.previous(self.cx)
    }
}

mod text_source_sealed {
    pub trait Sealed {}
    impl Sealed for super::RecordText {}
    impl Sealed for super::RecordListText {}
    impl Sealed for super::RecordMapText {}
}
/// A data-only text field source accepted by controlled record editors.
pub trait RecordTextSource: text_source_sealed::Sealed {}
impl RecordTextSource for RecordText {}
impl RecordTextSource for RecordListText {}
impl RecordTextSource for RecordMapText {}
/// Context-free text field in one keyed record collection row. The host copies
/// the business key and selector; no decoder, context or callback is retained.
#[derive(Clone)]
pub struct RecordMapText {
    pub(crate) field: RecordText,
    pub(crate) key: String,
}
impl RecordMapText {
    pub(crate) fn identity_key(&self, namespace: &str) -> Key {
        self.field.identity_key(namespace).scoped_named(&self.key)
    }
}
