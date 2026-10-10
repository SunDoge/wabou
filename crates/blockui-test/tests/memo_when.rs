use blockui::*;
use blockui_test::{Role, Screen};

#[app]
struct MemoWhen {
    level: State<i64>,
    count: State<i64>,
    draft: State<String>,
}
#[app_impl]
impl MemoWhen {
    #[computed]
    fn visible(&self) -> bool {
        self.level.get() > 0
    }
    #[action]
    fn raise(&self) -> Result<()> {
        self.level.update(|value| Ok(value + 1))
    }
    #[action]
    fn close(&self) -> Result<()> {
        self.level.set(0)
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
            Field::new("Draft", TextInput::new(&self.draft).key("draft"));
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
            Button::new("Raise", self.raise_action());
            #[ui]
            Button::new("Close", self.close_action());
            #[ui]
            Button::new("Outside increment", self.increment_action());
            #[ui]
            self.visible_computed()
                .when(live!(self.details()))
                .mount("details");
        }
    }
}

#[test]
fn equal_truthy_condition_preserves_mounted_branch() -> std::result::Result<(), String> {
    let mut screen = Screen::render::<MemoWhen>()?;
    let key = Key::named("details").0;
    let closed = screen.actor().projected_view().scope_dependencies[&key].evaluations;
    screen.click(screen.get_by_role(Role::Button, "Outside increment")?)?;
    assert_eq!(
        screen.actor().projected_view().scope_dependencies[&key].evaluations,
        closed
    );
    let raise = screen.get_by_role(Role::Button, "Raise")?;
    screen.click(raise)?;
    let scope = screen.get_by_key("details")?;
    let input = screen.within(scope)?.get_by_role(Role::Textbox, "Draft")?;
    let text = screen.within(scope)?.get_by_text("1")?;
    screen.input(input, "Retained")?;
    let mounted = screen.scope_evaluations(scope)?;
    let input_evaluations = screen.evaluations(input)?;
    let dependencies = screen.scope_dependencies(scope)?;
    screen.click(raise)?;
    screen.click(raise)?;
    assert_eq!(screen.scope_evaluations(scope)?, mounted);
    assert_eq!(screen.evaluations(input)?, input_evaluations);
    assert_eq!(screen.scope_dependencies(scope)?, dependencies);
    assert_eq!(screen.text(input)?, "Retained");
    screen.click(
        screen
            .within(scope)?
            .get_by_role(Role::Button, "Increment")?,
    )?;
    screen.expect_text(text, "2")?;
    assert_eq!(screen.scope_evaluations(scope)?, mounted);
    screen.click(screen.get_by_role(Role::Button, "Close")?)?;
    assert!(screen.input(input, "stale").is_err());
    screen.click(raise)?;
    screen.within(scope)?.get_by_text("2")?;
    assert_eq!(
        screen.text(screen.within(scope)?.get_by_role(Role::Textbox, "Draft")?)?,
        "Retained"
    );
    assert!(screen.root_dependencies().is_empty());
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<MemoWhen>()?;
    let after_reload = screen.scope_evaluations(scope)?;
    screen.click(raise)?;
    assert_eq!(screen.scope_evaluations(scope)?, after_reload);
    screen.within(scope)?.get_by_text("2")?;
    Ok(())
}
