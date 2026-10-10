# Rust native frontend verification

Run the default checks without opening a window:

```sh
python3 scripts/native/check.py
```

Add `--native-state` for native widget state and stable-host service tests without
a window. Native library tests intentionally leave GPU pixel and benchmark cases
ignored; those require separate explicit validation.

Add `--pixels` for explicit native hover/pressed/disabled, button appearance and
reactive-style pixel checks in isolated Xvfb. This requires an available Vello
adapter; it does not synthesize desktop input.

Add `--reload` for real dylib and async replacement checks, or `--release` for
an isolated linked release capture. The optional capture requires Linux/Xvfb.
`--full` runs all native checks and strict SDK/CLI clippy. It requires
rust-analyzer, Xvfb and a working Vello adapter.

Results are written to `reports/native-check.json`; the command exits on the
first failure. Existing renderer dependencies may be compiled for CLI tests,
but the default checks do not initialize a renderer.

Individual checks from the Wabou repository root on Linux:

```sh
python3 scripts/native/verify_watch.py --trailing-ui --wabou-native
python3 scripts/native/verify_watch.py --async-value --wabou-native
cargo build -p wabou-cli
python3 scripts/native/verify_static_native.py --wabou-cli target/debug/wabou
```

These tests compile and replace actual dylibs, retain actor state through build
failures, test source/dependency changes, measure allocated fixture storage and
check generation cleanup. They do not initialize a renderer or send desktop
input. Timing excludes layout, rendering and presentation. Plugins are trusted
local fixtures. Native window proofs require an isolated display.

Use `--editor` for actual rust-analyzer queries in the workspace helper fixture
and independent ordinary/renamed facade consumers (eight queries each). It
requires the rust-analyzer rustup component. The native facade and counter are
default workspace members so the default Cargo check loads their procedural
macros; removing them from that set can break workspace completion even when
external projects still work.

The normal `--reload` check also exercises a compiled candidate with a changed
state schema whose migration writes count=999 and then rejects replacement.
It checks rollback of the write, retention of the previous implementation,
continued processing by its old action, recovery after restoring the source,
and one retained generation. Run it individually with:

```sh
python3 scripts/native/verify_watch.py --trailing-ui --wabou-native --migration
```

Add `--diagnostics` for actual compiler primary-span checks against normal and
renamed SDK imports. Text type, style type, unsupported child and unknown
property errors must point to authored source lines; restored source must check
successfully. The check uses independent temporary Cargo consumers and writes
`reports/native-diagnostics.json`.
