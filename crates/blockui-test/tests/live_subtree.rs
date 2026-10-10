use blockui::*;
use blockui_test::{Role, Screen};

#[app]
struct Dynamic {
    open: State<bool>,
    count: State<i64>,
    draft: State<String>,
}
#[app_impl]
impl Dynamic {
    #[action]
    fn toggle(&self) -> Result<()> {
        self.open.update(|value| Ok(!value))
    }
    #[action]
    fn increment(&self) -> Result<()> {
        self.count.update(|value| Ok(value + 1))
    }
    #[view(block)]
    fn details(&self) -> View {
        #[ui(Column::default())]
        {
            #[ui]
            Text::new(live!(self.count.get().to_string())).key("count");
            #[ui]
            TextInput::new(&self.draft).key("draft");
            #[ui]
            Button::new("Increment", self.increment_action());
        }
    }
    fn view(&self) -> View {
        #[ui(Column::default())]
        {
            #[ui]
            Text::new("Outside").key("outside");
            #[ui]
            Button::new("Toggle", self.toggle_action());
            #[ui]
            Button::new("Outside increment", self.increment_action());
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

#[test]
fn live_subtree_mounts_locally_and_retains_actor_state() -> std::result::Result<(), String> {
    let mut screen = Screen::render::<Dynamic>()?;
    let outside = screen.get_by_key("outside")?;
    let outside_evaluations = screen.evaluations(outside)?;
    let toggle = screen.get_by_role(Role::Button, "Toggle")?;
    let scope_key = Key::named("details").0;
    assert!(screen.root_dependencies().is_empty());
    assert!(screen.get_by_role(Role::Textbox, "").is_err());
    let closed = screen.actor().projected_view().scope_dependencies[&scope_key].evaluations;
    screen.click(screen.get_by_role(Role::Button, "Outside increment")?)?;
    assert_eq!(
        screen.actor().projected_view().scope_dependencies[&scope_key].evaluations,
        closed
    );
    screen.click(toggle)?;
    let scope = screen.get_by_key("details")?;
    let opened = screen.actor().projected_view().scope_dependencies[&scope_key].evaluations;
    let content = screen.within(scope)?;
    let text = content.get_by_text("1")?;
    let input = content.get_by_role(Role::Textbox, "")?;
    let increment = content.get_by_role(Role::Button, "Increment")?;
    screen.input(input, "Retained draft")?;
    screen.click(increment)?;
    screen.expect_text(text, "2")?;
    assert_eq!(
        screen.actor().projected_view().scope_dependencies[&scope_key].evaluations,
        opened
    );
    screen.click(toggle)?;
    assert!(screen.click(increment).is_err());
    assert!(screen.input(input, "stale").is_err());
    let closed = screen.actor().projected_view().scope_dependencies[&scope_key].evaluations;
    screen.click(screen.get_by_role(Role::Button, "Outside increment")?)?;
    assert_eq!(
        screen.actor().projected_view().scope_dependencies[&scope_key].evaluations,
        closed
    );
    screen.click(toggle)?;
    let content = screen.within(scope)?;
    content.get_by_text("3")?;
    assert_eq!(
        screen.text(content.get_by_role(Role::Textbox, "Retained draft")?)?,
        "Retained draft"
    );
    assert_eq!(screen.evaluations(outside)?, outside_evaluations);
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<Dynamic>()?;
    screen.within(scope)?.get_by_text("3")?;
    screen.click(
        screen
            .within(scope)?
            .get_by_role(Role::Button, "Increment")?,
    )?;
    screen.within(scope)?.get_by_text("4")?;
    assert_eq!(screen.root_evaluations(), 2);
    Ok(())
}
