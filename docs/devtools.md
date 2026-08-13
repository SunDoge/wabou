# Wabou DevTools MVP

Debug builds start a read-only local DevTools server automatically. The server
publishes immutable copies of runtime state; its socket thread never accesses
QuickJS, Taffy, widgets, the window or the GPU directly.

## Native inspector

The visual inspector is itself a Wabou application. Build its Solid bundle,
then launch its Rust host while a debug Wabou application is running:

```bash
mise exec -- cargo run -p wabou-cli -- devtools
```

It auto-discovers the newest live target, or accepts an explicit Unix socket
path. The inspector provides searchable nodes, computed layout and style,
event listeners, recent binary-protocol frames, on-demand screenshots, and a
selected-node overlay. Disconnect errors retain the last useful snapshot.

The inspector disables its own DevTools server, so it cannot accidentally
discover or recursively inspect itself.

For distribution, build the GUI like any other Wabou application:

```bash
mise exec -- cargo run -p wabou-cli -- build apps/devtools --release
```

This produces `dist/devtools/wabou-devtools` with its
`resources/bundle.js`. Install that directory intact. `wabou devtools` and
`wabou dev --devtools` look for `wabou-devtools` next to the `wabou` binary
first, then on `PATH`; `WABOU_DEVTOOLS_SOCKET` selects a specific target.

## CLI

The CLI discovers the newest live Wabou process in `$XDG_RUNTIME_DIR`. Set
`WABOU_DEVTOOLS_SOCKET` when more than one application is running.

```bash
mise exec -- cargo run -p wabou-cli -- inspect status
mise exec -- cargo run -p wabou-cli -- inspect query comments
mise exec -- cargo run -p wabou-cli -- inspect node 42
mise exec -- cargo run -p wabou-cli -- inspect at 820 600
mise exec -- cargo run -p wabou-cli -- inspect frames --limit 20
mise exec -- cargo run -p wabou-cli -- inspect screenshot
mise exec -- cargo run -p wabou-cli -- inspect capture --x 820 --y 600 --output /tmp/wabou-case
```

`query` searches tags, text and classes. `node` returns structure, attributes,
listeners, border/content rectangles and a compact computed-style summary.
`frames` returns the bounded trace of both `hostToJs` HostEventFrames and
`jsToHost` mutation frames. Raw bytes are omitted because they may contain
application data. Set `WABOU_DEVTOOLS_RAW_FRAMES=1` to opt in; previews are
then capped at 4 KiB per trace record.

`at` finds the topmost hit-testable node at logical window coordinates and
returns its ancestor chain. Node inspection includes the widget-local content
clip, every contributing ancestor overflow clip, the effective window-logical
clip, the content-to-window transform, and the device scale. Coordinate spaces
are named explicitly in the response.

`capture` performs one screenshot handshake and freezes the immutable tree,
recent protocol frames, and optional point inspection when that same rendered
frame completes. It writes `screenshot.png`, `manifest.json`, `tree.json`, and,
when a point hits a node, `selected-node.json` into the output directory.

Screenshots are rendered only on request and written as mode `0600` PNG files.
Normal frames do not perform a GPU readback.

## MCP

`wabou-mcp` is a thin stdio adapter over the same socket protocol:

```bash
mise exec -- cargo build -p wabou-devtools --bin wabou-mcp
```

Configure Codex, Claude or another MCP client to launch:

```text
/home/me/code/projects/wabou/target/debug/wabou-mcp
```

Optionally provide `WABOU_DEVTOOLS_SOCKET=/run/user/1000/wabou-<pid>.sock`.
The server exposes:

- `wabou_status`
- `wabou_query_nodes`
- `wabou_inspect_node`
- `wabou_inspect_at_point`
- `wabou_recent_frames`
- `wabou_set_layout_overlay`
- `wabou_capture_screenshot`
- `wabou_capture_case`

The screenshot and capture-case tools return standard MCP `image` content
(`image/png`) plus structured JSON, so multimodal clients can inspect pixels
and frame-matched runtime evidence in one call. The MCP server uses the
official Rust `rmcp` SDK; tool schemas come from the serde/schemars parameter
types rather than hand-maintained JSON.

## Headless screenshots

`wabou render` evaluates the application with the same host-global ordering as
the native runtime. It defaults to logical window 1 and a 1× device scale:

```sh
wabou render apps/gallery --out /tmp/gallery.png
```

Multi-window and HiDPI states can be selected explicitly. Width, height, and
interaction coordinates remain logical pixels; the PNG dimensions are scaled:

```sh
wabou render apps/gallery --out /tmp/gallery@2x.png \
  --width 1440 --height 900 --window-id 1 --scale-factor 2
```

Repeat `--click X Y` to replay a short navigation path. `--text` is committed
to the element focused by the final click:

```sh
wabou render apps/gallery --out /tmp/input.png \
  --click 45 250 --click 700 760 --text $'hello\nworld'
```

## Runtime API and security

`HostBuilder` enables DevTools by default only in debug builds. Override it
explicitly when embedding:

```rust
HostBuilder::new().devtools(false).run()?;
```

The transport is a Unix domain socket with mode `0600`; it never listens on
TCP. Requests are capped at 1 MiB, node queries at 1000 results, traces at 128
records and screenshots use Host-chosen temporary paths. Attribute names that
contain `password`, `token`, `secret` or `authorization` are redacted, and text
or attribute values are capped at 4096 characters. Stale socket files are
removed during discovery only after confirming that they are Unix sockets;
ordinary files are never replaced. Windows named-pipe support is not part of
this Unix MVP yet.
