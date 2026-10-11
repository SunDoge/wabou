use blockui::*;
use blockui_test::{Role, Screen};

#[app]
struct Branch {
    left_selected: State<bool>,
    left: State<i64>,
    right: State<i64>,
}
#[app_impl]
impl Branch {
    #[action]
    fn toggle(&self) -> Result<()> {
        self.left_selected.update(|value| Ok(!value))
    }
    #[action]
    fn increment_left(&self) -> Result<()> {
        self.left.update(|value| Ok(value + 1))
    }
    #[action]
    fn increment_right(&self) -> Result<()> {
        self.right.update(|value| Ok(value + 1))
    }
    // Ordinary Rust construction: live_text does not require #[view].
    fn view(&self) -> View {
        Column::default().children(vec![
            Text::new(self.live_text(|this| {
                if this.left_selected.get() {
                    this.left.get().to_string()
                } else {
                    this.right.get().to_string()
                }
            }))
            .key("branch.text")
            .into_view(),
            Button::new("Toggle", self.toggle_action()).into_view(),
            Button::new("Left", self.increment_left_action()).into_view(),
            Button::new("Right", self.increment_right_action()).into_view(),
        ])
    }
}

#[test]
fn ordinary_function_tracks_only_the_active_branch() -> std::result::Result<(), String> {
    let mut screen = Screen::render::<Branch>()?;
    let text = screen.get_by_key("branch.text")?;
    let right_dependencies = screen.dependencies(text)?.text;
    let initial = screen.evaluations(text)?;
    screen.click(screen.get_by_role(Role::Button, "Left")?)?;
    assert_eq!(screen.evaluations(text)?, initial);
    screen.click(screen.get_by_role(Role::Button, "Right")?)?;
    screen.expect_text(text, "1")?;
    let before_toggle = screen.evaluations(text)?;
    screen.click(screen.get_by_role(Role::Button, "Toggle")?)?;
    screen.expect_text(text, "1")?;
    assert_eq!(screen.evaluations(text)?.text, before_toggle.text + 1);
    assert_ne!(screen.dependencies(text)?.text, right_dependencies);
    let left_only = screen.evaluations(text)?;
    screen.click(screen.get_by_role(Role::Button, "Right")?)?;
    assert_eq!(screen.evaluations(text)?, left_only);
    screen.click(screen.get_by_role(Role::Button, "Left")?)?;
    screen.expect_text(text, "2")?;
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<Branch>()?;
    screen.expect_text(text, "2")?;
    let replaced = screen.evaluations(text)?;
    screen.click(screen.get_by_role(Role::Button, "Right")?)?;
    assert_eq!(screen.evaluations(text)?, replaced);
    assert_eq!(screen.root_evaluations(), 2);
    Ok(())
}

#[component]
struct LiveToggle {
    count: State<bool>,
}
#[component_impl]
impl LiveToggle {
    #[computed]
    fn double(&self) -> bool {
        !self.count.get()
    }
    #[action]
    fn increment(&self) -> Result<()> {
        self.count.update(|value| Ok(!value))
    }
    #[view]
    fn view(&self, ui: &Ui) -> View {
        #[node]
        {
            Column::default();
            #[node]
            Text::new(self.live_text(|this| if this.count.get() { "On" } else { "Off" }))
                .key("live.state");
            #[node]
            Text::new(live!(if self.double() { "inactive" } else { "active" })).key("live.memo");
            #[node]
            Button::new("Increment", self.increment_action());
        }
    }
}
#[app]
struct LivePair {
    left: LiveToggle,
    right: LiveToggle,
}
#[app_impl]
impl LivePair {
    fn view(&self) -> View {
        Row::default().children(vec![self.left.render(), self.right.render()])
    }
}
#[test]
fn live_text_keeps_component_dependencies_and_replacement_isolated()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<LivePair>()?;
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
    screen.replace::<LivePair>()?;
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
