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
- Browser-targeted Router Core expects `self`, `AbortController`, and
  `Response`. Wabou now supplies the global alias, a cancellation polyfill, and
  a small Fetch API object layer backed by its existing native fetch bridge.
- The original QuickJS failure was not an async scheduler incompatibility.
  Router Core checks loader results with `value instanceof Response`; the
  missing global threw inside a transaction that Router Core rolled back. With
  the Web API surface installed, the same QuickJS probe passes params, search,
  loaders, navigation, and history restoration.
- The complete minified experiment is about 93 KB raw or 33 KB gzip, including
  Solid, Router Core, URL/fetch/timer compatibility, and cancellation support.

Recommendation: use Router Core as Wabou's router engine. This lets apps reuse
typed search, loaders, caching, preloading, guards, pending/error states, and
blockers without importing TanStack's DOM-facing Solid components or
maintaining a second lightweight router.

The experiment informed `@wabou/router`'s native `createDataRouter`,
`RouterProvider`, and hooks. Links, navigation-blocker UI, and native scroll
restoration remain Wabou-owned presentation concerns.
