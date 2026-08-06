# @wabou/terminal

Typed Solid bindings for the native `wabou-terminal` Rust widget.

```tsx
import { Terminal } from "@wabou/terminal";

<Terminal
  command="ssh"
  args={["example.com"]}
  onTerminalSelectionChange={(event) => console.log(event.text)}
/>;
```

The application host must register `wabou_terminal::terminal_widget` as the
`terminal` widget factory. `Terminal` maps its camelCase props to the native
widget's wire attributes and provides event types; it does not own the PTY.
