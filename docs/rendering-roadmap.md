# Rendering roadmap

GPUI-CE remains Wabou's default application runtime and production baseline.
Winit, Taffy, Parley, and Vello Hybrid form a parallel first-class backend
track. Both run the same Solid bundle and mutation protocol, but are selected
at compile time rather than through `WindowOptions`, so backend-specific
widget types remain explicit and neither backend silently emulates the other.

Backend behavior was first evaluated with the isolated
[`experiments/anyrender-backends`](../experiments/anyrender-backends/README.md)
harness. Its implementation currently still lives under transitional
`wabou-legacy-*` crate names, but `WinitHostBuilder` makes the Hybrid path a
real, independently runnable application backend. Formal GPUI layout fixtures
remain authoritative for the default backend; Winit fixtures must report their
own backend and never masquerade as GPUI results.

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
wabou run apps/7guis --features vello-hybrid
wabou run apps/gallery --features vello-hybrid
wabou test apps/7guis/tests/app.behavior.ts \
  --app apps/7guis --native --features vello-hybrid
```

Vite HMR and typed application capabilities now share the same transport and
contract APIs as the GPUI host. Host services, Rust-to-JavaScript producers,
application directories, SQLite KV, window persistence, and DevTools are also
mounted by `WinitHostBuilder`. A single public `HostMessageRouter` can address
either backend, and the Winit builder additionally exposes effect tape
record/replay plus native event-loop extensions. Initial and JavaScript-created
windows now share one runtime source factory, so dynamic windows inherit the
same capabilities, message producers, resources, source maps, and HMR setup.
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
as `WinitHostBuilder` and does not change GPUI's default role.

Keep GPUI as the default backend until the required imaging, native-widget,
pixel-fixture, and platform input contracts pass on Hybrid. New shared runtime
features must remain backend-neutral; renderer-specific features require an
explicit backend implementation and test instead of fallback behavior.

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
