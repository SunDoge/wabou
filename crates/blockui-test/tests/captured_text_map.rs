use blockui::*;
use blockui_test::{Role, Screen};

#[app]
struct CapturedText {
    count: State<i64>,
    alternate_labels: State<bool>,
}
#[app_impl]
impl CapturedText {
    #[action]
    fn relabel(&self) -> Result<()> {
        self.alternate_labels.set(true)
    }
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
        let tag = if self.alternate_labels.get() {
            "Alternate "
        } else {
            ""
        };
        let bound_prefix = format!("{tag}Bound: ");
        let state_prefix = format!("{tag}State: ");
        let memo_prefix = format!("{tag}Memo: ");
        let even_prefix = format!("{tag}Even: ");
        let captured_prefix = format!("{tag}Captured: ");
        #[node]
        {
            Column::default();
            #[node]
            Text::new(self.count.try_map_text_with(move |value| {
                if value == 1 {
                    return Err(Error::InvalidView("bound text rejected"));
                }
                Ok(format!("{bound_prefix}{value}"))
            }))
            .key("fallible.bound");
            #[node]
            Text::new(State::from(&self.count).try_map_text_with(move |value| {
                if value == 3 {
                    return Err(Error::InvalidView("state text rejected"));
                }
                Ok(format!("{state_prefix}{value}"))
            }))
            .key("fallible.state");
            #[node]
            Text::new(self.memo_computed().try_map_text_with(move |value| {
                if value == 5 {
                    return Err(Error::InvalidView("computed text rejected"));
                }
                Ok(format!("{memo_prefix}{value}"))
            }))
            .key("fallible.memo");
            #[node]
            Text::new(
                self.even_computed()
                    .map_text_with(move |value| format!("{even_prefix}{value}")),
            )
            .key("fallible.even");
            #[node]
            Text::new(
                self.count
                    .map_text_with(move |value| format!("{captured_prefix}{value}")),
            )
            .key("captured.count");
            #[node]
            Button::new("Advance", self.advance_action());
            #[node]
            Button::new("Reject", self.reject_action());
            #[node]
            Button::new("Relabel", self.relabel_action());
        }
    }
}
#[test]
fn captured_parameters_preserve_rollback_memo_equality_and_replacement()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<CapturedText>()?;
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
        assert_eq!(
            screen.text(screen.get_by_key("captured.count")?)?,
            format!("Captured: {}", value + 2)
        );
        assert_eq!(screen.evaluations(even)?, even_before);
    }
    let memo_before_relabel = screen.actor().computed_evaluations("fallible.text.memo");
    screen.click(screen.get_by_role(Role::Button, "Relabel")?)?;
    assert_eq!(screen.text(labels[0])?, "Alternate Bound: 6");
    assert_eq!(screen.text(labels[1])?, "Alternate State: 6");
    assert_eq!(screen.text(labels[2])?, "Alternate Memo: 6");
    assert_eq!(screen.text(even)?, "Alternate Even: true");
    assert_eq!(
        screen.actor().computed_evaluations("fallible.text.memo"),
        memo_before_relabel
    );
    assert_eq!(screen.root_evaluations(), 2);
    let even_after_relabel = screen.evaluations(even)?;
    screen.click(screen.get_by_role(Role::Button, "Advance")?)?;
    assert_eq!(screen.text(labels[2])?, "Alternate Memo: 8");
    assert_eq!(screen.evaluations(even)?, even_after_relabel);
    assert_eq!(screen.root_evaluations(), 2);
    screen.replace::<CapturedText>()?;
    assert_eq!(screen.text(labels[2])?, "Alternate Memo: 8");
    screen.click(screen.get_by_role(Role::Button, "Advance")?)?;
    assert_eq!(screen.text(labels[2])?, "Alternate Memo: 10");
    assert_eq!(screen.root_evaluations(), 3);
    Ok(())
}
