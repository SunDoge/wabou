# Rendering roadmap

Wabou currently targets predictable desktop UI rendering with Vello Classic.
The supported surface is text, paths, gradients, images, clipping, transforms,
shadows, and native widgets. Examples should demonstrate capabilities that are
stable in the active renderer rather than approximate browser or shader effects.

## Deferred work

### Vello Hybrid migration

Evaluate Vello Hybrid behind a separate backend before replacing Vello Classic.
The migration must preserve Wabou's text, image, SVG icon, clipping, rounded
corner, transparency, and HiDPI fixtures. `vello_svg` currently emits a Classic
`vello::Scene`, so the experiment also needs a small `usvg` to Hybrid scene
adapter and pixel comparisons for the SVG subset used by Wabou.

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
