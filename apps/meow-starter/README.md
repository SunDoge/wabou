# Meow Starter visual parity experiment

This app rebuilds the visual direction of the WebView-based
[`meow-starter`](https://github.com/Shiaoming123/meow-starter) concept with Wabou's
Solid renderer, Vello Hybrid backend, Style IR, and native window APIs. It is a
rendering and component-system comparison rather than a port of the original
Tauri APIs.

Run it from the Wabou workspace:

```sh
bun install
bun run --filter @wabou/meow-starter build
cargo run -p meow-starter-wabou
```

Render deterministic 1x and HiDPI captures without opening a window:

```sh
target/release/wabou render apps/meow-starter \
  --width 1000 --height 680 \
  --out /tmp/meow-starter-wabou.png

target/release/wabou render apps/meow-starter \
  --width 1000 --height 680 --scale-factor 2 \
  --out /tmp/meow-starter-wabou@2x.png
```

The layout contract covers the normal 1000×680 window and the supported
minimum 820×560 window:

```sh
bun run test:layout:quick --app meow-starter
```
