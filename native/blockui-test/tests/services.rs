use blockui::*;
use blockui_test::{Screen, Services};
use std::{cell::Cell, rc::Rc};
use ui_contract::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "ui_contract::serde")]
struct Increment;
impl ServiceRequest for Increment {
    type Response = i64;
}
const COUNT: State<i64> = State::<i64>::new("services.count", 0);
const INCREMENT: Action = Action::new("services.increment", |cx, _| {
    // A failed service must roll back writes performed earlier in this turn.
    COUNT.set(cx, -1)?;
    let result = cx.call(Increment)?;
    COUNT.set(cx, result)
});
struct App;
impl Actor for App {
    const STATE: &'static [StateSpec] = &[COUNT.spec()];
    const ACTIONS: &'static [Action] = &[INCREMENT];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default().children(vec![
            Text::new(COUNT).key("count").into_view(),
            Button::new("Increment", INCREMENT).into_view(),
        ]))
    }
}

#[test]
fn host_services_survive_replacement_and_errors_preserve_ui_state()
-> std::result::Result<(), String> {
    let calls = Rc::new(Cell::new(0));
    let fail = Rc::new(Cell::new(false));
    let mut services = Services::default();
    let (host_calls, host_fail) = (calls.clone(), fail.clone());
    services.handle::<Increment>(move |_| {
        host_calls.set(host_calls.get() + 1);
        if host_fail.get() {
            Err("business unavailable".into())
        } else {
            Ok(host_calls.get())
        }
    })?;
    let duplicate = services.handle::<Increment>(|_| Ok(999)).unwrap_err();
    assert!(
        duplicate.contains(&format!("{:#018x}", Increment::id())),
        "{duplicate}"
    );
    assert!(duplicate.contains("already registered"), "{duplicate}");
    let mut screen = Screen::with_services::<App>(services)?;
    assert_eq!(calls.get(), 0, "mount does not call business services");
    let count = screen.get_by_key("count")?;
    let increment = screen.get_by_role(blockui_test::Role::Button, "Increment")?;
    screen.click(increment)?;
    assert_eq!(screen.text(count)?, "1");
    screen.replace::<App>()?;
    assert_eq!(
        calls.get(),
        1,
        "replacement does not call business services"
    );
    screen.click(increment)?;
    assert_eq!(screen.text(count)?, "2");
    let before = screen.evaluations(count)?;
    fail.set(true);
    let error = screen.click(increment).unwrap_err();
    assert!(error.contains("business unavailable"), "{error}");
    assert_eq!(screen.text(count)?, "2");
    assert_eq!(screen.evaluations(count)?, before);
    assert_eq!(calls.get(), 3, "host calls themselves are not rolled back");
    fail.set(false);
    screen.click(increment)?;
    assert_eq!(screen.text(count)?, "4");
    let mut isolated = Screen::render::<App>()?;
    let other_button = isolated.get_by_role(blockui_test::Role::Button, "Increment")?;
    assert!(
        isolated.click(other_button).is_err(),
        "services are not shared implicitly"
    );
    assert_eq!(isolated.text(isolated.get_by_key("count")?)?, "0");
    assert_eq!(calls.get(), 4);
    Ok(())
}

const START: Action = Action::new("services.start", |cx, _| {
    Task::new("services.delayed").delay(cx, std::time::Duration::from_millis(10), INCREMENT, 0)
});
struct Delayed;
impl Actor for Delayed {
    const STATE: &'static [StateSpec] = App::STATE;
    const ACTIONS: &'static [Action] = &[START, INCREMENT];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default().children(vec![
            Text::new(COUNT).key("count").into_view(),
            Button::new("Schedule", START).into_view(),
        ]))
    }
}
#[test]
fn virtual_timer_delivers_service_call_after_replacement() -> std::result::Result<(), String> {
    let calls = Rc::new(Cell::new(0));
    let host_calls = calls.clone();
    let mut services = Services::default();
    services.handle::<Increment>(move |_| {
        host_calls.set(host_calls.get() + 1);
        Ok(11)
    })?;
    let mut screen = Screen::with_manual_time::<Delayed>()?;
    screen.set_services(services);
    screen.click(screen.get_by_role(blockui_test::Role::Button, "Schedule")?)?;
    assert_eq!(calls.get(), 0);
    assert_eq!(screen.advance_time(std::time::Duration::from_millis(9))?, 0);
    screen.replace::<Delayed>()?;
    assert_eq!(calls.get(), 0);
    let current_calls = Rc::new(Cell::new(0));
    let host_current_calls = current_calls.clone();
    let mut current_services = Services::default();
    current_services.handle::<Increment>(move |_| {
        host_current_calls.set(host_current_calls.get() + 1);
        Ok(17)
    })?;
    screen.set_services(current_services);
    assert_eq!(
        Rc::strong_count(&calls),
        1,
        "old handler capture is released"
    );
    assert_eq!(screen.advance_time(std::time::Duration::from_millis(1))?, 1);
    assert_eq!(
        calls.get(),
        0,
        "pending timer does not retain the old catalogue"
    );
    assert_eq!(current_calls.get(), 1);
    assert_eq!(screen.text(screen.get_by_key("count")?)?, "17");
    assert_eq!(screen.advance_time(std::time::Duration::from_secs(1))?, 0);
    assert_eq!(current_calls.get(), 1);
    Ok(())
}

#[test]
fn failed_virtual_timer_reports_time_rolls_back_and_can_be_rescheduled()
-> std::result::Result<(), String> {
    use std::time::Duration;
    let calls = Rc::new(Cell::new(0));
    let host_calls = calls.clone();
    let mut services = Services::default();
    services.handle::<Increment>(move |_| {
        host_calls.set(host_calls.get() + 1);
        Err("timer business unavailable".into())
    })?;
    let mut screen = Screen::with_manual_time::<Delayed>()?;
    screen.set_services(services);
    let schedule = screen.get_by_role(blockui_test::Role::Button, "Schedule")?;
    let count = screen.get_by_key("count")?;
    screen.click(schedule)?;
    let before = screen.debug_tree();
    let error = screen.advance_time(Duration::from_millis(10)).unwrap_err();
    assert!(
        error.starts_with("advance_time failed at 10ms after admitting 1 timer action(s):"),
        "{error}"
    );
    assert!(error.contains("timer business unavailable"), "{error}");
    assert_eq!(calls.get(), 1);
    assert_eq!(screen.text(count)?, "0", "pre-service write is rolled back");
    assert_eq!(screen.debug_tree(), before);
    let mut services = Services::default();
    services.handle::<Increment>(|_| Ok(23))?;
    screen.set_services(services);
    screen.click(schedule)?;
    assert_eq!(screen.advance_time(Duration::from_millis(9))?, 0);
    assert_eq!(screen.text(count)?, "0");
    assert_eq!(screen.advance_time(Duration::from_millis(1))?, 1);
    assert_eq!(screen.text(count)?, "23");
    assert_eq!(screen.advance_time(Duration::from_secs(1))?, 0);
    assert_eq!(calls.get(), 1, "old host handler is not reused");
    Ok(())
}
