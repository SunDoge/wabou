# Performance tracing

Wabou records bounded CPU/elapsed measurements in the resident runtime. It is
**off by default**; disabled spans do not allocate records, read the clock, or
acquire the recorder lock after initial setup. The profiler is separate from the
compile-time `profiling` feature and also emits completion events to the existing
`wabou::perf` tracing target when enabled.

Start an app with recording and export at normal resident-runtime shutdown:

```sh
WABOU_PROFILE=trace WABOU_TRACE_FILE=/tmp/wabou-trace.json cargo run -p wabou-cli -- dev apps/pulse
```

Use `WABOU_PROFILE=counters` for summaries without a timeline. For an app with
DevTools enabled, control and export it while running:

```sh
cargo run -p wabou-cli -- inspect profile-mode trace --capacity 16384
cargo run -p wabou-cli -- inspect profile --output /tmp/wabou-trace.json
cargo run -p wabou-cli -- inspect profile-mode off
```

Use `inspect --socket PATH` to target a particular application. Mode changes clear
existing records. The shutdown file is not written after abort, SIGKILL or a
forced termination; export through DevTools before terminating in those cases.
Load the JSON file in https://ui.perfetto.dev. It includes a Chrome JSON timeline
plus `summaries`, `droppedEvents`, `recorderOverheadUs`, mode and capacity.

## Recorded boundaries

- JS: host-event dispatch, queued async jobs, and JS/protocol ticks. Dispatch/tick
  records include the host frame sequence, and dispatch includes event count.
- Frames: native redraw/build, projection, style inheritance, widget measurement,
  layout, scene construction, and CPU present. Frame build/scene/present have a
  parent redraw ID. Headless frames record projection/layout but do not claim
  native GPU present timings.
- Capabilities: typed sync/async handler duration, request decoding and response
  encoding; dynamic-module invocation includes request/response byte counts.
  Typed handlers include a numeric failure flag. Namespace-qualified method names
  are recorded when mounted through the runtime;
  typed transport records are aggregate boundaries, without per-call payloads.
- Reload: Rust cargo build and candidate library load/swap; Vite HMR apply and
  full entry reboot. Initial builds also appear. Compilation failures still
  produce duration records and preserve the previous implementation.

Async handlers and custom async measurements export async tracks, not CPU thread
slices. Their duration includes waiting; it is not CPU usage. CPU present means
submission/present call duration, not GPU completion or input-to-photon latency.
Correlation IDs aid inspection; automatic end-to-end input-to-display flow IDs,
GPU timestamps and task/resource inventories are not included in this first
version. `wabou inspect status` separately exposes live process and QuickJS
memory counters; they are point samples rather than timeline events.

## Application measurements

```ts
import { measurePerformance, measureAsyncPerformance } from "@wabou/ui/performance";
const rows = measurePerformance("rows.filter", () => filterRows(rowsSource));
const result = await measureAsyncPerformance("document.load", () => loadDocument());
```

The `performance` subpath has no renderer initialization side effects; the same
helpers are also exported from `@wabou/ui`; low-level integrations can use
`@wabou/core/performance`.

Use the sync helper for synchronous work and async helper for promises. Results
and thrown values are preserved. An optional third argument is a nonnegative
integer correlation ID. Completed JS measurements flush in one microtask, up to
128 per batch; `flushPerformance()` exports pending measurements explicitly.
They are anchored to the resident clock using duration and age at batch receipt.
Labels are bounded; no document contents, request bodies, or response values are
recorded. Each explicit measurement queries recording mode once, including when
recording is off.

## Limits and overhead

The timeline retains the most recent completed measurements (default 16,384,
maximum 65,536). Async measurements expand to two JSON events on export. Summaries
retain at most 256 category/name pairs and 128 duration samples per pair. Counts,
total duration and max cover accepted measurements since configuration; p50/p95/
p99 cover the most recent 128 completions per pair. New labels beyond the bound
and evicted timeline records increment `droppedEvents`.

`recorderOverheadUs` estimates Rust span setup/recording overhead, including lock
wait and tracing emission; it excludes attribute additions, JS helpers, batch
parsing, export/serialization and UI work. Recording adds overhead, and parent
measurements include nested recording overhead. Snapshot export can briefly block
recording; use counters for longer sessions and export traces after reproducing
an issue. In-progress measurements only appear after completion/cancellation.

For a repeatable local probe (not an app benchmark):

```sh
cargo run --release -p wabou-telemetry --example overhead
```

## Existing tracing subscriber integration

The bounded recorder complements Rust's `tracing` ecosystem. Builds enabling the
existing compile-time `profiling` features can still expose span lifetimes to a
custom subscriber and log frame completion with:

```sh
RUST_LOG='info,wabou::perf=trace' wabou run apps/stress
```

The existing frame hierarchy is `frame` → `frame.build` → `quick.build_frame` →
`quick.js_tick`, plus `frame.scene` and `frame.present`. Completion events include
window ID, node count, stage timings and presentation status. Native-effect
submission/completion events carry request and operation IDs without payloads.
The bounded recorder does not collect arbitrary dependency spans automatically.

Wabou avoids spans per node, style declaration, glyph or rendering operation. The
existing FrameStats EMA remains the cheapest always-available overlay. Keep full
tracing/logging to bounded diagnostic sessions. Deterministic native-effect tapes
remain separate: traces explain elapsed time; tapes reproduce effect results.
