#![cfg(feature = "testing")]
use blockui::*;
use std::time::Duration;

macro_rules! clock {
    ($name:ident, $step:expr $(, $field:ident: $ty:ty)?) => {
        #[actor(name = "test.clock")]
        struct $name {
            count: State<i64>,
            alarm: Task,
            $($field: $ty,)?
        }
        #[actor_impl]
        impl $name {
            #[action(name = "clock.start")]
            fn start(&self, cx: &Cx<'_>, periodic: bool) -> Result<()> {
                if periodic {
                    self.alarm
                        .interval(cx, Duration::from_secs(86_400), Self::fired_action(), 0)
                } else {
                    self.alarm
                        .delay(cx, Duration::from_secs(86_400), Self::fired_action(), 0)
                }
            }
            #[action(name = "clock.fired")]
            fn fired(&self, cx: &Cx<'_>) -> Result<()> {
                self.count.update(cx, |count| Ok(count + $step))
            }
            #[action(name = "clock.sample")]
            fn sample(&self, cx: &Cx<'_>) -> Result<()> {
                self.count.set(cx, i64::try_from(cx.monotonic_ms()?).map_err(|_| Error::Overflow)?)
            }
            #[action(name = "clock.cancel")]
            fn cancel(&self, cx: &Cx<'_>) -> Result<()> {
                self.alarm.cancel(cx)
            }
            fn view(&self, _: &Cx<'_>) -> Result<View> {
                Ok(Column::default().children(vec![Text::new(self.count).into_view()]))
            }
        }
    };
}
clock!(Clock, 1);
clock!(NextClock, 2);
clock!(AddedClock, 3, added: State<bool>);

#[test]
fn injected_completion_executes_replaced_code_once_without_waiting() {
    let mut actor = blockui_runtime::Actor::for_test::<Clock>().unwrap();
    actor.tell(Clock::start_action(), false).unwrap();
    actor.tick().unwrap();
    let timer = abi::key("test.clock.alarm");
    let slot = actor.slot_address("test.clock.count");
    actor.complete_timer_for_test(timer).unwrap();
    assert!(actor.complete_timer_for_test(timer).is_err());
    assert_eq!(actor.integer("test.clock.count"), 0);
    actor.replace_for_test::<NextClock>().unwrap();
    actor.tick().unwrap();
    assert_eq!(actor.integer("test.clock.count"), 2);
    assert_eq!(actor.slot_address("test.clock.count"), slot);
    assert!(actor.complete_timer_for_test(timer).is_err());
}

#[test]
fn injected_interval_rearms_after_delivery_and_cancel_rejects_injection() {
    let mut actor = blockui_runtime::Actor::for_test::<Clock>().unwrap();
    actor.tell(Clock::start_action(), true).unwrap();
    actor.tick().unwrap();
    let timer = abi::key("test.clock.alarm");
    for count in 1..=3 {
        actor.complete_timer_for_test(timer).unwrap();
        assert!(actor.complete_timer_for_test(timer).is_err());
        actor.tick().unwrap();
        assert_eq!(actor.integer("test.clock.count"), count);
    }
    actor.tell(Clock::cancel_action(), 0).unwrap();
    actor.tick().unwrap();
    assert!(actor.complete_timer_for_test(timer).is_err());
    actor.tick().unwrap();
    assert_eq!(actor.integer("test.clock.count"), 3);
}

#[test]
fn cancelling_injected_completion_before_tick_prevents_callback() {
    let mut actor = blockui_runtime::Actor::for_test::<Clock>().unwrap();
    actor.tell(Clock::start_action(), false).unwrap();
    actor.tick().unwrap();
    let timer = abi::key("test.clock.alarm");
    actor.complete_timer_for_test(timer).unwrap();
    actor
        .dispatch(blockui_runtime::Message {
            action: abi::key("clock.cancel"),
            value: 0,
        })
        .unwrap();
    actor.replace_for_test::<NextClock>().unwrap();
    actor.tick().unwrap();
    assert_eq!(actor.integer("test.clock.count"), 0);
    assert!(actor.complete_timer_for_test(timer).is_err());
}

#[test]
fn application_time_is_actor_local_and_survives_schema_addition() {
    let mut first = blockui_runtime::Actor::for_test::<Clock>().unwrap();
    let mut second = blockui_runtime::Actor::for_test::<Clock>().unwrap();
    first.set_monotonic_ms_for_test(1000).unwrap();
    second.set_monotonic_ms_for_test(7000).unwrap();
    let sample = |actor: &mut blockui_runtime::Actor| {
        actor
            .dispatch(blockui_runtime::Message {
                action: abi::key("clock.sample"),
                value: 0,
            })
            .unwrap();
    };
    sample(&mut first);
    sample(&mut second);
    assert_eq!(first.integer("test.clock.count"), 1000);
    assert_eq!(second.integer("test.clock.count"), 7000);
    let slot = first.slot_address("test.clock.count");
    first.replace_for_test::<AddedClock>().unwrap();
    assert_eq!(first.slot_address("test.clock.count"), slot);
    assert_ne!(first.slot_address("test.clock.added"), 0);
    sample(&mut first);
    assert_eq!(first.integer("test.clock.count"), 1000);
    first.set_monotonic_ms_for_test(1250).unwrap();
    assert!(first.set_monotonic_ms_for_test(1249).is_err());
    sample(&mut first);
    sample(&mut second);
    assert_eq!(first.integer("test.clock.count"), 1250);
    assert_eq!(second.integer("test.clock.count"), 7000);
}

#[test]
fn manual_scheduler_retains_pending_work_across_replacement() {
    let mut actor = blockui_runtime::Actor::for_test::<Clock>().unwrap();
    actor.use_manual_timers_for_test().unwrap();
    actor.tell(Clock::actions().start, true).unwrap();
    actor.tick().unwrap();
    assert!(actor.use_manual_timers_for_test().is_err());
    actor.replace_for_test::<NextClock>().unwrap();
    for expected in [2, 4, 6] {
        actor
            .complete_timer_for_test(abi::key("test.clock.alarm"))
            .unwrap();
        actor.tick().unwrap();
        assert_eq!(actor.integer("test.clock.count"), expected);
    }
    actor.tell(NextClock::actions().cancel, 0).unwrap();
    actor.tick().unwrap();
    assert!(
        actor
            .complete_timer_for_test(abi::key("test.clock.alarm"))
            .is_err()
    );
}

#[test]
fn virtual_timer_deadlines_survive_replacement_and_coalesce_intervals() {
    let mut actor = blockui_runtime::Actor::for_test::<Clock>().unwrap();
    assert!(actor.advance_timer_time_for_test(Duration::ZERO).is_err());
    actor.use_manual_timers_for_test().unwrap();
    actor.tell(Clock::actions().start, true).unwrap();
    actor.tick().unwrap();
    assert_eq!(
        actor
            .advance_timer_time_for_test(Duration::from_secs(86_399))
            .unwrap(),
        0
    );
    actor.tick().unwrap();
    assert_eq!(actor.integer("test.clock.count"), 0);
    assert_eq!(
        actor
            .advance_timer_time_for_test(Duration::from_secs(1))
            .unwrap(),
        1
    );
    assert_eq!(
        actor.advance_timer_time_for_test(Duration::ZERO).unwrap(),
        0
    );
    actor.replace_for_test::<NextClock>().unwrap();
    actor.tick().unwrap();
    assert_eq!(actor.integer("test.clock.count"), 2);
    assert_eq!(
        actor
            .advance_timer_time_for_test(Duration::from_secs(864_000))
            .unwrap(),
        1
    );
    actor.tick().unwrap();
    assert_eq!(actor.integer("test.clock.count"), 4);
    assert_eq!(
        actor.advance_timer_time_for_test(Duration::ZERO).unwrap(),
        0
    );
    actor.tell(NextClock::actions().cancel, 0).unwrap();
    actor.tick().unwrap();
    assert_eq!(
        actor
            .advance_timer_time_for_test(Duration::from_secs(86_400))
            .unwrap(),
        0
    );
}

struct RejectedClock;
impl Actor for RejectedClock {
    const STATE: &'static [StateSpec] = <AddedClock as Actor>::STATE;
    const ACTIONS: &'static [Action] = <Clock as Actor>::ACTIONS;
    fn migrate(_: &Cx<'_>, _: u64) -> Result<()> {
        Err(Error::Application("timer upgrade rejected".into()))
    }
    fn view(cx: &Cx<'_>) -> Result<View> {
        <Clock as Actor>::view(cx)
    }
}

#[test]
fn virtual_deadline_survives_rejected_and_accepted_schema_changes() {
    let mut actor = blockui_runtime::Actor::for_test::<Clock>().unwrap();
    actor.use_manual_timers_for_test().unwrap();
    actor.tell(Clock::actions().start, true).unwrap();
    actor.tick().unwrap();
    let state = actor.state_address();
    let slot = actor.slot_address("test.clock.count");
    assert_eq!(
        actor
            .advance_timer_time_for_test(Duration::from_secs(43_200))
            .unwrap(),
        0
    );
    assert!(
        actor
            .replace_for_test::<RejectedClock>()
            .unwrap_err()
            .contains("timer upgrade rejected")
    );
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("test.clock.count"), slot);
    assert_eq!(actor.slot_address("test.clock.added"), 0);
    assert_eq!(
        actor
            .advance_timer_time_for_test(Duration::from_secs(43_199))
            .unwrap(),
        0
    );
    assert_eq!(
        actor
            .advance_timer_time_for_test(Duration::from_secs(1))
            .unwrap(),
        1
    );
    actor.tick().unwrap();
    assert_eq!(actor.integer("test.clock.count"), 1);
    actor.replace_for_test::<AddedClock>().unwrap();
    assert_ne!(actor.slot_address("test.clock.added"), 0);
    assert_eq!(actor.slot_address("test.clock.count"), slot);
    assert_eq!(
        actor
            .advance_timer_time_for_test(Duration::from_secs(86_399))
            .unwrap(),
        0
    );
    assert_eq!(
        actor
            .advance_timer_time_for_test(Duration::from_secs(1))
            .unwrap(),
        1
    );
    actor.tick().unwrap();
    assert_eq!(actor.integer("test.clock.count"), 4);
    assert_eq!(actor.slot_address("test.clock.count"), slot);
}
