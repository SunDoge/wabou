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
  --tag v0.1.0-alpha.3 --locked wabou-cli
wabou new hello-wabou
cd hello-wabou
bun install
bun run dev
```

Until Wabou publishes crates.io and npm packages, `new` records the selected
Wabou revision as `vendor/wabou`, a Git submodule shared by Cargo and Bun. This
avoids accidentally combining Rust and JavaScript packages from different
commits. The default revision is `v0.1.0-alpha.3`; maintainers can exercise a
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
bun run wabou dev apps/gallery --rust-hot-reload
bun run wabou clean apps/gallery
```

`doctor` checks the required Rust and Bun tools, platform build dependencies,
the selected application, and generated workspace package artifacts. It exits
with a nonzero status when a required check fails, so it is also suitable for
setup scripts and CI diagnostics.

`wabou clean [APP]` removes the selected application's debug/release frontend
bundles, its Vite dependency cache, and Wabou's frontend lock/cache directory.
It does not touch Cargo's `target/` or installed dependencies. In the Wabou
source workspace, pass `--packages` to additionally remove every local
`packages/*/dist` directory; rebuild those package artifacts with
`bun run packages:build` before the next application build.

Development starts Vite and compiles the Rust host with the `wabou/vite` and
`wabou/devtools` facade features. QuickJS imports the live Vite graph and
receives HMR updates, while `wabou inspect` can discover the local DevTools
socket. `--devtools` additionally opens the visual inspector; it is not needed
for the socket. No JavaScript production build is required before Rust
compilation. Ctrl-C terminates Vite, the host, and the optional inspector.

Rust capability hot reload is an experimental, explicit opt-in:

```bash
wabou dev apps/gallery --rust-hot-reload
```

The CLI launches the stable host through PATH `cargo` and sets
`WABOU_RUST_HOT_RELOAD=1`. An opted-in application registers a
`CapabilityDevelopment` service and mounts a `CapabilityModule` through
`module_method` or `json_module_method`. The service builds/watches an independent
application cdylib; QuickJS keeps the same host-owned entry across replacements.
Dioxus CLI and Subsecond are no longer required.

Gallery demonstrates this split: `gallery-api` owns the shared DTO contract,
`gallery-functions` owns palette behavior, and Gallery's host chooses the module
service when the environment flag is present. Ordinary development and packaged
runs use the same application implementation linked normally into the executable.
Other applications must register their own module service; this flag does not
make arbitrary linked Rust functions replaceable.

To use a local precompiled runtime instead of compiling the linked application
host, supply both a runtime library and its launch configuration:

```bash
wabou launch app.json --runtime /path/to/libwabou_runtime_library.so
wabou dev apps/gallery --runtime-library /path/to/libwabou_runtime_library.so --runtime-config app.json --rust-hot-reload
```

A CLI built with `cargo build -p wabou-cli --no-default-features` can use this
path without linking QuickJS or the native renderer. The frontend can be a
`package.json` + `ui/index.tsx` directory without a Cargo host manifest. The
ordinary CLI build keeps embedded render/layout and experimental native UI
commands enabled.

The configuration declares windows, capabilities and library/project sources.
`dev` replaces its UI source with the application's Vite entry and supervises
Vite plus the runtime process. Use Project implementations for Rust hot reload;
module Cargo features are configured there. Paths in the JSON are relative to
that file. Runtime downloads and shared-runtime packaging are not integrated.
See [runtime distribution](runtime-distribution.md) for the configuration format
and the current limits on application-specific native widgets.

Pending calls retain their original library until completion or cancellation.
Host state/resources remain resident. A failed subsequent compilation leaves the
working implementation installed. Initial module compilation failure prevents
startup. Changing runtime, windows or widget implementations requires restarting;
a DTO contract version mismatch rejects replacement. See
[capability modules](capability-modules.md) for ABI, watching and async constraints.

The CLI invokes the project-local Vite executable directly. An application
does not need specially named `build` or `dev` package scripts; those scripts
are optional shortcuts such as `"build": "wabou build"`. Build customization
belongs in `vite.config.ts`, where it applies consistently to development,
tests, rendering, and packaging.

When running directly from the Wabou source workspace, `@wabou/vite`
automatically enables the packages' `wabou-source` export condition. HMR,
captures, and application builds therefore see package source edits without a
manual package rebuild. Turbo still builds publishable package artifacts
according to workspace dependencies, caches them independently, and promotes
them into each `dist/` directory atomically. Run `bun run packages:build` when
validating package consumers or preparing a release. Source-to-artifact drift
is checked deterministically by `bun run gen:check` in CI; standalone
applications and published packages continue to resolve compiled `dist`
artifacts.

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

Render any named component layout fixture without adding an application-level
screenshot scenario:

```bash
bun run wabou render apps/pi-agent \
  --fixture conversation/complete-turn \
  --out target/complete-turn.png \
  --snapshot target/complete-turn.json
```

The command selects the application's `layout-test` bundle and reuses the
fixture's declared viewport, device scale, and settling time. The PNG and
DevTools snapshot therefore describe the same frame used by layout contracts.
Use this for focused visual review after component behavior and geometry tests
pass; it is not a replacement for those deterministic assertions. Pixel
capture uses the offscreen Vello Hybrid renderer. Use `wabou layout` when only
structured geometry is required.

## Run and package

```bash
bun run wabou run apps/gallery
bun run wabou build apps/gallery --release
bun run wabou build apps/gallery --release --source-map
bun run wabou package apps/gallery
bun run wabou devtools
```

Commands that compile the application host accept Cargo-style feature
forwarding. Values may be comma-separated or supplied more than once:

```bash
bun run wabou run apps/gallery --features diagnostics
bun run wabou test apps/gallery --features diagnostics
```

`--features` is forwarded to the application package for `check`, `dev`,
`build`, `package`, `run`, `test`, `bindings`, and host-backed `render`.
Bundle-only `render` rejects it because that mode does not compile an
application host.

To diagnose a release build without adding instrumentation to normal releases,
enable the compile-time profiler for that run only:

```bash
bun run wabou run apps/stress --release \
  --profile-trace /tmp/wabou-trace.json
```

See the [performance profiling guide](performance.md) for trace contents and
privacy guarantees.

For immediate native frame feedback, show the native performance HUD without
putting a reactive FPS component in the application tree:

```bash
bun run wabou dev apps/gallery --hud
bun run wabou run apps/gallery --hud
```

The HUD reports FPS, JavaScript time, projection time, frame time, projected
node count, viewport size, and projection revision. `WABOU_PERFORMANCE_HUD=1`
remains available for direct host launches.

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

To lower the glibc baseline of a Linux package, install Cargo Zigbuild and pass
its versioned GNU target directly to `package`:

```bash
cargo install cargo-zigbuild --locked
bun run wabou package apps/example \
  --target x86_64-unknown-linux-gnu.2.28 \
  --format appimage
```

When `--target` is present, Wabou uses `cargo zigbuild` for the Rust host and
stages the executable from Cargo Zigbuild's normalized target directory (for
example, `.2.28` builds live under `target/x86_64-unknown-linux-gnu/release/`).
The frontend bundle and native package are still produced by Wabou. A missing
Cargo Zigbuild command is reported with the installation command above.

Dependencies containing prebuilt native libraries must themselves match the
requested glibc and C++ ABI. Zigbuild cannot lower the baseline of an already
compiled `.so` or static archive; supply a compatible build/sysroot or package
that application on its oldest supported distribution.

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
use wabou::{Capability, CapabilityContract, JsonMethod, Type, specta};

#[derive(serde::Deserialize, Type)]
struct RenameRequest {
    name: String,
}

const RENAME: JsonMethod<RenameRequest, bool> = JsonMethod::new("rename");
const STATUS: JsonMethod<(), WorkspaceStatus> = JsonMethod::no_request("status");
const WORKSPACE: CapabilityContract = CapabilityContract::new("workspace", 1);

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
HostBuilder::new().capability(WORKSPACE, |capability| {
    capability.json_method(RENAME, rename)
});
```

Application capabilities always use this structured adapter. Framework-owned
synchronous QuickJS functions are declared explicitly in the native host API;
applications do not receive a raw `rquickjs::Object` escape hatch.

### Project development defaults

Store local prebuilt-runtime selection in the application's `wabou.toml`:

```toml
[dev]
runtime-library = "../../target/debug/libwabou_runtime_library.so"
runtime-config = "app.json"
rust-hot-reload = true
```

Then use `wabou dev APP` (or `wabou dev` inside the frontend). Paths in `[dev]`
are relative to that application's directory; command-line paths remain relative
to the invocation directory and override the corresponding configuration value.
The library and JSON configuration must both resolve. Missing/incompatible
artifacts produce an error rather than falling back to a runtime build. Without
prebuilt configuration, a linked Cargo application keeps its existing development
path. A dev-only file needs no `[package]` section; packaging still requires one.
`rust-hot-reload` requires Project implementations in the launch JSON. This does
not download runtime artifacts or make arbitrary Rust host code reloadable.

Rust capability development publishes building/ready/error status, attempt count,
combined compile-and-replace time and compiler diagnostics through
`wabou:reload-status`. The runtime-library host and Gallery module host register
this producer. Other linked hosts can register `development.diagnostics()` with
`host_message_producer`. ComponentsProvider's development overlay shows JS and
Rust errors; a successful JS update does not dismiss an unrelated Rust error.
A matching Rust project's successful replacement clears its diagnostic. Completed
status is replayed when a window attaches after initial compilation.

Replacement import/evaluation failures now retain the JS hot boundary and return
an error with guest message/stack, instead of requesting a full reload. Missing,
declined or invalidated hot boundaries still request full reload. This is not a
transactional rollback of arbitrary top-level module side effects or accept
callbacks that have already run.

### Embedded runtime development and distribution

The default CLI includes QuickJS, windowing, layout and rendering. A frontend-only
project can select its launch contract without specifying a runtime library:

```toml
[dev]
runtime-config = "app.json"
```

Run `wabou dev path/to/app`. The CLI starts Vite and a supervised copy of itself
as the GUI host. Capability `project` implementations are built and watched inside
that resident host. Only application capability code is rebuilt. Native code is
trusted; use the capability SDK's host-owned resources and task lifetime contract.
`--runtime-library` remains available for an explicitly selected external runtime.
A CLI installed with `--no-default-features` requires that external runtime.

Portable distribution using the installed CLI and separately loaded capabilities:

```sh
wabou bundle path/to/app --config path/to/app/app.json --out dist/my-app
./dist/my-app/wabou launch ./dist/my-app/app.json
```

The output contains the runtime executable, production UI/resources, relative
capability library paths and launch configuration. Project capabilities build in
release mode. The output directory must not already exist. Build failures leave
no partial distribution. This bundles the CLI you are running: use a release CLI
for production. Bundles target the current operating system and architecture;
third-party native dependencies are not collected automatically.

Link the runtime and capability Rust libraries into a generated application:

```sh
wabou bundle path/to/app --config path/to/app/app.json --out dist/my-app --static
./dist/my-app/wabou
```

Static mode requires `project` capabilities whose crates provide an `rlib` and
use the SDK's `export_module!` or `export_handlers!`. The generated host enables
SDK `static-link` to avoid duplicate exported plugin symbols. The same handlers
and host-owned services run through the existing function table; this first
version does not eliminate its encoding/polling overhead. No capability dynamic
libraries are required at runtime. UI/resources remain beside the executable.
System libraries may still be dynamically linked. Static mode currently retains
the runtime's development support code; it does not yet produce a minimal runtime.
Compilation uses the PATH Cargo and reuses the workspace target directory.
Existing Rust-host applications can continue using `wabou build --release` and
`wabou package` for application-owned registration and native installers.
