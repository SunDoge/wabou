//! Safe, typed application API over a synchronous versioned actor boundary.
//! Durable state belongs to the host. Application definitions and views are ephemeral.
extern crate self as blockui;
#[doc(hidden)]
pub use blockui_abi as abi;
pub use blockui_abi::canvas::{CanvasScene, DrawCircle};
pub use blockui_abi::grid::{GridExtent, GridMove};
pub use ui_contract::Request as ServiceRequest;
#[doc(hidden)]
#[path = "event_registry.rs"]
pub mod __events;
mod bound_router;
mod choices;
mod disclosure;
mod export;
mod labels;
mod menu;
mod popover;
pub use menu::{ContextMenu, DropdownMenu, MenuEntry, MenuItem, Menubar};
mod pagination;
pub use pagination::Pagination;
mod table;
mod tooltip;
pub use labels::{Badge, BadgeVariant, Separator};
pub use popover::{Popover, PopoverPlacement};
pub use table::{Table, TableRow, TableSortDirection};
pub use tooltip::Tooltip;
mod toast;
pub use disclosure::{Accordion, AccordionItem, Collapsible};
pub use toast::{Toast, ToastEntry, ToastPosition, ToastVariant, Toasts};
mod components;
pub use choices::{ChoiceOption, RadioGroup, Switch, Tab, Tabs};
mod computed_output;
#[doc(hidden)]
pub use computed_output::ComputedOutput as __ComputedOutput;
mod prefixed_text;
mod style_map;
mod ui_scope;
pub use components::{
    ActionGroup, Alert, Card, Description, EmptyState, Heading, ValidationMessage,
};
#[doc(hidden)]
pub use ui_scope::{FinishView as __FinishView, finish_view as __finish_view};
pub use ui_scope::{Ui, UiScope};
#[doc(hidden)]
pub mod record_codec;
mod record_list;
mod record_map;
mod record_state;
pub use record_state::{
    RecordBoolean, RecordDisplayValue, RecordField, RecordFloat, RecordInteger, RecordListText,
    RecordMapText, RecordScalarText, RecordText, RecordTextSource,
};
#[doc(hidden)]
pub mod route_codec;
mod router;
pub use bound_router::{BoundPathRouter, BoundRouter};
mod socket;
pub use socket::Socket;
mod bound_resource;
mod bound_task;
pub use bound_resource::BoundResource;
pub use bound_task::BoundTask;
mod captured_text_map;
mod text_map;
mod try_text_map;
pub use record_map::{Record, RecordMap, RecordUpgrade};
pub use text_map::TextMap;
mod virtual_rows;
use blockui_abi::{HostApi, SlotSpec};
pub use blockui_abi::{IntegerRange, LocalPoint, RequestError};
#[cfg(feature = "macros")]
/// Derive full-path encoding and typed segment parsing for unit/named-field enums.
/// Parameters use `Display` and `FromStr`; optional guards are normal Rust expressions.
/// Overlapping matches and noncanonical paths are rejected. Paths are a durable
/// contract: changing them requires explicit migration of stored history.
///
/// ```
/// use blockui::RoutePath;
/// #[derive(Clone, Debug, Default, PartialEq, Eq, RoutePath)]
/// enum Page {
///     #[default]
///     #[route(path = "/")]
///     Home,
///     #[route(path = "/projects/{project}/documents/{id}", guard = id > 0)]
///     Document { project: u64, id: i64 },
/// }
/// let page = Page::Document { project: 2, id: 42 };
/// assert_eq!(Page::from_path(&page.to_path()), Some(page));
/// #[blockui::actor(name = "application")]
/// struct Application {
///     #[state(route = Page)]
///     history: blockui::State<Vec<String>>,
/// }
/// ```
/// Handwritten routes used in state annotations must provide a nonempty FORMAT:
/// ```compile_fail
/// use blockui::{actor, RoutePath, State};
/// #[derive(Clone, Default, PartialEq, Eq)]
/// struct Page;
/// impl RoutePath for Page {
///     fn to_path(&self) -> String { "/".into() }
///     fn from_path(path: &str) -> Option<Self> { (path == "/").then_some(Self) }
/// }
/// #[actor]
/// struct Bad { #[state(route = Page)] history: State<Vec<String>> }
/// ```
pub use blockui_macros::RoutePath;
/// Experimental XML-style UI syntax; requires the `rsx` feature.
#[cfg(feature = "rsx")]
pub use blockui_macros::rsx;
#[cfg(feature = "macros")]
/// Declare typed state tokens and generate SCHEMA/new/Default without owning durable data.
/// Defaults remain normal constant Rust expressions and are type-checked by rustc.
///
/// ```compile_fail
/// use blockui::{actor, State};
/// #[actor]
/// struct Bad { #[state(default = "not an integer")] count: State<i64> }
/// ```
///
/// ```compile_fail
/// use blockui::{actor, Cx};
/// #[actor]
/// struct Captured { context: Cx<'static> }
/// ```
///
/// ```compile_fail
/// use blockui::{actor, Cx};
/// #[actor]
/// struct BadRecord { #[state(record)] context: Cx<'static> }
/// ```
///
/// Unconditional fields must have distinct durable identities. Mutually exclusive
/// conditional declarations are validated against the active schema at load time.
/// ```compile_fail
/// use blockui::{actor, State};
/// #[actor]
/// struct Duplicate {
///     #[state(key = "same")]
///     first: State<i64>,
///     #[state(key = "same")]
///     second: State<bool>,
/// }
/// ```
/// Conditional task/resource fields must also have unique keys when enabled.
/// ```compile_fail
/// use blockui::{actor, Task};
/// #[actor]
/// struct DuplicateTasks {
///     #[cfg(all())] #[task(key = "same")] first: Task,
///     #[cfg(all())] #[task(key = "same")] second: Task,
/// }
/// ```
/// ```compile_fail
/// use blockui::{actor, Resource, File};
/// #[actor]
/// struct DuplicateFiles {
///     #[cfg(all())] #[resource(key = "same")] first: Resource<File>,
///     #[cfg(all())] #[resource(key = "same")] second: Resource<File>,
/// }
/// ```
/// ```
/// use blockui::{actor, Task};
/// #[actor]
/// struct Exclusive {
///     #[cfg(all())] #[task(key = "same")] active: Task,
///     #[cfg(any())] #[task(key = "same")] absent: Task,
/// }
/// let _ = Exclusive::new();
/// ```
/// `#[scope]` declares an independently cached structural boundary. The macro
/// generates its stable identity, registry entry and `method_scope()` accessor;
/// state reads are collected dynamically when the builder runs. Conditional
/// branches replace the builder's dependency set with the reads actually made.
/// Direct reads in the parent view belong to the parent, so put structural reads
/// inside the scope method when only that subtree should rebuild. Reactive text
/// and style inside the subtree retain their own independent subscriptions.
///
/// A scope builder may return a single widget. The SDK wraps leaf roots in a
/// zero-padding, zero-gap Column, preserving the widget identity and bindings.
/// Return an explicit container when you need to control the subtree layout.
///
/// Action method doc comments also appear on the generated action descriptor
/// accessors and `Actor::actions()` fields, so editor hover explains the handler
/// without invoking it. Generated descriptors dispatch through the current
/// implementation; they do not retain a handler callback in host state.
///
/// In render mode, `#[action(bound)]` methods run on the turn-bound receiver.
/// The generated action adapter binds runtime-owned slots only for this call;
/// the method cannot be async or retain the turn context beyond its lifetime.
///
/// ```
/// use blockui::*;
/// #[actor]
/// struct Counter {
///     count: State<i64>,
/// }
/// #[actor_impl(render)]
/// impl Counter {
///     #[scope(bound)]
///     #[blockui::view]
///     fn count_panel(&self, ui: &Ui) -> View {
///         #[node] {
///             Column::default();
///             #[node] Text::new(&self.count);
///         }
///     }
///     #[computed(bound)]
///     fn double(&self) -> Result<i64> {
///         self.count.get().checked_mul(2).ok_or(Error::Overflow)
///     }
///     #[action(bound)]
///     fn increment(&self) -> Result<()> {
///         self.count
///             .update(|value| value.checked_add(1).ok_or(Error::Overflow))
///     }
/// }
/// impl Render for CounterBound<'_, '_> {
///     #[blockui::view]
///     fn render(&self, ui: &Ui) -> View {
///         #[node] {
///             Column::default();
///             #[node] self.count_panel_scope();
///             #[node] Text::new(self.double_computed());
///             #[node] Button::new("Increment", self.increment_action());
///         }
///     }
/// }
/// ```
/// Cached getters such as `self.double()?` are also available on bound receivers.
/// The receiver borrows only the current turn and cannot escape into durable state.
/// ```compile_fail
/// use blockui::*;
/// #[actor]
/// struct Counter { count: State<i64> }
/// fn escape<'cx, 'host>(bound: CounterBound<'cx, 'host>) -> CounterBound<'static, 'static> {
///     bound
/// }
/// ```
#[cfg(feature = "macros")]
pub use blockui_macros::{
    Record, ViewKey, actor, actor_impl, app, app_impl, component, component_impl, ui, view,
};
#[doc(hidden)]
pub use router::__validate_pending_route;
pub use router::{Navigation, PathRouter, Route, RoutePath, Router};
pub use virtual_rows::{RowSpan, VirtualRows};
/// A validated static schema descriptor; application code cannot forge borrowed pointers.
///
/// ```compile_fail
/// use blockui::{StateSpec, abi::SlotSpec};
/// let forged = StateSpec(SlotSpec::text("unvalidated"));
/// ```
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct StateSpec(SlotSpec);
impl StateSpec {
    pub const fn key(self) -> u64 {
        self.0.key
    }
    /// Explicit identity migration. Compatible reloads never call this path.
    /// Existing current-identity slots win; absent predecessor slots keep defaults.
    pub fn copy_previous_from(self, cx: &Cx<'_>, previous: Key) -> Result<()> {
        macro_rules! copy {
            ($ty:ty) => {{
                let target = State::<$ty> {
                    spec: self.0,
                    marker: PhantomData,
                };
                if target.previous(cx).is_ok() {
                    return Ok(());
                }
                let mut source = target;
                source.spec.key = previous.0;
                if let Ok(value) = source.previous(cx) {
                    target.set(cx, &value)?;
                }
            }};
            (scalar $ty:ty) => {{
                let target = State::<$ty> {
                    spec: self.0,
                    marker: PhantomData,
                };
                if target.previous(cx).is_ok() {
                    return Ok(());
                }
                let mut source = target;
                source.spec.key = previous.0;
                if let Ok(value) = source.previous(cx) {
                    target.set(cx, value)?;
                }
            }};
        }
        match self.0.kind {
            abi::I64 => copy!(scalar i64),
            abi::F64 => copy!(scalar f64),
            abi::BOOLEAN => copy!(scalar bool),
            abi::TEXT => copy!(String),
            abi::I64_LIST => copy!(Vec<i64>),
            abi::TEXT_LIST => copy!(Vec<String>),
            _ => return Err(Error::Host(abi::ERROR)),
        }
        Ok(())
    }
}
/// Validate active host descriptor identities after cfg filtering.
#[doc(hidden)]
pub const fn __validate_host_keys(keys: &[u64]) {
    let mut first = 0;
    while first < keys.len() {
        let mut second = first + 1;
        while second < keys.len() {
            assert!(
                keys[first] != keys[second],
                "duplicate active host descriptor key"
            );
            second += 1;
        }
        first += 1;
    }
}
/// Macro support for flattening host-owned field schemas, including nested records.
#[doc(hidden)]
pub const fn __schema_len(parts: &[&[StateSpec]]) -> usize {
    let mut total = 0;
    let mut index = 0;
    while index < parts.len() {
        total += parts[index].len();
        index += 1;
    }
    total
}
#[doc(hidden)]
pub const fn __merge_schema<const N: usize>(parts: &[&[StateSpec]]) -> [StateSpec; N] {
    assert!(N == __schema_len(parts));
    let mut fields = [StateSpec(SlotSpec::integer("unused", 0)); N];
    let mut part = 0;
    let mut out = 0;
    while part < parts.len() {
        let mut field = 0;
        while field < parts[part].len() {
            fields[out] = parts[part][field];
            out += 1;
            field += 1;
        }
        part += 1;
    }
    fields
}
#[doc(hidden)]
pub const fn __scope_schema<const N: usize>(fields: &[StateSpec], scope: Key) -> [StateSpec; N] {
    assert!(fields.len() == N);
    let mut out = [StateSpec(SlotSpec::integer("unused", 0)); N];
    let mut index = 0;
    while index < N {
        out[index] = fields[index];
        out[index].0.key = scope.scoped(out[index].0.key).0;
        index += 1;
    }
    out
}
use std::{
    marker::PhantomData,
    panic::{AssertUnwindSafe, catch_unwind},
};

/// An SDK operation failure. Integrates with application error reporting via `?`.
///
/// ```
/// fn application() -> std::result::Result<(), Box<dyn std::error::Error>> {
///     let operation: blockui::Result<()> = Err(blockui::Error::Overflow);
///     operation?;
///     Ok(())
/// }
/// assert_eq!(application().unwrap_err().to_string(), "actor value overflow");
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Host(i32),
    /// A live slot is missing or has a different kind than its descriptor.
    StateValidation {
        field: Option<&'static str>,
        key: u64,
        kind: u32,
        status: i32,
    },
    InvalidView(&'static str),
    UnknownAction,
    Overflow,
    Io(String),
    /// Recoverable application validation or domain failure.
    /// The normal turn rollback applies; this string stays inside the plugin.
    Application(String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Host(code) => write!(formatter, "runtime operation rejected (code {code})"),
            Self::StateValidation {
                field,
                key,
                kind,
                status,
            } => {
                if let Some(field) = field {
                    write!(formatter, "binding {field}: ")?;
                }
                let expected = match *kind {
                    abi::I64 => "i64",
                    abi::TEXT => "String",
                    abi::BOOLEAN => "bool",
                    abi::I64_LIST => "Vec<i64>",
                    abi::F64 => "f64",
                    abi::TEXT_LIST => "Vec<String>",
                    _ => "unknown kind",
                };
                write!(
                    formatter,
                    "state {key:#018x} validation failed: expected {expected} (kind {kind}), slot missing or kind mismatch (code {status})"
                )
            }
            Self::InvalidView(reason) => write!(formatter, "invalid actor view: {reason}"),
            Self::UnknownAction => formatter.write_str("unknown actor action"),
            Self::Overflow => formatter.write_str("actor value overflow"),
            Self::Io(reason) => write!(formatter, "actor I/O operation failed: {reason}"),
            Self::Application(reason) => formatter.write_str(reason),
        }
    }
}
impl Error {
    #[doc(hidden)]
    pub fn __binding_field(mut self, name: &'static str) -> Self {
        if let Self::StateValidation { field, .. } = &mut self {
            // Preserve the innermost field when binding nested components.
            if field.is_none() {
                *field = Some(name);
            }
        }
        self
    }
}
impl std::error::Error for Error {}

impl From<i32> for Error {
    fn from(code: i32) -> Self {
        Self::Host(code)
    }
}
pub type Result<T> = std::result::Result<T, Error>;

/// Data-only component instance path. No context or plugin code is retained.
#[doc(hidden)]
#[derive(Clone, Copy, Default)]
pub struct InstancePath {
    keys: [u64; 16],
    len: usize,
}
impl InstancePath {
    pub const EMPTY: Self = Self {
        keys: [0; 16],
        len: 0,
    };
    pub const fn scoped(mut self, scope: Key) -> Self {
        assert!(
            self.len < 16,
            "component nesting exceeds 16 identity scopes"
        );
        self.keys[self.len] = scope.0;
        self.len += 1;
        self
    }
    pub const fn resolve(self, mut key: u64) -> u64 {
        let mut index = 0;
        while index < self.len {
            key = Key(self.keys[index]).scoped(key).0;
            index += 1;
        }
        key
    }
    pub fn scopes(&self) -> impl Iterator<Item = Key> + '_ {
        self.keys[..self.len].iter().copied().map(Key)
    }
}
/// Borrowed access to runtime services for exactly one synchronous actor turn.
/// It cannot be constructed by application code or sent to a worker thread.
///
/// ```compile_fail
/// use blockui::{Cx, State};
/// fn escape(cx: &Cx<'_>) {
///     std::thread::scope(|scope| {
///         scope.spawn(move || State::<i64>::new("count", 0).get(cx));
///     });
/// }
/// ```
pub struct Cx<'a> {
    api: &'a HostApi,
    instance: InstancePath,
}
impl Cx<'_> {
    /// Call a typed business operation implemented by the stable host.
    pub fn call<R: ui_contract::Request>(&self, request: R) -> Result<R::Response> {
        let input = ui_contract::encode(&request).map_err(Error::Application)?;
        let bytes = self.service(R::id(), &input)?;
        let result: std::result::Result<R::Response, String> =
            ui_contract::decode(&bytes).map_err(Error::Application)?;
        result.map_err(Error::Application)
    }

    /// Invoke a stable application service once. No Rust object crosses the UI ABI.
    /// Business effects are external to UI state transactions and are not rolled back.
    pub fn service(&self, id: u64, request: &[u8]) -> Result<Vec<u8>> {
        let mut output = vec![0; abi::MAX_MESSAGE_TEXT_BYTES];
        let length = unsafe {
            (self.api.call_service)(
                self.api.context,
                id,
                abi::Bytes {
                    ptr: request.as_ptr(),
                    len: request.len(),
                },
                output.as_mut_ptr(),
                output.len(),
            )
        };
        if length < 0 || length as usize > output.len() {
            return Err(Error::Host(abi::ERROR));
        }
        output.truncate(length as usize);
        Ok(output)
    }

    #[doc(hidden)]
    pub fn __instance(&self) -> InstancePath {
        self.instance
    }
    #[doc(hidden)]
    pub fn __child(&self, key: Key) -> Cx<'_> {
        let mut path = InstancePath::EMPTY.scoped(key);
        for scope in self.instance.scopes() {
            path = path.scoped(scope);
        }
        Cx {
            api: self.api,
            instance: path,
        }
    }
    /// Host-owned monotonic milliseconds, valid only in actions.
    /// Persist baselines in State; the epoch survives reload but not process restart.
    /// Views/computed functions must project state rather than read this clock.
    pub fn monotonic_ms(&self) -> Result<u64> {
        let mut value = 0;
        let code = unsafe { (self.api.monotonic_ms)(self.api.context, &mut value) };
        if code == abi::OK {
            Ok(value)
        } else {
            Err(Error::Host(code))
        }
    }

    /// Owned diagnostic for the current SDK request-error completion, if provided.
    /// Available only during that completion turn; timeout/cancellation have no handler text.
    pub fn request_error(&self) -> Result<Option<String>> {
        let read = self.api.read_request_error;
        let length = unsafe { read(self.api.context, std::ptr::null_mut(), 0) };
        if !(0..=4096).contains(&length) {
            return Err(Error::Host(abi::ERROR));
        }
        if length == 0 {
            return Ok(None);
        }
        let mut bytes = vec![0; length as usize];
        if unsafe { read(self.api.context, bytes.as_mut_ptr(), bytes.len()) } != length {
            return Err(Error::Host(abi::ERROR));
        }
        String::from_utf8(bytes)
            .map(Some)
            .map_err(|_| Error::Host(abi::ERROR))
    }

    /// Stage a scalar reply for the current request. Published only after commit.
    /// Ordinary tell handlers and a second reply in the same turn return an error.
    pub fn reply(&self, value: i64) -> Result<()> {
        let status = unsafe { (self.api.reply)(self.api.context, value) };
        if status == abi::OK {
            Ok(())
        } else {
            Err(Error::Host(status))
        }
    }

    /// Stage a bounded text reply; ordinary tells, scalar requests and duplicate replies reject.
    /// Borrowed bytes are copied by the host and published only after successful commit.
    pub fn reply_text(&self, value: &str) -> Result<()> {
        if value.len() > abi::MAX_MESSAGE_TEXT_BYTES {
            return Err(Error::Host(abi::ERROR));
        }
        let code = unsafe { (self.api.reply_text)(self.api.context, abi::Bytes::new(value)) };
        if code == abi::OK {
            Ok(())
        } else {
            Err(Error::Host(code))
        }
    }
    fn message_text(&self) -> Result<String> {
        let read = self.api.read_message_text;
        let length = unsafe { read(self.api.context, std::ptr::null_mut(), 0) };
        if length < 0 || length as usize > abi::MAX_MESSAGE_TEXT_BYTES {
            return Err(Error::Host(abi::ERROR));
        }
        let mut bytes = vec![0; length as usize];
        if unsafe { read(self.api.context, bytes.as_mut_ptr(), bytes.len()) } != length {
            return Err(Error::Host(abi::ERROR));
        }
        String::from_utf8(bytes).map_err(|_| Error::Host(abi::ERROR))
    }

    /// Resolve a committed direct child by its declaration key during an action.
    /// Missing children return None; query again after a removed child is recreated.
    pub fn child(&self, key: Key) -> Result<Option<ActorRef>> {
        let mut id = 0;
        let status = unsafe { (self.api.child_actor)(self.api.context, key.0, &mut id) };
        if status != abi::OK {
            return Err(Error::Host(status));
        }
        Ok((id != 0).then_some(ActorRef(id)))
    }
}

/// A durable slot's identity and default, independent of a particular turn or code version.
pub struct State<T> {
    spec: SlotSpec,
    marker: PhantomData<fn() -> T>,
}
impl<T> Copy for State<T> {}
impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> State<T> {
    /// Bind an exact slot identity to this synchronous turn without reading it.
    pub fn bind<'cx, 'host>(self, cx: &'cx Cx<'host>) -> Result<BoundState<'cx, 'host, T>> {
        self.validate(cx)?;
        Ok(BoundState { state: self, cx })
    }
    /// Check that this descriptor names a live slot of exactly the declared kind.
    /// Validation does not read a value or subscribe reactive computations.
    pub fn validate(self, cx: &Cx<'_>) -> Result<()> {
        let status = unsafe { (cx.api.validate_state)(cx.api.context, self.key(), self.spec.kind) };
        if status == abi::OK {
            Ok(())
        } else {
            Err(Error::StateValidation {
                field: None,
                key: self.key(),
                kind: self.spec.kind,
                status,
            })
        }
    }
    /// Scope a leaf identity; the descriptor and reads/writes use the same key.
    pub const fn scoped(mut self, scope: Key) -> Self {
        self.spec.key = scope.scoped(self.spec.key).0;
        self
    }

    pub const fn spec(self) -> StateSpec {
        StateSpec(self.spec)
    }
    pub const fn key(self) -> u64 {
        self.spec.key
    }
}
/// Stateful UI composition without an independent actor identity or mailbox.
/// Component state, actions and computations are registered by the parent actor.
/// This trait is normally generated by `#[component_impl]`.
/// A component is not independently spawnable as an Actor:
/// ```compile_fail
/// use blockui::*;
/// #[component] struct Counter { count: State<i64> }
/// #[component_impl] impl Counter {
///     fn view(&self) -> View { Text::new(&self.count).into_view() }
/// }
/// fn needs_actor<T: Actor>() {}
/// needs_actor::<Counter>();
/// ```
pub trait Component: BindState {
    const STATE: &'static [StateSpec];
    const ACTIONS: &'static [Action];
    const SCOPES: &'static [Scope] = &[];
    const COMPUTEDS: &'static [ComputedSpec] = &[];
    fn migrate(cx: &Cx<'_>, previous: u64) -> Result<()>;
    fn view(cx: &Cx<'_>) -> Result<View>;
}

#[doc(hidden)]
pub const fn __parts_len<T>(parts: &[&[T]]) -> usize {
    let mut size = 0;
    let mut index = 0;
    while index < parts.len() {
        size += parts[index].len();
        index += 1;
    }
    size
}
#[doc(hidden)]
pub const fn __merge_parts<T: Copy, const N: usize>(
    parts: &[&[T]],
    local: &[T],
    placeholder: T,
) -> [T; N] {
    assert!(N == __parts_len(parts) + local.len());
    let mut result = [placeholder; N];
    let mut part = 0;
    let mut index = 0;
    while part < parts.len() {
        let mut item = 0;
        while item < parts[part].len() {
            result[index] = parts[part][item];
            index += 1;
            item += 1;
        }
        part += 1;
    }
    let mut item = 0;
    while item < local.len() {
        result[index] = local[item];
        index += 1;
        item += 1;
    }
    result
}

#[doc(hidden)]
pub const fn __scope_actions<const N: usize>(items: &[Action], scope: Key) -> [Action; N] {
    assert!(items.len() == N);
    let mut out = [Action::remote("unused"); N];
    let mut i = 0;
    while i < N {
        out[i] = items[i].__scoped(scope);
        i += 1;
    }
    out
}
#[doc(hidden)]
pub const fn __scope_computeds<const N: usize>(
    items: &[ComputedSpec],
    scope: Key,
) -> [ComputedSpec; N] {
    assert!(items.len() == N);
    let mut out = [Computed::<i64>::new("unused", |_| Ok(0)).spec(); N];
    let mut i = 0;
    while i < N {
        out[i] = items[i].__scoped(scope);
        i += 1;
    }
    out
}
#[doc(hidden)]
pub const fn __scope_views<const N: usize>(items: &[Scope], scope: Key) -> [Scope; N] {
    assert!(items.len() == N);
    let mut out = [Scope::new("unused", |_| Err(Error::Overflow)); N];
    let mut i = 0;
    while i < N {
        out[i] = items[i].__scoped(scope);
        i += 1;
    }
    out
}

/// Bind declared state identities to one synchronous host turn.
pub trait StateMode {
    type Field<T: BindState>;
    #[doc(hidden)]
    type Context;
}

pub enum DescriptorMode {}
impl StateMode for DescriptorMode {
    type Field<T: BindState> = T;
    type Context = ();
}
pub struct TurnMode<'cx, 'host: 'cx>(PhantomData<&'cx Cx<'host>>);
impl<'cx, 'host: 'cx> StateMode for TurnMode<'cx, 'host> {
    type Field<T: BindState> = T::Bound<'cx, 'host>;
    type Context = &'cx Cx<'host>;
}

pub trait BindState {
    type Bound<'cx, 'host>
    where
        'host: 'cx;
    fn bind<'cx, 'host>(&self, cx: &'cx Cx<'host>) -> Result<Self::Bound<'cx, 'host>>;
}
impl<T> BindState for State<T> {
    type Bound<'cx, 'host>
        = BoundState<'cx, 'host, T>
    where
        'host: 'cx;
    fn bind<'cx, 'host>(&self, cx: &'cx Cx<'host>) -> Result<Self::Bound<'cx, 'host>> {
        (*self).bind(cx)
    }
}

/// Turn-local access to a validated slot. It owns neither the slot nor its value.
/// Scalars read through the host so writes and reactive dependencies stay live.
/// A host violating a successful binding is an invariant fault, never a default value.
///
/// ```compile_fail
/// use blockui::{BoundState, Cx, State};
/// fn escape(cx: &Cx<'_>) -> BoundState<'static, 'static, i64> {
///     State::<i64>::new("count", 0).bind(cx).unwrap()
/// }
/// ```
/// ```compile_fail
/// use blockui::{Cx, State};
/// fn send(cx: &Cx<'_>) {
///     let count = State::<i64>::new("count", 0).bind(cx).unwrap();
///     std::thread::scope(|scope| { scope.spawn(move || count.get()); });
/// }
/// ```
pub struct BoundState<'cx, 'host, T> {
    state: State<T>,
    cx: &'cx Cx<'host>,
}
/// A borrowed, live state input for an ordinary Rust view component.
/// This is only a reference alias: it allocates nothing and keeps no context
/// beyond the current render turn. Components need no actor or lifecycle.
///
/// ```
/// use blockui::{Action, Button, Column, StateRef, Text, View, default};
/// fn counter(count: StateRef<'_, i64>, increment: Action) -> View {
///     {
///         let ui = ::blockui::Ui::new();
///         {
///             let _scope_0 = ui.scope(Column::default());
///             ui.add(Text::new(count));
///             ui.add(Button {
///                 text: "Increment".into(),
///                 on_click: increment,
///                 ..default()
///             });
///         }
///         ui.finish()
///     }
/// }
/// ```
/// A component input cannot outlive its render turn:
/// ```compile_fail
/// use blockui::StateRef;
/// fn retain(count: StateRef<'_, i64>) -> StateRef<'static, i64> { count }
/// ```
pub type StateRef<'a, T> = &'a BoundState<'a, 'a, T>;

/// Extract the durable slot descriptor; the borrowed turn context is not retained.
impl<T> From<&BoundState<'_, '_, T>> for State<T> {
    fn from(value: &BoundState<'_, '_, T>) -> Self {
        value.state
    }
}
macro_rules! bound_scalar {
    ($ty:ty) => {
        impl BoundState<'_, '_, $ty> {
            pub fn get(&self) -> $ty {
                self.state
                    .get(self.cx)
                    .expect("host invalidated a turn-bound state slot")
            }
            pub fn handle(&self) -> State<$ty> {
                self.state
            }
            pub fn set(&self, value: $ty) -> Result<()> {
                self.state.set(self.cx, value)
            }
            /// Update synchronously within this turn; errors keep the transaction fallible.
            pub fn update(&self, update: impl FnOnce($ty) -> Result<$ty>) -> Result<()> {
                self.set(update(self.get())?)
            }
        }
    };
}
bound_scalar!(i64);
bound_scalar!(f64);
bound_scalar!(bool);
impl BoundState<'_, '_, i64> {
    /// Bind this slot to text without retaining this turn's context.
    /// The formatter must be a function item or noncapturing closure.
    pub fn map_text(&self, format: fn(i64) -> String) -> Binding {
        self.state.map_text(format)
    }
}
macro_rules! bound_owned {
    ($ty:ty, $input:ty) => {
        impl BoundState<'_, '_, $ty> {
            /// Copy the current value into plugin-owned storage; no host allocation escapes.
            pub fn get(&self) -> $ty {
                self.state
                    .get(self.cx)
                    .expect("host invalidated a turn-bound state slot")
            }
            pub fn handle(&self) -> State<$ty> {
                self.state
            }
            pub fn set(&self, value: &$input) -> Result<()> {
                self.state.set(self.cx, value)
            }
            /// Modify a plugin-owned copy and publish it only on success.
            pub fn update(&self, update: impl FnOnce(&mut $ty) -> Result<()>) -> Result<()> {
                let mut value = self.get();
                update(&mut value)?;
                self.set(&value)
            }
        }
    };
}
bound_owned!(String, str);
bound_owned!(Vec<i64>, [i64]);
bound_owned!(Vec<String>, [String]);

impl State<i64> {
    /// A reactive text projection. No current-turn context is retained.
    /// Use `map_text_with` to capture owned format parameters:
    /// ```
    /// use blockui::{State, Text};
    /// let prefix = String::from("Count: ");
    /// let count = State::<i64>::new("count", 0);
    /// let _label = Text::new(count.map_text_with(move |value| format!("{prefix}{value}")));
    /// ```
    /// Captured mappers cannot borrow a rendering context:
    /// ```compile_fail
    /// use blockui::{Cx, State};
    /// fn label(cx: &Cx<'_>) {
    ///     let count = State::<i64>::new("count", 0);
    ///     let _binding = count.map_text_with(move |value| {
    ///         format!("{}: {value}", count.get(cx).unwrap())
    ///     });
    /// }
    /// ```
    /// The lightweight `map_text` formatter rejects captures:
    /// ```compile_fail
    /// use blockui::State;
    /// let prefix = String::from("Count: ");
    /// let count = State::<i64>::new("count", 0);
    /// count.map_text(move |value| format!("{prefix}{value}"));
    /// ```
    pub fn map_text(self, format: fn(i64) -> String) -> Binding {
        Binding::MappedInteger {
            slot: self.key(),
            format,
        }
    }
    /// Read the old schema during migrate; outside migration this returns an error.
    pub fn previous(self, cx: &Cx<'_>) -> Result<i64> {
        let mut value = 0;
        let code = unsafe { (cx.api.previous_i64)(cx.api.context, self.key(), &mut value) };
        if code == abi::OK {
            Ok(value)
        } else {
            Err(code.into())
        }
    }
    pub const fn new(name: &'static str, default: i64) -> Self {
        Self {
            spec: SlotSpec::integer(name, default),
            marker: PhantomData,
        }
    }
    pub fn get(self, cx: &Cx<'_>) -> Result<i64> {
        cx.api
            .state_key::<i64>(self.key())
            .get()
            .map_err(Into::into)
    }
    pub fn set(self, cx: &Cx<'_>, value: i64) -> Result<()> {
        cx.api
            .state_key::<i64>(self.key())
            .set(value)
            .map_err(Into::into)
    }
    pub fn update(self, cx: &Cx<'_>, f: impl FnOnce(i64) -> Result<i64>) -> Result<()> {
        self.set(cx, f(self.get(cx)?)?)
    }
}
impl State<f64> {
    pub const fn new(name: &'static str, default: f64) -> Self {
        Self {
            spec: SlotSpec::float(name, default),
            marker: PhantomData,
        }
    }
    pub fn get(self, cx: &Cx<'_>) -> Result<f64> {
        let mut value = 0.0;
        let status = unsafe { (cx.api.read_f64)(cx.api.context, self.key(), &mut value) };
        if status == abi::OK {
            Ok(value)
        } else {
            Err(status.into())
        }
    }
    pub fn previous(self, cx: &Cx<'_>) -> Result<f64> {
        let mut value = 0.0;
        let status = unsafe { (cx.api.previous_f64)(cx.api.context, self.key(), &mut value) };
        if status == abi::OK {
            Ok(value)
        } else {
            Err(status.into())
        }
    }
    pub fn set(self, cx: &Cx<'_>, value: f64) -> Result<()> {
        let status = unsafe { (cx.api.write_f64)(cx.api.context, self.key(), value) };
        if status == abi::OK {
            Ok(())
        } else {
            Err(status.into())
        }
    }
    pub fn update(self, cx: &Cx<'_>, update: impl FnOnce(f64) -> Result<f64>) -> Result<()> {
        self.set(cx, update(self.get(cx)?)?)
    }
}
impl State<bool> {
    pub fn previous(self, cx: &Cx<'_>) -> Result<bool> {
        let mut value = 0;
        let code = unsafe { (cx.api.previous_i64)(cx.api.context, self.key(), &mut value) };
        if code != abi::OK {
            return Err(code.into());
        }
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::Host(abi::ERROR)),
        }
    }
    pub const fn new(name: &'static str, default: bool) -> Self {
        Self {
            spec: SlotSpec::boolean(name, default),
            marker: PhantomData,
        }
    }
    pub fn get(self, cx: &Cx<'_>) -> Result<bool> {
        match cx
            .api
            .state_key::<i64>(self.key())
            .get()
            .map_err(Error::from)?
        {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::Host(abi::ERROR)),
        }
    }
    pub fn set(self, cx: &Cx<'_>, value: bool) -> Result<()> {
        cx.api
            .state_key::<i64>(self.key())
            .set(i64::from(value))
            .map_err(Into::into)
    }
    pub fn update(self, cx: &Cx<'_>, f: impl FnOnce(bool) -> Result<bool>) -> Result<()> {
        self.set(cx, f(self.get(cx)?)?)
    }
}
impl State<String> {
    /// Read the old schema during migrate, including removed or type-changed slots.
    pub fn previous(self, cx: &Cx<'_>) -> Result<String> {
        let length =
            unsafe { (cx.api.previous_text)(cx.api.context, self.key(), std::ptr::null_mut(), 0) };
        if length < 0 {
            return Err(Error::Host(length as i32));
        }
        let mut bytes = vec![0; length as usize];
        let copied = unsafe {
            (cx.api.previous_text)(cx.api.context, self.key(), bytes.as_mut_ptr(), bytes.len())
        };
        if copied != length {
            return Err(Error::Host(copied as i32));
        }
        String::from_utf8(bytes).map_err(|_| Error::Host(abi::ERROR))
    }
    pub const fn new(name: &'static str) -> Self {
        Self::with_default(name, "")
    }
    pub const fn with_default(name: &'static str, default: &'static str) -> Self {
        Self {
            spec: SlotSpec::text_default(name, default),
            marker: PhantomData,
        }
    }
    pub fn get(self, cx: &Cx<'_>) -> Result<String> {
        cx.api
            .state_key::<String>(self.key())
            .get()
            .map_err(Into::into)
    }
    pub fn set(self, cx: &Cx<'_>, value: &str) -> Result<()> {
        cx.api
            .state_key::<String>(self.key())
            .set(value)
            .map_err(Into::into)
    }
    /// Modify a temporary owned copy, publishing only when the callback succeeds.
    /// The callback runs synchronously and is never retained across replacement.
    pub fn update(self, cx: &Cx<'_>, update: impl FnOnce(&mut String) -> Result<()>) -> Result<()> {
        let mut value = self.get(cx)?;
        update(&mut value)?;
        self.set(cx, &value)
    }
}

fn integer_collection_len(
    context: *mut std::ffi::c_void,
    key: u64,
    read: unsafe extern "C" fn(*mut std::ffi::c_void, u64, *mut i64, usize) -> isize,
) -> Result<usize> {
    let len = unsafe { read(context, key, std::ptr::null_mut(), 0) };
    if len < 0 || len as usize > abi::MAX_LIST_ITEMS {
        Err(Error::Host(abi::ERROR))
    } else {
        Ok(len as usize)
    }
}

fn read_integer_collection_range(
    context: *mut std::ffi::c_void,
    key: u64,
    read: unsafe extern "C" fn(*mut std::ffi::c_void, u64, usize, usize, *mut i64, usize) -> isize,
    range: std::ops::Range<usize>,
) -> Result<Vec<i64>> {
    if range.start > range.end {
        return Err(Error::Host(abi::ERROR));
    }
    let count = unsafe {
        read(
            context,
            key,
            range.start,
            range.end,
            std::ptr::null_mut(),
            0,
        )
    };
    if count < 0 || count as usize > abi::MAX_LIST_ITEMS || count as usize > range.end - range.start
    {
        return Err(Error::Host(abi::ERROR));
    }
    let mut items = vec![0; count as usize];
    if count != 0
        && unsafe {
            read(
                context,
                key,
                range.start,
                range.end,
                items.as_mut_ptr(),
                items.len(),
            )
        } != count
    {
        return Err(Error::Host(abi::ERROR));
    }
    Ok(items)
}

impl State<Vec<i64>> {
    pub const fn new(name: &'static str) -> Self {
        Self::with_default(name, &[])
    }
    pub const fn with_default(name: &'static str, default: &'static [i64]) -> Self {
        Self {
            spec: SlotSpec::integer_list(name, default),
            marker: PhantomData,
        }
    }
    fn read(self, cx: &Cx<'_>, previous: bool) -> Result<Vec<i64>> {
        let read = if previous {
            cx.api.previous_list
        } else {
            cx.api.read_list
        };
        let len = unsafe { read(cx.api.context, self.key(), std::ptr::null_mut(), 0) };
        if len < 0 || len as usize > abi::MAX_LIST_ITEMS {
            return Err(Error::Host(abi::ERROR));
        }
        let mut values = vec![0; len as usize];
        let copied = unsafe {
            read(
                cx.api.context,
                self.key(),
                values.as_mut_ptr(),
                values.len(),
            )
        };
        if copied != len {
            return Err(Error::Host(abi::ERROR));
        }
        Ok(values)
    }
    /// Copy only the clipped range, tracking the whole collection dependency.
    /// Reversed ranges return an error.
    pub fn get_range(self, cx: &Cx<'_>, range: std::ops::Range<usize>) -> Result<Vec<i64>> {
        read_integer_collection_range(cx.api.context, self.key(), cx.api.read_list_range, range)
    }
    pub fn get(self, cx: &Cx<'_>) -> Result<Vec<i64>> {
        self.read(cx, false)
    }
    /// Read the collection length without copying its items.
    pub fn len(self, cx: &Cx<'_>) -> Result<usize> {
        integer_collection_len(cx.api.context, self.key(), cx.api.read_list)
    }
    /// Check emptiness without copying its items.
    pub fn is_empty(self, cx: &Cx<'_>) -> Result<bool> {
        Ok(self.len(cx)? == 0)
    }
    pub fn previous(self, cx: &Cx<'_>) -> Result<Vec<i64>> {
        self.read(cx, true)
    }
    pub fn set(self, cx: &Cx<'_>, values: &[i64]) -> Result<()> {
        if values.len() > abi::MAX_LIST_ITEMS {
            return Err(Error::Host(abi::ERROR));
        }
        let code = unsafe {
            (cx.api.write_list)(cx.api.context, self.key(), values.as_ptr(), values.len())
        };
        if code == abi::OK {
            Ok(())
        } else {
            Err(code.into())
        }
    }
    pub fn update(
        self,
        cx: &Cx<'_>,
        update: impl FnOnce(&mut Vec<i64>) -> Result<()>,
    ) -> Result<()> {
        let mut values = self.get(cx)?;
        update(&mut values)?;
        self.set(cx, &values)
    }
}

/// Runtime-owned text collection. Defaults to empty; reads return a local copy.
/// At most 4096 items and 1 MiB of aggregate UTF-8 bytes are accepted.
/// Actor declarations can supply a static array with `#[state(default = &["Draft"])]`.
///
/// ```
/// use blockui::{Cx, Result, State};
/// const LABELS: State<Vec<String>> = State::<Vec<String>>::new("labels");
/// fn append(cx: &Cx<'_>) -> Result<()> {
///     LABELS.bind(cx)?.update(|items| { items.push("标题😀".into()); Ok(()) })
/// }
/// ```
///
/// ```compile_fail
/// use blockui::{Cx, State};
/// fn wrong(cx: &Cx<'_>) {
///     let _ = State::<Vec<String>>::new("labels").set(cx, &[1_i64]);
/// }
/// ```
///
/// ```
/// # #[cfg(feature = "macros")]
/// # mod example {
/// use blockui::{actor, State};
/// const DEFAULTS: [&str; 2] = ["Draft", "标题😀"];
/// #[actor(name = "labels")]
/// struct Labels {
///     #[state(default = &DEFAULTS)]
///     items: State<Vec<String>>,
/// }
/// # }
/// ```
///
/// ```compile_fail
/// use blockui::{actor, State};
/// #[actor]
/// struct Wrong { #[state(default = &[1, 2])] items: State<Vec<String>> }
/// ```
///
/// ```compile_fail
/// use blockui::{actor, State};
/// #[actor]
/// struct TooMany {
///     #[state(default = &[""; 4097])]
///     items: State<Vec<String>>,
/// }
/// ```
impl State<Vec<String>> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            spec: SlotSpec::text_list(name),
            marker: PhantomData,
        }
    }
    #[doc(hidden)]
    pub const fn __with_defaults(name: &'static str, defaults: &'static [abi::Bytes]) -> Self {
        Self {
            spec: SlotSpec::text_list_default(name, defaults),
            marker: PhantomData,
        }
    }
    fn read(self, cx: &Cx<'_>, previous: bool) -> Result<Vec<String>> {
        let read = if previous {
            cx.api.previous_text_list
        } else {
            cx.api.read_text_list
        };
        read_text_collection(cx.api.context, self.key(), read)
    }
    pub fn get(self, cx: &Cx<'_>) -> Result<Vec<String>> {
        self.read(cx, false)
    }
    /// Read the collection length without copying its strings.
    pub fn len(self, cx: &Cx<'_>) -> Result<usize> {
        text_collection_len(cx.api.context, self.key(), cx.api.read_text_list)
    }
    /// Check emptiness without copying strings.
    pub fn is_empty(self, cx: &Cx<'_>) -> Result<bool> {
        Ok(self.len(cx)? == 0)
    }
    /// Copy only the requested range. Bounds are clipped to the collection length.
    /// Reversed ranges are rejected; reads track the collection dependency.
    pub fn get_range(self, cx: &Cx<'_>, range: std::ops::Range<usize>) -> Result<Vec<String>> {
        read_text_collection_range(
            cx.api.context,
            self.key(),
            cx.api.read_text_list,
            Some(range),
        )
    }
    pub fn previous(self, cx: &Cx<'_>) -> Result<Vec<String>> {
        self.read(cx, true)
    }
    /// Internal schema admission probe. Reads only previous collection metadata;
    /// no previous strings are copied. Outside migration the host rejects it.
    #[doc(hidden)]
    pub fn __previous_len(self, cx: &Cx<'_>) -> Result<usize> {
        text_collection_len(cx.api.context, self.key(), cx.api.previous_text_list)
    }
    pub fn set(self, cx: &Cx<'_>, values: &[String]) -> Result<()> {
        let total = values
            .iter()
            .try_fold(0_usize, |total, text| total.checked_add(text.len()));
        if values.len() > abi::MAX_LIST_ITEMS
            || total.is_none_or(|total| total > abi::MAX_TEXT_LIST_BYTES)
        {
            return Err(Error::Host(abi::ERROR));
        }
        let descriptors: Vec<_> = values.iter().map(|text| abi::Bytes::new(text)).collect();
        let code = unsafe {
            (cx.api.write_text_list)(
                cx.api.context,
                self.key(),
                descriptors.as_ptr(),
                descriptors.len(),
            )
        };
        if code == abi::OK {
            Ok(())
        } else {
            Err(code.into())
        }
    }
    pub fn update(
        self,
        cx: &Cx<'_>,
        update: impl FnOnce(&mut Vec<String>) -> Result<()>,
    ) -> Result<()> {
        let mut values = self.get(cx)?;
        update(&mut values)?;
        self.set(cx, &values)
    }
}

/// Translate ordinary static string slices into C-layout schema descriptors.
#[doc(hidden)]
pub const fn __text_list_defaults<const N: usize>(values: &[&'static str]) -> [abi::Bytes; N] {
    assert!(
        N <= abi::MAX_LIST_ITEMS,
        "text list default exceeds item limit"
    );
    assert!(values.len() == N, "text list default length mismatch");
    let mut descriptors = [abi::Bytes {
        ptr: std::ptr::null(),
        len: 0,
    }; N];
    let mut index = 0;
    let mut total = 0_usize;
    while index < N {
        total += values[index].len();
        assert!(
            total <= abi::MAX_TEXT_LIST_BYTES,
            "text list default exceeds byte limit"
        );
        descriptors[index] = abi::Bytes::new(values[index]);
        index += 1;
    }
    descriptors
}

/// A typed identity for a host-cached derived computation.
pub struct Computed<T> {
    key: u64,
    instance: InstancePath,
    compute: Compute,
    marker: PhantomData<fn() -> T>,
}
impl<T> Copy for Computed<T> {}
impl<T> Clone for Computed<T> {
    fn clone(&self) -> Self {
        *self
    }
}
#[derive(Clone, Copy)]
enum Compute {
    Float(for<'a> fn(&Cx<'a>) -> Result<f64>),
    Integer(for<'a> fn(&Cx<'a>) -> Result<i64>),
    Boolean(for<'a> fn(&Cx<'a>) -> Result<bool>),
    Text(for<'a> fn(&Cx<'a>) -> Result<String>),
    List(for<'a> fn(&Cx<'a>) -> Result<Vec<i64>>),
    TextList(for<'a> fn(&Cx<'a>) -> Result<Vec<String>>),
}
/// A private descriptor and callback pair; only data crosses the ABI.
#[derive(Clone, Copy)]
pub struct ComputedSpec {
    key: u64,
    instance: InstancePath,
    compute: Compute,
}
impl ComputedSpec {
    #[doc(hidden)]
    pub const fn __scoped(mut self, scope: Key) -> Self {
        self.key = scope.scoped(self.key).0;
        self.instance = self.instance.scoped(scope);
        self
    }
}
impl<T> Computed<T> {
    /// Resolved host identity, including any component instance namespace.
    pub const fn key(self) -> u64 {
        self.key
    }

    #[doc(hidden)]
    pub const fn __instance(mut self, instance: InstancePath) -> Self {
        self.key = instance.resolve(self.key);
        self.instance = instance;
        self
    }

    pub const fn spec(self) -> ComputedSpec {
        ComputedSpec {
            key: self.key,
            instance: self.instance,
            compute: self.compute,
        }
    }
}
impl Computed<i64> {
    /// Format the memo's value as an independently tracked text binding.
    /// The formatter cannot capture parameters. Evaluation uses the current
    /// context, so constructing the binding does not read the memo in the root.
    pub fn map_text(self, format: fn(i64) -> String) -> Binding {
        Binding::NamedCaptured {
            identity: Key::named("identity.binding.mapped.computed").scoped(self.key),
            compute: std::rc::Rc::new(move |cx| Ok(format(self.get(cx)?))),
        }
    }

    pub const fn new(name: &str, compute: for<'a> fn(&Cx<'a>) -> Result<i64>) -> Self {
        Self {
            key: abi::key(name),
            instance: InstancePath::EMPTY,
            compute: Compute::Integer(compute),
            marker: PhantomData,
        }
    }
    pub fn get(self, cx: &Cx<'_>) -> Result<i64> {
        let mut value = 0;
        let status = unsafe { (cx.api.computed_i64)(cx.api.context, self.key, &mut value) };
        if status == abi::OK {
            Ok(value)
        } else {
            Err(status.into())
        }
    }
}
impl Computed<f64> {
    /// Format the memo's value as an independently tracked text binding.
    /// The formatter cannot capture parameters. Evaluation uses the current
    /// context, so constructing the binding does not read the memo in the root.
    pub fn map_text(self, format: fn(f64) -> String) -> Binding {
        Binding::NamedCaptured {
            identity: Key::named("identity.binding.mapped.computed").scoped(self.key),
            compute: std::rc::Rc::new(move |cx| Ok(format(self.get(cx)?))),
        }
    }

    pub const fn new(name: &str, compute: for<'a> fn(&Cx<'a>) -> Result<f64>) -> Self {
        Self {
            key: abi::key(name),
            instance: InstancePath::EMPTY,
            compute: Compute::Float(compute),
            marker: PhantomData,
        }
    }
    pub fn get(self, cx: &Cx<'_>) -> Result<f64> {
        let mut value = 0.0;
        let status = unsafe { (cx.api.computed_f64)(cx.api.context, self.key, &mut value) };
        if status == abi::OK {
            Ok(value)
        } else {
            Err(status.into())
        }
    }
}
impl Computed<bool> {
    /// Format the memo's value as an independently tracked text binding.
    /// The formatter cannot capture parameters. Evaluation uses the current
    /// context, so constructing the binding does not read the memo in the root.
    pub fn map_text(self, format: fn(bool) -> String) -> Binding {
        Binding::NamedCaptured {
            identity: Key::named("identity.binding.mapped.computed").scoped(self.key),
            compute: std::rc::Rc::new(move |cx| Ok(format(self.get(cx)?))),
        }
    }

    pub const fn new(name: &str, compute: for<'a> fn(&Cx<'a>) -> Result<bool>) -> Self {
        Self {
            key: abi::key(name),
            instance: InstancePath::EMPTY,
            compute: Compute::Boolean(compute),
            marker: PhantomData,
        }
    }
    pub fn get(self, cx: &Cx<'_>) -> Result<bool> {
        let mut value = 0;
        let status = unsafe { (cx.api.computed_i64)(cx.api.context, self.key, &mut value) };
        if status != abi::OK {
            return Err(status.into());
        }
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::Host(abi::ERROR)),
        }
    }
}
impl Computed<String> {
    /// Format the memo's value as an independently tracked text binding.
    /// The formatter cannot capture parameters. Evaluation uses the current
    /// context, so constructing the binding does not read the memo in the root.
    pub fn map_text(self, format: fn(String) -> String) -> Binding {
        Binding::NamedCaptured {
            identity: Key::named("identity.binding.mapped.computed").scoped(self.key),
            compute: std::rc::Rc::new(move |cx| Ok(format(self.get(cx)?))),
        }
    }

    pub const fn new(name: &str, compute: for<'a> fn(&Cx<'a>) -> Result<String>) -> Self {
        Self {
            key: abi::key(name),
            instance: InstancePath::EMPTY,
            compute: Compute::Text(compute),
            marker: PhantomData,
        }
    }
    pub fn get(self, cx: &Cx<'_>) -> Result<String> {
        let length =
            unsafe { (cx.api.computed_text)(cx.api.context, self.key, std::ptr::null_mut(), 0) };
        if length < 0 {
            return Err(abi::ERROR.into());
        }
        let mut bytes = vec![0; length as usize];
        if unsafe {
            (cx.api.computed_text)(cx.api.context, self.key, bytes.as_mut_ptr(), bytes.len())
        } != length
        {
            return Err(abi::ERROR.into());
        }
        String::from_utf8(bytes).map_err(|_| abi::ERROR.into())
    }
}

/// A cached host-owned integer list. Reads return a plugin-local copy; no Vec
/// allocation or destructor escapes the currently loaded implementation.
/// ```
/// use blockui::{Computed, State};
/// const EVEN: Computed<Vec<i64>> = Computed::<Vec<i64>>::new("even", |cx| {
///     Ok(State::<Vec<i64>>::new("items").get(cx)?.into_iter()
///         .filter(|item| item % 2 == 0).collect())
/// });
/// ```
impl Computed<Vec<i64>> {
    /// Format the memo's value as an independently tracked text binding.
    /// The formatter cannot capture parameters. Evaluation uses the current
    /// context, so constructing the binding does not read the memo in the root.
    pub fn map_text(self, format: fn(Vec<i64>) -> String) -> Binding {
        Binding::NamedCaptured {
            identity: Key::named("identity.binding.mapped.computed").scoped(self.key),
            compute: std::rc::Rc::new(move |cx| Ok(format(self.get(cx)?))),
        }
    }

    pub const fn new(name: &str, compute: for<'a> fn(&Cx<'a>) -> Result<Vec<i64>>) -> Self {
        Self {
            key: abi::key(name),
            instance: InstancePath::EMPTY,
            compute: Compute::List(compute),
            marker: PhantomData,
        }
    }
    /// Read the memo length without copying its items; tracks its dependency.
    pub fn len(self, cx: &Cx<'_>) -> Result<usize> {
        integer_collection_len(cx.api.context, self.key, cx.api.computed_list)
    }
    pub fn is_empty(self, cx: &Cx<'_>) -> Result<bool> {
        Ok(self.len(cx)? == 0)
    }
    /// Copy only the clipped range, tracking the whole collection dependency.
    /// Reversed ranges return an error.
    pub fn get_range(self, cx: &Cx<'_>, range: std::ops::Range<usize>) -> Result<Vec<i64>> {
        read_integer_collection_range(cx.api.context, self.key, cx.api.computed_list_range, range)
    }
    pub fn get(self, cx: &Cx<'_>) -> Result<Vec<i64>> {
        let length =
            unsafe { (cx.api.computed_list)(cx.api.context, self.key, std::ptr::null_mut(), 0) };
        if length < 0 || length as usize > abi::MAX_LIST_ITEMS {
            return Err(abi::ERROR.into());
        }
        let mut items = vec![0; length as usize];
        if unsafe {
            (cx.api.computed_list)(cx.api.context, self.key, items.as_mut_ptr(), items.len())
        } != length
        {
            return Err(abi::ERROR.into());
        }
        Ok(items)
    }
}

/// A cached host-owned UTF-8 collection, copied locally on reads.
/// ```
/// use blockui::{Computed, State};
/// const NONEMPTY: Computed<Vec<String>> = Computed::<Vec<String>>::new("nonempty", |cx| {
///     Ok(State::<Vec<String>>::new("labels").get(cx)?.into_iter()
///         .filter(|label| !label.is_empty()).collect())
/// });
/// ```
/// ```compile_fail
/// use blockui::Computed;
/// const WRONG: Computed<Vec<String>> = Computed::<Vec<String>>::new("wrong", |_| Ok(vec![1_i64]));
/// ```
impl Computed<Vec<String>> {
    /// Format the memo's value as an independently tracked text binding.
    /// The formatter cannot capture parameters. Evaluation uses the current
    /// context, so constructing the binding does not read the memo in the root.
    pub fn map_text(self, format: fn(Vec<String>) -> String) -> Binding {
        Binding::NamedCaptured {
            identity: Key::named("identity.binding.mapped.computed").scoped(self.key),
            compute: std::rc::Rc::new(move |cx| Ok(format(self.get(cx)?))),
        }
    }

    pub const fn new(name: &str, compute: for<'a> fn(&Cx<'a>) -> Result<Vec<String>>) -> Self {
        Self {
            key: abi::key(name),
            instance: InstancePath::EMPTY,
            compute: Compute::TextList(compute),
            marker: PhantomData,
        }
    }
    pub fn get(self, cx: &Cx<'_>) -> Result<Vec<String>> {
        read_text_collection(cx.api.context, self.key, cx.api.computed_text_list)
    }
    /// Read the memo length without copying its strings.
    pub fn len(self, cx: &Cx<'_>) -> Result<usize> {
        text_collection_len(cx.api.context, self.key, cx.api.computed_text_list)
    }
    /// Check emptiness without copying strings.
    pub fn is_empty(self, cx: &Cx<'_>) -> Result<bool> {
        Ok(self.len(cx)? == 0)
    }
    /// Copy a clipped range from the host memo, tracking its dependency.
    pub fn get_range(self, cx: &Cx<'_>, range: std::ops::Range<usize>) -> Result<Vec<String>> {
        read_text_collection_range(
            cx.api.context,
            self.key,
            cx.api.computed_text_list,
            Some(range),
        )
    }
}
fn read_text_collection(
    context: *mut std::ffi::c_void,
    key: u64,
    read: unsafe extern "C" fn(*mut std::ffi::c_void, u64, usize, *mut u8, usize) -> isize,
) -> Result<Vec<String>> {
    read_text_collection_range(context, key, read, None)
}
fn text_collection_len(
    context: *mut std::ffi::c_void,
    key: u64,
    read: unsafe extern "C" fn(*mut std::ffi::c_void, u64, usize, *mut u8, usize) -> isize,
) -> Result<usize> {
    let count = unsafe { read(context, key, usize::MAX, std::ptr::null_mut(), 0) };
    if count < 0 || count as usize > abi::MAX_LIST_ITEMS {
        return Err(Error::Host(abi::ERROR));
    }
    Ok(count as usize)
}
fn read_text_collection_range(
    context: *mut std::ffi::c_void,
    key: u64,
    read: unsafe extern "C" fn(*mut std::ffi::c_void, u64, usize, *mut u8, usize) -> isize,
    range: Option<std::ops::Range<usize>>,
) -> Result<Vec<String>> {
    let count = text_collection_len(context, key, read)?;
    let range = range.unwrap_or(0..count);
    if range.start > range.end {
        return Err(Error::Host(abi::ERROR));
    }
    let range = range.start.min(count)..range.end.min(count);
    let mut values = Vec::with_capacity(range.len());
    let mut total = 0_usize;
    for index in range {
        let length = unsafe { read(context, key, index, std::ptr::null_mut(), 0) };
        if length < 0 {
            return Err(Error::Host(abi::ERROR));
        }
        total = total.checked_add(length as usize).ok_or(Error::Overflow)?;
        if total > abi::MAX_TEXT_LIST_BYTES {
            return Err(Error::Host(abi::ERROR));
        }
        let mut bytes = vec![0; length as usize];
        let copied = unsafe { read(context, key, index, bytes.as_mut_ptr(), bytes.len()) };
        if copied != length {
            return Err(Error::Host(abi::ERROR));
        }
        values.push(String::from_utf8(bytes).map_err(|_| Error::Host(abi::ERROR))?);
    }
    Ok(values)
}

/// Semantic identity. Explicit collection keys can be scoped without source positions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Key(pub u64);
impl Key {
    pub const fn named(name: &str) -> Self {
        Self(abi::key(name))
    }
    /// Scope a string business identity under this namespace.
    /// Stable through content edits/reordering; changing the ID changes identity.
    pub const fn scoped_named(self, item: &str) -> Self {
        self.scoped(abi::key(item))
    }
    pub const fn scoped(self, item: u64) -> Self {
        let bytes = item.to_le_bytes();
        let mut hash = self.0;
        let mut index = 0;
        while index < bytes.len() {
            hash = (hash ^ bytes[index] as u64).wrapping_mul(0x100000001b3);
            index += 1;
        }
        Self(hash)
    }
}
impl From<&str> for Key {
    fn from(name: &str) -> Self {
        Self::named(name)
    }
}
impl From<&String> for Key {
    fn from(name: &String) -> Self {
        Self::named(name)
    }
}
impl From<String> for Key {
    fn from(name: String) -> Self {
        Self::named(&name)
    }
}
impl From<usize> for Key {
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}
impl From<i64> for Key {
    fn from(value: i64) -> Self {
        Self(value as u64)
    }
}
impl From<u64> for Key {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

pub type ActionFn = for<'a> fn(&Cx<'a>, i64) -> Result<()>;
/// Marker for a read-only regular file owned and closed by the host.
pub struct File;
/// A typed identity token, never a native handle or a plugin-owned resource value.
pub struct Resource<T> {
    id: u64,
    policy: ReloadPolicy,
    marker: PhantomData<fn() -> T>,
}
impl<T> Copy for Resource<T> {}
impl<T> Clone for Resource<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Resource<T> {
    pub const fn new(name: &str) -> Self {
        Self {
            id: abi::key(name),
            policy: ReloadPolicy::Preserve,
            marker: PhantomData,
        }
    }
    pub const fn key(self) -> u64 {
        self.id
    }
    /// Namespace the identity without moving or cloning the host resource.
    pub const fn scoped(mut self, scope: Key) -> Self {
        self.id = scope.scoped(self.id).0;
        self
    }
    pub const fn on_reload(mut self, policy: ReloadPolicy) -> Self {
        self.policy = policy;
        self
    }
}
impl BindState for Resource<File> {
    type Bound<'cx, 'host>
        = BoundResource<'cx, 'host, File>
    where
        'host: 'cx;
    fn bind<'cx, 'host>(&self, cx: &'cx Cx<'host>) -> Result<BoundResource<'cx, 'host, File>> {
        Ok(BoundResource::new(*self, cx))
    }
}
impl Resource<File> {
    /// Open during an action. A failed turn closes the candidate and restores old resources.
    pub fn open(self, cx: &Cx<'_>, path: &str) -> Result<()> {
        self.open_access(cx, path, 0)
    }
    /// Open an existing regular file for reading and writing, without creating or truncating it.
    pub fn open_writable(self, cx: &Cx<'_>, path: &str) -> Result<()> {
        self.open_access(cx, path, 1)
    }
    fn open_access(self, cx: &Cx<'_>, path: &str, access: u32) -> Result<()> {
        let code = unsafe {
            (cx.api.open_file)(
                cx.api.context,
                self.id,
                abi::Bytes::new(path),
                self.policy as u32,
                access,
            )
        };
        if code == abi::OK {
            Ok(())
        } else {
            Err(code.into())
        }
    }
    pub fn close(self, cx: &Cx<'_>) -> Result<()> {
        let code = unsafe { (cx.api.close_file)(cx.api.context, self.id) };
        if code == abi::OK {
            Ok(())
        } else {
            Err(code.into())
        }
    }
    /// Positional read, at most 1 MiB. External bytes are not automatically reactive;
    /// copy desired data into State within the action. Reads do not change a file cursor.
    pub fn read(self, cx: &Cx<'_>, offset: u64, limit: usize) -> Result<Vec<u8>> {
        if limit > 1024 * 1024 {
            return Err(Error::Overflow);
        }
        let mut bytes = vec![0; limit];
        let length = unsafe {
            (cx.api.read_file)(cx.api.context, self.id, offset, bytes.as_mut_ptr(), limit)
        };
        if length < 0 {
            return Err(Error::Host(length as i32));
        }
        bytes.truncate(length as usize);
        Ok(bytes)
    }
}

/// Stable identity for host-owned work. Completion dispatches an action ID through
/// the implementation installed when the message is handled, never a retained callback.
#[derive(Clone, Copy)]
pub struct Task {
    id: u64,
    policy: ReloadPolicy,
}
impl BindState for Task {
    type Bound<'cx, 'host>
        = BoundTask<'cx, 'host>
    where
        'host: 'cx;
    fn bind<'cx, 'host>(&self, cx: &'cx Cx<'host>) -> Result<BoundTask<'cx, 'host>> {
        Ok(BoundTask::new(*self, cx))
    }
}
/// Policy for host-owned task recipes and resources after successful replacement.
/// Already admitted mailbox messages remain. Failed replacement applies no policy.
/// Preserve retains existing work and delivers its data to the current action.
/// Cancel retires pending work/results; it cannot undo external effects already run.
/// Restart replays supported host recipes (timers/reads), never plugin futures.
/// File write/save/create tasks reject Restart to avoid replaying external effects.
///
/// ```
/// use blockui::{Action, Cx, ReloadPolicy, Result, State, Task};
/// const TICKS: State<i64> = State::<i64>::new("ticks", 0);
/// const DONE: Action = Action::new("tick", |cx, _| TICKS.update(cx, |v| Ok(v + 1)));
/// const CLOCK: Task = Task::new("clock").on_reload(ReloadPolicy::Preserve);
/// fn start(cx: &Cx<'_>) -> Result<()> {
///     CLOCK.interval(cx, std::time::Duration::from_millis(100), DONE, 0)
/// }
/// fn stop(cx: &Cx<'_>) -> Result<()> { CLOCK.cancel(cx) }
/// ```
#[derive(Clone, Copy, Default)]
#[repr(u32)]
pub enum ReloadPolicy {
    #[default]
    Preserve = 0,
    Cancel = 1,
    Restart = 2,
}
fn timer_millis(duration: std::time::Duration) -> Result<u64> {
    if duration > std::time::Duration::from_secs(86_400) {
        return Err(Error::Host(abi::ERROR));
    }
    // The host wire format has millisecond precision. Never shorten a requested
    // positive duration into an immediate timer by truncating fractional units.
    let millis = u64::try_from(duration.as_millis()).map_err(|_| Error::Overflow)?;
    millis
        .checked_add(u64::from(
            !duration.subsec_nanos().is_multiple_of(1_000_000),
        ))
        .ok_or(Error::Overflow)
}

fn read_task_result(
    context: *mut std::ffi::c_void,
    id: u64,
    read: unsafe extern "C" fn(*mut std::ffi::c_void, u64, *mut u8, usize, *mut u32) -> isize,
) -> Result<Option<Vec<u8>>> {
    let mut status = 0;
    let length = unsafe { read(context, id, std::ptr::null_mut(), 0, &mut status) };
    if length < 0 {
        return Err(Error::Host(i32::try_from(length).unwrap_or(abi::ERROR)));
    }
    if status == 0 && length == 0 {
        return Ok(None);
    }
    if !matches!(status, 1 | 2) || length as usize > 1024 * 1024 {
        return Err(Error::Host(abi::ERROR));
    }
    let expected_status = status;
    let mut bytes = vec![0; length as usize];
    let copied = unsafe { read(context, id, bytes.as_mut_ptr(), bytes.len(), &mut status) };
    if copied != length || status != expected_status {
        return Err(Error::Host(abi::ERROR));
    }
    match status {
        1 => Ok(Some(bytes)),
        2 => Err(Error::Io(
            String::from_utf8(bytes).map_err(|_| Error::Host(abi::ERROR))?,
        )),
        _ => unreachable!("validated task result status"),
    }
}

impl Task {
    pub const fn key(self) -> u64 {
        self.id
    }
    pub const fn new(name: &str) -> Self {
        Self {
            id: abi::key(name),
            policy: ReloadPolicy::Preserve,
        }
    }
    /// Namespace a host task identity, preserving its reload policy.
    pub const fn scoped(mut self, scope: Key) -> Self {
        self.id = scope.scoped(self.id).0;
        self
    }
    pub const fn on_reload(mut self, policy: ReloadPolicy) -> Self {
        self.policy = policy;
        self
    }
    /// Read a host-owned file on the host worker pool after this action commits.
    /// The completion action receives this task ID as its integer payload.
    pub fn read_file(
        self,
        cx: &Cx<'_>,
        file: Resource<File>,
        offset: u64,
        limit: usize,
        completion: Action,
    ) -> Result<()> {
        let code = unsafe {
            (cx.api.read_file_async)(
                cx.api.context,
                self.id,
                file.id,
                offset,
                limit,
                completion.id,
                self.policy as u32,
            )
        };
        if code == abi::OK {
            Ok(())
        } else {
            Err(code.into())
        }
    }
    /// Positional write after successful turn commit. Does not truncate or sync.
    /// Success is an empty result. Cancel cannot undo an in-flight write; Restart
    /// is rejected to avoid replaying external effects on every code reload.
    pub fn write_file(
        self,
        cx: &Cx<'_>,
        file: Resource<File>,
        offset: u64,
        bytes: &[u8],
        completion: Action,
    ) -> Result<()> {
        let code = unsafe {
            (cx.api.write_file_async)(
                cx.api.context,
                self.id,
                file.id,
                offset,
                abi::Bytes {
                    ptr: bytes.as_ptr(),
                    len: bytes.len(),
                },
                completion.id,
                self.policy as u32,
            )
        };
        if code == abi::OK {
            Ok(())
        } else {
            Err(code.into())
        }
    }
    /// Create a new complete regular file after commit, refusing any existing destination.
    /// Copies path/data into host-owned work (at most 1 MiB). Preserve/Cancel only;
    /// no Restart replay. Files are staged with private permissions in the same directory.
    /// Cancellation is checked before publication and cleans unpublished staging.
    /// A cancellation racing after the check cannot undo publication; a directory
    /// sync error can follow creation.
    pub fn create_file(
        self,
        cx: &Cx<'_>,
        path: &str,
        bytes: &[u8],
        completion: Action,
    ) -> Result<()> {
        let code = unsafe {
            (cx.api.create_file_async)(
                cx.api.context,
                self.id,
                abi::Bytes::new(path),
                abi::Bytes {
                    ptr: bytes.as_ptr(),
                    len: bytes.len(),
                },
                completion.id,
                self.policy as u32,
            )
        };
        if code == abi::OK {
            Ok(())
        } else {
            Err(code.into())
        }
    }
    /// Replace the opened resource's canonical pathname with complete contents.
    /// Runs after commit; preserves file mode, syncs the replacement then directory.
    /// Existing Preserve handles retain the old inode. No automatic Restart/replay.
    /// Cancellation is checked before publication and cleans unpublished staging.
    /// A cancellation racing after the check cannot undo a rename. A directory sync error
    /// is reported after replacement, so an error does not always mean unchanged content.
    pub fn save_file(
        self,
        cx: &Cx<'_>,
        file: Resource<File>,
        bytes: &[u8],
        completion: Action,
    ) -> Result<()> {
        let code = unsafe {
            (cx.api.save_file_async)(
                cx.api.context,
                self.id,
                file.id,
                abi::Bytes {
                    ptr: bytes.as_ptr(),
                    len: bytes.len(),
                },
                completion.id,
                self.policy as u32,
            )
        };
        if code == abi::OK {
            Ok(())
        } else {
            Err(code.into())
        }
    }
    /// Immutable latest result. None means pending/absent; Io contains the host I/O error.
    /// Results are bounded to 1 MiB; invalid lengths/statuses are rejected before copying.
    /// Cancel clears retained results. No result callback or buffer survives in the plugin.
    pub fn result(self, cx: &Cx<'_>) -> Result<Option<Vec<u8>>> {
        read_task_result(cx.api.context, self.id, cx.api.task_result)
    }
    /// Queue a timer after this action commits. Existing identity must be canceled
    /// before reuse. Timers survive implementation replacement; maximum delay is one day.
    pub fn delay<A: ActionToken>(
        self,
        cx: &Cx<'_>,
        duration: std::time::Duration,
        completion: A,
        value: A::Payload,
    ) -> Result<()> {
        let millis = timer_millis(duration)?;
        let (completion, value) = completion.encode(value);
        let code = unsafe {
            (cx.api.delay)(
                cx.api.context,
                self.id,
                millis,
                completion,
                value,
                self.policy as u32,
            )
        };
        if code == abi::OK {
            Ok(())
        } else {
            Err(code.into())
        }
    }
    /// Start a fixed-delay periodic task after commit. Preserved intervals retain their
    /// identity across reload. Missed ticks coalesce; callback delivery rearms the period.
    /// Supports the same Preserve/Cancel/Restart policy as delay, with period 1 ms–1 day.
    pub fn interval<A: ActionToken>(
        self,
        cx: &Cx<'_>,
        period: std::time::Duration,
        completion: A,
        value: A::Payload,
    ) -> Result<()> {
        let millis = timer_millis(period)?;
        if millis == 0 {
            return Err(Error::InvalidView("interval period must be positive"));
        }
        let (action, value) = completion.encode(value);
        let code = unsafe {
            (cx.api.interval)(
                cx.api.context,
                self.id,
                millis,
                action,
                value,
                self.policy as u32,
            )
        };
        if code == abi::OK {
            Ok(())
        } else {
            Err(code.into())
        }
    }
    /// Cancel pending work after commit. Ordinary delivered messages remain.
    /// File/socket completion callbacks require their cached result; cancellation clears
    /// that result and invalidates any queued I/O callback for this task.
    pub fn cancel(self, cx: &Cx<'_>) -> Result<()> {
        let code = unsafe { (cx.api.cancel_task)(cx.api.context, self.id) };
        if code == abi::OK {
            Ok(())
        } else {
            Err(code.into())
        }
    }
}

/// Data-only request identity, scoped by the runtime to the originating actor lifecycle.
/// Cancel commits with the current turn and delivers `RequestError::Cancelled`
/// through the request's error action if its completion has not entered the mailbox.
/// Executed target effects and already delivered completion messages remain.
/// Unknown, retired or another actor's identities are harmless no-ops after commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request(pub u64);
impl Request {
    pub fn cancel(self, cx: &Cx<'_>) -> Result<()> {
        let code = unsafe { (cx.api.cancel_request)(cx.api.context, self.0) };
        if code == abi::OK {
            Ok(())
        } else {
            Err(code.into())
        }
    }
}

/// Durable data-only address. Sending is committed only when the action succeeds.
/// The receiving actor must register the supplied action's semantic identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActorRef(pub u64);
impl ActorRef {
    /// Stage a bounded owned UTF-8 request. Reply/error actions execute in the
    /// sender's current implementation. No plugin closure or future is retained.
    /// Reply must be an ordinary text action; the target token must name a text
    /// request handler. Timeout is an integral number of milliseconds (1..=1 day).
    ///
    /// ```compile_fail
    /// use blockui::{Action, ActorRef, Cx, TextAction};
    /// fn wrong(cx: &Cx<'_>, query: TextAction, reply: Action, error: Action) {
    ///     ActorRef(2).ask_text(cx, query, String::new(), reply, error,
    ///         std::time::Duration::from_secs(1)).unwrap();
    /// }
    /// ```
    pub fn ask_text(
        self,
        cx: &Cx<'_>,
        request: TextAction,
        value: String,
        on_reply: TextAction,
        on_error: Action,
        timeout: std::time::Duration,
    ) -> Result<Request> {
        let millis = timeout.as_millis();
        if value.len() > abi::MAX_MESSAGE_TEXT_BYTES
            || millis == 0
            || millis > 86_400_000
            || !timeout.subsec_nanos().is_multiple_of(1_000_000)
        {
            return Err(Error::Host(abi::ERROR));
        }
        let mut id = 0;
        let code = unsafe {
            (cx.api.ask_actor_text)(
                cx.api.context,
                self.0,
                request.spec().id,
                abi::Bytes::new(&value),
                on_reply.spec().id,
                on_error.id,
                millis as u64,
                &mut id,
            )
        };
        if code == abi::OK {
            Ok(Request(id))
        } else {
            Err(code.into())
        }
    }
    /// Stage a scalar request; replies execute an action in the sender's current code.
    /// No plugin future or callback capture is retained while waiting.
    /// The returned data token can cancel the request in a later actor turn.
    ///
    /// ```compile_fail
    /// use blockui::{Action, ActorRef, Cx, FloatAction};
    /// fn wrong(cx: &Cx<'_>, request: FloatAction, reply: FloatAction, error: Action) {
    ///     ActorRef(2).ask(cx, request, 1_i64, reply, error, std::time::Duration::from_secs(1)).unwrap();
    /// }
    /// ```
    ///
    /// ```compile_fail
    /// use blockui::{Action, ActorRef, Cx, FloatAction};
    /// fn wrong_reply(cx: &Cx<'_>, request: FloatAction, reply: Action, error: Action) {
    ///     ActorRef(2).ask(cx, request, 0.25_f64, reply, error, std::time::Duration::from_secs(1)).unwrap();
    /// }
    /// ```
    pub fn ask<A: ActionToken, R: ActionToken<Payload = A::Reply>>(
        self,
        cx: &Cx<'_>,
        request: A,
        value: A::Payload,
        on_reply: R,
        on_error: Action,
        timeout: std::time::Duration,
    ) -> Result<Request> {
        let (request_id, value) = request.encode(value);
        let reply_id = on_reply.reply_id();
        let millis = timeout.as_millis();
        if millis == 0 || millis > 86_400_000 || !timeout.subsec_nanos().is_multiple_of(1_000_000) {
            return Err(Error::Host(abi::ERROR));
        }
        let mut id = 0;
        let code = unsafe {
            (cx.api.ask_actor)(
                cx.api.context,
                self.0,
                request_id,
                value,
                reply_id,
                on_error.id,
                millis as u64,
                &mut id,
            )
        };
        if code == abi::OK {
            Ok(Request(id))
        } else {
            Err(code.into())
        }
    }
    /// Queue a one-way message after this actor turn commits successfully.
    /// Acceptance does not imply that the recipient has processed the message.
    /// Committed messages survive sender termination but are discarded when the
    /// recipient terminates. ActorRef addresses the current actor under its ID;
    /// newly sent messages can reach a later lifecycle that reuses that ID.
    pub fn tell<A: TellToken>(self, cx: &Cx<'_>, action: A, value: A::Payload) -> Result<()> {
        action.send(cx, self, value)
    }
}

/// Named action dispatches through the currently loaded actor implementation.
#[derive(Clone, Copy)]
pub struct Action {
    pub id: u64,
    instance: InstancePath,
    handler: Handler,
}
impl Action {
    #[doc(hidden)]
    pub const fn __scoped(mut self, scope: Key) -> Self {
        self.id = scope.scoped(self.id).0;
        self.instance = self.instance.scoped(scope);
        self
    }
    #[doc(hidden)]
    pub const fn __instance(mut self, instance: InstancePath) -> Self {
        self.id = instance.resolve(self.id);
        self.instance = instance;
        self
    }

    /// A remote action identity; its handler is supplied by the receiving actor.
    /// This declaration cannot handle a local message.
    pub const fn remote(name: &str) -> Self {
        Self {
            id: abi::key(name),
            instance: InstancePath::EMPTY,
            handler: Handler::Integer(|_, _| Err(Error::UnknownAction)),
        }
    }
    /// A scalar request handler. Its returned value is staged as the turn's reply.
    /// A tell using this action fails and rolls back because no request is active.
    ///
    /// ```compile_fail
    /// use blockui::{Action, Cx, Result};
    /// fn wrong(_: &Cx<'_>, _: i64) -> Result<bool> { Ok(true) }
    /// let _ = Action::request("query", wrong);
    /// ```
    pub const fn request(name: &str, handler: for<'a> fn(&Cx<'a>, i64) -> Result<i64>) -> Self {
        Self {
            id: abi::key(name),
            instance: InstancePath::EMPTY,
            handler: Handler::Request(handler),
        }
    }
    /// Handle a request failure using a checked enum instead of raw status integers.
    /// Unknown wire codes reject the action before invoking the handler.
    ///
    /// ```compile_fail
    /// use blockui::{Action, Cx, Result};
    /// fn wrong(_: &Cx<'_>, _: i64) -> Result<()> { Ok(()) }
    /// let _ = Action::request_error("failed", wrong);
    /// ```
    pub const fn request_error(
        name: &str,
        handler: for<'a> fn(&Cx<'a>, RequestError) -> Result<()>,
    ) -> Self {
        Self {
            id: abi::key(name),
            instance: InstancePath::EMPTY,
            handler: Handler::RequestError(handler),
        }
    }
    pub const fn new(name: &str, handler: ActionFn) -> Self {
        Self {
            id: abi::key(name),
            instance: InstancePath::EMPTY,
            handler: Handler::Integer(handler),
        }
    }
}
#[derive(Clone, Copy)]
enum Handler {
    Integer(ActionFn),
    Point(for<'a> fn(&Cx<'a>, LocalPoint) -> Result<()>),
    Text(for<'a> fn(&Cx<'a>, String) -> Result<()>),
    KeyedText(for<'a> fn(&Cx<'a>, KeyedText) -> Result<()>),
    TextRequest(for<'a> fn(&Cx<'a>, String) -> Result<String>),
    RequestError(for<'a> fn(&Cx<'a>, RequestError) -> Result<()>),
    Request(for<'a> fn(&Cx<'a>, i64) -> Result<i64>),
    Boolean(for<'a> fn(&Cx<'a>, bool) -> Result<()>),
    BooleanRequest(for<'a> fn(&Cx<'a>, bool) -> Result<bool>),
    Float(for<'a> fn(&Cx<'a>, f64) -> Result<()>),
    FloatRequest(for<'a> fn(&Cx<'a>, f64) -> Result<f64>),
}
impl Handler {
    fn call(self, cx: &Cx<'_>, value: i64) -> Result<()> {
        if !matches!(
            self,
            Self::Text(_) | Self::TextRequest(_) | Self::KeyedText(_)
        ) && unsafe { (cx.api.read_message_text)(cx.api.context, std::ptr::null_mut(), 0) } >= 0
        {
            return Err(Error::Host(abi::ERROR));
        }
        match self {
            Self::Text(handler) => handler(cx, cx.message_text()?),
            Self::KeyedText(handler) => {
                let wire = cx.message_text()?;
                let (key, text) = abi::keyed_text::decode(&wire)
                    .map_err(|reason| Error::Application(reason.into()))?;
                handler(
                    cx,
                    KeyedText {
                        key: key.into(),
                        text: text.into(),
                    },
                )
            }
            Self::TextRequest(handler) => cx.reply_text(&handler(cx, cx.message_text()?)?),
            Self::Integer(handler) => handler(cx, value),
            Self::Point(handler) => handler(
                cx,
                LocalPoint::from_wire(value).ok_or(Error::Host(abi::ERROR))?,
            ),
            Self::Float(handler) => handler(cx, f64::from_wire(value)?),
            Self::FloatRequest(handler) => {
                cx.reply(handler(cx, f64::from_wire(value)?)?.into_wire())
            }
            Self::RequestError(handler) => handler(
                cx,
                RequestError::from_code(value).ok_or(Error::Host(abi::ERROR))?,
            ),
            Self::Request(handler) => cx.reply(handler(cx, value)?),
            Self::BooleanRequest(handler) => {
                cx.reply(handler(cx, bool::from_wire(value)?)?.into_wire())
            }
            Self::Boolean(handler) => handler(cx, bool::from_wire(value)?),
        }
    }
}
mod action_sealed {
    pub trait Sealed {}
}
/// A message token with a Rust payload type; only scalar data crosses the ABI.
pub trait ActionToken: action_sealed::Sealed + Copy {
    type Payload: Scalar;
    type Reply: Scalar;
    #[doc(hidden)]
    fn reply_id(self) -> u64;
    #[doc(hidden)]
    fn encode(self, value: Self::Payload) -> (u64, i64) {
        (self.reply_id(), value.into_wire())
    }
}
impl action_sealed::Sealed for Action {}
impl ActionToken for Action {
    type Payload = i64;
    type Reply = i64;
    fn reply_id(self) -> u64 {
        self.id
    }
}
/// Local representation for typed admission. Never passed across a DLL boundary.
#[doc(hidden)]
pub enum MessagePayload {
    Scalar(i64),
    Text(String),
}
/// One-way message token. Payload ownership remains local on each side of the ABI.
pub trait TellToken: action_sealed::Sealed + Copy {
    type Payload;
    #[doc(hidden)]
    fn encode_message(self, value: Self::Payload) -> Result<(u64, MessagePayload)>;
    #[doc(hidden)]
    fn send(self, cx: &Cx<'_>, target: ActorRef, value: Self::Payload) -> Result<()> {
        let (id, payload) = self.encode_message(value)?;
        let code = match payload {
            MessagePayload::Scalar(value) => unsafe {
                (cx.api.send_actor)(cx.api.context, target.0, id, value)
            },
            MessagePayload::Text(value) => unsafe {
                (cx.api.send_actor_text)(cx.api.context, target.0, id, abi::Bytes::new(&value))
            },
        };
        if code == abi::OK {
            Ok(())
        } else {
            Err(Error::Host(code))
        }
    }
}
impl<A: ActionToken> TellToken for A {
    type Payload = A::Payload;
    fn encode_message(self, value: Self::Payload) -> Result<(u64, MessagePayload)> {
        let (id, value) = self.encode(value);
        Ok((id, MessagePayload::Scalar(value)))
    }
}
/// One-way typed canvas coordinate action. No callback escapes the loaded table.
#[derive(Clone, Copy)]
pub struct PointAction(Action);
impl PointAction {
    #[doc(hidden)]
    pub const fn __instance(self, instance: InstancePath) -> Self {
        Self(self.0.__instance(instance))
    }

    pub const fn new(name: &str, handler: for<'a> fn(&Cx<'a>, LocalPoint) -> Result<()>) -> Self {
        Self(Action {
            id: abi::key(name),
            instance: InstancePath::EMPTY,
            handler: Handler::Point(handler),
        })
    }
    pub const fn remote(name: &str) -> Self {
        Self(Action::remote(name))
    }
    pub const fn spec(self) -> Action {
        self.0
    }
}
impl action_sealed::Sealed for PointAction {}
impl TellToken for PointAction {
    type Payload = LocalPoint;
    fn encode_message(self, point: LocalPoint) -> Result<(u64, MessagePayload)> {
        let value = point.to_wire().ok_or(Error::Host(abi::ERROR))?;
        Ok((self.0.id, MessagePayload::Scalar(value)))
    }
}

/// One row's text edit. These owned values exist only during a plugin call;
/// the runtime keeps a bounded encoded message, never a Rust struct or callback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedText {
    pub key: String,
    pub text: String,
}
/// Shared text handler for keyed record collection editors and ordinary tells.
///
/// ```
/// use blockui::{ActorRef, Cx, KeyedText, KeyedTextAction, Result};
/// fn changed(_: &Cx<'_>, edit: KeyedText) -> Result<()> {
///     assert!(!edit.key.is_empty());
///     Ok(())
/// }
/// let action = KeyedTextAction::new("changed", changed);
/// fn send(cx: &Cx<'_>, target: ActorRef) -> Result<()> {
///     target.tell(cx, KeyedTextAction::remote("changed"), KeyedText {
///         key: "document-42".into(), text: "Updated".into(),
///     })
/// }
/// ```
/// A keyed edit cannot target a single-record source:
/// ```compile_fail
/// use blockui::{Record, State, KeyedTextAction, TextInput};
/// #[derive(Record, Default)] struct Document { name: String }
/// let document = State::<Document>::new("document");
/// TextInput::field(document.text(Document::name_field()), KeyedTextAction::remote("edit"));
/// ```
#[derive(Clone, Copy)]
pub struct KeyedTextAction(Action);
impl KeyedTextAction {
    #[doc(hidden)]
    pub const fn __instance(self, instance: InstancePath) -> Self {
        Self(self.0.__instance(instance))
    }

    pub const fn new(name: &str, handler: for<'a> fn(&Cx<'a>, KeyedText) -> Result<()>) -> Self {
        Self(Action {
            id: abi::key(name),
            instance: InstancePath::EMPTY,
            handler: Handler::KeyedText(handler),
        })
    }
    pub const fn remote(name: &str) -> Self {
        Self::new(name, |_, _| Err(Error::UnknownAction))
    }
    pub const fn spec(self) -> Action {
        self.0
    }
}
impl action_sealed::Sealed for KeyedTextAction {}
impl TellToken for KeyedTextAction {
    type Payload = KeyedText;
    fn encode_message(self, value: KeyedText) -> Result<(u64, MessagePayload)> {
        let wire = abi::keyed_text::encode(&value.key, &value.text)
            .map_err(|reason| Error::Application(reason.into()))?;
        Ok((self.0.id, MessagePayload::Text(wire)))
    }
}
/// Typed edit action accepted by the matching record source. A keyed action
/// requires a collection source; ordinary TextAction remains compatible.
pub trait RecordTextAction<S>: action_sealed::Sealed {
    #[doc(hidden)]
    fn into_record_action(self) -> TextAction;
}
impl<S> RecordTextAction<S> for TextAction {
    fn into_record_action(self) -> TextAction {
        self
    }
}
impl RecordTextAction<RecordMapText> for KeyedTextAction {
    fn into_record_action(self) -> TextAction {
        TextAction(self.0)
    }
}
/// A bounded UTF-8 ordinary message, resolved through the current implementation.
#[derive(Clone, Copy)]
pub struct TextAction(Action);
impl TextAction {
    #[doc(hidden)]
    pub const fn __instance(self, instance: InstancePath) -> Self {
        Self(self.0.__instance(instance))
    }

    /// Text input and text reply handler. A tell rejects and rolls back without a reply scope.
    pub const fn request(
        name: &str,
        handler: for<'a> fn(&Cx<'a>, String) -> Result<String>,
    ) -> Self {
        Self(Action {
            id: abi::key(name),
            instance: InstancePath::EMPTY,
            handler: Handler::TextRequest(handler),
        })
    }
    pub const fn new(name: &str, handler: for<'a> fn(&Cx<'a>, String) -> Result<()>) -> Self {
        Self(Action {
            id: abi::key(name),
            instance: InstancePath::EMPTY,
            handler: Handler::Text(handler),
        })
    }
    pub const fn remote(name: &str) -> Self {
        Self(Action::remote(name))
    }
    pub const fn spec(self) -> Action {
        self.0
    }
}
impl action_sealed::Sealed for TextAction {}
impl TellToken for TextAction {
    type Payload = String;
    fn encode_message(self, value: String) -> Result<(u64, MessagePayload)> {
        if value.len() > abi::MAX_MESSAGE_TEXT_BYTES {
            return Err(Error::Host(abi::ERROR));
        }
        Ok((self.0.id, MessagePayload::Text(value)))
    }
}

mod scalar_sealed {
    pub trait Sealed {}
    impl Sealed for i64 {}
    impl Sealed for bool {}
    impl Sealed for f64 {}
}
/// Primitive data carried by actor messages; no Rust allocation crosses the ABI.
pub trait Scalar: scalar_sealed::Sealed + Copy {
    #[doc(hidden)]
    fn from_wire(value: i64) -> Result<Self>;
    #[doc(hidden)]
    fn into_wire(self) -> i64;
}
impl Scalar for i64 {
    fn from_wire(value: i64) -> Result<Self> {
        Ok(value)
    }
    fn into_wire(self) -> i64 {
        self
    }
}
impl Scalar for bool {
    fn from_wire(value: i64) -> Result<Self> {
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::Host(abi::ERROR)),
        }
    }
    fn into_wire(self) -> i64 {
        i64::from(self)
    }
}
impl Scalar for f64 {
    fn from_wire(value: i64) -> Result<Self> {
        Ok(Self::from_bits(value as u64))
    }
    fn into_wire(self) -> i64 {
        self.to_bits() as i64
    }
}
/// A typed request with independently declared input and reply types.
/// Construct a local handler with `request!`; register its `spec()` in ACTIONS.
/// Remote declarations must match the receiving application's message contract.
///
/// ```compile_fail
/// use blockui::{Action, ActorRef, BoolAction, Cx, RequestAction};
/// fn wrong(cx: &Cx<'_>, reply: BoolAction, error: Action) {
///     ActorRef(2).ask(cx, RequestAction::<bool, f64>::remote("query"), true,
///         reply, error, std::time::Duration::from_secs(1)).unwrap();
/// }
/// ```
///
/// ```compile_fail
/// use blockui::{Action, ActorRef, Cx, FloatAction, RequestAction};
/// fn wrong(cx: &Cx<'_>, reply: FloatAction, error: Action) {
///     ActorRef(2).ask(cx, RequestAction::<bool, f64>::remote("query"), 1_i64,
///         reply, error, std::time::Duration::from_secs(1)).unwrap();
/// }
/// ```
#[derive(Clone, Copy)]
pub struct RequestAction<Input: Scalar, Reply: Scalar> {
    action: Action,
    types: PhantomData<fn(Input) -> Reply>,
}
impl<Input: Scalar, Reply: Scalar> RequestAction<Input, Reply> {
    #[doc(hidden)]
    pub const fn __instance(mut self, instance: InstancePath) -> Self {
        self.action = self.action.__instance(instance);
        self
    }

    pub const fn remote(name: &str) -> Self {
        Self {
            action: Action::remote(name),
            types: PhantomData,
        }
    }
    #[doc(hidden)]
    pub const fn __new(name: &str, handler: for<'a> fn(&Cx<'a>, i64) -> Result<i64>) -> Self {
        Self {
            action: Action::request(name, handler),
            types: PhantomData,
        }
    }
    pub const fn spec(self) -> Action {
        self.action
    }
}
impl<Input: Scalar, Reply: Scalar> action_sealed::Sealed for RequestAction<Input, Reply> {}
impl<Input: Scalar, Reply: Scalar> ActionToken for RequestAction<Input, Reply> {
    type Payload = Input;
    type Reply = Reply;
    fn reply_id(self) -> u64 {
        self.action.id
    }
}
/// Declare a request using ordinary Rust handler expressions and primitive types.
///
/// ```
/// use blockui::{request, Cx, RequestAction, Result};
/// fn query(_: &Cx<'_>, input: bool) -> Result<f64> { Ok(if input { 1.5 } else { -0.0 }) }
/// const QUERY: RequestAction<bool, f64> = request!(bool => f64, "query", query);
/// ```
///
/// ```compile_fail
/// use blockui::{request, Cx, Result};
/// fn wrong(_: &Cx<'_>, _: bool) -> Result<i64> { Ok(1) }
/// let _ = request!(bool => f64, "query", wrong);
/// ```
///
/// ```compile_fail
/// use blockui::request;
/// let _ = request!(String => i64, "query", |_, _| Ok(1));
/// ```
#[macro_export]
macro_rules! request {
    ($input:ty => $reply:ty, $name:expr, $handler:expr) => {
        $crate::RequestAction::<$input, $reply>::__new($name, |cx, wire| {
            let input = <$input as $crate::Scalar>::from_wire(wire)?;
            let handler: for<'a> fn(&$crate::Cx<'a>, $input) -> $crate::Result<$reply> = $handler;
            let reply = handler(cx, input)?;
            Ok(<$reply as $crate::Scalar>::into_wire(reply))
        })
    };
}

/// Boolean message declaration. Register `spec()` in `Actor::ACTIONS`.
///
/// ```compile_fail
/// use blockui::{ActorRef, BoolAction, Cx};
/// fn wrong(cx: &Cx<'_>, action: BoolAction) {
///     ActorRef(1).tell(cx, action, 1).unwrap();
/// }
/// ```
#[derive(Clone, Copy)]
pub struct BoolAction(Action);
impl BoolAction {
    #[doc(hidden)]
    pub const fn __instance(self, instance: InstancePath) -> Self {
        Self(self.0.__instance(instance))
    }

    /// Boolean request input and reply; rejects nonboolean wire values.
    pub const fn request(name: &str, handler: for<'a> fn(&Cx<'a>, bool) -> Result<bool>) -> Self {
        Self(Action {
            id: abi::key(name),
            instance: InstancePath::EMPTY,
            handler: Handler::BooleanRequest(handler),
        })
    }
    pub const fn new(name: &str, handler: for<'a> fn(&Cx<'a>, bool) -> Result<()>) -> Self {
        Self(Action {
            id: abi::key(name),
            instance: InstancePath::EMPTY,
            handler: Handler::Boolean(handler),
        })
    }
    pub const fn remote(name: &str) -> Self {
        Self(Action::remote(name))
    }
    pub const fn spec(self) -> Action {
        self.0
    }
}
impl action_sealed::Sealed for BoolAction {}
impl ActionToken for BoolAction {
    type Payload = bool;
    type Reply = bool;
    fn reply_id(self) -> u64 {
        self.0.id
    }
}
/// Floating-point message declaration, preserving every IEEE bit pattern.
/// Register `spec()` in `Actor::ACTIONS`; only scalar bits cross the ABI.
///
/// ```compile_fail
/// use blockui::{ActorRef, FloatAction, Cx};
/// fn wrong(cx: &Cx<'_>, action: FloatAction) {
///     ActorRef(1).tell(cx, action, 1_i64).unwrap();
/// }
/// ```
#[derive(Clone, Copy)]
pub struct FloatAction(Action);
impl FloatAction {
    #[doc(hidden)]
    pub const fn __instance(self, instance: InstancePath) -> Self {
        Self(self.0.__instance(instance))
    }

    /// A floating-point request input and reply, preserving IEEE bits.
    /// Tells reject/roll back because no request reply scope is active.
    pub const fn request(name: &str, handler: for<'a> fn(&Cx<'a>, f64) -> Result<f64>) -> Self {
        Self(Action {
            id: abi::key(name),
            instance: InstancePath::EMPTY,
            handler: Handler::FloatRequest(handler),
        })
    }
    pub const fn new(name: &str, handler: for<'a> fn(&Cx<'a>, f64) -> Result<()>) -> Self {
        Self(Action {
            id: abi::key(name),
            instance: InstancePath::EMPTY,
            handler: Handler::Float(handler),
        })
    }
    pub const fn remote(name: &str) -> Self {
        Self(Action::remote(name))
    }
    pub const fn spec(self) -> Action {
        self.0
    }
}
impl action_sealed::Sealed for FloatAction {}
impl ActionToken for FloatAction {
    type Payload = f64;
    type Reply = f64;
    fn reply_id(self) -> u64 {
        self.0.id
    }
}
impl Default for Action {
    fn default() -> Self {
        Self {
            id: 0,
            instance: InstancePath::EMPTY,
            handler: Handler::Integer(|_, _| Err(Error::UnknownAction)),
        }
    }
}

/// Implementation-local structural builder with owned captures.
pub type ScopeCompute = dyn for<'a> Fn(&Cx<'a>) -> Result<View>;

/// Implementation-local captured text computation.
pub type TextCompute = dyn for<'a> Fn(&Cx<'a>) -> Result<String>;
pub type BindingFn = for<'a> fn(&Cx<'a>) -> Result<String>;
/// A binding is plugin-local; the host retains only its semantic key and copied output.
/// Function items and noncapturing closures use the lightweight function path.
/// `bind!(move |cx| ...)` captures owned parameters and state handles inside the
/// current implementation; a retained context is never accepted.
///
/// ```
/// use blockui::{bind, State, Text};
/// let count = State::<i64>::new("count", 0);
/// let caption = String::from("Count");
/// let label = Text::new(bind!(move |cx| Ok(format!("{caption}: {}", count.get(cx)?))));
/// ```
///
/// ```compile_fail
/// use blockui::bind;
/// let transient = String::from("owned by old code");
/// let _ = bind!(|_| Ok(transient.clone()));
/// ```
///
/// ```compile_fail
/// use blockui::{bind, Cx, State};
/// fn retain_context(cx: &Cx<'_>) {
///     let _ = bind!(move |_| Ok(State::<i64>::new("count", 0).get(cx)?.to_string()));
/// }
/// ```
/// A text binding is lazy: mount it in a widget to subscribe and evaluate.
/// Discarding an adapter does not create a reactive observer.
#[must_use = "mount this text binding in a widget; constructing it does not subscribe or evaluate"]
pub enum Binding {
    #[doc(hidden)]
    RecordScalar(RecordScalarText),
    RecordSelect {
        value: RecordInteger,
        wire: String,
    },
    RecordProgress {
        value: RecordFloat,
        label: String,
    },
    RecordSlider {
        value: RecordInteger,
        label: String,
    },
    RecordCheckbox {
        value: RecordBoolean,
        label: String,
    },
    #[doc(hidden)]
    RecordText {
        value: RecordText,
        label: String,
    },
    #[doc(hidden)]
    RecordMapText {
        value: RecordMapText,
        label: String,
        keyed: bool,
    },
    #[doc(hidden)]
    ComputedEditor {
        value: Computed<String>,
        label: String,
    },
    #[doc(hidden)]
    Scope(u64),
    #[doc(hidden)]
    CapturedScope {
        id: u64,
        build: std::rc::Rc<ScopeCompute>,
        source: Option<std::rc::Rc<()>>,
    },
    ComputedText(Computed<String>),
    ComputedBoolean(Computed<bool>),
    ComputedFloat(Computed<f64>),
    ComputedInteger(Computed<i64>),
    Static(String),
    Reactive(BindingFn),
    Captured(std::rc::Rc<TextCompute>),
    #[doc(hidden)]
    NamedCaptured {
        identity: Key,
        compute: std::rc::Rc<TextCompute>,
    },
    NamedReactive {
        identity: Key,
        compute: BindingFn,
    },
    MappedInteger {
        slot: u64,
        format: fn(i64) -> String,
    },
    MappedList {
        slot: u64,
        format: fn(Vec<i64>) -> String,
    },
    MappedTextList {
        slot: u64,
        format: fn(Vec<String>) -> String,
    },
    MappedText {
        slot: u64,
        format: fn(String) -> String,
    },
    MappedBoolean {
        slot: u64,
        format: fn(bool) -> String,
    },
    MappedFloat {
        slot: u64,
        format: fn(f64) -> String,
    },
}
impl Binding {
    /// Capture owned component parameters and state handles. Read handles inside
    /// the callback with its fresh context; borrowed view contexts cannot escape.
    pub fn captured(compute: impl for<'a> Fn(&Cx<'a>) -> Result<String> + 'static) -> Self {
        Self::__owned_captured(Box::new(compute))
    }
    // Keep Rc conversion concrete instead of repeating it for every closure
    // type. The temporary Box can add allocation/copy work at construction;
    // evaluation already dispatches through the implementation-local callback.
    fn __owned_captured(compute: Box<TextCompute>) -> Self {
        Self::Captured(std::rc::Rc::from(compute))
    }
    pub fn reactive(compute: BindingFn) -> Self {
        Self::Reactive(compute)
    }
    /// A stable semantic name for a reactive binding, independent of its callback address.
    pub fn named(name: &str, compute: BindingFn) -> Self {
        Self::NamedReactive {
            identity: Key::named(name),
            compute,
        }
    }
    #[doc(hidden)]
    pub fn __path(module: &str, path: &str, compute: BindingFn) -> Self {
        Self::NamedReactive {
            identity: Key::named(module).scoped(abi::key(path)),
            compute,
        }
    }
    fn identity(&self) -> Option<Key> {
        match self {
            Self::NamedCaptured { identity, .. } => Some(*identity),
            Self::ComputedText(value) => {
                Some(Key::named("identity.binding.computed.text").scoped(value.key))
            }
            Self::ComputedInteger(value) => {
                Some(Key::named("identity.binding.computed.integer").scoped(value.key))
            }
            Self::ComputedFloat(value) => {
                Some(Key::named("identity.binding.computed.float").scoped(value.key))
            }
            Self::ComputedBoolean(value) => {
                Some(Key::named("identity.binding.computed.boolean").scoped(value.key))
            }
            Self::RecordCheckbox { value, .. } => {
                Some(value.0.identity_key("identity.binding.record.checkbox"))
            }
            Self::RecordSlider { value, .. } => {
                Some(value.0.identity_key("identity.binding.record.slider"))
            }
            Self::RecordProgress { value, .. } => {
                Some(value.0.identity_key("identity.binding.record.progress"))
            }
            Self::RecordSelect { value, .. } => {
                Some(value.0.identity_key("identity.binding.record.select"))
            }
            Self::RecordScalar(value) => {
                Some(value.value.identity_key("identity.binding.record.scalar"))
            }
            Self::RecordMapText { value, .. } => {
                Some(value.identity_key("identity.binding.record.map.text"))
            }
            Self::RecordText { value, .. } => {
                Some(value.identity_key("identity.binding.record.text"))
            }
            Self::Scope(id) | Self::CapturedScope { id, .. } => Some(Key(*id)),
            Self::NamedReactive { identity, .. } => Some(*identity),
            Self::MappedFloat { slot, .. } => {
                Some(Key::named("identity.binding.mapped.float").scoped(*slot))
            }
            Self::MappedBoolean { slot, .. } => {
                Some(Key::named("identity.binding.mapped.boolean").scoped(*slot))
            }
            Self::MappedText { slot, .. } => {
                Some(Key::named("identity.binding.mapped.text").scoped(*slot))
            }
            Self::MappedList { slot, .. } => {
                Some(Key::named("identity.binding.mapped.list").scoped(*slot))
            }
            Self::MappedTextList { slot, .. } => {
                Some(Key::named("identity.binding.mapped.text_list").scoped(*slot))
            }
            Self::MappedInteger { slot, .. } => {
                Some(Key::named("identity.binding.mapped.integer").scoped(*slot))
            }
            _ => None,
        }
    }
}
impl Default for Binding {
    fn default() -> Self {
        Self::Static(String::new())
    }
}
/// Bind a registered text computation directly; no capturing callback is retained.
/// Include `CAPTION.spec()` in the actor's `COMPUTEDS` registry.
/// ```
/// use blockui::{Computed, Text};
/// const CAPTION: Computed<String> = Computed::<String>::new("caption", |_| Ok("Hello".into()));
/// let label = Text::new(CAPTION);
/// ```
impl From<Computed<String>> for Binding {
    fn from(value: Computed<String>) -> Self {
        Self::ComputedText(value)
    }
}
impl From<RecordMapText> for Binding {
    fn from(value: RecordMapText) -> Self {
        Self::RecordMapText {
            value,
            label: String::new(),
            keyed: false,
        }
    }
}
impl From<RecordListText> for Binding {
    fn from(value: RecordListText) -> Self {
        value.0.into()
    }
}
impl From<RecordText> for Binding {
    fn from(value: RecordText) -> Self {
        Self::RecordText {
            value,
            label: String::new(),
        }
    }
}
impl From<RecordScalarText> for Binding {
    fn from(value: RecordScalarText) -> Self {
        Self::RecordScalar(value)
    }
}
/// Display a registered computation using its default Rust value formatting.
impl From<Computed<i64>> for Binding {
    fn from(value: Computed<i64>) -> Self {
        Self::ComputedInteger(value)
    }
}
/// Display a registered computation using its default Rust value formatting.
impl From<Computed<f64>> for Binding {
    fn from(value: Computed<f64>) -> Self {
        Self::ComputedFloat(value)
    }
}
/// Display a registered computation using its default Rust value formatting.
impl From<Computed<bool>> for Binding {
    fn from(value: Computed<bool>) -> Self {
        Self::ComputedBoolean(value)
    }
}
/// Display the live slot using its default value formatting.
/// ```
/// use blockui::{Cx, Result, State, Text};
/// const COUNT: State<i64> = State::<i64>::new("count", 0);
/// let label = Text::new(COUNT);
/// fn bound_label(cx: &Cx<'_>) -> Result<Text> {
///     let field = COUNT.bind(cx)?;
///     Ok(Text::new(&field)) // returned widget has no context lifetime
/// }
/// ```
impl From<State<i64>> for Binding {
    fn from(value: State<i64>) -> Self {
        value.map_text(|value| value.to_string())
    }
}
/// Copy only the slot identity; the binding does not retain the turn context.
impl From<&BoundState<'_, '_, i64>> for Binding {
    fn from(value: &BoundState<'_, '_, i64>) -> Self {
        value.state.into()
    }
}
/// Display the live slot using its default value formatting.
impl From<State<f64>> for Binding {
    fn from(value: State<f64>) -> Self {
        value.map_text(|value| value.to_string())
    }
}
/// Copy only the slot identity; the binding does not retain the turn context.
impl From<&BoundState<'_, '_, f64>> for Binding {
    fn from(value: &BoundState<'_, '_, f64>) -> Self {
        value.state.into()
    }
}
/// Display the live slot using its default value formatting.
impl From<State<bool>> for Binding {
    fn from(value: State<bool>) -> Self {
        value.map_text(|value| value.to_string())
    }
}
/// Copy only the slot identity; the binding does not retain the turn context.
impl From<&BoundState<'_, '_, bool>> for Binding {
    fn from(value: &BoundState<'_, '_, bool>) -> Self {
        value.state.into()
    }
}
/// Display the live slot using its default value formatting.
impl From<State<String>> for Binding {
    fn from(value: State<String>) -> Self {
        value.map_text(|value| value)
    }
}
/// Copy only the slot identity; the binding does not retain the turn context.
impl From<&BoundState<'_, '_, String>> for Binding {
    fn from(value: &BoundState<'_, '_, String>) -> Self {
        value.state.into()
    }
}
impl From<&str> for Binding {
    fn from(value: &str) -> Self {
        Self::Static(value.into())
    }
}
impl From<String> for Binding {
    fn from(value: String) -> Self {
        Self::Static(value)
    }
}

/// Ephemeral view tree; it never leaves the current library call.
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct View {
    reactive_style: Option<std::rc::Rc<StyleCompute>>,
    icon: Option<abi::icon::IconData>,
    explicit_identity: bool,
    local_identity: Option<&'static str>,
    event: Option<std::rc::Rc<__events::Callback>>,
    node: abi::Node,
    text: Binding,
    children: Vec<View>,
}
pub use abi::theme::{Density, ThemeMode};
pub mod theme {
    pub use blockui_abi::theme::*;
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextTone {
    #[default]
    Default,
    Muted,
    Accent,
    /// Shared readable error feedback; keep actionable text as well as color.
    Error,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Surface {
    #[default]
    Default,
    Canvas,
    Panel,
}
/// Shared presentation properties. Zero dimensions keep the native defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewStyle {
    pub width: f32,
    pub max_width: f32,
    pub font_size: f32,
    pub text_tone: TextTone,
    pub surface: Surface,
    pub radius: f32,
    pub theme: ThemeMode,
    pub density: Density,
    /// Fill remaining height in a bounded container; useful for independent scroll areas.
    pub fill_height: bool,
}
/// Presentation sugar for any widget; the canonical output is still a View.
pub type StyleFn = for<'a> fn(&Cx<'a>) -> Result<ViewStyle>;
pub(crate) type StyleCompute = dyn for<'a> Fn(&Cx<'a>) -> Result<ViewStyle>;

pub trait Styled: IntoView + Sized {
    /// Bind this node's presentation to tracked state reads. The function runs
    /// inside the current implementation; it must not capture a turn context.
    /// Text and style track their dependencies independently. Conditional reads
    /// replace old dependencies after each evaluation, and writes are forbidden.
    /// The result is a complete presentation, replacing static style values.
    /// A second call replaces the previous reactive style computation.
    /// Owned parameters and state handles may be captured. Captures stay inside
    /// the current implementation and are disposed on unmount/replacement.
    /// Reads must happen inside the callback to remain reactive:
    ///
    /// ```
    /// use blockui::*;
    /// fn component(invalid: State<bool>, caption: String) -> View {
    ///     Text::new(caption).style_reactive(move |cx| Ok(ViewStyle {
    ///         text_tone: if invalid.get(cx)? { TextTone::Error } else { TextTone::Default },
    ///         ..default()
    ///     }))
    /// }
    /// ```
    ///
    /// ```
    /// use blockui::*;
    /// const INVALID: State<bool> = State::<bool>::new("form.invalid", false);
    /// let label = Text::new("Temperature").style_reactive(|cx| {
    ///     Ok(ViewStyle {
    ///         text_tone: if INVALID.get(cx)? { TextTone::Error } else { TextTone::Default },
    ///         ..default()
    ///     })
    /// });
    /// ```
    ///
    /// Capturing a current view context is rejected:
    /// ```compile_fail
    /// use blockui::*;
    /// fn view(cx: &Cx<'_>) -> View {
    ///     Text::new("label").style_reactive(move |_| {
    ///         let _ = State::<i64>::new("count", 0).get(cx)?;
    ///         Ok(default())
    ///     })
    /// }
    /// ```
    fn style_reactive(
        self,
        compute: impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static,
    ) -> View {
        self.into_view()
            .with_reactive_style(std::rc::Rc::new(compute))
    }

    fn style(self, style: impl Into<StyleBinding>) -> View {
        self.into_view().style(style)
    }
}
impl<T: IntoView> Styled for T {}
impl View {
    // Shared by Button::children and block construction. A Text label is data
    // belonging to the button, never a separate native child of that button.
    pub(crate) fn finish_children(&mut self) {
        if self.node.kind == abi::BUTTON
            && self.children.len() == 1
            && self.children[0].node.kind == abi::LABEL
            && self.children[0].children.is_empty()
        {
            self.text = self.children.pop().unwrap().text;
        }
    }

    /// Add a leading monochrome icon without introducing another focus or hit target.
    pub fn leading_icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into().data(false));
        self
    }

    /// See [`Styled::style_reactive`] for tracked reads and replacement semantics.
    pub fn style_reactive(
        self,
        compute: impl for<'a> Fn(&Cx<'a>) -> Result<ViewStyle> + 'static,
    ) -> Self {
        self.with_reactive_style(std::rc::Rc::new(compute))
    }
    // Erase at the API entry so widget adapters do not instantiate another
    // generic View method for each component closure.
    fn with_reactive_style(mut self, compute: std::rc::Rc<StyleCompute>) -> Self {
        self.reactive_style = Some(compute);
        self
    }

    /// Mark an ordinary navigation button as the current page. Admission rejects
    /// this metadata on incompatible widget roles.
    pub fn current_page(mut self, current: bool) -> Self {
        self.node.semantics.current = u32::from(current);
        self
    }

    /// Set this node's static or dynamic style, composing explicit patches in tuples.
    /// A later call replaces the earlier style observer as well as its values.
    pub fn style(self, style: impl Into<StyleBinding>) -> Self {
        style.into().apply_to(self)
    }
}
impl ViewStyle {
    fn presentation(self) -> abi::Presentation {
        let style = self;
        abi::Presentation {
            width: style.width,
            max_width: style.max_width,
            font_size: style.font_size,
            text_tone: match style.text_tone {
                TextTone::Default => 0,
                TextTone::Muted => 1,
                TextTone::Accent => 2,
                TextTone::Error => 3,
            },
            surface: style.surface as u32,
            radius: style.radius,
            theme: style.theme as u32,
            density: style.density as u32,
            fill_height: u32::from(style.fill_height),
        }
    }
}
impl View {
    // Every actor projection has exactly one root; its identity needs no source location.
    fn actor_root(mut self) -> Self {
        if self.node.key == 0 {
            self.node.key = abi::key("identity.auto.actor.root");
        }
        self.assign_static_children();
        self
    }
    fn assign_static_children(&mut self) {
        for child in &mut self.children {
            if child.node.key == 0
                && (child.local_identity.is_some()
                    || matches!(
                        child.node.kind,
                        abi::COLUMN | abi::ROW | abi::LABEL | abi::SCROLL
                    ))
            {
                child.node.key = if let Some(name) = child.local_identity {
                    Key::named("identity.auto.child.local")
                        .scoped(self.node.key)
                        .scoped(abi::key(name))
                        .0
                } else {
                    Key::named("identity.auto.child.kind")
                        .scoped(self.node.key)
                        .scoped(child.node.kind as u64)
                        .0
                };
            }
            child.assign_static_children();
        }
    }
    /// Set this node's stable identity. Explicit/semantic child identities remain unchanged.
    /// Unnamed static children derive their default identity from their parent at projection.
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.node.key = key.into().0;
        self.explicit_identity = self.node.key != 0;
        self.local_identity = None;
        self
    }
    /// An empty container that occupies no layout space and is absent from
    /// semantic queries. Useful as a closed structural scope's returned root.
    pub fn empty() -> Self {
        let mut view = Column::default().children(Vec::new());
        view.node.semantics.hidden = 1;
        view
    }
    /// Use another visible node as this control's accessible name.
    pub fn labelled_by(mut self, key: impl Into<Key>) -> Self {
        self.node.semantics.labelled_by = key.into().0;
        self
    }
    /// Internal #[node] instance hint; explicit keys win over names and binding defaults.
    #[doc(hidden)]
    pub fn __local_identity(mut self, name: &'static str) -> Self {
        if !self.explicit_identity {
            self.local_identity = Some(name);
            self.node.key = 0;
        }
        self
    }
    /// Append children to an existing view, retaining the component's own children.
    /// This also lets styled or custom IntoView components act as UI parents.
    pub fn children(mut self, children: Vec<View>) -> Self {
        self.children.extend(children);
        self.finish_children();
        self
    }
    pub fn push(&mut self, child: impl IntoView) {
        self.children.push(child.into_view());
    }
    pub fn set_gap(&mut self, gap: f32) {
        self.node.gap = gap;
    }
    #[cfg(test)]
    fn flatten(&self, parent: u64, output: &mut Vec<abi::Node>) -> Result<()> {
        let mut events = Vec::new();
        self.flatten_events(parent, output, &mut events)?;
        if !events.is_empty() {
            return Err(Error::InvalidView(
                "closure events currently require a root component projection",
            ));
        }
        Ok(())
    }
    fn flatten_events(
        &self,
        parent: u64,
        output: &mut Vec<abi::Node>,
        events: &mut Vec<(u64, std::rc::Rc<__events::Callback>)>,
    ) -> Result<()> {
        if output.len() >= abi::MAX_VIEW_NODES {
            return Err(Error::InvalidView("view exceeds node budget"));
        }
        let mut node = self.node;
        node.parent = parent;
        node.binding = if matches!(node.kind, abi::ENTRY | abi::TEXTAREA) {
            match &self.text {
                Binding::ComputedEditor { value, .. } => {
                    Key::named("identity.binding.editor.computed")
                        .scoped(value.key)
                        .0
                }
                Binding::RecordMapText { value, .. } => {
                    value.identity_key("identity.binding.record.map.editor").0
                }
                Binding::RecordText { value, .. } => {
                    value.identity_key("identity.binding.record.editor").0
                }
                _ => Key::named("identity.binding.state").scoped(node.slot).0,
            }
        } else {
            match &self.text {
                Binding::Static(text) => {
                    Key::named("identity.binding.static")
                        .scoped(abi::key(text))
                        .0
                }
                Binding::NamedReactive { identity, .. } => {
                    Key::named("identity.binding.named").scoped(identity.0).0
                }
                Binding::RecordMapText { value, .. } => {
                    value.identity_key("identity.binding.record.map.text").0
                }
                Binding::RecordText { value, .. } => {
                    value.identity_key("identity.binding.record.text").0
                }
                Binding::RecordScalar(value) => {
                    value.value.identity_key("identity.binding.record.scalar").0
                }
                Binding::RecordCheckbox { value, .. } => {
                    value.0.identity_key("identity.binding.record.checkbox").0
                }
                Binding::RecordSlider { value, .. } => {
                    value.0.identity_key("identity.binding.record.slider").0
                }
                Binding::RecordProgress { value, .. } => {
                    value.0.identity_key("identity.binding.record.progress").0
                }
                Binding::RecordSelect { value, .. } => {
                    value.0.identity_key("identity.binding.record.select").0
                }
                Binding::Reactive(_) | Binding::Captured(_) | Binding::NamedCaptured { .. } => 0,
                Binding::Scope(id) | Binding::CapturedScope { id, .. } => {
                    Key::named("identity.binding.scope").scoped(*id).0
                }
                Binding::ComputedBoolean(value) => {
                    Key::named("identity.binding.computed.boolean")
                        .scoped(value.key)
                        .0
                }
                Binding::ComputedFloat(value) => {
                    Key::named("identity.binding.computed.float")
                        .scoped(value.key)
                        .0
                }
                Binding::ComputedInteger(value) => {
                    Key::named("identity.binding.computed.integer")
                        .scoped(value.key)
                        .0
                }
                Binding::ComputedText(value) | Binding::ComputedEditor { value, .. } => {
                    Key::named("identity.binding.computed.text")
                        .scoped(value.key)
                        .0
                }
                Binding::MappedFloat { slot, .. } => {
                    Key::named("identity.binding.mapped.float").scoped(*slot).0
                }
                Binding::MappedBoolean { slot, .. } => {
                    Key::named("identity.binding.mapped.boolean")
                        .scoped(*slot)
                        .0
                }
                Binding::MappedText { slot, .. } => {
                    Key::named("identity.binding.mapped.text").scoped(*slot).0
                }
                Binding::MappedList { slot, .. } => {
                    Key::named("identity.binding.mapped.list").scoped(*slot).0
                }
                Binding::MappedTextList { slot, .. } => {
                    Key::named("identity.binding.mapped.text_list")
                        .scoped(*slot)
                        .0
                }
                Binding::MappedInteger { slot, .. } => {
                    Key::named("identity.binding.mapped.integer")
                        .scoped(*slot)
                        .0
                }
            }
        };
        if node.kind == abi::CHECKBOX {
            node.binding = Key::named("identity.binding.checkbox")
                .scoped(node.slot)
                .scoped(node.binding)
                .0;
        }
        if let Some(callback) = &self.event {
            let event = if matches!(node.kind, abi::ENTRY | abi::TEXTAREA) {
                "identity.event.input"
            } else {
                "identity.event.click"
            };
            node.action = Key::named(event).scoped(node.key).0;
            events.push((node.action, callback.clone()));
        }
        output.push(node);
        for child in &self.children {
            child.flatten_events(node.key, output, events)?;
        }
        Ok(())
    }
    fn emit_bindings(
        &self,
        context: *mut std::ffi::c_void,
        emit: abi::EmitBinding,
        owner: u64,
        scope: u64,
    ) -> Result<()> {
        let mut spec = abi::BindingSpec {
            key: self.node.key,
            kind: abi::BIND_STATIC,
            token: 0,
            slot: 0,
            text: abi::Bytes::new(""),
        };
        let record_source = match &self.text {
            Binding::RecordSelect { value, wire } => {
                Some((value.0, wire.as_str(), abi::BIND_RECORD_SELECT, 0))
            }
            Binding::RecordProgress { value, label } => {
                Some((value.0, label.as_str(), abi::BIND_RECORD_PROGRESS, 0))
            }
            Binding::RecordSlider { value, label } => {
                Some((value.0, label.as_str(), abi::BIND_RECORD_SLIDER, 0))
            }
            Binding::RecordCheckbox { value, label } => {
                Some((value.0, label.as_str(), abi::BIND_RECORD_CHECKBOX, 0))
            }
            Binding::RecordText { value, label } => {
                Some((*value, label.as_str(), abi::BIND_RECORD_TEXT, 0))
            }
            Binding::RecordScalar(value) => {
                Some((value.value, "", abi::BIND_RECORD_SCALAR, value.kind))
            }
            _ => None,
        };
        let record_payload = match &record_source {
            Some((value, label, _, _)) => {
                if value.format.is_empty()
                    || value.format.len() > 128
                    || value.format.contains('\n')
                {
                    return Err(Error::InvalidView("invalid record field format"));
                }
                Some(value.wire(label)?)
            }
            _ => None,
        };
        let map_payload = if let Binding::RecordMapText { value, label, .. } = &self.text {
            if value.field.format.is_empty()
                || value.field.format.len() > 128
                || value.field.format.contains('\n')
                || value.key.is_empty()
            {
                return Err(Error::InvalidView("invalid record map text selector"));
            }
            let mut wire = format!(
                "{}\n{}\n{}\n",
                value.field.format,
                value.field.index,
                value.key.len()
            );
            if wire
                .len()
                .saturating_add(value.key.len())
                .saturating_add(label.len())
                > abi::MAX_TEXT_LIST_BYTES
            {
                return Err(Error::InvalidView("record map text binding exceeds limit"));
            }
            wire.push_str(&value.key);
            wire.push_str(label);
            Some(wire)
        } else {
            None
        };
        if let (Binding::RecordMapText { value, keyed, .. }, Some(payload)) =
            (&self.text, &map_payload)
        {
            spec.kind = if *keyed {
                abi::BIND_RECORD_MAP_KEYED_TEXT
            } else {
                abi::BIND_RECORD_MAP_TEXT
            };
            spec.slot = value.field.slot;
            spec.text = abi::Bytes::new(payload);
        } else if let (Some((value, _, kind, token)), Some(payload)) =
            (&record_source, &record_payload)
        {
            spec.kind = *kind;
            spec.token = *token;
            spec.slot = value.slot;
            spec.text = abi::Bytes::new(payload);
        } else if matches!(self.node.kind, abi::ENTRY | abi::TEXTAREA) {
            match &self.text {
                Binding::ComputedEditor { value, label } => {
                    spec.kind = abi::BIND_COMPUTED_EDITOR;
                    spec.slot = value.key;
                    spec.text = abi::Bytes::new(label);
                }
                _ => {
                    spec.kind = abi::BIND_TEXT_SLOT;
                    spec.slot = self.node.slot;
                    if let Binding::Static(label) = &self.text {
                        spec.text = abi::Bytes::new(label);
                    }
                }
            }
        } else {
            match &self.text {
                Binding::RecordText { .. }
                | Binding::RecordMapText { .. }
                | Binding::RecordScalar(_)
                | Binding::RecordCheckbox { .. }
                | Binding::RecordSlider { .. }
                | Binding::RecordProgress { .. }
                | Binding::RecordSelect { .. } => {
                    unreachable!("record bindings emitted above")
                }
                Binding::ComputedEditor { .. } => {
                    return Err(Error::InvalidView(
                        "computed editor binding requires a text editor",
                    ));
                }
                Binding::Static(text) => spec.text = abi::Bytes::new(text),
                Binding::Scope(id) => {
                    spec.kind = abi::BIND_SCOPE;
                    spec.slot = *id;
                }
                Binding::CapturedScope { id, build, source } => {
                    spec.kind = abi::BIND_CAPTURED_SCOPE;
                    spec.slot = *id;
                    spec.token =
                        __events::register_scope(owner, scope, *id, build.clone(), source.clone())?;
                }
                Binding::ComputedBoolean(value) => {
                    spec.kind = abi::BIND_COMPUTED_BOOL;
                    spec.slot = value.key;
                }
                Binding::ComputedFloat(value) => {
                    spec.kind = abi::BIND_COMPUTED_F64;
                    spec.slot = value.key;
                }
                Binding::ComputedInteger(value) => {
                    spec.kind = abi::BIND_COMPUTED_I64;
                    spec.slot = value.key;
                }
                Binding::ComputedText(value) => {
                    spec.kind = abi::BIND_COMPUTED_TEXT;
                    spec.slot = value.key;
                }
                Binding::Captured(compute) | Binding::NamedCaptured { compute, .. } => {
                    spec.kind = abi::BIND_CAPTURED_TEXT;
                    spec.token = __events::register_text(owner, scope, compute.clone())?;
                }
                Binding::Reactive(compute) | Binding::NamedReactive { compute, .. } => {
                    spec.kind = abi::BIND_REACTIVE;
                    spec.token = *compute as usize;
                }
                Binding::MappedFloat { slot, format } => {
                    spec.kind = abi::BIND_MAP_F64;
                    spec.slot = *slot;
                    spec.token = *format as usize;
                }
                Binding::MappedBoolean { slot, format } => {
                    spec.kind = abi::BIND_MAP_BOOL;
                    spec.slot = *slot;
                    spec.token = *format as usize;
                }
                Binding::MappedText { slot, format } => {
                    spec.kind = abi::BIND_MAP_TEXT;
                    spec.slot = *slot;
                    spec.token = *format as usize;
                }
                Binding::MappedList { slot, format } => {
                    spec.kind = abi::BIND_MAP_LIST_I64;
                    spec.slot = *slot;
                    spec.token = *format as usize;
                }
                Binding::MappedTextList { slot, format } => {
                    spec.kind = abi::BIND_MAP_TEXT_LIST;
                    spec.slot = *slot;
                    spec.token = *format as usize;
                }
                Binding::MappedInteger { slot, format } => {
                    spec.kind = abi::BIND_MAP_I64;
                    spec.slot = *slot;
                    spec.token = *format as usize;
                }
            }
            if self.node.kind == abi::CANVAS {
                spec.kind = abi::BIND_CANVAS;
            }
            if self.node.kind == abi::PROGRESS {
                spec.kind = abi::BIND_PROGRESS;
                spec.slot = self.node.slot;
            }
            if self.node.kind == abi::SELECT {
                spec.kind = abi::BIND_SELECT;
                spec.slot = self.node.slot;
            }
            if self.node.kind == abi::SLIDER {
                spec.kind = abi::BIND_SLIDER;
                spec.slot = self.node.slot;
            }
            if self.node.kind == abi::CHECKBOX {
                spec.kind = abi::BIND_CHECKBOX;
                spec.slot = self.node.slot;
            }
        }
        let status = unsafe { emit(context, spec) };
        if status != abi::OK {
            return Err(status.into());
        }
        if let Some(icon) = &self.icon {
            let wire = icon.encode();
            let spec = abi::BindingSpec {
                key: self.node.key,
                kind: abi::BIND_ICON,
                token: 0,
                slot: 0,
                text: abi::Bytes::new(&wire),
            };
            let status = unsafe { emit(context, spec) };
            if status != abi::OK {
                return Err(status.into());
            }
        }
        if let Some(compute) = &self.reactive_style {
            let style = abi::BindingSpec {
                key: self.node.key,
                kind: abi::BIND_STYLE,
                token: __events::register_style(owner, scope, compute.clone())?,
                slot: 0,
                text: abi::Bytes::new(""),
            };
            let status = unsafe { emit(context, style) };
            if status != abi::OK {
                return Err(status.into());
            }
        }
        for child in &self.children {
            child.emit_bindings(context, emit, owner, scope)?;
        }
        Ok(())
    }
    #[cfg(test)]
    fn find(&self, key: u64) -> Option<&View> {
        if self.node.key == key {
            Some(self)
        } else {
            self.children.iter().find_map(|child| child.find(key))
        }
    }
}
#[diagnostic::on_unimplemented(
    message = "unsupported UI child type `{Self}`",
    note = "pass a widget or View; wrap text with Text::new(...), resolve fallible results with ? in a Result-returning builder, and mount collection elements individually with a loop and #[ui], #[node] or Ui::add"
)]
pub trait IntoView {
    fn into_view(self) -> View;
}
impl IntoView for View {
    fn into_view(self) -> View {
        self
    }
}
/// Heterogeneous child expressions using ordinary, rustfmt-compatible Rust syntax.
/// Named local views keep their identity through content edits and sibling insertion.
/// Dynamic collections still provide business keys.
/// ```
/// use blockui::{children, Column, Text, Button, Action};
/// let title = Text::new("Counter");
/// let view = Column::default().children(children![
///     title,
///     Button::new("Increment", Action::remote("increment")),
/// ]);
/// ```
#[macro_export]
macro_rules! children {
    () => { ::std::vec::Vec::<$crate::View>::new() };
    ($($tokens:tt)+) => {{
        let mut children = ::std::vec::Vec::<$crate::View>::new();
        $crate::__children_push!([children] $($tokens)+);
        children
    }};
}
#[doc(hidden)]
#[macro_export]
macro_rules! __children_push {
    ([$sink:ident]) => {};
    ([$sink:ident] $base:ident $(.$field:ident)* , $($rest:tt)*) => {
        $sink.push($crate::IntoView::into_view($base $(.$field)*).__local_identity(::core::stringify!($base $(.$field)*)));
        $crate::__children_push!([$sink] $($rest)*);
    };
    ([$sink:ident] $base:ident $(.$field:ident)*) => {
        $sink.push($crate::IntoView::into_view($base $(.$field)*).__local_identity(::core::stringify!($base $(.$field)*)));
    };
    ([$sink:ident] $view:expr, $($rest:tt)*) => {
        $sink.push($crate::IntoView::into_view($view));
        $crate::__children_push!([$sink] $($rest)*);
    };
    ([$sink:ident] $view:expr) => { $sink.push($crate::IntoView::into_view($view)); };
}

/// Build collection children from business identities and ordinary Rust views.
/// The namespace separates collections with overlapping IDs. Identity is independent
/// of item order/content; duplicate IDs still fail host view admission.
/// Only each item's root key is assigned; explicitly keyed descendants are unchanged.
///
/// ```
/// use blockui::{keyed_children, Column, Text};
/// let servers = [("alpha", "Online"), ("beta", "Offline")];
/// let view = Column::default().children(keyed_children("servers", servers.iter().map(
///     |(id, status)| (*id, Text::new(*status))
/// )));
/// ```
pub fn keyed_children<K: Into<Key>, V: IntoView>(
    namespace: impl Into<Key>,
    children: impl IntoIterator<Item = (K, V)>,
) -> Vec<View> {
    let namespace = namespace.into();
    children
        .into_iter()
        .map(|(identity, view)| view.into_view().key(namespace.scoped(identity.into().0)))
        .collect()
}

fn node(key: Key, kind: u32, height: f32) -> abi::Node {
    abi::Node {
        key: key.0,
        parent: 0,
        kind,
        binding: 0,
        shortcut: 0,
        disabled: 0,
        read_only: 0,
        button_variant: 0,
        presentation: Default::default(),
        semantics: Default::default(),
        action: 0,
        secondary_action: 0,
        submit_action: 0,
        cancel_action: 0,
        grid_rows: 0,
        grid_columns: 0,
        navigate_action: 0,
        scroll_action: 0,
        viewport_action: 0,
        focus_boundary: 0,
        action_payload: 0,
        slot: 0,
        gap: 0.0,
        padding: 0.0,
        height,
        content_width: 0.0,
        range: Default::default(),
        overlay_position: 0,
        tooltip_anchor: 0,
        tooltip_delay_ms: 0,
    }
}

/// Host-registered actor implementation. Versions are generated from the loaded code.
#[derive(Clone, Copy)]
pub struct Implementation(u64);
impl Implementation {
    pub const fn named(name: &str) -> Self {
        Self(abi::key(name))
    }
}
/// Declarative child instance. Its explicit key is scoped to the owning actor.
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct ChildActor {
    pub key: Key,
    pub implementation: Implementation,
}
impl IntoView for ChildActor {
    fn into_view(self) -> View {
        let mut node = node(self.key, abi::CHILD, 0.0);
        node.action = self.implementation.0;
        View {
            reactive_style: None,
            icon: None,
            explicit_identity: true,
            local_identity: None,
            event: None,
            node,
            text: Binding::default(),
            children: Vec::new(),
        }
    }
}

/// Modal subtree. The host contains focus and routes Escape to the current action.
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Dialog {
    pub key: Key,
    pub label: String,
    pub on_escape: Action,
    pub gap: f32,
    pub padding: f32,
}
impl Dialog {
    pub fn new(label: impl Into<String>, on_escape: Action) -> Self {
        Self {
            key: Key::default(),
            label: label.into(),
            on_escape,
            gap: 8.0,
            padding: 16.0,
        }
    }
    pub fn children(self, children: Vec<View>) -> View {
        let mut view = self.into_view();
        view.children = children;
        view
    }
}
impl IntoView for Dialog {
    fn into_view(self) -> View {
        let identity = if self.key.0 == 0 {
            Key::named("identity.auto.dialog.action").scoped(self.on_escape.id)
        } else {
            self.key
        };
        let mut node = node(identity, abi::DIALOG, 0.0);
        node.action = self.on_escape.id;
        node.gap = self.gap;
        node.padding = self.padding;
        View {
            reactive_style: None,
            icon: None,
            explicit_identity: self.key.0 != 0,
            local_identity: None,
            event: None,
            node,
            text: Binding::Static(self.label),
            children: Vec::new(),
        }
    }
}
/// Empty fixed-height layout space, useful for virtualized scroll content.
/// Heights are logical pixels and are validated at runtime admission.
#[derive(Default)]
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Spacer {
    pub key: Key,
    pub height: f32,
}
impl Spacer {
    pub fn new(height: f32) -> Self {
        Self {
            height,
            ..Self::default()
        }
    }
}
impl IntoView for Spacer {
    fn into_view(self) -> View {
        View {
            reactive_style: None,
            icon: None,
            explicit_identity: self.key.0 != 0,
            local_identity: None,
            event: None,
            node: node(self.key, abi::COLUMN, self.height),
            text: Binding::default(),
            children: Vec::new(),
        }
    }
}
/// A declarative page/focus scope with ordinary column layout. Use stable keys
/// for page identity; nesting selects the nearest scope. State belongs to actors,
/// not this container. The host still controls bounded renderer retention.
#[derive(Default)]
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Outlet {
    pub key: Key,
    pub gap: f32,
    pub padding: f32,
}
impl Outlet {
    pub fn new(key: impl Into<Key>) -> Self {
        Self {
            key: key.into(),
            gap: 12.0,
            padding: 0.0,
        }
    }
    pub fn children(self, children: Vec<View>) -> View {
        let mut view = self.into_view();
        view.children = children;
        view
    }
}
impl IntoView for Outlet {
    fn into_view(self) -> View {
        let mut view = Column {
            key: self.key,
            gap: self.gap,
            padding: self.padding,
        }
        .into_view();
        view.node.focus_boundary = 1;
        view
    }
}
/// A control with a visible caption and the same accessible name.
/// The editor's key supplies field identity, so changing the label does not
/// replace its state binding or native editor. Repeated editors still need
/// distinct editor keys, just as they do without a Field.
/// ```
/// use blockui::{Field, IntoView, State, TextInput};
/// let title = State::<String>::new("document.title");
/// let field = Field { label: "Title".into(), control: TextInput::new(title) };
/// let view = field.into_view();
/// ```
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Field<C> {
    pub label: String,
    pub control: C,
}
/// Controls supporting Field's caption/name contract.
#[diagnostic::on_unimplemented(
    message = "unsupported Field control type `{Self}`",
    note = "pass TextInput, TextArea, Select or Slider directly; keep callback controls typed until Field::new rather than converting them to View first"
)]
pub trait FieldControl: IntoView {
    fn with_field_label(self, label: String) -> View;
}
/// A control and its implementation-owned callback. Keeps the control's type
/// available for composition (for example Field) until it becomes a View.
/// ```
/// use blockui::{Cx, Field, IntoView, State, TextInput};
/// let draft = State::<String>::new("draft");
/// let status = State::<String>::new("status");
/// let input = TextInput::new(draft)
///     .on_input(move |cx, text| status.set(cx, &text))
///     .key("draft.editor");
/// let view = Field::new("Draft", input).into_view();
/// ```
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct EventControl<W> {
    widget: W,
    key: Option<Key>,
    enabled: Option<bool>,
    callback: std::rc::Rc<__events::Callback>,
}
impl<W> EventControl<W> {
    /// Set the control identity while preserving its callback and Field support.
    /// Repeated calls use the last key, as with ordinary control builders.
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }
    /// Override the underlying control's enabled state without losing its
    /// callback or Field support. Repeated calls use the last value.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }
}
impl<W: IntoView> IntoView for EventControl<W> {
    fn into_view(self) -> View {
        event_control_view(
            self.widget.into_view(),
            self.key,
            self.enabled,
            self.callback,
        )
    }
}
impl<W: FieldControl> FieldControl for EventControl<W> {
    fn with_field_label(self, label: String) -> View {
        event_control_view(
            self.widget.with_field_label(label),
            self.key,
            self.enabled,
            self.callback,
        )
    }
}
fn event_control_view(
    mut view: View,
    key: Option<Key>,
    enabled: Option<bool>,
    callback: std::rc::Rc<__events::Callback>,
) -> View {
    view.event = Some(callback);
    if let Some(key) = key {
        view = view.key(key);
    }
    if let Some(enabled) = enabled {
        view.node.disabled = u32::from(!enabled);
    }
    view
}
impl<V> FieldControl for TextInput<V>
where
    Self: IntoView,
{
    fn with_field_label(self, label: String) -> View {
        self.label(label)
    }
}
impl<V> FieldControl for TextArea<V>
where
    Self: IntoView,
{
    fn with_field_label(self, label: String) -> View {
        self.label(label)
    }
}
impl<V> FieldControl for Select<V>
where
    Self: IntoView,
{
    fn with_field_label(mut self, label: String) -> View {
        self.label = label;
        self.into_view()
    }
}
impl<V> FieldControl for Slider<V>
where
    Self: IntoView,
{
    fn with_field_label(self, label: String) -> View {
        self.label(label).into_view()
    }
}
impl<C: FieldControl> Field<C> {
    /// Compose a labeled control using its normal Rust type and stable identity.
    pub fn new(label: impl Into<String>, control: C) -> Self {
        Self {
            label: label.into(),
            control,
        }
    }

    /// Attach static or reactive error feedback below the same control.
    /// Empty error text takes no native layout space; control identity is unchanged.
    /// ```
    /// use blockui::{Field, State, TextInput};
    /// let name = State::<String>::new("name");
    /// let error = State::<String>::new("name.error");
    /// let view = Field { label: "Name".into(), control: TextInput::new(name) }.error(error);
    /// ```
    pub fn error(self, message: impl Into<Binding>) -> View {
        field_error(self.into_view(), message.into())
    }
}
impl<C: FieldControl> IntoView for Field<C> {
    fn into_view(self) -> View {
        let control = self.control.with_field_label(self.label.clone());
        field_view(control, self.label)
    }
}
fn field_error(mut view: View, message: Binding) -> View {
    let identity = Key(view.node.key).scoped(abi::key("error"));
    view.children
        .push(Text::new(message).key(identity).style(ViewStyle {
            font_size: 13.0,
            text_tone: TextTone::Error,
            ..Default::default()
        }));
    view
}
fn field_view(control: View, label: String) -> View {
    let identity = Key::named("identity.auto.field").scoped(control.node.key);
    let caption = Text::new(label)
        .key(identity.scoped(abi::key("caption")))
        .style(ViewStyle {
            font_size: 13.0,
            text_tone: TextTone::Muted,
            ..Default::default()
        });
    Column {
        key: identity,
        gap: 6.0,
        padding: 0.0,
    }
    .children(vec![caption, control])
}

#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Column {
    pub key: Key,
    pub gap: f32,
    pub padding: f32,
}
impl Default for Column {
    fn default() -> Self {
        Self {
            key: Key::default(),
            gap: 12.0,
            padding: 24.0,
        }
    }
}
impl Column {
    pub fn children(self, children: Vec<View>) -> View {
        let mut view = self.into_view();
        view.children = children;
        view
    }
}
impl IntoView for Column {
    fn into_view(self) -> View {
        let mut node = node(self.key, abi::COLUMN, 0.0);
        node.gap = self.gap;
        node.padding = self.padding;
        View {
            reactive_style: None,
            icon: None,
            explicit_identity: self.key.0 != 0,
            local_identity: None,
            event: None,
            node,
            text: Binding::default(),
            children: Vec::new(),
        }
    }
}
/// Horizontal layout. Children evenly share the available width.
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Row {
    pub key: Key,
    pub gap: f32,
    pub padding: f32,
}
impl Default for Row {
    fn default() -> Self {
        Self {
            key: Key::default(),
            gap: 12.0,
            padding: 0.0,
        }
    }
}
impl Row {
    pub fn children(self, children: Vec<View>) -> View {
        let mut view = self.into_view();
        view.children = children;
        view
    }
}
impl IntoView for Row {
    fn into_view(self) -> View {
        let mut node = node(self.key, abi::ROW, 0.0);
        node.gap = self.gap;
        node.padding = self.padding;
        View {
            reactive_style: None,
            icon: None,
            explicit_identity: self.key.0 != 0,
            local_identity: None,
            event: None,
            node,
            text: Binding::default(),
            children: Vec::new(),
        }
    }
}
#[derive(Default)]
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Text {
    pub key: Key,
    pub text: Binding,
}
impl Text {
    /// Reactive text with implementation-local captures, evaluated independently
    /// of the component view. Use a key for repeated component instances.
    pub fn reactive(compute: impl for<'a> Fn(&Cx<'a>) -> Result<String> + 'static) -> Self {
        Self::new(Binding::captured(compute))
    }
    pub fn new(text: impl Into<Binding>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }
}
impl IntoView for Text {
    fn into_view(self) -> View {
        let identity = if self.key.0 == 0 {
            self.text
                .identity()
                .map(|binding| Key::named("identity.auto.text.binding").scoped(binding.0))
                .unwrap_or(self.key)
        } else {
            self.key
        };
        View {
            reactive_style: None,
            icon: None,
            explicit_identity: self.key.0 != 0,
            local_identity: None,
            event: None,
            node: node(identity, abi::LABEL, 30.0),
            text: self.text,
            children: Vec::new(),
        }
    }
}
/// A controlled boolean input. Activation sends the opposite current value to on_change.
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Checkbox<V = State<bool>> {
    pub key: Key,
    pub text: String,
    pub value: V,
    pub on_change: BoolAction,
    pub enabled: bool,
}
/// A typed boolean action or an implementation-owned checkbox closure.
pub trait BooleanControlHandler<W: IntoView> {
    type Output;
    #[doc(hidden)]
    fn action(&self) -> BoolAction;
    #[doc(hidden)]
    fn attach(self, widget: W) -> Self::Output;
}
impl<W: IntoView> BooleanControlHandler<W> for BoolAction {
    type Output = W;
    fn action(&self) -> BoolAction {
        *self
    }
    fn attach(self, widget: W) -> W {
        widget
    }
}
impl<W: IntoView, F> BooleanControlHandler<W> for F
where
    F: for<'cx, 'host> Fn(&'cx Cx<'host>, bool) -> Result<()> + 'static,
{
    type Output = EventControl<W>;
    fn action(&self) -> BoolAction {
        BoolAction::new("identity.internal.control", |_, _| Ok(()))
    }
    fn attach(self, widget: W) -> EventControl<W> {
        EventControl {
            key: None,
            enabled: None,
            widget,
            callback: std::rc::Rc::new(move |cx, value| self(cx, bool::from_wire(value)?)),
        }
    }
}
impl Checkbox {
    /// Create a checkbox with an implementation-local closure. Unlike the
    /// action/closure overload in `new`, this provides argument types directly
    /// to closure inference. The handler owns writes and may reject the turn.
    /// ```
    /// use blockui::*;
    /// let enabled = State::<bool>::new("enabled", false);
    /// let checkbox = Checkbox::with_handler("Enabled", enabled, move |cx, value| enabled.set(cx, value));
    /// ```
    pub fn with_handler(
        text: impl Into<String>,
        value: impl Into<State<bool>>,
        on_change: impl for<'cx, 'host> Fn(&'cx Cx<'host>, bool) -> Result<()> + 'static,
    ) -> EventControl<Self> {
        Self::new(text, value, on_change)
    }
    /// Accept a boolean descriptor or borrowed bound field; never retain its context.
    /// ```
    /// use blockui::{BoolAction, Checkbox, Cx, Result, State};
    /// fn checkbox(cx: &Cx<'_>, action: BoolAction) -> Result<Checkbox> {
    ///     let field = State::<bool>::new("enabled", false).bind(cx)?;
    ///     Ok(Checkbox::new("Enabled", &field, action))
    /// }
    /// ```
    /// ```compile_fail
    /// use blockui::{Action, Checkbox, State};
    /// Checkbox::new("Enabled", State::<bool>::new("enabled", false), Action::new("change", |_, _| Ok(())));
    /// ```
    pub fn new<H: BooleanControlHandler<Self>>(
        text: impl Into<String>,
        value: impl Into<State<bool>>,
        on_change: H,
    ) -> H::Output {
        let value = value.into();
        let action = on_change.action();
        on_change.attach(Self {
            key: Key::default(),
            text: text.into(),
            value,
            on_change: action,
            enabled: true,
        })
    }
}
impl IntoView for Checkbox<State<bool>> {
    fn into_view(self) -> View {
        let identity = if self.key.0 == 0 {
            Key::named("identity.auto.checkbox.state").scoped(self.value.key())
        } else {
            self.key
        };
        let mut node = node(identity, abi::CHECKBOX, 36.0);
        node.slot = self.value.key();
        node.action = self.on_change.0.id;
        node.disabled = u32::from(!self.enabled);
        View {
            reactive_style: None,
            icon: None,
            explicit_identity: self.key.0 != 0,
            local_identity: None,
            event: None,
            node,
            text: Binding::Static(self.text),
            children: Vec::new(),
        }
    }
}
impl Checkbox<RecordBoolean> {
    /// Controlled record checkbox; the current action owns validation and writes.
    /// This example requires the `macros` feature for `#[derive(Record)]`.
    /// ```
    /// # #[cfg(feature = "macros")]
    /// # fn main() {
    /// use blockui::{BoolAction, Checkbox, Record, RecordBoolean, State};
    /// #[derive(Record, Default)]
    /// #[record(name = "checkbox.Settings")]
    /// struct Settings { enabled: bool }
    /// fn control(action: BoolAction) -> Checkbox<RecordBoolean> {
    ///     let settings = State::<Settings>::new("settings");
    ///     Checkbox::field("Enabled", settings.boolean(Settings::enabled_field()), action)
    /// }
    /// # }
    /// # #[cfg(not(feature = "macros"))]
    /// # fn main() {}
    /// ```
    pub fn field<H: BooleanControlHandler<Self>>(
        text: impl Into<String>,
        value: RecordBoolean,
        on_change: H,
    ) -> H::Output {
        let action = on_change.action();
        on_change.attach(Self {
            key: Key::default(),
            text: text.into(),
            value,
            on_change: action,
            enabled: true,
        })
    }
}
impl<V> Checkbox<V> {
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = key.into();
        self
    }
}
impl IntoView for Checkbox<RecordBoolean> {
    fn into_view(self) -> View {
        let identity = if self.key.0 == 0 {
            self.value.0.identity_key("identity.auto.checkbox.field")
        } else {
            self.key
        };
        let mut node = node(identity, abi::CHECKBOX, 36.0);
        node.action = self.on_change.0.id;
        node.disabled = u32::from(!self.enabled);
        View {
            reactive_style: None,
            icon: None,
            explicit_identity: self.key.0 != 0,
            local_identity: None,
            event: None,
            node,
            text: Binding::RecordCheckbox {
                value: self.value,
                label: self.text,
            },
            children: Vec::new(),
        }
    }
}
/// An option's value is its stable business identity, independent of list order.
pub use abi::select::OptionItem as SelectOption;
/// Controlled selection. The host sends an i64 option value to current code.
/// Controlled native selection with keyboard navigation and a retained popup.
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Select<V = State<i64>> {
    pub key: Key,
    pub value: V,
    pub options: Vec<SelectOption>,
    pub on_change: Action,
    pub enabled: bool,
    pub label: String,
}
/// A declared action or an implementation-owned integer-control closure.
pub trait IntegerControlHandler<W: IntoView> {
    type Output;
    #[doc(hidden)]
    fn action(&self) -> Action;
    #[doc(hidden)]
    fn attach(self, widget: W) -> Self::Output;
}
impl<W: IntoView> IntegerControlHandler<W> for Action {
    type Output = W;
    fn action(&self) -> Action {
        *self
    }
    fn attach(self, widget: W) -> W {
        widget
    }
}
impl<W: IntoView, F> IntegerControlHandler<W> for F
where
    F: for<'cx, 'host> Fn(&'cx Cx<'host>, i64) -> Result<()> + 'static,
{
    type Output = EventControl<W>;
    fn action(&self) -> Action {
        Action::new("identity.internal.control", |_, _| Ok(()))
    }
    fn attach(self, widget: W) -> EventControl<W> {
        EventControl {
            key: None,
            enabled: None,
            widget,
            callback: std::rc::Rc::new(self),
        }
    }
}
impl Select {
    /// Create a select with a closure whose context/value types are inferred.
    /// The handler owns writes; captures stay in the current implementation.
    /// ```
    /// use blockui::*;
    /// let choice = State::<i64>::new("choice", 0);
    /// let select = Select::with_handler("Choice", choice,
    ///     vec![SelectOption {value:0,label:"Automatic".into(),enabled:true}],
    ///     move |cx, value| choice.set(cx, value));
    /// ```
    pub fn with_handler(
        label: impl Into<String>,
        value: impl Into<State<i64>>,
        options: Vec<SelectOption>,
        on_change: impl for<'cx, 'host> Fn(&'cx Cx<'host>, i64) -> Result<()> + 'static,
    ) -> EventControl<Self> {
        Self::new(label, value, options, on_change)
    }
    pub fn new<H: IntegerControlHandler<Self>>(
        label: impl Into<String>,
        value: impl Into<State<i64>>,
        options: Vec<SelectOption>,
        on_change: H,
    ) -> H::Output {
        let action = on_change.action();
        on_change.attach(Self {
            key: Key::default(),
            value: value.into(),
            options,
            on_change: action,
            enabled: true,
            label: label.into(),
        })
    }
}
impl IntoView for Select<State<i64>> {
    fn into_view(self) -> View {
        let identity = if self.key.0 == 0 {
            Key::named("identity.auto.select.state").scoped(self.value.key())
        } else {
            self.key
        };
        let mut node = node(identity, abi::SELECT, 36.0);
        node.slot = self.value.key();
        node.action = self.on_change.id;
        node.disabled = u32::from(!self.enabled);
        // Invalid direct struct properties are rejected by host admission.
        let wire = abi::select::encode_control(&self.label, &self.options).unwrap_or_default();
        View {
            reactive_style: None,
            icon: None,
            explicit_identity: self.key.0 != 0,
            local_identity: None,
            event: None,
            node,
            text: Binding::Static(wire),
            children: Vec::new(),
        }
    }
}
impl Select<RecordInteger> {
    /// Controlled record selection. The current action owns validation and writes.
    /// This example requires the `macros` feature for `#[derive(Record)]`.
    /// ```
    /// # #[cfg(feature = "macros")]
    /// # fn main() {
    /// use blockui::{Action, Record, RecordInteger, Select, SelectOption, State};
    /// #[derive(Record, Default)]
    /// #[record(name = "select.Settings")]
    /// struct Settings { theme: i64 }
    /// fn control(action: Action) -> Select<RecordInteger> {
    ///     let settings = State::<Settings>::new("settings");
    ///     Select::field("Theme", settings.integer(Settings::theme_field()),
    ///         vec![SelectOption { value: 0, label: "Automatic".into(), enabled: true }], action)
    /// }
    /// # }
    /// # #[cfg(not(feature = "macros"))]
    /// # fn main() {}
    /// ```
    pub fn field(
        label: impl Into<String>,
        value: RecordInteger,
        options: Vec<SelectOption>,
        on_change: Action,
    ) -> Self {
        Self {
            key: Key::default(),
            value,
            options,
            on_change,
            enabled: true,
            label: label.into(),
        }
    }
}
impl<V> Select<V> {
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = key.into();
        self
    }
}
impl IntoView for Select<RecordInteger> {
    fn into_view(self) -> View {
        let identity = if self.key.0 == 0 {
            self.value.0.identity_key("identity.auto.select.field")
        } else {
            self.key
        };
        let mut node = node(identity, abi::SELECT, 36.0);
        node.action = self.on_change.id;
        node.disabled = u32::from(!self.enabled);
        let wire = abi::select::encode_control(&self.label, &self.options).unwrap_or_default();
        View {
            reactive_style: None,
            icon: None,
            explicit_identity: self.key.0 != 0,
            local_identity: None,
            event: None,
            node,
            text: Binding::RecordSelect {
                value: self.value,
                wire,
            },
            children: Vec::new(),
        }
    }
}
/// Controlled integer slider; changes invoke the current implementation's action.
/// Native scene routing is implemented by the host.
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Slider<V = State<i64>> {
    pub key: Key,
    pub value: V,
    pub range: IntegerRange,
    pub on_change: Action,
    pub enabled: bool,
    pub label: String,
}
impl Slider {
    /// Create a slider with a closure whose context/value types are inferred.
    /// The handler owns writes and may reject the entire edit transaction.
    /// ```
    /// use blockui::*;
    /// let level = State::<i64>::new("level", 0);
    /// let slider = Slider::with_handler(level, IntegerRange {min:0,max:100,step:1},
    ///     move |cx, value| level.set(cx, value));
    /// ```
    pub fn with_handler(
        value: impl Into<State<i64>>,
        range: IntegerRange,
        on_change: impl for<'cx, 'host> Fn(&'cx Cx<'host>, i64) -> Result<()> + 'static,
    ) -> EventControl<Self> {
        Self::new(value, range, on_change)
    }
    pub fn new<H: IntegerControlHandler<Self>>(
        value: impl Into<State<i64>>,
        range: IntegerRange,
        on_change: H,
    ) -> H::Output {
        let action = on_change.action();
        on_change.attach(Self {
            key: Key::default(),
            value: value.into(),
            range,
            on_change: action,
            enabled: true,
            label: String::new(),
        })
    }
}
impl IntoView for Slider<State<i64>> {
    fn into_view(self) -> View {
        let identity = if self.key.0 == 0 {
            Key::named("identity.auto.slider.state").scoped(self.value.key())
        } else {
            self.key
        };
        let mut node = node(identity, abi::SLIDER, 36.0);
        node.slot = self.value.key();
        node.action = self.on_change.id;
        node.disabled = u32::from(!self.enabled);
        node.range = self.range;
        View {
            reactive_style: None,
            icon: None,
            explicit_identity: self.key.0 != 0,
            local_identity: None,
            event: None,
            node,
            text: Binding::Static(self.label),
            children: Vec::new(),
        }
    }
}
impl Slider<RecordInteger> {
    /// Controlled record slider; the current action owns validation and writes.
    /// This example requires the `macros` feature for `#[derive(Record)]`.
    /// ```
    /// # #[cfg(feature = "macros")]
    /// # fn main() {
    /// use blockui::{Action, IntegerRange, Record, RecordInteger, Slider, State};
    /// #[derive(Record, Default)]
    /// #[record(name = "slider.Settings")]
    /// struct Settings { volume: i64 }
    /// fn control(action: Action) -> Slider<RecordInteger> {
    ///     let settings = State::<Settings>::new("settings");
    ///     Slider::field(settings.integer(Settings::volume_field()),
    ///         IntegerRange { min: 0, max: 100, step: 10 }, action).label("Volume")
    /// }
    /// # }
    /// # #[cfg(not(feature = "macros"))]
    /// # fn main() {}
    /// ```
    pub fn field(value: RecordInteger, range: IntegerRange, on_change: Action) -> Self {
        Self {
            key: Key::default(),
            value,
            range,
            on_change,
            enabled: true,
            label: String::new(),
        }
    }
}
impl<V> Slider<V> {
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = key.into();
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}
impl IntoView for Slider<RecordInteger> {
    fn into_view(self) -> View {
        let identity = if self.key.0 == 0 {
            self.value.0.identity_key("identity.auto.slider.field")
        } else {
            self.key
        };
        let mut node = node(identity, abi::SLIDER, 36.0);
        node.action = self.on_change.id;
        node.disabled = u32::from(!self.enabled);
        node.range = self.range;
        View {
            reactive_style: None,
            icon: None,
            explicit_identity: self.key.0 != 0,
            local_identity: None,
            event: None,
            node,
            text: Binding::RecordSlider {
                value: self.value,
                label: self.label,
            },
            children: Vec::new(),
        }
    }
}
/// Determinate progress, controlled by a finite f64 state slot in [0, 1].
/// Progress is display-only and never captures an actor callback.
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Progress<V = State<f64>> {
    pub key: Key,
    pub value: V,
    pub label: String,
}
impl Progress {
    pub fn new(value: impl Into<State<f64>>) -> Self {
        Self {
            key: Key::default(),
            value: value.into(),
            label: String::new(),
        }
    }
}
impl IntoView for Progress<State<f64>> {
    fn into_view(self) -> View {
        let identity = if self.key.0 == 0 {
            Key::named("identity.auto.progress.state").scoped(self.value.key())
        } else {
            self.key
        };
        let mut node = node(identity, abi::PROGRESS, 16.0);
        node.slot = self.value.key();
        View {
            reactive_style: None,
            icon: None,
            explicit_identity: self.key.0 != 0,
            local_identity: None,
            event: None,
            node,
            text: Binding::Static(self.label),
            children: Vec::new(),
        }
    }
}
impl Progress<RecordFloat> {
    /// A read-only record fraction. The host rejects values outside finite [0, 1].
    /// This example requires the `macros` feature for `#[derive(Record)]`.
    /// ```
    /// # #[cfg(feature = "macros")]
    /// # fn main() {
    /// use blockui::{Progress, Record, State};
    /// #[derive(Record, Default)]
    /// #[record(name = "progress.Transfer")]
    /// struct Transfer { fraction: f64 }
    /// let transfer = State::<Transfer>::new("transfer");
    /// let progress = Progress::field(transfer.float(Transfer::fraction_field())).label("Download");
    /// # }
    /// # #[cfg(not(feature = "macros"))]
    /// # fn main() {}
    /// ```
    pub fn field(value: RecordFloat) -> Self {
        Self {
            key: Key::default(),
            value,
            label: String::new(),
        }
    }
}
impl<V> Progress<V> {
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = key.into();
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}
impl IntoView for Progress<RecordFloat> {
    fn into_view(self) -> View {
        let identity = if self.key.0 == 0 {
            self.value.0.identity_key("identity.auto.progress.field")
        } else {
            self.key
        };
        View {
            reactive_style: None,
            icon: None,
            explicit_identity: self.key.0 != 0,
            local_identity: None,
            event: None,
            node: node(identity, abi::PROGRESS, 16.0),
            text: Binding::RecordProgress {
                value: self.value,
                label: self.label,
            },
            children: Vec::new(),
        }
    }
}
/// Data-only native canvas. Scene and actions are admitted by the host.
/// Native routing/drawing support is supplied by the stable executable.
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Canvas {
    pub key: Key,
    pub scene: CanvasScene,
    pub height: f32,
    pub on_primary: Option<PointAction>,
    pub on_secondary: Option<PointAction>,
}
impl Canvas {
    pub fn new(scene: CanvasScene) -> Self {
        Self {
            key: Key::default(),
            scene,
            height: 240.0,
            on_primary: None,
            on_secondary: None,
        }
    }
}
impl IntoView for Canvas {
    fn into_view(self) -> View {
        let identity = if self.key.0 == 0 {
            if let Some(action) = self.on_primary.or(self.on_secondary) {
                Key::named("identity.auto.canvas.action").scoped(action.spec().id)
            } else {
                Key::named("identity.auto.canvas.label").scoped(abi::key(&self.scene.label))
            }
        } else {
            self.key
        };
        let mut node = node(identity, abi::CANVAS, self.height);
        node.action = self.on_primary.map_or(0, |action| action.spec().id);
        node.secondary_action = self.on_secondary.map_or(0, |action| action.spec().id);
        View {
            reactive_style: None,
            icon: None,
            explicit_identity: self.key.0 != 0,
            local_identity: None,
            event: None,
            node,
            text: Binding::Static(self.scene.to_wire()),
            children: Vec::new(),
        }
    }
}
/// Shared action hierarchy; state and identity are independent of appearance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Danger,
    Ghost,
}
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Button {
    pub variant: ButtonVariant,
    pub key: Key,
    pub text: Binding,
    pub on_click: Action,
    pub enabled: bool,
}
impl Default for Button {
    fn default() -> Self {
        Self {
            variant: ButtonVariant::Primary,
            key: Key::default(),
            text: Binding::default(),
            on_click: Action::default(),
            enabled: true,
        }
    }
}
impl Button {
    pub fn leading_icon(self, icon: impl Into<Icon>) -> View {
        self.into_view().leading_icon(icon)
    }
    pub fn label(text: impl Into<Binding>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }
    /// Register an ordinary Rust closure in the current UI implementation.
    /// Capture data-only state handles; the fresh context is valid for this call.
    pub fn on_click(
        self,
        callback: impl for<'cx, 'host> Fn(&'cx Cx<'host>) -> Result<()> + 'static,
    ) -> View {
        let mut view = self.into_view();
        view.event = Some(std::rc::Rc::new(move |cx, _| callback(cx)));
        view
    }

    /// Create a primary button with a static or reactive label and a typed action.
    /// The action provides default identity; repeated buttons for the same action
    /// need explicit keys. No application closure is retained by the renderer.
    ///
    /// ```
    /// use blockui::{Action, Button, IntoView, State};
    /// let save = Action::new("document.save", |_, _| Ok(()));
    /// let button = Button::new("Save", save);
    /// let reactive = Button::new(State::<String>::new("save.label"), save);
    /// let _ = (button.into_view(), reactive.into_view());
    /// ```
    pub fn new(text: impl Into<Binding>, on_click: Action) -> Self {
        Self {
            text: text.into(),
            on_click,
            ..Self::default()
        }
    }

    /// Bind a typed data argument to a current-code action. No closure/context is retained.
    /// Integer, boolean and float values use the same types as ActorRef::tell.
    /// Give an explicit key when changing this argument must retain widget identity.
    ///
    /// Register these descriptors in the actor's action table for dispatch.
    /// A key overrides the payload-derived identity when the argument changes.
    /// ```
    /// use blockui::{Action, BoolAction, Button, FloatAction};
    /// let select = Action::new("row.select", |_, _| Ok(()));
    /// let enabled = BoolAction::new("settings.enabled", |_, _| Ok(()));
    /// let amount = FloatAction::new("settings.amount", |_, _| Ok(()));
    /// let _buttons = [
    ///     Button::label("Select row").on_click_with(select, 42_i64).key("row.42"),
    ///     Button::label("Enable").on_click_with(enabled, true),
    ///     Button::label("Half").on_click_with(amount, 0.5_f64),
    /// ];
    /// ```
    /// ```compile_fail
    /// use blockui::{BoolAction, Button};
    /// let action = BoolAction::new("flag", |_, _| Ok(()));
    /// let _ = Button::default().on_click_with(action, 0.5);
    /// ```
    pub fn on_click_with<A: ActionToken>(self, action: A, value: A::Payload) -> View {
        let (id, payload) = action.encode(value);
        let mut view = self.into_view();
        view.node.action = id;
        view.node.action_payload = payload;
        if !view.explicit_identity {
            view.node.key = Key::named("identity.auto.button.payload")
                .scoped(id)
                .scoped(payload as u64)
                .0;
        }
        view
    }
    /// Declare Control+letter (Command+letter on macOS), refreshed on code reload.
    /// Admission rejects non-ASCII letters and duplicate shortcuts in one actor.
    pub fn primary_shortcut(self, letter: char) -> View {
        let mut view = self.into_view();
        view.node.shortcut = letter.to_ascii_lowercase() as u32;
        view
    }
    /// Button text is a single Text child, preserving ordinary Rust tree syntax.
    pub fn children(self, children: Vec<View>) -> View {
        let mut view = self.into_view();
        view.children = children;
        view.finish_children();
        view
    }
}
impl IntoView for Button {
    fn into_view(self) -> View {
        let identity = if self.key.0 == 0 && self.on_click.id != 0 {
            Key::named("identity.auto.button.action").scoped(self.on_click.id)
        } else {
            self.key
        };
        let mut node = node(identity, abi::BUTTON, 32.0);
        node.button_variant = match self.variant {
            ButtonVariant::Primary => 0,
            ButtonVariant::Secondary => 1,
            ButtonVariant::Danger => 2,
            ButtonVariant::Ghost => 3,
        };
        node.action = self.on_click.id;
        node.disabled = u32::from(!self.enabled);
        View {
            reactive_style: None,
            icon: None,
            explicit_identity: self.key.0 != 0,
            local_identity: None,
            event: None,
            node,
            text: self.text,
            children: Vec::new(),
        }
    }
}
// Keep label attachment shared across editor source and label argument types.
fn editor_label(mut view: View, label: String) -> View {
    match &mut view.text {
        Binding::ComputedEditor { label: name, .. }
        | Binding::RecordText { label: name, .. }
        | Binding::RecordMapText { label: name, .. } => *name = label,
        _ => view.text = Binding::Static(label),
    }
    view
}

#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct TextInput<V = State<String>> {
    pub key: Key,
    pub value: V,
    pub read_only: bool,
    /// Current implementation handler, invoked atomically with a user edit.
    pub on_change: Option<TextAction>,
    pub enabled: bool,
    pub on_submit: Option<Action>,
    pub on_cancel: Option<Action>,
}
impl<V> TextInput<V> {
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = key.into();
        self
    }
    pub fn on_submit(mut self, action: Action) -> Self {
        self.on_submit = Some(action);
        self
    }
    pub fn on_cancel(mut self, action: Action) -> Self {
        self.on_cancel = Some(action);
        self
    }
    pub fn on_change(mut self, action: impl RecordTextAction<V>) -> Self {
        self.on_change = Some(action.into_record_action());
        self
    }
    /// Set the accessible name separately from the editor's durable text value.
    /// This changes semantic metadata without painting a visible caption.
    /// ```
    /// use blockui::{State, TextInput};
    /// let notes = State::<String>::new("notes");
    /// let input = TextInput::new(notes).label("Notes");
    /// ```
    pub fn label(self, label: impl Into<String>) -> View
    where
        Self: IntoView,
    {
        editor_label(self.into_view(), label.into())
    }
}
impl TextInput<State<String>> {
    /// Observe a user edit after the state slot has been updated, in the same
    /// transaction. Returning an error rolls back both the edit and callback writes.
    /// Capture data-only handles; the implementation owns the closure until reload.
    pub fn on_input(
        self,
        callback: impl for<'cx, 'host> Fn(&'cx Cx<'host>, String) -> Result<()> + 'static,
    ) -> EventControl<Self> {
        EventControl {
            key: None,
            enabled: None,
            widget: self,
            callback: std::rc::Rc::new(move |cx, _| callback(cx, cx.message_text()?)),
        }
    }
    /// Identity defaults to the bound state slot; repeated views of that same slot
    /// need distinct #[node] names or explicit keys. No source location or sibling position is used.
    /// Accept a durable descriptor or a borrowed turn-bound field. Only the slot
    /// identity is copied, so the resulting widget has no context lifetime.
    /// ```
    /// use blockui::{Cx, Result, State, TextInput, TextArea};
    /// fn editors(cx: &Cx<'_>) -> Result<(TextInput, TextArea)> {
    ///     let field = State::<String>::new("draft").bind(cx)?;
    ///     Ok((TextInput::new(&field), TextArea::new(&field)))
    /// }
    /// ```
    /// ```compile_fail
    /// use blockui::{State, TextInput};
    /// TextInput::new(State::<i64>::new("count", 0));
    /// ```
    pub fn new(value: impl Into<State<String>>) -> Self {
        let value = value.into();
        Self {
            key: Key::default(),
            value,
            read_only: false,
            enabled: true,
            on_change: None,
            on_submit: None,
            on_cancel: None,
        }
    }
}
impl TextInput<Computed<String>> {
    /// Read the model projection and send edits to the current text action.
    pub fn computed(value: Computed<String>, on_change: TextAction) -> Self {
        Self {
            key: Key::default(),
            value,
            read_only: false,
            enabled: true,
            on_change: Some(on_change),
            on_submit: None,
            on_cancel: None,
        }
    }
    fn into_editor(self) -> View {
        let mut view = textarea_view(
            self.key,
            0,
            Binding::ComputedEditor {
                value: self.value,
                label: String::new(),
            },
            self.read_only,
            44.0,
            self.on_change,
            self.enabled,
        );
        if self.key.0 == 0 {
            view.node.key = Key::named("identity.auto.input.computed")
                .scoped(self.value.key)
                .0;
        }
        view.node.kind = abi::ENTRY;
        view.node.submit_action = self.on_submit.map_or(0, |action| action.id);
        view.node.cancel_action = self.on_cancel.map_or(0, |action| action.id);
        view
    }
}
impl IntoView for TextInput<Computed<String>> {
    fn into_view(self) -> View {
        self.into_editor()
    }
}
impl TextInput<RecordText> {
    /// Read the model projection and send edits to the current text action.
    pub fn field<T: RecordTextSource>(
        value: T,
        on_change: impl RecordTextAction<T>,
    ) -> TextInput<T> {
        TextInput {
            key: Key::default(),
            value,
            read_only: false,
            enabled: true,
            on_change: Some(on_change.into_record_action()),
            on_submit: None,
            on_cancel: None,
        }
    }
    fn into_editor(self) -> View {
        let mut view = textarea_view(
            self.key,
            0,
            Binding::RecordText {
                value: self.value,
                label: String::new(),
            },
            self.read_only,
            44.0,
            self.on_change,
            self.enabled,
        );
        if self.key.0 == 0 {
            view.node.key = self.value.identity_key("identity.auto.input.field").0;
        }
        view.node.kind = abi::ENTRY;
        view.node.submit_action = self.on_submit.map_or(0, |action| action.id);
        view.node.cancel_action = self.on_cancel.map_or(0, |action| action.id);
        view
    }
}
impl IntoView for TextInput<RecordText> {
    fn into_view(self) -> View {
        self.into_editor()
    }
}
impl IntoView for TextInput<RecordListText> {
    fn into_view(self) -> View {
        TextInput {
            key: self.key,
            value: self.value.0,
            read_only: self.read_only,
            enabled: self.enabled,
            on_change: self.on_change,
            on_submit: self.on_submit,
            on_cancel: self.on_cancel,
        }
        .into_view()
    }
}
impl IntoView for TextInput<RecordMapText> {
    fn into_view(self) -> View {
        let identity = self.value.identity_key("identity.auto.input.map.field");
        let mut binding = Binding::from(self.value);
        if let Binding::RecordMapText { keyed, .. } = &mut binding {
            *keyed = self
                .on_change
                .is_some_and(|action| matches!(action.0.handler, Handler::KeyedText(_)));
        }
        let mut view = textarea_view(
            self.key,
            0,
            binding,
            self.read_only,
            44.0,
            self.on_change,
            self.enabled,
        );
        if self.key.0 == 0 {
            view.node.key = identity.0;
        }
        view.node.kind = abi::ENTRY;
        view.node.submit_action = self.on_submit.map_or(0, |action| action.id);
        view.node.cancel_action = self.on_cancel.map_or(0, |action| action.id);
        view
    }
}
impl IntoView for TextInput<State<String>> {
    fn into_view(self) -> View {
        let mut view = textarea_view(
            self.key,
            self.value.key(),
            Binding::default(),
            self.read_only,
            44.0,
            self.on_change,
            self.enabled,
        );
        if self.key.0 == 0 {
            view.node.key = Key::named("identity.auto.input.state")
                .scoped(self.value.key())
                .0;
        }
        view.node.kind = abi::ENTRY;
        view.node.submit_action = self.on_submit.map_or(0, |action| action.id);
        view.node.cancel_action = self.on_cancel.map_or(0, |action| action.id);
        view
    }
}
/// Native multiline plain-text editor. State owns text; the renderer owns selection/scroll.
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct TextArea<V = State<String>> {
    pub key: Key,
    pub value: V,
    pub read_only: bool,
    pub height: f32,
    /// Current implementation handler, invoked atomically with a user edit.
    pub on_change: Option<TextAction>,
    pub enabled: bool,
}
impl<V> TextArea<V> {
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = key.into();
        self
    }
    pub fn on_change(mut self, action: impl RecordTextAction<V>) -> Self {
        self.on_change = Some(action.into_record_action());
        self
    }
    /// Set the accessible name separately from the editor value.
    pub fn label(self, label: impl Into<String>) -> View
    where
        Self: IntoView,
    {
        editor_label(self.into_view(), label.into())
    }
}
impl TextArea<State<String>> {
    /// Observe a user edit atomically with updating the retained state slot.
    pub fn on_input(
        self,
        callback: impl for<'cx, 'host> Fn(&'cx Cx<'host>, String) -> Result<()> + 'static,
    ) -> EventControl<Self> {
        EventControl {
            key: None,
            enabled: None,
            widget: self,
            callback: std::rc::Rc::new(move |cx, _| callback(cx, cx.message_text()?)),
        }
    }
    pub fn new(value: impl Into<State<String>>) -> Self {
        Self {
            key: Key::default(),
            value: value.into(),
            read_only: false,
            enabled: true,
            on_change: None,
            height: 140.0,
        }
    }
}
impl TextArea<Computed<String>> {
    /// Read a registered computation and send edits to the current text action.
    /// The action owns normalization/validation and writes the application model.
    /// No duplicate durable text slot or captured context is required.
    pub fn computed(value: Computed<String>, on_change: TextAction) -> Self {
        Self {
            key: Key::default(),
            value,
            read_only: false,
            enabled: true,
            on_change: Some(on_change),
            height: 140.0,
        }
    }
}
impl TextArea<RecordText> {
    /// Read a record field and send edits to the current text action.
    /// The action owns normalization/validation and writes the application model.
    /// No duplicate durable text slot or captured context is required.
    pub fn field<T: RecordTextSource>(
        value: T,
        on_change: impl RecordTextAction<T>,
    ) -> TextArea<T> {
        TextArea {
            key: Key::default(),
            value,
            read_only: false,
            enabled: true,
            on_change: Some(on_change.into_record_action()),
            height: 140.0,
        }
    }
}
fn textarea_view(
    key: Key,
    slot: u64,
    text: Binding,
    read_only: bool,
    height: f32,
    on_change: Option<TextAction>,
    enabled: bool,
) -> View {
    let identity = if key.0 != 0 {
        key
    } else {
        match &text {
            Binding::ComputedEditor { value, .. } => {
                Key::named("identity.auto.textarea.computed").scoped(value.key)
            }
            Binding::RecordText { value, .. } => value.identity_key("identity.auto.textarea.field"),
            Binding::RecordMapText { value, .. } => {
                value.identity_key("identity.auto.textarea.map.field")
            }
            _ => Key::named("identity.auto.textarea.state").scoped(slot),
        }
    };
    let mut node = node(identity, abi::TEXTAREA, height);
    node.read_only = u32::from(read_only);
    node.disabled = u32::from(!enabled);
    node.slot = slot;
    node.action = on_change.map_or(0, |action| action.spec().id);
    View {
        event: None,
        reactive_style: None,
        icon: None,
        explicit_identity: key.0 != 0,
        local_identity: None,
        node,
        text,
        children: Vec::new(),
    }
}
impl IntoView for TextArea<State<String>> {
    fn into_view(self) -> View {
        textarea_view(
            self.key,
            self.value.key(),
            Binding::default(),
            self.read_only,
            self.height,
            self.on_change,
            self.enabled,
        )
    }
}
impl IntoView for TextArea<Computed<String>> {
    fn into_view(self) -> View {
        textarea_view(
            self.key,
            0,
            Binding::ComputedEditor {
                value: self.value,
                label: String::new(),
            },
            self.read_only,
            self.height,
            self.on_change,
            self.enabled,
        )
    }
}
impl IntoView for TextArea<RecordText> {
    fn into_view(self) -> View {
        textarea_view(
            self.key,
            0,
            Binding::RecordText {
                value: self.value,
                label: String::new(),
            },
            self.read_only,
            self.height,
            self.on_change,
            self.enabled,
        )
    }
}
impl IntoView for TextArea<RecordListText> {
    fn into_view(self) -> View {
        TextArea {
            key: self.key,
            value: self.value.0,
            read_only: self.read_only,
            height: self.height,
            enabled: self.enabled,
            on_change: self.on_change,
        }
        .into_view()
    }
}
impl IntoView for TextArea<RecordMapText> {
    fn into_view(self) -> View {
        let mut binding = Binding::from(self.value);
        if let Binding::RecordMapText { keyed, .. } = &mut binding {
            *keyed = self
                .on_change
                .is_some_and(|action| matches!(action.0.handler, Handler::KeyedText(_)));
        }
        textarea_view(
            self.key,
            0,
            binding,
            self.read_only,
            self.height,
            self.on_change,
            self.enabled,
        )
    }
}
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct ScrollView {
    /// Actual logical scroll-port width/height after layout.
    pub on_viewport: Option<PointAction>,
    pub on_scroll: Option<PointAction>,
    pub grid: Option<(GridExtent, Action)>,
    /// Zero follows the viewport; positive values enable wide content.
    pub content_width: f32,
    pub key: Key,
    pub text: Binding,
    pub height: f32,
}
impl Default for ScrollView {
    fn default() -> Self {
        Self {
            key: Key::default(),
            text: Binding::default(),
            height: 200.0,
            content_width: 0.0,
            grid: None,
            on_scroll: None,
            on_viewport: None,
        }
    }
}
impl ScrollView {
    pub fn on_viewport(mut self, action: PointAction) -> Self {
        self.on_viewport = Some(action);
        self
    }
    pub fn on_scroll(mut self, action: PointAction) -> Self {
        self.on_scroll = Some(action);
        self
    }
    /// Enable arrow/Home/End navigation for button cells carrying distinct
    /// row-major `on_click_with` indices. The action receives the destination
    /// index. Focus and scrolling belong to the host, not this action token.
    pub fn grid(mut self, extent: GridExtent, navigate: Action) -> Self {
        self.grid = Some((extent, navigate));
        self
    }
    /// Scroll ordinary child views in both dimensions. Optional text precedes children.
    pub fn children(self, children: Vec<View>) -> View {
        let mut view = self.into_view();
        view.children = children;
        view
    }
}
impl IntoView for ScrollView {
    fn into_view(self) -> View {
        let identity = if self.key.0 == 0 {
            self.text
                .identity()
                .map(|binding| Key::named("identity.auto.scroll.binding").scoped(binding.0))
                .unwrap_or(self.key)
        } else {
            self.key
        };
        View {
            reactive_style: None,
            icon: None,
            explicit_identity: self.key.0 != 0,
            local_identity: None,
            event: None,
            node: abi::Node {
                grid_rows: self
                    .grid
                    .map_or(0, |(extent, _)| extent.rows().min(u32::MAX as usize) as u32),
                grid_columns: self.grid.map_or(0, |(extent, _)| {
                    extent.columns().min(u32::MAX as usize) as u32
                }),
                navigate_action: self.grid.map_or(0, |(_, action)| action.id),
                scroll_action: self.on_scroll.map_or(0, |action| action.spec().id),
                viewport_action: self.on_viewport.map_or(0, |action| action.spec().id),
                content_width: self.content_width,
                ..node(identity, abi::SCROLL, self.height)
            },
            text: self.text,
            children: Vec::new(),
        }
    }
}

// Inherent methods keep key assignment available without a builder trait import.
macro_rules! widget_key_builders {
    ($($widget:ty),+ $(,)?) => { $(
        impl $widget {
            /// Set an explicit stable identity, overriding any binding-derived default.
            /// Keys must be unique within an actor; scope business IDs for separate lists.
            pub fn key(mut self, key: impl Into<Key>) -> Self {
                self.key = key.into();
                self
            }
        }
    )+ };
}
widget_key_builders!(
    Outlet,
    Spacer,
    Column,
    Row,
    Dialog,
    Text,
    Button,
    Canvas,
    ScrollView,
    ChildActor,
    Card,
    ActionGroup
);

/// An application implements behavior and view; the host owns all durable values.
/// Internal adapter for ordinary Rust view return types; no syntax rewriting.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "unsupported actor/scope view return type `{Self}`",
    note = "return View or blockui::Result<View>; type aliases must resolve to one of these types with blockui::Error as the error type"
)]
pub trait __ViewOutput {
    fn into_view_result(self) -> Result<View>;
}
impl __ViewOutput for View {
    fn into_view_result(self) -> Result<View> {
        Ok(self)
    }
}
impl __ViewOutput for Result<View> {
    fn into_view_result(self) -> Result<View> {
        self
    }
}
#[doc(hidden)]
pub fn __view_result(value: impl __ViewOutput) -> Result<View> {
    value.into_view_result()
}

#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "unsupported component view return type `{Self}`",
    note = "return View or blockui::Result<View>; type aliases must resolve to one of these types with blockui::Error as the error type"
)]
pub trait __ComponentOutput: Sized {
    fn __instance(self, path: InstancePath, root: Key) -> Self;
}
impl __ComponentOutput for View {
    fn __instance(mut self, path: InstancePath, root: Key) -> Self {
        if path.len == 0 {
            return self;
        }
        if self.node.key == 0 {
            self.node.key = root.0;
        }
        fn scope(view: &mut View, path: InstancePath) {
            if let Binding::CapturedScope { id, .. } = &mut view.text {
                *id = path.resolve(*id);
            }
            if view.node.key != 0 && !matches!(view.text, Binding::Scope(_)) {
                view.node.key = path.resolve(view.node.key);
                for related in [
                    &mut view.node.semantics.controls,
                    &mut view.node.semantics.labelled_by,
                ] {
                    if *related != 0 {
                        *related = path.resolve(*related);
                    }
                }
            }
            for child in &mut view.children {
                scope(child, path);
            }
        }
        scope(&mut self, path);
        self
    }
}
impl __ComponentOutput for Result<View> {
    fn __instance(self, path: InstancePath, root: Key) -> Self {
        self.map(|view| __ComponentOutput::__instance(view, path, root))
    }
}
#[doc(hidden)]
pub fn __component_view<T: __ComponentOutput>(view: T, path: InstancePath, root: Key) -> T {
    view.__instance(path, root)
}

/// A view projection from an already bound, synchronous state receiver.
/// Keep the default `Render` when projection cannot fail. Use
/// `Render<Result<View>>` for ordinary fallible Rust logic; `?` propagates to
/// the enclosing actor transaction and no partial view is published.
pub trait Render<Output = View> {
    fn render(&self) -> Output;
}

/// Validate declared fields once, then call the renderer. Implement Render for
/// an infallible View, or Render<Result<View>> to use ordinary Rust `?` for
/// recoverable decoding/application errors. Neither renderer is rewritten.
pub fn render_bound<'cx, 'host, A: BindState, Output>(actor: &A, cx: &'cx Cx<'host>) -> Result<View>
where
    A::Bound<'cx, 'host>: Render<Output>,
    Output: __ViewOutput,
{
    __view_result(actor.bind(cx)?.render())
}

/// Named structural projection supplied by the current implementation.
/// Mount with `IntoView`/`#[node]`; register this descriptor in `Actor::SCOPES`.
/// Leaf roots receive a zero-padding, zero-gap Column wrapper; mount each identity once.
/// Structural reads invalidate this scope; root view reads invalidate the whole tree.
/// Root text and a reactive style on the returned container update independently
/// and survives cached/nested scope refreshes. Apply it to that real root rather
/// than the mount descriptor.
///
/// ```
/// use blockui::*;
/// const WIDE: State<bool> = State::<bool>::new("panel.wide", false);
/// const PANEL: Scope = Scope::new("panel", |_| {
///     Ok(Column::default().children(vec![Text::new("Content").into_view()])
///         .style_reactive(|cx| Ok(ViewStyle {
///             width: if WIDE.get(cx)? { 280.0 } else { 180.0 },
///             surface: Surface::Panel,
///             ..default()
///         })))
/// });
/// ```
/// Builders are noncapturing Rust function pointers; no closure state is retained.
/// `actor_impl` can generate the registry and descriptor accessor from `#[scope]`:
/// ```
/// #[cfg(feature = "macros")]
/// fn main() {
///     use blockui::*;
///     #[actor(name = "counter")]
///     struct Counter {
///         count: State<i64>,
///     }
///     #[actor_impl]
///     impl Counter {
///         #[scope(name = "counter.panel")]
///         fn panel(&self, cx: &Cx<'_>) -> Result<View> {
///             let count = self.count.get(cx)?;
///             Ok({
///                 let ui = ::blockui::Ui::new();
///                 {
///                     let _scope_0 = ui.scope(Column::default());
///                     ui.add(Text::new(count.to_string()));
///                 }
///                 ui.finish()
///             })
///         }
///         fn view(&self, _: &Cx<'_>) -> View {
///             {
///                 let ui = ::blockui::Ui::new();
///                 {
///                     let _scope_0 = ui.scope(Column::default());
///                     ui.add(Self::panel_scope());
///                 }
///                 ui.finish()
///             }
///         }
///     }
///     assert_eq!(Counter::SCOPES.len(), 1);
/// }
/// #[cfg(not(feature = "macros"))]
/// fn main() {}
/// ```
/// Without `name`, identity defaults to `Type.method`; supply a name to preserve
/// identity across a method/type rename. Conditional declarations follow `cfg`.
/// ```compile_fail
/// use blockui::{Scope, Text, IntoView};
/// let title = String::from("captured");
/// Scope::new("panel", move |_| Ok(Text::new(title.clone()).into_view()));
/// ```
/// Ignoring a descriptor does not execute or mount its builder:
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use blockui::*;
/// Scope::new("forgotten.panel", |_| Ok(Text::new("Panel").into_view()));
/// ```
#[must_use = "a scope descriptor must be mounted in a view for its builder to run"]
#[derive(Clone, Copy)]
pub struct Scope {
    id: u64,
    instance: InstancePath,
    build: for<'a> fn(&Cx<'a>) -> Result<View>,
}
impl Scope {
    #[doc(hidden)]
    pub const fn __scoped(mut self, scope: Key) -> Self {
        self.id = scope.scoped(self.id).0;
        self.instance = self.instance.scoped(scope);
        self
    }
    #[doc(hidden)]
    pub const fn __instance(mut self, instance: InstancePath) -> Self {
        self.id = instance.resolve(self.id);
        self.instance = instance;
        self
    }

    /// Define a local structural boundary without an `Actor::SCOPES` entry.
    /// Owned parameters and State handles may be captured; a borrowed Cx may not.
    /// Reads in the builder refresh this subtree, while reactive text/styles
    /// remain independent. Leaf roots receive a zero-padding, zero-gap Column
    /// wrapper; explicit containers and View::empty() are also supported.
    /// Child widget keys are scoped by this identity; captured State handles
    /// keep their existing identities. Use a stable, distinct key per mount.
    ///
    /// ```
    /// use blockui::*;
    /// const OPEN: State<bool> = State::<bool>::new("open", false);
    /// let title = String::from("Details");
    /// let panel = Scope::reactive("details", move |cx| {
    ///     let children = if OPEN.get(cx)? {
    ///         vec![Text::new(title.clone()).key("title").into_view()]
    ///     } else { vec![] };
    ///     Ok(Column::default().children(children))
    /// });
    /// ```
    /// ```compile_fail
    /// use blockui::*;
    /// fn panel(cx: &Cx<'_>) -> View {
    ///     Scope::reactive("panel", move |_| {
    ///         let _ = cx.monotonic_ms()?;
    ///         Ok(Column::default().children(vec![]))
    ///     })
    /// }
    /// ```
    pub fn reactive(
        key: impl Into<Key>,
        build: impl for<'a> Fn(&Cx<'a>) -> Result<View> + 'static,
    ) -> View {
        Self::captured_box_view(key.into(), Box::new(build))
    }

    // Keep the Rc conversion concrete rather than instantiating it per builder.
    // The temporary Box may add allocation/copy work during construction.
    fn captured_box_view(key: Key, build: Box<ScopeCompute>) -> View {
        Self::captured_view(key, std::rc::Rc::from(build))
    }

    // Construct mount metadata once in the SDK rather than once per closure.
    // The erased callback remains implementation-local and is registered only
    // during transactional projection, exactly like the typed entry above.
    fn captured_view(key: Key, build: std::rc::Rc<ScopeCompute>) -> View {
        let mut view = Column::default().children(Vec::new()).key(key);
        view.text = Binding::CapturedScope {
            id: key.0,
            build,
            source: None,
        };
        view
    }

    pub(crate) fn captured_row_view(
        key: Key,
        build: std::rc::Rc<ScopeCompute>,
        source: std::rc::Rc<()>,
    ) -> View {
        let mut view = Self::captured_view(key, build);
        if let Binding::CapturedScope {
            source: identity, ..
        } = &mut view.text
        {
            *identity = Some(source);
        }
        view
    }

    /// A conditional structural boundary. The condition is tracked on every
    /// evaluation; content is built only when it returns true. A false branch
    /// unmounts content and releases its callbacks, retaining actor-owned state.
    /// The boundary's condition/build captures remain owned while it is mounted.
    /// Owned parameters may be captured by either closure. Child keys follow the
    /// same namespace and container-root rules as `Scope::reactive`; style the
    /// returned content root rather than this mount marker.
    ///
    /// ```
    /// use blockui::*;
    /// let open = State::<bool>::new("details.open", false);
    /// let draft = State::<String>::new("details.draft");
    /// let details = Scope::when("details", move |cx| open.get(cx), move |_| {
    ///     Ok(Card::default().children(vec![
    ///         Field::new("Draft", TextInput::new(draft)).into_view(),
    ///     ]))
    /// });
    /// ```
    pub fn when(
        key: impl Into<Key>,
        condition: impl for<'a> Fn(&Cx<'a>) -> Result<bool> + 'static,
        build: impl for<'a> Fn(&Cx<'a>) -> Result<View> + 'static,
    ) -> View {
        Self::reactive(key, move |cx| {
            if condition(cx)? {
                build(cx)
            } else {
                Ok(View::empty())
            }
        })
    }

    pub const fn new(name: &str, build: for<'a> fn(&Cx<'a>) -> Result<View>) -> Self {
        Self {
            id: abi::key(name),
            instance: InstancePath::EMPTY,
            build,
        }
    }
    pub const fn id(self) -> u64 {
        self.id
    }
}

impl IntoView for Scope {
    fn into_view(self) -> View {
        let mut view = Column::default().children(Vec::new());
        view = view.key(Key(self.id));
        view.text = Binding::Scope(self.id);
        view
    }
}

pub trait Actor: 'static {
    const STATE: &'static [StateSpec];
    const ACTIONS: &'static [Action];
    const COMPUTEDS: &'static [ComputedSpec] = &[];
    const SCOPES: &'static [Scope] = &[];
    fn view(cx: &Cx<'_>) -> Result<View>;
    fn migrate(_: &Cx<'_>, _: u64) -> Result<()> {
        Ok(())
    }
}
pub fn default<T: Default>() -> T {
    T::default()
}
pub trait Px {
    fn px(self) -> f32;
}
impl Px for i32 {
    fn px(self) -> f32 {
        self as f32
    }
}
impl Px for f32 {
    fn px(self) -> f32 {
        self
    }
}

fn guard(f: impl FnOnce() -> Result<()>) -> i32 {
    guard_reporting(None, f)
}
fn guard_with_api(api: &HostApi, f: impl FnOnce() -> Result<()>) -> i32 {
    guard_reporting(Some(api), f)
}
fn guard_reporting(api: Option<&HostApi>, f: impl FnOnce() -> Result<()>) -> i32 {
    let text = match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(())) => return abi::OK,
        Ok(Err(error)) => {
            eprintln!("actor callback rejected: {error:?}");
            error.to_string()
        }
        Err(payload) => {
            let reason = payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied())
                .unwrap_or("non-string panic payload");
            format!("actor callback panicked: {reason}")
        }
    };
    if let Some(api) = api {
        let mut end = text.len().min(4096);
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        // The host copies immediately; the owned text and panic payload stay local.
        unsafe { (api.report_error)(api.context, abi::Bytes::new(&text[..end])) };
    }
    abi::ERROR
}
/// Internal ABI adapter; application code only invokes export_actor!.
#[doc(hidden)]
pub struct Export<A>(PhantomData<A>);
impl<A: Actor> Export<A> {
    pub const TABLE: abi::Implementation = abi::Implementation {
        abi_version: abi::ABI_VERSION,
        struct_size: std::mem::size_of::<abi::Implementation>(),
        panic_strategy: if cfg!(panic = "unwind") {
            abi::PANIC_UNWIND
        } else {
            abi::PANIC_ABORT
        },
        // StateSpec is transparent over its private, statically constructed ABI descriptor.
        slots: A::STATE.as_ptr().cast(),
        slot_count: A::STATE.len(),
        events: export::events,
        handle: Self::handle,
        view: Self::view,
        render_binding: export::render_binding,
        migrate: Some(Self::migrate),
        scope_ids: Self::scope_ids,
        scope_view: Self::scope_view,
        computed_specs: Self::computed_specs,
        compute: Self::compute,
    };
    unsafe extern "C" fn scope_ids(context: *mut std::ffi::c_void, emit: abi::EmitScope) -> i32 {
        unsafe { export::scope_ids(A::SCOPES, context, emit) }
    }
    unsafe extern "C" fn scope_view(
        api: *const HostApi,
        id: u64,
        context: *mut std::ffi::c_void,
        emit: abi::Emit,
        emit_binding: abi::EmitBinding,
    ) -> i32 {
        unsafe { export::scope_view(A::SCOPES, A::ACTIONS, api, id, context, emit, emit_binding) }
    }
    unsafe extern "C" fn computed_specs(
        context: *mut std::ffi::c_void,
        emit: abi::EmitComputed,
    ) -> i32 {
        unsafe { export::computed_specs(A::COMPUTEDS, context, emit) }
    }
    unsafe extern "C" fn compute(
        api: *const HostApi,
        key: u64,
        context: *mut std::ffi::c_void,
        write: abi::WriteValue,
    ) -> i32 {
        unsafe { export::compute(A::COMPUTEDS, api, key, context, write) }
    }
    unsafe extern "C" fn handle(api: *const HostApi, id: u64, value: i64) -> i32 {
        unsafe { export::handle(A::ACTIONS, api, id, value) }
    }
    unsafe extern "C" fn view(
        api: *const HostApi,
        context: *mut std::ffi::c_void,
        emit: abi::Emit,
        emit_binding: abi::EmitBinding,
    ) -> i32 {
        unsafe { export::view(A::ACTIONS, A::view, api, context, emit, emit_binding) }
    }
    unsafe extern "C" fn migrate(api: *const HostApi, old_schema: u64) -> i32 {
        unsafe { export::migrate(A::migrate, api, old_schema) }
    }
}

#[macro_export]
macro_rules! export_actor {
    ($actor:ty) => {
        $crate::export_actor!($actor, $crate::abi::ABI_VERSION);
    };
    ($actor:ty, $version:expr) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn actor_entry() -> *const $crate::abi::Implementation {
            static TABLE: $crate::abi::Implementation = $crate::abi::Implementation {
                abi_version: $version,
                ..$crate::Export::<$actor>::TABLE
            };
            &TABLE
        }
    };
}
#[macro_export]
macro_rules! bind {
    (move $($compute:tt)*) => {
        $crate::Binding::captured(move $($compute)*)
    };
    ($compute:path) => {
        $crate::Binding::__path(
            ::core::module_path!(),
            ::core::stringify!($compute),
            $compute,
        )
    };
    ($compute:expr) => {
        $crate::Binding::reactive($compute)
    };
}
#[doc(hidden)]
pub fn __empty_migration(_: &Cx<'_>, _: u64) -> Result<()> {
    Ok(())
}
#[doc(hidden)]
pub fn __empty_scoped_validation(_: &Cx<'_>, _: &[Key]) -> Result<()> {
    Ok(())
}
#[doc(hidden)]
pub fn __empty_record_validation(_: &Cx<'_>, _: u64, _: &[Key]) -> Result<()> {
    Ok(())
}

/// Shared generated migration sequencing; callbacks remain implementation-local.
#[doc(hidden)]
pub fn __migrate_actor(
    cx: &Cx<'_>,
    previous: u64,
    initialize: fn(&Cx<'_>, &[Key]) -> Result<()>,
    validate: fn(&Cx<'_>, u64, &[Key]) -> Result<()>,
    components: fn(&Cx<'_>, u64) -> Result<()>,
    routes: fn(&Cx<'_>, &[Key]) -> Result<()>,
    migration: Option<fn(&Cx<'_>, u64) -> Result<()>>,
) -> Result<()> {
    let scopes: Vec<_> = cx.__instance().scopes().collect();
    initialize(cx, &scopes)?;
    if let Some(migration) = migration {
        migration(cx, previous)?;
    } else {
        validate(cx, previous, &scopes)?;
    }
    components(cx, previous)?;
    routes(cx, &scopes)
}

#[cfg(test)]
mod ui_loop_tests {
    use super::*;
    #[test]
    fn inner_loop_exit_keeps_prefix_and_continues_the_outer_list() {
        let tree = {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                for page in 0..2 {
                    __blockui_tree.add(Text::new("page").key(format!("page.{page}")));
                    for row in 0..3 {
                        __blockui_tree.add(Text::new("row").key(format!("row.{page}.{row}")));
                        if row == 1 {
                            break;
                        }
                    }
                    __blockui_tree.add(Text::new("end").key(format!("end.{page}")));
                }
            }
            __blockui_tree.finish()
        };
        assert_eq!(
            tree.children
                .iter()
                .map(|child| child.node.key)
                .collect::<Vec<_>>(),
            [
                "page.0", "row.0.0", "row.0.1", "end.0", "page.1", "row.1.0", "row.1.1", "end.1"
            ]
            .map(abi::key)
        );
    }
    #[test]
    fn loop_exits_preserve_already_emitted_children_in_branches() {
        let tree = {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                for id in 0..4_u64 {
                    __blockui_tree
                        .add(Text::new(format!("before {id}")).key(format!("before {id}")));
                    if id == 1 {
                        __blockui_tree.add(Text::new("continued").key("continued"));
                        continue;
                    }
                    match id {
                        2 => {
                            __blockui_tree.add(Text::new("broken").key("broken"));
                            break;
                        }
                        _ => {
                            __blockui_tree
                                .add(Text::new(format!("after {id}")).key(format!("after {id}")));
                        }
                    }
                }
                __blockui_tree.add(Text::new("finished").key("finished"));
            }
            __blockui_tree.finish()
        };
        assert_eq!(tree.children.len(), 7);
        assert_eq!(
            tree.children
                .iter()
                .map(|child| child.node.key)
                .collect::<Vec<_>>(),
            [
                "before 0",
                "after 0",
                "before 1",
                "continued",
                "before 2",
                "broken",
                "finished"
            ]
            .map(abi::key)
        );
    }
    #[test]
    fn rust_loop_break_and_continue_filter_children_without_partial_rows() {
        let tree = {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                for id in 0..10_u64 {
                    if id == 6 {
                        break;
                    }
                    if id % 2 == 1 {
                        continue;
                    }
                    __blockui_tree
                        .add(Text::new(format!("row {id}")).key(Key::named("row").scoped(id)));
                }
                __blockui_tree.add(Text::new("after loop"));
            }
            __blockui_tree.finish()
        };
        assert_eq!(tree.children.len(), 4);
        for (node, id) in tree.children[..3].iter().zip([0, 2, 4]) {
            assert_eq!(node.node.key, Key::named("row").scoped(id).0);
        }
    }
    #[test]
    fn ui_loop_branches_can_target_an_authored_outer_label() {
        let mut completed = Vec::new();
        'pages: for page in 0..3 {
            let tree = {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    for id in 0..2_u64 {
                        if page == 1 {
                            continue 'pages;
                        }
                        if page == 2 {
                            break 'pages;
                        }
                        __blockui_tree.add(
                            Text::new(format!("{page}/{id}")).key(Key::named("row").scoped(id)),
                        );
                    }
                }
                __blockui_tree.finish()
            };
            assert_eq!(tree.children.len(), 2);
            completed.push(page);
        }
        assert_eq!(completed, [0]);
    }
    #[test]
    fn button_arguments_preserve_scalar_bits_and_distinguish_presets() {
        let integer = Action::new("integer", |_, _| Ok(()));
        let boolean = BoolAction::new("boolean", |_, _| Ok(()));
        let float = FloatAction::new("float", |_, _| Ok(()));
        for value in [i64::MIN, -42, 0, i64::MAX] {
            assert_eq!(
                Button::default()
                    .on_click_with(integer, value)
                    .node
                    .action_payload,
                value
            );
        }
        for value in [false, true] {
            assert_eq!(
                Button::default()
                    .on_click_with(boolean, value)
                    .node
                    .action_payload,
                i64::from(value)
            );
        }
        for bits in [
            (-0.0_f64).to_bits(),
            0xfff8_0000_0000_1234,
            f64::INFINITY.to_bits(),
        ] {
            assert_eq!(
                Button::default()
                    .on_click_with(float, f64::from_bits(bits))
                    .node
                    .action_payload as u64,
                bits
            );
        }
        let first = Button::default().on_click_with(integer, 1);
        let second = Button::default().on_click_with(integer, 2);
        assert_ne!(first.node.key, second.node.key);
        assert_eq!(first.key("preset").node.key, second.key("preset").node.key);
    }
    #[test]
    fn key_builders_override_defaults_and_reparent_children() {
        let item = Key::named("servers.row").scoped(42);
        assert_eq!(Text::new("server").key(item).into_view().node.key, item.0);
        assert_eq!(
            TextInput::new(State::<String>::new("notes"))
                .key(42_u64)
                .into_view()
                .node
                .key,
            42
        );
        assert_eq!(
            Button::default().key("button").into_view().node.key,
            abi::key("button")
        );
        assert_eq!(ScrollView::default().key(item).into_view().node.key, item.0);
        assert_eq!(
            ChildActor {
                key: Key::default(),
                implementation: Implementation::named("server")
            }
            .key(item)
            .into_view()
            .node
            .key,
            item.0
        );
        let view = Column::default()
            .key("old-root")
            .children(vec![Text::new("server").key(item).into_view()])
            .key("new-root");
        let mut flat = Vec::new();
        view.flatten(0, &mut flat).unwrap();
        assert_eq!(flat[0].key, abi::key("new-root"));
        assert_eq!(flat[1].key, item.0);
        assert_eq!(flat[1].parent, flat[0].key);
    }

    #[test]
    fn row_uses_rust_properties_children_and_parent_scoped_defaults() {
        let view = {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Row {
                    gap: 8.px(),
                    padding: 4.px(),
                    ..default()
                });
                __blockui_tree.add(Text::new("first").key("first"));
                {
                    let _scope_1 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new("nested"));
                };
            }
            __blockui_tree.finish()
        }
        .actor_root();
        let mut nodes = Vec::new();
        view.flatten(0, &mut nodes).unwrap();
        assert_eq!(nodes[0].kind, abi::ROW);
        assert_eq!(nodes[0].gap, 8.0);
        assert_eq!(nodes[0].padding, 4.0);
        assert_eq!(nodes[1].parent, nodes[0].key);
        assert_eq!(nodes[2].parent, nodes[0].key);
        assert_eq!(nodes[3].parent, nodes[2].key);
        assert!(nodes.iter().all(|node| node.key != 0));
    }

    #[test]
    fn button_default_is_enabled_and_disabled_preserves_key() {
        let enabled = Button::default().key("save").into_view();
        let disabled = Button {
            enabled: false,
            ..default()
        }
        .key("save")
        .into_view();
        assert_eq!(enabled.node.disabled, 0);
        assert_eq!(disabled.node.disabled, 1);
        assert_eq!(enabled.node.key, disabled.node.key);
    }

    #[test]
    fn declared_shortcut_copies_letter_and_keeps_semantic_action_identity() {
        let normal = Button::default().key("save").primary_shortcut('S');
        assert_eq!(normal.node.shortcut, 's' as u32);
        assert_eq!(normal.node.key, abi::key("save"));
        // Invalid declarations remain visible to host admission, never silently truncate.
        assert_eq!(
            Button::default().primary_shortcut('界').node.shortcut,
            '界' as u32
        );
    }

    #[test]
    fn unique_static_children_ignore_content_style_and_sibling_position() {
        let build = |changed: bool| {
            let mut children = vec![Column::default().children(vec![
                Text::new(if changed { "edited" } else { "original" }).into_view(),
            ])];
            if changed {
                children.insert(0, Text::new("new sibling").key("inserted").into_view());
            }
            Column::default().children(children).actor_root()
        };
        let old = build(false);
        let new = build(true);
        let old_column = &old.children[0];
        let new_column = &new.children[1];
        assert_ne!(old_column.node.key, 0);
        assert_eq!(old_column.node.key, new_column.node.key);
        assert_eq!(
            old_column.children[0].node.key,
            new_column.children[0].node.key
        );
        assert!(new.find(old_column.children[0].node.key).is_some());
        let ambiguous = Column::default()
            .children(vec![
                Text::new("one").into_view(),
                Text::new("two").into_view(),
            ])
            .actor_root();
        assert_eq!(
            ambiguous.children[0].node.key,
            ambiguous.children[1].node.key
        );
        // Host duplicate validation rejects ambiguity; do not silently pick sibling positions.
    }

    #[test]
    fn checkbox_defaults_use_state_identity_and_typed_boolean_action() {
        let state = State::<bool>::new("checked", false);
        let action = BoolAction::remote("change");
        let first = Checkbox::new("first", state, action).into_view();
        let mut edited = Checkbox::new("edited", state, action);
        edited.enabled = false;
        let edited = edited.into_view();
        assert_eq!(first.node.key, edited.node.key);
        assert_eq!(first.node.kind, abi::CHECKBOX);
        assert_eq!(first.node.slot, state.key());
        assert_eq!(first.node.action, action.spec().id);
        assert_eq!(first.node.disabled, 0);
        assert_eq!(edited.node.disabled, 1);
        assert_eq!(
            Checkbox::new("label", state, action)
                .key("explicit")
                .into_view()
                .node
                .key,
            abi::key("explicit")
        );
    }

    #[test]
    fn local_static_identity_survives_content_edits_reorder_and_insertion() {
        fn build(edited: bool) -> View {
            let title = Text::new(if edited { "new title" } else { "title" });
            let subtitle = Text::new("subtitle");
            if edited {
                let inserted = Text::new("inserted");
                {
                    let __blockui_tree = ::blockui::Ui::new();
                    {
                        let _scope_0 = __blockui_tree.scope(Column::default());
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(inserted)
                                .__local_identity(stringify!(inserted)),
                        );
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(subtitle)
                                .__local_identity(stringify!(subtitle)),
                        );
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(title)
                                .__local_identity(stringify!(title)),
                        );
                    }
                    __blockui_tree.finish()
                }
                .actor_root()
            } else {
                {
                    let __blockui_tree = ::blockui::Ui::new();
                    {
                        let _scope_0 = __blockui_tree.scope(Column::default());
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(title)
                                .__local_identity(stringify!(title)),
                        );
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(subtitle)
                                .__local_identity(stringify!(subtitle)),
                        );
                    }
                    __blockui_tree.finish()
                }
                .actor_root()
            }
        }
        let old = build(false);
        let new = build(true);
        assert_ne!(old.children[0].node.key, old.children[1].node.key);
        assert_eq!(old.children[0].node.key, new.children[2].node.key);
        assert_eq!(old.children[1].node.key, new.children[1].node.key);
        let title = Text::new("explicit").key("explicit");
        let explicit = {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree
                    .add(::blockui::IntoView::into_view(title).__local_identity(stringify!(title)));
            }
            __blockui_tree.finish()
        }
        .actor_root();
        assert_eq!(explicit.children[0].node.key, abi::key("explicit"));
        let repeated = {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                for _ in 0..2 {
                    let title = Text::new("row");
                    __blockui_tree.add(
                        ::blockui::IntoView::into_view(title).__local_identity(stringify!(title)),
                    );
                }
            }
            __blockui_tree.finish()
        }
        .actor_root();
        assert_eq!(repeated.children[0].node.key, repeated.children[1].node.key);
        let title = Column::default();
        let nested = {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                {
                    let _scope_1 = __blockui_tree.scope(
                        ::blockui::IntoView::into_view(title).__local_identity(stringify!(title)),
                    );
                    __blockui_tree.add(Text::new("nested"));
                }
            }
            __blockui_tree.finish()
        }
        .actor_root();
        assert_ne!(nested.children[0].children[0].node.key, 0);
    }

    #[test]
    fn button_appearance_changes_preserve_action_and_identity() {
        let action = Action::new("remove", |_, _| Ok(()));
        let make = |variant| {
            Button {
                variant,
                on_click: action,
                ..default()
            }
            .into_view()
        };
        let primary = make(ButtonVariant::Primary);
        for (variant, wire) in [(ButtonVariant::Secondary, 1), (ButtonVariant::Danger, 2)] {
            let view = make(variant);
            assert_eq!(view.node.key, primary.node.key);
            assert_eq!(view.node.action, primary.node.action);
            assert_eq!(view.node.button_variant, wire);
        }
    }

    #[test]
    fn cloned_local_views_keep_named_identity_across_edits_and_reorder() {
        #[derive(Clone)]
        struct Caption(&'static str, Key);
        impl IntoView for Caption {
            fn into_view(self) -> View {
                Text::new(self.0).key(self.1).into_view()
            }
        }
        fn build(edited: bool) -> View {
            let title = Caption(if edited { "Edited" } else { "Original" }, Key::default());
            let subtitle = Caption("Subtitle", Key::default());
            let explicit = Caption("Explicit", Key::named("fixed.title"));
            if edited {
                {
                    let __blockui_tree = ::blockui::Ui::new();
                    {
                        let _scope_0 = __blockui_tree.scope(Column::default());
                        __blockui_tree.add(Text::new("Inserted"));
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(subtitle.clone())
                                .__local_identity(stringify!(subtitle)),
                        );
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(explicit.clone())
                                .__local_identity(stringify!(explicit)),
                        );
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(title.clone())
                                .__local_identity(stringify!(title)),
                        );
                    }
                    __blockui_tree.finish()
                }
                .actor_root()
            } else {
                {
                    let __blockui_tree = ::blockui::Ui::new();
                    {
                        let _scope_0 = __blockui_tree.scope(Column::default());
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(title.clone())
                                .__local_identity(stringify!(title)),
                        );
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(subtitle.clone())
                                .__local_identity(stringify!(subtitle)),
                        );
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(explicit.clone())
                                .__local_identity(stringify!(explicit)),
                        );
                    }
                    __blockui_tree.finish()
                }
                .actor_root()
            }
        }
        let old = build(false);
        let new = build(true);
        assert_ne!(old.children[0].node.key, old.children[1].node.key);
        assert_eq!(old.children[0].node.key, new.children[3].node.key);
        assert_eq!(old.children[1].node.key, new.children[1].node.key);
        assert_eq!(old.children[2].node.key, abi::key("fixed.title"));
        assert_eq!(old.children[2].node.key, new.children[2].node.key);
    }

    #[test]
    fn field_widget_identity_survives_edits_reorder_and_match_syntax() {
        struct Parts {
            title: Text,
            subtitle: Text,
            container: Column,
        }
        fn build(edited: bool) -> View {
            let parts = Parts {
                title: Text::new(if edited { "edited" } else { "original" }),
                subtitle: Text::new("subtitle"),
                container: Column::default(),
            };
            if edited {
                {
                    let __blockui_tree = ::blockui::Ui::new();
                    {
                        let _scope_0 = __blockui_tree.scope(Column::default());
                        __blockui_tree.add(Text::new("inserted"));
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(parts.subtitle)
                                .__local_identity(stringify!(parts.subtitle)),
                        );
                        match edited {
                            true if edited => {
                                __blockui_tree.add(
                                    ::blockui::IntoView::into_view(parts.title)
                                        .__local_identity(stringify!(parts.title)),
                                );
                            }
                            _ => {
                                __blockui_tree.add(Text::new("fallback"));
                            }
                        };
                        {
                            let _scope_1 = __blockui_tree.scope(
                                ::blockui::IntoView::into_view(parts.container)
                                    .__local_identity(stringify!(parts.container)),
                            );
                            __blockui_tree.add(Text::new("nested edited"));
                        }
                    }
                    __blockui_tree.finish()
                }
                .actor_root()
            } else {
                {
                    let __blockui_tree = ::blockui::Ui::new();
                    {
                        let _scope_0 = __blockui_tree.scope(Column::default());
                        match edited {
                            true => {
                                __blockui_tree.add(Text::new("fallback"));
                            }
                            false => {
                                __blockui_tree.add(
                                    ::blockui::IntoView::into_view(parts.title)
                                        .__local_identity(stringify!(parts.title)),
                                );
                            }
                        };
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(parts.subtitle)
                                .__local_identity(stringify!(parts.subtitle)),
                        );
                        {
                            let _scope_1 = __blockui_tree.scope(
                                ::blockui::IntoView::into_view(parts.container)
                                    .__local_identity(stringify!(parts.container)),
                            );
                            __blockui_tree.add(Text::new("nested"));
                        };
                    }
                    __blockui_tree.finish()
                }
                .actor_root()
            }
        }
        let old = build(false);
        let new = build(true);
        assert_ne!(old.children[0].node.key, old.children[1].node.key);
        assert_eq!(old.children[0].node.key, new.children[2].node.key);
        assert_eq!(old.children[1].node.key, new.children[1].node.key);
        assert_eq!(old.children[2].node.key, new.children[3].node.key);
        assert_eq!(
            old.children[2].children[0].node.key,
            new.children[3].children[0].node.key
        );
        let parts = Parts {
            title: Text::new("explicit").key("explicit"),
            subtitle: Text::new("s"),
            container: Column::default(),
        };
        let explicit = {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    ::blockui::IntoView::into_view(parts.subtitle)
                        .__local_identity(stringify!(parts.subtitle)),
                );
                __blockui_tree.add(
                    ::blockui::IntoView::into_view(parts.title)
                        .__local_identity(stringify!(parts.title)),
                );
            }
            __blockui_tree.finish()
        }
        .actor_root();
        assert_eq!(explicit.children[1].node.key, abi::key("explicit"));
        struct Outer {
            parts: Parts,
        }
        let outer = Outer {
            parts: Parts {
                title: Text::new("nested field"),
                subtitle: Text::new("subtitle"),
                container: Column::default(),
            },
        };
        let nested = {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    ::blockui::IntoView::into_view(outer.parts.title)
                        .__local_identity(stringify!(outer.parts.title)),
                );
                __blockui_tree.add(
                    ::blockui::IntoView::into_view(outer.parts.subtitle)
                        .__local_identity(stringify!(outer.parts.subtitle)),
                );
                {
                    let _scope_1 = __blockui_tree.scope(
                        ::blockui::IntoView::into_view(outer.parts.container)
                            .__local_identity(stringify!(outer.parts.container)),
                    );
                    __blockui_tree.add(Text::new("child"));
                }
            }
            __blockui_tree.finish()
        }
        .actor_root();
        assert_ne!(nested.children[0].node.key, nested.children[1].node.key);
        assert_ne!(nested.children[2].children[0].node.key, 0);
    }

    #[test]
    fn named_controls_share_bindings_without_sharing_widget_identity() {
        fn build(edited: bool) -> View {
            let state = State::<String>::new("document");
            let action = Action::new("save", |_, _| Ok(()));
            let primary = Button {
                on_click: action,
                ..default()
            };
            let secondary = Button {
                on_click: action,
                ..default()
            };
            let editor = TextInput::new(state);
            let preview = TextInput::new(state);
            if edited {
                {
                    let __blockui_tree = ::blockui::Ui::new();
                    {
                        let _scope_0 = __blockui_tree.scope(Column::default());
                        __blockui_tree.add(Text::new("inserted"));
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(preview)
                                .__local_identity(stringify!(preview)),
                        );
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(secondary)
                                .__local_identity(stringify!(secondary)),
                        );
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(editor)
                                .__local_identity(stringify!(editor)),
                        );
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(primary)
                                .__local_identity(stringify!(primary)),
                        );
                    }
                    __blockui_tree.finish()
                }
                .actor_root()
            } else {
                {
                    let __blockui_tree = ::blockui::Ui::new();
                    {
                        let _scope_0 = __blockui_tree.scope(Column::default());
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(primary)
                                .__local_identity(stringify!(primary)),
                        );
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(secondary)
                                .__local_identity(stringify!(secondary)),
                        );
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(editor)
                                .__local_identity(stringify!(editor)),
                        );
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(preview)
                                .__local_identity(stringify!(preview)),
                        );
                    }
                    __blockui_tree.finish()
                }
                .actor_root()
            }
        }
        let old = build(false);
        let new = build(true);
        assert_ne!(old.children[0].node.key, old.children[1].node.key);
        assert_ne!(old.children[2].node.key, old.children[3].node.key);
        for (before, after) in [(0, 4), (1, 2), (2, 3), (3, 1)] {
            assert_eq!(old.children[before].node.key, new.children[after].node.key);
        }
        assert_eq!(old.children[2].node.slot, old.children[3].node.slot);
        let explicit = TextInput::new(State::<String>::new("document"))
            .key("explicit")
            .into_view();
        let view = {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    ::blockui::IntoView::into_view(explicit).__local_identity(stringify!(explicit)),
                );
            }
            __blockui_tree.finish()
        }
        .actor_root();
        assert_eq!(view.children[0].node.key, abi::key("explicit"));
    }

    #[test]
    fn keyed_collection_children_retain_business_identity_and_descendant_keys() {
        let old = keyed_children(
            "servers",
            [
                ("alpha", Text::new("Online")),
                ("beta", Text::new("Offline")),
            ],
        );
        let new = keyed_children(
            "servers",
            [
                ("gamma", Text::new("New")),
                ("beta", Text::new("Online")),
                ("alpha", Text::new("Renamed")),
            ],
        );
        assert_eq!(old[0].node.key, new[2].node.key);
        assert_eq!(old[1].node.key, new[1].node.key);
        assert_eq!(
            old[0].node.key,
            Key::named("servers").scoped_named("alpha").0
        );
        assert_ne!(
            old[0].node.key,
            keyed_children("documents", [("alpha", Text::new("Other"))])[0]
                .node
                .key
        );
        let nested = keyed_children(
            "servers",
            [(
                7_u64,
                Column::default()
                    .children(vec![Text::new("Child").key("explicit.child").into_view()]),
            )],
        );
        assert_eq!(nested[0].node.key, Key::named("servers").scoped(7).0);
        assert_eq!(
            nested[0].children[0].node.key,
            Key::named("explicit.child").0
        );
        let duplicates = keyed_children(
            "servers",
            [("alpha", Text::new("One")), ("alpha", Text::new("Two"))],
        );
        assert_eq!(duplicates[0].node.key, duplicates[1].node.key);
        assert!(keyed_children::<&str, Text>("servers", []).is_empty());
    }

    #[test]
    fn string_business_keys_preserve_rows_through_insert_reorder_and_content_edit() {
        fn build(edited: bool) -> View {
            let ids = if edited {
                vec!["gamma", "beta", "alpha"]
            } else {
                vec!["alpha", "beta"]
            };
            {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    for id in ids {
                        __blockui_tree.add(
                            Text::new(if edited { "edited" } else { "original" })
                                .key(Key::named("servers").scoped_named(id)),
                        );
                    }
                }
                __blockui_tree.finish()
            }
            .actor_root()
        }
        let old = build(false);
        let new = build(true);
        assert_eq!(old.children[0].node.key, new.children[2].node.key);
        assert_eq!(old.children[1].node.key, new.children[1].node.key);
        assert_ne!(new.children[0].node.key, old.children[0].node.key);
        const ALPHA: Key = Key::named("servers").scoped_named("alpha");
        assert_eq!(old.children[0].node.key, ALPHA.0);
        assert_ne!(ALPHA, Key::named("documents").scoped_named("alpha"));
        let borrowed = String::from("alpha");
        assert_eq!(
            Text::new("borrowed string")
                .key(&borrowed)
                .into_view()
                .node
                .key,
            Text::new("str").key("alpha").into_view().node.key
        );
        assert_eq!(
            Text::new("owned")
                .key(String::from("alpha"))
                .into_view()
                .node
                .key,
            Text::new("borrowed").key("alpha").into_view().node.key
        );
    }

    #[test]
    fn actor_root_identity_is_stable_and_preserves_explicit_keys() {
        let first = Column::default().into_view().actor_root();
        let edited = Column {
            gap: 20.0,
            padding: 16.0,
            ..Default::default()
        }
        .children(vec![
            Text::new("inserted sibling").key("inserted").into_view(),
        ])
        .actor_root();
        assert_ne!(first.node.key, 0);
        assert_eq!(first.node.key, edited.node.key);
        assert_eq!(
            Column::default()
                .key("explicit")
                .into_view()
                .actor_root()
                .node
                .key,
            abi::key("explicit")
        );
        assert_eq!(Column::default().into_view().node.key, 0);
        assert_eq!(first.find(first.node.key).unwrap().node.key, first.node.key);
    }

    #[test]
    fn bind_identity_macros_do_not_resolve_callers_macro_names() {
        macro_rules! module_path {
            () => {
                123
            };
        }
        macro_rules! stringify {
            ($($tokens:tt)*) => {
                456
            };
        }
        assert_eq!(module_path!(), 123);
        assert_eq!(stringify!(caller), 456);
        fn caption(_: &Cx<'_>) -> Result<String> {
            Ok("caption".into())
        }
        assert_ne!(Text::new(bind!(caption)).into_view().node.key, 0);
    }

    #[test]
    fn path_binding_identity_ignores_value_and_widget_style() {
        fn caption(_: &Cx<'_>) -> Result<String> {
            Ok("caption".into())
        }
        fn other(_: &Cx<'_>) -> Result<String> {
            Ok("other".into())
        }
        let first = Text::new(bind!(caption)).into_view();
        let repeated = Text::new(bind!(caption)).into_view();
        let other = Text::new(bind!(other)).into_view();
        assert_ne!(first.node.key, 0);
        assert_eq!(first.node.key, repeated.node.key);
        assert_ne!(first.node.key, other.node.key);
        let scroll = ScrollView {
            text: bind!(caption),
            ..default()
        }
        .into_view();
        let styled = ScrollView {
            text: bind!(caption),
            height: 300.0,
            ..default()
        }
        .into_view();
        assert_ne!(scroll.node.key, first.node.key);
        assert_eq!(scroll.node.key, styled.node.key);
        let explicit = Key::named("caption.explicit");
        assert_eq!(
            Text {
                key: explicit,
                text: bind!(caption)
            }
            .into_view()
            .node
            .key,
            explicit.0
        );
        assert_eq!(
            Text::new(bind!(|_| Ok("anonymous".into())))
                .into_view()
                .node
                .key,
            0
        );
        assert_eq!(Text::new("literal").into_view().node.key, 0);
    }

    #[test]
    fn semantic_widget_defaults_respect_explicit_overrides() {
        let state = State::<String>::new("document");
        let first = TextInput::new(state).into_view();
        let second = TextInput::new(state).into_view();
        assert_ne!(first.node.key, 0);
        assert_eq!(first.node.key, second.node.key);
        let explicit = Key::named("document.secondary");
        assert_eq!(
            TextInput {
                key: explicit,
                value: state,
                read_only: false,
                on_change: None,
                on_submit: None,
                on_cancel: None,
                enabled: true,
            }
            .into_view()
            .node
            .key,
            explicit.0
        );
        let action = Action::new("increment", |_, _| Ok(()));
        let button = Button {
            on_click: action,
            ..default()
        }
        .into_view();
        assert_ne!(button.node.key, 0);
        assert_ne!(button.node.key, first.node.key);
        assert_eq!(
            Button {
                key: explicit,
                on_click: action,
                ..default()
            }
            .into_view()
            .node
            .key,
            explicit.0
        );
    }

    #[test]
    #[allow(clippy::single_match)] // Exercise the match construction branch.
    fn final_nested_container_can_omit_semicolon() {
        let tree = {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column {
                    key: "root".into(),
                    ..default()
                });
                __blockui_tree.add(Text::new("before"));
                {
                    let _scope_1 = __blockui_tree.scope(Column {
                        key: "nested".into(),
                        ..default()
                    });
                    for item in 0..2 {
                        {
                            let _scope_2 = __blockui_tree.scope(Column {
                                key: Key::named("row").scoped(item),
                                ..default()
                            });
                            __blockui_tree.add(Text::new("item"));
                        }
                    }
                    if true {
                        {
                            let _scope_3 = __blockui_tree.scope(Column {
                                key: "conditional".into(),
                                ..default()
                            });
                        }
                    }
                    match 1 {
                        1 => {
                            let _scope_4 = __blockui_tree.scope(Column {
                                key: "matched".into(),
                                ..default()
                            });
                        }
                        _ => {}
                    }
                }
            }
            __blockui_tree.finish()
        };
        assert_eq!(tree.children.len(), 2);
        assert_eq!(tree.children[1].children.len(), 4);
        assert_eq!(tree.children[1].children[0].children.len(), 1);
    }

    #[test]
    #[allow(clippy::single_match)] // Exercise the match construction branch.
    fn ui_allocations_do_not_resolve_callers_vec_names() {
        struct Vec;
        macro_rules! vec {
            ($($tokens:tt)*) => {
                123_i64
            };
        }
        let _shadowed_type = Vec;
        assert_eq!(vec![], 123);
        let children = "authored local";
        let tree = {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column {
                    key: "root".into(),
                    ..default()
                });
                __blockui_tree.add(Text::new(children));
                {
                    let _scope_1 = __blockui_tree.scope(Column {
                        key: "nested".into(),
                        ..default()
                    });
                    for n in 0..2 {
                        if n == 0 {
                            __blockui_tree.add(Text::new("zero"));
                        } else {
                            __blockui_tree.add(Text::new("one"));
                        }
                    }
                    match 1 {
                        1 => {
                            __blockui_tree.add(Text::new("matched"));
                        }
                        _ => {}
                    }
                };
            }
            __blockui_tree.finish()
        };
        assert_eq!(tree.children.len(), 2);
        assert_eq!(tree.children[1].children.len(), 3);
        assert_eq!(children, "authored local");
    }

    #[test]
    fn rust_locals_types_destructuring_shadowing_and_let_else() {
        fn build(choice: Option<u64>) -> View {
            {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column {
                        key: "root".into(),
                        ..default()
                    });
                    let Some(id) = choice else {
                        return Text::new("fallback").into_view();
                    };
                    let (id, label): (u64, &str) = (id + 1, "row");
                    __blockui_tree.add(Text {
                        key: Key::named("first").scoped(id),
                        text: label.into(),
                    });
                    let label = format!("{label} {id}");
                    for part in 0..2_u64 {
                        let value = format!("{label}/{part}");
                        __blockui_tree.add(Text {
                            key: Key::named("part").scoped(part),
                            text: value.into(),
                        });
                    }
                }
                __blockui_tree.finish()
            }
        }
        let tree = build(Some(2));
        assert_eq!(tree.children.len(), 3);
        assert_eq!(tree.children[0].node.key, Key::named("first").scoped(3).0);
        assert!(build(None).children.is_empty());
    }

    #[test]
    fn match_local_widget_identity_matches_braced_child_syntax() {
        for choice in [true, false] {
            let title = Text::new("title");
            let subtitle = Text::new("subtitle");
            let direct = {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column {
                        key: "root".into(),
                        ..default()
                    });
                    match choice {
                        true => {
                            __blockui_tree.add(
                                ::blockui::IntoView::into_view(title)
                                    .__local_identity(stringify!(title)),
                            );
                        }
                        false => {
                            __blockui_tree.add(
                                ::blockui::IntoView::into_view(subtitle)
                                    .__local_identity(stringify!(subtitle)),
                            );
                        }
                    }
                }
                __blockui_tree.finish()
            };
            let title = Text::new("edited title");
            let subtitle = Text::new("edited subtitle");
            let braced = {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column {
                        key: "root".into(),
                        ..default()
                    });
                    match choice {
                        true => {
                            __blockui_tree.add(
                                ::blockui::IntoView::into_view(title)
                                    .__local_identity(stringify!(title)),
                            );
                        }
                        false => {
                            __blockui_tree.add(
                                ::blockui::IntoView::into_view(subtitle)
                                    .__local_identity(stringify!(subtitle)),
                            );
                        }
                    }
                }
                __blockui_tree.finish()
            };
            let mut a = Vec::new();
            let mut b = Vec::new();
            let direct = direct.actor_root();
            direct.flatten(0, &mut a).unwrap();
            braced.actor_root().flatten(0, &mut b).unwrap();
            assert_eq!(a[1].key, b[1].key);
            assert_eq!(
                direct.children[0].local_identity,
                Some(if choice { "title" } else { "subtitle" })
            );
        }
    }
    #[test]
    fn match_patterns_guards_sequences_and_following_siblings() {
        for choice in [None, Some(0_u64), Some(2)] {
            let tree = {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column {
                        key: "root".into(),
                        ..default()
                    });
                    match choice {
                        Some(id) if id > 0 => {
                            for part in 0..id {
                                __blockui_tree.add(Text {
                                    key: Key::named("part").scoped(part),
                                    text: "part".into(),
                                });
                            }
                        }
                        Some(_) => {}
                        None => {
                            __blockui_tree.add(Text {
                                key: "missing".into(),
                                text: "missing".into(),
                            });
                            __blockui_tree.add(Text {
                                key: "hint".into(),
                                text: "hint".into(),
                            });
                        }
                    };
                    __blockui_tree.add(Text {
                        key: "after".into(),
                        text: "after".into(),
                    });
                }
                __blockui_tree.finish()
            };
            let single = {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column {
                        key: "single".into(),
                        ..default()
                    });
                    match choice {
                        Some(_) => {
                            __blockui_tree.add(Text::new("present"));
                        }
                        None => {
                            __blockui_tree.add(Text::new("missing"));
                        }
                    }
                }
                __blockui_tree.finish()
            };
            assert_eq!(single.children.len(), 1);
            assert_eq!(tree.children.len(), if choice == Some(0) { 1 } else { 3 });
            assert_eq!(
                tree.children.last().unwrap().node.key,
                Key::named("after").0
            );
        }
    }

    #[test]
    fn conditional_sequences_if_let_else_if_and_nested_loops() {
        for choice in [None, Some(0_u64), Some(1), Some(2)] {
            let tree = {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column {
                        key: "root".into(),
                        ..default()
                    });
                    if let Some(id) = choice {
                        if id == 0 {
                            __blockui_tree.add(Text {
                                key: "zero".into(),
                                text: "zero".into(),
                            });
                            __blockui_tree.add(Text {
                                key: "extra".into(),
                                text: "extra".into(),
                            });
                        } else if id == 1 {
                            __blockui_tree.add(Text {
                                key: "one".into(),
                                text: "one".into(),
                            });
                            __blockui_tree.add(Text {
                                key: "extra".into(),
                                text: "extra".into(),
                            });
                        } else {
                            for part in 0..id {
                                __blockui_tree.add(Text {
                                    key: Key::named("part").scoped(part),
                                    text: "part".into(),
                                });
                            }
                        }
                    } else {
                        __blockui_tree.add(Text {
                            key: "none".into(),
                            text: "none".into(),
                        });
                    };
                    if choice.is_none() {
                        __blockui_tree.add(Text {
                            key: "missing".into(),
                            text: "missing".into(),
                        });
                    }
                    __blockui_tree.add(Text {
                        key: "after".into(),
                        text: "after".into(),
                    });
                }
                __blockui_tree.finish()
            };
            assert_eq!(tree.children.len(), 3);
            assert_eq!(tree.children[2].node.key, Key::named("after").0);
            assert_eq!(
                tree.children[0].node.key,
                match choice {
                    None => Key::named("none").0,
                    Some(0) => Key::named("zero").0,
                    Some(1) => Key::named("one").0,
                    Some(_) => Key::named("part").scoped(0).0,
                }
            );
        }
    }

    #[test]
    fn rust_loop_patterns_nested_children_order_and_scoped_keys() {
        let rows = [(2_u64, "two"), (1, "one")];
        let tree = {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column {
                    key: "root".into(),
                    ..default()
                });
                __blockui_tree.add(Text {
                    key: "before".into(),
                    text: "before".into(),
                });
                for (id, label) in rows.into_iter().filter(|(id, _)| *id > 0) {
                    {
                        let _scope_1 = __blockui_tree.scope(Column {
                            key: Key::named("row").scoped(id),
                            ..default()
                        });
                        for part in 0..2_u64 {
                            __blockui_tree.add(Text {
                                key: Key::named("part").scoped(id).scoped(part),
                                text: label.into(),
                            });
                        }
                    };
                }
                for ignored in std::iter::empty::<u64>() {
                    __blockui_tree.add(Text {
                        key: ignored.into(),
                        text: "unreachable".into(),
                    });
                }
                __blockui_tree.add(Text {
                    key: "after".into(),
                    text: "after".into(),
                });
            }
            __blockui_tree.finish()
        };
        assert_eq!(tree.children.len(), 4);
        assert_eq!(tree.children[0].node.key, Key::named("before").0);
        assert_eq!(tree.children[1].node.key, Key::named("row").scoped(2).0);
        assert_eq!(tree.children[2].node.key, Key::named("row").scoped(1).0);
        assert_eq!(tree.children[3].node.key, Key::named("after").0);
        for (row, id) in tree.children[1..3].iter().zip([2, 1]) {
            assert_eq!(row.children.len(), 2);
            for (part, child) in row.children.iter().enumerate() {
                assert_eq!(
                    child.node.key,
                    Key::named("part").scoped(id).scoped(part as u64).0
                );
            }
        }
    }
}

#[cfg(test)]
mod timer_precision_tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn timer_wire_precision_rounds_up_and_enforces_exact_duration_bound() {
        for (duration, expected) in [
            (Duration::ZERO, 0),
            (Duration::from_nanos(1), 1),
            (Duration::from_nanos(999_999), 1),
            (Duration::from_millis(1), 1),
            (Duration::from_nanos(1_000_001), 2),
            (Duration::from_secs(86_400), 86_400_000),
        ] {
            assert_eq!(timer_millis(duration).unwrap(), expected);
        }
        assert!(timer_millis(Duration::from_secs(86_400) + Duration::from_nanos(1)).is_err());
        assert!(timer_millis(Duration::MAX).is_err());
    }
}

#[cfg(test)]
mod request_scalar_tests {
    use super::*;
    #[test]
    fn primitive_request_codecs_preserve_bits_and_validate_booleans() {
        for bits in [
            0_u64,
            1_u64 << 63,
            1,
            1.25_f64.to_bits(),
            f64::INFINITY.to_bits(),
            f64::NEG_INFINITY.to_bits(),
            0x7ff8_0000_0000_0042,
            0xfff8_0000_0000_0043,
        ] {
            let value = f64::from_wire(bits as i64).unwrap();
            assert_eq!(value.into_wire() as u64, bits);
            assert_eq!(
                RequestAction::<f64, bool>::remote("query").encode(value),
                (abi::key("query"), bits as i64)
            );
        }
        for value in [i64::MIN, -1, 0, 1, i64::MAX] {
            assert_eq!(i64::from_wire(value).unwrap().into_wire(), value);
            assert_eq!(
                RequestAction::<i64, f64>::remote("query").encode(value).1,
                value
            );
        }
        for value in [false, true] {
            assert_eq!(bool::from_wire(value.into_wire()).unwrap(), value);
            assert_eq!(
                RequestAction::<bool, i64>::remote("query").encode(value).1,
                i64::from(value)
            );
        }
        for invalid in [i64::MIN, -1, 2, i64::MAX] {
            assert!(bool::from_wire(invalid).is_err());
        }
        const II: RequestAction<i64, i64> = request!(i64 => i64, "ii", |_, x| Ok(x));
        const IB: RequestAction<i64, bool> = request!(i64 => bool, "ib", |_, x| Ok(x != 0));
        const IF: RequestAction<i64, f64> = request!(i64 => f64, "if", |_, x| Ok(x as f64));
        const BI: RequestAction<bool, i64> = request!(bool => i64, "bi", |_, x| Ok(i64::from(x)));
        const BB: RequestAction<bool, bool> = request!(bool => bool, "bb", |_, x| Ok(x));
        const BF: RequestAction<bool, f64> =
            request!(bool => f64, "bf", |_, x| Ok(if x { 1.0 } else { 0.0 }));
        const FI: RequestAction<f64, i64> =
            request!(f64 => i64, "fi", |_, x| Ok(x.to_bits() as i64));
        const FB: RequestAction<f64, bool> =
            request!(f64 => bool, "fb", |_, x| Ok(x.is_sign_negative()));
        const FF: RequestAction<f64, f64> = request!(f64 => f64, "ff", |_, x| Ok(x));
        let actions = [
            II.spec(),
            IB.spec(),
            IF.spec(),
            BI.spec(),
            BB.spec(),
            BF.spec(),
            FI.spec(),
            FB.spec(),
            FF.spec(),
        ];
        for (action, name) in actions
            .iter()
            .zip(["ii", "ib", "if", "bi", "bb", "bf", "fi", "fb", "ff"])
        {
            assert_eq!(action.id, abi::key(name));
            assert!(matches!(action.handler, Handler::Request(_)));
        }
    }
}

impl State<f64> {
    /// Project the current slot into text with a noncapturing formatter.
    pub fn map_text(self, format: fn(f64) -> String) -> Binding {
        Binding::MappedFloat {
            slot: self.key(),
            format,
        }
    }
}
impl BoundState<'_, '_, f64> {
    /// Project without retaining this turn's context.
    pub fn map_text(&self, format: fn(f64) -> String) -> Binding {
        self.state.map_text(format)
    }
}

impl State<bool> {
    /// Project the current slot into text with a noncapturing formatter.
    pub fn map_text(self, format: fn(bool) -> String) -> Binding {
        Binding::MappedBoolean {
            slot: self.key(),
            format,
        }
    }
}
impl BoundState<'_, '_, bool> {
    /// Project without retaining this turn's context.
    pub fn map_text(&self, format: fn(bool) -> String) -> Binding {
        self.state.map_text(format)
    }
}

impl State<String> {
    /// Project the current slot into text with a noncapturing formatter.
    pub fn map_text(self, format: fn(String) -> String) -> Binding {
        Binding::MappedText {
            slot: self.key(),
            format,
        }
    }
}
impl BoundState<'_, '_, String> {
    /// Project without retaining this turn's context.
    pub fn map_text(&self, format: fn(String) -> String) -> Binding {
        self.state.map_text(format)
    }
}

impl State<Vec<i64>> {
    /// Project the current slot into text with a noncapturing formatter.
    pub fn map_text(self, format: fn(Vec<i64>) -> String) -> Binding {
        Binding::MappedList {
            slot: self.key(),
            format,
        }
    }
}
impl BoundState<'_, '_, Vec<i64>> {
    /// Copy a clipped range. Reversed ranges are programmer errors; use the
    /// State descriptor for fallible ranges supplied by a caller.
    pub fn get_range(&self, range: std::ops::Range<usize>) -> Vec<i64> {
        self.state
            .get_range(self.cx, range)
            .expect("invalid range or invalidated turn-bound state slot")
    }
    /// Read the current length without copying items.
    pub fn len(&self) -> usize {
        self.state
            .len(self.cx)
            .expect("host invalidated a turn-bound state slot")
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Project without retaining this turn's context.
    pub fn map_text(&self, format: fn(Vec<i64>) -> String) -> Binding {
        self.state.map_text(format)
    }
}

impl State<Vec<String>> {
    /// Project a text collection with a noncapturing formatter owned by current code.
    /// ```
    /// use blockui::State;
    /// let caption = State::<Vec<String>>::new("labels").map_text(|labels| labels.join(", "));
    /// ```
    pub fn map_text(self, format: fn(Vec<String>) -> String) -> Binding {
        Binding::MappedTextList {
            slot: self.key(),
            format,
        }
    }
}
impl BoundState<'_, '_, Vec<String>> {
    /// Decode a record collection for this turn. The runtime keeps the encoded
    /// storage; returned Rust records are temporary current-code values.
    /// Codec and migration errors remain recoverable instead of panicking.
    /// Dependencies cover the whole collection, not individual record fields.
    pub fn records<T: Record>(&self) -> Result<Vec<(String, T)>> {
        RecordMap::<T>::new(self.state).entries(self.cx)
    }
    /// Read the current length without copying strings.
    pub fn len(&self) -> usize {
        self.state
            .len(self.cx)
            .expect("host invalidated a turn-bound state slot")
    }
    /// Check emptiness without copying strings.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Copy a clipped range during this turn. A reversed range is a programming
    /// error and panics; use State::get_range for fallible caller-supplied ranges.
    pub fn get_range(&self, range: std::ops::Range<usize>) -> Vec<String> {
        self.state
            .get_range(self.cx, range)
            .expect("invalid range or invalidated turn-bound state slot")
    }
    /// Project without retaining this turn's context.
    pub fn map_text(&self, format: fn(Vec<String>) -> String) -> Binding {
        self.state.map_text(format)
    }
}

#[cfg(test)]
mod text_range_copy_tests {
    use super::*;
    struct Trace {
        indices: Vec<usize>,
        copied_bytes: usize,
    }
    unsafe extern "C" fn read(
        context: *mut std::ffi::c_void,
        _: u64,
        index: usize,
        output: *mut u8,
        capacity: usize,
    ) -> isize {
        let trace = unsafe { &mut *context.cast::<Trace>() };
        trace.indices.push(index);
        if index == usize::MAX {
            return 4096;
        }
        if !(64..128).contains(&index) {
            return -1;
        }
        let text = "标题😀\0".as_bytes();
        if !output.is_null() {
            if capacity < text.len() {
                return -1;
            }
            unsafe {
                std::ptr::copy_nonoverlapping(text.as_ptr(), output, text.len());
            }
            trace.copied_bytes += text.len();
        }
        text.len() as isize
    }
    #[test]
    fn visible_page_reads_only_64_items_from_4096() {
        let mut trace = Trace {
            indices: Vec::new(),
            copied_bytes: 0,
        };
        let context = std::ptr::from_mut(&mut trace).cast();
        assert_eq!(text_collection_len(context, 1, read).unwrap(), 4096);
        assert_eq!(trace.indices, [usize::MAX]);
        trace.indices.clear();
        let values = read_text_collection_range(context, 1, read, Some(64..128)).unwrap();
        assert_eq!(values, vec!["标题😀\0"; 64]);
        assert_eq!(trace.indices.len(), 129); // One count plus length/copy per item.
        assert!(
            trace.indices[1..]
                .iter()
                .all(|index| (64..128).contains(index))
        );
        assert_eq!(trace.copied_bytes, 64 * "标题😀\0".len());
        trace.indices.clear();
        let values = read_text_collection_range(context, 1, read, Some(4096..usize::MAX)).unwrap();
        assert!(values.is_empty());
        assert_eq!(trace.indices, [usize::MAX]);
    }
}

#[cfg(test)]
mod integer_length_tests {
    use super::*;
    unsafe extern "C" fn read(
        context: *mut std::ffi::c_void,
        _: u64,
        output: *mut i64,
        capacity: usize,
    ) -> isize {
        assert!(output.is_null());
        assert_eq!(capacity, 0);
        unsafe { *context.cast::<isize>() }
    }
    #[test]
    fn length_queries_never_request_items_and_reject_invalid_host_lengths() {
        for mut length in [0_isize, 4096, -1, 4097] {
            let result = integer_collection_len((&mut length as *mut isize).cast(), 1, read);
            if (0..=4096).contains(&length) {
                assert_eq!(result.unwrap(), length as usize);
            } else {
                assert!(result.is_err());
            }
        }
    }
}

#[cfg(test)]
mod integer_range_copy_tests {
    use super::*;
    struct Trace {
        calls: usize,
        copied: usize,
    }
    unsafe extern "C" fn read(
        context: *mut std::ffi::c_void,
        _: u64,
        start: usize,
        end: usize,
        out: *mut i64,
        capacity: usize,
    ) -> isize {
        let trace = unsafe { &mut *context.cast::<Trace>() };
        trace.calls += 1;
        assert_eq!((start, end), (64, 128));
        if !out.is_null() {
            assert_eq!(capacity, 64);
            for i in 0..64 {
                unsafe {
                    out.add(i).write((start + i) as i64);
                }
            }
            trace.copied += 64;
        }
        64
    }
    unsafe extern "C" fn malformed(
        context: *mut std::ffi::c_void,
        _: u64,
        _: usize,
        _: usize,
        out: *mut i64,
        _: usize,
    ) -> isize {
        assert!(
            out.is_null(),
            "malformed count must be rejected before copying"
        );
        unsafe { *context.cast::<isize>() }
    }
    #[test]
    fn malformed_counts_are_rejected_before_copying() {
        for mut count in [-1_isize, 2, 4097] {
            assert!(
                read_integer_collection_range(
                    (&mut count as *mut isize).cast(),
                    1,
                    malformed,
                    0..1
                )
                .is_err()
            );
        }
    }
    #[test]
    fn page_copies_only_64_items_and_reverse_range_never_calls_host() {
        let mut trace = Trace {
            calls: 0,
            copied: 0,
        };
        let context = (&mut trace as *mut Trace).cast();
        let page = read_integer_collection_range(context, 1, read, 64..128).unwrap();
        assert_eq!(page, (64..128).collect::<Vec<i64>>());
        assert_eq!((trace.calls, trace.copied), (2, 64));
        let start = 2;
        assert!(read_integer_collection_range(context, 1, read, start..1).is_err());
        assert_eq!((trace.calls, trace.copied), (2, 64));
    }
}

#[cfg(test)]
mod task_result_buffer_tests {
    use super::*;
    struct Reply {
        length: isize,
        status: u32,
        copied: isize,
        copied_status: u32,
        payload: &'static [u8],
        calls: usize,
    }
    unsafe extern "C" fn read(
        context: *mut std::ffi::c_void,
        _: u64,
        out: *mut u8,
        capacity: usize,
        status: *mut u32,
    ) -> isize {
        let reply = unsafe { &mut *context.cast::<Reply>() };
        reply.calls += 1;
        if out.is_null() {
            unsafe {
                *status = reply.status;
            }
            reply.length
        } else {
            assert!(capacity >= reply.payload.len());
            unsafe {
                *status = reply.copied_status;
                std::ptr::copy_nonoverlapping(reply.payload.as_ptr(), out, reply.payload.len());
            }
            reply.copied
        }
    }
    fn reply(length: isize, status: u32) -> Reply {
        Reply {
            length,
            status,
            copied: length,
            copied_status: status,
            payload: b"",
            calls: 0,
        }
    }
    fn result(reply: &mut Reply) -> Result<Option<Vec<u8>>> {
        read_task_result((reply as *mut Reply).cast(), 1, read)
    }
    #[test]
    fn invalid_metadata_never_requests_a_copy() {
        for (length, status) in [(-1, 1), (isize::MAX, 1), (1048577, 2), (1, 0), (0, 3)] {
            let mut reply = reply(length, status);
            assert!(matches!(result(&mut reply), Err(Error::Host(abi::ERROR))));
            assert_eq!(reply.calls, 1);
        }
    }
    #[test]
    fn pending_empty_success_and_io_error_remain_distinct() {
        let mut pending = reply(0, 0);
        assert!(result(&mut pending).unwrap().is_none());
        assert_eq!(pending.calls, 1);
        let mut empty = reply(0, 1);
        assert_eq!(result(&mut empty).unwrap(), Some(vec![]));
        assert_eq!(empty.calls, 2);
        let mut binary = reply(3, 1);
        binary.payload = &[0, 0xff, 0x80];
        assert_eq!(result(&mut binary).unwrap(), Some(vec![0, 0xff, 0x80]));
        let mut error = reply(6, 2);
        error.payload = b"denied";
        assert!(matches!(result(&mut error), Err(Error::Io(text)) if text == "denied"));
    }
    #[test]
    fn copy_count_or_status_changes_reject() {
        let mut short = reply(1, 1);
        short.copied = 0;
        assert!(result(&mut short).is_err());
        let mut changed = reply(0, 1);
        changed.copied_status = 2;
        assert!(result(&mut changed).is_err());
        let mut invalid_utf8 = reply(1, 2);
        invalid_utf8.payload = &[0xff];
        assert!(result(&mut invalid_utf8).is_err());
    }
}

#[cfg(test)]
mod conditional_widget_tests {
    use super::*;
    #[test]
    fn disabled_widgets_do_not_typecheck_or_remove_following_siblings() {
        let view = {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                #[cfg(any())]
                __blockui_tree.add(DoesNotExist::new());
                #[cfg_attr(test, cfg(any()))]
                {
                    let _scope_1 = __blockui_tree.scope(UnknownContainer);
                    __blockui_tree.add(
                        ::blockui::IntoView::into_view(MissingChild)
                            .__local_identity(stringify!(MissingChild)),
                    );
                };
                #[cfg(test)]
                __blockui_tree.add(Text::new("enabled").key("enabled"));
                __blockui_tree.add(Text::new("following").key("following"));
                #[cfg(any())]
                __blockui_tree.add(MissingTerminal);
            }
            __blockui_tree.finish()
        };
        assert_eq!(view.children.len(), 2);
        assert_eq!(view.children[0].node.key, abi::key("enabled"));
        assert_eq!(view.children[1].node.key, abi::key("following"));
    }
    #[test]
    fn enabled_container_attributes_preserve_local_identity() {
        let local = Column::default();
        let plain = {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                {
                    let _scope_1 = __blockui_tree.scope(
                        ::blockui::IntoView::into_view(local).__local_identity(stringify!(local)),
                    );
                    __blockui_tree.add(Text::new("child"));
                };
            }
            __blockui_tree.finish()
        };
        let local = Column::default();
        let attributed = {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                {
                    let _scope_1 = __blockui_tree.scope(
                        ::blockui::IntoView::into_view(local).__local_identity(stringify!(local)),
                    );
                    __blockui_tree.add(Text::new("child"));
                };
            }
            __blockui_tree.finish()
        };
        assert_eq!(
            plain.children[0].local_identity,
            attributed.children[0].local_identity
        );
        assert_eq!(attributed.children[0].children.len(), 1);
    }
}

#[cfg(test)]
mod record_map_text_identity_tests {
    use super::*;
    struct Row {
        name: String,
    }
    impl Record for Row {
        const FORMAT: &'static str = "row";
        fn encode(&self) -> Result<String> {
            Ok(self.name.clone())
        }
        fn decode(text: &str) -> Result<Self> {
            Ok(Self { name: text.into() })
        }
    }
    impl Row {
        fn name_field() -> RecordField<Self, String> {
            RecordField::__new(
                0,
                abi::key("name"),
                |text| Ok(text.into()),
                |row| &mut row.name,
            )
        }
    }
    #[test]
    fn excessive_or_empty_keys_are_rejected_before_emitting_borrowed_wire() {
        unsafe extern "C" fn emitted(ctx: *mut std::ffi::c_void, _: abi::BindingSpec) -> i32 {
            unsafe {
                *ctx.cast::<bool>() = true;
            }
            abi::OK
        }
        let rows = RecordMap::<Row>::new(State::<Vec<String>>::new("rows"));
        for key in [String::new(), "x".repeat(abi::MAX_TEXT_LIST_BYTES)] {
            let view = Text::new(rows.text(key, Row::name_field())).into_view();
            let mut called = false;
            assert!(
                view.emit_bindings(std::ptr::from_mut(&mut called).cast(), emitted, 0, 0)
                    .is_err()
            );
            assert!(!called);
        }
    }
    #[test]
    fn shared_keyed_action_is_data_only_and_remote_tokens_keep_editor_routing() {
        let rows = RecordMap::<Row>::new(State::<Vec<String>>::new("rows"));
        let action = KeyedTextAction::remote("edit");
        let input = TextInput::field(rows.text("a\n🦀", Row::name_field()), action).into_view();
        let area = TextArea::field(
            rows.text("a\n🦀", Row::name_field()),
            TextAction::remote("old"),
        )
        .on_change(action)
        .into_view();
        assert!(matches!(
            input.text,
            Binding::RecordMapText { keyed: true, .. }
        ));
        assert!(matches!(
            area.text,
            Binding::RecordMapText { keyed: true, .. }
        ));
        let (id, payload) = action
            .encode_message(KeyedText {
                key: "a\n🦀".into(),
                text: "\nβ".into(),
            })
            .unwrap();
        assert_eq!(id, abi::key("edit"));
        let MessagePayload::Text(wire) = payload else {
            panic!("wrong keyed payload kind");
        };
        assert_eq!(abi::keyed_text::decode(&wire), Ok(("a\n🦀", "\nβ")));
        assert!(
            action
                .encode_message(KeyedText {
                    key: String::new(),
                    text: "value".into()
                })
                .is_err()
        );
    }
    #[test]
    fn business_key_defaults_distinguish_rows_and_editor_kinds() {
        let rows = RecordMap::<Row>::new(State::<Vec<String>>::new("rows"));
        let action = TextAction::new("edit", |_, _| Ok(()));
        let first = rows.text("a\n🦀", Row::name_field());
        let input = TextInput::field(first.clone(), action).into_view();
        let area = TextArea::field(first, action).into_view();
        let other = TextInput::field(rows.text("b", Row::name_field()), action).into_view();
        assert_ne!(input.node.key, area.node.key);
        assert_ne!(input.node.key, other.node.key);
        let same =
            TextInput::field(rows.text("a\n🦀", Row::name_field()), action).label("New caption");
        assert_eq!(input.node.key, same.node.key);
        assert!(
            matches!(same.text, Binding::RecordMapText { label, .. } if label == "New caption")
        );
    }
}

#[cfg(test)]
mod field_tests {
    use super::*;
    #[test]
    fn slider_field_caption_changes_preserve_range_action_and_identity() {
        let make = |label: &str| {
            Field {
                label: label.into(),
                control: Slider::new(
                    State::<i64>::new("duration", 10000),
                    IntegerRange {
                        min: 0,
                        max: 60000,
                        step: 100,
                    },
                    Action::new("change", |_, _| Ok(())),
                )
                .key("duration.slider"),
            }
            .into_view()
        };
        let first = make("Duration: 10 s");
        let updated = make("Duration: 20 s");
        assert_eq!(first.node.key, updated.node.key);
        let control = &updated.children[1];
        assert_eq!(first.children[1].node.key, control.node.key);
        assert_eq!(control.node.slot, abi::key("duration"));
        assert_eq!(control.node.action, abi::key("change"));
        assert_eq!(
            (
                control.node.range.min,
                control.node.range.max,
                control.node.range.step
            ),
            (0, 60000, 100)
        );
        assert!(matches!(&control.text, Binding::Static(label) if label == "Duration: 20 s"));
    }
    #[test]
    fn field_error_changes_preserve_control_and_feedback_identity() {
        let make = |error: &str| {
            Field {
                label: "Name".into(),
                control: TextInput::new(State::<String>::new("name")),
            }
            .error(error)
        };
        let empty = make("");
        let invalid = make("Enter a name.");
        assert_eq!(empty.node.key, invalid.node.key);
        assert_eq!(empty.children[1].node.key, invalid.children[1].node.key);
        assert_eq!(empty.children[2].node.key, invalid.children[2].node.key);
        assert_eq!(invalid.children[2].node.presentation.text_tone, 3);
        assert_eq!(invalid.children[1].node.slot, abi::key("name"));
        assert!(matches!(&invalid.children[1].text, Binding::Static(label) if label == "Name"));
    }

    #[test]
    fn renaming_caption_preserves_editor_binding_and_field_identity() {
        let make = |label: &str| {
            Field {
                label: label.into(),
                control: TextInput::new(State::<String>::new("document.title")),
            }
            .into_view()
        };
        let first = make("Title");
        let renamed = make("Document title");
        assert_eq!(first.node.key, renamed.node.key);
        assert_eq!(first.children[0].node.key, renamed.children[0].node.key);
        assert_eq!(first.children[1].node.key, renamed.children[1].node.key);
        assert_eq!(first.children[1].node.slot, renamed.children[1].node.slot);
        assert!(
            matches!(&renamed.children[1].text, Binding::Static(name) if name == "Document title")
        );
        assert!(
            matches!(&renamed.children[0].text, Binding::Static(name) if name == "Document title")
        );
    }
    #[test]
    fn computed_editor_keeps_computation_and_current_edit_action() {
        let computed = Computed::<String>::new("projected.title", |_| Ok("value".into()));
        let edit = TextAction::new("edit.title", |_, _| Ok(()));
        let field = Field {
            label: "Projected title".into(),
            control: TextInput::computed(computed, edit),
        }
        .into_view();
        let editor = &field.children[1];
        assert_eq!(editor.node.action, edit.spec().id);
        assert!(
            matches!(&editor.text, Binding::ComputedEditor { value, label } if value.key == computed.key && label == "Projected title")
        );
    }
    #[test]
    fn renamed_select_field_retains_options_action_disabled_state_and_identity() {
        let make = |label: &str| {
            Field {
                label: label.into(),
                control: Select {
                    enabled: false,
                    ..Select::new(
                        "Old name",
                        State::<i64>::new("trip.kind", 7),
                        vec![SelectOption {
                            value: 7,
                            label: "Return".into(),
                            enabled: true,
                        }],
                        Action::new("trip.change", |_, _| Ok(())),
                    )
                },
            }
            .into_view()
        };
        let old = make("Trip");
        let renamed = make("Flight type");
        assert_eq!(old.node.key, renamed.node.key);
        assert_eq!(old.children[0].node.key, renamed.children[0].node.key);
        assert_eq!(old.children[1].node, renamed.children[1].node);
        let control = &renamed.children[1];
        assert_eq!(control.node.kind, abi::SELECT);
        assert_eq!(control.node.slot, abi::key("trip.kind"));
        assert_eq!(control.node.action, abi::key("trip.change"));
        assert_eq!(control.node.disabled, 1);
        let Binding::Static(wire) = &control.text else {
            panic!("select wire missing")
        };
        let (name, options) = abi::select::decode_control(wire).unwrap();
        assert_eq!(name, "Flight type");
        assert_eq!(options.len(), 1);
        assert_eq!(options[0].value, 7);
        assert_eq!(options[0].label, "Return");
        assert!(options[0].enabled);
    }
    #[test]
    fn readonly_multiline_field_keeps_control_contract() {
        let field = Field {
            label: "Response".into(),
            control: TextArea {
                read_only: true,
                ..TextArea::new(State::<String>::new("response"))
            },
        }
        .into_view();
        assert_eq!(field.children[1].node.kind, abi::TEXTAREA);
        assert_eq!(field.children[1].node.read_only, 1);
        assert_eq!(field.children[1].node.slot, abi::key("response"));
    }
}

#[cfg(all(test, feature = "macros"))]
mod node_identity_tests {
    use super::*;
    #[derive(Clone)]
    struct Caption(&'static str);
    impl IntoView for Caption {
        fn into_view(self) -> View {
            Text::new(self.0).into_view()
        }
    }

    #[blockui::view]
    fn labeled_button(ui: &Ui) -> View {
        #[node]
        {
            Button::label("fallback");
            #[node]
            Text::new("child label");
        }
    }
    #[test]
    fn button_scopes_consume_the_text_label_like_the_builder() {
        let view = labeled_button();
        assert_eq!(view.node.kind, abi::BUTTON);
        assert!(view.children.is_empty());
        assert!(matches!(&view.text, Binding::Static(label) if label == "child label"));
    }

    #[blockui::view]
    fn named_tree(reversed: bool, ui: &Ui) -> View {
        let title = Caption("Title");
        let subtitle = Caption("Subtitle");
        #[node]
        {
            Column::default();
            if reversed {
                #[node]
                subtitle.clone();
                #[node]
                title.clone();
            } else {
                #[node]
                title;
                #[node]
                subtitle;
            }
        }
    }

    #[test]
    fn marked_clones_keep_local_identity_through_reorder() {
        let before = named_tree(false).actor_root();
        let after = named_tree(true).actor_root();
        assert_ne!(before.children[0].node.key, before.children[1].node.key);
        assert_eq!(before.children[0].node.key, after.children[1].node.key);
        assert_eq!(before.children[1].node.key, after.children[0].node.key);
    }

    #[blockui::view]
    fn conditional_tree(ui: &Ui) -> View {
        #[node]
        {
            Column::default();
            #[cfg(any())]
            #[node]
            MissingLeaf::new();
            #[cfg(any())]
            #[node]
            {
                MissingContainer::new();
                #[node]
                MissingChild::new();
            }
            #[node]
            Text::new("active");
        }
    }

    #[test]
    fn rust_cfg_removes_whole_marked_statements() {
        assert_eq!(conditional_tree().children.len(), 1);
    }
}

mod icons;
pub use icons::{Icon, IconButton, IconSource};

mod suffixed_text;

mod conditional_text;

mod live;
pub use live::{Live, LiveEach, StyleBinding};

mod style_patch;
pub use style_patch::StylePatch;

mod async_value;
pub use async_value::{AsyncPhase, AsyncReadError, AsyncResult, AsyncResultExt, AsyncValue};

mod record_computed;
pub use record_computed::RecordComputed;
