# Rendering roadmap

Wabou has one production path: Solid 2 and QuickJS drive a retained Taffy,
Parley, and Vello GPU scene presented through Winit.

## Linebender alignment

Wabou follows Xilem and Masonry's text and rendering stack: Fontique and
Parley resolve fonts, shape and lay out text; Glifo prepares, hints and caches
glyphs; Vello CPU/GPU rasterizes and composites the scene. Wabou should keep
its adapters compatible with those upstream abstractions and contribute fixes
there when a problem is shared.

Do not add parallel Core Text, DirectWrite, or Pango shaping/rasterization
backends. Platform APIs remain appropriate for the window, IME, clipboard and
accessibility adapters, but glyph pixels stay on the shared Linebender path so
native and web rendering have one model. Text quality work should start with a
focused reproduction against current Xilem/Masonry and Glifo, then fix Wabou's
integration or follow the upstream change as the evidence indicates.

### Text quality reference tool

On Linux, compare Wabou's production text path against system Pango/Cairo and
Chrome's Skia-backed Canvas2D rasterizer with:

```bash
bun run text:compare
```

The report is written to `reports/text-quality/` and contains aligned 1x and 2x
PNGs, labelled comparison sheets, difference images, RMSE values, backend
versions, fontconfig's selected family, and Pango's actual fallback runs and
glyph identifiers. The shared cases live in `tools/text-quality/cases.json`.

Use Pango as a configurable visual reference rather than a Wabou runtime
dependency. For example, compare hinting and antialiasing policies with:

```bash
bun run text:compare --hinting full --antialias gray
bun run text:compare --hinting slight --antialias subpixel
bun run text:compare --hinting none --integer-positions
```

The initial Linux reference run found that Wabou is closest to Pango's full
hinting with grayscale antialiasing. A controlled Glifo experiment also
produced byte-identical Wabou images for Skrifa's `Normal`, `Light`, and `Lcd`
smooth hint targets. Glifo only applies vertical hinting on this outline path,
so changing the target is not a text-sharpness control here. Investigate final
pixel placement and Vello coverage/compositing before adding a platform hint
setting to Wabou.

Ordinary outline fonts must stay on Vello's hinted outline path. Glifo's atlas
path is reserved for fonts with embedded CBDT or sbix bitmap strikes, such as
color emoji; putting ordinary UI text in that grayscale atlas visibly softens
small text at 1x. The first reference sheet found nearly identical foreground
coverage for its leading Latin row: 817 non-background pixels at mean gray
124.95 for Wabou, 816 at 125.08 for Pango, and 803 at 124.27 for Skia. A
fractional-baseline matrix also preserved edge sharpness, so neither generic
gamma compensation nor unconditional baseline rounding is currently justified.

The tool requires `pangocairo`, a C compiler, Chrome and ImageMagick. Platform
captures are still required for Core Text and DirectWrite because Linux output
cannot establish macOS or Windows text quality.

Run the current path with the ordinary commands:

```bash
wabou dev apps/gallery
wabou run apps/7guis
wabou test apps/7guis/tests/app.behavior.ts --app apps/7guis --native
```

## Current priorities

1. Keep text editing and IME correct on macOS, Windows, Wayland, and X11.
   Platform adapters stay behind the shared focused-text-client contract.
2. Make renderer resource lifetime bounded. Image, SVG, glyph, and native
   widget allocations must be released or reused instead of exhausting Vello
   Vello GPU atlases during long sessions.
3. Preserve Solid's batching through the protocol and classify invalidation as
   structure, layout, paint, semantics, or widget work. Avoid polling and root
   rebuild clocks.
4. Expand deterministic headless/layout evidence and focused platform captures.
   A capture proves only the backend, scale factor, and platform that produced
   it.
5. Improve SVG coverage. Solid/gradient fills, strokes, transforms, simple
   clips, opacity/blending, nested SVG, and raster images are implemented;
   masks, filter graphs, patterns, and complex clips require explicit support
   or diagnostics.
6. Keep GPU effects Rust-owned. Solid may select a registered shader and update
   typed parameters, while Rust owns pipelines, textures, resize, device
   lifetime, clipping, and composition.

## Crate boundaries

```text
                         wabou
          QuickJS session, projection, host services
                   /                    \
          wabou-shell              wabou-runtime
       Winit and platform I/O       JS/runtime services
                   |
             wabou-renderer
          Taffy, Parley, Vello GPU
                   |
            wabou-shell-api
          shared platform contracts
```

Small support crates remain independent only when they provide a real
dependency or incremental-compilation boundary: protocol, style, host API,
bindgen, database, and SVG. Ordinary subsystems belong in
modules, and experimental backends must not leak into the public facade.

New renderer features require a real implementation and test; silent fallback
or an emulated second backend is not accepted. Use external projects as quality
references without importing their state model into Wabou's public API.
