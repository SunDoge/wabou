# Terminal demo

This example is a small multi-session terminal application rather than a
single isolated widget. It keeps inactive PTYs mounted, follows OSC window
titles per tab, and demonstrates native focus, close/reopen lifecycle, and
application-level shortcuts:

- `Ctrl/Cmd+T` opens a terminal;
- `Ctrl/Cmd+W` closes the active terminal;
- `Ctrl+Tab` and `Ctrl+Shift+Tab` switch sessions.

The demo requests the system font family `Hack Nerd Font Mono`. Wabou's
Parley/fontique text stack discovers fonts installed through the operating
system, so the application does not contain a machine-specific font path.

Install Hack Nerd Font with your platform's normal font manager before running
the demo if you want Powerline and devicon glyphs. When that family is not
available, fontique falls back to another installed font; ordinary terminal
text remains usable, but Nerd Font icons may be missing.

```sh
bun run wabou dev apps/terminal
```

`useHost().fonts.load(path)` remains available for local experiments, but
application source should not depend on an absolute path. A distributable
application that requires identical glyphs on every machine should embed a
redistributable font through the Rust host instead.
