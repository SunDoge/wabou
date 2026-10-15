# Pulse reference study

A Wabou/Vello Hybrid recreation of [Pulse's light Overview](https://pulse.egoist.dev/).
All numbers are fixed sample data. This is a visual study, not a system monitor.
Lucide icons and local fonts differ from the reference. Native decorations are
disabled; the custom traffic-light controls invoke close, minimize and maximize.
The empty header region starts native dragging; double-click toggles maximize.
Compositor dragging still needs desktop/platform verification. App icons and the full process breakdown are
not reproduced in this first study.

From the repository root:

```sh
bun install
cargo run -p wabou-cli -- dev apps/pulse
```

The app is frontend-only and uses `wabou.config.ts`. Its shared screen also has
Gallery layout fixtures, allowing native offscreen comparison:

```sh
cargo run -p wabou-cli -- render apps/gallery --fixture demos/pulse-overview --out /tmp/pulse.png
cargo run -p wabou-cli -- render apps/gallery --fixture demos/pulse-compact --out /tmp/pulse-compact.png
bun x vitest run tests/component/pulse.vitest.tsx
WABOU_LAYOUT_COMMAND=target/debug/wabou-render bun apps/gallery/tests/layout.ts demos/pulse-overview demos/pulse-compact
```

Interval selection changes the sample history projection. Pause/resume changes
control state; no live telemetry or periodic simulation runs. Sidebar rows summarize
the sample and are intentionally not interactive navigation into unimplemented pages.
