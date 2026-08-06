# Windows

Each native window owns an independent QuickJS runtime, Solid root, layout
tree, renderer surface, input state, and animation queue. The runtimes execute
the same application bundle; branch on the current window ID when a secondary
window needs different content.

```tsx
import { createWindow, useWindow } from "@wabou/core";

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
The handle can close, maximize, or retitle that specific window. `useWindow()`
returns the current runtime's reactive metrics plus the same controls.

Rust applications may also create initial windows before the event loop starts:

```rust
HostBuilder::new()
    .window(WindowOptions::new().title("Main"))
    .additional_window(WindowOptions::new().title("Inspector"))
    .run()?;
```
