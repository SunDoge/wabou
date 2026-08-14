# Solid 2 runtime

Wabou uses Solid 2 as its renderer-neutral reactive runtime.

## Transaction boundary

Solid 2 batches signal writes until a microtask or an explicit `flush()`.
Wabou treats one decoded native `HostEventFrame` as the explicit boundary:

```text
native HostEventFrame
  -> dispatch node, resize, and application records
  -> update Solid signals
  -> flush Solid once
  -> leave generated binary operations queued for the host tick
```

This prevents intermediate reactive states from becoming separate native
frames while retaining Wabou's existing writer and render scheduling.

## Verify

```sh
mise exec -- bun run check
mise exec -- bun run test
mise exec -- bun run --cwd apps/stress build
mise exec -- bun run wabou render apps/stress \
  --out /tmp/wabou-solid2-stress.png --width 1100 --height 776
```

The tests use Bun's `browser` condition. Without it Bun resolves the
server Solid runtime, whose effects are intentionally inert.

## Ecosystem boundary

Icons use `lucide-static` SVG sources instead of a framework-specific wrapper,
so no Solid 1 runtime is installed. Wabou's router, interactions, primitives,
and components are tested directly against Solid 2.

The Solid Vite plugin owns component refresh through `solid-js/refresh`.
The obsolete standalone `solid-refresh` package is intentionally not used.

## Renderer ABI changes found

- JSX types are owned by `@wabou/solid-renderer/jsx-runtime` rather than
  `solid-js`.
- The universal runtime moved to `@solidjs/universal`.
- Compiled refs use the renderer's `ref` and `applyRef` helpers.
- Static JSX properties are passed to `createElement(tag, staticProps)` and
  must be applied by the renderer.
- Effects use separate compute and apply functions; cleanup is returned from
  the apply function.
