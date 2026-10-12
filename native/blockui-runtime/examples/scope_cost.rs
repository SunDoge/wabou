//! Deterministic runtime-only comparison; excludes compilation, DLL loading and GPU work.
use blockui::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
const COUNT: State<i64> = State::<i64>::new("count", 0);
const FIXED: State<i64> = State::<i64>::new("fixed", 0);
static ROOTS: AtomicUsize = AtomicUsize::new(0);
static PANELS: AtomicUsize = AtomicUsize::new(0);
const SET: Action = Action::new("set", |cx, value| COUNT.set(cx, value));
const SCOPES: &[Scope] = &[
    Scope::new("panel.0", |cx| panel(cx, 0)),
    Scope::new("panel.1", |cx| panel(cx, 1)),
    Scope::new("panel.2", |cx| panel(cx, 2)),
    Scope::new("panel.3", |cx| panel(cx, 3)),
    Scope::new("panel.4", |cx| panel(cx, 4)),
    Scope::new("panel.5", |cx| panel(cx, 5)),
    Scope::new("panel.6", |cx| panel(cx, 6)),
    Scope::new("panel.7", |cx| panel(cx, 7)),
    Scope::new("panel.8", |cx| panel(cx, 8)),
    Scope::new("panel.9", |cx| panel(cx, 9)),
    Scope::new("panel.10", |cx| panel(cx, 10)),
    Scope::new("panel.11", |cx| panel(cx, 11)),
    Scope::new("panel.12", |cx| panel(cx, 12)),
    Scope::new("panel.13", |cx| panel(cx, 13)),
    Scope::new("panel.14", |cx| panel(cx, 14)),
    Scope::new("panel.15", |cx| panel(cx, 15)),
];
fn panel(cx: &Cx<'_>, index: u64) -> Result<View> {
    PANELS.fetch_add(1, Ordering::Relaxed);
    let value = if index == 0 {
        COUNT.get(cx)?
    } else {
        FIXED.get(cx)?
    };
    let mut view = Column::default().children(
        (0..8)
            .map(|child| {
                Text::new(format!("Panel {index}, row {child}: {value}"))
                    .key(Key::named("row").scoped(index).scoped(child))
                    .into_view()
            })
            .collect(),
    );
    if index == 0 && value % 2 != 0 {
        view.push(Text::new("Odd count").key("conditional"));
    }
    Ok(view)
}
fn root(cx: &Cx<'_>, full: bool) -> Result<View> {
    ROOTS.fetch_add(1, Ordering::Relaxed);
    if full {
        let _ = COUNT.get(cx)?;
    }
    Ok(Column::default().children(SCOPES.iter().map(|scope| scope.into_view()).collect()))
}
struct Local;
struct Full;
macro_rules! model {
    ($name:ident, $full:literal) => {
        impl Actor for $name {
            const STATE: &'static [StateSpec] = &[COUNT.spec(), FIXED.spec()];
            const ACTIONS: &'static [Action] = &[SET];
            const SCOPES: &'static [Scope] = SCOPES;
            fn view(cx: &Cx<'_>) -> Result<View> {
                root(cx, $full)
            }
        }
    };
}
model!(Local, false);
model!(Full, true);
struct Measurement {
    median: u128,
    p95: u128,
    roots: usize,
    panels: usize,
}
fn measure<A: Actor>(rounds: usize, full: bool) -> std::result::Result<Measurement, String> {
    let mut actor = blockui_runtime::Actor::for_test::<A>()?;
    let storage = actor.state_address();
    let slot = actor.slot_address("count");
    for value in 1..=50 {
        actor.dispatch(blockui_runtime::Message {
            action: SET.id,
            value,
        })?;
    }
    let roots = ROOTS.load(Ordering::Relaxed);
    let panels = PANELS.load(Ordering::Relaxed);
    let mut samples = Vec::with_capacity(rounds);
    for round in 0..rounds {
        let value = 51 + round as i64;
        let start = Instant::now();
        actor.dispatch(blockui_runtime::Message {
            action: SET.id,
            value,
        })?;
        samples.push(start.elapsed().as_nanos());
        assert_eq!(actor.integer("count"), value);
        assert_eq!(actor.state_address(), storage);
        assert_eq!(actor.slot_address("count"), slot);
        assert_eq!(
            actor.view.bindings.contains_key(&abi::key("conditional")),
            value % 2 != 0
        );
    }
    samples.sort_unstable();
    let result = Measurement {
        median: samples[rounds / 2],
        p95: samples[rounds * 95 / 100],
        roots: ROOTS.load(Ordering::Relaxed) - roots,
        panels: PANELS.load(Ordering::Relaxed) - panels,
    };
    assert_eq!(result.roots, if full { rounds } else { 0 });
    assert_eq!(result.panels, rounds);
    Ok(result)
}
fn main() -> std::result::Result<(), String> {
    let rounds = 1000;
    let local = measure::<Local>(rounds, false)?;
    let full = measure::<Full>(rounds, true)?;
    println!(
        "{{\"rounds\":{rounds},\"scopes\":16,\"rows_per_scope\":8,\"optimized\":{},\"local\":{{\"median_ns\":{},\"p95_ns\":{},\"root_builds\":{},\"scope_builds\":{}}},\"full\":{{\"median_ns\":{},\"p95_ns\":{},\"root_builds\":{},\"scope_builds\":{}}}}}",
        !cfg!(debug_assertions),
        local.median,
        local.p95,
        local.roots,
        local.panels,
        full.median,
        full.p95,
        full.roots,
        full.panels
    );
    Ok(())
}
