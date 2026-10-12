# Native Rust counter

Minimal tag UI application with a live text observer and actor actions. It uses
Wabou's experimental Rust frontend. SDK/runtime live inside this Wabou checkout. This example does not use TSX or QuickJS.

From the Wabou repository root, after `cargo build -p wabou-cli --features native-ui`:

```sh
./target/debug/wabou native check native/apps/native-counter --tests
./target/debug/wabou native test native/apps/native-counter
./target/debug/wabou native dev native/apps/native-counter
./target/debug/wabou native build native/apps/native-counter --static --release
./target/debug/wabou native run native/apps/native-counter --static --release
```

The headless test clicks Increment and checks that only the live text observer
reevaluates. It also verifies state retention across compiled-in implementation
replacement. That test does not prove real dylib replacement; `dev` uses the
actual watcher. Formatting uses rustfmt.

The optional `native` feature enables the linked executable. Development keeps
platform callbacks and resources in the stable host while replacing actor code.
