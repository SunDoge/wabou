# Runtime boundary contract

Status: accepted. This document is normative for new cross-language and
resource APIs.

## QuickJS resource limits

`HostBuilder` uses a 6 MiB QuickJS stack by default. Applications with unusually
deep generated component trees or large third-party bundles can opt into a
larger limit before starting the host:

```rust
HostBuilder::new()
    .quickjs_stack_size(8 * 1024 * 1024)
    .run()?;
```

Pure-library compatibility tests use the same `JsRuntimeOptions` type through
`JsRuntime::new_with_options`, so a probe and the eventual application can run
under identical limits. Keep the configured limit below the native thread
stack; an arbitrarily large value does not create more native stack. When a
bundle exhausts this limit during boot, Wabou reports the configured limit with
the JavaScript exception and mapped stack trace.

Wabou keeps transport, application APIs, and resource lifetime as separate
concerns. New features must use one of the existing mechanisms below instead
of adding another bridge.

## Communication mechanisms

### Frame protocol

The binary frame protocol is Wabou's data plane. It carries work that is
frequent, naturally batchable, or produced once per input/render cycle:

- JavaScript to Rust retained-tree mutations;
- Rust to JavaScript input, window, widget, resource, and application events.

A frame contains versioned opcodes or records and is submitted across the
engine boundary once per batch. A high-frequency property update must not
become one FFI call per property. JavaScript encodes the updates first and a
private native intrinsic submits the complete frame.

Binary formats use explicit little-endian fields, fixed-width integers,
declared limits, and whole-frame validation. Unknown versions fail atomically;
record formats that permit skipping include their encoded length.

### Protocol evolution

An opcode number and its payload layout become immutable once checked in.
Adding a field to an existing behavior must use a new opcode or a separately
versioned, length-delimited record; it must not append bytes to the old payload.
Removed opcode numbers remain tombstones and are never reused. Increment the
whole-frame version only for an intentional protocol-wide break, not as a way
to hide an otherwise avoidable operation-level incompatibility.

Every payload affected by an evolution must retain at least one checked-in
golden frame under `fixtures/protocol`. The TypeScript writer must reproduce
the exact bytes and the Rust decoder must decode those same bytes. Unit tests
that independently construct equivalent-looking frames are useful for input
validation, but do not by themselves prove that the two implementations share
one stable wire contract.

### Native intrinsics

Native intrinsics are the small private FFI surface required to boot and drive
the runtime. They are appropriate only when an operation:

- submits or drains a frame;
- must return synchronously before JavaScript can continue;
- adapts a JavaScript-engine primitive such as UTF-8 encoding or string
  interning.

They are not an application extension API. Adding a direct intrinsic merely to
avoid defining a capability is a boundary violation. Framework-owned native
effects are implementation details of typed services such as clipboard,
dialog, notification, and window APIs; their numeric ABI is not a fourth
application-visible communication mechanism.

`packages/core/effect-abi.json` is the sole source for the Effect ABI version
and operation identifiers. Code generation produces both the Rust constants
used by `wabou-shell` and the TypeScript constants used by `@wabou/core`.

### Native capabilities

Native capabilities are the direct structured-value control path for
application-owned Rust APIs. A typed `HostMethod<Request, Response>` is mounted
through `HostBuilder::native_capability`; Serde converts QuickJS objects
directly without an intermediate JSON string and asynchronous Rust handlers
become JavaScript Promises.

Use native capabilities for hot request/response calls or stable typed object
operations where JSON encoding is measurable or unnecessarily duplicates an
already in-process object representation. Public DTO declarations come from
Specta, while the flat function names and sync/async behavior remain explicit
in a `FunctionModule`. Native capability methods are not per-frame rendering
operations and must not replace batching.

### JSON capabilities

JSON capabilities are Wabou's control plane for application-owned, low-rate,
asynchronous request/response operations. They provide named, versioned,
generated contracts and Promise completion without exposing QuickJS values to
application Rust code.

Use a capability for database operations, configuration, account services, or
other business requests. A long-running producer starts or configures through
a capability and publishes subsequent values through the ordinary host event
frame. It must not create a new callback channel.

JSON is chosen for contract clarity and diagnostics, not for per-frame data.
If measurement shows that JSON conversion is a bottleneck for an otherwise
request/response-shaped operation, move that operation to a native capability.
If the work is frequent and naturally batchable, define a binary frame record
instead.

## Resource identity and lifetime

Long-lived Rust-owned resources use typed generational registries. SlotMap is
the default implementation when resources can be created and removed in an
arbitrary order:

```rust
slotmap::new_key_type! {
    struct ImageKey;
    struct FontKey;
}
```

Each resource family owns its own map. Do not replace these with one
heterogeneous `Resource` enum or accept a key from one family in another.
JavaScript owns no Rust object; it holds only an opaque handle. Removing a
resource invalidates stale handles through the registry generation.

Tree nodes are governed by the retained-tree identity contract and do not gain
a redundant Rust SlotMap layer. Their JavaScript allocator nevertheless uses a
full-width generational `NodeKey`, transported as two `u32` fields just like a
SlotMap key. Rust maps that key to Taffy's independently generational `NodeId`.
Native widget instances are node-owned: one widget exists for one widget node,
is looked up by the Taffy `NodeId`, and is unmounted and removed with that node.
They therefore stay in a node-keyed map instead of receiving an independent
`WidgetKey`. A SlotMap becomes appropriate only if a future native object can
outlive, move independently of, or be shared by multiple nodes.

## Wire handles

A generational key crosses a Wabou binary protocol as two little-endian `u32`
fields:

```text
u32 handle_lo
u32 handle_hi
```

Rust reconstructs the value as:

```rust
let raw = u64::from(handle_lo) | (u64::from(handle_hi) << 32);
```

For SlotMap, encode and decode only through `KeyData::as_ffi` and
`KeyData::from_ffi`. The Wabou contract names the fields `lo` and `hi`; it does
not expose or promise SlotMap's internal slot/generation bit layout. This
avoids JavaScript safe-integer loss without requiring BigInt and preserves the
full generational key.

Retained `NodeKey` values use the same wire shape. Both fields are part of the
identity: protocol maps, event routing, resize observations, layout queries and
imperative node methods must compare and transport the complete pair. Code may
index an internal table by `lo` only as an optimization, but must validate the
stored `hi` before reading, mutating or dispatching anything. `hi` must not
silently wrap onto a live or stale key.

The pair representation was measured with the existing DataView-style binary
path over 5,000 nodes. A representative creation frame grew from 200 KB to
280 KB while encode/decode changed from 0.0535/0.0725 ms to 0.0706/0.0826 ms.
A representative update frame grew from 80 KB to 100 KB while encode/decode
changed from 0.0190/0.0254 ms to 0.0237/0.0284 ms. The sub-0.03 ms combined CPU
cost is negligible beside layout and scene construction, so Wabou chooses the
full key instead of a packed 20-bit slot/12-bit generation compromise.

TypeScript APIs brand handles by resource family even though the wire shape is
shared. Rust validates the key against the typed map before use. Optional
handles use an explicit presence field or a distinct clear operation; `(0, 0)`
is not a universal null sentinel.

The shared implementations are `createResourceKeyFamily()` and
`ResourceKeyTable` in `@wabou/core`, `ResourceKey<K>` in `wabou-host-api`, and
`ResourceRegistry<K, V>` in `wabou_runtime::resource`. A JS resource family has
a private runtime token in addition to its TypeScript brand, so accidental
casts between two live families fail before lookup. Rust uses a distinct
family type for each key or registry, making the equivalent mix-up a type
error. Shell resources such as windows reuse the shared Rust wire key instead
of defining another `{ lo, hi }` serializer. NodeKey's JS allocator reuses the
same validation and table machinery, but nodes remain owned by the retained
tree rather than a Rust `ResourceRegistry`.

Native windows are the first independently owned built-in resource using this
model. The shell allocates their SlotMap key only while accepting a window
creation effect, returns the pair in the asynchronous completion, and removes
the key when the window is permanently closed. Effect request ids remain
process-unique `u32` request-routing identities and are never exposed as window
handles. Their bounded representation is exactly representable by JavaScript.

Window scopes and targets in low-frequency native effects and recorded effect
tapes use the same `{ lo, hi }` representation. A packed `u64` is permitted as
an internal Rust lookup value, but it must not appear in JavaScript or JSON.

JSON capabilities should normally return a structured `{ lo, hi }` handle
when an application must retain one. Subsequent high-frequency operations
carry the same pair in binary frames.

## Selection rule

For every new cross-language feature, decide in this order:

1. Does a Rust object have identity or lifetime independent of an existing
   generational owner? Put it in a typed generational registry. Otherwise key
   it by that owner rather than creating a second identity.
2. Is the operation frequent or batchable? Add it to the frame protocol.
3. Must JavaScript receive a synchronous result to drive the runtime itself?
   Add a private native intrinsic and document why batching is impossible.
4. Is it a hot, stable, typed request/response operation? Define or extend a
   native capability.
5. Otherwise, define or extend a JSON capability.
6. If Rust produces later unsolicited values, deliver them through the host
   event frame.

Protocol declarations remain single-source and generated across Rust and
TypeScript. Node tests cover stale keys, malformed halves, removal and
round-trip preservation. Before adding the first independently owned resource
handle, its tests must additionally prove that handles from another resource
family are rejected.
