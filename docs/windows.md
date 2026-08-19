# Windows

Each native window owns an independent QuickJS runtime, Solid root, layout
tree, renderer surface, input state, and animation queue. The runtimes execute
the same application bundle; branch on the current window ID when a secondary
window needs different content.

```tsx
import { createWindow, useWindow } from "@wabou/ui";

function App() {
  const window = useWindow();

  if (window.id !== 1) {
    return <Settings onDone={() => window.close()} />;
  }

  return (
    <Button
      onPress={() =>
        createWindow({ title: "Settings", width: 720, height: 480 })
      }
    >
      Settings
    </Button>
  );
}
```

`createWindow(options)` creates a new runtime and returns a `WindowHandle`.
The handle can close, minimize, maximize, retitle, or begin dragging that
specific window. `useWindow()`
returns the current runtime's reactive metrics plus the same controls.

## Custom title bars

Disable native decorations when the window is created, then explicitly start
the compositor-managed drag operation from the non-interactive part of your
title bar. Do not start dragging from title-bar buttons.

```rust
HostBuilder::new()
    .window(WindowOptions::new().title("Wabou").decorations(false))
    .run()?;
```

```tsx
import { useWindow } from "@wabou/ui";
import {
  Button,
  Text,
  TitleBar,
  TitleBarDragRegion,
} from "@wabou/ui";

function TitleBar() {
  const window = useWindow();
  return (
    <TitleBar>
      <TitleBarDragRegion class="justify-center px-3">
        <Text>Wabou</Text>
      </TitleBarDragRegion>
      <Button variant="ghost" size="icon" onPress={() => window.minimize()}>
        <Text>_</Text>
      </Button>
      <Button variant="ghost" size="icon" onPress={() => window.close()}>
        <Text>X</Text>
      </Button>
    </TitleBar>
  );
}
```

`startDragging()` delegates movement to the OS compositor, so it works across
Wayland, X11, Windows, and macOS without manually updating window coordinates.

Rust applications may also create initial windows before the event loop starts:

```rust
HostBuilder::new()
    .window(WindowOptions::new().title("Main"))
    .additional_window(WindowOptions::new().title("Inspector"))
    .run()?;
```
