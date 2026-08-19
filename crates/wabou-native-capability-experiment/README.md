# Native capability experiment

This excluded crate preserves Wabou's direct QuickJS-value capability
prototype. It is deliberately outside the root workspace, CI, release graph,
and public API while the main runtime uses the simpler JSON capability
contract.

The experiment records the useful design properties without forcing them on
the current framework:

- request and response DTOs cross QuickJS through `rquickjs-serde` without
  intermediate JSON text;
- a Rust handler becomes a JavaScript Promise;
- serde owns the runtime shape and a typed method token owns the name;
- native failures reject the Promise instead of returning a result envelope.

Check it independently with:

```sh
cargo check --manifest-path crates/wabou-native-capability-experiment/Cargo.toml
```

This is reference code, not a compatibility promise. If the direct-value
design is revived, move the contract and its binding generator back into the
workspace only after measuring a real need.

`direct-ffi.patch` is the full working-tree patch from the validated prototype,
including the Specta binding generator, generated clients, runtime adapter,
tests, and documentation. The small crate in `src/lib.rs` keeps its essential
runtime mechanism independently compilable; the patch preserves the complete
implementation for later comparison or revival.
