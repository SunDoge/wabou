# Wabou CLI

Wabou applications use one typed configuration file:

```text
my-app/
├── Cargo.toml
├── package.json
├── wabou.config.ts
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
  --tag v0.1.0-alpha.4 --locked wabou-cli
wabou new hello-wabou
cd hello-wabou
bun install
bun run dev
```

Until Wabou publishes crates.io and npm packages, `new` records the selected
Wabou revision as `vendor/wabou`, a Git submodule shared by Cargo and Bun. This
avoids accidentally combining Rust and JavaScript packages from different
commits. The default revision is `v0.1.0-alpha.4`; maintainers can exercise a
candidate commit with `--wabou-repository` and `--wabou-ref`.

The app's Wabou config contains native application metadata and delegates the
frontend compiler, Solid universal renderer, aliases, optimizer policy, and
bundle defaults to `@wabou/vite`:

```ts
import { defineConfig } from "@wabou/vite";

export default defineConfig({
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

Rust host modules and widgets are linked into the application executable. A
Rust edit uses Cargo incremental compilation and restarts the host; a TSX edit
stays on the Vite/Solid HMR path and does not rebuild Rust. Generate the
Rust-owned client declarations with `wabou bindings write` and verify them with
`wabou bindings check`.

The installed `wabou` executable is an orchestration CLI and does not link the
renderer, QuickJS, Winit, or wgpu. Headless `render` and `layout` commands are
forwarded to the separate `wabou-render` helper. In the Wabou source workspace
the CLI builds that helper on first use; binary distributions should place it
next to `wabou`.

The CLI invokes the project-local Vite executable directly. An application
does not need specially named `build` or `dev` package scripts; those scripts
are optional shortcuts such as `"build": "wabou build"`. Build customization
belongs in `wabou.config.ts`, where it applies consistently to development,
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
automatically. Frontend-only preview configurations have no application Rust
targets to check. `doctor` recognizes these projects too. For a pass without native behavior scenarios, use
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
capture uses the offscreen Vello GPU renderer. Use `wabou layout` when only
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
Set `sourceMap` in `wabou.config.ts` to override that default, or pass
`--source-map`/`--source-map=false` for one build:

```ts
export default defineConfig({ sourceMap: true });
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
separate packaging executable. Each application opts into native packaging in
`wabou.config.ts`:

```ts
export default defineConfig({
  package: {
    productName: "Example",
    identifier: "dev.example.desktop",
    description: "A native Wabou application.",
    authors: ["Example Team"],
    licenseFile: "../../LICENSE",
    icons: ["icons/*.png"],
    resources: ["assets"],
    formats: ["appimage", "deb"],
  },
});
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

`wabou bindings` scans application Rust sources for explicit JavaScript
contracts and writes ambient TypeScript declarations without compiling the
application:

```bash
bun run wabou bindings write apps/gallery
bun run wabou bindings check apps/gallery
```

`write` explicitly updates the committed declaration. `check` never rewrites
files and fails when the declaration has drifted; run it in CI. The default
source is `src/` and the default output is
`ui/generated/rust-bindings.d.ts`. Both can be changed in the shared project
configuration:

```ts
export default defineConfig({
  bindings: {
    sources: ["src", "../application-api/src"],
    output: "ui/generated/rust-bindings.d.ts",
  },
})
```

The marker macros do not change runtime behavior. `rquickjs` continues to own
registration while the Wabou attributes state the public JavaScript contract:

```rust
#[wabou::js_class(name = "Database")]
#[rquickjs::class]
pub struct Database {
    // Rust-only state is not part of the declaration.
}

#[rquickjs::methods]
impl Database {
    #[qjs(constructor)]
    #[wabou::js(args(path = string))]
    pub fn new(path: String) -> rquickjs::Result<Self> {
        // ...
    }

    #[qjs(get)]
    #[wabou::js(returns = string)]
    pub fn path(&self) -> String {
        // ...
    }

    #[wabou::js(
        args(key = handle(Resource), fallback = optional(string)),
        returns = promise(nullable(string))
    )]
    pub fn read(&self) {
        // ...
    }
}
```

DTO declarations use the same explicit type vocabulary. The Rust fields may
remain private and may use different implementation types; the marker records
the JavaScript wire contract rather than attempting to infer it from Rust:

```rust
#[wabou::js_type(fields(
    id = string,
    rows = array(named(Row)),
    next = optional(nullable(string))
))]
pub struct QueryResult {
    id: String,
    rows: Vec<Row>,
    next: Option<String>,
}

#[wabou::js_type(values(idle, loading, not_ready = "not-ready"))]
pub enum Status {
    Idle,
    Loading,
    NotReady,
}

#[wabou::js_type(name = "RecordId", alias = union(string, number))]
pub type Id = String;
```

These emit a TypeScript `interface`, string-literal union, and type alias,
respectively. `name = "..."` overrides the generated declaration name. Object
field documentation is copied from matching named Rust fields.

The generator understands `string`, `boolean`, `number`, `safe_integer`,
`bigint`, `unknown`, `void`, `object`, `uint8_array`, `array_buffer`, and
`undefined`. It also supports `array(T)`, `readonly_array(T)`, `promise(T)`,
`nullable(T)`, `optional(T)`, `record(T)`, `union(A, B)`, `literal(...)`,
`named(Type)`, `json(Type)`, and `handle(Name)`. Dynamic Rust values such as
`rquickjs::Value` must be assigned an explicit JavaScript type; the generator
does not silently emit `any`.

The source markers are the only binding manifest. `wabou bindings` no longer
builds a Cargo example or runs a Specta compatibility generator, so binding
checks remain fast and deterministic. Host functions and classes are authored
in Rust, registered with `rquickjs`, and described beside that implementation;
generated TypeScript contains declarations rather than a second runtime
client layer.

For a named object exposed through `useHost()`, add Wabou's declaration metadata
to an ordinary `rquickjs` native module. `rquickjs` generates the runtime
registration and conversions; Wabou only reads the inert attributes when it
generates declarations:

```rust
#[wabou::js_module(name = "database")]
#[rquickjs::module(rename_vars = "camelCase")]
pub mod database {
    #[rquickjs::function]
    #[qjs(rename = "runQuery")]
    #[wabou::js(
        args(query = string),
        returns = promise(array(named(QueryResult)))
    )]
    pub async fn run_query(query: String) -> Vec<QueryResult> {
        service().query(query).await
    }
}
```

This augments `HostCapabilities` in `@wabou/core`; it does not add a version
field or generate a JavaScript adapter. Mount the generated module definition
with `HostBuilder::host_module_def::<js_database>("database")`. JavaScript
names come from `rquickjs` (`#[qjs(rename = ...)]` above); Wabou's method
marker only describes argument and return types.

Stateful native services use the same module definition. Wrap the service in a
module-specific type deriving `JsLifetime`, pass it once with
`host_module_def_with_state`, and retrieve it from the `Ctx` argument inside an
exported function. This avoids closures retaining JavaScript handles and keeps
registration under `rquickjs`:

```rust
HostBuilder::new().host_module_def_with_state::<js_database, _>(
    "database",
    wabou::HostModuleState(database),
);
```

Each window runtime receives a clone. A runtime accepts one value for each
`HostModuleState<T>` promises that `T` is ordinary owned Rust state with no
QuickJS-borrowed handles. A runtime accepts one value for each `T`, so modules
using the same service type should introduce distinct newtype wrappers.

### Development boundary

Keep frequently edited presentation code in `ui/` and stable native services in
the Rust application. Host registration is ordinary Rust code in `src/main.rs`.
Long-lived native objects can live in
`ResidentResources`, whose generational handles remain valid across Solid HMR
and full UI remounts.

### Optional Intl data

The Vite configuration includes English and Chinese Intl data by default. Apps
that do not use Intl or widgets requiring it can omit that data:

```ts
export default defineConfig({ intl: false });
```

Apps needing only number formatting can avoid date and time-zone data:

```ts
export default defineConfig({
  intl: { locales: ["en"], apis: ["number"] },
});
```

`apis` accepts `number`, `date`, and `plural`; required FormatJS dependencies are
included automatically. This is configured in `wabou.config.ts`. Disabling Intl
is explicit: date/number widgets and libraries
using Intl still require the relevant APIs.
