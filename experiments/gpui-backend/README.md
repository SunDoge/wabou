# GPUI backend experiment

This is a standalone, non-workspace experiment. It does not replace or alter the
current Winit + AnyRender/Vello implementation.

The experiment uses **GPUI-CE** while importing it as `gpui`. GPUI-CE is aimed at
general-purpose applications, publishes versioned crates, tracks upstream GPUI,
and intentionally preserves the upstream API. That gives Wabou a practical
dependency today without coupling the adapter to the fork: switching to Zed's
upstream GPUI later should be a dependency/patch decision.

Run it with:

```bash
cargo run --manifest-path experiments/gpui-backend/Cargo.toml
```

## Migration boundary

The first production adapter should keep these Wabou layers:

- Solid 2 and the explicit JS protocol;
- retained `NodeKey` identity and resource handles;
- Style IR and its compile-time diagnostics;
- the existing component and layout test contracts.

GPUI should initially own:

- native windows and the application event loop;
- text shaping/rasterization;
- painting, clipping, shadows, images and SVG;
- pointer/keyboard/IME delivery.

Do not translate Wabou JSX into handwritten GPUI views. Project Wabou's retained
frame into one GPUI root element instead. This avoids creating a second component
model and lets the existing backend remain available until parity is demonstrated.

## Solid updates and GPUI rebuilds

Solid remains useful here because it already tells Wabou exactly which retained
properties and child relationships changed. The adapter should preserve that
information instead of calling a monolithic GPUI `Render` implementation for every
signal write:

1. apply Solid opcodes to the existing `NodeStore`;
2. accumulate layout, text, paint and interaction dirty flags until Solid flushes;
3. update only the affected GPUI projection state;
4. notify the GPUI window once for the completed frame.

GPUI carries element state across frames by stable element IDs and has internal
prepaint/paint subtree reuse. Map Wabou's generational `NodeKey` into those stable
IDs. Do not create one heavyweight GPUI `Entity` per text or style node: use
entities for coarse independently invalidated surfaces (window roots, overlays,
scroll regions and native widgets), and retain lightweight nodes inside the Wabou
projection.

This preserves Solid's fine-grained component execution while allowing GPUI to
reuse unchanged text layouts and paint ranges. A naive adapter that rebuilds one
large GPUI element tree after every opcode would discard the main architectural
advantage and is not an acceptable production path.

## Promotion gates

Do not make GPUI the default until the adapter proves all of the following on
Linux, macOS and Windows:

1. text, IME, clipboard, hit testing, scrolling and native widget replacement;
2. transparent and custom-titlebar windows, tray lifetime and multi-window flows;
3. current layout fixtures and behavior scenarios can target the GPUI backend;
4. representative Gallery and Pi Agent captures are at least as correct as the
   current backend;
5. idle memory, frame cost, binary size and build time are measured rather than
   assumed.
