use blockui::*;
use blockui_test::{Role, Screen};

#[derive(Record, Clone, Debug, PartialEq)]
struct Snapshot {
    value: i64,
}
#[app]
struct RecordMemo {
    count: State<i64>,
    unavailable: State<bool>,
}
#[app_impl]
impl RecordMemo {
    #[action]
    fn pair(&self) -> Result<()> {
        self.count.update(|value| Ok(value + 2))
    }
    #[action]
    fn single(&self) -> Result<()> {
        self.count.update(|value| Ok(value + 1))
    }
    #[action]
    fn fail(&self) -> Result<()> {
        self.unavailable.set(true)
    }
    #[computed(record)]
    fn parity(&self) -> Result<Snapshot> {
        if self.unavailable.get() {
            return Err(Error::Application("source unavailable".into()));
        }
        Ok(Snapshot {
            value: self.count.get() % 2,
        })
    }
    #[computed(record)]
    fn forwarded(&self) -> Snapshot {
        self.parity().expect("parity read failed")
    }
    fn view(&self) -> View {
        #[ui(Column::default())]
        {
            #[ui]
            Text::new(live!(self.forwarded().value.to_string())).key("value");
            #[ui]
            Text::new(live!(self.count.get().to_string())).key("count");
            #[ui]
            Button::new("Pair", self.pair_action());
            #[ui]
            Button::new("Single", self.single_action());
            #[ui]
            Button::new("Fail", self.fail_action());
        }
    }
}
#[test]
fn canonical_record_memos_suppress_equal_outputs_and_rollback_failure()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<RecordMemo>()?;
    let value = screen.get_by_key("value")?;
    let before = screen.evaluations(value)?;
    screen.click(screen.get_by_role(Role::Button, "Pair")?)?;
    screen.expect_text(screen.get_by_key("count")?, "2")?;
    assert_eq!(screen.evaluations(value)?, before);
    screen.click(screen.get_by_role(Role::Button, "Single")?)?;
    screen.expect_text(value, "1")?;
    let after = screen.evaluations(value)?;
    assert_eq!(after.text, before.text + 1);
    assert!(
        screen
            .click(screen.get_by_role(Role::Button, "Fail")?)
            .is_err()
    );
    screen.expect_text(value, "1")?;
    assert_eq!(screen.evaluations(value)?, after);
    screen.click(screen.get_by_role(Role::Button, "Pair")?)?;
    screen.expect_text(screen.get_by_key("count")?, "5")?;
    assert_eq!(screen.evaluations(value)?, after);
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<RecordMemo>()?;
    screen.expect_text(value, "1")?;
    Ok(())
}
