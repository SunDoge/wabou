use blockui::*;
use blockui_test::{Role, Screen};

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
    // The tags opt this ordinary helper into UI construction.
    fn section(&self, title: &str) -> View {
        #[ui(Column::default())]
        {
            #[ui]
            Text::new(title);
            #[ui]
            Text::new(live!(self.count.get().to_string())).key("count");
        }
    }
    fn view(&self) -> View {
        #[ui(Column::default())]
        {
            #[ui]
            self.section("Helper title");
            #[ui]
            Button::new("Increment", self.increment_action());
        }
    }
}
#[test]
fn receiver_tag_helpers_need_no_second_marker_and_keep_local_updates()
-> std::result::Result<(), String> {
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
