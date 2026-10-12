# Experimental Rust UI

Blockui, its runtime/tooling crates, `ui-contract`, the `wabou-native` adapter,
and the native counter example live here. The primary Wabou frontend uses
QuickJS, SolidJS and TSX; this directory preserves the Rust UI experiment.

These packages share the root Cargo workspace and lockfile, but are excluded
from its default members. Select them explicitly:

```sh
cargo check -p blockui-runtime --all-features
cargo test -p blockui --all-features
cargo test -p wabou-native --all-features
cargo run -p wabou-cli --features native-ui -- native check native/apps/native-counter --tests
```

The renderer is shared with Wabou through `wabou-shell-vello`. Native widgets
are available in its `widgets` module with the `widgets` feature. The experimental
Rust UI does not require Wabou's optional `widget-hot-reload` adapter.
