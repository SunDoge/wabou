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

Applications still using the legacy entry crate use
`--features wabou-runtime/profiling` instead.

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
Compare identical release builds, viewport sizes, scale factors and renderer
backends. A change is a regression candidate when the median of at least three
runs increases by 10% in any stage without reducing work in another stage.
Do not add a batch API solely to improve `apps/stress`; first prove the same
cost appears in a real retained UI or virtualized list.

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
