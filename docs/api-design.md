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

`useFps` previously created a requestAnimationFrame loop and interval, so it
violated this rule. The canonical name is `createFps`; `useFps` remains only as
a deprecated compatibility alias.

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
3. Embedded integration tests bundle the real public `@wabou/core` entry into
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

## TypeScript declarations for application capabilities

`HostCapabilities` and `WabouIntrinsicElements` are renderer extension
registries. Generated application bindings manage the internal augmentation;
ordinary application code imports `Host`, `useHost`, and renderer types from
`@wabou/core`. Native widget packages that add low-level intrinsic tags are the
one supported exception and augment the stable registry subpath directly:

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

Rust remains the source of truth for serialized request, response, event DTOs
and exported function signatures. Wabou uses Specta to reflect explicitly
annotated Rust DTOs and functions. `wabou-bindgen` adds the capability
namespace and Wabou wire policy, then generates the `HostCapabilities`
augmentation and typed client. Primitive arguments remain native QuickJS
arguments; structured values are JSON encoded; sync and Promise-like native
results are normalized at the client boundary.

Do not infer the public API by parsing arbitrary `rquickjs::Function` closures.
Their captured state, argument conversion, error policy, and Promise behavior
are not a stable schema. Do not generate declarations from persistence or SDK
models either: define explicit bridge DTOs so a database or upstream SDK change
cannot silently become a JavaScript API change.

Typeshare is appropriate when several foreign languages are a primary
requirement. A full RPC framework is not required for
the in-process QuickJS capability boundary, and would couple the renderer to
routing and transport concerns it does not have.

Generated files must be committed and checked for drift in CI. Generation is
an explicit `write` operation; the corresponding `check` operation generates
in memory and fails when the committed output differs. Ordinary compilation
and build scripts must not rewrite the source tree.

Framework-owned synchronous host calls use the same Rust/Specta source but a
flat `FunctionModule` rather than an async capability client. The generated
`NativeHostApi` checks the TypeScript adapter for URL opening, font loading,
frame diagnostics and layout snapshots while preserving their synchronous
semantics. Per-frame binary rendering operations and guest callbacks remain a
separate, versioned ABI because routing those through JSON RPC would add cost
and erase useful protocol constraints.
