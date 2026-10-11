//! Explicit headless host-sync measurement; never creates a window or GPU.
use super::*;
use blockui::{Badge, Column, Icon, IconSource, IntoView, Row, Text, ThemeMode, ViewStyle};

struct Fixture<const ROWS: usize>;
impl<const ROWS: usize> blockui::Actor for Fixture<ROWS> {
    const STATE: &'static [blockui::StateSpec] = &[];
    const ACTIONS: &'static [blockui::Action] = &[];
    fn view(_: &blockui::Cx<'_>) -> blockui::Result<blockui::View> {
        let icon = IconSource::svg(include_str!("../../blockui-icons-lucide/icons/search.svg"));
        Ok(Column::default()
            .children(
                (0..ROWS)
                    .map(|i| {
                        Row::default().key(format!("row.{i}")).children(vec![
                            Badge::new("Ready").key(format!("badge.{i}")).into_view(),
                            Icon::new(icon).key(format!("icon.{i}")).into_view(),
                            Text::new("Stable text")
                                .key(format!("text.{i}"))
                                .into_view(),
                        ])
                    })
                    .collect(),
            )
            .style(ViewStyle {
                theme: ThemeMode::Light,
                ..Default::default()
            }))
    }
}

fn measure<const ROWS: usize>() -> Result<serde_json::Value, String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<Fixture<ROWS>>()?)?;
    let mut idle = Vec::new();
    let mut forced = Vec::new();
    let reconciliations = source.sync_reconciliations;
    for sample in 0..5 {
        for refresh in if sample % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let start = Instant::now();
            for _ in 0..200 {
                if refresh {
                    // A comparison workload, not a historical implementation:
                    // force the three cache passes that idle sync now avoids.
                    source.sync_badge_rows();
                    source.sync_icons()?;
                    source.theme_system_dark = true; // Fixture has no system-dark preference.
                }
                source.sync()?;
            }
            let micros = start.elapsed().as_secs_f64() * 1_000_000.0 / 200.0;
            if refresh {
                forced.push(micros);
            } else {
                idle.push(micros);
            }
        }
    }
    assert_eq!(source.sync_reconciliations, reconciliations);
    let median = |values: &[f64]| {
        let mut sorted = values.to_vec();
        sorted.sort_by(f64::total_cmp);
        sorted[sorted.len() / 2]
    };
    Ok(serde_json::json!({
        "nodes": source.actor.view.bindings.len(), "rounds_per_sample":200,
        "idle_median_us":median(&idle), "forced_refresh_median_us":median(&forced),
        "idle_samples_us":idle, "forced_refresh_samples_us":forced,
    }))
}

#[test]
#[ignore = "explicit native host-sync benchmark"]
fn native_idle_sync_benchmark() -> Result<(), String> {
    println!(
        "NATIVE_SYNC_BENCH {}",
        serde_json::json!({
            "debug_assertions":cfg!(debug_assertions),
            "cases":[measure::<8>()?,measure::<32>()?,measure::<128>()?,measure::<512>()?],
            "excludes":["actor construction/admission", "layout", "text shaping", "scene construction", "GPU", "window"],
            "comparison":"current idle sync versus forced cache refresh; not a historical before/after or frame-time measurement",
        })
    );
    Ok(())
}
