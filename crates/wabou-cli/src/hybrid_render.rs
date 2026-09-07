//! Deterministic QuickJS + Taffy capture through the Vello Hybrid renderer.

use std::{
    fs,
    path::Path,
    process::Command,
    sync::Arc,
    sync::atomic::AtomicBool,
    thread,
    time::{Duration, Instant},
};

use serde::Deserialize;
use wabou_backend_vello_hybrid::{AppConfig, Applier, JsRuntime};
use wabou_shell_vello::{
    FrameSource, KeyEvent, KeyLocation, KeyPhase, Modifiers, Point, PointerButton, PointerEvent,
    PointerPhase, Scene, TextContext, UiEvent, WheelEvent, layout::PlacedNode,
};
use wabou_widgets_vello::{PasswordInput, SecretStore};

use super::{
    Result,
    artifact::{app_binary, app_framework_feature},
    behavior_test_runtime, build_behavior_host,
    config::{BuildProfile, bundle_path},
    ensure, frontend,
    gpui_render::{HeadlessColorScheme, RenderAction, RenderOptions, prepare_frontend},
    manifest,
    process::{configure_test_backend, wait_for_behavior_host},
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
    if options.with_host {
        return run_with_host(workspace, app, options);
    }
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
        .and_then(|lo| wabou_shell_vello::WindowResourceKey::from_parts(lo, 1))
        .ok_or("--window-id must be a non-zero 32-bit logical window id")?;
    let base_color = AppConfig::new("").base_color;
    let runtime =
        JsRuntime::new().map_err(|error| format!("cannot create JavaScript runtime: {error:?}"))?;
    let mut factories = wabou_widgets_vello::builtin_factories();
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
    applier.handle_event(UiEvent::WindowMetrics(wabou_shell_vello::WindowMetrics {
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
            HeadlessColorScheme::Light => wabou_shell_vello::ColorScheme::Light,
            HeadlessColorScheme::Dark => wabou_shell_vello::ColorScheme::Dark,
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
        let mut snapshot = serde_json::to_value(state.snapshot())?;
        label_snapshot_renderer(&mut snapshot, "vello-hybrid")?;
        fs::write(path, serde_json::to_vec_pretty(&snapshot)?)?;
    }

    let mut scene = Scene::new();
    wabou_shell_vello::scene::build_scene_scaled(
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
    wabou_shell_vello::renderer::render_to_png(
        &scene,
        physical_width,
        physical_height,
        base_color,
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
    if options.with_host {
        if options.fixture.is_some() {
            return Err("--fixture is not supported with --with-host".into());
        }
        if !options.actions.is_empty() {
            return Err(
                "--with-host does not yet support --click, --wheel, --key, or --text".into(),
            );
        }
    } else {
        if options.scenario.is_some() {
            return Err("--scenario requires --with-host".into());
        }
        if !options.cargo_features.is_empty() {
            return Err("--features requires --with-host for `wabou render`".into());
        }
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

fn run_with_host(workspace: &Path, app: &App, options: &RenderOptions) -> Result<()> {
    prepare_frontend(workspace, app, options.mode.as_deref(), options.skip_build)?;

    let render_dir = workspace.join("target/wabou-render").join(&app.name);
    fs::create_dir_all(&render_dir)?;
    let scenario = render_dir.join("capture.ts");
    let test_runtime = behavior_test_runtime(workspace)?;
    let authored_scenario = options
        .scenario
        .as_deref()
        .map(|path| {
            fs::canonicalize(path).map_err(|error| {
                format!("cannot resolve render scenario {}: {error}", path.display())
            })
        })
        .transpose()?;
    if authored_scenario
        .as_deref()
        .is_some_and(|path| !path.is_file())
    {
        return Err("--scenario must point to a TypeScript file".into());
    }
    fs::write(
        &scenario,
        host_capture_scenario_source(&test_runtime, authored_scenario.as_deref(), options.wait_ms)?,
    )?;
    let scenario_bundle = render_dir.join("scenario.js");
    ensure(
        frontend::build_test_script(workspace, app, &scenario, &scenario_bundle)?,
        "Vite render scenario build",
    )?;

    let manifest = manifest(app);
    let binary = app_binary(workspace, app)?;
    let mut cargo_features = options.cargo_features.clone();
    if options.snapshot.is_some() {
        cargo_features.push(app_framework_feature(workspace, app, "devtools")?);
    }
    let executable = build_behavior_host(workspace, &manifest, &binary, &cargo_features)?;
    let test_data = tempfile::tempdir_in(&render_dir)?;
    let absolute = |path: &Path| {
        if path.is_absolute() {
            path.to_owned()
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| workspace.to_owned())
                .join(path)
        }
    };
    let output = absolute(&options.out);
    let snapshot = options.snapshot.as_deref().map(absolute);
    let color_scheme = match options.color_scheme {
        HeadlessColorScheme::Light => "light",
        HeadlessColorScheme::Dark => "dark",
    };

    let mut host = Command::new(executable);
    host.current_dir(workspace)
        .env(
            "WABOU_BUNDLE_PATH",
            bundle_path(workspace, app, BuildProfile::Debug)?,
        )
        .env("WABOU_TEST_SCRIPT", scenario_bundle)
        .env("WABOU_TEST_CAPTURE_PATH", &output)
        .env("WABOU_TEST_VIEWPORT_WIDTH", options.width.to_string())
        .env("WABOU_TEST_VIEWPORT_HEIGHT", options.height.to_string())
        .env("WABOU_TEST_SCALE_FACTOR", options.scale_factor.to_string())
        .env("WABOU_TEST_COLOR_SCHEME", color_scheme)
        .env(
            "WABOU_TEST_CAPTURE_WINDOW_ID",
            options.window_id.to_string(),
        )
        .env("WABOU_TEST_APP_DATA_ROOT", test_data.path())
        .env("XDG_CONFIG_HOME", test_data.path().join("xdg-config"))
        .env("XDG_DATA_HOME", test_data.path().join("xdg-data"))
        .env("XDG_CACHE_HOME", test_data.path().join("xdg-cache"));
    if let Some(snapshot) = &snapshot {
        host.env("WABOU_TEST_SNAPSHOT_PATH", snapshot);
    }
    configure_test_backend(&mut host, false);
    let status = wait_for_behavior_host(host, Duration::from_secs(310), &AtomicBool::new(false))?;
    ensure(status, "Wabou host-backed Vello Hybrid render")?;
    if !output.is_file() {
        return Err(format!("host-backed render did not create {}", output.display()).into());
    }
    if let Some(snapshot) = snapshot {
        if !snapshot.is_file() {
            return Err(format!("host-backed render did not create {}", snapshot.display()).into());
        }
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&snapshot)?)?;
        label_snapshot_renderer(&mut value, "vello-hybrid")?;
        fs::write(&snapshot, serde_json::to_vec_pretty(&value)?)?;
    }
    println!(
        "[wabou] wrote Vello Hybrid capture {} with application host",
        options.out.display()
    );
    Ok(())
}

fn label_snapshot_renderer(snapshot: &mut serde_json::Value, renderer: &str) -> Result<()> {
    let status = snapshot
        .get_mut("status")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("capture snapshot is missing its status object")?;
    status.insert("renderer".into(), renderer.into());
    Ok(())
}

fn host_capture_scenario_source(
    test_runtime: &Path,
    authored_scenario: Option<&Path>,
    wait_ms: u64,
) -> Result<String> {
    let mut source = String::new();
    if let Some(authored_scenario) = authored_scenario {
        source.push_str(&format!(
            "import {};\n",
            serde_json::to_string(&authored_scenario.to_string_lossy())?
        ));
    }
    source.push_str(&format!(
        "import {{ test }} from {};\n\
         test(\"settle host-backed capture\", async ({{ page }}) => {{\n\
         await page.waitForIdle();\n\
         await new Promise((resolve) => setTimeout(resolve, {wait_ms}));\n\
         await page.waitForIdle();\n\
         }}, {{ timeout: {} }});\n",
        serde_json::to_string(&test_runtime.to_string_lossy())?,
        wait_ms.saturating_add(5_000),
    ));
    Ok(source)
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
                    delta_mode: wabou_shell_vello::WheelDeltaMode::Pixel,
                    phase: wabou_shell_vello::GesturePhase::Changed,
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::host_capture_scenario_source;

    #[test]
    fn host_capture_runs_authored_scenario_before_settling_the_final_frame() {
        let source = host_capture_scenario_source(
            Path::new("/workspace/packages/test/runtime.ts"),
            Some(Path::new("/workspace/apps/gallery/captures/input.ts")),
            250,
        )
        .expect("generate capture scenario");

        let authored = source
            .find("/workspace/apps/gallery/captures/input.ts")
            .expect("authored scenario import");
        let settle = source
            .find("settle host-backed capture")
            .expect("settling test");
        assert!(authored < settle);
        assert!(source.contains("setTimeout(resolve, 250)"));
        assert!(source.contains("timeout: 5250"));
    }
}
