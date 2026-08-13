# Performance tracing

Wabou uses the standard Rust `tracing` ecosystem for logs and performance
timelines. This keeps Wabou in the same trace context as wgpu, winit, network
clients, application code, and other instrumented dependencies.

The normal `info` filter disables Wabou's frame spans. Enable them when needed:

```bash
RUST_LOG='info,wabou::perf=trace' wabou run apps/stress
```

The built-in formatting subscriber prints one `frame.complete` event with
window ID, node count, stage timings, and presentation status. A subscriber
layer that records span lifetimes can additionally reconstruct this hierarchy:

- `frame`
  - `frame.build`
    - `quick.build_frame`
      - `quick.js_tick`
  - `frame.scene`
  - `frame.present`

Native capability submission and completion emit `native_effect.submit` and
`native_effect.complete` events containing only request and operation IDs.
Payloads, clipboard text, passwords, menu labels, and window titles are never
attached to performance traces.

Wabou deliberately avoids spans per node, style declaration, glyph, or render
operation. The existing `FrameStats` EMA remains the cheapest always-available
overlay. `tracing` is intended for a bounded diagnostic run or a sampling
subscriber, not permanently logging every frame at 120 Hz.

Deterministic native-effect recording is a separate facility. Traces explain
where time went; effect tapes reproduce nondeterministic results.

