# wabou

`wabou` is the public Rust application API for the Wabou native SolidJS
runtime. It is a facade over Wabou's implementation crates so applications do
not need to depend directly on `wabou-quick`, `wabou-shell`, or
`wabou-widgets`.

During the developer preview, depend on a tagged Git revision:

```toml
[dependencies]
wabou = {
  git = "https://github.com/SunDoge/wabou.git",
  tag = "v0.1.0-alpha.1",
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
