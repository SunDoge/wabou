# AnyRender backend comparison

This isolated experiment records one `anyrender::Scene` and renders the exact
same commands with the Skia, Vello Classic, and Vello CPU image renderers.
It intentionally does not participate in the Wabou workspace.

```bash
cargo run --release --manifest-path experiments/anyrender-backends/Cargo.toml -- \
  experiments/anyrender-backends/out
```

The scene contains the primitives Wabou cares about most when evaluating a
backend: gradients, rounded paths, clipping, translucent strokes, blurred box
shadows, and a filtered layer. Generated PNGs are ignored by Git.

`anyrender_vello_hybrid` currently exposes a window renderer but no
`ImageRenderer`, so it cannot participate in this deterministic PNG comparison
without adding a window/surface harness or extending the backend.

## Result on 2026-08-24

Tested on Linux x86_64 with Rust 1.98.0. The first release build took 2m46s;
the isolated target directory occupied 982 MiB because it compiled Skia, WGPU,
Vello Classic, and Vello CPU together. The combined executable was 15 MiB.

| Backend | Cold initialization | Warm-process initialization | 960x560 render | Filter result |
| --- | ---: | ---: | ---: | --- |
| Skia | 26.69 ms | 25.15 ms | 3.68–7.79 ms | Gaussian blur applied |
| Vello Classic | 995.42 ms | 54.48 ms | 67.41–74.17 ms | Filter silently ignored |
| Vello CPU | 0.23 ms | 0.21 ms | 3.49–4.33 ms | Filter silently ignored |
| Vello Hybrid | — | — | — | No AnyRender image renderer |

These are startup/first-frame observations from one cold run followed by one
warm-process run, not a throughput benchmark. Pipeline and driver caches make
the Vello Classic cold initialization particularly expensive, so sustained
frame timing needs a separate multi-sample benchmark with a reused renderer.

The unfiltered left panel is visually very close across the three rendered
backends. Vello Classic versus Vello CPU has an RMSE of 138.9/65535
(normalized 0.00212), mostly from antialiasing and shadow rasterization. The
filtered right panel is materially different: Skia blends the three circles
into a continuous field while both Vello backends preserve hard circle edges.
Skia versus Vello Classic has normalized RMSE 0.0277.

This proved that AnyRender was useful as a Wabou backend contract and comparison
harness, but not as feature parity by itself. Wabou now records its canonical frame
as an `anyrender::Scene`, uses Vello by default, and offers Skia behind the optional
`renderer-skia` feature. Unsupported effects may still degrade silently, so an
explicit capability matrix and deterministic fallback policy remain future work.
