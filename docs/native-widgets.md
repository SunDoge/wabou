# Native GPUI widgets

Wabou projects ordinary Solid nodes into GPUI elements. Applications can mount
an application-owned GPUI element behind an explicit tag when a feature needs
native state or direct GPUI APIs:

```text
Solid node and generational NodeKey
                 |
        wabou-runtime registry
                 |
 NativeWidgetContext -> GPUI element/entity
```

This is the only production native-widget model. The retired
`wabou-legacy-widgets` crate contains Winit/Vello implementations solely for
migration comparison; it is not a selectable backend or a dependency for new
widgets.

## Stateless widgets

Register a stateless element with `HostBuilder::native_widget`. The factory is
called while GPUI materializes a frame and receives the exact authored
attributes plus the stable generational node key:

```rust
use wabou::{HostBuilder, gpui};
use wabou::gpui::{IntoElement as _, Styled as _};

# fn host() -> HostBuilder {
HostBuilder::new().native_widget("meter", |context, _window, _cx| {
    let config = context.config_json().unwrap_or(r#"{"value":0}"#);
    gpui::div()
        .size_full()
        .child(format!("Config: {config}"))
        .into_any_element()
})
# }
```

On the Solid side, mount the matching public `NativeWidget` primitive. Prefer
one typed application component around it so callers never repeat the tag or
transport shape:

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

`config` is the complete typed authored snapshot. Ordinary attributes remain
available for lightweight string metadata and semantics. Wabou does not infer
an HTML element or CSS behavior from the tag name.

## Stateful widgets

GPUI elements are ephemeral descriptions. Put stable mutable state in a GPUI
entity and return it with `HostBuilder::native_entity_widget`. On later frames,
recover the entity from `NativeWidgetContext::entity`:

```rust
use wabou::{HostBuilder, NativeWidgetMount, gpui};
use wabou::gpui::{AppContext as _, IntoElement as _};

struct MeterState {
    value: String,
}

# fn host() -> HostBuilder {
HostBuilder::new().native_entity_widget("meter", |context, _window, cx| {
    let entity = context.entity::<MeterState>().unwrap_or_else(|| {
        cx.new(|_| MeterState {
            value: context.attribute("value").unwrap_or("0").to_owned(),
        })
    });
    entity.update(cx, |state, _| {
        state.value = context.attribute("value").unwrap_or("0").to_owned();
    });
    let value = entity.read(cx).value.clone();
    NativeWidgetMount::entity(
        entity,
        gpui::div().child(format!("Value: {value}")).into_any_element(),
    )
})
# }
```

The entity lifetime follows the Solid node's generational identity. Reusing a
numeric slot after node removal cannot recover the stale entity because both
halves of the `NodeKey` participate in identity.

## Ownership

- Solid owns application state, composition, and the authored attributes.
- Wabou owns the protocol node identity and mounts the factory at that node.
- GPUI owns element layout, paint, input, focus, text, and entity updates.
- Shared resources that outlive one node belong in an application store keyed
  by typed Wabou resource handles or another explicit application identity.

Native callbacks use `context.events()` to send activation, numeric change,
text input, focus, selection, and submit events through the same typed event
path as built-in controls. Controlled components send their next complete
`config` snapshot back on the following Solid flush; widgets do not need a
second JSON capability or application-specific channel for this loop.

The widget receives `&mut Window` and `&mut App`, so it can use normal GPUI
facilities directly. It must not retain either reference. Use a retained entity,
GPUI task, or application-owned service for asynchronous work.

## Bidirectional contract

Native controls use one controlled-component loop. Do not add a second widget
message bus or call JavaScript from a GPUI render callback:

```text
Solid props/config snapshot
          │ one mutation frame
          ▼
generational NodeKey + retained GPUI Entity
          │ typed native event
          ▼
HostEventFrame ──► Solid handler ──► next complete props/config snapshot
```

The safety rules are:

1. **Identity is `(lo, hi)`.** Both halves of `NodeKey` cross the boundary.
   An event from a removed entity cannot target a later node that reused its
   numeric slot.
2. **Props are snapshots, not patches.** A widget synchronizes its GPUI entity
   from the complete current `config`; it never reconstructs application state
   from an event history.
3. **Events are facts, not state replication.** Use `activate`, `change_f64`,
   `input_text`, `focus`, `text_selection`, or `submit`. The guest handles them
   once from a versioned `HostEventFrame`, then Solid decides the next state.
4. **No re-entrant JavaScript.** GPUI callbacks enqueue/dispatch through the
   normal host-event boundary. JavaScript never runs while GPUI is rendering,
   laying out, or painting.
5. **One owner per kind of state.** Solid owns durable application state; the
   GPUI entity owns focus, composition, selection, pointer gestures and other
   transient native state. Controlled text reconciliation prevents either side
   from silently winning a same-frame echo.
6. **Async work retains identities, not contexts.** Retain an entity/weak
   entity and `NativeWidgetEventSink`; never retain `Window`, `App`, or
   `NativeWidgetContext`. Delivery validates the generational target again.

This division also applies to framework-owned GPUI-base containers. For
example, `NotificationRegion` keeps arbitrary toast content in TSX while a
native `ToastStack` owns measurement, overlap, hover expansion and stack
motion. JS does not duplicate GPUI's layout state.

## Testing

Test widget state as ordinary GPUI entities where possible. Add a Wabou
component test for authored attributes and semantics, then a focused GPUI
headless test when the contract depends on native layout, input, focus, or
paint. A legacy oracle test can compare migration behavior, but passing it does
not prove the GPUI implementation.

`@wabou/terminal` is the current end-to-end reference: terminal state lives in
a retained GPUI entity, while the Solid tag controls placement and authored
configuration.
