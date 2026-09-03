# Rendering roadmap

GPUI-CE is Wabou's sole public application runtime and owns the production
window, text, layout, input, and rendering lifecycle. Applications do not
select a renderer through a Cargo feature or `WindowOptions`: such a switch
would imply compatible implementations where none exist.

Backend behavior was first evaluated with the isolated
[`experiments/anyrender-backends`](../experiments/anyrender-backends/README.md)
harness. That AnyRender implementation now lives under explicitly named
`wabou-legacy-*` crates and remains useful only as a migration oracle and
focused backend comparison. Formal layout fixtures run through GPUI itself;
the CLI never falls back to the retired renderer.

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

## Deferred work

### Vello Hybrid backend

Evaluate Vello Hybrid inside GPUI when GPUI exposes a suitable integration
boundary. The migration must preserve Wabou's text, image, SVG icon, clipping,
rounded corner, transparency, and HiDPI fixtures.

Do not switch the default backend until the required imaging features and APIs
are sufficiently stable upstream.

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
