# wabou-native (experimental)

A Rust UI frontend for Wabou with tag UI, dynamically tracked signals, live
properties, components and actor hot replacement. SDK, macros, runtime, test
harness and CLI implementation live in this checkout. The packages are local
experiments with `publish = false`.

Start with [tag UI](TAG_UI.md), [async results](ASYNC.md), or the runnable
[native counter](../../apps/native-counter/README.md). UI expressions are
ordinary Rust; source formatting uses rustfmt.

## Create and develop

From the Wabou checkout:

```sh
cargo build -p wabou-cli
./target/debug/wabou native new /tmp/my-native-app
./target/debug/wabou check /tmp/my-native-app --tests
./target/debug/wabou test /tmp/my-native-app
./target/debug/wabou dev /tmp/my-native-app
```

Generated apps reference this SDK by local paths. They include:

```toml
[package.metadata.wabou]
frontend = "rust"
```

This marker selects the Rust command parser. Put an explicit application path
immediately after the command. Inside an app or its subdirectories, omit the
path; discovery stops at the nearest Cargo manifest. Unmarked projects retain
the TSX route. `wabou native …` explicitly selects Rust commands.

For a known test, package selection avoids metadata discovery:

```sh
# From apps/native-counter:
wabou check native-counter --tests
wabou test native-counter --lib --filter increment
cargo fmt
```

Automatic package-name routing requires a match with the nearest Rust package.
Use `wabou native …` for other package selection. Rust frontend commands accept
the Rust CLI's options rather than TSX/Vite options.

Development replaces UI actor implementations between actor turns. Failed
builds retain the accepted implementation. Host/core changes require a restart.
The stable executable owns platform callbacks and background work; task results
are owned messages. Rust allocations, futures and plugin callbacks must not
escape across the dependency-free actor ABI.

## Dependencies and features

| Feature | Provides |
| --- | --- |
| `macros` (default) | Actor, component and tag UI procedural macros |
| `testing` | `wabou_native::testing` headless interaction and virtual-time harness |
| `native` | `wabou_native::native::{mount, run, run_capture}` and stable-host services |
| `lucide` | Selected embedded SVG constants under `wabou_native::lucide` |
| `rsx` | Optional RSX experiment; tag UI is the primary authoring path |

Enable `testing` in dev-dependencies. Default features do not pull in the native
runtime, test harness or Lucide asset crate.

With `lucide`, use
`Icon::new(wabou_native::lucide::SEARCH).size(16.0).label("Search")` or
`IconButton::new(wabou_native::lucide::PLUS, action)`. The selected vendored set
is not the complete catalogue. See [asset provenance and licence](../blockui-icons-lucide/README.md).

## Linked release

Generated apps include a native executable behind the `native` feature:

```sh
wabou check APP --static
wabou doctor APP --static
wabou build APP --static --release
wabou run APP --static --release
```

The static command enables the selected binary's required features. Set
`package.metadata.blockui.static-bin` for ambiguous binaries. The linked
application runs without an external UI dylib, Cargo, watcher or generation
copies. This does not yet strip every loader dependency from the runtime.
Static entry helpers currently support one root actor.

## Verification

From the checkout:

```sh
python3 scripts/native/check.py
python3 scripts/native/check.py --full
```

Default checks run SDK, executable-guide and CLI tests without opening a
window. Optional checks cover native widget state and services, actual dylib
and async replacements, selected Vello pixels, and executable-only release
capture. Pixel/window checks use isolated Xvfb, require an available adapter,
and never send input to the desktop. Reports live under `reports/`.

See [verification details](../../scripts/native/README.md) for coverage and
individual commands. Watcher timings exclude rendering/presentation. Current
native runtime proofs are local Linux evidence, not cross-platform guarantees.

The Rust and TSX frontends share native shell/widgets. Their UI projection
protocols, capability providers and background schedulers are not unified.
Native service tests cover the actor service catalogue, not a capability adapter.
