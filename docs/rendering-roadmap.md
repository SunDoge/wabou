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
