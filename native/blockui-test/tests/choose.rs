use blockui::*;
use blockui_test::{Role, Screen};

#[app]
struct Choose {
    level: State<i64>,
    count: State<i64>,
    draft: State<String>,
    placeholder: State<i64>,
}
#[app_impl]
impl Choose {
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
    #[action]
    fn lower(&self) -> Result<()> {
        self.level.update(|value| Ok(value - 1))
    }
    #[action]
    fn advance_placeholder(&self) -> Result<()> {
        self.placeholder.update(|value| Ok(value + 1))
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
            Button::new("Lower", self.lower_action());
            #[ui]
            Button::new("Placeholder", self.advance_placeholder_action());
            #[ui]
            Button::new("Close", self.close_action());
            #[ui]
            Button::new("Outside increment", self.increment_action());
            #[ui]
            self.visible_computed()
                .choose(
                    live!(self.details()),
                    live!(Text::new(format!("Empty {}", self.placeholder.get()))),
                )
                .mount("details");
        }
    }
}

#[test]
fn fallback_is_lazy_and_old_branch_dependencies_are_pruned() -> std::result::Result<(), String> {
    let mut screen = Screen::render::<Choose>()?;
    let scope = screen.get_by_key("details")?;
    screen.within(scope)?.get_by_text("Empty 0")?;
    let initial = screen.scope_evaluations(scope)?;
    let false_dependencies = screen.scope_dependencies(scope)?;
    screen.click(screen.get_by_role(Role::Button, "Lower")?)?;
    assert_eq!(screen.scope_evaluations(scope)?, initial);
    screen.click(screen.get_by_role(Role::Button, "Outside increment")?)?;
    assert_eq!(screen.scope_evaluations(scope)?, initial);
    screen.click(screen.get_by_role(Role::Button, "Placeholder")?)?;
    screen.within(scope)?.get_by_text("Empty 1")?;
    assert_eq!(screen.scope_evaluations(scope)?, initial + 1);
    let raise = screen.get_by_role(Role::Button, "Raise")?;
    screen.click(raise)?; // -1 -> 0 remains false
    assert_eq!(screen.scope_evaluations(scope)?, initial + 1);
    screen.click(raise)?; // 0 -> 1 mounts content
    let input = screen.within(scope)?.get_by_role(Role::Textbox, "Draft")?;
    screen.input(input, "Retained")?;
    let opened = screen.scope_evaluations(scope)?;
    assert_ne!(screen.scope_dependencies(scope)?, false_dependencies);
    screen.click(screen.get_by_role(Role::Button, "Placeholder")?)?;
    screen.click(raise)?;
    assert_eq!(screen.scope_evaluations(scope)?, opened);
    assert_eq!(screen.text(input)?, "Retained");
    screen.click(screen.get_by_role(Role::Button, "Close")?)?;
    assert!(screen.input(input, "stale").is_err());
    screen.within(scope)?.get_by_text("Empty 2")?;
    assert_eq!(screen.scope_dependencies(scope)?, false_dependencies);
    let closed = screen.scope_evaluations(scope)?;
    screen.click(screen.get_by_role(Role::Button, "Outside increment")?)?;
    assert_eq!(screen.scope_evaluations(scope)?, closed);
    screen.replace::<Choose>()?;
    screen.within(scope)?.get_by_text("Empty 2")?;
    screen.click(raise)?;
    screen.within(scope)?.get_by_text("2")?;
    assert_eq!(
        screen.text(screen.within(scope)?.get_by_role(Role::Textbox, "Draft")?)?,
        "Retained"
    );
    assert_eq!(screen.root_evaluations(), 2);
    Ok(())
}

const SELECT: State<bool> = State::<bool>::new("choose.select", true);
const REJECT: State<bool> = State::<bool>::new("choose.reject", true);
const CONDITION: Computed<bool> = Computed::<bool>::new("choose.condition", |cx| SELECT.get(cx));
struct FallibleChoose;
impl Actor for FallibleChoose {
    const STATE: &'static [StateSpec] = &[SELECT.spec(), REJECT.spec()];
    const COMPUTEDS: &'static [ComputedSpec] = &[CONDITION.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default().children(Vec::from([
            Button::label("Switch")
                .key("choose.switch")
                .on_click(|cx| SELECT.set(cx, !SELECT.get(cx)?)),
            Button::label("Allow fallback")
                .key("choose.allow")
                .on_click(|cx| REJECT.set(cx, false)),
            CONDITION
                .choose(
                    Live::captured(|_| Ok(Text::new("Primary"))),
                    Live::captured(|cx| {
                        if REJECT.get(cx)? {
                            return Err(Error::Application("Rejected fallback".into()));
                        }
                        Ok(Text::new("Recovered fallback"))
                    }),
                )
                .mount("fallible.details"),
        ])))
    }
}
#[test]
fn rejected_fallback_rolls_back_condition_state_and_dependencies() -> std::result::Result<(), String>
{
    let mut screen = Screen::render::<FallibleChoose>()?;
    let scope = screen.get_by_key("fallible.details")?;
    let initial = screen.scope_evaluations(scope)?;
    let dependencies = screen.scope_dependencies(scope)?;
    let switch = screen.get_by_role(Role::Button, "Switch")?;
    assert!(
        screen
            .click(switch)
            .unwrap_err()
            .contains("Rejected fallback")
    );
    assert!(screen.actor().boolean("choose.select"));
    screen.within(scope)?.get_by_text("Primary")?;
    assert_eq!(screen.scope_evaluations(scope)?, initial);
    assert_eq!(screen.scope_dependencies(scope)?, dependencies);
    screen.click(screen.get_by_role(Role::Button, "Allow fallback")?)?;
    assert_eq!(screen.scope_evaluations(scope)?, initial);
    screen.click(switch)?;
    screen.within(scope)?.get_by_text("Recovered fallback")?;
    assert!(!screen.actor().boolean("choose.select"));
    assert_eq!(screen.root_evaluations(), 1);
    Ok(())
}
