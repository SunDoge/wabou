# wabou-terminal

`wabou-terminal` adapts `rio-vt` to Wabou's Rust-side `Widget` API.

`rio-vt` is deliberately renderer-free: it parses PTY bytes into a terminal
grid and emits PTY-facing events. `TerminalWidget` owns that grid, uses Rio's
`Machine` to drive it from a real PTY, and renders the visible cells with
Parley and Wabou's AnyRender scene.

```rust
HostBuilder::new()
    .widget("terminal", wabou_terminal::terminal_widget)
    .run()?;
```

Solid applications can consume the typed PascalCase wrapper and event types
from `@wabou/terminal`; the package also declares the low-level `terminal` JSX
intrinsic for applications that need direct wire-attribute access.

The widget factory launches lazily on its first paint, after initial JSX
attributes have arrived. This makes per-tab process configuration reliable:

```tsx
<Terminal command="ssh" args={["example.com"]} cwd="/tmp" />
```

`command`, `args`, and `cwd` are initial launch options. Updating them after
the PTY starts does not implicitly kill or restart the running process.

Unmounting a terminal shuts down its Rio reader and PTY. On Unix, Wabou owns
the final process-group cleanup: it sends `SIGHUP`, allows a short graceful
exit window, escalates to `SIGKILL` when necessary, and calls `waitpid`. This
keeps closing tabs from leaking background jobs or zombie child processes.

After the child exits, the retained terminal becomes read-only: scrollback,
text selection, links, and copy remain available, while keyboard input,
paste, focus reports, and stale application mouse modes no longer write to or
capture input for the closed PTY.

On Unix, Wabou advertises terminal capabilities to the child process with
`TERM=xterm-256color`, `COLORTERM=truecolor`, `TERM_PROGRAM=wabou`, and
`TERM_PROGRAM_VERSION`. The variables are injected through the child command
instead of mutating the host process environment, so independently launched
terminals and runtimes cannot race over global environment state. Windows
continues to use the native ConPTY launch path; Wabou applies Windows command
line quoting before entering teletypewriter so executable paths and arguments
containing spaces, quotes, or trailing backslashes retain their argv boundaries.

The Solid application can then size and compose it normally:

```tsx
<terminal class="w-full h-full overflow-hidden" />
```

Font props are reactive. Wabou measures terminal cell metrics before layout,
so auto-sized terminals update their intrinsic size in the same frame; an
explicit `lineHeight` is retained as authored and clamped against the current
font size without making prop order observable.

Selection colors can be themed without changing terminal palette semantics:

```tsx
<Terminal selectionBackground="#2563eb80" selectionForeground="#ffffff" />
```

If `selectionForeground` is omitted, selected cells retain their ANSI foreground
color under the translucent selection fill.

Set `inheritTheme` to use the terminal host element's resolved `color` and
`background-color` as its default terminal foreground/background. It is opt-in
so an unstyled terminal keeps its built-in readable palette. Explicit ANSI
colors and OSC palette changes continue to take priority.

`onTerminalSelectionChange` receives
`{ text: string | null, kind: "simple" | "word" | "line" | "block" | null }`
when a pointer selection gesture commits or another operation changes the effective selection.
Dragging updates the highlight locally without repeatedly serializing large
scrollback selections. Other events are deduplicated when copied text is unchanged.

OSC title changes are node-scoped by default through `onTerminalTitleChange`.
This prevents a background tab from overwriting the native window title. A
single-terminal app may opt into direct mirroring with
`<Terminal syncWindowTitle />`; removing the prop restores the host's default
title. Unmounting the widget performs the same cleanup through Wabou's widget
lifecycle hook.

Implemented paths:

- VT/ANSI parsing, SGR foreground/background, 256 colors and cursor shape;
- real shell PTY input/output and terminal resize (`SIGWINCH` through the PTY);
- idempotent PTY exit reporting and reconfigurable launch options after a spawn failure;
- committed text, control keys, navigation keys and smooth accumulated scrollback wheel input;
- mode-aware cursor/function keys, modifiers, focus reporting and alternate-screen scrolling;
- simple, semantic (double-click), line (triple-click) and Alt/Option block selection,
  with distance-sensitive viewport autoscroll;
- system clipboard copy/paste and bracketed-paste mode, with control-character
  sanitization on both paste paths;
- platform terminal clipboard shortcuts (`Cmd+C/V` on macOS, `Ctrl+Shift+C/V` elsewhere),
  without stealing `Ctrl+C/V` process input;
- SGR, UTF-8 and legacy mouse reporting, with Shift bypass for local selection;
- node-scoped OSC title events, opt-in native window-title synchronization,
  and OSC 52 clipboard-store requests;
- cancellable primary-shortcut OSC 8 link clicks, opened through the host's
  HTTP(S)-only external URL boundary;
- OSC palette/foreground/background/cursor color queries and terminal bell;
- application-controlled steady and blinking cursor modes;
- cell-aligned built-in light/heavy box drawing for contiguous terminal borders,
  with font fallback for box-drawing variants not yet covered;
- HiDPI-aware glyph and bitmap emoji rendering;
- Kitty, Sixel and iTerm2 inline graphics with scrollback-aware placement,
  source cropping, z-ordering and panel clipping;
- measured font advances for cell geometry and multi-codepoint grapheme rendering;
- incremental Rio damage snapshots with reusable row/style/extras storage, so
  cursor-only and partial frames do not reallocate and clone the full viewport;
- event-loop wakeups from the PTY reader, without polling continuously;
- headless byte feeding for deterministic parser and input tests.

OSC 52 clipboard reads are denied by default and return an empty response. An
application may opt in explicitly with `<terminal allow-clipboard-read />`.
Desktop notification events are exposed to Solid so the application can apply
its own permission and presentation policy.
