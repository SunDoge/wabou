# Rendering roadmap

Winit, Taffy, Parley, and Vello Hybrid are Wabou's default application runtime.
GPUI-CE remains an explicit comparison backend and implementation reference.
Both run the same Solid bundle and mutation protocol, but are selected at
compile time rather than through `WindowOptions`, so backend-specific widget
types remain explicit and neither backend silently emulates the other.

Backend behavior was first evaluated with the isolated
[`experiments/anyrender-backends`](../experiments/anyrender-backends/README.md)
harness. Its implementation currently still lives under transitional
`wabou-legacy-*` crate names, but `HostBuilder` now selects the Hybrid path and
`GpuiHostBuilder` selects the comparison path. Fixtures and captures must
always report their actual backend; results from one backend never prove the
other.

## Highest priority: preserve Solid invalidation in GPUI

The current coarse `GpuiRuntimeView` proves the projection but can still
materialize the complete GPUI element tree when any root notification occurs.
That discards a central reason for using Solid: a local reactive update should
remain local after it crosses the runtime boundary.

Implement this before broad widget migration or renderer experiments:

1. Move the animation clock and Performance HUD out of the application root
   into independent GPUI entities. A diagnostic sampler must never force every
   application node to rebuild each display frame.
2. Introduce explicit `ProjectionBoundary` units for route content, scroll
   regions, overlays, native widgets, and other stable retained regions. Do not
   guess boundaries from Solid compiler/component output and do not allocate an
   entity for every leaf.
3. Track `structure_revision`, `layout_revision`, and `paint_revision` per
   boundary. Notify only the entity and GPUI phase required by the strongest
   dirty revision.

The first acceptance workload is Gallery Colors: opening a live performance
HUD may update the HUD entity, but a static color-grid boundary must not be
materialized on every sampled frame. Stress and virtual-list workloads then
verify animation and scrolling without weakening this typical-UI requirement.

## Vello Hybrid backend

The first vertical slice is operational: QuickJS/Solid emits the shared binary
protocol, the retained Winit projection resolves Style IR and Taffy layout,
and AnyRender replays the resulting scene into a Vello Hybrid window surface.
The window and SVG paths use `vello_hybrid` 0.2 and share one `vello_common`
release line; the repository-local AnyRender adapter carries the small API and
non-blocking presentation patches needed until upstream catches up.
Run it against the shared 7GUIs application with:

```bash
wabou run apps/7guis
wabou run apps/gallery
wabou test apps/7guis/tests/app.behavior.ts \
  --app apps/7guis --native

# Comparison backend
wabou run apps/gallery --features gpui
```

Vite HMR and typed application capabilities now share the same transport and
contract APIs as the GPUI host. Host services, Rust-to-JavaScript producers,
application directories, SQLite KV, window persistence, and DevTools are also
mounted by the default `HostBuilder`. A single public `HostMessageRouter` can address
either backend, and the Winit builder additionally exposes effect tape
record/replay plus native event-loop extensions. Initial and JavaScript-created
windows now share one runtime source factory, so dynamic windows inherit the
same capabilities, message producers, resources, source maps, and HMR setup.
Both backends now also execute the same `JsRuntime`, encode unsolicited native
events through the same binary Host Frame implementation, submit native effects
through the same queue and record/replay implementation, and consume the same
coalesced HMR inbox. Those pieces now live in one shared per-window
`RuntimeSession`, including cancellation and graceful producer shutdown. The
former Winit copies and their duplicate tests have been removed, so scheduler,
wake, stack-limit, source-map, event, effect, reload queue, and session-lifetime
fixes cannot drift between backends.

Packaged bundle and source-map discovery is shared as well. Vello Hybrid now
uses the same development override, adjacent-resource, Debian `/usr/lib`, and
macOS `.app/Contents/Resources` candidates as GPUI instead of its older subset.
The standard Winit controls support native text, paste, IME, pointer, wheel,
selection, and value synchronization, and the shared semantic behavior driver
runs against the real Winit event loop. The remaining promotion work is
backend-labelled pixel fixtures, application-defined widget parity, and
removal of the transitional GPUI session wrapper from the Winit runtime.

The Hybrid style projection preserves independent corner radii through
background, border, outline, overflow, and native-widget clipping. AnyRender's
current box-shadow command accepts one radius, so asymmetric shadows use the
largest corner radius while the box itself remains exact.

`wabou-vello-hybrid-svg` now isolates the first reusable renderer-side piece:
it converts a normalized `usvg` tree into Vello Hybrid scene commands. It
supports solid and gradient path fills, strokes, transforms, simple clips,
group opacity/blending, nested SVG images, and embedded raster images. Masks,
filter graphs, pattern paints, and complex clip paths produce structured
diagnostics instead of entering unsupported Hybrid code paths. This adapter is
renderer-side infrastructure; the application backend is exposed separately
through the default `HostBuilder`; `WinitHostBuilder` remains a transitional
explicit name while backend crates are reorganized.

New shared runtime features must remain backend-neutral. Renderer-specific
features require an explicit implementation and test instead of fallback
behavior. GPUI is retained as a quality comparison and source of implementation
lessons, not as the semantic owner of new Wabou APIs.

## Target crate boundaries

The current `wabou-legacy-*` names describe history, not architecture, and are
now misleading. Do not perform a big-bang merge or rename while shared host
code is still duplicated. Move toward these three layers instead:

```text
                         wabou (public facade)
                                  |
                         wabou-runtime (shared)
                    QuickJS, Solid flush, capabilities,
                    messages, resources, HMR, persistence
                         /                    \
             wabou-backend-gpui     wabou-backend-vello-hybrid
             GPUI projection,       Winit, Taffy, Parley,
             native GPUI widgets    Vello Hybrid projection
                         \                    /
                    wabou-shell-api + protocol + style
```

Here, "backend" includes window lifecycle, input, text, accessibility, layout,
paint, and native widgets; it is deliberately broader than a renderer. GPUI and
Vello Hybrid should not be hidden behind one lowest-common-denominator render
trait. They consume the same retained UI intent and host contracts, then own
their backend-specific state and optimized paths.

Apply the reorganization in this order:

1. Continue extracting duplicated backend-neutral files into `wabou-runtime`.
   `jsrt`, Host Frame encoding, host ABI/FFI, host messages, capabilities, and
   resources (including the generational registry), effect dispatch/recording,
   HMR queueing, the per-window runtime session, and bundle/source-map discovery
   are shared already; persistence and HMR application policy must follow before
   any crate rename.
2. Move GPUI-specific `gpui_*` modules and the current `wabou-shell` projection
   into `wabou-backend-gpui`.
3. Rename the Winit implementation to `wabou-backend-vello-hybrid`. Keep large
   renderer helpers such as SVG conversion in a private supporting crate when
   that preserves incremental compilation; package count is an implementation
   detail and must not expand the public API.
4. Keep `HostBuilder` as the Vello Hybrid default and export
   `GpuiHostBuilder` for comparison. Backend choice remains compile-time,
   so applications do not ship both platform stacks accidentally.
5. Run shared protocol/component behavior suites once per backend, followed by
   backend-labelled layout, pixel, native-input, and platform smoke tests. A
   screenshot must always use the renderer that presented its window.

The small support crates (`protocol`, `style`, `host-api`, `bindgen`,
`database`, and `terminal-core`) stay independent because they are genuine
dependency and incremental-compilation boundaries. Backend-only widget,
terminal, and accessibility implementations should not leak into the public
facade as separate concepts.

### GPU effects

Design GPU effects only after the Hybrid experiment establishes the renderer's
external-texture and composition contracts. The intended boundary is a
Rust-owned, registered shader effect with typed parameters:

- Rust owns pipelines, uniforms, textures, resize, and device lifetime.
- Solid selects a registered effect and updates typed parameters.
- The renderer handles layout, clipping, transforms, opacity, and composition.
- Normal UI does not expose WebGPU or arbitrary WGSL to JavaScript.

The first experiment should cover one animated shader texture at 1x and 2x,
resize, rounded clipping, device loss, and deterministic headless capture. It
must avoid GPU-to-CPU readback on normal frames.
