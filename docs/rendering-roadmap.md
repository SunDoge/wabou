# Rendering roadmap

Wabou records text, paths, gradients, images, clipping, transforms, shadows,
SVGs, and native widgets into an AnyRender scene. Vello Classic is the default
window and image backend; Skia is available behind the optional
`renderer-skia` feature and `WindowOptions::renderer` selection. Unsupported
backends fail explicitly instead of silently falling back.

Backend behavior was first evaluated with the isolated
[`experiments/anyrender-backends`](../experiments/anyrender-backends/README.md)
harness. The same contract now owns Wabou's production frame scene, while the
harness remains useful for focused backend comparisons.

## Deferred work

### Vello Hybrid backend

Add Vello Hybrid as a separate AnyRender backend before replacing Vello Classic.
The migration must preserve Wabou's text, image, SVG icon, clipping, rounded
corner, transparency, and HiDPI fixtures. Wabou's canonical scene and SVG path
are now backend-neutral, so remaining work is a capability matrix and pixel
comparisons for the subset used by Wabou.

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
