# Native widgets

Wabou keeps the native widget contract separate from its built-in
implementations:

```text
wabou-shell       public Widget trait and host-facing data types
      ↑
    wabou-widgets     canvas, image, inputs, textarea, code editor
      ↑
wabou-runtime       registry and JavaScript protocol adapter
      ↑
wabou             stable application-facing facade
```

`wabou-widgets` deliberately does not depend on `wabou-runtime`. It is the
reference external implementation of the same trait available to application
authors. QuickJS only stores `WidgetFactory` values keyed by element tag; a
built-in and an application widget enter the applier through the same path.

Each mounted widget is owned by exactly one retained-tree node. The runtime
stores widget instances by the already-generational Taffy `NodeId`; dropping
the node calls `unmount`, drains permitted host actions, and removes every
widget projection. An additional SlotMap would duplicate identity without
improving stale-key safety. Independently shared native resources used by a
widget, such as decoded images or fonts, belong in their own typed SlotMaps and
are referenced by opaque resource handles.

Applications normally depend on `wabou`, which re-exports the public SDK
through `widget_api`:

```rust
use wabou::widget_api::{
    HostBuilder, PaintContext, UiEvent, Widget, WidgetChanges,
    WidgetEventResult,
};

struct Meter;

impl Widget for Meter {
    fn paint(&mut self, cx: &mut PaintContext<'_>) {
        // Paint in content-local logical pixels. `scene_mut()` exposes Vello
        // directly while Wabou's higher-level painting API is still small.
        let _ = (cx.size(), cx.device_scale(), cx.scene_mut());
    }

    fn handle_event(&mut self, _event: &UiEvent) -> WidgetEventResult {
        WidgetEventResult::HANDLED
    }

    fn attribute_changed(&mut self, name: &str, _value: &str) -> WidgetChanges {
        if name == "value" {
            WidgetChanges::REDRAW
        } else {
            WidgetChanges::empty()
        }
    }
}

# fn run() -> wabou::Result<()> {
HostBuilder::new()
    .widget("meter", || Box::new(Meter))
    .run()
# }
```

## Ownership and coordinates

The host owns the CSS box model, layout, transforms, rounded clipping, focus
routing, hit testing, and scene composition. A widget measures and paints only
its content box. Pointer and wheel events are localized before
`Widget::handle_event`; `(0, 0)` is the content-box origin even when ancestors
are translated, scaled, or scrolled. `Widget::layout_changed` supplies both
directions of the affine transform for APIs such as IME placement that need
window coordinates.

Painting uses logical pixels. `PaintContext::device_scale` is metadata for
scale-sensitive resources; the host applies the window scale during final
composition. Widgets must not apply that scale to the whole scene themselves.

## State and invalidation

Attribute strings are suitable for small scalar native-widget properties. They
do not imply HTML element compatibility. Structured configuration uses the
single `widgetConfig` object and
`decode_widget_config`; a derived Serde type with `deny_unknown_fields` keeps
the boundary typed without teaching the style system widget-specific nested
properties.

Every mutating callback returns `WidgetChanges`. These flags are part of the
widget contract, not hints:

- `REDRAW` rebuilds only the widget scene fragment;
- `MEASURE` reruns intrinsic measurement and therefore layout;
- `LAYOUT` recomputes geometry without claiming intrinsic metrics changed;
- `VALUE` synchronizes `current_value()` and dispatches an input event to JS;
- `SEMANTICS` republishes accessibility state;
- `CONSUME_KEY_TEXT` prevents a handled key from also arriving as text input.

Lifecycle callbacks run in host order: mount, initial attributes/config and
style synchronization, measurement, layout notification, visibility, then
paint. `unmount` runs while host-action routing still exists; use it to enqueue
cleanup that needs the host, and use `Drop` for resource-only cleanup.

The trait additionally exposes focus and IME state, animation deadlines,
asynchronous wakeups, native host actions, and semantic events sent back to the
owning Solid element. `WidgetHarness` exercises these contracts without a
window; protocol tests are still needed when JS routing itself matters.

If this contract eventually needs its own crate, its intended name is
`wabou-widget-trait`; `wabou-widgets` remains the plural implementation crate.

`@wabou/ui` also exposes an experimental `ConfigEditor`. It is backed
by the native `code-editor` widget and editor-core, supports Unicode-aware
editing, selection and pointer dragging, undo/redo, IME, soft wrapping,
scrolling, and JSON highlighting. Its public API intentionally exposes only
Wabou concepts (`value`, `readOnly`, `disabled`, `language`, and input events),
so the Rust editor engine can evolve without leaking into application code.
