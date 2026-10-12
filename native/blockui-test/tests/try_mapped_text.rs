use blockui::*;
use blockui_test::{Role, Screen};

#[app]
struct FallibleText {
    count: State<i64>,
}
#[app_impl]
impl FallibleText {
    #[action]
    fn advance(&self) -> Result<()> {
        self.count.update(|value| Ok(value + 2))
    }
    #[action]
    fn reject(&self) -> Result<()> {
        self.count.update(|value| Ok(value + 1))
    }
    #[computed(name = "fallible.text.memo")]
    fn memo(&self) -> i64 {
        self.count.get()
    }
    #[computed]
    fn even(&self) -> bool {
        self.count.get() % 2 == 0
    }
    #[view]
    fn view(&self, ui: &Ui) -> View {
        #[node]
        {
            Column::default();
            #[node]
            Text::new(self.count.try_map_text(|value| {
                if value == 1 {
                    return Err(Error::InvalidView("bound text rejected"));
                }
                Ok(format!("Bound: {value}"))
            }))
            .key("fallible.bound");
            #[node]
            Text::new(State::from(&self.count).try_map_text(|value| {
                if value == 3 {
                    return Err(Error::InvalidView("state text rejected"));
                }
                Ok(format!("State: {value}"))
            }))
            .key("fallible.state");
            #[node]
            Text::new(self.memo_computed().try_map_text(|value| {
                if value == 5 {
                    return Err(Error::InvalidView("computed text rejected"));
                }
                Ok(format!("Memo: {value}"))
            }))
            .key("fallible.memo");
            #[node]
            Text::new(
                self.even_computed()
                    .try_map_text(|value| Ok(format!("Even: {value}"))),
            )
            .key("fallible.even");
            #[node]
            Button::new("Advance", self.advance_action());
            #[node]
            Button::new("Reject", self.reject_action());
        }
    }
}
#[test]
fn all_fallible_text_sources_roll_back_and_recover_without_root_updates()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<FallibleText>()?;
    let labels = ["fallible.bound", "fallible.state", "fallible.memo"]
        .map(|key| screen.get_by_key(key).unwrap());
    let even = screen.get_by_key("fallible.even")?;
    let even_before = screen.evaluations(even)?;
    for (value, message) in [
        (0, "bound text rejected"),
        (2, "state text rejected"),
        (4, "computed text rejected"),
    ] {
        let before = labels.map(|label| {
            (
                screen.text(label).unwrap().to_owned(),
                screen.evaluations(label).unwrap(),
            )
        });
        let memo_count = screen.actor().computed_evaluations("fallible.text.memo");
        let error = screen
            .click(screen.get_by_role(Role::Button, "Reject")?)
            .unwrap_err();
        assert!(error.contains(message), "{error}");
        let failed_key =
            ["fallible.bound", "fallible.state", "fallible.memo"][(value / 2) as usize];
        assert!(
            error.contains(&format!(
                "text binding {:#018x} evaluation failed",
                Key::named(failed_key).0
            )),
            "{error}"
        );
        assert_eq!(
            screen.actor().computed("fallible.text.memo"),
            Some(&blockui_runtime::Value::Integer(value))
        );
        assert_eq!(
            screen.actor().computed_evaluations("fallible.text.memo"),
            memo_count
        );
        for (label, (text, counts)) in labels.into_iter().zip(before) {
            assert_eq!(screen.text(label)?, text);
            assert_eq!(screen.evaluations(label)?, counts);
        }
        assert_eq!(screen.root_evaluations(), 1);
        screen.click(screen.get_by_role(Role::Button, "Advance")?)?;
        assert_eq!(screen.text(labels[0])?, format!("Bound: {}", value + 2));
        assert_eq!(screen.text(labels[1])?, format!("State: {}", value + 2));
        assert_eq!(screen.text(labels[2])?, format!("Memo: {}", value + 2));
        assert_eq!(screen.evaluations(even)?, even_before);
    }
    screen.replace::<FallibleText>()?;
    assert_eq!(screen.text(labels[2])?, "Memo: 6");
    screen.click(screen.get_by_role(Role::Button, "Advance")?)?;
    assert_eq!(screen.text(labels[2])?, "Memo: 8");
    assert_eq!(screen.root_evaluations(), 2);
    Ok(())
}
