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

Create a standalone preview application with:

```bash
cargo install --git https://github.com/SunDoge/wabou.git \
  --tag v0.1.0-alpha.1 --locked wabou-cli
wabou new hello-wabou
cd hello-wabou
bun install
bun run dev
```

Until Wabou publishes crates.io and npm packages, `new` records the selected
Wabou revision as `vendor/wabou`, a Git submodule shared by Cargo and Bun. This
avoids accidentally combining Rust and JavaScript packages from different
commits. The default revision is `v0.1.0-alpha.1`; maintainers can exercise a
candidate commit with `--wabou-repository` and `--wabou-ref`.

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
each application gets its own `dist/<app>/` directory. The CLI overrides the
configured frontend directory with the selected `debug` or `release` artifact
directory; the configured path remains the direct `bun x vite build`
destination.

## Development

```bash
bun run wabou doctor
bun run wabou dev apps/gallery
bun run wabou dev apps/hackernews --devtools
```

`doctor` checks the required Rust and Bun tools, platform build dependencies,
the selected application, and generated workspace package artifacts. It exits
with a nonzero status when a required check fails, so it is also suitable for
setup scripts and CI diagnostics.

Development starts Vite and compiles the Rust host with the `wabou/vite` and
`wabou/devtools` facade features. QuickJS imports the live Vite graph and
receives HMR updates, while `wabou inspect` can discover the local DevTools
socket. `--devtools` additionally opens the visual inspector; it is not needed
for the socket. No JavaScript production build is required before Rust
compilation. Ctrl-C terminates Vite, the host, and the optional inspector.

The CLI invokes the project-local Vite executable directly. An application
does not need specially named `build` or `dev` package scripts; those scripts
are optional shortcuts such as `"build": "wabou build"`. Build customization
belongs in `vite.config.ts`, where it applies consistently to development,
tests, rendering, and packaging.

When running directly from the Wabou source workspace, the CLI checks the
runtime entrypoints declared by every `packages/*/package.json` before starting
Vite. Package builds are scheduled by Turbo according to workspace
dependencies, cached independently, and promoted into each `dist/` directory
atomically. The preflight lists missing files and asks for
`bun run packages:build`; unchanged packages are restored from the Turbo cache.
Source-to-artifact drift is checked deterministically by `bun run gen:check` in
CI rather than guessed by the runtime CLI;
standalone applications and published packages do not incur this workspace-only
check.

## Application verification

Use one application-level command before handing a change to another developer
or agent:

```bash
bun run wabou check apps/gallery
```

`check` verifies the nearest `tsconfig.json`, runs discovered `ui/**/*.test.ts`
and `ui/**/*.test.tsx` files with Solid's browser condition, runs Cargo against
every target in the application package, checks Rust-owned TypeScript bindings
when the app has a conventional `examples/wabou-bindgen.rs`, and runs every
discovered `tests/**/*.behavior.ts` scenario. Missing optional unit tests,
bindings, and behavior tests are skipped; failures are never rewritten
automatically. For a pass without native behavior scenarios, use
`--skip-behavior`; compile and unit checks still run.

## Run and package

```bash
bun run wabou run apps/gallery
bun run wabou build apps/gallery --release
bun run wabou build apps/gallery --release --source-map
bun run wabou package apps/gallery
bun run wabou devtools
```

To diagnose a release build without adding instrumentation to normal releases,
enable the compile-time profiler for that run only:

```bash
bun run wabou run apps/stress --release \
  --profile-trace /tmp/wabou-trace.json
```

See the [performance profiling guide](performance.md) for trace contents and
privacy guarantees.

`run` builds the UI and supplies its path to the Rust host through
`WABOU_BUNDLE_PATH`; changing the bundle never recompiles Rust.

`build` produces a self-contained, profile-specific application directory
beside Cargo's `target/` directory:

```text
dist/gallery/
├── debug/
│   ├── gallery
│   └── resources/
│       ├── bundle.js
│       └── bundle.js.map
└── release/
    ├── gallery
    └── resources/
        └── bundle.js
```

Debug builds generate JavaScript source maps by default; release builds do not.
Set `build.source-map` in `wabou.toml` to override that default, or pass
`--source-map`/`--source-map=false` for one build:

```toml
[build]
out-dir = "dist/resources"
source-map = true
```

The CLI profile and explicit command-line override are the source of truth for
both the Rust host and Vite. `package` always builds the release profile and
does not copy `bundle.js.map` into the staged application, even when release
source-map generation was explicitly enabled in configuration.

The packaged executable resolves `resources/bundle.js` relative to itself, so
it runs without the source tree or CLI. On macOS the same resource contract can
later map to `Gallery.app/Contents/Resources/wabou` before signing.

`package` always performs a release build, copies the executable and resources
into a deterministic `dist/<app>/stage/` directory, then invokes the typed
`cargo-packager` library embedded in `wabou-cli`. Users do not install a
separate packaging executable. Each application opts into native packaging
with `wabou.toml`:

```toml
[package]
product-name = "Example"
identifier = "dev.example.desktop"
description = "A native Wabou application."
authors = ["Example Team"]
license-file = "../../LICENSE"
icons = ["icons/*.png"]
resources = ["assets"]
formats = ["appimage", "deb"]
```

Command-line formats override the file without changing it:

```bash
bun run wabou package apps/gallery --format appimage
```

Supported adapters are `app`, `dmg`, `nsis`, `wix`, `deb`, `appimage`, and
`pacman`. Final artifacts are written to `dist/<app>/bundles/`; the generated
`packager.json` records the exact backend input for diagnostics and CI
reproduction. Package on the target operating system so platform signing and
native packaging tools are available.

For experimentally verified Cargo Zigbuild targets, glibc baselines, required
Linux sysroots, and the current boundary between cross-compilation and Wabou
packaging, see the [cross-compilation guide](cross-compilation.md).

## Behavior tests

`wabou test` discovers `tests/**/*.behavior.ts`, bundles the scenarios, and
evaluates them beside the normal application bundle in QuickJS. An application
directory is the normal entry point; a scenario file remains available for
focused debugging:

```bash
bun run wabou test /path/to/app
bun run wabou test /path/to/app/tests/window-lifecycle.behavior.ts
bun run wabou test --replay target/wabou-test/app/artifacts/trace.json \
  --app /path/to/app
```

For a scenario nested beneath a Wabou application, the CLI infers the nearest
application root. Use `--app` when the authored scenario lives elsewhere or
when intentionally running it against a different application.

The default deterministic backend does not require a display server. Use
`--native` for a real platform smoke test. See the [behavior testing
guide](testing.md) for semantic locators, artifacts and trace replay.

## Rust-owned TypeScript bindings

An application can expose a conventional Cargo example named
`wabou-bindgen`. The example builds a `wabou_bindgen::Bindings` manifest and
writes its generated module beneath `ui/`:

```bash
bun run wabou bindings write apps/gallery
bun run wabou bindings check apps/gallery
```

`write` explicitly updates the committed declaration. `check` never rewrites
files and fails when a Rust DTO, capability name, or method name has drifted;
run it in CI.

The generated module exposes a typed client while retaining the native
QuickJS ABI underneath. Each capability method accepts zero or one request;
the bridge JSON-encodes methods that have a request. A `no_request` method is
also argument-free at the native boundary; it does not rely on a hidden `null`
sentinel. Group
multiple fields into a named DTO. Application code does not manually call
`JSON.stringify` or decode the native result envelope.

For a Rust-owned capability, derive `specta::Type` on its bridge DTOs and define
one typed method constant. The same constant is consumed by binding generation
and runtime registration. `JsonMethod::new` requires the request to implement
`Deserialize` and the response to implement `Serialize`, so a schema that
cannot cross the actual wire fails where the contract is declared:

```toml
[dependencies]
wabou = { version = "...", features = ["bindings"] }
```

Applications that do not generate Rust-owned TypeScript capabilities leave the
feature disabled and do not compile Specta or the TypeScript exporter.

```rust
use wabou::{Capability, JsonCapabilityContract, JsonMethod, Type, specta};

#[derive(serde::Deserialize, Type)]
struct RenameRequest {
    name: String,
}

const RENAME: JsonMethod<RenameRequest, bool> = JsonMethod::new("rename");
const STATUS: JsonMethod<(), WorkspaceStatus> = JsonMethod::no_request("status");
const WORKSPACE: JsonCapabilityContract = JsonCapabilityContract::new("workspace", 1);

let capability = Capability::new(WORKSPACE)
    .method(RENAME)
    .method(STATUS);
```

The `specta` import is intentionally re-exported by the `bindings` feature so
its derive expansion resolves without a second version declaration in the
application's `Cargo.toml`. Serde remains a normal direct dependency because
its derive macros require the downstream crate to link `serde` by name.

This generates DTO declarations, a module-local raw host contract, injectable
`createXClient(host)` factories, and `useXClient()` context consumers from the
shared contracts. It does not expose JSON wire methods as public typed host
methods. Only explicit bridge DTOs should derive `Type`; persistence entities,
upstream SDK models and renderer internals are not automatically public
JavaScript contracts.

Register structured async implementations through the matching runtime
adapter. It owns JSON decoding, native Promise creation and the standard result
envelope, so application code does not repeat the wire protocol:

```rust
HostBuilder::new().json_capability(WORKSPACE, |capability| {
    capability.method(RENAME, rename)
});
```

Application capabilities always use this structured adapter. Framework-owned
synchronous QuickJS functions are declared explicitly in the native host API;
applications do not receive a raw `rquickjs::Object` escape hatch.
