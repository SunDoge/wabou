//! Deterministic QuickJS + Taffy capture through the Vello Hybrid renderer.

use std::{
    fs,
    path::Path,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use anyrender::Scene;
use serde::Deserialize;
use wabou_legacy_runtime::{AppConfig, Applier, JsRuntime};
use wabou_legacy_shell::{
    FrameSource, KeyEvent, KeyLocation, KeyPhase, Modifiers, Point, PointerButton, PointerEvent,
    PointerPhase, RendererBackend, TextContext, UiEvent, WheelEvent, layout::PlacedNode,
};
use wabou_legacy_widgets::{PasswordInput, SecretStore};

use super::{
    Result,
    config::{BuildProfile, bundle_path},
    gpui_render::{HeadlessColorScheme, RenderAction, RenderOptions, prepare_frontend},
    project::App,
};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LayoutFixture {
    id: String,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
    #[serde(default)]
    scale_factor: Option<f64>,
    #[serde(default)]
    wait_ms: Option<u64>,
}

pub(super) fn run(workspace: &Path, app: &App, options: &RenderOptions) -> Result<()> {
    validate_options(options)?;
    let frontend_mode = options
        .fixture
        .as_ref()
        .map(|_| "layout-test")
        .or(options.mode.as_deref());
    prepare_frontend(workspace, app, frontend_mode, options.skip_build)?;

    let bundle = bundle_path(workspace, app, BuildProfile::Debug)?;
    let source = fs::read_to_string(&bundle).map_err(|error| {
        format!(
            "failed to read JavaScript bundle {}: {error}",
            bundle.display()
        )
    })?;
    let window_key = u32::try_from(options.window_id)
        .ok()
        .and_then(|lo| wabou_legacy_shell::WindowResourceKey::from_parts(lo, 1))
        .ok_or("--window-id must be a non-zero 32-bit logical window id")?;
    let base_color = AppConfig::new("").base_color;
    let runtime =
        JsRuntime::new().map_err(|error| format!("cannot create JavaScript runtime: {error:?}"))?;
    let mut factories = wabou_legacy_widgets::builtin_factories();
    factories.insert(
        "password-input".into(),
        Arc::new(|| Box::new(PasswordInput::new(SecretStore::default()))),
    );
    let mut applier =
        Applier::from_runtime_with_factories_and_window(runtime, factories, base_color, window_key);
    let debug_state = options.snapshot.as_ref().map(|_| {
        let state = wabou_devtools::DebugState::shared();
        applier.set_debug_state(state.clone());
        state
    });
    applier
        .boot(&source)
        .map_err(|error| format!("cannot boot JavaScript bundle: {error:?}"))?;

    let fixture = options
        .fixture
        .as_deref()
        .map(|id| mount_fixture(&mut applier, id))
        .transpose()?;
    let width = fixture
        .as_ref()
        .and_then(|value| value.width)
        .unwrap_or(options.width);
    let height = fixture
        .as_ref()
        .and_then(|value| value.height)
        .unwrap_or(options.height);
    let scale_factor = fixture
        .as_ref()
        .and_then(|value| value.scale_factor)
        .unwrap_or(options.scale_factor);
    let wait_ms = fixture
        .as_ref()
        .and_then(|value| value.wait_ms)
        .unwrap_or(options.wait_ms);
    validate_dimensions(width, height, scale_factor)?;

    let physical_width = physical_size(width, scale_factor);
    let physical_height = physical_size(height, scale_factor);
    applier.set_device_scale(scale_factor);
    applier.handle_event(UiEvent::WindowMetrics(wabou_legacy_shell::WindowMetrics {
        window_key,
        logical_width: width,
        logical_height: height,
        physical_width,
        physical_height,
        scale_factor,
        maximized: false,
        focused: true,
        outer_x: None,
        outer_y: None,
        occluded: false,
        reduced_motion: false,
        color_scheme: Some(match options.color_scheme {
            HeadlessColorScheme::Light => wabou_legacy_shell::ColorScheme::Light,
            HeadlessColorScheme::Dark => wabou_legacy_shell::ColorScheme::Dark,
        }),
    }));

    let mut text = TextContext::new();
    let mut nodes = applier.build_frame(&mut text, width, height);
    settle(&mut applier, &mut text, &mut nodes, width, height);
    replay_actions(
        &mut applier,
        &mut text,
        &mut nodes,
        width,
        height,
        &options.actions,
    );
    drive_for(&mut applier, &mut text, &mut nodes, width, height, wait_ms);

    if let (Some(path), Some(state)) = (&options.snapshot, debug_state) {
        applier.set_debug_state(state.clone());
        nodes = applier.build_frame(&mut text, width, height);
        create_parent(path)?;
        let state = state
            .read()
            .map_err(|_| "headless debug snapshot lock was poisoned")?;
        fs::write(path, serde_json::to_vec_pretty(state.snapshot())?)?;
    }

    let mut scene = Scene::new();
    wabou_legacy_shell::scene::build_scene_scaled(
        &mut scene,
        &nodes,
        &mut text,
        width,
        height,
        base_color,
        scale_factor,
    );
    applier.paint_debug_overlay(&mut scene, &nodes, &mut text, scale_factor);
    create_parent(&options.out)?;
    let out = options
        .out
        .to_str()
        .ok_or_else(|| format!("output path is not valid UTF-8: {}", options.out.display()))?;
    wabou_legacy_shell::renderer::render_to_png_with_backend(
        &scene,
        physical_width,
        physical_height,
        base_color,
        RendererBackend::VelloHybrid,
        out,
    )
    .map_err(|error| format!("failed to render {}: {error:?}", options.out.display()))?;
    println!(
        "[wabou] wrote Vello Hybrid capture {}",
        options.out.display()
    );
    Ok(())
}

fn validate_options(options: &RenderOptions) -> Result<()> {
    validate_dimensions(options.width, options.height, options.scale_factor)?;
    if options.with_host || options.scenario.is_some() || !options.cargo_features.is_empty() {
        return Err("host-backed Vello Hybrid capture has not been migrated yet".into());
    }
    if options.metrics.is_some() || options.samples != 20 {
        return Err("Vello Hybrid capture metrics have not been migrated yet".into());
    }
    if options.batch.is_some() || options.layout_only || options.projection_probe.is_some() {
        return Err("use `wabou layout` for layout batches and projection probes".into());
    }
    for action in &options.actions {
        let values: &[f64] = match action {
            RenderAction::Click(values) => values,
            RenderAction::Wheel(values) => values,
            RenderAction::Text(_) | RenderAction::Key(_) => continue,
        };
        if values.iter().any(|value| !value.is_finite()) {
            return Err("capture coordinates and deltas must be finite".into());
        }
    }
    Ok(())
}

fn validate_dimensions(width: u32, height: u32, scale_factor: f64) -> Result<()> {
    if width == 0 || height == 0 {
        return Err("capture width and height must be non-zero".into());
    }
    if !scale_factor.is_finite() || scale_factor <= 0.0 {
        return Err("--scale-factor must be a finite number greater than zero".into());
    }
    Ok(())
}

fn physical_size(logical: u32, scale_factor: f64) -> u32 {
    (f64::from(logical) * scale_factor)
        .round()
        .clamp(1.0, f64::from(u32::MAX)) as u32
}

fn create_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn mount_fixture(applier: &mut Applier, id: &str) -> Result<LayoutFixture> {
    let encoded = applier
        .eval_string(
            "typeof globalThis.__wabou_layout_fixture_cases === 'function' \
                ? globalThis.__wabou_layout_fixture_cases() \
                : '[]'",
        )
        .map_err(|error| format!("failed to list layout fixtures: {error:?}"))?;
    let fixtures: Vec<LayoutFixture> = serde_json::from_str(&encoded)
        .map_err(|error| format!("layout fixture registry returned invalid cases: {error}"))?;
    let fixture = fixtures
        .iter()
        .find(|fixture| fixture.id == id)
        .cloned()
        .ok_or_else(|| {
            let available = fixtures
                .iter()
                .map(|fixture| fixture.id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!("unknown layout fixture `{id}`; available fixtures: {available}")
        })?;
    let id = serde_json::to_string(id)?;
    applier
        .eval_script_diagnostic(&format!("globalThis.__wabou_layout_fixture_mount({id});"))
        .map_err(|error| format!("failed to mount layout fixture: {error}"))?;
    Ok(fixture)
}

fn settle(
    applier: &mut Applier,
    text: &mut TextContext,
    nodes: &mut Vec<PlacedNode>,
    width: u32,
    height: u32,
) {
    const MAX_FRAMES: usize = 32;
    const QUIET_FRAMES: usize = 2;
    let mut quiet = 0;
    for _ in 0..MAX_FRAMES {
        let revision = applier.protocol_revision();
        *nodes = applier.build_frame(text, width, height);
        if applier.protocol_revision() == revision {
            quiet += 1;
            if quiet == QUIET_FRAMES {
                break;
            }
        } else {
            quiet = 0;
        }
    }
}

fn replay_actions(
    applier: &mut Applier,
    text: &mut TextContext,
    nodes: &mut Vec<PlacedNode>,
    width: u32,
    height: u32,
    actions: &[RenderAction],
) {
    for action in actions {
        match action {
            RenderAction::Click([x, y]) => {
                let position = Point { x: *x, y: *y };
                for (phase, buttons) in [(PointerPhase::Down, 1), (PointerPhase::Up, 0)] {
                    applier.handle_event(UiEvent::Pointer(PointerEvent {
                        phase,
                        position,
                        button: Some(PointerButton::Primary),
                        buttons,
                        modifiers: Modifiers::default(),
                        properties: Default::default(),
                    }));
                }
            }
            RenderAction::Wheel([x, y, delta_x, delta_y]) => {
                applier.handle_event(UiEvent::Wheel(WheelEvent {
                    position: Point { x: *x, y: *y },
                    delta_x: *delta_x,
                    delta_y: *delta_y,
                    delta_mode: wabou_legacy_shell::WheelDeltaMode::Pixel,
                    phase: wabou_legacy_shell::GesturePhase::Changed,
                    modifiers: Modifiers::default(),
                }));
            }
            RenderAction::Text(value) => {
                applier.handle_event(UiEvent::TextInput(value.clone()));
            }
            RenderAction::Key(key) => {
                for phase in [KeyPhase::Down, KeyPhase::Up] {
                    applier.handle_event(UiEvent::Key(KeyEvent {
                        phase,
                        key: key.clone(),
                        key_without_modifiers: key.clone(),
                        code: key.clone(),
                        text: None,
                        text_with_all_modifiers: None,
                        location: KeyLocation::Standard,
                        modifiers: Modifiers::default(),
                        repeat: false,
                        synthetic: false,
                    }));
                }
            }
        }
        settle(applier, text, nodes, width, height);
    }
}

fn drive_for(
    applier: &mut Applier,
    text: &mut TextContext,
    nodes: &mut Vec<PlacedNode>,
    width: u32,
    height: u32,
    wait_ms: u64,
) {
    let deadline = Instant::now() + Duration::from_millis(wait_ms);
    while Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
        *nodes = applier.build_frame(text, width, height);
    }
}
