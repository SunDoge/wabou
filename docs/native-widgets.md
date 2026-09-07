# Native widgets

Use a native widget when a feature needs renderer-level painting, intrinsic
measurement, native text/IME behavior, or high-frequency local interaction.
Ordinary application structure remains TSX.

```text
Solid NativeWidget + generational NodeKey
                  |
       complete config/style snapshots
                  |
       wabou-shell-vello::Widget
          /        |         \
      measure     paint      input
                  |
          typed host event frame
                  |
             Solid handler
```

## Registration

Register a factory on the Rust host. One widget instance is created for one
live Solid node:

```rust
use wabou::{HostBuilder, VelloHybridWidget};

fn meter_widget() -> Box<dyn VelloHybridWidget> {
    Box::new(Meter::default())
}

HostBuilder::new()
    .widget("meter", meter_widget)
    .run()?;
```

On the Solid side, wrap the low-level primitive in a typed component so callers
do not repeat the tag or transport shape:

```tsx
import { NativeWidget } from "@wabou/ui";

export function Meter(props: { value: number; onChange(value: number): void }) {
  return (
    <NativeWidget
      tag="meter"
      role="slider"
      aria-label="Meter"
      config={{ value: props.value }}
      onChange={(event) => props.onChange(event.value)}
    />
  );
}
```

`config` is the complete authored snapshot. Prefer a concrete Rust DTO with
`#[serde(deny_unknown_fields)]` and decode it with
`wabou_shell_vello::decode_widget_config`. Lightweight string attributes remain
available for semantics and simple metadata. The tag does not imply HTML or CSS
behavior.

## Widget contract

Implement `wabou_shell_vello::Widget` for the renderer-owned object:

- `measure` returns intrinsic content size when the host cannot derive it;
- `paint` records content-local Vello scene operations;
- `handle_event` updates transient native state and returns precise
  `WidgetChanges` invalidation flags;
- `config_changed`, `attribute_changed`, and `style_changed` consume authored
  snapshots;
- `current_value`, selection, and accessibility methods expose native state to
  the standard event and semantic pipelines;
- `mounted`, `visibility_changed`, and `unmount` own native side-effect
  lifetimes.

The host owns the outer content box, transforms, clipping, focus routing, scene
composition, and generational routing. A widget paints in local coordinates and
must not create a parallel layout tree.

## Bidirectional ownership

Native controls use one controlled-component loop:

```text
Solid props/config snapshot
          | one mutation frame
          v
NodeKey + retained widget instance
          | typed native event
          v
HostEventFrame -> Solid handler -> next complete snapshot
```

The safety rules are:

1. Identity is the complete `(lo, hi)` `NodeKey`. Events from removed widgets
   cannot target a later node that reused the slot.
2. Props are snapshots, not patches. Widgets never reconstruct durable state
   from event history.
3. Events report facts, not replicated state. Solid decides the next durable
   value.
4. JavaScript is not called re-entrantly while layout, paint, or widget state is
   borrowed. Native callbacks enqueue a host event frame.
5. Solid owns durable application state. The widget owns transient focus,
   composition, selection, gestures, and renderer resources.
6. Async work retains validated identities and event sinks, not temporary
   paint, layout, or runtime contexts.
7. `WidgetChanges` must describe the smallest real invalidation: redraw,
   measure, layout, value, selection, semantics, or animation.

## Testing

Test pure widget state and configuration with the widget harness. Add component
tests for authored attributes and events, native behavior tests for focus/input,
and pixel captures only when paint is the claim. `@wabou/terminal` is the
largest end-to-end reference: Rio owns the terminal model, the Vello widget owns
measurement/paint/input, and Solid owns placement and configuration.
