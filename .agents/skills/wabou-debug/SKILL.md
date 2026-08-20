---
name: wabou-debug
description: Reproduce and diagnose Wabou native UI bugs using layout snapshots, DevTools sockets, debug overlays, headless PNG rendering, protocol tests, and platform/HiDPI comparisons. Use for incorrect layout, clipping, custom-widget rendering, blank or stale windows, resize/maximize failures, HMR discrepancies, hit-testing issues, Linux-versus-macOS differences, and performance regressions in this repository.
---

# Debug Wabou

Gather evidence at the failing layer before editing. Do not treat a passing Linux 1× render as proof of macOS 2× correctness.

## Start with state

Run:

```bash
git status --short
git rev-parse --short HEAD
```

Preserve unrelated changes. Record whether the user runs `dev`, `run`, or a packaged binary.

Restart the native process after Rust changes. Vite/Solid HMR updates JS and Style IR only; it does not reload `wabou-shell`, `wabou-quick`, Vello scene code, or Rust widgets.

## Isolate the failing layer

Trace one suspect node through these layers:

1. Static JSX candidate extraction and Wabou preset output.
2. Applied classes and typed style in `ComputedNodeSnapshot`.
3. Taffy `PlacedNode`: border box, content box, overflow clip, radius, depth.
4. Vello scene composition: transform, clip stack, `Scene::append`, layer lifetime.
5. Surface/backend: logical size, physical size, device scale, GPU backend.

Prefer an assertion at the earliest incorrect layer. If geometry is correct but pixels are wrong, add an offscreen render or platform-specific reproduction instead of more layout assertions.

## Inspect a running app

Start with DevTools enabled:

```bash
mise exec -- bun run wabou dev apps/gallery --devtools
```

Then inspect the discovered socket:

```bash
mise exec -- bun run wabou inspect status
mise exec -- bun run wabou inspect query fractal
mise exec -- bun run wabou inspect node <id>
mise exec -- bun run wabou inspect screenshot
```

Use the DevTools `Layout` control for native overlays:

- blue: layout bounds
- orange: overflow/scroll clips
- red: current hit target
- purple: selected node

The MCP tool `wabou_set_layout_overlay` exposes the same target-window overlay for agent-driven diagnosis.

## Capture without a display server

Use `scripts/capture-png.sh` for a deterministic offscreen render:

```bash
.agents/skills/wabou-debug/scripts/capture-png.sh gallery /tmp/gallery.png 1440 900
.agents/skills/wabou-debug/scripts/capture-png.sh gallery /tmp/platform.png 1440 900 80 825
WABOU_CAPTURE_SCALE_FACTOR=2 .agents/skills/wabou-debug/scripts/capture-png.sh gallery /tmp/gallery@2x.png 1440 900
WABOU_CAPTURE_WINDOW_ID=2 .agents/skills/wabou-debug/scripts/capture-png.sh gallery /tmp/child.png 800 600
mise exec -- bun run wabou render apps/gallery --out /tmp/gallery.png --snapshot /tmp/gallery-tree.json
```

The script uses the real application host by default, so registered services,
capabilities, message producers, and widget factories participate in the
capture. Set `WABOU_CAPTURE_BUNDLE_ONLY=1` for a faster frontend-only capture.
Coordinate click capture currently selects the bundle-only path.

Inspect the resulting PNG with an image viewer/tool, not by file existence alone.

Limitations:

- The script defaults to logical window id 1 and device scale 1. Override them
  with `WABOU_CAPTURE_WINDOW_ID` and `WABOU_CAPTURE_SCALE_FACTOR`.
- `wabou render` can replay multiple `--click X Y` arguments in order and
  commits `--text` to the element focused by the final click.
- A coordinate click is suitable only after first capturing the current layout.
- `--snapshot` writes the DevTools tree from the exact final logical frame used
  by the PNG, so text and layout diagnostics remain available without a display
  server. Rebuild workspace package artifacts before using it against package
  source changes.

Do not patch app logic merely to obtain a screenshot and accidentally commit the patch. If a temporary diagnostic edit is unavoidable, revert it immediately and verify `git diff`.

## Diagnose platform and HiDPI differences

Compare these values before blaming Taffy:

- `WindowMetrics.logical_*`
- surface physical dimensions
- `window.scale_factor()`
- transform applied to the parent scene
- scale already encoded inside widget glyph/image fragments

Keep layout and hit testing in logical pixels. Apply the device transform once at scene encoding. Bitmap/glyph resources may be generated at physical resolution, but compensate when appended.

For custom widgets, prefer clipping inside the widget fragment before appending it. A parent clip wrapped around `Scene::append` can expose backend-specific behavior, especially with Metal and HiDPI. Test 1× and 2× paths separately.

When behavior differs only on macOS:

1. Confirm a full Rust process restart.
2. Capture `scale_factor`, logical size, and physical size.
3. Reduce the scene to one solid widget rectangle plus one rounded clip.
4. Compare local-fragment clipping with parent-scene clipping.
5. Search current Vello/wgpu issues only after the minimal case proves the framework geometry is correct.

### Diagnose text weight and raster differences

Do not infer double bolding from pixels alone. Restart the native process, then
inspect the suspect text node and runtime status:

```bash
mise exec -- bun run wabou inspect status
mise exec -- bun run wabou inspect query '<visible text>'
mise exec -- bun run wabou inspect node <id>
```

Compare these fields between platforms:

- `status.textBackend`: `swash` or `vello-outline`;
- `status.textOutlineFallback`: the platform outline fallback policy;
- `node.computed.fontFamily` and `fontWeight`: the requested style;
- `node.computed.syntheticBold` and `syntheticItalic`: Parley's resolved glyph
  runs, obtained from the exact cached layout used by paint.

If `syntheticBold` is false, the renderer did not geometrically embolden the
run; investigate typography tokens, the resolved system face, hinting, and
device scale instead. If it is true, first verify whether the requested family
actually provides that weight. Keep ordinary Swash rasterization shared across
platforms. Platform-specific outline behavior belongs in the single
`OutlineFallback` policy, not in scene call sites or component styles.

## Choose the right test

- Parser/preset bug: TypeScript unit test for candidate → typed Style IR.
- Cascade bug: `computed_style` snapshot test.
- Layout/clip bug: `layout_fixtures` or `wabou-shell::layout` geometry test.
- Event bug: encoded op/event replay test with hit target assertions.
- Scene/backend bug: offscreen pixel test or a minimal platform reproduction.
- Resize/maximize bug: dispatch real `WindowMetrics` transitions; do not test only the initial size.

An op replay can verify deterministic tree, style, layout, invalidation, and events. It cannot alone verify GPU pixels or OS surface behavior. Pair it with a screenshot/pixel assertion for rendering regressions.

## Validate proportionally

Run the narrow test first, then the affected app build:

```bash
cargo test -p wabou-shell --lib
cargo test -p wabou-quick --lib
mise exec -- bun x tsc --noEmit
mise exec -- bun run wabou build apps/gallery
git diff --check
```

Report what was directly reproduced, the platform/scale tested, and what remains inferred. Never say a visual bug is fixed based only on compilation or geometry tests.
