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
    let value = context.attribute("value").unwrap_or("0");
    gpui::div()
        .size_full()
        .child(format!("Value: {value}"))
        .into_any_element()
})
# }
```

Attributes are explicit widget input. Wabou does not infer an HTML element or
CSS behavior from the tag name.

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

The widget receives `&mut Window` and `&mut App`, so it can use normal GPUI
facilities directly. It must not retain either reference. Use a retained entity,
GPUI task, or application-owned service for asynchronous work.

## Testing

Test widget state as ordinary GPUI entities where possible. Add a Wabou
component test for authored attributes and semantics, then a focused GPUI
headless test when the contract depends on native layout, input, focus, or
paint. A legacy oracle test can compare migration behavior, but passing it does
not prove the GPUI implementation.

`@wabou/terminal` is the current end-to-end reference: terminal state lives in
a retained GPUI entity, while the Solid tag controls placement and authored
configuration.
