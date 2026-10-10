use blockui::*;
use blockui_test::{Role, Screen};

fn caption(value: i64) -> String {
    format!("Value {value}")
}
fn checked(value: i64) -> Result<String> {
    if value == 2 {
        Err(Error::InvalidView("rejected value"))
    } else {
        Ok(caption(value))
    }
}
#[app]
struct Mapped {
    count: State<i64>,
}
#[app_impl]
impl Mapped {
    #[action]
    fn increment(&self) -> Result<()> {
        self.count.update(|value| Ok(value + 1))
    }
    fn view(&self) -> View {
        #[ui(Column::default())]
        {
            #[ui]
            Text::new(live!(self.count.get()).map(caption)).key("mapped");
            #[ui]
            Text::new(live!(self.count.get()).try_map(checked)).key("checked");
            #[ui]
            Button::new("Increment", self.increment_action());
        }
    }
}
#[test]
fn mapped_observers_are_lazy_and_errors_roll_back_the_turn() -> std::result::Result<(), String> {
    let mut screen = Screen::render::<Mapped>()?;
    let mapped = screen.get_by_key("mapped")?;
    let checked = screen.get_by_key("checked")?;
    let button = screen.get_by_role(Role::Button, "Increment")?;
    screen.expect_text(mapped, "Value 0")?;
    assert_eq!(screen.root_evaluations(), 1);
    screen.click(button)?;
    screen.expect_text(mapped, "Value 1")?;
    screen.expect_text(checked, "Value 1")?;
    let evaluations = screen.evaluations(mapped)?;
    assert!(screen.click(button).is_err());
    screen.expect_text(mapped, "Value 1")?;
    screen.expect_text(checked, "Value 1")?;
    assert_eq!(screen.evaluations(mapped)?, evaluations);
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<Mapped>()?;
    screen.expect_text(mapped, "Value 1")?;
    assert!(screen.click(button).is_err());
    Ok(())
}
