# wabou

`wabou` is the public Rust application API for the Wabou native SolidJS
runtime. It is a facade over Wabou's implementation crates so applications do
not need to depend directly on `wabou-runtime` or `wabou-shell`. The public
runtime is GPUI-CE-only; retired `wabou-legacy-*` crates are repository-local
migration oracles rather than application dependencies or optional backends.

During the developer preview, depend on a tagged Git revision:

```toml
[dependencies]
wabou = {
  git = "https://github.com/SunDoge/wabou.git",
  tag = "v0.1.0-alpha.3",
}
```

```rust
use wabou::{HostBuilder, WindowOptions};

fn main() -> wabou::Result<()> {
    HostBuilder::new()
        .window(WindowOptions::new().title("My Wabou app"))
        .run()
}
```

The facade is intentionally not published to crates.io yet. Its first job is
to validate and stabilize the public boundary while internal crates can still
be merged or renamed.
