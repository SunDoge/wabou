# Native widgets

Wabou keeps the native widget contract separate from its built-in
implementations:

```text
wabou-shell       public Widget trait and host-facing data types
      ↑
wabou-widgets     canvas, image, text input, textarea, password input
      ↑
wabou-quick       registry and JavaScript protocol adapter
```

`wabou-widgets` deliberately does not depend on `wabou-quick`. It is the
reference external implementation of the same trait available to application
authors. QuickJS only stores `WidgetFactory` values keyed by element tag; a
built-in and an application widget enter the applier through the same path.

Applications normally depend on `wabou-quick`, which re-exports the public SDK
through `widget_api`:

```rust
use wabou_quick::widget_api::{
    HostBuilder, TextContext, UiEvent, Widget, WidgetEventResult, vello,
};

struct Meter;

impl Widget for Meter {
    fn paint(
        &mut self,
        width: f32,
        height: f32,
        text: &mut TextContext,
    ) -> vello::Scene {
        let _ = (width, height, text);
        vello::Scene::new()
    }

    fn handle_event(&mut self, _event: &UiEvent) -> WidgetEventResult {
        WidgetEventResult::HANDLED
    }
}

HostBuilder::new()
    .widget("meter", || Box::new(Meter))
    .run()?;
```

The trait also exposes measurement, resolved content styles, focus and IME
state, window-to-local transforms, animation deadlines, asynchronous wakeups,
native host actions, and events sent back to the owning Solid element. Standard
layout, clipping, transforms, pointer hit-testing and semantic attributes stay
owned by the framework around the widget's content scene.

If this contract eventually needs its own crate, its intended name is
`wabou-widget-trait`; `wabou-widgets` remains the plural implementation crate.
