#![cfg(feature = "testing")]
use blockui::*;
use blockui_runtime::{Actor as HostActor, Message};
use std::time::Duration;
const TOASTS: Toasts = Toasts::new("test.toasts").limit(2);
const OTHER: Toasts = Toasts::new("other.toasts");
const ACTED: State<i64> = State::<i64>::new("toast.acted", 0);
const EXPIRE: Action = Action::new("toast.expire", |cx, revision| {
    TOASTS.expire(cx, revision, EXPIRE)
});
const OTHER_EXPIRE: Action = Action::new("other.expire", |cx, revision| {
    OTHER.expire(cx, revision, OTHER_EXPIRE)
});
const SHOW: Action = Action::new("toast.show", |cx, value| {
    let toast = match value {
        1 => {
            Toast::new("Saved 🦀\n草稿", "Changes: \0 محفوظة").duration(Duration::from_millis(100))
        }
        2 => Toast::new("Second", "Another message").duration(Duration::from_millis(100)),
        3 => Toast::new("Removed", "Undo the operation").action("Undo", 42),
        4 => Toast::new("Failed", "Retry without losing your work").variant(ToastVariant::Error),
        5 => Toast::new("", "Invalid"),
        6 => Toast::new("Invalid", "Zero delay").duration(Duration::ZERO),
        7 => Toast::new("Invalid", "Too long").duration(Duration::from_secs(86401)),
        8 => Toast::new("Invalid", "No action label").action(" ", 0),
        9 => Toast::new("Recovery", "Failing action").action("Fail action", 99),
        _ => Toast::new("Third", "Persistent").persistent(),
    };
    TOASTS.show(cx, toast, EXPIRE)?;
    Ok(())
});
const CLEAR: Action = Action::new("toast.clear", |cx, _| TOASTS.clear(cx));
const SHOW_OTHER: Action = Action::new("toast.other", |cx, _| {
    OTHER.show(
        cx,
        Toast::new("Independent", "Another queue").persistent(),
        OTHER_EXPIRE,
    )?;
    Ok(())
});
const UPDATE: Action = Action::new("toast.update", |cx, mode| {
    let toast = match mode {
        1 => Toast::new("Updated", "Same notification").duration(Duration::from_millis(200)),
        2 => Toast::new("Updated", "Persistent now").action("Undo updated", 42),
        3 => Toast::new(" ", "Invalid replacement"),
        _ => Toast::new("Missing", "Must not resurrect"),
    };
    ACTED.set(cx, i64::from(TOASTS.update(cx, 1, toast, EXPIRE)?))
});
fn action(cx: &Cx<'_>, value: i64) -> Result<()> {
    ACTED.set(cx, value)?;
    if value == 99 {
        return Err(Error::InvalidView("action rejected"));
    }
    Ok(())
}
struct Fixture;
impl Actor for Fixture {
    const STATE: &'static [StateSpec] = &[
        TOASTS.state()[0],
        TOASTS.state()[1],
        TOASTS.state()[2],
        TOASTS.state()[3],
        OTHER.state()[0],
        OTHER.state()[1],
        OTHER.state()[2],
        OTHER.state()[3],
        ACTED.spec(),
    ];
    const ACTIONS: &'static [Action] = &[EXPIRE, OTHER_EXPIRE, SHOW, SHOW_OTHER, CLEAR, UPDATE];
    fn view(cx: &Cx<'_>) -> Result<View> {
        Ok(Column::default().children(vec![
            TOASTS.view(cx, EXPIRE, action)?,
            OTHER.view(cx, OTHER_EXPIRE, action)?,
        ]))
    }
}
fn actor() -> std::result::Result<HostActor, String> {
    let mut actor = HostActor::for_test::<Fixture>()?;
    actor.use_manual_timers_for_test()?;
    actor.set_monotonic_ms_for_test(0)?;
    Ok(actor)
}
fn dispatch(actor: &mut HostActor, action: Action, value: i64) -> std::result::Result<(), String> {
    actor.dispatch(Message {
        action: action.encode(value).0,
        value,
    })
}
fn has(actor: &HostActor, title: &str) -> bool {
    actor
        .projected_view()
        .bindings
        .values()
        .any(|b| b.text == title)
}
fn advance(actor: &mut HostActor, now: u64, delta: u64) -> std::result::Result<usize, String> {
    actor.set_monotonic_ms_for_test(now)?;
    let n = actor.advance_timer_time_for_test(Duration::from_millis(delta))?;
    actor.tick()?;
    Ok(n)
}
#[test]
fn deadlines_expire_individually_and_reschedule_one_host_timer() -> std::result::Result<(), String>
{
    let mut a = actor()?;
    dispatch(&mut a, SHOW, 1)?;
    assert!(has(&a, "Saved 🦀\n草稿"));
    assert_eq!(advance(&mut a, 50, 50)?, 0);
    dispatch(&mut a, SHOW, 2)?;
    // A delivered callback from the earlier schedule must not close a newer item.
    dispatch(&mut a, EXPIRE, 1)?;
    assert!(has(&a, "Saved 🦀\n草稿") && has(&a, "Second"));
    assert_eq!(advance(&mut a, 99, 49)?, 0);
    assert_eq!(advance(&mut a, 100, 1)?, 1);
    assert!(!has(&a, "Saved 🦀\n草稿") && has(&a, "Second"));
    assert_eq!(advance(&mut a, 150, 50)?, 1);
    assert!(!has(&a, "Second"));
    assert_eq!(advance(&mut a, 1000, 850)?, 0);
    Ok(())
}
#[test]
fn bounded_queues_keep_independent_identity_and_persistent_recovery()
-> std::result::Result<(), String> {
    let mut a = actor()?;
    dispatch(&mut a, SHOW, 1)?;
    dispatch(&mut a, SHOW, 2)?;
    dispatch(&mut a, SHOW, 3)?;
    assert!(!has(&a, "Saved 🦀\n草稿"));
    dispatch(&mut a, SHOW_OTHER, 0)?;
    assert_eq!(advance(&mut a, 10000, 10000)?, 1);
    assert!(has(&a, "Removed") && has(&a, "Independent"));
    let undo = Key::named("test.toasts")
        .scoped(3)
        .scoped(abi::key("action"))
        .0;
    a.dispatch_widget(undo)?;
    assert_eq!(a.integer("toast.acted"), 42);
    assert!(!has(&a, "Removed") && has(&a, "Independent"));
    dispatch(&mut a, SHOW, 4)?;
    assert_eq!(advance(&mut a, 20000, 10000)?, 0);
    assert!(has(&a, "Failed"));
    let error = a
        .projected_view()
        .bindings
        .values()
        .find(|b| b.node.semantics.role == abi::semantics::ALERT)
        .unwrap();
    assert_ne!(error.node.semantics.labelled_by, 0);
    dispatch(&mut a, CLEAR, 0)?;
    assert!(!has(&a, "Failed") && has(&a, "Independent"));
    dispatch(&mut a, EXPIRE, 4)?;
    assert!(has(&a, "Independent"));
    Ok(())
}
#[test]
fn invalid_notifications_rollback_data_and_deadline() -> std::result::Result<(), String> {
    let mut a = actor()?;
    dispatch(&mut a, SHOW, 1)?;
    for mode in 5..=8 {
        assert!(dispatch(&mut a, SHOW, mode).is_err());
        assert!(has(&a, "Saved 🦀\n草稿"));
    }
    assert_eq!(advance(&mut a, 100, 100)?, 1);
    assert!(!has(&a, "Saved 🦀\n草稿"));
    Ok(())
}
#[test]
fn manual_dismissal_cancels_old_deadlines_without_reusing_ids() -> std::result::Result<(), String> {
    let mut a = actor()?;
    dispatch(&mut a, SHOW, 1)?;
    a.dispatch_widget(
        Key::named("test.toasts")
            .scoped(1)
            .scoped(abi::key("dismiss"))
            .0,
    )?;
    dispatch(&mut a, SHOW, 2)?;
    assert!(
        !a.projected_view()
            .bindings
            .contains_key(&Key::named("test.toasts").scoped(1).0)
    );
    assert!(
        a.projected_view()
            .bindings
            .contains_key(&Key::named("test.toasts").scoped(2).0)
    );
    dispatch(&mut a, EXPIRE, 1)?;
    assert!(has(&a, "Second"));
    assert_eq!(advance(&mut a, 100, 100)?, 1);
    assert!(!has(&a, "Second"));
    Ok(())
}

#[test]
fn rejected_recovery_action_keeps_notification_and_rolls_back_effects()
-> std::result::Result<(), String> {
    let mut a = actor()?;
    dispatch(&mut a, SHOW, 9)?;
    let target = Key::named("test.toasts")
        .scoped(1)
        .scoped(abi::key("action"))
        .0;
    assert!(a.dispatch_widget(target).is_err());
    assert_eq!(a.integer("toast.acted"), 0);
    assert!(has(&a, "Recovery"));
    assert_eq!(advance(&mut a, 10000, 10000)?, 0);
    assert!(has(&a, "Recovery"));
    Ok(())
}

#[test]
fn updates_keep_identity_and_order_and_restart_only_the_selected_deadline()
-> std::result::Result<(), String> {
    let mut a = actor()?;
    dispatch(&mut a, SHOW, 1)?;
    dispatch(&mut a, SHOW, 2)?;
    let key = Key::named("test.toasts").scoped(1).0;
    let old_order = a.projected_view().order.clone();
    assert_eq!(advance(&mut a, 50, 50)?, 0);
    dispatch(&mut a, UPDATE, 1)?;
    assert_eq!(a.integer("toast.acted"), 1);
    assert!(a.projected_view().bindings.contains_key(&key));
    assert_eq!(a.projected_view().order, old_order);
    assert!(!has(&a, "Saved 🦀\n草稿") && has(&a, "Updated"));
    dispatch(&mut a, EXPIRE, 2)?; // admitted before update
    assert_eq!(advance(&mut a, 100, 50)?, 1);
    assert!(!has(&a, "Second") && has(&a, "Updated"));
    assert_eq!(advance(&mut a, 249, 149)?, 0);
    assert_eq!(advance(&mut a, 250, 1)?, 1);
    assert!(!has(&a, "Updated"));
    dispatch(&mut a, UPDATE, 0)?;
    assert_eq!(a.integer("toast.acted"), 0);
    assert!(!has(&a, "Missing"));
    Ok(())
}

#[test]
fn updates_can_become_persistent_and_replace_current_recovery_action()
-> std::result::Result<(), String> {
    let mut a = actor()?;
    dispatch(&mut a, SHOW, 1)?;
    dispatch(&mut a, UPDATE, 2)?;
    assert_eq!(advance(&mut a, 10000, 10000)?, 0);
    assert!(has(&a, "Updated"));
    a.dispatch_widget(
        Key::named("test.toasts")
            .scoped(1)
            .scoped(abi::key("action"))
            .0,
    )?;
    assert_eq!(a.integer("toast.acted"), 42);
    assert!(!has(&a, "Updated"));
    Ok(())
}

#[test]
fn invalid_updates_rollback_and_expired_ids_cannot_be_resurrected()
-> std::result::Result<(), String> {
    let mut a = actor()?;
    dispatch(&mut a, SHOW, 1)?;
    assert!(dispatch(&mut a, UPDATE, 3).is_err());
    assert!(has(&a, "Saved 🦀\n草稿"));
    assert_eq!(a.integer("toast.acted"), 0);
    // The application clock may pass a deadline before the host message is admitted.
    a.set_monotonic_ms_for_test(100)?;
    dispatch(&mut a, UPDATE, 0)?;
    assert_eq!(a.integer("toast.acted"), 0);
    assert!(!has(&a, "Missing"));
    assert_eq!(advance(&mut a, 100, 100)?, 1);
    assert!(!has(&a, "Saved 🦀\n草稿"));
    Ok(())
}

#[test]
fn activity_preserves_remaining_time_and_repeated_observations_do_not_reset_it()
-> std::result::Result<(), String> {
    let mut a = actor()?;
    dispatch(&mut a, SHOW, 1)?;
    let viewport = Key::named("test.toasts").0;
    assert_eq!(advance(&mut a, 30, 30)?, 0);
    a.observe_toast_activity(viewport, true)?;
    assert_eq!(
        a.projected_view().bindings[&viewport].node.action_payload,
        1
    );
    assert_eq!(advance(&mut a, 1000, 970)?, 0);
    a.observe_toast_activity(viewport, true)?;
    dispatch(&mut a, EXPIRE, 2)?; // already admitted callbacks cannot expire paused data
    assert!(has(&a, "Saved 🦀\n草稿"));
    assert_eq!(advance(&mut a, 1030, 30)?, 0);
    a.observe_toast_activity(viewport, false)?;
    a.observe_toast_activity(viewport, false)?;
    assert_eq!(advance(&mut a, 1099, 69)?, 0);
    assert!(has(&a, "Saved 🦀\n草稿"));
    assert_eq!(advance(&mut a, 1100, 1)?, 1);
    assert!(!has(&a, "Saved 🦀\n草稿"));
    assert!(a.observe_toast_activity(0, true).is_err());
    Ok(())
}

#[test]
fn insertion_and_update_while_paused_start_full_durations_on_resume()
-> std::result::Result<(), String> {
    let mut a = actor()?;
    dispatch(&mut a, SHOW, 1)?;
    let viewport = Key::named("test.toasts").0;
    advance(&mut a, 30, 30)?;
    a.observe_toast_activity(viewport, true)?;
    advance(&mut a, 1030, 1000)?;
    dispatch(&mut a, UPDATE, 1)?;
    dispatch(&mut a, SHOW, 2)?;
    advance(&mut a, 2030, 1000)?;
    assert!(has(&a, "Updated") && has(&a, "Second"));
    a.observe_toast_activity(viewport, false)?;
    assert_eq!(advance(&mut a, 2129, 99)?, 0);
    assert_eq!(advance(&mut a, 2130, 1)?, 1);
    assert!(has(&a, "Updated") && !has(&a, "Second"));
    assert_eq!(advance(&mut a, 2230, 100)?, 1);
    assert!(!has(&a, "Updated"));
    Ok(())
}

#[test]
fn pause_is_queue_local_clear_is_safe_and_late_activity_cannot_resurrect()
-> std::result::Result<(), String> {
    let mut a = actor()?;
    let viewport = Key::named("test.toasts").0;
    dispatch(&mut a, SHOW, 1)?;
    a.set_monotonic_ms_for_test(100)?;
    a.observe_toast_activity(viewport, true)?;
    assert!(!has(&a, "Saved 🦀\n草稿"));
    dispatch(&mut a, SHOW, 2)?;
    dispatch(&mut a, SHOW_OTHER, 0)?;
    assert_eq!(
        a.projected_view().bindings[&Key::named("other.toasts").0]
            .node
            .action_payload,
        0
    );
    dispatch(&mut a, CLEAR, 0)?;
    advance(&mut a, 1000, 1000)?;
    dispatch(&mut a, SHOW, 1)?;
    a.observe_toast_activity(viewport, false)?;
    advance(&mut a, 1099, 99)?;
    assert!(has(&a, "Saved 🦀\n草稿") && has(&a, "Independent"));
    advance(&mut a, 1100, 1)?;
    assert!(!has(&a, "Saved 🦀\n草稿") && has(&a, "Independent"));
    Ok(())
}
