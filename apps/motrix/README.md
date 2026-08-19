# Motrix · Wabou

A Wabou dogfood application inspired by the MIT-licensed
[`motrix-gpui`](https://github.com/missuo/motrix-gpui) prototype in the adjacent development
checkout. It exercises a realistic desktop-download-manager surface rather
than serving as a component-only demo.

The current implementation includes routed Dashboard, Downloads, Trackers,
Plugins, Notifications, and Settings surfaces; URL task creation and torrent
metadata preview with pre-download file selection;
search, status filters, task inspection with file/peer details, a bounded
single-path BitTorrent piece map, and editable per-task trackers; batch
actions, waiting-queue reordering, per-task speed limits, task-row native
context menus, advanced HTTP request options, and optional system-Trash removal
of downloaded files; durable Stop Seeding and Re-seed for completed torrents;
revisioned Rust-to-JavaScript task patches; animated live
RPC speed charts; a persistent year-long transfer heatmap; tracker synchronization; native
and in-app completion/error notifications; configurable BT/DHT listen ports;
automatic PCP/NAT-PMP/UPnP port mapping with network-change renewal;
editable paired download/upload speed profiles;
light/dark/follow-system themes; and an optional in-app confirmation before
terminating active downloads.

Desktop shortcuts follow the official Motrix conventions: `Primary+N` creates
a URL task, `Primary+Shift+N` or `Primary+O` opens torrent creation,
`Primary+L` opens Downloads, `Primary+,` opens Settings, and
`Primary+Shift+P` / `Primary+Shift+R` pause or resume all tasks. `Primary+B`
toggles the sidebar.

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
Use **Open Motrix** to restore the window. **Quit Motrix** restores the window
when necessary, asks before interrupting unfinished work, saves the aria2
session, and stops the managed process cleanly. The same flow is available via
`Primary+Q` and can be disabled in General settings.

Daily transfer totals are derived from completed-byte deltas rather than UI
frame timing and are saved as `activity.json` in the same platform-native
configuration directory.

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
