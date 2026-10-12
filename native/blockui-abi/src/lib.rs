//! C ABI and a small typed SDK. Every pointer is borrowed for one synchronous call.
use std::{ffi::c_void, marker::PhantomData};
pub mod keyed_text;
pub mod semantics;
pub mod theme;

/// Stable scalar request failure codes. Only their i64 code crosses the message ABI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i64)]
pub enum RequestError {
    Timeout = 1,
    TargetTerminated = 2,
    RuntimeStopped = 3,
    Cancelled = 4,
    HandlerFailed = 5,
}
impl RequestError {
    pub const fn code(self) -> i64 {
        self as i64
    }
    pub const fn from_code(code: i64) -> Option<Self> {
        match code {
            1 => Some(Self::Timeout),
            2 => Some(Self::TargetTerminated),
            3 => Some(Self::RuntimeStopped),
            4 => Some(Self::Cancelled),
            5 => Some(Self::HandlerFailed),
            _ => None,
        }
    }
}

pub mod canvas;
pub mod grid;
pub mod icon;
pub mod select;
pub const ABI_VERSION: u32 = 109;
/// Missing keyed-map entry; distinct from an empty value and host errors.
pub const MAP_ABSENT: isize = -2;
/// Admission budget for ordinary and expanded native views.
pub const MAX_VIEW_NODES: usize = 4096;
/// Maximum UTF-8 bytes in one ordinary text message.
pub const MAX_MESSAGE_TEXT_BYTES: usize = 64 * 1024;
pub const I64: u32 = 1;
pub const TEXT: u32 = 2;
pub const BOOLEAN: u32 = 3;
pub const I64_LIST: u32 = 4;
pub const TEXT_LIST: u32 = 6;
pub const F64: u32 = 5;
pub const MAX_LIST_ITEMS: usize = 4096;
pub const MAX_TEXT_LIST_BYTES: usize = 1024 * 1024;
pub const COLUMN: u32 = 1;
pub const ROW: u32 = 8;
pub const CHECKBOX: u32 = 9;
pub const SLIDER: u32 = 10;
pub const PROGRESS: u32 = 11;
pub const CANVAS: u32 = 12;
pub const DIALOG: u32 = 13;
pub const SELECT: u32 = 14;
pub const ICON: u32 = 15;
pub const LABEL: u32 = 2;
pub const BUTTON: u32 = 3;
pub const ENTRY: u32 = 4;
pub const TEXTAREA: u32 = 7;
pub const SCROLL: u32 = 5;
/// Child implementation identity is encoded in Node.action; Node.key is its scoped identity.
pub const CHILD: u32 = 6;
pub const OK: i32 = 0;
pub const ERROR: i32 = -1;

pub const fn key(name: &str) -> u64 {
    let bytes = name.as_bytes();
    let mut hash = 0xcbf29ce484222325_u64;
    let mut i = 0;
    while i < bytes.len() {
        hash = (hash ^ bytes[i] as u64).wrapping_mul(0x100000001b3);
        i += 1;
    }
    hash
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Bytes {
    pub ptr: *const u8,
    pub len: usize,
}
impl Bytes {
    pub const fn new(value: &str) -> Self {
        Self {
            ptr: value.as_ptr(),
            len: value.len(),
        }
    }
}
// Immutable static descriptors contain only pointers to immutable static bytes.
unsafe impl Sync for Bytes {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SlotSpec {
    pub key: u64,
    pub kind: u32,
    pub default_i64: i64,
    pub default_text: Bytes,
}
impl SlotSpec {
    pub const fn integer(name: &str, value: i64) -> Self {
        Self {
            key: key(name),
            kind: I64,
            default_i64: value,
            default_text: Bytes {
                ptr: b"".as_ptr(),
                len: 0,
            },
        }
    }
    pub const fn float(name: &str, value: f64) -> Self {
        Self {
            kind: F64,
            ..Self::integer(name, value.to_bits() as i64)
        }
    }
    pub const fn boolean(name: &str, value: bool) -> Self {
        Self {
            kind: BOOLEAN,
            ..Self::integer(name, value as i64)
        }
    }
    pub const fn integer_list(name: &str, default: &'static [i64]) -> Self {
        Self {
            key: key(name),
            kind: I64_LIST,
            default_i64: 0,
            default_text: Bytes {
                ptr: default.as_ptr().cast(),
                len: default.len() * 8,
            },
        }
    }
    pub const fn text_list(name: &str) -> Self {
        Self::text_list_default(name, &[])
    }
    /// Defaults are static C-layout descriptors, never Rust str/Vec layouts.
    pub const fn text_list_default(name: &str, default: &'static [Bytes]) -> Self {
        Self {
            kind: TEXT_LIST,
            default_text: Bytes {
                ptr: default.as_ptr().cast(),
                len: std::mem::size_of_val(default),
            },
            ..Self::text(name)
        }
    }
    pub const fn text(name: &str) -> Self {
        Self::text_default(name, "")
    }
    pub const fn text_default(name: &str, default: &'static str) -> Self {
        Self {
            key: key(name),
            kind: TEXT,
            default_i64: 0,
            default_text: Bytes {
                ptr: default.as_ptr(),
                len: default.len(),
            },
        }
    }
}

/// Local logical-pixel canvas coordinates. Encoded into an existing scalar message.
/// Both components must be finite; their IEEE f32 bits are preserved exactly.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalPoint {
    pub x: f32,
    pub y: f32,
}
impl LocalPoint {
    pub fn to_wire(self) -> Option<i64> {
        (self.x.is_finite() && self.y.is_finite())
            .then(|| ((u64::from(self.y.to_bits()) << 32) | u64::from(self.x.to_bits())) as i64)
    }
    pub fn from_wire(value: i64) -> Option<Self> {
        let point = Self {
            x: f32::from_bits(value as u32),
            y: f32::from_bits((value as u64 >> 32) as u32),
        };
        (point.x.is_finite() && point.y.is_finite()).then_some(point)
    }
}

/// Exact integer range for a native slider. No plugin-owned allocation.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IntegerRange {
    pub min: i32,
    pub max: i32,
    pub step: i32,
}
impl IntegerRange {
    pub fn is_valid(self) -> bool {
        self.min < self.max
            && self.step > 0
            && self.min >= -8_388_608
            && self.max <= 8_388_608
            && (i64::from(self.max) - i64::from(self.min)) % i64::from(self.step) == 0
    }
    pub fn contains(self, value: i64) -> bool {
        self.is_valid()
            && value >= i64::from(self.min)
            && value <= i64::from(self.max)
            && (value - i64::from(self.min)) % i64::from(self.step) == 0
    }
}

/// Data-only presentation, independent of durable state and widget identity.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Presentation {
    pub width: f32,
    pub max_width: f32,
    pub font_size: f32,
    /// Label color role: 0 default, 1 muted, 2 accent, 3 error.
    pub text_tone: u32,
    pub surface: u32,
    pub radius: f32,
    /// Inherited scope: 0 inherit, 1 light, 2 dark, 3 system.
    pub theme: u32,
    /// Inherited control density: 0 inherit, 1 comfortable, 2 compact.
    pub density: u32,
    /// Layout containers/scroll ports: fill the remaining available height (0/1).
    pub fill_height: u32,
}
impl Presentation {
    /// Fixed data-only wire format; no padding, pointers, or Rust allocations.
    pub const WIRE_BYTES: usize = 36;
    pub fn encode(self) -> [u8; Self::WIRE_BYTES] {
        let words = [
            self.width.to_bits(),
            self.max_width.to_bits(),
            self.font_size.to_bits(),
            self.text_tone,
            self.surface,
            self.radius.to_bits(),
            self.theme,
            self.density,
            self.fill_height,
        ];
        let mut bytes = [0; Self::WIRE_BYTES];
        for (chunk, word) in bytes.as_chunks_mut::<4>().0.iter_mut().zip(words) {
            chunk.copy_from_slice(&word.to_le_bytes());
        }
        bytes
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_BYTES {
            return None;
        }
        let mut words = [0; 9];
        for (word, chunk) in words.iter_mut().zip(bytes.as_chunks::<4>().0.iter()) {
            *word = u32::from_le_bytes(*chunk);
        }
        Some(Self {
            width: f32::from_bits(words[0]),
            max_width: f32::from_bits(words[1]),
            font_size: f32::from_bits(words[2]),
            text_tone: words[3],
            surface: words[4],
            radius: f32::from_bits(words[5]),
            theme: words[6],
            density: words[7],
            fill_height: words[8],
        })
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Node {
    pub key: u64,
    pub parent: u64,
    pub kind: u32,
    /// Semantic binding identity; zero opts out of reuse when a node is re-emitted.
    pub binding: u64,
    /// Lowercase ASCII letter for a primary-modifier shortcut; zero means absent.
    pub shortcut: u32,
    /// Button/checkbox/text editor disabled state: 0 enabled, 1 disabled. Other kinds require zero.
    pub disabled: u32,
    /// Text input/area read-only flag (0/1); other kinds require zero.
    pub read_only: u32,
    /// Button: 0 primary, 1 secondary, 2 danger. Badge label: 0 default,
    /// 1 secondary, 2 outline, 3 destructive. Other nodes require zero.
    pub button_variant: u32,
    pub presentation: Presentation,
    pub semantics: semantics::Semantics,
    pub action: u64,
    /// Canvas secondary-pointer action; other kinds require zero.
    pub secondary_action: u64,
    /// Single-line editor Enter/Escape actions; other nodes require zero.
    pub submit_action: u64,
    pub cancel_action: u64,
    pub grid_rows: u32,
    pub grid_columns: u32,
    pub navigate_action: u64,
    /// Scroll offset observer, encoded as LocalPoint; scroll containers only.
    pub scroll_action: u64,
    /// Measured scroll-port width/height observer, encoded as LocalPoint.
    pub viewport_action: u64,
    /// A declarative focus/restoration scope on a layout container.
    pub focus_boundary: u32,
    /// Button argument (i64/bool/f64 bits), or toast viewport paused state (0/1).
    /// Toast viewport actions observe aggregate activity; other kinds require zero.
    pub action_payload: i64,
    pub slot: u64,
    pub gap: f32,
    pub padding: f32,
    pub height: f32,
    /// Scroll content width in logical pixels; zero follows viewport width.
    pub content_width: f32,
    /// Slider only; all other nodes require the zero default.
    pub range: IntegerRange,
    /// Toast viewport position: 0 bottom-right, 1 bottom-left, 2 bottom-center,
    /// 3 top-right, 4 top-left, 5 top-center.
    /// Popover panel: side*3+alignment (0..=11), sides bottom/top/right/left,
    /// alignment start/center/end. Other nodes require zero.
    pub overlay_position: u32,
    /// Tooltip label anchor; zero on other nodes.
    pub tooltip_anchor: u64,
    /// Tooltip hover delay (0..=10000 ms); zero on other nodes.
    pub tooltip_delay_ms: u32,
}
pub type Emit = unsafe extern "C" fn(*mut c_void, Node) -> i32;
pub type Write = unsafe extern "C" fn(*mut c_void, Bytes) -> i32;
/// Binding recipes are borrowed during view emission. Tokens belong to this DLL.
pub const BIND_STATIC: u32 = 0;
pub const BIND_REACTIVE: u32 = 1;
/// Editor value from `slot`; `text` carries an optional accessible name (ABI 38).
pub const BIND_TEXT_SLOT: u32 = 2;
/// Editor value from a registered text computation; edits dispatch its text
/// action without writing a state slot. text carries its accessible name.
pub const BIND_COMPUTED_EDITOR: u32 = 21;
/// Text record projection: slot is record storage; text contains format, ordinal,
/// and accessible label separated by two newlines. No executable token.
pub const BIND_RECORD_TEXT: u32 = 22;
/// Read-only scalar field text. Same selector wire as BIND_RECORD_TEXT, with an
/// empty label. Token is a format kind below, never an executable address.
pub const BIND_RECORD_SCALAR: u32 = 23;
/// Controlled boolean record field; selector and label use the record text wire.
pub const BIND_RECORD_CHECKBOX: u32 = 24;
/// Controlled integer record slider; selector/label use the record text wire.
pub const BIND_RECORD_SLIDER: u32 = 25;
/// Read-only fraction field; selector/label use the record text wire.
pub const BIND_RECORD_PROGRESS: u32 = 26;
/// Integer record selection; record selector followed by canonical select wire.
pub const BIND_RECORD_SELECT: u32 = 27;
/// Selected collection text: format, ordinal and key byte length separated by
/// newlines, then key bytes followed by an optional editor label. No token.
pub const BIND_RECORD_MAP_TEXT: u32 = 28;
/// Controlled collection editor using a keyed-text message to its action.
pub const BIND_RECORD_MAP_KEYED_TEXT: u32 = 29;
pub const RECORD_DISPLAY_I64: usize = 1;
pub const RECORD_DISPLAY_U64: usize = 2;
pub const RECORD_DISPLAY_F64: usize = 3;
pub const RECORD_DISPLAY_BOOL: usize = 4;
pub const BIND_CHECKBOX: u32 = 3;
pub const BIND_SLIDER: u32 = 17;
pub const BIND_PROGRESS: u32 = 18;
pub const BIND_CANVAS: u32 = 19;
pub const BIND_SELECT: u32 = 20;
pub const BIND_MAP_I64: u32 = 4;
pub const BIND_MAP_F64: u32 = 5;
pub const BIND_MAP_BOOL: u32 = 6;
pub const BIND_MAP_TEXT: u32 = 7;
pub const BIND_MAP_LIST_I64: u32 = 8;
pub const BIND_COMPUTED_TEXT: u32 = 9;
pub const BIND_COMPUTED_I64: u32 = 10;
pub const BIND_COMPUTED_F64: u32 = 11;
pub const BIND_COMPUTED_BOOL: u32 = 12;
/// Plugin-local style registry token (ABI 107), never a pointer to a capture.
/// Emitted after the node's ordinary recipe.
/// render_binding returns Presentation::encode() bytes, copied during the call.
pub const BIND_STYLE: u32 = 30;
/// Data-only icon decoration; follows text and precedes style.
pub const BIND_ICON: u32 = 31;
/// Plugin-local captured text registry token; not a callback or allocation pointer.
pub const BIND_CAPTURED_TEXT: u32 = 32;
/// Implementation-local structural callback; slot is identity, token is opaque.
pub const BIND_CAPTURED_SCOPE: u32 = 33;
pub const BIND_SCOPE: u32 = 13;
pub const BIND_MAP_TEXT_LIST: u32 = 14;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BindingSpec {
    pub key: u64,
    pub kind: u32,
    pub token: usize,
    pub slot: u64,
    pub text: Bytes,
}
pub type EmitBinding = unsafe extern "C" fn(*mut c_void, BindingSpec) -> i32;

#[repr(C)]
pub struct HostApi {
    /// Current implementation instance; data only, never a context pointer.
    pub event_owner: u64,
    /// Host-owned application service; request and output are borrowed for this call.
    /// Never retains UI pointers. A negative result reports failure.
    pub call_service: unsafe extern "C" fn(*mut c_void, u64, Bytes, *mut u8, usize) -> isize,
    pub read_f64: unsafe extern "C" fn(*mut c_void, u64, *mut f64) -> i32,
    pub write_f64: unsafe extern "C" fn(*mut c_void, u64, f64) -> i32,
    pub previous_f64: unsafe extern "C" fn(*mut c_void, u64, *mut f64) -> i32,
    pub computed_f64: unsafe extern "C" fn(*mut c_void, u64, *mut f64) -> i32,
    pub ask_actor: unsafe extern "C" fn(*mut c_void, u64, u64, i64, u64, u64, u64, *mut u64) -> i32,
    pub cancel_request: unsafe extern "C" fn(*mut c_void, u64) -> i32,
    /// Stage one scalar reply during a request handler; host publishes after commit.
    pub reply: unsafe extern "C" fn(*mut c_void, i64) -> i32,
    pub child_actor: unsafe extern "C" fn(*mut c_void, u64, *mut u64) -> i32,
    pub send_actor: unsafe extern "C" fn(*mut c_void, u64, u64, i64) -> i32,
    pub context: *mut c_void,
    pub read_i64: unsafe extern "C" fn(*mut c_void, u64, *mut i64) -> i32,
    pub write_i64: unsafe extern "C" fn(*mut c_void, u64, i64) -> i32,
    pub read_text: unsafe extern "C" fn(*mut c_void, u64, *mut u8, usize) -> isize,
    pub write_text: unsafe extern "C" fn(*mut c_void, u64, Bytes) -> i32,
    pub computed_i64: unsafe extern "C" fn(*mut c_void, u64, *mut i64) -> i32,
    pub computed_list: unsafe extern "C" fn(*mut c_void, u64, *mut i64, usize) -> isize,
    /// Indexed UTF-8 collection read; usize::MAX queries the item count.
    pub computed_text_list: unsafe extern "C" fn(*mut c_void, u64, usize, *mut u8, usize) -> isize,
    pub computed_text: unsafe extern "C" fn(*mut c_void, u64, *mut u8, usize) -> isize,
    pub delay: unsafe extern "C" fn(*mut c_void, u64, u64, u64, i64, u32) -> i32,
    pub interval: unsafe extern "C" fn(*mut c_void, u64, u64, u64, i64, u32) -> i32,
    pub cancel_task: unsafe extern "C" fn(*mut c_void, u64) -> i32,
    pub open_file: unsafe extern "C" fn(*mut c_void, u64, Bytes, u32, u32) -> i32,
    pub close_file: unsafe extern "C" fn(*mut c_void, u64) -> i32,
    pub read_file: unsafe extern "C" fn(*mut c_void, u64, u64, *mut u8, usize) -> isize,
    pub connect_socket_async:
        unsafe extern "C" fn(*mut c_void, u64, u64, Bytes, u64, u64, u32, u32) -> i32,
    pub read_socket_async: unsafe extern "C" fn(*mut c_void, u64, u64, usize, u64, u64, u32) -> i32,
    pub write_socket_async:
        unsafe extern "C" fn(*mut c_void, u64, u64, Bytes, u64, u64, u32) -> i32,
    pub close_socket: unsafe extern "C" fn(*mut c_void, u64) -> i32,
    pub create_file_async: unsafe extern "C" fn(*mut c_void, u64, Bytes, Bytes, u64, u32) -> i32,
    pub save_file_async: unsafe extern "C" fn(*mut c_void, u64, u64, Bytes, u64, u32) -> i32,
    pub write_file_async: unsafe extern "C" fn(*mut c_void, u64, u64, u64, Bytes, u64, u32) -> i32,
    pub read_file_async: unsafe extern "C" fn(*mut c_void, u64, u64, u64, usize, u64, u32) -> i32,
    pub task_result: unsafe extern "C" fn(*mut c_void, u64, *mut u8, usize, *mut u32) -> isize,
    pub previous_i64: unsafe extern "C" fn(*mut c_void, u64, *mut i64) -> i32,
    pub previous_text: unsafe extern "C" fn(*mut c_void, u64, *mut u8, usize) -> isize,
    /// Counts are i64 items; all buffers are borrowed for this call only.
    pub read_list: unsafe extern "C" fn(*mut c_void, u64, *mut i64, usize) -> isize,
    pub write_list: unsafe extern "C" fn(*mut c_void, u64, *const i64, usize) -> i32,
    pub previous_list: unsafe extern "C" fn(*mut c_void, u64, *mut i64, usize) -> isize,
    /// Check exact live slot kind without reading its value or tracking a dependency.
    pub validate_state: unsafe extern "C" fn(*mut c_void, u64, u32) -> i32,
    /// index == usize::MAX queries item count; other indices read one UTF-8 item.
    /// Canonical sorted text map lookup. Tracks this exact key, including absence.
    /// Null output queries bytes; MAP_ABSENT denotes a missing key.
    pub read_text_map: unsafe extern "C" fn(*mut c_void, u64, Bytes, *mut u8, usize) -> isize,
    pub read_text_list: unsafe extern "C" fn(*mut c_void, u64, usize, *mut u8, usize) -> isize,
    /// Every descriptor and its bytes are borrowed only until the callback returns.
    pub write_text_list: unsafe extern "C" fn(*mut c_void, u64, *const Bytes, usize) -> i32,
    pub previous_text_list: unsafe extern "C" fn(*mut c_void, u64, usize, *mut u8, usize) -> isize,
    /// Clip [start, end) to the list; null output queries the clipped count.
    /// All buffers are borrowed only for this call. Reversed ranges are invalid.
    pub read_list_range:
        unsafe extern "C" fn(*mut c_void, u64, usize, usize, *mut i64, usize) -> isize,
    pub computed_list_range:
        unsafe extern "C" fn(*mut c_void, u64, usize, usize, *mut i64, usize) -> isize,
    /// Copy a synchronous callback diagnostic into host ownership (UTF-8, at most 4096 bytes).
    /// Does not publish application state or transfer ownership of the borrowed bytes.
    pub report_error: unsafe extern "C" fn(*mut c_void, Bytes) -> i32,
    /// Current request-error completion text. Null output queries length; zero means absent.
    pub read_request_error: unsafe extern "C" fn(*mut c_void, *mut u8, usize) -> isize,
    /// Copy borrowed UTF-8 into a transactional host-owned outgoing message.
    pub send_actor_text: unsafe extern "C" fn(*mut c_void, u64, u64, Bytes) -> i32,
    /// Current message text: negative means absent/error, zero is valid empty text.
    /// Null output queries length; insufficient capacity does not write output.
    pub read_message_text: unsafe extern "C" fn(*mut c_void, *mut u8, usize) -> isize,
    /// Stage a borrowed UTF-8 reply (at most MAX_MESSAGE_TEXT_BYTES) in a text request turn.
    /// Copies into host ownership; only published after the actor turn commits.
    pub reply_text: unsafe extern "C" fn(*mut c_void, Bytes) -> i32,
    /// Process-local monotonic milliseconds, only during a writable actor turn.
    /// Epoch belongs to the host executable and survives implementation replacement.
    pub monotonic_ms: unsafe extern "C" fn(*mut c_void, *mut u64) -> i32,
    /// Stage a transactional text request. Bytes are copied into host ownership.
    pub ask_actor_text:
        unsafe extern "C" fn(*mut c_void, u64, u64, Bytes, u64, u64, u64, *mut u64) -> i32,
    /// Selected length-delimited record field; copies bytes, tracks only this field.
    pub read_record_field:
        unsafe extern "C" fn(*mut c_void, u64, Bytes, usize, *mut u8, usize) -> isize,
    /// Selected field in a keyed record collection. MAP_ABSENT denotes no row.
    /// Tracks key/format/ordinal independently of whole-row reads.
    pub read_record_map_field:
        unsafe extern "C" fn(*mut c_void, u64, Bytes, Bytes, usize, *mut u8, usize) -> isize,
}

/// A derived computation descriptor contains no executable pointer or durable state.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ComputedSpec {
    pub key: u64,
    pub kind: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Output {
    pub kind: u32,
    pub integer: i64,
    pub text: Bytes,
}
pub type EmitScope = unsafe extern "C" fn(*mut c_void, u64) -> i32;

pub type EmitComputed = unsafe extern "C" fn(*mut c_void, ComputedSpec) -> i32;
pub type WriteValue = unsafe extern "C" fn(*mut c_void, Output) -> i32;

pub const PANIC_UNWIND: u32 = 1;
pub const PANIC_ABORT: u32 = 2;

/// Stable prefix checked before interpreting a version-specific function table.
#[repr(C)]
pub struct ImplementationHeader {
    pub abi_version: u32,
    pub struct_size: usize,
}

#[repr(C)]
pub struct Implementation {
    pub abi_version: u32,
    pub struct_size: usize,
    /// SDK build policy; the reload host requires unwind for candidate recovery.
    pub panic_strategy: u32,
    pub slots: *const SlotSpec,
    pub slot_count: usize,
    /// 1 begin, 2 commit, 3 abort, 4 dispose; 5 resets the supplied scope,
    /// 6 retains live actions, 7 retains live text/style registry tokens. IDs are borrowed.
    pub events: unsafe extern "C" fn(u64, u32, *const u64, usize) -> i32,
    pub handle: unsafe extern "C" fn(*const HostApi, u64, i64) -> i32,
    pub view: unsafe extern "C" fn(*const HostApi, *mut c_void, Emit, EmitBinding) -> i32,
    pub render_binding:
        unsafe extern "C" fn(*const HostApi, u32, u64, usize, *mut c_void, Write) -> i32,
    pub migrate: Option<unsafe extern "C" fn(*const HostApi, u64) -> i32>,
    pub scope_ids: unsafe extern "C" fn(*mut c_void, EmitScope) -> i32,
    pub scope_view:
        unsafe extern "C" fn(*const HostApi, u64, *mut c_void, Emit, EmitBinding) -> i32,
    pub computed_specs: unsafe extern "C" fn(*mut c_void, EmitComputed) -> i32,
    pub compute: unsafe extern "C" fn(*const HostApi, u64, *mut c_void, WriteValue) -> i32,
}
// A plugin's function table and schema are immutable for its loaded lifetime.
unsafe impl Sync for Implementation {}

/// A typed reference to a host-owned slot; it never owns a plugin allocation.
pub struct State<'a, T> {
    api: &'a HostApi,
    key: u64,
    marker: PhantomData<T>,
}
impl HostApi {
    pub fn state<T>(&self, name: &str) -> State<'_, T> {
        self.state_key(key(name))
    }
    /// Borrow a host slot by an already scoped semantic identity.
    pub fn state_key<T>(&self, key: u64) -> State<'_, T> {
        State {
            api: self,
            key,
            marker: PhantomData,
        }
    }
}
impl State<'_, i64> {
    pub fn get(&self) -> Result<i64, i32> {
        let mut value = 0;
        let code = unsafe { (self.api.read_i64)(self.api.context, self.key, &mut value) };
        if code == OK { Ok(value) } else { Err(code) }
    }
    pub fn set(&self, value: i64) -> Result<(), i32> {
        let code = unsafe { (self.api.write_i64)(self.api.context, self.key, value) };
        if code == OK { Ok(()) } else { Err(code) }
    }
}
impl State<'_, String> {
    pub fn get(&self) -> Result<String, i32> {
        let length =
            unsafe { (self.api.read_text)(self.api.context, self.key, std::ptr::null_mut(), 0) };
        if length < 0 {
            return Err(ERROR);
        }
        let mut bytes = vec![0; length as usize];
        let written = unsafe {
            (self.api.read_text)(self.api.context, self.key, bytes.as_mut_ptr(), bytes.len())
        };
        if written != length {
            return Err(ERROR);
        }
        String::from_utf8(bytes).map_err(|_| ERROR)
    }
    pub fn set(&self, value: &str) -> Result<(), i32> {
        let code = unsafe { (self.api.write_text)(self.api.context, self.key, Bytes::new(value)) };
        if code == OK { Ok(()) } else { Err(code) }
    }
}

#[cfg(test)]
mod request_error_tests {
    use super::RequestError;
    #[test]
    fn request_error_wire_codes_are_closed_and_stable() {
        let errors = [
            RequestError::Timeout,
            RequestError::TargetTerminated,
            RequestError::RuntimeStopped,
            RequestError::Cancelled,
            RequestError::HandlerFailed,
        ];
        for (index, error) in errors.into_iter().enumerate() {
            assert_eq!(error.code(), index as i64 + 1);
            assert_eq!(RequestError::from_code(error.code()), Some(error));
        }
        for code in [i64::MIN, -1, 0, 6, i64::MAX] {
            assert_eq!(RequestError::from_code(code), None);
        }
    }
}

#[cfg(test)]
mod local_point_tests {
    use super::*;
    #[test]
    fn coordinates_round_trip_ieee_bits_and_reject_nonfinite_values() {
        for point in [
            LocalPoint { x: -0.0, y: 0.0 },
            LocalPoint { x: -12.5, y: 20.25 },
            LocalPoint {
                x: f32::MAX,
                y: f32::MIN,
            },
        ] {
            let restored = LocalPoint::from_wire(point.to_wire().unwrap()).unwrap();
            assert_eq!(restored.x.to_bits(), point.x.to_bits());
            assert_eq!(restored.y.to_bits(), point.y.to_bits());
        }
        for point in [
            LocalPoint {
                x: f32::NAN,
                y: 0.0,
            },
            LocalPoint {
                x: 0.0,
                y: f32::INFINITY,
            },
        ] {
            assert_eq!(point.to_wire(), None);
            let malformed =
                ((u64::from(point.y.to_bits()) << 32) | u64::from(point.x.to_bits())) as i64;
            assert_eq!(LocalPoint::from_wire(malformed), None);
        }
    }
}
