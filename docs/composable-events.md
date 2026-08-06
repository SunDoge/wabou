# Composable event pipeline

Status: exploratory extension of the accepted Host→JS transport in
[`host-to-js.md`](./host-to-js.md). This document does not commit the runtime to
an authoring API.

## Motivation

Wabou currently has a deliberately small event path:

```text
winit input -> normalize -> hit test -> capture/target/bubble -> JS listener
```

That path is sufficient for DOM-like `onClick` handlers, but the framework will
also need native Rust handlers, pointer capture, shortcuts, drag gestures and
high-frequency event processing without crossing the JS/Rust boundary for every
intermediate step. Those requirements suggest compiling event declarations into
an executable pipeline rather than growing a collection of unrelated special
cases.

## Design boundaries

The following behavior remains owned by the Rust runtime and cannot be replaced
by arbitrary user operators:

- native event normalization;
- hit testing and the selected target;
- focus ownership and pointer capture invariants;
- the target and lifetime inputs from which the guest performs capture, target
  and bubble ordering;
- lifetime validation for node and listener IDs;
- execution limits that prevent an event plan from looping indefinitely.

Composition starts after normalization. Capture/target/bubble listener
invocation remains in the guest renderer, as specified by the Host frame
contract. Default actions are a separate, cancellable Host phase after guest
propagation. For example, clicking an anchor first dispatches listeners, then
the Host opens its URL unless the returned frame disposition contains that
event's `preventDefault()` marker.

## Conceptual pipeline

```text
EventSource
  -> normalize
  -> hit-test
  -> propagation phase
  -> filter/map/coalesce
  -> optional gesture recognizer
  -> Rust or JS sink
  -> default action
```

Ordinary JSX listeners are syntax sugar for a trivial plan. They should not pay
for the general pipeline when no operators are present.

## Compiled representation

One possible internal representation is:

```rust
struct EventPlan {
    source: EventKind,
    phase: EventPhase,
    ops: Box<[EventOp]>,
    sink: EventSink,
}

enum EventOp {
    RequireButtons(u32),
    RequireModifiers(u8),
    MapPosition(CoordinateSpace),
    CoalescePointerMoves,
    StopPropagation,
    Gesture(GestureId),
}

enum EventSink {
    Js(ListenerId),
    Rust(HostFnId),
}
```

This is illustrative rather than a public API. Plans should be registered once,
validated and compiled into compact numeric instructions. Event names, host
function names and structural strings use the existing atom pool; dispatch does
not repeatedly compare or allocate strings.

## Hot-path rules

- Do not allocate JSON for pointer, wheel or gesture updates.
- Do not allocate a `Vec` per event; use the registered instruction slice and a
  fixed/shared payload.
- Do not invoke JS for native filters or native-only sinks. For a JS sink, cross
  FFI once with the targeted Host frame; the guest then performs its own
  propagation bookkeeping without another Host round trip.
- Coalesce pointer moves before crossing FFI.
- Keep stateless plans shareable. Store state for debounce, drag and multi-click
  recognizers separately and reclaim it with the owning node/listener.
- Preserve stack traces for JS and user Rust sinks by retaining explicit manual
  registration rather than hiding calls behind a generic dynamic dispatcher.

## Gestures are not propagation operators

Gestures consume low-level events over time and emit a new high-level event.
They therefore need state machines with cancellation and arbitration, rather
than being modeled as simple `map` functions. Initial candidates are:

- click and double-click;
- long press;
- drag with a movement threshold;
- keyboard shortcut/chord;
- scroll/pinch gestures where the platform exposes enough data.

Gesture state is keyed by a generational node/listener ID and is removed when
that owner is dropped. It must not rely on an LRU cache.

## Possible authoring shape

The eventual TypeScript surface might look like this:

```ts
on(pointer.down)
  .primaryButton()
  .capturePointer()
  .drag({ threshold: 4 })
  .to(rustFn("move_window"));
```

A Rust-native builder should produce the same compiled plan. Neither authoring
API should dictate the wire representation.

## Suggested implementation order

1. Stabilize click, input, wheel, focus, pointer capture and default actions.
2. Make capture/target/bubble phases explicit in the current dispatcher.
3. Introduce `EventPlanId` for existing listeners while retaining a fast trivial
   listener path.
4. Add native Rust sinks and verify that native-only plans never enter QuickJS.
5. Add stateless filters and mappings.
6. Add coalescing and gesture recognizers with explicit state lifetime.
7. Only then expose public TypeScript and Rust builders.

The abstraction should be extracted from demonstrated UI requirements rather
than implemented speculatively before the base event semantics are complete.
