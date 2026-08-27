# Subsecond hot-reload experiment

This experiment checks the minimum property Wabou needs from Subsecond: a Rust
function changes while the original process and its long-lived state remain alive.
The process id models the native window/event-loop lifetime. The `Arc<AtomicU64>`
created outside the hot call models capability state owned by the stable host and
passed into a replaceable handler.

Requirements:

```sh
cargo binstall dioxus-cli@0.7.10
```

Run from this directory:

```sh
dx serve --hot-patch --desktop --interactive false --open false --verbose
```

After `READY`, change `handler_version` in `capability_handler` from `1_u64` to
`2_u64`. A successful patch has all three properties:

1. `handler` changes in subsequent `TICK` lines;
2. `pid` remains unchanged;
3. `tick` continues increasing instead of returning to one.

Observed result:

```text
TICK pid=1116421 tick=29 handler=1
TICK pid=1116421 tick=30 handler=2
```

The hot boundary is deliberately an explicit `subsecond::HotFn`. Subsecond does
not automatically redirect arbitrary callees, and changing string literals is not
a reliable test because read-only static data has hot-patching limitations.

Do not move persistent state into a static owned by the hot tip crate. The experiment
confirmed that a patched function can bind to a newly emitted copy of that static,
resetting its value even though the process id stays unchanged. Wabou capability
state must remain host-owned and cross the hot boundary as an argument or handle.

This deliberately does not open a Wabou window yet. It isolates patch generation,
delivery, function dispatch, and state retention before involving winit's
platform-owned event loop.

## Wabou integration boundary

This is suitable for opt-in development-time capability reloading:

- Wabou owns the window, event loop, renderer, JavaScript runtime, and capability
  state for the lifetime of the process.
- A capability registry stores explicit hot function handles and passes stable
  state handles or `Arc`s into them.
- In-flight asynchronous calls finish on their old implementation; new calls use
  the patched implementation. Changing state or DTO layout requires rebuilding
  the capability instance or restarting the process.
- Runtime configuration such as initial window size and decorations remains
  restart-required. It does not justify making the whole runtime hot-reloadable.

The experiment depends on Dioxus CLI because it supplies ThinLink patch generation
and delivery. The `subsecond` crate alone only provides the runtime hot-call
boundary.
