---
name: wabou-debug
description: Reproduce, test, and diagnose Wabou UI bugs with component Vitest tests, QuickJS + Style IR + Taffy layout contracts, native behavior scenarios, protocol tests, DevTools inspection, and focused pixel/platform captures. Use for component behavior, reactive warnings, incorrect layout, clipping, custom-widget rendering, blank or stale windows, resize/maximize failures, HMR discrepancies, hit-testing issues, Linux-versus-macOS differences, and performance regressions in this repository.
---

# Debug Wabou

Gather evidence at the failing layer before editing. Make the cheapest deterministic
test reproduce the bug, then keep that test as the regression. Do not use screenshots
as the default validation loop. Do not treat a passing Linux 1× render as proof of
macOS 2× correctness.

## Default verification ladder

Choose the first layer that can prove the requested property and stop there unless a
lower layer remains suspect:

1. **Component Vitest** for state, roles, semantic attributes, event handlers,
   composition, timers, capability calls, authored style, and transforms.
2. **Protocol/style tests** for candidate parsing, class resolution, computed style,
   op encoding, resource handles, and generated DTO boundaries.
3. **Component layout fixtures** for QuickJS effects, Style IR, text measurement,
   Taffy geometry, overflow, clipping, collision, scroll ranges, and responsive
   layout. Run fixtures in one release CLI batch.
4. **Native behavior tests** only for native hit testing, keyboard/IME, clipboard,
   native widgets, multiple windows, tray, resize, drag/drop, child-process/service
   boundaries, or completed semantic frames.
5. **Headless pixel capture** only when geometry and semantics are correct but paint
   is wrong: glyph rasterization, shadows, rounded clips, image decoding, native
   widget paint, Vello scene composition, or transparency.
6. **Platform capture** only for backend, compositor, DPI, font-resolution, or
   operating-system behavior that cannot be established at earlier layers.

Do not add a broad behavior or screenshot test when a component or layout test can
express the contract. Do not repeatedly inspect screenshots after style-only edits;
add or update the component fixture and run the batch.

## Fast component and layout loop

Use `@wabou/test/component` with Vitest for independently testable components. Drive
controls by role and accessible name, then assert semantic state or authored output.
Use fake time for tooltip, debounce, and finite animation behavior; use typed host and
platform fixtures for capabilities. Read `docs/testing.md` when adding a component
test.

Run the narrow unit test first:

```bash
bun --conditions=browser --conditions=wabou-source test path/to/component.test.tsx
```

For real layout, add or reuse a fixture in
`apps/gallery/ui/layout-fixture-components.tsx` and its focused contract in
`apps/gallery/tests/layout.ts`. Prefer semantic node lookup plus explicit overflow or
collision checks over full-tree snapshots.

```bash
# Reuse the current release CLI and fixture bundle while editing TSX/styles.
bun run test:layout:quick widgets/Button widgets/Card
# Restrict the quick loop to one application when unrelated fixtures are dirty.
bun run test:layout:quick --app pi-agent shell/sidebar

# Rebuild packages, release CLI, and fixture bundle after Rust, dependency,
# generated-output, or fixture-registry changes, and before committing.
bun run test:layout
```

The layout runner enables Solid development diagnostics. Treat
`[STRICT_READ_UNTRACKED]` and `[REACTIVITY_HALTED]` as test failures, not ignorable
logs. If a warning only appears during application startup, exercise the real entry
with `renderAppLayout` from `@wabou/test/layout/node`; the captured diagnostic stack
is source-mapped by the runtime.

Use `waitMs` only for an authored timer, promise, or finite animation. Never add a
sleep merely to make a flaky assertion pass. Use `page.waitForIdle()` in native
behavior tests to cross completed JS/native frame boundaries; it intentionally does
not wait for infinite animations.

After changing a shared component, require:

- a component Vitest contract;
- a layout fixture when it owns geometry, clipping, scrolling, overlay placement,
  native paint, or responsive behavior;
- a behavior test only for a genuinely native interaction;
- a pixel capture only for a paint-specific regression.

When an application provides a deterministic service/process fixture, use it for
behavior tests instead of a live network service. Read the application's README or
verification script for the required environment injection. This preserves the real
Solid → capability → Rust service → child process → host-message path without making
the test depend on credentials or remote output.

## Start with state

Run:

```bash
git status --short
git rev-parse --short HEAD
```

Preserve unrelated changes. Record whether the user runs `dev`, `run`, or a packaged binary.

Restart the native process after Rust changes. Vite/Solid HMR updates JS and Style IR only; it does not reload `wabou-shell`, `wabou-runtime`, Vello scene code, or Rust widgets.

If a failure indicates stale frontend bundles or Vite caches, use `wabou clean
[APP]` before rebuilding instead of manually deleting broad directories. Add
`--packages` only when local package `dist` artifacts are themselves stale; the
command intentionally does not replace `cargo clean` or dependency installation.

## Isolate the failing layer

Trace one suspect node through these layers:

1. Static JSX candidate extraction and Wabou preset output.
2. Applied classes and typed style in `ComputedNodeSnapshot`.
3. Taffy `PlacedNode`: border box, content box, overflow clip, radius, depth.
4. Vello scene composition: transform, clip stack, `Scene::append`, layer lifetime.
5. Surface/backend: logical size, physical size, device scale, GPU backend.

Prefer an assertion at the earliest incorrect layer. If geometry is correct but pixels are wrong, add an offscreen render or platform-specific reproduction instead of more layout assertions.

## Inspect a running app when deterministic tests cannot isolate it

Start with DevTools enabled:

```bash
mise exec -- bun run wabou dev apps/gallery --devtools
```

Then inspect the discovered socket. Query and validate the tree before taking any
screenshot:

```bash
mise exec -- bun run wabou inspect status
mise exec -- bun run wabou inspect query fractal
mise exec -- bun run wabou inspect node <id>
mise exec -- bun run wabou inspect validate
```

Run `validate` before interpreting pixels. An invalid report means the
retained tree, geometry, clip chain, interaction targets, or semantic
references are internally inconsistent; diagnose that earliest failing layer
before relying on the screenshot. Warning-only reports remain structurally
valid and commonly identify rejected style declarations.

Use the DevTools `Layout` control for native overlays:

- blue: layout bounds
- orange: overflow/scroll clips
- red: current hit target
- purple: selected node

The MCP tool `wabou_set_layout_overlay` exposes the same target-window overlay for agent-driven diagnosis.

## Capture pixels only after earlier layers pass

First record why component, protocol, layout, and behavior evidence cannot prove the
property. Then use `scripts/capture-png.sh` for a deterministic offscreen render:

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
Prefer a focused crop or pixel assertion over repeatedly judging the whole app by eye.

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

- Component state/composition bug: `@wabou/test/component` Vitest test.
- Parser/preset bug: TypeScript unit test for candidate → typed Style IR.
- Cascade bug: `computed_style` snapshot test.
- Layout/clip/scroll/collision bug: batched TypeScript layout fixture; use a lower-level
  `wabou-shell::layout` test only for engine internals.
- Reactive warning/effect bug: development-mode `renderAppLayout` or layout fixture;
  the command must fail on the diagnostic.
- JavaScript event bug: component test; use encoded op/event replay only when native
  routing is in question.
- Native input/window/tray bug: focused behavior scenario with semantic and state
  assertions.
- Rust service, child-process, or host-message integration: deterministic application
  fixture plus a focused behavior scenario.
- Scene/backend bug: offscreen pixel test or a minimal platform reproduction.
- Resize/maximize bug: dispatch real `WindowMetrics` transitions; do not test only the initial size.

An op replay can verify deterministic tree, style, layout, invalidation, and events. It cannot alone verify GPU pixels or OS surface behavior. Pair it with a screenshot/pixel assertion for rendering regressions.

## Validate proportionally

Run the narrow test first, then the affected verification layer:

```bash
cargo test -p wabou-shell --lib
bun --conditions=browser --conditions=wabou-source test path/to/test.tsx
bun run test:layout:quick affected/Fixture
bun run test:layout
bun x tsc --noEmit
git diff --check
```

For a behavior run, select an explicit artifact directory while diagnosing:

```bash
bun run wabou test /path/to/app --artifacts /tmp/wabou-app-test
```

Do not infer success from the process exit alone. Read `report.json`, confirm the
intended named tests ran, and use their `traceStart`/`traceEnd` ranges with
`trace.json` when isolating a failure. A green deterministic behavior report proves
the exercised semantic/native state machine, not GPU pixels or platform integration.

Do not run every command mechanically; select commands that prove the changed layer.
Run the full layout command before committing layout-affecting shared components.
Report what was directly reproduced, the layer tested, the platform/scale when
relevant, and what remains inferred. Never say a visual bug is fixed based only on
compilation. Never claim pixel correctness from geometry tests, and never claim
layout correctness merely because a screenshot looked acceptable.
