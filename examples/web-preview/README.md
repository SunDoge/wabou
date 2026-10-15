# Browser canvas preview

Run from the repository root:

```sh
bun scripts/web-preview.ts dev
```

Open the Vite URL printed in the terminal. Edit `app.tsx` to compose Wabou
`view` nodes with Solid, inline styles or the existing UnoCSS/Style IR classes.
The preview uses the browser's JavaScript engine, shared Rust Taffy/Parley layout,
and Vello WebGL2 canvas rendering. It does not use DOM elements for the Wabou tree
and does not embed QuickJS.

The application entry is identical to a native Wabou entry:

```tsx
import { mount } from "@wabou/ui";
import "virtual:wabou-stylesheet";
import { App } from "./app";

mount(() => <App />);
```

`defineWabouConfig({ target: "browser", browser: { canvas, runtimeModule, fonts } })`
generates the host bootstrap. The application does not initialize Wasm, intern
protocol atoms, flush writers, dispatch browser input or manage a frame loop.
Canvas readiness/errors are exposed as `wabou:ready` and `wabou:error` events for
the surrounding page. `mount` returns the same component disposer on both targets;
the generated host owns runtime disposal. Embedders implementing their own host
can use `@wabou/core/browser`'s `startBrowserApp`, which returns a host disposer.
Only one host may own a JavaScript realm; disposal is idempotent, and a later host
gets a fresh protocol/interner lifetime.

Prerequisites: Bun and workspace dependencies, Rust's `wasm32-unknown-unknown`
target (`rustup target add wasm32-unknown-unknown`), and `wasm-bindgen-cli` **0.2.127**
(`cargo install wasm-bindgen-cli --version 0.2.127 --locked`). The script invokes
`cargo` from PATH, including wrappers such as mbx. It builds Rust once before
starting Vite; TSX changes subsequently use Vite's update path without rebuilding
Rust. The initial host remounts changed components, so their local signals may
reset; state-preserving UI HMR is not guaranteed. Rust
changes require restarting the command. Generated Wasm and the static site go
under `target/web-preview`; they are not committed.

Build a deployable static preview:

```sh
bun scripts/web-preview.ts build
```

Serve `target/web-preview/site` over HTTP. The preview font is bundled with its
license so browser layout does not depend on the machine's installed fonts.

Browser verification, against a running preview:

```sh
bun scripts/test-web-preview.ts http://127.0.0.1:5173/
```

This launches an isolated headless Chrome/Chromium, checks counter updates,
reset, narrow layout and 2× scaling/hit testing, and captures actual GPU pixels.
Set `CHROME` if its executable is not on PATH. Reports and PNGs are written to
`target/web-preview/proof`. Pass `--hmr` against a **dev** server to also
verify a TSX update retains the same Wasm host. This optional check temporarily
changes the example header and restores the exact source afterward.

## Boundaries of this first preview

The browser host currently accepts structural `view` nodes, text, Style IR class
rules, inline style changes, tree insert/remove/drop, and click events. It exposes
the exact Rust layout through `globalThis.__wabou_preview.snapshot()`.

It does **not** yet implement the native widget registry (inputs, editors,
terminal, etc.), keyboard/IME editing, scroll/focus operations, OS capabilities,
or loading application Rust dylibs. Unsupported widget tags or protocol
operations produce a visible error instead of silently drawing an imitation.
The shared WebGL painter can replay retained images, SVG, clips, shadows, paths
and glyphs; wiring their application-level resource/protocol operations is
separate work. Custom WGSL effects require a future WebGPU backend.

Only one renderer mount is supported per JavaScript realm. Place independent
previews in separate iframes; the existing Solid renderer's writer/interner and
mount state are globals. This is an initial canvas preview backend, not full
native/browser parity or a general browser accessibility adapter.

## Crate ownership

- `wabou-render-core`: shared styles, layout, fonts, paint scenes and widget
  contracts. No Winit, clipboard, dialogs or QuickJS dependency.
- `wabou-shell-vello`: native window host, input/platform services and resident
  widgets. Existing public rendering paths remain compatibility re-exports.
- `wabou-web`: Wasm/browser protocol adapter and canvas entry points; JavaScript
  execution stays in the browser.

The preview's initial composition is deliberately a compact counter with one
primary action. Loading/error messages sit outside the canvas. Responsive
layout and captured 1×/2× pixels are the visual acceptance evidence.
