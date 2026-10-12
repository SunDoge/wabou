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
