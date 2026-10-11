//! Headless native frame/scene timing: no desktop input, GPU or swapchain.
use blockui_runtime::{App, native_model::NativeModel};
use std::time::Instant;
use wabou_shell_vello::{
    FrameSource, PointerEvent, PointerPhase, TextContext, UiEvent, headless::HeadlessFrameProfiler,
};
fn main() -> Result<(), String> {
    let library = std::env::args()
        .nth(1)
        .ok_or("gallery_perf LIBRARY [ITERATIONS]")?;
    let iterations = std::env::args()
        .nth(2)
        .map(|v| v.parse::<usize>())
        .transpose()
        .map_err(|e| e.to_string())?
        .unwrap_or(200)
        .clamp(20, 10000);
    let (mut source, _) = App::new(library, ".runtime").into_window()?;
    let mut text = TextContext::new();
    let mut profiler = HeadlessFrameProfiler::default();
    let mut rows = Vec::new();
    for page in [0, 1, 13] {
        let nav = source
            .actor
            .root_widget(blockui_abi::key(&format!("nav.{page}")))
            .ok_or("missing page")?;
        source.actor.dispatch_widget(nav)?;
        for mode in ["idle", "hover", "wheel_pixel", "wheel_line"] {
            let height = if mode.starts_with("wheel") { 480 } else { 820 };
            let mut times = Vec::new();
            let scroll = source
                .actor
                .root_widget(blockui_abi::key("gallery.scroll"))
                .ok_or("missing gallery scroll")?;
            let mut last_offset = source.actor.view().renderer[&scroll].scroll;
            let mut moved_frames = 0;

            for i in 0..iterations + 10 {
                let started = Instant::now();
                if mode == "hover" {
                    source.handle_event(UiEvent::Pointer(PointerEvent {
                        phase: PointerPhase::Move,
                        position: wabou_shell_vello::Point {
                            x: 80.0,
                            y: if i % 2 == 0 { 240.0 } else { 275.0 },
                        },
                        buttons: 0,
                        button: None,
                        modifiers: Default::default(),
                        properties: Default::default(),
                    }));
                }
                if mode.starts_with("wheel") {
                    source.handle_event(UiEvent::Wheel(wabou_shell_vello::WheelEvent {
                        position: wabou_shell_vello::Point {
                            x: 1080.0,
                            y: 240.0,
                        },
                        delta_x: 0.0,
                        delta_y: if i / 16 % 2 == 0 { 40.0 } else { -40.0 },
                        delta_mode: if mode == "wheel_line" {
                            wabou_shell_vello::WheelDeltaMode::Line
                        } else {
                            wabou_shell_vello::WheelDeltaMode::Pixel
                        },
                        phase: wabou_shell_vello::GesturePhase::Changed,
                        modifiers: Default::default(),
                    }));
                }
                profiler.build(
                    &mut source,
                    &mut text,
                    1120,
                    height,
                    1.0,
                    vello_common::peniko::Color::WHITE,
                );
                let offset = source.actor.view().renderer[&scroll].scroll;
                if i >= 10 && offset != last_offset {
                    moved_frames += 1;
                }
                last_offset = offset;
                if i >= 10 {
                    times.push(started.elapsed().as_secs_f64() * 1000.0);
                }
            }
            times.sort_by(f64::total_cmp);
            let stats = profiler.stats();
            rows.push(serde_json::json!({"page": page, "mode": mode, "viewport_height": height, "scroll_moved_frames": moved_frames, "frames": iterations, "nodes": stats.node_count, "frame_scene_p50_ms": times[times.len()/2], "frame_scene_p95_ms": times[times.len()*95/100], "last_build_ms": stats.build_frame_ms, "last_scene_ms": stats.scene_ms}));
        }
    }
    println!(
        "{}",
        serde_json::json!({"optimized_host": !cfg!(debug_assertions), "includes_gpu": false, "samples": rows})
    );
    Ok(())
}
