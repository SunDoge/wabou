# Tag UI in ordinary Rust

Use `#[app]` for actor state and `#[app_impl]` for actions and UI methods.
Inside a marked `#[ui]` block, the first expression creates the parent and
subsequent marked expressions add children. Constructors, style builders and
helper calls remain normal Rust expressions; formatting uses `cargo fmt`.

| Changing value | Expression | Work triggered |
| --- | --- | --- |
| Text | `Text::new(live!(expression))` | One text observer |
| Style | `.style(live!(expression))` | One style observer |
| Conditional children | `live!(expression).mount("details")` | One structural subtree |

State reads are tracked dynamically inside observers. Read state inside
`live!`; reading first and capturing the result gives a snapshot. Keep static
properties ordinary values. Helpers marked by `#[ui]` return views and compose
with user-defined components implementing `IntoView`. Repeated items need
business keys to preserve identity when reordered.

This complete example keeps counter text and conditional content independent.
Its headless test checks local updates, branch removal/remounting and state
retention after compiled-in replacement. It does not establish real dylib
replacement by itself.

```rust
use wabou_native::*;
#[app(name = "guide.counter")]
struct Counter {
    count: State<i64>,
    open: State<bool>,
}
#[app_impl]
impl Counter {
    #[action]
    fn increment(&self) -> Result<()> {
        self.count
            .update(|n| n.checked_add(1).ok_or(Error::Overflow))
    }
    #[action]
    fn toggle(&self) -> Result<()> {
        self.open.update(|open| Ok(!open))
    }
    fn details(&self) -> View {
        #[ui]
        Text::new("Details");
    }

    fn view(&self) -> View {
        #[ui]
        {
            Column::default();
            #[ui]
            Text::new(live!(format!("Count: {}", self.count.get())));
            #[ui]
            Button::new("Increment", self.increment_action());
            #[ui]
            Button::new("Toggle details", self.toggle_action());
            #[ui]
            live!(if self.open.get() {
                self.details()
            } else {
                View::empty()
            })
            .mount("details");
        }
    }
}
export_actor!(Counter);
#[test]
fn increment_keeps_the_parent_and_details_cached() -> std::result::Result<(), String> {
    use wabou_native::testing::{Role, Screen};
    let mut screen = Screen::render::<Counter>()?;
    screen.click(screen.get_by_role(Role::Button, "Toggle details")?)?;
    let details = screen.get_by_text("Details")?;
    let scope = screen.scope_of(details)?;
    let before = screen.scope_evaluations(scope)?;
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    screen.get_by_text("Count: 1")?;
    assert_eq!(screen.root_evaluations(), 1, "{}", screen.debug_tree());
    assert_eq!(
        screen.scope_evaluations(scope)?,
        before,
        "{}",
        screen.debug_tree()
    );
    screen.click(screen.get_by_role(Role::Button, "Toggle details")?)?;
    assert!(screen.get_by_text("Details").is_err());
    screen.click(screen.get_by_role(Role::Button, "Toggle details")?)?;
    screen.get_by_text("Details")?;
    screen.replace::<Counter>()?;
    screen.get_by_text("Count: 1")?;
    screen.get_by_text("Details")?;
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    screen.get_by_text("Count: 2")?;
    assert_eq!(screen.root_evaluations(), 2, "{}", screen.debug_tree());
    Ok(())
}
```

Run the exact example without a renderer:

```sh
cargo test -p wabou-native --features testing --test tag_ui_guide
```

For an application, enable `testing` only through dev-dependencies. Native
hot reload uses `wabou dev APP` with the Rust frontend metadata marker, or
`wabou native dev APP`. Host/core changes require a restart. A failed plugin
build keeps the accepted actor implementation running. Release applications
can use `wabou build APP --static --release` with the generated linked entry.
See [async results](ASYNC.md) for loading, cancellation and completion generations.
