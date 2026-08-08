# Wabou CLI

Wabou applications use a convention-based layout with no framework config
file:

```text
my-app/
├── Cargo.toml
├── package.json
├── vite.config.ts
├── src/
│   └── main.rs
└── ui/
    └── index.tsx
```

`src/` is the Rust host, `ui/` is the Solid application, and the default Vite
entry is `ui/index.tsx`. Shared Rust packages belong in `crates/`; shared
JavaScript packages belong in `packages/`. From an app directory the name can be omitted;
from this repository root it selects `apps/gallery` by default.

The app's Vite config delegates Wabou's compiler, Solid universal renderer,
aliases, optimizer policy, and bundle defaults to `@wabou/vite`:

```ts
import { defineWabouConfig } from "@wabou/vite";

export default defineWabouConfig({
  outDir: "../../dist/gallery/resources",
});
```

For a standalone app, `dist/` and Cargo's `target/` are siblings at the app
root. Inside a Cargo workspace, both directories live at the workspace root and
each application gets its own `dist/<app>/` directory.

## Development

```bash
bun run wabou dev --app-dir apps/gallery
bun run wabou dev --app-dir apps/hackernews --devtools
```

Development starts Vite and compiles the Rust host with `wabou-quick/vite`.
QuickJS imports the live Vite graph and receives HMR updates. No JavaScript
production build is required before Rust compilation. Ctrl-C terminates Vite,
the host, and the optional inspector.

## Run and package

```bash
bun run wabou run --app-dir apps/gallery
bun run wabou build --app-dir apps/gallery --release
bun run wabou devtools
```

`run` builds the UI and supplies its path to the Rust host through
`WABOU_BUNDLE_PATH`; changing the bundle never recompiles Rust.

`build` produces a self-contained application directory beside Cargo's
`target/` directory:

```text
dist/gallery/
├── gallery
└── resources/
    └── bundle.js
```

The packaged executable resolves `resources/bundle.js` relative to itself, so
it runs without the source tree or CLI. On macOS the same resource contract can
later map to `Gallery.app/Contents/Resources/wabou` before signing.

## Rust-owned TypeScript bindings

An application can expose a conventional Cargo example named
`wabou-bindings`. The example builds a `wabou_bindings::Bindings` manifest and
writes its generated module beneath `ui/`:

```bash
bun run wabou bindings --app-dir apps/gallery write
bun run wabou bindings --app-dir apps/gallery check
```

`write` explicitly updates the committed declaration. `check` never rewrites
files and fails when a Rust DTO, capability name, or method name has drifted;
run it in CI.

The generated module exposes a typed client while accurately retaining the
native `string -> Promise<string>` QuickJS ABI underneath. Application code
passes DTO objects to the client and does not manually call `JSON.stringify`
or `JSON.parse`. See `apps/gallery/examples/wabou-bindings.rs` and
`apps/gallery/ui/generated/host-bindings.ts` for the complete convention.

Only explicit bridge DTOs should derive `TS`. Database entities, upstream SDK
models, and renderer internals are not public JavaScript contracts.
