# JavaScript library compatibility

This page records libraries that have been executed against Wabou's embedded
QuickJS runtime. A successful Vite or Bun build is not enough: runtime claims
require the bundled code to boot in `JsRuntime` and exercise meaningful success
and failure paths.

Versions are the versions tested in this repository. Treat an upgrade as
untested until the same checks pass again.

## Runtime libraries

| Library | Tested version | Status | Evidence and constraints |
| --- | --- | --- | --- |
| `solid-js` | 2.0.0-rc.0 | Preview baseline | Drives every Wabou application through `@solidjs/universal` 2.0, including signals, memos, effects, context, lifecycle, `For` and `Show`. Wabou deliberately targets the Solid 2 RC during its own developer preview; pin the exact workspace-compatible version. Use Wabou's renderer aliases; DOM-specific APIs remain unsupported. |
| `valibot` | 1.4.2 | Supported | Runs without polyfills in QuickJS. Warden Desktop validates native response envelopes, preferences, login results, vault snapshots and item details with it. An isolated email/URL/union probe tree-shook to about 4.5 KB minified. |
| `lucide-static` | 1.31.0 | Supported | `@wabou/components` consumes framework-independent SVG strings and sends them through Wabou's cached native SVG path. This avoids DOM assumptions and keeps icon selection tree-shakeable. |
| `@floating-ui/core` | 1.8.0 | Supported through a Wabou adapter | `@wabou/primitives` supplies native layout rectangles to its platform-independent middleware. `offset`, `flip`, `shift`, `size`, `arrow` and `autoPlacement` are tested. Do not use `@floating-ui/dom`. |
| `whatwg-url` | 17.1.0 | Runtime implementation detail | Wabou bundles it into the core prelude to provide `URL` and `URLSearchParams`. Applications should use those globals rather than depending on this package directly. |

## Conditional or rejected libraries

| Library | Tested version | Status | Reason |
| --- | --- | --- | --- |
| `arktype` | 2.2.3 | Not supported out of the box | Its bundle reaches QuickJS but module initialization unconditionally reads browser constructors including `Blob`, `File`, `FormData`, `Headers`, `Request` and `Response`. A probe passed after installing placeholder constructors, including useful validation errors, but fake Web APIs would create a misleading compatibility contract. Prefer Valibot until Wabou implements the required APIs for real or ArkType guards those references. The isolated minified probe was about 157 KB. |
| DOM-targeted component libraries | any | Unsupported unless adapted | Libraries that require `document`, `HTMLElement`, DOM measurement, CSSOM, browser portals or synthetic browser events cannot run directly. A Solid package name alone does not imply Wabou compatibility. |

Build-time tools such as Vite, UnoCSS and the Wabou style compiler run under
Bun/Node during development; their compatibility does not imply that their
runtime APIs exist inside QuickJS.

## Adding a compatibility entry

For a platform-independent library:

1. Bundle a minimal application with the same Vite/Bun target used by Wabou.
2. Boot the bundle in `wabou_runtime::JsRuntime`, without adding undeclared
   globals.
3. Exercise a valid result, an invalid/error result, and any asynchronous path
   the application will use.
4. Run TypeScript checks and the affected production application build.
5. For rendered output or platform behavior, also capture and inspect the
   relevant 1× and 2× native scenes.

Document required adapters and unsupported subpackages precisely. Avoid broad
claims such as “library X works” when only one DOM-free module was tested.
