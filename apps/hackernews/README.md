# Hacker News

The application uses deterministic local Hacker News-shaped fixtures and is
rendered by Wabou's Solid renderer. It intentionally performs no network
requests, so layout and interaction regressions are reproducible offline.

Set `VITE_HN_LIVE=1` to opt into the live Hacker News API. Convenience scripts
are available as `bun run dev:live` and `bun run build:live`.

## Production bundle

From the workspace root:

```sh
bun run --cwd apps/hackernews build
cargo run -p wabou-quick --bin wabou-quick -- \
  --js dist/hackernews/resources/bundle.js
```

## Development with HMR

Run Vite in one terminal:

```sh
bun run --cwd apps/hackernews dev
```

Then start Wabou in another terminal:

```sh
cargo run -p wabou-quick --features vite --bin wabou-quick -- \
  --vite http://127.0.0.1:5174 \
  --entry ui/index.tsx
```

Changes to TSX update the Solid component and regenerate the typed Style IR in
the same HMR transaction.

### HMR behaviour

- **Accepted updates** — solid-refresh swaps the component; Style IR is pushed
  through `virtual:wabou-stylesheet` → `__wabou_set_stylesheet` in the same frame.
- **Declined / missing hot context / import error** — the host clears the scene
  tree and re-imports the Vite entry (in-process full reload). Check logs with
  `RUST_LOG=hmr=debug`.
- **Native Vite `css-update`** — ignored for layout; use Uno/class utilities so
  styles flow through the Style IR path above (not browser CSSOM).
