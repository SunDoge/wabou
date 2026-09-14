# Performance profiling

Wabou performance instrumentation is opt-in at compile time. Normal builds do
not enable the `profiling` feature, do not link the trace exporter, and do not
contain Wabou's profiling span names. A trace is only written when both the
feature and an explicit output path are present.

Use the CLI to build and run a profiled release application:

```bash
bun run wabou run apps/stress --release \
  --profile-trace /tmp/wabou-trace.json
```

Open the resulting JSON in [Perfetto](https://ui.perfetto.dev/) or
`chrome://tracing`. The trace separates the native frame into JavaScript,
protocol decode/apply, style inheritance, widget measurement and paint, layout,
projection, scene construction, and presentation. Span arguments contain only
numeric workload metadata such as node, operation, and byte counts; source
locations and application text are not recorded.

Applications invoking Cargo directly can enable the same path through the
public facade:

```bash
WABOU_PROFILE_TRACE=/tmp/wabou-trace.json \
  cargo run --release --features wabou/profiling
```

Profiling changes timing and should not be used to report absolute production
overhead. Use it to identify stage proportions and unexpected work, then verify
an optimization in an uninstrumented release build. In particular, compare a
span's duration with its workload fields: a slow `quick.protocol.apply` with a
single class-cache miss suggests a different problem than the same duration
with thousands of misses.

Trace files may reveal window size, node counts, operation counts, and timing.
Treat them as diagnostic artifacts and do not ship them with an application.

## Regression workloads

Use three workloads when evaluating a performance change:

| Workload | Application | What it protects |
| --- | --- | --- |
| Typical UI | `apps/gallery` | component, text, overlay and mixed-layout overhead at ordinary node counts |
| Large list | `apps/vlist` | bounded visible-node work while scrolling a much larger data set |
| Pathological animation | `apps/stress` | dirty propagation, protocol traffic and scene construction at 1,000–25,000 moving nodes |

Record `js`, `build`, `scene`, `present`, node count and viewport for all three.
Compare identical release builds, viewport sizes, scale factors, and platforms.
A change is a regression candidate when the median of at least three
runs increases by 10% in any stage without reducing work in another stage.
Do not add a batch API solely to improve `apps/stress`; first prove the same
cost appears in a real retained UI or virtualized list.

### Invalidation evidence

Frame duration alone cannot prove that Solid's fine-grained reactivity reaches
native rendering. Performance traces and metric reports must separately count:

- committed protocol mutations;
- projection boundaries notified;
- retained nodes visited or materialized into scene commands;
- boundaries that required native layout;
- boundaries that required paint only.

For a local signal update, the expected count is the affected boundary rather
than the full application node count. A continuously updating animation clock
or Performance HUD must not increase materialized-node counts in an unrelated
static boundary. Gallery Colors is the regression workload for this invariant;
`apps/stress` remains a pathological animation workload, not the design target.

CI also records non-blocking headless medians for these workloads through
`wabou render --metrics`. The JSON artifacts contain build and scene samples,
node count, viewport, and scale factor. They intentionally exclude native
surface presentation and are not an FPS claim. Summarize local captures with:

```bash
bun run wabou render apps/gallery --out /tmp/gallery.png \
  --metrics /tmp/gallery-perf.json --samples 20
bun run perf:report -- /tmp/gallery-perf.json
```

The reports are observational until enough CI history exists to choose stable,
machine-aware thresholds. Real frame-rate decisions still require the release
trace workflow above on the target renderer and display.

## Memory baselines

Use `bun run perf:memory` to separate GPU startup cost from application and
long-session resource growth. It writes versioned JSON with platform, viewport,
checkout revision/dirty state, timestamped samples and explicitly named metrics.
The checkout metadata describes the sampling checkout, not proof of the measured
binary's provenance: rebuild before comparing and archive the build revision with
the reports. No memory budget is enforced until repeatable platform baselines exist.

### Minimal Hybrid probe

Build once, then sample the release executable directly (not Cargo):

```bash
cargo build --release -p wabou-shell-vello --example memory_probe
bun run perf:memory --probe target/release/examples/memory_probe \
  --width 800 --height 600 --scale 1 --seconds 60 --out /tmp/hybrid-memory-1x.json
bun run perf:memory --probe target/release/examples/memory_probe \
  --width 800 --height 600 --scale 2 --seconds 60 --out /tmp/hybrid-memory-2x.json
```

Use the executable under your configured Cargo target directory if it differs.
The probe renders a red background rectangle and a green inset rectangle using
the workspace's Vello Hybrid dependency and default renderer settings. It has no
QuickJS, font loading, widgets, application state, or native window. It pauses at
each checkpoint until the external sampler has measured its PID:

| Checkpoint | What has happened |
| --- | --- |
| `process_started` | Before wgpu instance/device creation |
| `device_created` | Device created; adapter/backend/device type recorded |
| `target_created` | Offscreen target and GPU readback staging buffer allocated |
| `renderer_created` | Hybrid renderer/resources created with default settings |
| `first_frame_completed` | Two rectangles submitted; GPU completion awaited |
| `steady_state` | Same scene repeatedly rendered for the requested duration |
| `pixels_verified` | CPU readback performed; background and foreground pixels asserted |

Use the first six stages for the memory baseline. The final stage deliberately
includes CPU readback costs. The target uses `BufferRenderer`, so its staging
allocation is present from `target_created`; this is not a native swapchain
measurement. OS accounting can defer charging allocations until they are used.
The loop waits for GPU completion and sleeps between frames; its frame count is
not an FPS benchmark. Steady state is an endpoint sample, not a continuous trace.
Pixel assertions establish this tiny scene's output only, not full UI correctness.
`--scale 2` doubles physical dimensions; it does not exercise OS display scaling.

### Application sampling

Start a release application with DevTools/profiling disabled, identify the actual
native app PID (not the CLI, Cargo, Vite, or a shell), and attach:

```bash
bun run wabou run apps/gallery --release
# In another terminal, replace 12345 with the native application PID.
bun run perf:memory --pid 12345 --label gallery-idle \
  --width 800 --height 600 --scale 1 --seconds 60 --interval 2 \
  --out /tmp/gallery-memory.json
```

PID mode neither launches nor controls nor terminates the application. It measures
only that PID, excluding child processes. Width, height and scale are operator
metadata; the sampler cannot detect or change window dimensions. For resize runs,
record the starting viewport and save the transition sequence/times beside the
JSON. Attach after the first completed frame for idle baselines; this cannot
recover startup stages. OS lifetime high-water metrics may predate attachment.

Record at least three independent runs per platform/configuration. Compare the
same machine, GPU/driver/backend, release build, workload, duration, logical size
and scale. Capture these scenarios using existing native behavior fixtures, an
isolated test display, or human interaction; never inject desktop-wide input into
the user's active session:

| Workload | Evidence to retain |
| --- | --- |
| Gallery idle, 60 seconds | Post-first-frame floor and sampled range |
| Gallery repeated image/SVG/text changes | Whether memory plateaus after warmup |
| Vlist repeated scrolling | Bounded memory as content enters/leaves the viewport |
| Stress at a fixed node count | Scene-complexity growth and allocation failures |
| Resize 800×600 → 1600×1200 → 800×600, at 1× and 2× | Peak and settled memory after returning |
| Open/close the same child window repeatedly | Whether each cycle adds retained resources |

For lifecycle scenarios, repeat identical cycles and allow an explicit idle period
after each. Record plateau/current values separately from peaks: a high-water mark
cannot fall when resources are freed. A retained allocator pool is not by itself
a leak; investigate continued growth across identical cycles. Pair memory changes
with the existing frame-time measurements and relevant pixel/platform evidence.

### Metric interpretation and failures

- macOS: `vmmap -summary PID` physical footprint and lifetime peak, converted to
  bytes. This requires permission to inspect that process.
- Linux: `/proc/PID/status` RSS and lifetime RSS high-water, plus
  `/proc/PID/smaps_rollup` PSS, converted from kernel kB to bytes. Read permission
  is required. These counters do not provide a complete GPU/VRAM allocation total.
- `sampledMaxBytes` contains maxima of the samples, including the probe's final
  readback stage. It is not an independently measured transient peak. PSS has no
  lifetime peak field here. RSS, PSS and macOS physical footprint are different
  metrics and must not be compared as equivalent totals.
- Sampling itself has overhead, especially `vmmap`; keep frame timing runs
  separate. `interval` is a delay between application samples, not a guaranteed
  cadence. Samples are sequential, not an atomic snapshot of every metric.
- Missing fields, permissions, process exit or probe failure produce a nonzero
  exit and an incomplete JSON report with an error. Do not use incomplete reports
  as successful baselines. Unsupported platforms fail explicitly.

The motivation is [Fission issue 81](https://github.com/fission-ui/fission/issues/81).
Its classic Vello buffer fix does not directly apply to Wabou's Hybrid renderer;
measure our own allocation stages before changing defaults or adopting a fork.

### Initial tool verification (2026-09-12)

One 5-second release probe per scale completed on Linux x64
(`6.12.107+deb13-amd64`), Intel Graphics (ARL), Vulkan, at logical 800×600.
The checkout was `91781b84` with local changes. Both runs verified the two pixels.
These are short smoke observations, not the three-run/60-second baseline or an
application memory measurement. Values below exclude final CPU readback.

| Stage | 1× RSS / PSS (MiB) | 2× RSS / PSS (MiB) |
| --- | ---: | ---: |
| Process started | 3.2 / 1.4 | 3.2 / 1.4 |
| Device created | 122.3 / 51.4 | 122.1 / 51.3 |
| Renderer created | 124.2 / 53.3 | 124.1 / 53.3 |
| First frame completed | 125.2 / 54.3 | 125.1 / 54.3 |
| Steady endpoint | 126.0 / 55.1 | 126.0 / 55.2 |

In these process counters the largest jump preceded Hybrid renderer creation.
This does not establish GPU allocation totals or macOS physical footprint.
Native Gallery/Vlist/Stress and macOS measurements remain to be collected.

## Artifact size

Application bundles are stored uncompressed because QuickJS evaluates them
directly. Installers may compress the complete application, so track both the
on-disk JavaScript and its gzip size instead of optimizing either number in
isolation. After building one or more applications, report comparable sizes
with:

```bash
bun run size:report -- dist/gallery/resources/bundle.js
```

CI records Gallery and Hacker News in the workflow summary. This is initially
an observation rather than a hard budget: locale data and other intentional
capabilities can cause legitimate step changes. Investigate unexpected growth
with the Vite/Rollup bundle graph before moving broadly useful APIs behind
Cargo features or duplicating mature JavaScript implementations in Rust.
