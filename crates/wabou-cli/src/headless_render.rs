use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::{Duration, Instant};

use anyrender::Scene;
use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use wabou_runtime::{AppConfig, Applier, JsRuntime, PasswordInput, SecretStore};
use wabou_shell::layout::PlacedNode;
use wabou_shell::renderer::render_to_png;
use wabou_shell::scene as scene_builder;
use wabou_shell::{
    FrameSource, KeyEvent, KeyLocation, KeyPhase, Modifiers, Point, PointerButton, PointerEvent,
    PointerPhase, TextContext, UiEvent, WheelEvent,
};

use super::artifact::{app_binary, app_framework_feature};
use super::config::{BuildProfile, bundle_path};
use super::frontend;
use super::process::{configure_test_backend, wait_for_managed_child};
use super::project::App;
use super::{Result, behavior_test_runtime, build_behavior_host, ensure, manifest, render_metrics};

pub(super) struct RenderOptions {
    pub(super) out: PathBuf,
    pub(super) batch: Option<PathBuf>,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) window_id: u64,
    pub(super) scale_factor: f64,
    pub(super) mode: Option<String>,
    pub(super) skip_build: bool,
    pub(super) with_host: bool,
    pub(super) scenario: Option<PathBuf>,
    pub(super) wait_ms: u64,
    pub(super) metrics: Option<PathBuf>,
    pub(super) snapshot: Option<PathBuf>,
    pub(super) samples: usize,
    pub(super) actions: Vec<RenderAction>,
    pub(super) layout_only: bool,
    pub(super) cargo_features: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LayoutBatchManifest {
    version: u32,
    #[serde(default)]
    all: bool,
    #[serde(default)]
    cases: Vec<LayoutBatchCase>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LayoutBatchCase {
    id: String,
    #[serde(default = "default_layout_width")]
    width: u32,
    #[serde(default = "default_layout_height")]
    height: u32,
    #[serde(default = "default_scale_factor")]
    scale_factor: f64,
    #[serde(default)]
    wait_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LayoutBatchReport {
    version: u32,
    total_duration_ms: f64,
    cases: Vec<LayoutBatchResult>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LayoutBatchResult {
    id: String,
    duration_ms: f64,
    snapshot: serde_json::Value,
}

const fn default_layout_width() -> u32 {
    1440
}
const fn default_layout_height() -> u32 {
    900
}
const fn default_scale_factor() -> f64 {
    1.0
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum RenderAction {
    Click([f64; 2]),
    Wheel([f64; 4]),
    Text(String),
    Key(String),
}

pub(super) fn legacy_actions(
    clicks: Vec<f64>,
    wheels: Vec<f64>,
    text: Option<String>,
    keys: Vec<String>,
) -> Vec<RenderAction> {
    let mut actions =
        clicks
            .as_chunks::<2>()
            .0
            .iter()
            .map(|values| RenderAction::Click([values[0], values[1]]))
            .chain(
                wheels.as_chunks::<4>().0.iter().map(|values| {
                    RenderAction::Wheel([values[0], values[1], values[2], values[3]])
                }),
            )
            .collect::<Vec<_>>();
    actions.extend(text.map(RenderAction::Text));
    actions.extend(keys.into_iter().map(RenderAction::Key));
    actions
}

pub(super) fn actions_from_matches(matches: &ArgMatches) -> Option<Vec<RenderAction>> {
    let (name, render) = matches.subcommand()?;
    if name != "render" {
        return None;
    }
    let mut indexed = Vec::<(usize, RenderAction)>::new();
    if let (Some(indices), Some(values)) =
        (render.indices_of("click"), render.get_many::<f64>("click"))
    {
        let positions = indices.collect::<Vec<_>>();
        let values = values.copied().collect::<Vec<_>>();
        for (positions, values) in positions
            .as_chunks::<2>()
            .0
            .iter()
            .zip(values.as_chunks::<2>().0.iter())
        {
            indexed.push((positions[0], RenderAction::Click([values[0], values[1]])));
        }
    }
    if let (Some(indices), Some(values)) =
        (render.indices_of("wheel"), render.get_many::<f64>("wheel"))
    {
        let positions = indices.collect::<Vec<_>>();
        let values = values.copied().collect::<Vec<_>>();
        for (positions, values) in positions
            .as_chunks::<4>()
            .0
            .iter()
            .zip(values.as_chunks::<4>().0.iter())
        {
            indexed.push((
                positions[0],
                RenderAction::Wheel([values[0], values[1], values[2], values[3]]),
            ));
        }
    }
    if let (Some(index), Some(value)) = (
        render
            .indices_of("text")
            .and_then(|mut values| values.next()),
        render.get_one::<String>("text"),
    ) {
        indexed.push((index, RenderAction::Text(value.clone())));
    }
    if let (Some(indices), Some(values)) =
        (render.indices_of("key"), render.get_many::<String>("key"))
    {
        indexed.extend(
            indices
                .zip(values)
                .map(|(index, value)| (index, RenderAction::Key(value.clone()))),
        );
    }
    indexed.sort_by_key(|(index, _)| *index);
    Some(indexed.into_iter().map(|(_, action)| action).collect())
}

fn settle(
    applier: &mut Applier,
    text: &mut TextContext,
    nodes: &mut Vec<PlacedNode>,
    width: u32,
    height: u32,
) {
    const MAX_SETTLE_FRAMES: usize = 32;
    const REQUIRED_QUIET_FRAMES: usize = 2;

    let mut quiet_frames = 0;
    for _ in 0..MAX_SETTLE_FRAMES {
        let revision = applier.protocol_revision();
        *nodes = applier.build_frame(text, width, height);
        if applier.protocol_revision() == revision {
            quiet_frames += 1;
            if quiet_frames == REQUIRED_QUIET_FRAMES {
                break;
            }
        } else {
            quiet_frames = 0;
        }
    }
}

fn apply_actions(
    applier: &mut Applier,
    text_context: &mut TextContext,
    nodes: &mut Vec<PlacedNode>,
    options: &RenderOptions,
) {
    let mut settle = |applier: &mut Applier| {
        settle(applier, text_context, nodes, options.width, options.height);
    };
    for action in &options.actions {
        match action {
            RenderAction::Click([x, y]) => {
                let point = Point { x: *x, y: *y };
                for (phase, buttons) in [(PointerPhase::Down, 1), (PointerPhase::Up, 0)] {
                    applier.handle_event(UiEvent::Pointer(PointerEvent {
                        phase,
                        position: point,
                        button: Some(PointerButton::Primary),
                        buttons,
                        modifiers: Modifiers::default(),
                    }));
                }
            }
            RenderAction::Wheel([x, y, delta_x, delta_y]) => {
                applier.handle_event(UiEvent::Wheel(WheelEvent {
                    position: Point { x: *x, y: *y },
                    delta_x: *delta_x,
                    delta_y: *delta_y,
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
                    }));
                }
            }
        }
        settle(applier);
    }
}

fn run_layout_batch(
    applier: &mut Applier,
    debug_state: &wabou_devtools::SharedDebugState,
    window_key: wabou_shell::WindowResourceKey,
    manifest_path: &Path,
    out: &Path,
    wait_ms: u64,
) -> Result<()> {
    let mut manifest: LayoutBatchManifest = serde_json::from_slice(&fs::read(manifest_path)?)
        .map_err(|error| {
            format!(
                "failed to parse layout batch manifest {}: {error}",
                manifest_path.display()
            )
        })?;
    if manifest.version != 1 {
        return Err(format!(
            "unsupported layout batch manifest version {}; expected 1",
            manifest.version
        )
        .into());
    }
    if manifest.all && !manifest.cases.is_empty() {
        return Err("layout batch manifest cannot combine `all` with explicit cases".into());
    }
    if manifest.all {
        let encoded = applier
            .eval_string(
                "typeof globalThis.__wabou_layout_fixture_cases === 'function' \
                    ? globalThis.__wabou_layout_fixture_cases() \
                    : JSON.stringify(JSON.parse(globalThis.__wabou_layout_fixture_ids()).map(id => ({ id })))",
            )
            .map_err(|error| format!("failed to list layout fixtures: {error:?}"))?;
        manifest.cases = serde_json::from_str(&encoded)
            .map_err(|error| format!("layout fixture registry returned invalid cases: {error}"))?;
    }
    if manifest.cases.is_empty() {
        return Err("layout batch manifest must contain at least one case".into());
    }
    let mut ids = HashSet::new();
    for case in &manifest.cases {
        if case.id.is_empty() {
            return Err("layout batch case id must not be empty".into());
        }
        if !ids.insert(case.id.clone()) {
            return Err(format!("duplicate layout batch case id `{}`", case.id).into());
        }
        if case.width == 0 || case.height == 0 {
            return Err(format!(
                "layout batch case `{}` requires non-zero width and height",
                case.id
            )
            .into());
        }
        if !case.scale_factor.is_finite() || case.scale_factor <= 0.0 {
            return Err(format!(
                "layout batch case `{}` requires a finite scaleFactor greater than zero",
                case.id
            )
            .into());
        }
    }

    let batch_started = Instant::now();
    let mut results = Vec::with_capacity(manifest.cases.len());
    let mut text = TextContext::new();
    for case in manifest.cases {
        let case_started = Instant::now();
        // The JS fixture harness disposes the preceding Solid owner. Resetting
        // the native projection as well prevents focus, scrolling, widgets,
        // and resources from surviving a malformed fixture cleanup.
        applier.reset_scene_tree();
        let id = serde_json::to_string(&case.id)?;
        applier
            .eval_script_diagnostic(&format!("globalThis.__wabou_layout_fixture_mount({id});"))
            .map_err(|error| format!("failed to mount layout fixture `{}`: {error}", case.id))?;
        applier.set_device_scale(case.scale_factor);
        let physical_width = (f64::from(case.width) * case.scale_factor)
            .round()
            .clamp(1.0, f64::from(u32::MAX)) as u32;
        let physical_height = (f64::from(case.height) * case.scale_factor)
            .round()
            .clamp(1.0, f64::from(u32::MAX)) as u32;
        applier.handle_event(UiEvent::WindowMetrics(wabou_shell::WindowMetrics {
            window_key,
            logical_width: case.width,
            logical_height: case.height,
            physical_width,
            physical_height,
            scale_factor: case.scale_factor,
            maximized: false,
            focused: true,
            color_scheme: Some(wabou_shell::ColorScheme::Light),
        }));
        let mut nodes = applier.build_frame(&mut text, case.width, case.height);
        settle(applier, &mut text, &mut nodes, case.width, case.height);
        let case_wait_ms = case.wait_ms.unwrap_or(wait_ms);
        if case_wait_ms > 0 {
            let deadline = Instant::now() + Duration::from_millis(case_wait_ms);
            while Instant::now() < deadline {
                thread::sleep(Duration::from_millis(10));
                nodes = applier.build_frame(&mut text, case.width, case.height);
            }
        }
        // Force publication of the exact frame captured for this case.
        applier.set_debug_state(debug_state.clone());
        let _ = applier.build_frame(&mut text, case.width, case.height);
        let state = debug_state
            .read()
            .map_err(|_| "headless debug snapshot lock was poisoned")?;
        let snapshot = serde_json::to_value(state.snapshot())?;
        results.push(LayoutBatchResult {
            id: case.id,
            duration_ms: case_started.elapsed().as_secs_f64() * 1_000.0,
            snapshot,
        });
    }

    if let Some(parent) = out.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        out,
        serde_json::to_vec_pretty(&LayoutBatchReport {
            version: 1,
            total_duration_ms: batch_started.elapsed().as_secs_f64() * 1_000.0,
            cases: results,
        })?,
    )?;
    println!(
        "[wabou] wrote {} layout fixtures to {}",
        ids.len(),
        out.display()
    );
    Ok(())
}

pub(super) fn run(workspace: &Path, app: &App, options: &RenderOptions) -> Result<()> {
    let RenderOptions {
        out,
        batch,
        width,
        height,
        window_id,
        scale_factor,
        mode,
        skip_build,
        with_host,
        wait_ms,
        layout_only,
        ..
    } = options;
    if !scale_factor.is_finite() || *scale_factor <= 0.0 {
        return Err("--scale-factor must be a finite number greater than zero".into());
    }
    if *with_host {
        return run_with_host(workspace, app, options);
    }
    if !options.cargo_features.is_empty() {
        return Err("--features requires --with-host for `wabou render`".into());
    }
    let window_key = u32::try_from(*window_id)
        .ok()
        .and_then(|lo| wabou_shell::WindowResourceKey::from_parts(lo, 1))
        .ok_or("--window-id must be a non-zero 32-bit logical window id")?;
    prepare_frontend(workspace, app, mode.as_deref(), *skip_build)?;
    let path = bundle_path(workspace, app, BuildProfile::Debug)?;
    let source = fs::read_to_string(&path).map_err(|error| {
        format!(
            "failed to read JavaScript bundle {}: {error}",
            path.display()
        )
    })?;
    let js =
        JsRuntime::new().map_err(|error| format!("cannot create JavaScript runtime: {error:?}"))?;

    let base_color = AppConfig::new("").base_color;
    let mut factories = wabou_widgets::builtin_factories();
    factories.insert(
        "password-input".into(),
        Arc::new(|| Box::new(PasswordInput::new(SecretStore::default()))),
    );
    let mut applier =
        Applier::from_runtime_with_factories_and_window(js, factories, base_color, window_key);
    let debug_state = options.snapshot.as_ref().map(|_| {
        let state = wabou_devtools::DebugState::shared();
        applier.set_debug_state(state.clone());
        state
    });
    applier
        .boot(&source)
        .map_err(|error| format!("cannot boot JavaScript bundle: {error:?}"))?;
    if let Some(manifest_path) = batch {
        if !layout_only {
            return Err("--batch is only supported by `wabou layout`".into());
        }
        let state = debug_state
            .as_ref()
            .ok_or("layout batch execution requires debug snapshots")?;
        return run_layout_batch(
            &mut applier,
            state,
            window_key,
            manifest_path,
            out,
            *wait_ms,
        );
    }
    applier.set_device_scale(*scale_factor);
    let physical_width = (f64::from(*width) * scale_factor)
        .round()
        .clamp(1.0, f64::from(u32::MAX)) as u32;
    let physical_height = (f64::from(*height) * scale_factor)
        .round()
        .clamp(1.0, f64::from(u32::MAX)) as u32;
    applier.handle_event(UiEvent::WindowMetrics(wabou_shell::WindowMetrics {
        window_key,
        logical_width: *width,
        logical_height: *height,
        physical_width,
        physical_height,
        scale_factor: *scale_factor,
        maximized: false,
        focused: true,
        color_scheme: Some(wabou_shell::ColorScheme::Light),
    }));
    let mut text_context = TextContext::new();
    let mut profiler = wabou_shell::headless::HeadlessFrameProfiler::default();
    let mut nodes = applier.build_frame(&mut text_context, *width, *height);
    settle(&mut applier, &mut text_context, &mut nodes, *width, *height);
    nodes = if *layout_only {
        applier.build_frame(&mut text_context, *width, *height)
    } else {
        profiler.build(
            &mut applier,
            &mut text_context,
            *width,
            *height,
            *scale_factor,
            base_color,
        )
    };

    if *wait_ms > 0 {
        let deadline = Instant::now() + Duration::from_millis(*wait_ms);
        while Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
            nodes = if *layout_only {
                applier.build_frame(&mut text_context, *width, *height)
            } else {
                profiler.build(
                    &mut applier,
                    &mut text_context,
                    *width,
                    *height,
                    *scale_factor,
                    base_color,
                )
            };
        }
    }
    apply_actions(&mut applier, &mut text_context, &mut nodes, options);

    if let Some(path) = &options.metrics {
        render_metrics::write(
            render_metrics::RenderMetricsOptions {
                path,
                application: &app.name,
                width: *width,
                height: *height,
                scale_factor: *scale_factor,
                samples: options.samples,
                base_color,
            },
            &mut applier,
            &mut text_context,
        )?;
        nodes = applier.build_frame(&mut text_context, *width, *height);
    }

    if let (Some(path), Some(state)) = (&options.snapshot, debug_state.as_ref()) {
        // Publish after every action, settling interval, and metrics sample.
        // The resulting nodes are passed unchanged into scene construction, so
        // the JSON and PNG describe one final logical frame.
        applier.set_debug_state(state.clone());
        nodes = applier.build_frame(&mut text_context, *width, *height);
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        let state = state
            .read()
            .map_err(|_| "headless debug snapshot lock was poisoned")?;
        fs::write(path, serde_json::to_vec_pretty(state.snapshot())?)?;
    }

    if *layout_only {
        let snapshot = options
            .snapshot
            .as_ref()
            .ok_or("layout-only execution requires a snapshot output")?;
        println!("[wabou] wrote layout snapshot {}", snapshot.display());
        return Ok(());
    }

    if let Some(parent) = out.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let mut scene = Scene::new();
    scene_builder::build_scene_scaled(
        &mut scene,
        &nodes,
        &mut text_context,
        *width,
        *height,
        base_color,
        *scale_factor,
    );
    // Keep bundle-only captures faithful to the native window path: debug
    // decorations are appended after the application scene and never
    // participate in layout, clipping, or hit testing.
    applier.paint_debug_overlay(&mut scene, &nodes, &mut text_context, *scale_factor);
    let out_text = out
        .to_str()
        .ok_or_else(|| format!("output path is not valid UTF-8: {}", out.display()))?;
    render_to_png(
        &scene,
        physical_width,
        physical_height,
        base_color,
        out_text,
    )
    .map_err(|error| format!("failed to render {}: {error:?}", out.display()))?;
    println!("[wabou] rendered {}", out.display());
    Ok(())
}

fn run_with_host(workspace: &Path, app: &App, options: &RenderOptions) -> Result<()> {
    if !options.actions.is_empty() {
        return Err("--with-host does not yet support --click, --wheel, --key, or --text".into());
    }
    if options.metrics.is_some() {
        return Err("--with-host does not support --metrics; use the bundle-only renderer".into());
    }

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
    let snapshot_feature = options
        .snapshot
        .as_ref()
        .map(|_| app_framework_feature(workspace, app, "devtools"))
        .transpose()?;
    let mut cargo_features = options.cargo_features.clone();
    if let Some(snapshot_feature) = snapshot_feature {
        cargo_features.push(snapshot_feature);
    }
    let executable = build_behavior_host(workspace, &manifest, &binary, &cargo_features)?;
    let test_data = tempfile::tempdir_in(&render_dir)?;
    let output = if options.out.is_absolute() {
        options.out.clone()
    } else {
        std::env::current_dir()?.join(&options.out)
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
        .env(
            "WABOU_TEST_CAPTURE_WINDOW_ID",
            options.window_id.to_string(),
        )
        .env("WABOU_TEST_APP_DATA_ROOT", test_data.path())
        .env("XDG_CONFIG_HOME", test_data.path().join("xdg-config"))
        .env("XDG_DATA_HOME", test_data.path().join("xdg-data"))
        .env("XDG_CACHE_HOME", test_data.path().join("xdg-cache"));
    let snapshot = options.snapshot.as_ref().map(|path| {
        if path.is_absolute() {
            path.clone()
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| workspace.to_owned())
                .join(path)
        }
    });
    if let Some(snapshot) = &snapshot {
        host.env("WABOU_TEST_SNAPSHOT_PATH", snapshot);
    }
    configure_test_backend(&mut host, false);
    let status = wait_for_managed_child(host, Duration::from_secs(70), &AtomicBool::new(false))?;
    ensure(status, "Wabou host-backed render")?;
    if !output.is_file() {
        return Err(format!("host-backed render did not create {}", output.display()).into());
    }
    if let Some(snapshot) = snapshot
        && !snapshot.is_file()
    {
        return Err(format!("host-backed render did not create {}", snapshot.display()).into());
    }
    println!(
        "[wabou] rendered {} with application host",
        options.out.display()
    );
    Ok(())
}

fn prepare_frontend(
    workspace: &Path,
    app: &App,
    mode: Option<&str>,
    skip_build: bool,
) -> Result<()> {
    let bundle = bundle_path(workspace, app, BuildProfile::Debug)?;
    if skip_build {
        if !bundle.is_file() {
            return Err(format!(
                "--skip-build requires an existing debug bundle at {}; run `wabou render` without --skip-build first",
                bundle.display()
            )
            .into());
        }
        return Ok(());
    }

    let frontend_lock = frontend::lock(workspace, app)?;
    let mode_args = mode.map(|mode| ["--mode", mode]);
    ensure(
        frontend::build_unlocked(
            workspace,
            app,
            mode_args.as_ref().map_or(&[], |args| args),
            BuildProfile::Debug,
            true,
        )?,
        "Vite build",
    )?;
    drop(frontend_lock);
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

#[cfg(test)]
mod tests {
    use super::{host_capture_scenario_source, prepare_frontend};
    use crate::project::App;
    use std::path::Path;

    #[test]
    fn authored_capture_scenario_runs_before_the_settling_test() {
        let source = host_capture_scenario_source(
            Path::new("/wabou/test.ts"),
            Some(Path::new("/app/capture.ts")),
            250,
        )
        .unwrap();

        let authored = source.find("/app/capture.ts").unwrap();
        let settle = source.find("settle host-backed capture").unwrap();
        assert!(authored < settle);
        assert!(source.contains("setTimeout(resolve, 250)"));
        assert!(source.contains("{ timeout: 5250 }"));
    }

    #[test]
    fn skip_build_requires_an_existing_debug_bundle() {
        let workspace = tempfile::tempdir().unwrap();
        let app = App {
            name: "demo".into(),
            root: workspace.path().join("apps/demo"),
            frontend: workspace.path().join("apps/demo"),
            entry: "ui/index.tsx".into(),
        };

        let error = prepare_frontend(workspace.path(), &app, None, true).unwrap_err();
        assert!(error.to_string().contains("--skip-build requires"));

        let bundle = workspace.path().join("dist/demo/debug/resources/bundle.js");
        std::fs::create_dir_all(bundle.parent().unwrap()).unwrap();
        std::fs::write(&bundle, "bundle").unwrap();
        prepare_frontend(workspace.path(), &app, None, true).unwrap();
    }
}
