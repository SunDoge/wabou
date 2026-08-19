use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use clap::ArgMatches;
use vello::Scene;
use wabou_runtime::{AppConfig, Applier, JsRuntime, PasswordInput, SecretStore};
use wabou_shell::layout::PlacedNode;
use wabou_shell::renderer::render_to_png;
use wabou_shell::scene as scene_builder;
use wabou_shell::{
    FrameSource, KeyEvent, KeyLocation, KeyPhase, Modifiers, Point, PointerButton, PointerEvent,
    PointerPhase, TextContext, UiEvent, WheelEvent,
};

use super::config::{BuildProfile, bundle_path};
use super::frontend;
use super::project::App;
use super::{Result, ensure, render_metrics};

pub(super) struct RenderOptions {
    pub(super) out: PathBuf,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) window_id: u64,
    pub(super) scale_factor: f64,
    pub(super) mode: Option<String>,
    pub(super) wait_ms: u64,
    pub(super) metrics: Option<PathBuf>,
    pub(super) samples: usize,
    pub(super) actions: Vec<RenderAction>,
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
    let mut actions = clicks
        .chunks_exact(2)
        .map(|values| RenderAction::Click([values[0], values[1]]))
        .chain(
            wheels
                .chunks_exact(4)
                .map(|values| RenderAction::Wheel([values[0], values[1], values[2], values[3]])),
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
        for (positions, values) in indices
            .collect::<Vec<_>>()
            .chunks_exact(2)
            .zip(values.copied().collect::<Vec<_>>().chunks_exact(2))
        {
            indexed.push((positions[0], RenderAction::Click([values[0], values[1]])));
        }
    }
    if let (Some(indices), Some(values)) =
        (render.indices_of("wheel"), render.get_many::<f64>("wheel"))
    {
        for (positions, values) in indices
            .collect::<Vec<_>>()
            .chunks_exact(4)
            .zip(values.copied().collect::<Vec<_>>().chunks_exact(4))
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

pub(super) fn run(workspace: &Path, app: &App, options: &RenderOptions) -> Result<()> {
    let RenderOptions {
        out,
        width,
        height,
        window_id,
        scale_factor,
        mode,
        wait_ms,
        ..
    } = options;
    if !scale_factor.is_finite() || *scale_factor <= 0.0 {
        return Err("--scale-factor must be a finite number greater than zero".into());
    }
    let frontend_lock = frontend::lock(workspace, app)?;
    let mode_args = mode.as_deref().map(|mode| ["--mode", mode]);
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
    let path = bundle_path(workspace, app, BuildProfile::Debug)?;
    let source = fs::read_to_string(&path).map_err(|error| {
        format!(
            "failed to read JavaScript bundle {}: {error}",
            path.display()
        )
    })?;
    drop(frontend_lock);
    let js =
        JsRuntime::new().map_err(|error| format!("cannot create JavaScript runtime: {error:?}"))?;

    let base_color = AppConfig::new("").base_color;
    let mut factories = wabou_widgets::builtin_factories();
    factories.insert(
        "password-input".into(),
        Arc::new(|| Box::new(PasswordInput::new(SecretStore::default()))),
    );
    let mut applier =
        Applier::from_runtime_with_factories_and_window(js, factories, base_color, *window_id);
    applier
        .boot(&source)
        .map_err(|error| format!("cannot boot JavaScript bundle: {error:?}"))?;
    applier.set_device_scale(*scale_factor);
    let physical_width = (f64::from(*width) * scale_factor)
        .round()
        .clamp(1.0, f64::from(u32::MAX)) as u32;
    let physical_height = (f64::from(*height) * scale_factor)
        .round()
        .clamp(1.0, f64::from(u32::MAX)) as u32;
    applier.handle_event(UiEvent::WindowMetrics(wabou_shell::WindowMetrics {
        window_id: *window_id,
        logical_width: *width,
        logical_height: *height,
        physical_width,
        physical_height,
        scale_factor: *scale_factor,
        maximized: false,
        focused: true,
    }));
    let mut text_context = TextContext::new();
    let mut nodes = applier.build_frame(&mut text_context, *width, *height);
    settle(&mut applier, &mut text_context, &mut nodes, *width, *height);

    if *wait_ms > 0 {
        let deadline = Instant::now() + Duration::from_millis(*wait_ms);
        while Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
            nodes = applier.build_frame(&mut text_context, *width, *height);
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
