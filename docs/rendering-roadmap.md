# Rendering roadmap

Wabou has one production path: Solid 2 and QuickJS drive a retained Taffy,
Parley, and Vello Hybrid scene presented through Winit. GPUI, AnyRender, and
Skia were useful experiments, but they are not selectable application backends.

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
   Hybrid atlases during long sessions.
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
                         wabou (public facade)
                                  |
               wabou-backend-vello-hybrid
          QuickJS session, projection, host services
                 /                         \
       wabou-shell-vello             wabou-runtime
  Winit, Taffy, text, paint       protocol/runtime services
        /          \
widgets-vello   accessibility-vello
```

Small support crates remain independent only when they provide a real
dependency or incremental-compilation boundary: protocol, style, host API,
bindgen, database, and SVG. Ordinary subsystems belong in
modules, and experimental backends must not leak into the public facade.

New renderer features require a real implementation and test; silent fallback
or an emulated second backend is not accepted. Use external projects as quality
references without importing their state model into Wabou's public API.
