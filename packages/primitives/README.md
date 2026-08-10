# `@wabou/primitives`

Solid primitives for Wabou applications.

```tsx
import { Image, Text, View } from "@wabou/primitives";

<View class="flex items-center gap-2">
  <Image class="w-6 h-6" src="avatar.png" />
  <Text class="text-sm">{count()} stories</Text>
</View>;
```

`View` is a layout container. `Text` is an explicit native text boundary: all
of its static and reactive text children form one measured run and one item in
its parent's layout. Use `Text` rather than placing bare text directly in a
flex or grid container.

`View` does not choose a layout mode. Alignment utilities such as
`items-center`, `justify-center`, and `gap-2` only take effect after the same
node explicitly enables flex or grid. Prefer the layout primitives when their
intent matches the UI:

```tsx
import { Center, Column, Row } from "@wabou/primitives";

<Row class="items-center gap-2">...</Row>
<Column class="gap-3">...</Column>
<Center class="w-8 h-8 rounded-full">...</Center>
```

Each layout primitive creates exactly one native `view`; it does not add an
extra wrapper. Use `View class="flex ..."` when more direct CSS control is
needed.

Repeating component motion has the same one-node guarantee:

```tsx
<Spin duration={0.9} class="w-4 h-4">...</Spin>
<Pulse from={0.45} to={0.85} duration={1.8} class="rounded bg-control" />
```

Both primitives expose `paused` and `speed`. Their Motion playback controls and
Solid cleanup are owned internally; use `createRotation`, `createPulse`, or
`createLoop` from `@wabou/animation` when the animated value must be composed
manually.

The capitalized APIs create internal `view`, `text`, `img`, and `textarea` host
nodes. Those tags are implementation details and should not be written directly
by apps.

`TextArea` is the native multiline editor. It supports soft wrapping, keyboard
and pointer selection, clipboard operations, controlled values, read-only and
disabled states, and internal vertical scrolling:

```tsx
<TextArea
  class="w-96 h-32"
  value={notes()}
  placeholder="Notes"
  onInput={(event) => setNotes(event.currentTarget.value)}
/>
```

`PasswordInput` stores its value only in a Rust `SecretStore`. QuickJS receives
the non-sensitive slot identifier but never a `value` attribute or input-event
payload. Register the store on the host and atomically take the secret in the
native capability:

```rust
let secrets = SecretStore::default();
let login_secrets = secrets.clone();
HostBuilder::new()
    .password_inputs(secrets)
    .capability("vault", move |ctx, capability| {
        // Inside the native login function:
        let password = login_secrets.take("master-password");
        // Pass `password` directly to the Rust SDK.
        Ok(())
    });
```

```tsx
<PasswordInput
  secret="master-password"
  aria-label="Master password"
  placeholder="Master password"
/>;
```

Taking, unmounting, or explicitly clearing a slot removes it from the store;
the returned `Zeroizing<String>` clears its allocation when dropped. Do not use
the ordinary controlled `Input type="password"` when the value must stay out of
JavaScript.

Use `Button` for clickable controls that need consistent native interaction
feedback without depending on CSS pseudo-classes:

```tsx
<Button tone="sky" selected={active()} onClick={activate}>
  Open
</Button>
```

It provides hover, pressed, focus, selected, and disabled visuals through
reactive inline styles. `variant="ghost"` is available for toolbar controls.
An application can preserve the interaction behavior while applying its own
theme with a state-aware style callback:

```tsx
<Button
  variant="ghost"
  style={(state) => ({
    "background-color": state.hovered ? palette().hover : "transparent",
    color: palette().text,
  })}
>
  Settings
</Button>
```

Use `unstyled` when layout and all visual properties come from application
classes or a design system. Hover, press, focus, and disabled state tracking
remain active.

Animations are explicit application state, not CSS state machines.
`createAnimationFrame` schedules work on the native display clock and binds
cleanup to its Solid owner. Update a signal, inline style, or paint-only
`transform` from the callback; return `false` when a finite animation is done
so the host can go back to sleep:

```tsx
const [progress, setProgress] = createSignal(0);
let started: number | undefined;

createAnimationFrame((timestamp) => {
  started ??= timestamp;
  const next = Math.min(1, (timestamp - started) / 180);
  setProgress(next);
  return next < 1;
});

<View
  style={{ opacity: progress() }}
  transform={[1, 0, 0, 1, 0, 8 * (1 - progress())]}
/>;
```

Wabou does not implement CSS transitions, keyframes, or `animate-*`
utilities. Timing, interpolation, interruption, and completion remain visible
in component code instead of being duplicated in the style engine and Rust
renderer.

Native text selection is observable without polling or a browser `Selection`
shim. `Text` and its ancestors can listen for a committed, bubbling event:

```tsx
<Text onTextSelectionChange={(event) => console.log(event.text, event.kind)}>
  Selectable text
</Text>
```

The payload is `{ text, kind }`, where `kind` is `simple`, `word`, `line`, or
`null` after the selection is cleared. Pointer dragging updates the renderer
locally; the bridge event is emitted once when the gesture commits.

`createTabs` owns a dynamic tab collection without imposing application UI or
terminal-session policy. It preserves the active tab across reorder operations
and chooses a deterministic adjacent tab when the active one closes:

```tsx
const tabs = createTabs({
  initialTabs: [{ id: "shell-1", title: "Shell" }],
  key: (tab) => tab.id,
});

tabs.add({ id: "shell-2", title: "Server" });
tabs.move("shell-2", 0);
tabs.close("shell-1");
```

Application shortcuts use exact modifier matching and automatically preempt a
focused native widget unless configured otherwise:

```tsx
const shortcuts = createShortcuts({
  "Primary+T": openTab,
  "Primary+W": closeActiveTab,
  "Control+Tab": {
    handler: selectNextTab,
    allowRepeat: true,
  },
});

<View {...shortcuts.bindings}>{children}</View>;
```

`Primary` accepts Control or Meta. Available modifier names are `Primary`,
`Control`/`Ctrl`, `Meta`/`Cmd`, `Alt`/`Option`, and `Shift`.

Floating overlays use `computeFloatingPosition`, backed by
`@floating-ui/core`. Applications provide Wabou layout and clipping rectangles;
the adapter does not emulate DOM elements or perform synchronous host queries.
The standard `offset`, `flip`, `shift`, `size`, `arrow`, and `autoPlacement`
middleware are exported alongside it.

`computeHostFloatingPosition(reference, floating, host, options)` is the normal
Wabou entry point. It batches both handles into one `host.layout.snapshot()` so
all middleware uses rectangles from the same completed layout revision.

`Popover` composes that positioner with a native root-layer `Portal`. Portals
default to the host's `floating` overlay plane, so they render above content
without a magic `z-index`; use `plane="modal"` for dialogs and blocking
backdrops. `z-index` only orders siblings within the same plane:

The renderer retains one synthetic root per overlay plane. Portal instances
share that root and release it when the last instance unmounts. The host paints
and hit-tests the same ordered planes: content, floating, modal, system, then
debug. System and debug remain native-only.

`ScrollArea` uses a node-local overlay attachment instead of a window portal.
Its native scrollbar is painted after the owner's descendants but before the
next sibling, and the same order is used for hit testing. Appearance can be
configured without moving pointer capture into JavaScript:

```tsx
<ScrollArea
  scrollbar={{
    visibility: "always",
    thickness: 12,
    minThumbLength: 36,
    thumbColor: 0x38bdf8cc,
    hoverColor: 0x38bdf8ff,
    activeColor: 0x0284c7ff,
  }}
>
  {children}
</ScrollArea>
```

```tsx
<Popover
  placement="bottom-start"
  trigger={(triggerProps) => <Button {...triggerProps}>Open</Button>}
  contentClass="w-64 p-3 rounded border"
>
  <Text>Popover content</Text>
</Popover>
```

It supports controlled or uncontrolled open state, outside-click and Escape
dismissal, focus restoration, collision-aware positioning, and repositioning
when its anchor, content, or viewport changes size.
