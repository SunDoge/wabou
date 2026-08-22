# Motrix · Wabou

A Wabou dogfood application inspired by the MIT-licensed
[`motrix-gpui`](https://github.com/missuo/motrix-gpui) prototype in the adjacent development
checkout. It exercises a realistic desktop-download-manager surface rather
than serving as a component-only demo.

The current implementation includes routed Dashboard, Downloads, Trackers,
Notifications, and Settings surfaces; HTTP/HTTPS multi-connection downloads;
magnet and torrent creation with metadata preview and pre-download file
selection; clipboard paste with browser cURL interpretation; native directory
selection; search and status filters; pause, resume, retry, remove, and batch actions;
revisioned Rust-to-JavaScript task patches; animated throughput charts;
persistent transfer activity; native and in-app completion/error notifications;
light/dark/follow-system themes; and graceful native-host shutdown.

The Trackers page reports DHT, PEX, listen-port, NAT, torrent-task, peer, and
magnet tracker endpoint state. `gosh-dl` currently keeps per-announce tracker
health internal, so the UI deliberately does not invent latency or health
values that the engine cannot report yet.

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

The Rust host embeds `gosh-dl`; there is no daemon, RPC port, secret, or child
process to install and supervise. Settings and download recovery data are kept
under Wabou's platform-native application directories. The engine is shut down
gracefully with the native host so active task state reaches SQLite before the
process exits.

Closing the main window keeps managed downloads running in the system tray.
Use **Open Motrix** to restore the window. **Quit Motrix** restores the window
when necessary, asks before interrupting unfinished work, persists engine
state, and stops the embedded runtime cleanly. The same flow is available via
`Primary+Q` and can be disabled in General settings.

Daily transfer totals are derived from completed-byte deltas rather than UI
frame timing and are saved as `activity.json` in the same platform-native
configuration directory.

## Backend boundary

Motrix uses the embedded, MIT-licensed `gosh-dl` engine. HTTP/HTTPS
multi-connection downloads are the first priority; BitTorrent initially
targets torrents and magnets, file selection, pause/resume, and basic seeding.

The backend boundary is:

- Rust owns the embedded engine, persistence, filesystem operations, and
  system notifications.
- A typed application capability carries user commands from JavaScript to
  Rust.
- `HostMessage` carries normalized task snapshots/diffs and engine events from
  Rust to Solid.
- Solid owns routing, selection, filtering, presentation state, and rendering.

`DownloadTask` is Motrix-owned: engine types are projected at the Rust boundary
and never exposed to JavaScript. This keeps the UI contract independent from
gosh-dl while retaining an entirely in-process Rust implementation.
