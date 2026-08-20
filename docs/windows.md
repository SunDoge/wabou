# Windows

Each native window owns an independent QuickJS runtime, Solid root, layout
tree, renderer surface, input state, and animation queue. The runtimes execute
the same application bundle; branch on the current window ID when a secondary
window needs different content.

```tsx
import { createWindow, useWindow } from "@wabou/ui";

function App() {
  const window = useWindow();

  if (window.id.lo !== 1 || window.id.hi !== 1) {
    return <Settings onDone={() => window.close()} />;
  }

  return (
    <Button
      onPress={() =>
        void createWindow({ title: "Settings", width: 720, height: 480 })
      }
    >
      Settings
    </Button>
  );
}
```

`createWindow(options)` creates a new runtime and resolves to a `WindowHandle`
after the native surface exists. Creation failures reject the Promise. The
handle carries a full `{ lo, hi }` generational key rather than reusing the
effect request id.
The handle can close, minimize, maximize, retitle, or begin dragging that
specific window. `useWindow()`
returns the current runtime's reactive metrics plus the same controls.

## Sizing and responsive layout

Treat the native size as a preferred starting point, not as a substitute for
responsive layout. Window managers may clamp it to the current work area, and
font metrics differ across platforms. Set a useful initial and minimum client
area, then adapt optional UI from the reactive logical size:

```rust
HostBuilder::new()
    .app_directories("com", "Example", "Workbench")
    .persist_window_size("main")
    .window(
        WindowOptions::new()
            .initial_inner_size(1280, 840)
            .min_inner_size(900, 600),
    )
    .run()?;
```

```tsx
import { createWindowMatch, View } from "@wabou/ui";

function Workspace() {
  const compact = createWindowMatch({ maxWidth: 1099 });
  return (
    <View class={compact() ? "flex flex-col" : "flex flex-row"}>
      {/* panes */}
    </View>
  );
}
```

Window width is the right input for application-wide navigation changes. A
component inside a sidebar, split pane, card, or dialog should instead respond
to its own completed native content box:

```tsx
import { createContainerMatch, View } from "@wabou/ui";

function Results() {
  const compact = createContainerMatch({ maxWidth: 639 });
  return (
    <View
      ref={compact.ref}
      class={compact.matches() ? "grid-cols-1" : "grid-cols-2"}
    >
      {/* results */}
    </View>
  );
}
```

The match remains false until the first native measurement. Resize observations
are delivered after layout and update Solid on the following turn, avoiding a
recursive layout pass.

`ResponsiveGrid` packages the common measured-column calculation. Descendant
components can call `useResponsiveGrid()` to read its reactive `columns`,
`width`, and `height`; this keeps card density and internal arrangement tied to
the space the grid actually received rather than a second window breakpoint.

Persisted sizes live under the configured application-local data directory.
Wabou records only a valid non-maximized logical size, clamps it to the current
minimum, and restores it before creating the native surface, so startup does
not visibly resize after JavaScript boots.

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
