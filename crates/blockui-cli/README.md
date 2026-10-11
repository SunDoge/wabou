# blockui

One Rust CLI for standalone and workspace applications. It requires no Python or
Node for native development, Wasm builds or the loopback browser server.

```sh
cargo install --path crates/blockui-cli --locked
blockui --help
```

Inside this repository `cargo blockui` is equivalent. `cargo xtask` is retained as
a compatibility entry point and uses the same parser and implementation.

| Command | Behavior |
| --- | --- |
| `blockui new PATH` | Create an independent Counter app with native and browser entries; refuse existing paths. |
| `blockui dev [APP]` | Build the stable host, build the UI plugin, open the native window and watch UI dependencies. |
| `blockui run [APP]` | Build and open the native app without a watcher; supports `--release`. |
| `blockui build [APP]` | Build host and plugin without opening a window; supports `--release`. |
| `blockui dev [APP] --web` | Watch, rebuild and hot-reload Wasm on loopback, preserving actor state. |
| `blockui build [APP] --web` | Generate static browser release assets atomically in `web/pkg`. |
| `blockui test [APP]` | Run app library and integration tests; no host, Wasm, examples or doctests by default. |
| `blockui check [APP] [--web]` | Check the application library for native or Wasm; `--tests` or `--test NAME` checks native tests without linking or executing them. |
| `blockui doctor [APP] [--web]` | Check Cargo/rustfmt, project resolution and optional Wasm target/binding-generator version. |

New apps use `#[app]` / `#[app_impl]` and `#[ui]` blocks. The first
expression in a marked block is its parent; marked expressions below it are
children. Widget constructors and style builders are ordinary Rust.
`live!` marks the counter text subscription, so changing count updates that
text without rerunning the root view. The generated headless test checks this
and retained state across compiled-in implementation replacement.
Use `cargo fmt` for formatting.

`blockui doctor APP --static` checks Cargo/rustc/rustfmt and linked binary
selection, including a binary-only Cargo project. It does not require a cdylib,
build the application, initialize graphics or verify native system libraries.
Use a real native build/capture for those later stages.

Use `blockui check APP --static` to type-check the same linked binary selected by
`build/run --static`, enabling its declared required features without linking or
opening a window. Cargo JSON/short diagnostic formats and `--timings` remain
available. Static checking conflicts with browser and test-target selection;
use normal `check --tests` for component tests. Binary selection requires project
metadata, so this path does not promise the direct-package library shortcut.

Use `blockui new PATH --wabou-native` to generate a prototype facade project
with `wabou_native::*` imports and `wabou_native::testing` headless tests. It
requires an SDK checkout containing `crates/wabou-native`; `--sdk-path` selects
that checkout. The project still uses this CLI and current host runtime, not
Wabou's CLI. Omitting the flag retains the BlockUI SDK template.

`blockui new` rejects application names that collide with SDK dependencies,
including underscore spellings such as `blockui_abi`, before creating any files.
Use a distinct application name so Cargo can resolve the generated SDK graph.

`APP` accepts a Cargo package name, application directory or manifest. A package
can also be supplied with `-p/--package`. `--manifest-path` resolves relative to
the invoking directory. With `--manifest-path`, a bare `APP` selects a package
even if a directory has the same name; explicit application paths cannot be
combined with `--manifest-path`. `--target-dir` overrides Cargo's metadata target directory;
otherwise Cargo configuration and `CARGO_TARGET_DIR` apply. Paths are passed as
process arguments, including paths with spaces or non-ASCII characters.

Native `blockui check PACKAGE` can delegate directly to Cargo without a metadata
query when PACKAGE is an unambiguous bare package selector. Cargo validates the
package and selected test targets. A misspelled `--test NAME` preserves Cargo’s
available-target suggestions and failing exit status for both `check` and `test`.
`blockui test PACKAGE --lib` and
`blockui test PACKAGE --test NAME` use the same direct path, including combined
library/integration selections. Default test selection still uses metadata to
exclude host unit tests and respect enabled integration targets. Explicit manifests, application paths and
implicit/default application selection retain project discovery. Browser checks
keep their existing discovery path.

Use `blockui check PACKAGE --message-format json` for Cargo JSON diagnostics
and build events, or `--message-format short` for compact terminal diagnostics.
The format is passed to Cargo unchanged, including its supported comma-separated
options, and does not add discovery to the explicit-package fast path. Compiler
errors keep their authored file/line spans and produce a failing exit status.

Use `blockui check PACKAGE --timings` to write Cargo's HTML build timing report
under `<target-dir>/cargo-timings/`; Cargo prints its location. It also combines
with `--tests`, `--test NAME`, `--web` and `--message-format json`. The report
measures this check invocation, including any fresh dependencies; it does not
measure native plugin linking, reload or rendering. Compare equivalent edits
and build settings when diagnosing compilation cost.

Use `blockui test PACKAGE --timings` to inspect test compilation, or combine
`--timings --no-run` to compile without executing tests. It works with default
component selection, `--lib`, `--test NAME`, filters and harness arguments. Cargo
writes its HTML report under `<target-dir>/cargo-timings/` and prints the path.
The report covers compilation; it does not time test execution. For comparable
samples, use the same target selection, edit and build settings.

Set `BLOCKUI_CARGO` to an executable path or name to select Cargo consistently
for project discovery, checking/testing, host/plugin builds and native reloads.
For example, `BLOCKUI_CARGO="$HOME/.cargo/bin/cargo" blockui dev gallery` compares
direct Cargo with a wrapper on PATH without changing global shell configuration.
Unset or empty uses `cargo` from PATH. The value is a single executable, including
spaces/non-ASCII paths; shell commands and extra arguments are not interpreted.
This is opt-in and does not change compiler wrappers or promise a speedup.
Relative executable paths such as `./tools/cargo` are resolved from the CLI
invocation directory and preserved when the native host changes directories.
Bare executable names use PATH lookup.

Native development options include `--host-package`, `--features`, `--capture`,
`--configuration-only` (print the watch plan without building), and `--prepare-only`
(build only the plugin and print its plan). The 7GUIs host accepts `--task NAME`.
Native dev uses an optimized release host and debug UI plugins. The stable renderer stays fast while UI edits retain debug incremental builds and dylib replacement. Use `run --release` or `build --release` to optimize both host and plugin. Without `--static`, these native release commands still use a separate dylib and dynamic loading. For a generated `--wabou-native` app, `build/run --static --release` selects its linked binary and enables its required native feature. Static mode needs no external UI dylib or generation directory at runtime; loader feature stripping remains open. Static mode conflicts with browser/host selection and does not support dev watching.
Every native save builds only the selected application library (`cargo build
--lib`), matching the initial plugin build. Auxiliary binaries in the same package
do not block reload when their own code fails to compile. Library dependencies and
build scripts still participate normally. Build failures preserve the running
actor; long diagnostics retain their beginning and final error within an 8 KB
display budget, with an explicit truncation marker. Run `blockui check` for full
application type-check diagnostics.
`run` never enables the native source watcher.
Successful native watcher updates report compilation/copy time (`Built in ... ms`)
and replacement time (`replaced in ... ms`) separately. Replacement measures the
synchronous load/migration/projection path; root replacement also includes its
native reconciliation. Child/group reconciliation can continue afterward. These
numbers exclude source detection, queue waiting and GPU presentation, so neither
is a save-to-visible measurement. Rejected replacements keep their error message
and preserve the live app.


Default `blockui test` explicitly selects library unit tests and enabled integration
test targets from Cargo metadata. Same-package binary unit tests are excluded,
which keeps a custom host's unit tests out of the UI feedback loop. Cargo may
still build ordinary binaries required by integration tests. Use `--lib` to
restrict the run to library unit tests, or `--test NAME` to select an integration
target; both can be combined. `--filter` filters test names after compilation.
Default selection also respects `required-features`: gated integration targets
run only when Cargo reports their features enabled. Both local feature names and
dependency features such as `blockui/macros` are supported; renamed dependencies
are matched by their Cargo alias. Projects with gated targets
resolve the Cargo dependency graph for this selection; ordinary projects keep
the lightweight metadata path. Explicit `--test NAME` leaves feature validation
to Cargo, which reports missing required features.



Browser development uses release builds, a checked wasm-bindgen version and a
loopback HTTP server. `--port` defaults to 8123; `--port 0` chooses a free port.
The server serves only files inside the configured asset directory, including
proper Wasm MIME headers. Failed compilation/binding generation leaves the last
generated `pkg` intact; successful builds replace it and retire the previous copy.
`dev --web` watches source files and sends updates over SSE. A new Wasm instance
restores a data-only checkpoint through the schema migration path before replacing
the current application. Compile, migration and projection failures preserve the
running app. Keyed DOM elements retain focus and text selection. `run --web`
serves a release without watching or injecting development code. There is no deployment.

The first prototype waits for active tasks, timers, resources and queued messages
to finish; active intervals must be canceled before an update can apply. It keeps
two immutable asset generations in `.runtime/web-generations`, with an exclusive
project lock and stale generations cleaned on the next start. After 20 imported Wasm generations, or
when the browser renderer changes, the status panel offers a restart preserving
actor state in session storage. Browser module caches cannot be explicitly unloaded.
Only SDK actor state is transferred; arbitrary JavaScript state and external side
effects are outside the checkpoint. Development still compiles release Wasm.

Project configuration uses ordinary Cargo metadata:

```toml
[package.metadata.blockui]
host-package = "my-stable-host" # optional separate executable in the Cargo graph
# host-example = "seven_guis"  # optional framework host example, instead of host-package
web-assets = "web"             # relative to this package, defaults to web
web-shared-assets = true        # optional: package SDK renderer.js and base.css inside pkg
```

A custom host receives `LIBRARY [--watch CONFIG.json] [--capture]`. Host binaries
are discovered from Cargo targets. Without a custom host, the CLI builds the
SDK's generic `blockui-host`. Only the stable executable owns native callbacks;
plugin snapshots, turn boundaries and migration rollback use the existing runtime.

Virtual workspaces can select defaults:

```toml
[workspace.metadata.blockui]
default-app = "my-ui"
default-web-app = "my-browser-app"
```

`new` uses local SDK path dependencies. The SDK checkout must remain present;
`--sdk-path` selects it explicitly. Both generated entries share the same Actor
and UI macros. `blockui_web::export_app!(ReleaseApp, Counter)` generates the browser wrapper
with snapshot/event/poll methods, and the browser renderer is copied from the SDK's current backend.

Verification: `python3 scripts/verify_cli.py` exercises a generated app outside
the SDK workspace, native capture in an isolated display, static Wasm output,
failed browser-build preservation and the CLI's HTTP server. With Playwright
installed it also verifies the generated app's browser state transitions.

## Fast component iteration

```sh
blockui test gallery
blockui test gallery --test component --filter toast
blockui test gallery --lib --no-run
blockui test gallery --test component -- --nocapture
```

`--test` can repeat; combine it with `--lib` to select both. Explicit `--features`
are forwarded to Cargo without automatically enabling native rendering. The
command honors the same package/path/manifest and target directory resolution as
other commands. Test failures propagate a failing exit status. Full doc/example,
GPU, browser and reload proofs remain explicit validation layers.

New projects use a fine-grained `Text::reactive` counter and include a `blockui-test`
scenario with semantic/text queries. The harness supports subtree queries and
static implementation replacement without renderer builds. Projects use SDK-generated browser
assets instead of copied renderer/style files. See [iteration guide](../../docs/iteration.md).

Use `blockui check --tests` for library and integration test type errors before
running scenarios. `blockui check --test component` checks the library plus the
selected integration target; repeat `--test` to select several. Test checking is
native-only and cannot combine with `--web`. `--tests` follows Cargo and also
checks binary unit-test targets; use `--test NAME` for a specific integration
target. This differs from the default `blockui test` selection, which excludes
binary unit tests. It uses `cargo check`, so it neither
executes tests nor builds a native host. Use `blockui test` for behavior and
linker validation; `check` alone cannot prove that tests pass.

Use an explicit package name, such as `blockui check my-app` or
`blockui check my-app --tests`, to skip metadata discovery entirely. Path-based
or inferred native checks discover the application using `cargo metadata --no-deps`.
Both paths skip the launch graph and host-target query; Cargo still checks the
application's actual dependencies and selected features.
Native launch/build and web checking retain their existing discovery paths.

Native platform detection and `blockui doctor` honor nonempty `RUSTC`, then
`CARGO_BUILD_RUSTC`, before falling back to PATH `rustc`. This selection covers
explicit environment overrides; compiler paths configured only in Cargo config
files are not resolved by these probes.

Explicit compiler paths containing directory components are resolved relative to
the CLI invocation directory before Cargo or the native host changes directory.
Bare executable names keep PATH lookup, and symlinks are preserved for rustup
proxy dispatch. The same normalized selection reaches the native watcher.

Framework CLIs may reuse `blockui_cli::main_from_named("wabou native", arguments)`
to preserve their command name in help and argument errors. Supply the executable
name as the first argument, followed by forwarded Rust command arguments.
