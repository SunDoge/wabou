use blockui::*;
use blockui_test::{Role, Screen};

#[component]
struct TreeToggle {
    count: State<bool>,
}
#[component_impl]
impl TreeToggle {
    #[computed]
    fn double(&self) -> bool {
        !self.count.get()
    }
    #[action]
    fn increment(&self) -> Result<()> {
        self.count.update(|value| Ok(!value))
    }
    #[view(tree)]
    fn view(&self) -> View {
        Column {
            gap: 8.px(),
            children: [
                Text::new("A tree built with Rust"),
                Row {
                    gap: 4.px(),
                    children: [
                        Text::new(live!(if self.count.get() { "On" } else { "Off" }))
                            .key("live.state"),
                        Text::new(live!(if self.double() { "inactive" } else { "active" }))
                            .key("live.memo"),
                    ],
                    ..default()
                },
                Button::new("Increment", self.increment_action()),
            ],
            ..default()
        }
    }
}
#[app]
struct TreePair {
    left: TreeToggle,
    right: TreeToggle,
}
#[app_impl]
impl TreePair {
    fn view(&self) -> View {
        Row::default().children(vec![self.left.render(), self.right.render()])
    }
}
#[test]
fn tree_children_keep_reactivity_and_component_isolation() -> std::result::Result<(), String> {
    let mut screen = Screen::render::<TreePair>()?;
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
    screen.replace::<TreePair>()?;
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
