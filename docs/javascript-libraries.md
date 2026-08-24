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
| `solid-js` | 2.0.0-rc.1 | Preview baseline | Drives every Wabou application through `@solidjs/universal` 2.0, including signals, memos, effects, context, lifecycle, `For` and `Show`. Wabou deliberately targets the Solid 2 RC during its own developer preview; pin the exact workspace-compatible version. Use Wabou's renderer aliases; DOM-specific APIs remain unsupported. |
| `valibot` | 1.4.2 | Supported | Runs without polyfills in QuickJS. Warden Desktop validates native response envelopes, preferences, login results, vault snapshots and item details with it. An isolated email/URL/union probe tree-shook to about 4.5 KB minified. |
| `ai` | 7.0.77 | Supported with buffered network streams | `generateText` and `streamText` run in Wabou QuickJS with both a mock provider and a one-time real OpenRouter request. The isolated AI SDK Core probe was about 556 KB minified / 141 KB gzip; adding the OpenAI-compatible provider produced about 910 KB minified / 222 KB gzip. Wabou exposes `Response.body`, WHATWG Streams, and Encoding Streams, so AI SDK can parse JSON and SSE. The Rust fetch bridge still receives the complete body before publishing one JavaScript chunk, so token events are not delivered incrementally yet. Keep model orchestration in application JavaScript; do not add AI SDK to `@wabou/core`. |
| `web-streams-polyfill` | 4.3.0 | Supported | The WHATWG reference-implementation-based ponyfill supplies readable, writable, transform, BYOB, controller, reader, writer, and queuing-strategy constructors. `@wabou/core` installs missing constructors without replacing native implementations. Transform pipelines, async iteration, and writable piping are verified in both TypeScript and embedded QuickJS. |
| `@stardazed/streams-text-encoding` | 1.0.2 | Supported | Supplies `TextEncoderStream` and `TextDecoderStream` on top of Wabou's WHATWG streams. Split UTF-8 byte sequences and split UTF-16 surrogate pairs are covered in TypeScript, and decoding is exercised in embedded QuickJS. |
| `lucide-static` | 1.31.0 | Supported | Wabou components consume framework-independent SVG strings and send them through the cached native SVG path. This avoids DOM assumptions and keeps icon selection tree-shakeable. |
| `@floating-ui/core` | 1.8.0 | Supported through a Wabou adapter | Wabou primitives supply native layout rectangles to its platform-independent middleware. `offset`, `flip`, `shift`, `size`, `arrow` and `autoPlacement` are tested. Do not use `@floating-ui/dom`. |
| `whatwg-url` | 17.1.0 | Runtime implementation detail | Wabou bundles it into the core prelude to provide `URL` and `URLSearchParams`. Applications should use those globals rather than depending on this package directly. |

## Conditional or rejected libraries

| Library | Tested version | Status | Reason |
| --- | --- | --- | --- |
| `arktype` | 2.2.3 | Not supported out of the box | Its bundle reaches QuickJS but module initialization unconditionally reads browser constructors including `Blob`, `File`, `FormData`, `Headers`, `Request` and `Response`. A probe passed after installing placeholder constructors, including useful validation errors, but fake Web APIs would create a misleading compatibility contract. Prefer Valibot until Wabou implements the required APIs for real or ArkType guards those references. The isolated minified probe was about 157 KB. |
| DOM-targeted component libraries | any | Unsupported unless adapted | Libraries that require `document`, `HTMLElement`, DOM measurement, CSSOM, browser portals or synthetic browser events cannot run directly. A Solid package name alone does not imply Wabou compatibility. |

Build-time tools such as Vite, UnoCSS and the Wabou style compiler run under
Bun/Node during development; their compatibility does not imply that their
runtime APIs exist inside QuickJS.

## Compatibility harness

Pure JavaScript libraries do not need to mount a Solid tree or enter the
layout renderer. Bundle the probe for a browser target, boot it directly in
`wabou_runtime::JsRuntime`, and use `eval_promise_json` to drive QuickJS jobs
until a JSON-serializable result or rejection is available:

```rust
use std::time::Duration;
use wabou_runtime::{JsRuntime, JsRuntimeOptions};

let mut runtime = JsRuntime::new_with_options(
    JsRuntimeOptions::default().max_stack_size(8 * 1024 * 1024),
)?;
runtime.boot(&bundle)?;
let result = runtime.eval_promise_json(
    "runLibraryProbe()",
    Duration::from_secs(5),
)?;
```

This runtime installs Wabou's native host functions and core prelude without
creating Taffy, renderer scenes, widgets, or a native window. The JavaScript bundle should
import `@wabou/core` when it needs Wabou's higher-level Fetch, Crypto, Streams,
timer, or other compatibility wrappers. Use the UI headless runner only when a
probe also needs Solid reconciliation, Style IR, layout, or paint.

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
