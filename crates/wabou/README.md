# wabou

`wabou` is the public Rust application runtime for Wabou. It combines the
SolidJS host with the Winit + Taffy + Parley + Vello renderer behind one
application-facing crate.

During the developer preview, depend on a tagged Git revision:

```toml
[dependencies]
wabou = {
  git = "https://github.com/SunDoge/wabou.git",
  tag = "v0.1.0-alpha.4",
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

The crate is intentionally not published to crates.io yet. Its first job is to
validate and stabilize the public boundary while internal crates can still be
merged or renamed.
