# Runtime boundary contract

Status: accepted. This document is normative for new cross-language and
resource APIs.

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
If measurement shows that a capability payload itself is a bottleneck, define
a dedicated binary frame record rather than weakening every capability.

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
`ResourceKeyTable` in `@wabou/core`, plus `ResourceRegistry<K, V>` in
`wabou_runtime::resource`. A JS resource family has a private runtime token in
addition to its TypeScript brand, so accidental casts between two live
families fail before lookup. Rust uses a distinct `slotmap::new_key_type!` key
for each registry, making the equivalent family mix-up a type error. NodeKey's
JS allocator reuses the same validation and table machinery, but nodes remain
owned by the retained tree rather than a Rust `ResourceRegistry`.

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
4. Otherwise, define or extend a JSON capability.
5. If Rust produces later unsolicited values, deliver them through the host
   event frame.

Protocol declarations remain single-source and generated across Rust and
TypeScript. Node tests cover stale keys, malformed halves, removal and
round-trip preservation. Before adding the first independently owned resource
handle, its tests must additionally prove that handles from another resource
family are rejected.
