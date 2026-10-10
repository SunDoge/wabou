use blockui::*;
use blockui_test::{Role, Screen};

type MemoResult<T> = std::result::Result<T, blockui::Error>;
type Flag = bool;

#[app(name = "alias.app")]
struct Aliases {
    #[state(key = "alias.count")]
    count: State<i64>,
}
#[app_impl]
impl Aliases {
    #[action]
    fn advance(&self) -> Result<()> {
        self.count.update(|value| Ok(value + 2))
    }
    #[action]
    fn reject(&self) -> Result<()> {
        self.count.set(1)
    }
    #[computed(name = "alias.total")]
    fn total(&self) -> MemoResult<i64> {
        let value = self.count.get();
        if value == 1 {
            return Err(Error::Application("aliased memo rejected".into()));
        }
        Ok(value * 2)
    }
    #[computed]
    fn even(&self) -> Flag {
        self.count.get() % 2 == 0
    }
    #[view]
    fn view(&self, ui: &Ui) -> View {
        #[node]
        {
            Column::default();
            #[node]
            Text::new(self.total_computed().text_with_prefix("Total: ")).key("alias.total.label");
            #[node]
            Text::new(self.even_computed()).key("alias.even.label");
            #[node]
            Button::new("Advance", self.advance_action());
            #[node]
            Button::new("Reject", self.reject_action());
        }
    }
}
#[test]
fn aliased_computed_errors_roll_back_and_recover_without_root_updates()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<Aliases>()?;
    let total = screen.get_by_text("Total: 0")?;
    let even = screen.get_by_key("alias.even.label")?;
    let before = screen.evaluations(total)?;
    let memo_count = screen.actor().computed_evaluations("alias.total");
    let error = screen
        .click(screen.get_by_role(Role::Button, "Reject")?)
        .unwrap_err();
    assert!(error.contains("aliased memo rejected"), "{error}");
    assert!(
        error.contains(&format!(
            "computed {:#018x} evaluation failed",
            Key::named("alias.total").0
        )),
        "{error}"
    );
    assert_eq!(screen.actor().integer("alias.count"), 0);
    assert_eq!(screen.text(total)?, "Total: 0");
    assert_eq!(screen.evaluations(total)?, before);
    assert_eq!(
        screen.actor().computed_evaluations("alias.total"),
        memo_count
    );
    let even_before = screen.evaluations(even)?;
    screen.click(screen.get_by_role(Role::Button, "Advance")?)?;
    assert_eq!(screen.get_by_text("Total: 4")?, total);
    assert_eq!(screen.evaluations(even)?, even_before);
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<Aliases>()?;
    screen.click(screen.get_by_role(Role::Button, "Advance")?)?;
    assert_eq!(screen.get_by_text("Total: 8")?, total);
    assert_eq!(screen.root_evaluations(), 2);
    Ok(())
}
