# Transparent window lab

This experiment validates a transparent, undecorated native Wabou window with
Vello content, pointer input, and an always-on-top request.

```sh
bun run wabou dev apps/transparent-window
```

The desktop should remain visible through the dimmed surface. Drag to draw a
selection rectangle and use **Close** to exit.

`AlwaysOnTop` is advisory and is currently ignored by Wayland compositors.
This app does not capture the desktop; screen capture and OCR are separate host
capabilities.
