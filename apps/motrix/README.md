# Motrix · Wabou

A Wabou dogfood application inspired by the MIT-licensed
[`motrix-gpui`](https://github.com/missuo/motrix-gpui) prototype in the adjacent development
checkout. It exercises a realistic desktop-download-manager surface rather
than serving as a component-only demo.

The current implementation includes routed Dashboard, Downloads, Trackers,
Plugins, Notifications, and Settings surfaces; URL and torrent task creation;
search, status filters, task inspection with file/tracker/peer details, batch
actions, and optional system-Trash removal of downloaded files; live RPC speed
charts; tracker synchronization; native completion/error notifications; and
light/dark themes.

## Run

```sh
bun install
wabou dev apps/motrix
```

By default the Rust host starts an owned `aria2c` on an available loopback port
with a generated RPC secret and stops it when Wabou exits. Set
`WABOU_ARIA2_BIN` to select another executable. To control an existing local or
remote daemon instead, set `WABOU_ARIA2_URL` and optionally
`WABOU_ARIA2_SECRET`; external daemons are never stopped by the app.
Set `WABOU_ARIA2_DISABLE_MANAGED=1` only when a test or restricted environment
must render the app without starting its managed daemon.

The Settings page can switch between the managed and external modes at
runtime. It persists its JSON configuration under Wabou's platform-native app
config directory. Environment variables override the saved RPC connection for
the current launch without overwriting it.

Managed aria2 tasks survive application restarts. Motrix initializes an
`aria2.session` beside its configuration, asks aria2 to update it periodically,
and performs `saveSession` followed by a graceful RPC shutdown before falling
back to terminating the owned process group.

Closing the main window keeps managed downloads running in the system tray.
Use **Open Motrix** to restore the window or **Quit Motrix** to save the aria2
session and stop the managed process cleanly.

## Backend boundary

The intended production boundary is:

- Rust owns the aria2c child process, RPC secret, WebSocket reconnect,
  persistence, filesystem operations, and system notifications.
- A typed application capability carries user commands from JavaScript to
  Rust.
- `HostMessage` carries normalized task snapshots/diffs and daemon events from
  Rust to Solid.
- Solid owns routing, selection, filtering, presentation state, and rendering.

The host uses `aria2-ws` and has been exercised against aria2c 1.37.0. The UI
does not import an aria2 client package directly.
