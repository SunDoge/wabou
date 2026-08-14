# Solid 2 experiment

This branch evaluates Solid 2 as Wabou's renderer-neutral reactive runtime.
It is intentionally not a production migration yet.

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
mise exec -- bun run check:solid2
mise exec -- bun run test:solid2
mise exec -- bun run --cwd apps/stress build
mise exec -- bun run wabou render apps/stress \
  --out /tmp/wabou-solid2-stress.png --width 1100 --height 776
```

The focused checks use Bun's `browser` condition. Without it Bun resolves the
server Solid runtime, whose effects are intentionally inert.

## Deliberately excluded

`apps/hackernews` is excluded by `tsconfig.solid2.json` because the current
`lucide-solid` release declares a Solid 1 peer dependency and installs a
second Solid 1 runtime. It should be restored only after the icon package has
native Solid 2 support. `bun run check:all` deliberately retains the
unfiltered check so this exception remains visible.

The Solid 2 Vite plugin currently declares Vite 6-8 support. Wabou's focused
Vite 5 production build succeeds, but this unsupported combination is another
reason not to merge the experiment before the surrounding ecosystem settles.

## Renderer ABI changes found

- JSX types are owned by `@wabou/solid-renderer/jsx-runtime` rather than
  `solid-js`.
- The universal runtime moved to `@solidjs/universal`.
- Compiled refs use the renderer's `ref` and `applyRef` helpers.
- Static JSX properties are passed to `createElement(tag, staticProps)` and
  must be applied by the renderer.
- Effects use separate compute and apply functions; cleanup is returned from
  the apply function.
