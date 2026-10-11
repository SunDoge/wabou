use blockui::*;
use blockui_test::{Role, Screen};

// A user type, with no component macro or knowledge in the UI parser.
struct Panel(String);
impl IntoView for Panel {
    fn into_view(self) -> View {
        Column {
            key: "panel.root".into(),
            ..default()
        }
        .children(vec![Text::new(self.0).key("panel.heading").into_view()])
    }
}
fn panel(title: &str) -> Panel {
    Panel(title.to_owned())
}

#[view(block)]
fn footer() -> View {
    #[ui(Row::default())]
    {
        #[ui]
        Text::new("Reusable footer");
    }
}

#[app]
struct Composition {
    child: TrailingToggle,
}
#[app_impl]
impl Composition {
    fn view(&self) -> View {
        let compact = true;
        let style = ViewStyle {
            max_width: 320.0,
            radius: 8.0,
            ..default()
        };
        #[ui((if compact { panel("Custom panel") } else { panel("Wide panel") }).style(style))]
        {
            for (index, label) in ["Alpha", "Beta"].into_iter().enumerate() {
                #[ui]
                Text::new(label).key(format!("item.{index}"));
            }
            if compact {
                #[ui]
                Text::new("Ordinary Rust branch");
            }
            #[ui(self.child.render())]
            {
                #[ui]
                Text::new("Appended to component root");
            }
            #[ui]
            footer();
        }
    }
}

#[test]
fn user_parents_helpers_loops_and_component_children_compose() -> std::result::Result<(), String> {
    let mut screen = Screen::render::<Composition>()?;
    let root = screen.get_by_key("panel.root")?;
    assert_eq!(screen.style(root)?.max_width, 320.0);
    assert_eq!(screen.style(root)?.radius, 8.0);
    for label in [
        "Custom panel",
        "Alpha",
        "Beta",
        "Ordinary Rust branch",
        "Reusable footer",
    ] {
        screen.within(root)?.get_by_text(label)?;
    }
    let appended = screen.get_by_text("Appended to component root")?;
    let count = screen.get_by_text("Off")?;
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    screen.expect_text(count, "On")?;
    screen.expect_text(appended, "Appended to component root")?;
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<Composition>()?;
    assert_eq!(screen.get_by_key("panel.root")?, root);
    assert_eq!(screen.get_by_text("Appended to component root")?, appended);
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    screen.expect_text(count, "Off")?;
    Ok(())
}

#[app]
struct NestedHelper {}
#[app_impl]
impl NestedHelper {
    fn view(&self) -> View {
        #[view(block)]
        fn inner() -> View {
            #[ui(Column::default())]
            {
                #[ui]
                Text::new("Owned by inner function");
            }
        }
        inner()
    }
}
#[test]
fn nested_ui_markers_do_not_change_outer_view_semantics() -> std::result::Result<(), String> {
    Screen::render::<NestedHelper>()?.get_by_text("Owned by inner function")?;
    Ok(())
}

#[component]
struct TrailingToggle {
    count: State<bool>,
}
#[component_impl]
impl TrailingToggle {
    #[computed]
    fn double(&self) -> bool {
        !self.count.get()
    }
    #[action]
    fn increment(&self) -> Result<()> {
        self.count.update(|value| Ok(!value))
    }
    fn view(&self) -> View {
        #[ui(Column { gap: 8.px(), ..default() })]
        {
            #[ui]
            Text::new("An explicit parent");
            #[ui(Row { gap: 4.px(), ..default() })]
            {
                #[ui]
                Text::new(live!(if self.count.get() { "On" } else { "Off" }));
                #[ui]
                Text::new(live!(if self.double() { "inactive" } else { "active" }));
            }
            #[ui]
            Button::new("Increment", self.increment_action());
        }
    }
}
#[app]
struct TrailingPair {
    left: TrailingToggle,
    right: TrailingToggle,
}
#[app_impl]
impl TrailingPair {
    fn view(&self) -> View {
        Row::default().children(vec![self.left.render(), self.right.render()])
    }
}
#[test]
fn ui_blocks_keep_reactivity_and_component_isolation() -> std::result::Result<(), String> {
    let mut screen = Screen::render::<TrailingPair>()?;
    let counts = screen.query_all_by_text("Off");
    let doubles = screen.query_all_by_text("inactive");
    let buttons = screen.query_all_by_role(Role::Button, "Increment");
    assert_eq!(counts.len(), 2);
    assert_eq!(doubles.len(), 2);
    assert_eq!(buttons.len(), 2);
    assert_ne!(counts[0], counts[1]);
    assert_ne!(doubles[0], doubles[1]);
    assert_ne!(
        screen.dependencies(counts[0])?.text,
        screen.dependencies(counts[1])?.text
    );
    assert_ne!(
        screen.dependencies(doubles[0])?.text,
        screen.dependencies(doubles[1])?.text
    );
    let untouched_count = screen.evaluations(counts[1])?;
    let untouched_memo = screen.evaluations(doubles[1])?;
    screen.click(buttons[0])?;
    assert_eq!(screen.text(counts[0])?, "On");
    assert_eq!(screen.text(doubles[0])?, "active");
    assert_eq!(screen.evaluations(counts[1])?, untouched_count);
    assert_eq!(screen.evaluations(doubles[1])?, untouched_memo);
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<TrailingPair>()?;
    assert_eq!(screen.get_by_text("On")?, counts[0]);
    assert_eq!(screen.get_by_text("active")?, doubles[0]);
    let left_count = screen.evaluations(counts[0])?;
    let left_memo = screen.evaluations(doubles[0])?;
    screen.click(buttons[1])?;
    assert_eq!(screen.text(counts[1])?, "On");
    assert_eq!(screen.text(doubles[1])?, "active");
    assert_eq!(screen.evaluations(counts[0])?, left_count);
    assert_eq!(screen.evaluations(doubles[0])?, left_memo);
    assert_eq!(screen.root_evaluations(), 2);
    Ok(())
}
