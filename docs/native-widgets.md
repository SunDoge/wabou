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

## Reloadable Rust widgets

The resident runtime can register a small custom widget implementation from a
watched Rust project using `rustWidget` in `wabou.config.ts`. The existing
`NativeWidget` TSX component selects its tag. See
[`examples/widget-reload`](../examples/widget-reload/README.md) for a runnable
counter and the supported callback/drawing protocol.

The optional `wabou-shell-vello/widget-hot-reload` adapter keeps each node's state,
config, focus and drawing cache in resident code. Its implementation travels
through the same stabby capability ABI, transactional loader, unique library
copies and Cargo watcher as host functions. A successful reload wakes the event
loop; the widget adopts a pinned generation at the host poll boundary and
invalidates measurement/layout/paint. Compilation or loading failure leaves the
previous generation running. Built-in widgets remain statically implemented.

This interface supports synchronous pointer, wheel, keyboard, text, IME and focus events,
intrinsic measurement and rectangle/text drawing. It does not upgrade arbitrary
Rust widget objects or expose renderer/GPU types across the ABI. State is JSON
data in a stable resident slot; callbacks encode snapshots, while reload itself
does not serialize or recreate state. Schema mismatch rejects replacement;
migration and resident resource operations
are future extensions. Callback errors are logged and retain the last valid
state/drawing. JSON callback overhead has not been benchmarked.

### Share one library with capabilities

Use `export_plugin!` from `wabou-capability-sdk` to register several `Module`
and `WidgetImplementation` types in one cdylib. In `wabou.config.ts`,
`plugins: [rustPlugin({ name, manifest, capabilities, widgets })]` selects their
contracts. The plugin is compiled/watched/loaded once, rather than once per
consumer. Legacy single-export registrations remain supported.

The resident loader queries the registry through the existing stabby ABI before
publishing a candidate. It checks every selected export's kind/name/version;
missing exports or incompatible schemas reject the complete replacement. Each
capability gets independent host state/tasks/resources, and each widget node
retains its own resident state. In-flight calls pin the previous library until
completion, while widgets adopt the new shared generation at their poll boundary.
Dynamic distribution packages the combined library once. Static combined-plugin
linking is not supported in this initial implementation.

### CLI-hosted plugin starter and static publishing

`wabou new my-app --native-plugin` creates a CLI-hosted frontend plus one
`native` crate exporting both capability and widget implementations. Use a CLI
matching the selected `--wabou-ref`. `bun run dev` uses Vite for UI changes and
rebuilds the native implementation for Rust changes. The runtime remains resident.

`wabou bundle . --out dist/app` packages a shared plugin library;
`wabou bundle . --static --out dist/static-app` links the registry into a generated
application executable. Static publishing requires project implementations whose
library target includes `rlib`. Already-built shared libraries cannot be converted
to static Rust dependencies. Capability, widget and plugin registrations use
separate namespaces in the linked host.

Reloadable widgets receive pointer down/up, motion/entry/exit/cancellation, other
buttons, logical-pixel wheel deltas, keyboard transitions, committed text, paste,
focus and IME lifecycle. Override `WidgetImplementation::accepts_text_input` to
enable native text input when focused. IME preedit offsets and delete lengths use
UTF-8 bytes; focus and state slots remain resident across implementation swaps.
