# Host to JavaScript flow

Status: core bridge, application-producer lifecycle, generated capability
wrappers and record/replay tooling implemented.

## Decision

All unsolicited Rust/OS-originated information delivered to the guest uses one
versioned, recordable `HostEventFrame`. The guest exposes exactly one push entry
point:

```ts
function __wabou_dispatch_host_frame(frame: Uint8Array): HostFrameDisposition;
```

The result of a guest-initiated JSON capability returns through that capability
Promise; it is not turned into an unrelated Host event. JavaScript never runs
re-entrantly during style resolution, layout, text
measurement, painting, widget painting or scene submission. A frame is
dispatched only from an event-loop delivery point or immediately before the JS
tick. Mutations produced by its handlers are flushed after dispatch and applied
as a separate JS to Host mutation frame.

This replaced the former independent guest callbacks:

- `__wabou_dispatch_event`;
- `__wabou_resize_dispatch`;
- `__wabou_host_messages`.

HMR remains a development-runtime control channel and is not a UI host event.
The public `HostMessageHandle` producer API sends messages as
`ApplicationMessage` records in the unified frame. `HostMessageContext` binds
each producer to a window, its Tokio runtime and window-lifetime cancellation.

## Ownership boundary

The Host owns facts and mechanisms:

- OS/window lifecycle and input normalization;
- hit testing, target validation, focus and pointer capture;
- layout geometry and resize observations;
- native widget semantic events;
- native capabilities and their direct/native-Promise completion;
- queue bounds, coalescing and wake-up behavior.

The guest owns application policy:

- capture/target/bubble listener invocation;
- Solid signal updates;
- `preventDefault` and propagation flags;
- subscriptions to observations and application topics;
- Promise reactions for JSON capabilities.

JSX/Solid node IDs are runtime-local handles. Every received ID is validated by
the Host immediately before encoding. IDs must become generational before node
slots are reused; until then, IDs are never reused within one runtime.

## End-to-end sequence

```text
winit / widget / background producer
                 │
                 ▼
       normalize and target in Rust
                 │
                 ▼
     queue/coalesce HostEvent records
                 │
                 ▼
       encode one HostEventFrame
                 │
                 ▼
 __wabou_dispatch_host_frame(bytes)
                 │
                 ▼
  JS dispatch/subscribers/Promise settlement
                 │
                 ▼
     run bounded microtasks + one JS tick
                 │
                 ▼
       flush JS Mutation Frame once
                 │
                 ▼
     apply ops -> style -> layout -> paint
```

An input event may be delivered immediately from the winit callback because the
shell needs a disposition before applying a cancellable default action. It is
still encoded as a one-or-more-record `HostEventFrame`; it does not call a
separate JS function. Observation, application and lifecycle records
are normally batched before the next JS tick.

## Wire format

All integers are little-endian. Unknown frame versions are rejected as a whole.
Unknown record kinds in a known version are skipped using `record_len`.

```text
HostEventFrameHeader (32 bytes)
  u32 magic            = 0x31464857 ("WHF1")
  u16 version          = 2
  u16 flags
  u64 sequence
  u64 monotonic_time_ns
  u32 record_count
  u32 byte_len         including header

repeated record_count times
  u8  kind
  u8  flags
  u16 reserved         = 0
  u32 record_len       including the 8-byte record header
  u8[record_len - 8] payload
```

Limits are checked before allocation:

- maximum frame: 4 MiB;
- maximum records per frame: 512;
- maximum application message payload: 1 MiB;
- maximum string: 64 KiB UTF-8;
- a malformed/truncated frame dispatches nothing.

`sequence` starts at one per runtime and increments for every delivered frame,
including an immediate input frame. It is used for diagnostics, recording and
ordering assertions, not as a globally unique identifier.

## Record kinds and ordering

Version 2 currently encodes these records:

```rust
enum HostEvent {
    Node(NodeEvent),
    Resize(ResizeObservation),
    Application(ApplicationMessage),
}
```

Within a batch, records preserve the producer order assembled by the runtime.
Conceptually, producers are drained in these phases:

1. `Node` — pointer, key, text, focus, widget and scroll events;
2. `Resize` — final content-box observations after layout;
3. `Application` — user-defined Host messages.

Input sequences that have semantic ordering (`pointerdown`, focus changes,
`pointerup`, `click`) are emitted in that order and never phase-sorted relative
to one another.

### Node event

```text
u32 target_lo
u32 target_hi
u8  event_code         @wabou/core/protocol EVENT_CODE
u8  payload_kind       0 none, 1 numeric, 2 utf8-json
u16 reserved
u32 event_id           zero for non-cancellable events

numeric payload:
  f64 client_x
  f64 client_y
  f64 offset_x
  f64 offset_y
  f64 button
  f64 buttons
  f64 modifiers
  f64 delta_x
  f64 delta_y
  f64 scroll_x
  f64 scroll_y

json payload:
  u32 len + utf8 bytes
```

Numeric input stays allocation-free on the Rust side and avoids JSON parsing in
the guest. Keyboard/text payloads remain UTF-8 JSON in version 2; they may gain
typed records in a later protocol version without changing the frame envelope.
Propagation paths are not serialized: the guest renderer derives the path from
its retained logical parent map after the Host has selected the target.

### Resize observation

```text
u32 target_lo
u32 target_hi
f32 content_width
f32 content_height
```

Only subscribed, live nodes are encoded. One batch contains at most the final
size for a target.

### Application message

The existing topic plus typed-scalar payload representation is embedded without
JSON. Topics are application-level notifications, not internal input events.

```text
u16 topic_len + topic utf8
u8 payload_kind        null/bool/i32/f64/string/bytes
payload
```

## Dispatch disposition and default actions

The single guest function returns:

```ts
interface HostFrameDisposition {
  preventedEventIds?: Uint32Array;
  needsTick: boolean;
}
```

Only records marked `CANCELLABLE` receive a frame-local `event_id`. The guest
adds an ID when `preventDefault()` was called. Rust performs the default action
after dispatch unless that ID was returned. Propagation stopping is entirely a
guest concern and is not returned.

Version 2 cancellable defaults include link activation, context menus and
wheel/native scrolling. Focus selection and pointer capture are Host invariants,
not arbitrary guest defaults. Non-cancellable frames return an empty
disposition without allocating an array.

## Batching and coalescing

Coalescing occurs before encoding and never crosses semantic boundaries:

- pointer move: latest sample per pointer ID per delivery batch; raw/coalesced
  samples require an explicit subscription option;
- wheel: accumulate adjacent samples with the same target/modifiers and no
  intervening discrete input;
- window resize/scale: latest value, with change flags ORed;
- resize observation: latest value per node;
- widget progress/title: latest value per widget and semantic kind;
- application message: never coalesced by default; a future keyed-latest API is
  explicit rather than inferred from topic names.

Discrete input, key transitions, committed text, focus transitions, click and
close requests are never dropped.

## Backpressure

There are two queues:

1. a UI-thread queue for normalized input/observations;
2. a bounded multi-producer queue for background application data.

Rules:

- UI discrete events reserve capacity and cannot be displaced by application
  messages;
- coalescible events overwrite their keyed pending slot;
- `HostMessageHandle::send` remains non-blocking and returns `Full`;
- `send_timeout` is allowed only off the UI thread;
- the event loop is woken on the empty-to-non-empty transition, not once per
  message;
- at most 512 records or 4 MiB are delivered in one turn; remaining work keeps
  the runtime awake for another turn.

PTY byte streams do not use `ApplicationMessage`. Terminal bytes stay between
the PTY and the Rust terminal model/widget. JS receives semantic, low-volume
events such as title, bell and exit. A future JS-consumed byte stream must use a
separate bounded stream API with explicit credits.

## User Rust capabilities

User Rust APIs are mounted as named capabilities. Structured async operations
use `json_capability`, which matches the client generated by `wabou-bindgen` and
owns request decoding, Promise creation, and result-envelope encoding:

```rust
const READ_FILE: JsonMethod<ReadFileRequest, ReadFileResponse> =
    JsonMethod::new("readFile");
const WORKSPACE: JsonCapabilityContract = JsonCapabilityContract::new("workspace", 1);

HostBuilder::new().json_capability(WORKSPACE, |capability| {
    capability.method(READ_FILE, read_file)
});
```

The same `JsonCapabilityContract` is consumed by binding generation and host
registration. Generated clients check its ABI version before the first call,
so a stale frontend bundle reports `incompatibleHost` instead of failing later
with an undefined native method.

Application capabilities have one public registration path. They are
asynchronous JSON methods with explicit request and response DTOs; applications
cannot mount arbitrary `rquickjs::Function` values through `HostBuilder`.
Framework-owned synchronous functions use the separately declared native host
API, whose TypeScript signature is reviewed explicitly beside its Rust
implementation. Numeric effects remain private framework ABI for replayable OS
operations. There is no generic public `invoke(string, unknown)` entry point.

## Subscriptions and lifetime

Observation APIs register with the Host only on the first local subscriber and
unregister after the last subscriber:

```text
Solid owner cleanup
  -> remove local callback
  -> last callback?
  -> unsubscribe op to Host
```

Rust stores numeric subscription/target IDs, never JS function handles. Node
drop automatically removes resize, focus, capture and widget subscriptions.
Queued records targeting a dropped node are filtered immediately before frame
encoding. Runtime shutdown stops application producers and rejects pending
Promises with `HostClosedError` and disconnects producer handles.

## Re-entrancy and frame boundary invariants

These invariants are mandatory:

1. no guest call while Taffy/Parley/Vello or a widget is mutably borrowed;
2. no Host frame dispatch from inside `__wabou_flush`;
3. handlers may enqueue mutation ops, but Rust applies them only after guest
   dispatch returns and the bounded JS tick flushes;
4. Host events produced while dispatching are queued for a later frame;
5. at most one layout pass follows one applied mutation frame unless an
   explicitly bounded stabilization pass is required for resize observation;
6. resize observation follows layout and cannot synchronously cause recursive
   layout; its mutations apply on the next turn.

## Recording and replay

The encoded frame is the recording unit. A trace contains:

```text
runtime configuration hash
initial viewport/scale
ordered HostEventFrame bytes
ordered JS Mutation Frame bytes
optional computed-layout snapshots
```

Replay replaces winit/background producers, preserves frame sequences and feeds
the recorded bytes into the same guest entry point. Deterministic tests compare
mutation frames and selected computed-layout snapshots. Mounted capability calls
and returns are traced separately at the capability boundary when a test depends on
them; replay installs a mock capability. Timestamps may be normalized, but event
ordering and sequence numbers may not.

Required test cases:

- codec round trip and malformed-frame atomic rejection;
- protocol version rejection and unknown-record skipping;
- stable phase ordering;
- pointer move, wheel and resize coalescing;
- click/default-action cancellation exactly once;
- resize callback mutations apply on the next turn, without recursive layout;
- JSON capability operations resolve or reject exactly once;
- dropped targets and late async completions are ignored safely;
- bounded queues wake once and report backpressure;
- recorded frame replay produces byte-identical mutation frames;
- theme, resize and input sequences preserve expected layout snapshots.

## Maintenance rule

One subsystem must use exactly one route; dual delivery is a test failure.

## Completion criteria

The design is implemented when all of the following are true:

- production Host-to-guest UI data enters through
  `__wabou_dispatch_host_frame` only;
- node events, resize observations and Host messages use the versioned envelope;
- legacy event/resize/message callbacks and shared numeric storage are removed;
- queue bounds, coalescing and mounted-operation cancellation match this
  document;
- no dispatch occurs inside layout/paint or recursively from a resize callback;
- Rust and TypeScript codec fixtures are cross-compatible;
- the required ordering/backpressure/replay tests pass;
- application JS APIs use generated JSON capability clients; framework-owned
  synchronous calls use the explicit native host contract. There is no public
  global `Wabou`, raw QuickJS mount API, numeric effect dispatcher or generic
  stringly typed invocation surface.
