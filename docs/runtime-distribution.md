The default `wabou` CLI now embeds the QuickJS GUI runtime. For normal development,
use `[dev] runtime-config = "app.json"` and `wabou dev`; no separate runtime dylib
is needed. `wabou bundle` and `wabou bundle --static` provide distribution paths.
See [CLI documentation](cli.md#embedded-runtime-development-and-distribution).
The external-runtime launcher below remains an optional deployment mode.

# Precompiled runtime distribution (experimental)

Applications can now launch a resident runtime shared library through the lightweight
`wabou-launch` SDK/command. Its normal dependency tree contains no QuickJS, winit,
Vello, Taffy or Tokio. `wabou-runtime-library` is the separate cdylib that contains
those implementations. A bundled UI or Vite entry and application capability libraries
are supplied as launch configuration rather than linked application host code.

The launch symbol is exported with stabby 72.1.16 and loaded through its checked
signature API. Configuration and errors use ABI-stable strings/results; ordinary
Rust String, trait objects and futures do not cross this launch boundary. The startup
payload is versioned JSON, parsed once inside the runtime. Capability invocations
still use the existing versioned C ABI behind the capability SDK. Adopting stabby
for that second boundary is separate work; this prototype does not claim it is done.

## Build and launch

```sh
cargo build -p wabou-runtime-library -p wabou-launch -p gallery-functions
cargo run -p wabou-launch -- target/debug/libwabou_runtime_library.so app.json
```

The runtime and capability filenames use `.dylib` on macOS and `.dll` on Windows.
The runtime is trusted native code and must be loaded/run on the main thread.
A RuntimeLibrary cannot move between threads and is consumed by run. It remains
loaded until process exit, even when the event loop returns, because platform/GPU
or TLS callbacks may retain code references. This is one resident runtime per
launcher process; capability libraries still retire by invocation generation.

Example app.json (paths are relative to this file):

```json
{
  "version": 1,
  "title": "Wabou Components",
  "width": 1280,
  "height": 840,
  "source": { "kind": "bundle", "path": "bundle.js" },
  "capabilities": [{
    "name": "bindingsDemo",
    "version": 1,
    "methods": [{ "name": "describePalette", "hasRequest": true, "json": true }],
    "implementation": { "kind": "library", "path": "libgallery_functions.so" }
  }]
}
```

For UI HMR, source can be `{ "kind": "vite", "url": "http://127.0.0.1:5173", "entry": "ui/index.tsx" }`.
For Rust replacement, implementation can be a Project:

```json
{
  "kind": "project",
  "manifest": "app-functions/Cargo.toml",
  "watch": ["app-api/src"],
  "features": []
}
```

The project service invokes PATH Cargo and swaps only the capability implementation.
An optional Project `targetDirectory` isolates Cargo outputs for independent
builders; relative values resolve from the JSON configuration directory. Omit it
to retain the normal Cargo/mbx cache configuration.
The runtime/launcher and JavaScript function entries remain alive. The runtime gives
each configured capability a resident in-memory JSON state namespace through the
`state.read` and `state.write` host operations. The first accepts a key string; the
second accepts `{ "key": "...", "value": ... }`. This is volatile host state, not
on-disk persistence. The SDK exposes `host.state::<AppState>().get()` and
`.set(&value)` using serde types. `state.snapshot` distinguishes an absent slot from
a stored JSON null. Decode failures return errors without resetting stored state.
Slots use the fully qualified Rust type name within the capability namespace:
renaming a type/crate changes identity, and schema changes require compatible
deserialization or migration. Separate reads and writes are not atomic transactions.
Linked hosts can use the same `wabou_runtime::capability_state::CapabilityState`:
construct one store per namespace and pass `store.operations()` to
`CapabilityModule::new`. The store never retains a module's Rust allocation.
The shared runtime also supplies resident file, TCP connection and timer/task providers. Optional
`resourceDirectory` resolves relative file paths and defaults to the JSON file's
directory. Files use typed handles and async chunk reads; see
[capability modules](capability-modules.md#resident-files) for lifecycle details.
More native resource providers remain to be integrated; this is not an arbitrary
Rust-object state transport.

## Evidence and limits

```sh
WABOU_PREBUILT_RUNTIME="$PWD/target/debug/libwabou_runtime_library.so" cargo test -p wabou-launch --test runtime_library -- --ignored
cargo tree -p wabou-launch --edges normal
cargo test -p wabou-runtime --features capability-modules --lib typed_sdk_state_survives_actual_library_replacement
```

The first test loads the actual library, rejects an incompatible function signature
before invoking it and rejects an incompatible launch contract. The dependency tree
establishes that the launcher is independent of the renderer implementation.
The typed-state test compiles two real SDK libraries, changes an increment from
1 to 10, confirms the result is 11 with the original text intact, and confirms
the old generation has been released. It additionally preserves a real file
handle/cursor across replacement. It exercises the same resident store used
by the shared runtime.

A Linux x86_64 deterministic native-host run of Gallery's generated-client palette
scenario passed through the launcher and shared runtime, including stripped copies
of both libraries. A local sample in `/tmp/wabou-runtime-dist` contains the launcher,
runtime, palette library, bundle and relative-path configuration. Stripped development
artifacts measured about 580 KiB for the launcher, 45 MiB for the runtime and
490 KiB for the palette library. These are local development samples, not release
package measurements or verified macOS/Windows results. They are not published.

The CLI can select a local precompiled runtime explicitly:

```sh
wabou launch app.json --runtime /path/to/libwabou_runtime_library.so
wabou dev apps/gallery --runtime-library /path/to/libwabou_runtime_library.so --runtime-config app.json --rust-hot-reload
```

`launch` resolves bundle/module paths relative to the configuration file.
`dev` starts/supervises Vite and the runtime process, replaces the configured UI
source with the application's Vite entry, and skips compiling the linked app host.
Use Project implementations for module hot replacement; the flag rejects a
configuration without any projects. Module features belong in their Project
configuration, not the linked-host `--features` option. The runtime library is
trusted native code and its path is explicitly selected by the developer.

This path is experimental. Runtime downloads and packaging of shared-runtime
applications are not integrated. Application-specific widgets such as Gallery's fractal still
belong to its linked host and have no portable extension ABI in this runtime. Asset
root/app-directory configuration, more platform options, typed configuration
registration and release distribution matrices remain to be added. A passing
capability scenario does not establish visual/platform correctness of every widget.

For the full QuickJS → SDK library → resident services path, build the services
probe and run the explicit native integration test (Linux artifact names shown):

```sh
cargo build -p wabou-launch -p wabou-runtime-library
cargo build -p wabou-capability-sdk --example services
WABOU_PREBUILT_RUNTIME="$PWD/target/debug/libwabou_runtime_library.so" \
WABOU_PREBUILT_LAUNCHER="$PWD/target/debug/wabou-launch" \
WABOU_PREBUILT_PROBE="$PWD/target/debug/examples/libservices.so" \
cargo test -p wabou-launch --test runtime_services -- --ignored
```

This test builds a minimal behavior script with the ordinary application bootstrap,
starts a loopback echo fixture, launches the real shared runtime in deterministic
headless mode, and inspects its named report. It verifies typed state accumulation,
file cursor preservation, timers, duplex TCP I/O, pending-read cancellation and
connection close. It uses Bun and prebuilt artifacts, so it is explicitly ignored
in the default library-only test run. Implementation replacement and UI HMR are
covered separately by the runtime tests, not claimed by this launch scenario.

Both CLI paths were exercised with a fresh local runtime on Linux x86_64:
`launch` with a bundled UI/fixed library, and `dev` with Vite plus a Project
implementation. Each deterministic native report contains a passing
`palette capability uses the generated client in the real host` scenario.
The dev process exited normally and its supervised Vite process stopped.
These checks establish startup/registration, not replacement during that same
process or visual correctness; replacement is covered by the runtime library tests.

## Development dependency boundary

Build a CLI without the embedded renderer or experimental Rust UI adapter:

```sh
cargo build -p wabou-cli --no-default-features
bun run scripts/check-runtime-boundaries.ts
```

This CLI supports launching a local prebuilt runtime and supervising Vite/Rust
capability development. Its normal dependency graph contains no QuickJS, Vello,
wgpu, winit or runtime implementation. `embedded-renderer` enables the CLI's
internal render/layout backend; `native-ui` enables the experimental Rust UI
commands. The default CLI enables `embedded-renderer`; `native-ui` is opt-in.
Disabled commands return an explicit feature error. Packaging dependencies remain
in the lightweight CLI; it is not yet a minimal launch-only executable.

In `dev --runtime-library ... --runtime-config ...` mode, the frontend directory
needs only `package.json` and `ui/index.tsx`; it does not need a host Cargo package.
Project capability manifests can live in separate directories. Their Cargo/mbx
cache is respected and only those projects are built by the runtime watcher.
The runtime dylib stays loaded, and is not recompiled or replaced in this path.
Changing engine/native widget code still requires rebuilding/restarting the
runtime. Automatic artifact acquisition is deferred; a future downloader can
supply the same local-library boundary without changing application capabilities.

The dependency check runs separately for the launcher, capability SDK, Gallery
functions and lightweight CLI, so workspace feature unification cannot disguise
a new implementation dependency. This establishes a build boundary, not complete
ABI compatibility across framework releases; the launch signature and contract
checks still apply.

A follow-up Linux x86_64 check used the lightweight CLI with a frontend-only
Vite app, an existing matching runtime dylib and a fixed Gallery capability.
A PATH Cargo interceptor would record and reject any invocation: none occurred.
The named generated-client palette scenario passed in its deterministic native
report, and the supervised Vite process stopped with the host. This proves the
no-build launch path; Rust watcher replacement is verified separately. The
standalone bundled scenario emitted a multiple-Solid-instance warning and this
check does not establish component refresh or visual correctness. An older local
runtime sample rejected the newer configuration's `resourceDirectory` field;
prebuilt runtime and launcher/config versions must be matched even when their
native entry signature is unchanged.
