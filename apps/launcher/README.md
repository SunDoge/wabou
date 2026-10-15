# Wabou Launcher

A keyboard-first native application launcher and a compact reference app for
Wabou's window, tray, capability, and accessibility APIs.

```bash
bun run wabou dev apps/launcher
```

Press **Super+Shift+Space** to reveal the launcher, type to filter installed
applications, use the arrow keys to select a result, and press **Enter** to
launch it. **Escape** and the window close action hide the launcher while its
tray process remains available.

Application discovery currently supports Linux desktop entries and macOS app
bundles. The global shortcut backend supports macOS, Windows, and Linux under
X11; on Wayland the tray's **Show Launcher** action remains available.

The launcher capability is declared once in Rust. Regenerate its typed Solid
client after changing a request or response DTO:

```bash
bun run wabou bindings write apps/launcher
```
