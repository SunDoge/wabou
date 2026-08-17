# TanStack Router Core experiment

This spike evaluates `@tanstack/router-core` as the state and matching engine
behind a future Wabou-native Solid router. It deliberately uses:

- a memory history instead of browser history;
- Solid 2 signals as the core store adapter;
- typed search validation, path parameters, route context, async loaders,
  navigation, and history restoration;
- an explicit native origin with the browser coordinator disabled.

Run the behavior test and produce the QuickJS bundle with:

```sh
mise exec -- bun --conditions=browser test \
  experiments/tanstack-router-core/index.test.ts
mise exec -- bun build experiments/tanstack-router-core/index.ts \
  --target browser --format iife --minify \
  --define 'process.env.NODE_ENV="production"' \
  --outfile target/router-core-experiment/router.js
mise exec -- cargo run -p wabou-runtime --example eval-bundle -- \
  target/router-core-experiment/router.js
```

## Findings

- The Bun/V8 behavior test passes for params, validated search, route context,
  async loaders, navigation, and memory history.
- The adapter must use Solid 2's `flush` transaction and explicit literal
  signal writes. The official Solid adapter targets Solid 1 and cannot be used
  unchanged.
- Browser-targeted Router Core expects `self` and `AbortController`. `self` is
  a trivial adapter alias; cancellation should become a general Wabou Web API.
- In Wabou QuickJS, `router.load()` currently resolves before the route-match
  presentation is published. The result reports `compatible: false`: location
  changes work, while matches and loader data remain empty. This remains true
  with Router Core's browser coordinator disabled, so adopting the core before
  fixing or adapting this async boundary would make navigation unreliable.
- The minified experiment is about 81 KB versus a 19 KB Solid/timer baseline;
  gzip is about 29 KB versus 8 KB. The measured incremental cost is therefore
  roughly 62 KB raw or 22 KB gzip.

Recommendation: keep the current router for now. Router Core is still a good
candidate once a small native adapter can make its load/commit lifecycle pass
the included QuickJS probe; then Wabou can reuse typed search, loaders, caching,
preloading, guards, pending/error states, and blockers without importing its
DOM-facing Solid components.

The experiment is not a public router API. A production adapter would still
need Wabou components for `RouterProvider`, `Outlet`, links, pending/error UI,
navigation blockers, and native scroll restoration.
