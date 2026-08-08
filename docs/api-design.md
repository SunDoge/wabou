# JavaScript API design and test contract

Wabou follows Solid's naming model rather than React's hook naming model.
Names describe ownership and lifetime, not merely whether an API is commonly
called from a component.

## Naming rules

- `createXxx` creates reactive state, an effect, a timer, a history, or another
  resource owned by the current Solid root. Examples: `createHover`,
  `createPress`, `createTabs`, `createScrollReset`, and `createFps`.
- `useXxx` reads a service or value from context. It must have a Provider seam
  so a test or subtree can replace the service. Examples: `useHost`,
  `useWindow`, `useClipboard`, router hooks, and `useComponentsTheme`.
- Imperative process-wide or runtime-wide operations use nouns or explicit
  verbs. Examples: `clipboard`, `currentWindow`, and `createWindow`.
- A function does not gain a `use` prefix merely because components call it.

`useFps` previously created a requestAnimationFrame loop and interval, so it
violated this rule. The canonical name is `createFps`; `useFps` remains only as
a deprecated compatibility alias.

`useWindow` and `useClipboard` return the native service for the current
QuickJS window. `PlatformProvider` can override either service for a Solid
subtree. This makes the names honest context consumers and lets component tests
avoid mutating globals or loading a real clipboard.

## Host API test levels

Host-facing APIs need evidence at three distinct boundaries:

1. TypeScript unit tests inject host functions or `PlatformProvider` services
   and verify public API behavior, reactivity, errors, and concurrent requests.
2. ABI drift tests parse the host-provided declarations in
   `packages/core/src/host.ts` and compare them with the globals installed in a
   real QuickJS `Applier`. Adding or renaming one side without the other fails.
3. Embedded integration tests bundle the real public `@wabou/core` entry into
   `gen/test-runtime.js`, execute it in QuickJS, inspect emitted Rust
   `HostAction`s, complete them as the shell would, run Promise jobs, and assert
   the public JavaScript result.

The generated fixture exposes only a test namespace,
`globalThis.__wabou_test_host_api`. Production applications still use package
exports; private `__wabou_*` globals are not public API.

This setup guarantees that an API is both mockable and executable in the
embedded engine. It does not prove operating-system behavior. Clipboard,
window-system, GPU, font, and HiDPI behavior still require the corresponding
native/platform test.

## Review checklist

For a new host API:

1. Add the private ABI declaration to `packages/core/src/host.ts`.
2. Install the Rust function before the application bundle boots.
3. Expose a typed public API; do not expose the private global directly.
4. Provide a context injection seam when the API is consumed as `useXxx`.
5. Test public behavior in TypeScript, ABI presence in QuickJS, and one full
   public-API-to-`HostAction` round trip.
6. Test the native platform layer separately when correctness depends on the
   OS rather than only on routing and serialization.
