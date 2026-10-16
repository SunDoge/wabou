# JavaScript API design and test contract

Wabou follows Solid's naming model rather than React's hook naming model.
Names describe ownership and lifetime, not merely whether an API is commonly
called from a component.

## Naming rules

- `createXxx` constructs state, a controller, or a resource. Pure factories
  such as `createMemoryHistory` can be owner-independent; factories with
  effects, animation frames, timers, or subscriptions must bind cleanup to the
  current Solid owner. Examples: `createHover`, `createPress`, `createTabs`,
  `createScrollReset`, and `createFps`.
- `useXxx` reads a service or value from context. It must have a Provider seam
  so a test or subtree can replace the service. Examples: `useHost`,
  `useWindow`, `useClipboard`, router hooks, and `useComponentsTheme`.
- Imperative process-wide or runtime-wide operations use nouns or explicit
  verbs. Examples: `clipboard`, `currentWindow`, and `createWindow`.
- A function does not gain a `use` prefix merely because components call it.
- Ecosystem protocol names remain intact when compatibility is the API:
  `createHotContext` follows Vite, while renderer-level `createElement`,
  `createTextNode`, and `createComponent` follow Solid's renderer contract.

## Current API review

| Family | APIs | Decision |
| --- | --- | --- |
| Owned interaction primitives | `createHover`, `createFocus`, `createFocusWithin`, `createPress`, `createActive`, `createButton` | Correct: each constructs local reactive state and bindings. |
| Stateful controllers | `createTabs`, `createShortcuts`, `createMemoryHistory` | Correct: each returns a new independently testable controller; no ambient host is required. |
| Owned effects | `createScrollReset`, `createFps` | Correct: both create effects or scheduled work and rely on Solid-owner cleanup. |
| Context consumers | `useHost`, `usePlatformServices`, `useWindow`, `useClipboard`, `useDialog`, `useNotification`, router hooks, `useComponentsTheme` | Correct: each reads a Provider-backed value; required contexts fail clearly while services with a real runtime default use that default. |
| Imperative native resources | `createWindow`, `currentWindow`, `clipboard`, `dialog`, `notification` | Correct: these are callable outside a component and do not pretend to be context hooks. |

`HostProvider` and `PlatformProvider` deliberately replace different layers.
`HostProvider` replaces renderer-facing layout, diagnostics, font, and system
capabilities. `PlatformProvider` replaces application-facing, window-scoped
services such as clipboard and window state. A partial nested
`PlatformProvider` inherits services it does not override.

The FPS primitive creates a requestAnimationFrame loop and interval, so its
canonical name is `createFps`; Wabou does not retain a hook-shaped compatibility
alias for an API that owns scheduled work.

`useWindow`, `useClipboard`, `useDialog`, and `useNotification` return the
native service for the current QuickJS window. `PlatformProvider` can override these services for a Solid
subtree. This makes the names honest context consumers and lets component tests
avoid mutating globals or loading a real clipboard.

## Host API test levels

Host-facing APIs need evidence at three distinct boundaries:

1. TypeScript unit tests inject host functions or `PlatformProvider` services
   and verify public API behavior, reactivity, errors, and concurrent requests.
2. ABI drift tests use the generated inventory from
   `packages/core/host-abi.json` and compare it with the globals installed in a
   real QuickJS `Applier`. Adding or renaming one side without regenerating or
   implementing it fails.
3. Embedded integration tests bundle the lower-level `@wabou/core` runtime entry into
   `gen/test-runtime.js`, execute it in QuickJS, inspect emitted Rust
   native effects, complete them as the shell would, run Promise jobs, and assert
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

1. Add the private ABI entry to `packages/core/host-abi.json` and regenerate.
2. Install the Rust function before the application bundle boots.
3. Expose a typed public API; do not expose the private global directly.
4. Provide a context injection seam when the API is consumed as `useXxx`.
5. Test public behavior in TypeScript, ABI presence in QuickJS, and one full
   public-API-to-`HostAction` round trip.
6. Test the native platform layer separately when correctness depends on the
OS rather than only on routing and serialization.

## TypeScript declarations for application host modules

`HostCapabilities` and `WabouIntrinsicElements` are low-level renderer
extension registries. Ordinary application code imports `Host`, `useHost`, and
renderer types from `@wabou/ui`. Native widget packages that add intrinsic tags
or deliberately expose direct synchronous host functions may augment the
stable registry subpath:

```ts
declare module "@wabou/core/registry" {
  interface HostCapabilities {
    readonly workspace: {
      readFile(request: ReadFileRequest): Promise<ReadFileResult>;
    };
  }

  interface WabouIntrinsicElements {
    chart: WabouElementProps & { series: string };
  }
}
```

Rust remains the source of truth for request, response, and event DTOs.
Application methods are ordinary functions in an `#[rquickjs::module]`.
`JsValue<T>` uses strict Serde conversion between Rust DTOs and ordinary
QuickJS values, so there is no JSON string envelope or parallel client runtime.
Async Rust functions naturally appear as JavaScript Promises.

Wabou's inert `#[wabou::js_module]`, `#[wabou::js]`, `#[wabou::js_type]`,
and `#[wabou::js_class]` markers describe the public contract beside the
registered Rust item. The source scanner emits ambient TypeScript declarations
without compiling the application. It does not infer a contract from Rust ABI
types, persistence models, or arbitrary closures.

Host state is stored as an owned `HostModuleState<T>` in QuickJS runtime
userdata. Functions request `Ctx` as their first argument and retrieve that
state when called. They must never capture `Ctx`, `Object`, or another
QuickJS-lifetime handle inside long-lived Rust closures.

Generated files must be committed and checked for drift in CI. Generation is
an explicit `write` operation; the corresponding `check` operation generates
in memory and fails when the committed output differs. Ordinary compilation
and build scripts must not rewrite the source tree.

Host modules are appropriate for commands, resource operations, and coarse
request/response work. Per-frame rendering and other batchable operations
remain a separate versioned binary mutation ABI; host functions do not replace
it.
