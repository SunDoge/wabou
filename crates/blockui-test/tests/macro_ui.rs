use blockui::*;
use blockui_test::{Role, Screen};

// Standalone macros also work outside app/component impls.
fn caption(title: &str) -> View {
    ui!(Text::new(title))
}

#[app]
struct Helpers {
    count: State<i64>,
}
#[app_impl]
impl Helpers {
    #[action]
    fn increment(&self) -> Result<()> {
        self.count.update(|value| Ok(value + 1))
    }
    fn section(&self, title: &str) -> View {
        ui!(Column::default(), {
            ui!(caption(title));
            ui!(Text::new("Static caption"));
            ui!(Text::new(live!(self.count.get().to_string())).key("count"));
        })
    }
    fn view(&self) -> View {
        ui!(Column::default(), {
            ui!(self.section("Helper title"));
            ui!(Row::default(), {
                ui!(Button::new("Increment", self.increment_action()));
            });
        })
    }
}
#[test]
fn macro_markers_compose_helpers_and_keep_local_updates() -> std::result::Result<(), String> {
    let mut screen = Screen::render::<Helpers>()?;
    screen.get_by_text("Helper title")?;
    let count = screen.get_by_key("count")?;
    screen.expect_text(count, "0")?;
    let before = screen.evaluations(count)?;
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    screen.expect_text(count, "1")?;
    assert_eq!(screen.evaluations(count)?.text, before.text + 1);
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<Helpers>()?;
    screen.expect_text(count, "1")?;
    Ok(())
}
