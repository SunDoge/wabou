//! Command-line entry point for developing, building, and testing Wabou apps.

use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde_json::Value;

mod artifact;
mod behavior_test;
mod clean;
mod config;
mod devtools;
mod distribution;
mod doctor;
mod frontend;
#[cfg(feature = "embedded-renderer")]
mod hybrid_render;
#[cfg(not(feature = "embedded-renderer"))]
mod hybrid_render {
    use super::{App, RenderOptions, Result};
    use std::path::Path;
    pub(super) fn run(_: &Path, _: &App, _: &RenderOptions) -> Result<()> {
        Err("this CLI was built without embedded-renderer; use a CLI build with that feature for render/layout".into())
    }
    pub(super) fn run_layout(workspace: &Path, app: &App, options: &RenderOptions) -> Result<()> {
        run(workspace, app, options)
    }
}
mod packaging;
mod process;
mod project;
mod render;
mod scaffold;
mod ts_config;

use artifact::{
    BindingsTarget, app_binary, app_bindings_target, app_dev_features, app_framework_feature,
    app_profiling_feature, artifact_from_metadata_for_target, cargo_metadata,
    optional_app_bindings_target,
};
#[cfg(test)]
use artifact::{
    artifact_from_metadata, binary_target, bindings_target, dev_features, framework_feature,
};
use behavior_test::{default_artifact_dir, prepare_artifact_dir, replay_actions};
use config::{
    BuildProfile, PackageFormat, bundle_path, configured_source_map, load_package_config,
    profile_application_dir,
};
use devtools::InspectCommand;
use frontend::{build as build_frontend, build_test_script};
#[cfg(all(test, unix))]
use process::wait_for_managed_child;
use process::{
    ManagedChild, configure_test_backend, ensure_host_exit, supervise, wait_for_behavior_host,
    wait_for_vite,
};
#[cfg(test)]
use project::missing_workspace_package_exports;
use project::{
    App, ensure_javascript_dependencies, ensure_workspace_package_exports, find_app_root,
    find_workspace, load_app,
};
use render::{HeadlessColorScheme, RenderOptions, actions as fallback_render_actions};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(usage_rs::Cli)]
#[usage(
    bin = "wabou",
    version,
    about = "Build and run Wabou applications",
    unknown_flags = "error"
)]
struct Cli {
    #[usage(subcommand)]
    command: Commands,
}

#[derive(usage_rs::Args, Clone, Debug, Default, Eq, PartialEq)]
struct CargoFeatures {
    /// Cargo features enabled for the application host. May be repeated or comma-separated.
    #[usage(long = "features", value_name = "FEATURES", delimiter = ',')]
    values: Vec<String>,
}

#[derive(usage_rs::Subcommands)]
enum Commands {
    /// Run a JSON application configuration through the linked fallback host.
    Launch {
        #[usage(value_name = "CONFIG_JSON")]
        config: PathBuf,
    },
    /// Build a portable application using the embedded runtime and capability libraries.
    Bundle {
        #[usage(value_name = "APP")]
        app: Option<PathBuf>,
        #[usage(long)]
        config: Option<PathBuf>,
        #[usage(long)]
        out: PathBuf,
    },
    /// Create a standalone Wabou application pinned to a Git revision.
    New {
        #[usage(value_name = "PATH")]
        path: PathBuf,
        /// Git repository used for the embedded Wabou submodule.
        #[usage(long, default = "https://github.com/SunDoge/wabou.git")]
        wabou_repository: String,
        /// Tag or commit used for the embedded Wabou submodule.
        #[usage(long, default_fn = scaffold::default_revision, default_note = "current CLI version")]
        wabou_ref: String,
    },
    /// Add or update the pinned Wabou source submodule in an existing application.
    Vendor {
        /// Application repository root. Defaults to the current directory.
        #[usage(value_name = "APP")]
        app: Option<PathBuf>,
        /// Git repository used when adding the Wabou submodule.
        #[usage(long, default = "https://github.com/SunDoge/wabou.git")]
        wabou_repository: String,
        /// Tag or commit checked out in the Wabou submodule.
        #[usage(long, default_fn = scaffold::default_revision, default_note = "current CLI version")]
        wabou_ref: String,
    },
    /// Check the development environment and Wabou project setup.
    Doctor {
        #[usage(value_name = "APP")]
        app: Option<PathBuf>,
    },
    /// Verify an application's Rust, TypeScript, bindings, and behavior contracts.
    Check {
        #[usage(value_name = "APP")]
        app: Option<PathBuf>,
        /// Skip discovered `tests/**/*.behavior.ts` scenarios.
        #[usage(long)]
        skip_behavior: bool,
        #[usage(flatten)]
        cargo_features: CargoFeatures,
    },
    /// Remove generated JavaScript bundles and frontend caches.
    Clean {
        #[usage(value_name = "APP")]
        app: Option<PathBuf>,
        /// Also remove dist directories for local packages/* workspaces.
        #[usage(long)]
        packages: bool,
    },
    /// Start Vite, the Rust host, and live HMR.
    Dev {
        #[usage(value_name = "APP")]
        app: Option<PathBuf>,
        #[usage(long, default = "5173")]
        port: u16,
        #[usage(long)]
        devtools: bool,
        /// Show the renderer-native performance HUD.
        #[usage(long)]
        hud: bool,
        /// Capability/window launch configuration; Vite source is set by dev.
        #[usage(long, value_name = "JSON")]
        runtime_config: Option<PathBuf>,
        /// Vite mode used to select an application-owned development entry.
        #[usage(long)]
        mode: Option<String>,
        #[usage(flatten)]
        cargo_features: CargoFeatures,
    },
    /// Build the frontend bundle and Rust host.
    Build {
        #[usage(value_name = "APP")]
        app: Option<PathBuf>,
        #[usage(long)]
        release: bool,
        /// Generate a source map even for a release build.
        #[usage(long, value_name = "BOOL", num_args = 0..=1, default_missing = "true")]
        source_map: Option<bool>,
        #[usage(flatten)]
        cargo_features: CargoFeatures,
    },
    /// Build a release application and create native installers or bundles.
    Package {
        #[usage(value_name = "APP")]
        app: Option<PathBuf>,
        /// Override the formats declared in wabou.toml.
        #[usage(long, value_enum)]
        format: Vec<PackageFormat>,
        /// Build the Rust host with cargo-zigbuild for this target before packaging.
        #[usage(long, value_name = "TARGET")]
        target: Option<String>,
        #[usage(flatten)]
        cargo_features: CargoFeatures,
    },
    /// Build the frontend bundle and run the Rust host.
    Run {
        #[usage(value_name = "APP")]
        app: Option<PathBuf>,
        #[usage(long)]
        release: bool,
        /// Generate a source map even for a release build.
        #[usage(long, value_name = "BOOL", num_args = 0..=1, default_missing = "true")]
        source_map: Option<bool>,
        /// Write an opt-in performance trace for Perfetto/Chrome tracing.
        #[usage(long, value_name = "JSON")]
        profile_trace: Option<PathBuf>,
        /// Show the renderer-native performance HUD.
        #[usage(long)]
        hud: bool,
        #[usage(flatten)]
        cargo_features: CargoFeatures,
    },
    /// Run TypeScript behavior tests against the native host.
    Test {
        /// Scenario file, test directory, or Wabou application directory.
        /// Defaults to the current application's tests directory.
        #[usage(value_name = "TARGET", conflicts = "--replay")]
        scenario: Option<PathBuf>,
        /// Run the scenario against an application outside the current directory.
        #[usage(long, value_name = "PATH")]
        app: Option<PathBuf>,
        /// Replay a JSON action trace produced by an earlier test run.
        #[usage(long, value_name = "TRACE_OR_REPORT", conflicts = "scenario")]
        replay: Option<PathBuf>,
        /// Replay through the named test in a report, including prior state-building actions.
        #[usage(long, value_name = "NAME", requires = "--replay")]
        replay_test: Option<String>,
        /// Directory for the JSON report and replayable action trace.
        #[usage(long, value_name = "DIR")]
        artifacts: Option<PathBuf>,
        /// Vite mode used to select an application-owned test fixture.
        #[usage(long)]
        mode: Option<String>,
        /// Render a PNG after failure; requires an available wgpu backend.
        #[usage(long)]
        failure_screenshot: bool,
        /// Use the real platform event loop instead of the deterministic backend.
        #[usage(long)]
        native: bool,
        /// Environment variable passed explicitly to the application host.
        #[usage(long = "env", value_name = "KEY=VALUE")]
        host_env: Vec<String>,
        #[usage(flatten)]
        cargo_features: CargoFeatures,
    },
    /// Generate or verify Rust-owned TypeScript capability bindings.
    Bindings {
        #[usage(subcommand)]
        command: BindingsCommand,
    },
    /// Render an application to a PNG without opening a native window.
    Render {
        #[usage(value_name = "APP")]
        app: Option<PathBuf>,
        #[usage(long, value_name = "PNG")]
        out: PathBuf,
        #[usage(long, default = "1440")]
        width: u32,
        #[usage(long, default = "900")]
        height: u32,
        /// Logical window id exposed to the application during boot.
        #[usage(long, default = "1")]
        window_id: u64,
        /// Device scale used for widget encoding and physical PNG dimensions.
        #[usage(long, default = "1.0")]
        scale_factor: f64,
        /// System color scheme exposed to the application during capture.
        #[usage(long, value_enum, default = "light")]
        color_scheme: HeadlessColorScheme,
        /// Vite mode used to select an application-owned render fixture.
        #[usage(long)]
        mode: Option<String>,
        /// Render one named application layout fixture.
        #[usage(
            long,
            value_name = "ID",
            conflicts("--mode", "--with-host", "--width", "--height", "--scale-factor")
        )]
        fixture: Option<String>,
        /// Reuse an existing debug frontend bundle instead of invoking Vite.
        #[usage(long)]
        skip_build: bool,
        /// Run the application's Rust host registrations before capturing.
        #[usage(long)]
        with_host: bool,
        /// Run an authored @wabou/test scenario before a host-backed capture.
        #[usage(long, value_name = "TS", requires = "--with-host")]
        scenario: Option<PathBuf>,
        /// Keep driving frames before capture so finite UI transitions can settle.
        /// Use zero to capture the earliest available frame.
        #[usage(long, default = "1000")]
        wait_ms: u64,
        /// Write non-blocking headless build/scene timing samples as JSON.
        #[usage(long, value_name = "JSON")]
        metrics: Option<PathBuf>,
        /// Write the DevTools tree snapshot represented by the rendered frame.
        #[usage(long, value_name = "JSON")]
        snapshot: Option<PathBuf>,
        /// Number of headless frames sampled for --metrics.
        #[usage(long, default = "20", requires = "--metrics")]
        samples: usize,
        /// Dispatch a primary click at X Y before capture.
        #[usage(
            long,
            num_args = 2,
            value_names = ["X", "Y"]
        )]
        click: Vec<f64>,
        /// Dispatch a wheel gesture at X Y with horizontal and vertical deltas.
        #[usage(
            long,
            num_args = 4,
            value_names = ["X", "Y", "DX", "DY"]
        )]
        wheel: Vec<f64>,
        /// Dispatch a named key press before capture, in command-line action order.
        #[usage(long, value_name = "KEY")]
        key: Vec<String>,
        /// Commit text before capture, in command-line action order.
        #[usage(long, requires = "--click")]
        text: Option<String>,
        /// Cargo features enabled for the application host used by `--with-host`.
        #[usage(flatten)]
        cargo_features: CargoFeatures,
    },
    /// Evaluate JavaScript, Style IR, and Taffy through the Vello Hybrid backend.
    Layout {
        #[usage(value_name = "APP")]
        app: Option<PathBuf>,
        /// Write the structured retained layout snapshot.
        #[usage(long, value_name = "JSON")]
        out: PathBuf,
        /// Render every fixture described by a batch manifest in an isolated runtime.
        #[usage(long, value_name = "JSON")]
        batch: Option<PathBuf>,
        /// Mount one application-owned layout fixture.
        #[usage(long, value_name = "ID", conflicts = "--batch")]
        fixture: Option<String>,
        #[usage(long, default = "1440")]
        width: u32,
        #[usage(long, default = "900")]
        height: u32,
        /// Logical window id exposed to the application during boot.
        #[usage(long, default = "1")]
        window_id: u64,
        /// Logical device scale used by text and native-widget measurement.
        #[usage(long, default = "1.0")]
        scale_factor: f64,
        /// System color scheme exposed to layout fixtures.
        #[usage(long, value_enum, default = "light")]
        color_scheme: HeadlessColorScheme,
        /// Vite mode used to select an application-owned fixture.
        #[usage(long)]
        mode: Option<String>,
        /// Reuse an existing debug frontend bundle instead of invoking Vite.
        #[usage(long)]
        skip_build: bool,
        /// Keep driving layout frames so finite reactive work can settle.
        #[usage(long, default = "0")]
        wait_ms: u64,
    },
    /// Open the native Wabou inspector.
    Devtools,
    /// Inspect a running Wabou application from the terminal.
    Inspect {
        #[usage(long, env = "WABOU_DEVTOOLS_SOCKET")]
        socket: Option<PathBuf>,
        #[usage(subcommand)]
        command: InspectCommand,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, usage_rs::Subcommands)]
enum BindingsCommand {
    /// Rewrite the committed TypeScript declarations.
    Write {
        #[usage(value_name = "APP")]
        app: Option<PathBuf>,
        #[usage(flatten)]
        cargo_features: CargoFeatures,
    },
    /// Fail when the committed declarations differ from Rust types.
    Check {
        #[usage(value_name = "APP")]
        app: Option<PathBuf>,
        #[usage(flatten)]
        cargo_features: CargoFeatures,
    },
}

struct TestOptions {
    scenario: Option<PathBuf>,
    replay: Option<PathBuf>,
    replay_test: Option<String>,
    artifacts: Option<PathBuf>,
    mode: Option<String>,
    failure_screenshot: bool,
    native: bool,
    host_env: Vec<String>,
    cargo_features: Vec<String>,
}

fn resolve_behavior_test_target(
    explicit_app: Option<&Path>,
    target: Option<PathBuf>,
) -> (Option<PathBuf>, Option<PathBuf>) {
    if let Some(app) = explicit_app {
        return (Some(app.to_path_buf()), target);
    }
    let Some(target) = target else {
        return (None, None);
    };
    if is_wabou_app_directory(&target) {
        return (Some(target), None);
    }
    let search_from = if target.is_dir() {
        target.as_path()
    } else {
        target.parent().unwrap_or(target.as_path())
    };
    (
        project::find_frontend_root(search_from).or_else(|| find_app_root(search_from)),
        Some(target),
    )
}

pub fn execute() -> Result<()> {
    let arguments = env::args_os().collect::<Vec<_>>();
    #[cfg(not(feature = "embedded-renderer"))]
    if matches!(
        arguments.get(1).and_then(|argument| argument.to_str()),
        Some("launch" | "bundle" | "render" | "layout")
    ) {
        return delegate_renderer(&arguments);
    }
    let render_actions = render::actions_from_argv(&arguments);
    let cli = Cli::parse();
    let cwd = env::current_dir()?;
    let resolve_app = |app_path: Option<&Path>| {
        let workspace = find_workspace(&cwd)?;
        let app = load_app(&workspace, &cwd, app_path)?;
        Ok::<_, Box<dyn Error>>((workspace, app))
    };
    match cli.command {
        Commands::Launch { config } => launch_application(&config),
        Commands::Bundle { app, config, out } => {
            let frontend_root = project::find_frontend_root(&cwd).unwrap_or_else(|| cwd.clone());
            let workspace = find_workspace(&cwd).unwrap_or(frontend_root);
            let mut app = project::load_frontend_app(&workspace, &cwd, app.as_deref())?;
            let prepared = if config.is_none() {
                ts_config::prepare(&app)?
            } else {
                None
            };
            if let Some(prepared) = &prepared {
                app.entry = prepared.entry.clone();
            }
            let config = config
                .or_else(|| prepared.as_ref().map(|p| p.file.path().to_owned()))
                .or(config::load_dev_config(&app)?.runtime_config)
                .ok_or("bundle needs wabou.config.ts or --config")?;
            distribution::bundle(&workspace, &app, &config, &out)
        }
        Commands::New {
            path,
            wabou_repository,
            wabou_ref,
        } => scaffold::create(&cwd.join(path), &wabou_repository, &wabou_ref),
        Commands::Vendor {
            app,
            wabou_repository,
            wabou_ref,
        } => scaffold::vendor(
            &app.map(|path| cwd.join(path)).unwrap_or(cwd),
            &wabou_repository,
            &wabou_ref,
        ),
        Commands::Doctor { app } => doctor::run(&cwd, app.as_deref()),
        Commands::Check {
            app,
            skip_behavior,
            cargo_features,
        } => {
            let start = app
                .as_ref()
                .map(|p| cwd.join(p))
                .unwrap_or_else(|| cwd.clone());
            let workspace = find_workspace(&start)?;
            let app = project::load_frontend_app(&workspace, &cwd, app.as_deref())?;
            check(&workspace, &app, skip_behavior, &cargo_features.values)
        }
        Commands::Clean { app, packages } => {
            let (workspace, app) = resolve_app(app.as_deref())?;
            clean::run(&workspace, &app, packages).map(|_| ())
        }
        Commands::Dev {
            app,
            port,
            devtools,
            hud,
            runtime_config,
            mode,
            cargo_features,
        } => {
            let frontend_root = project::find_frontend_root(&cwd).unwrap_or_else(|| cwd.clone());
            let workspace = find_workspace(&cwd).unwrap_or(frontend_root);
            let mut app = project::load_frontend_app(&workspace, &cwd, app.as_deref())?;
            let configured = config::load_dev_config(&app)?;
            let runtime_config = runtime_config.or(configured.runtime_config);
            let prepared = if runtime_config.is_none() {
                ts_config::prepare(&app)?
            } else {
                None
            };
            if let Some(prepared) = &prepared {
                app.entry = prepared.entry.clone();
            }
            let runtime_config =
                runtime_config.or_else(|| prepared.as_ref().map(|p| p.file.path().to_owned()));
            if runtime_config.is_none() && !app.root.join("Cargo.toml").is_file() {
                return Err(
                    "frontend-only dev needs wabou.config.ts or [dev] runtime-config in wabou.toml"
                        .into(),
                );
            }
            dev(
                &workspace,
                app,
                DevOptions {
                    port,
                    open_devtools: devtools,
                    hud,
                    runtime_config: runtime_config.as_deref(),
                    mode: mode.as_deref(),
                    cargo_features: &cargo_features.values,
                },
            )
        }
        Commands::Build {
            app,
            release,
            source_map,
            cargo_features,
        } => {
            let start = app
                .as_ref()
                .map(|p| cwd.join(p))
                .unwrap_or_else(|| cwd.clone());
            let workspace = find_workspace(&start)?;
            let app = project::load_frontend_app(&workspace, &cwd, app.as_deref())?;
            build(
                &workspace,
                &app,
                release,
                source_map,
                &cargo_features.values,
            )
        }
        Commands::Package {
            app,
            format,
            target,
            cargo_features,
        } => {
            let (workspace, app) = resolve_app(app.as_deref())?;
            package(
                &workspace,
                &app,
                &format,
                target.as_deref(),
                &cargo_features.values,
            )
        }
        Commands::Run {
            app,
            release,
            source_map,
            profile_trace,
            hud,
            cargo_features,
        } => {
            let start = app
                .as_ref()
                .map(|p| cwd.join(p))
                .unwrap_or_else(|| cwd.clone());
            let workspace = find_workspace(&start)?;
            let app = project::load_frontend_app(&workspace, &cwd, app.as_deref())?;
            let profile_trace = profile_trace.map(|path| cwd.join(path));
            run(
                &workspace,
                &app,
                release,
                source_map,
                profile_trace.as_deref(),
                hud,
                &cargo_features.values,
            )
        }
        Commands::Test {
            scenario,
            app,
            replay,
            replay_test,
            artifacts,
            mode,
            failure_screenshot,
            native,
            host_env,
            cargo_features,
        } => {
            let target = scenario.map(|path| cwd.join(path));
            let (app_path, scenario) = resolve_behavior_test_target(app.as_deref(), target);
            let start = app_path
                .as_ref()
                .map(|p| cwd.join(p))
                .unwrap_or_else(|| cwd.clone());
            let workspace = find_workspace(&start)?;
            let app = project::load_frontend_app(&workspace, &cwd, app_path.as_deref())?;
            let options = TestOptions {
                scenario,
                replay: replay.map(|path| cwd.join(path)),
                replay_test,
                artifacts,
                mode,
                failure_screenshot,
                native,
                host_env,
                cargo_features: cargo_features.values,
            };
            test_scenario(&workspace, &app, &options)
        }
        Commands::Bindings { command } => {
            let app_path = match &command {
                BindingsCommand::Write { app, .. } | BindingsCommand::Check { app, .. } => {
                    app.as_deref()
                }
            };
            let (workspace, app) = resolve_app(app_path)?;
            bindings(&workspace, &app, command)
        }
        Commands::Render {
            app,
            out,
            width,
            height,
            window_id,
            scale_factor,
            color_scheme,
            mode,
            fixture,
            skip_build,
            with_host,
            scenario,
            wait_ms,
            metrics,
            snapshot,
            samples,
            click,
            wheel,
            key,
            text,
            cargo_features,
        } => {
            let start = app
                .as_ref()
                .map(|path| cwd.join(path))
                .unwrap_or_else(|| cwd.clone());
            let workspace = find_workspace(&start)?;
            let app = project::load_frontend_app(&workspace, &cwd, app.as_deref())?;
            let options = RenderOptions {
                out,
                batch: None,
                width,
                height,
                window_id,
                scale_factor,
                color_scheme,
                mode,
                fixture,
                skip_build,
                with_host,
                scenario,
                wait_ms,
                metrics,
                snapshot,
                samples,
                actions: render_actions
                    .unwrap_or_else(|| fallback_render_actions(click, wheel, text, key)),
                layout_only: false,
                cargo_features: cargo_features.values,
            };
            hybrid_render::run(&workspace, &app, &options)
        }
        Commands::Layout {
            app,
            out,
            batch,
            fixture,
            width,
            height,
            window_id,
            scale_factor,
            color_scheme,
            mode,
            skip_build,
            wait_ms,
        } => {
            let start = app
                .as_ref()
                .map(|path| cwd.join(path))
                .unwrap_or_else(|| cwd.clone());
            let workspace = find_workspace(&start)?;
            let app = project::load_frontend_app(&workspace, &cwd, app.as_deref())?;
            hybrid_render::run_layout(
                &workspace,
                &app,
                &RenderOptions {
                    out: out.clone(),
                    batch,
                    width,
                    height,
                    window_id,
                    scale_factor,
                    color_scheme,
                    mode,
                    fixture,
                    skip_build,
                    with_host: false,
                    scenario: None,
                    wait_ms,
                    metrics: None,
                    snapshot: Some(out),
                    samples: 0,
                    actions: Vec::new(),
                    layout_only: true,
                    cargo_features: Vec::new(),
                },
            )
        }
        Commands::Devtools => devtools::run(&find_workspace(&cwd).unwrap_or(cwd)),
        Commands::Inspect { socket, command } => devtools::inspect(socket, command),
    }
}

/// Run the engine-backed helper. The separate entry point keeps the normal CLI
/// free of renderer dependencies while reusing its render/layout contracts.
#[cfg(feature = "embedded-renderer")]
pub fn run_renderer() -> Result<()> {
    let command = env::args_os().nth(1);
    if !matches!(
        command.as_deref().and_then(|value| value.to_str()),
        Some("launch" | "bundle" | "render" | "layout")
    ) {
        return Err(
            "wabou-render only accepts the launch, bundle, render, and layout commands".into(),
        );
    }
    execute()
}

#[cfg(not(feature = "embedded-renderer"))]
pub fn run_renderer() -> Result<()> {
    Err("wabou-render requires the embedded-renderer feature".into())
}

#[cfg(not(feature = "embedded-renderer"))]
fn delegate_renderer(arguments: &[std::ffi::OsString]) -> Result<()> {
    let current_exe = env::current_exe()?;
    let executable = format!("wabou-render{}", env::consts::EXE_SUFFIX);
    let sibling = current_exe
        .parent()
        .map(|parent| parent.join(&executable))
        .filter(|path| path.is_file());
    let installed = sibling.or_else(|| {
        env::var_os("PATH").and_then(|path| {
            env::split_paths(&path)
                .map(|directory| directory.join(&executable))
                .find(|candidate| candidate.is_file())
        })
    });
    let mut command = if let Some(executable) = installed {
        Command::new(executable)
    } else {
        let cwd = env::current_dir()?;
        let workspace = find_workspace(&cwd)?;
        let manifest = [
            workspace.join("crates/wabou-cli/Cargo.toml"),
            workspace.join("vendor/wabou/crates/wabou-cli/Cargo.toml"),
        ]
        .into_iter()
        .find(|candidate| candidate.is_file())
        .ok_or("wabou-render is not installed; install the engine-backed helper next to wabou")?;
        let mut cargo = Command::new("cargo");
        cargo.current_dir(&workspace).args([
            "run",
            "--manifest-path",
            manifest
                .to_str()
                .ok_or("wabou-render manifest path is not UTF-8")?,
            "--features",
            "embedded-renderer",
            "--bin",
            "wabou-render",
            "--",
        ]);
        cargo
    };
    command.args(arguments.iter().skip(1));
    let status = command.status()?;
    std::process::exit(status.code().unwrap_or(1));
}

fn manifest(app: &App) -> String {
    app.root.join("Cargo.toml").to_string_lossy().into_owned()
}

fn apply_cargo_features(command: &mut Command, features: &[String]) {
    if !features.is_empty() {
        command.arg("--features").arg(features.join(","));
    }
}

fn check(
    workspace: &Path,
    app: &App,
    skip_behavior: bool,
    cargo_features: &[String],
) -> Result<()> {
    ensure_javascript_dependencies(workspace, app)?;
    ensure_workspace_package_exports(workspace)?;

    let tsconfig = app
        .root
        .ancestors()
        .map(|directory| directory.join("tsconfig.json"))
        .find(|path| path.is_file())
        .ok_or_else(|| format!("{} has no tsconfig.json", app.root.display()))?;
    println!("[wabou check] TypeScript");
    let mut typescript = Command::new("bun");
    typescript
        .current_dir(&app.root)
        .args(["x", "tsc", "--noEmit", "--project"])
        .arg(tsconfig);
    let status = typescript
        .status()
        .map_err(|error| format!("failed to start Bun for TypeScript check: {error}"))?;
    ensure(status, "TypeScript check")?;

    let ui_dir = app.root.join("ui");
    if has_unit_tests(&ui_dir)? {
        println!("[wabou check] JavaScript unit tests");
        let mut tests = Command::new("bun");
        tests.current_dir(&app.root).args([
            "--conditions=browser",
            "test",
            ui_dir.to_string_lossy().as_ref(),
        ]);
        let status = tests
            .status()
            .map_err(|error| format!("failed to start Bun unit tests: {error}"))?;
        ensure(status, "JavaScript unit tests")?;
    }

    let manifest = app.root.join("Cargo.toml");
    if manifest.is_file() {
        println!("[wabou check] Rust {}", manifest.display());
        let mut cargo = Command::new("cargo");
        cargo
            .current_dir(&app.root)
            .args(["check", "--manifest-path"])
            .arg(&manifest)
            .arg("--all-targets");
        apply_cargo_features(&mut cargo, cargo_features);
        let status = cargo
            .status()
            .map_err(|error| format!("failed to start Cargo check: {error}"))?;
        ensure(status, &format!("Cargo check {}", manifest.display()))?;
    }

    if app.root.join("Cargo.toml").is_file()
        && let Some(target) = optional_app_bindings_target(workspace, app)?
    {
        println!("[wabou check] generated bindings");
        run_bindings_target(workspace, app, &target, "check", cargo_features)?;
    }

    let behavior_dir = app.root.join("tests");
    if !skip_behavior && has_behavior_scenarios(&behavior_dir)? {
        println!("[wabou check] behavior scenarios");
        test_scenario(
            workspace,
            app,
            &TestOptions {
                scenario: None,
                replay: None,
                replay_test: None,
                artifacts: None,
                mode: None,
                failure_screenshot: false,
                native: false,
                host_env: Vec::new(),
                cargo_features: cargo_features.to_vec(),
            },
        )?;
    }

    println!("[wabou check] all application checks passed");
    Ok(())
}

fn build(
    workspace: &Path,
    app: &App,
    release: bool,
    source_map_override: Option<bool>,
    cargo_features: &[String],
) -> Result<()> {
    let mut app = app.clone();
    let launch = app_launch_configuration(&mut app)?;
    let app = &app;
    let profile = BuildProfile::from_release(release);
    let source_map = source_map_override.unwrap_or(configured_source_map(app, profile)?);
    if let Some(mut config) = launch {
        ensure(
            build_frontend(workspace, app, &[], profile, source_map)?,
            "Vite build",
        )?;
        let bundle = bundle_path(workspace, app, profile)?;
        if config.resource_directory.is_none() {
            config.resource_directory = bundle.parent().map(Path::to_path_buf);
        }
        config.source = wabou_launch::UiSource::Bundle {
            path: bundle.clone(),
        };
        let configuration = bundle
            .parent()
            .ok_or("bundle has no parent")?
            .join("app.json");
        fs::write(&configuration, serde_json::to_vec_pretty(&config)?)?;
        println!(
            "[wabou] built CLI-hosted application: {}",
            configuration.display()
        );
        return Ok(());
    }
    let manifest = manifest(app);
    let mut cargo = Command::new("cargo");
    cargo
        .current_dir(workspace)
        .args(["build", "--manifest-path", &manifest]);
    apply_cargo_features(&mut cargo, cargo_features);
    if release {
        cargo.arg("--release");
    }
    ensure(cargo.status()?, "Cargo build")?;
    ensure(
        build_frontend(workspace, app, &[], profile, source_map)?,
        "Vite build",
    )?;
    package_executable(workspace, app, release)
}

fn package(
    workspace: &Path,
    app: &App,
    format_override: &[PackageFormat],
    target: Option<&str>,
    cargo_features: &[String],
) -> Result<()> {
    let config = load_package_config(app)?;
    if let Some(target) = target {
        build_zig_release(workspace, app, target, cargo_features)?;
    } else {
        build(workspace, app, true, None, cargo_features)?;
    }
    packaging::package_built_application(workspace, app, &config, format_override)
}

fn build_zig_release(
    workspace: &Path,
    app: &App,
    target: &str,
    cargo_features: &[String],
) -> Result<()> {
    if target.trim().is_empty() {
        return Err("package target cannot be empty".into());
    }
    let available = Command::new("cargo")
        .args(["zigbuild", "--help"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    if !available {
        return Err(
            "`wabou package --target` requires cargo-zigbuild; install it with `cargo install cargo-zigbuild --locked`"
                .into(),
        );
    }

    let manifest = manifest(app);
    let mut cargo = Command::new("cargo");
    cargo.current_dir(workspace).args([
        "zigbuild",
        "--manifest-path",
        &manifest,
        "--release",
        "--target",
        target,
    ]);
    apply_cargo_features(&mut cargo, cargo_features);
    ensure(cargo.status()?, "Cargo Zigbuild")?;
    ensure(
        build_frontend(workspace, app, &[], BuildProfile::Release, false)?,
        "Vite build",
    )?;
    package_executable_for_target(workspace, app, true, Some(target))
}

fn run(
    workspace: &Path,
    app: &App,
    release: bool,
    source_map_override: Option<bool>,
    profile_trace: Option<&Path>,
    hud: bool,
    cargo_features: &[String],
) -> Result<()> {
    let mut app = app.clone();
    let launch = app_launch_configuration(&mut app)?;
    let app = &app;
    let profile = BuildProfile::from_release(release);
    let source_map = source_map_override.unwrap_or(configured_source_map(app, profile)?);
    ensure(
        build_frontend(workspace, app, &[], profile, source_map)?,
        "Vite build",
    )?;
    if let Some(config) = launch {
        let (mut host, _configuration) =
            bundled_launch_command(workspace, app, config, profile, cargo_features)?;
        host.current_dir(workspace);
        if let Some(path) = profile_trace {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            host.env("WABOU_PROFILE_TRACE", path);
        }
        if hud {
            host.env("WABOU_PERFORMANCE_HUD", "1");
        }
        return ensure_host_exit(host.status()?);
    }
    let manifest = manifest(app);
    let binary = app_binary(workspace, app)?;
    let mut cargo = Command::new("cargo");
    cargo
        .current_dir(workspace)
        .args(["run", "--manifest-path", &manifest, "--bin", &binary]);
    if release {
        cargo.arg("--release");
    }
    apply_cargo_features(&mut cargo, cargo_features);
    if let Some(path) = profile_trace {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        cargo
            .args(["--features", &app_profiling_feature(workspace, app)?])
            .env("WABOU_PROFILE_TRACE", path);
    }
    cargo.env("WABOU_BUNDLE_PATH", bundle_path(workspace, app, profile)?);
    if hud {
        cargo.env("WABOU_PERFORMANCE_HUD", "1");
    }
    ensure_host_exit(cargo.status()?)
}

fn test_scenario(workspace: &Path, app: &App, options: &TestOptions) -> Result<()> {
    let mut app = app.clone();
    let launch = app_launch_configuration(&mut app)?;
    let app = &app;
    // The JavaScript runner derives a bounded suite budget from its registered
    // tests (currently capped at five minutes). Keep this watchdog outside that
    // budget so it remains a final process-level fallback instead of killing a
    // healthy large scenario before it can write a diagnostic report.
    const HOST_TIMEOUT: Duration = Duration::from_secs(310);
    let test_dir = workspace.join("target/wabou-test").join(&app.name);
    fs::create_dir_all(&test_dir)?;
    let artifact_dir = options
        .artifacts
        .clone()
        .unwrap_or_else(|| default_artifact_dir(&test_dir, options.replay.is_some()));
    prepare_artifact_dir(&artifact_dir)?;
    let generated_replay = test_dir.join("replay.ts");
    let scenario = if let Some(trace) = options.replay.as_deref() {
        // Validate replay artifacts before paying the cost of building either
        // the frontend or its Rust host.
        let parsed = replay_actions(trace, options.replay_test.as_deref())?;
        fs::write(
            &generated_replay,
            format!(
                "import {{ replay }} from {};\nreplay({});\n",
                serde_json::to_string(&behavior_test_runtime(workspace)?.to_string_lossy())?,
                serde_json::to_string(&parsed)?
            ),
        )?;
        generated_replay.clone()
    } else {
        let target = options
            .scenario
            .as_deref()
            .map_or_else(|| app.root.join("tests"), Path::to_path_buf);
        prepare_behavior_scenario(&target, &test_dir)?
    };
    if !scenario.is_file() {
        return Err(format!("test scenario {} does not exist", scenario.display()).into());
    }

    let mode_args = options.mode.as_deref().map(|mode| ["--mode", mode]);
    ensure(
        build_frontend(
            workspace,
            app,
            mode_args.as_ref().map_or(&[], |args| args),
            BuildProfile::Debug,
            true,
        )?,
        "Vite build",
    )?;

    let scenario_bundle = test_dir.join("scenario.js");
    ensure(
        build_test_script(workspace, app, &scenario, &scenario_bundle)?,
        "Vite test scenario build",
    )?;

    let test_data = tempfile::tempdir_in(&test_dir)?;
    let (mut host, _configuration) = if let Some(config) = launch {
        let (host, file) = bundled_launch_command(
            workspace,
            app,
            config,
            BuildProfile::Debug,
            &options.cargo_features,
        )?;
        (host, Some(file))
    } else {
        let manifest = manifest(app);
        let binary = app_binary(workspace, app)?;
        let mut features = options.cargo_features.clone();
        features.push(app_framework_feature(workspace, app, "devtools")?);
        if !options.native {
            features.push(app_framework_feature(workspace, app, "headless")?);
        }
        let executable = build_behavior_host(workspace, &manifest, &binary, &features)?;
        (Command::new(executable), None)
    };
    host.current_dir(workspace)
        .env(
            "WABOU_BUNDLE_PATH",
            bundle_path(workspace, app, BuildProfile::Debug)?,
        )
        .env("WABOU_TEST_SCRIPT", scenario_bundle)
        .env("WABOU_TEST_ARTIFACT_DIR", artifact_dir)
        .env("WABOU_TEST_RENDERER", "vello-hybrid")
        .env("WABOU_TEST_APP_DATA_ROOT", test_data.path())
        // Also isolate libraries that use the XDG convention directly rather
        // than resolving paths through Wabou's AppDirectories API.
        .env("XDG_CONFIG_HOME", test_data.path().join("xdg-config"))
        .env("XDG_DATA_HOME", test_data.path().join("xdg-data"))
        .env("XDG_CACHE_HOME", test_data.path().join("xdg-cache"));
    configure_test_backend(&mut host, options.native);
    for assignment in &options.host_env {
        let (name, value) = assignment
            .split_once('=')
            .ok_or_else(|| format!("invalid --env value {assignment:?}; expected KEY=VALUE"))?;
        if name.is_empty() || name.contains('=') || name.contains('\0') || value.contains('\0') {
            return Err(format!("invalid --env value {assignment:?}; expected KEY=VALUE").into());
        }
        host.env(name, value);
    }
    if options.failure_screenshot {
        host.env("WABOU_TEST_FAILURE_SCREENSHOT", "1");
    }
    let stopped = Arc::new(AtomicBool::new(false));
    let signal = stopped.clone();
    ctrlc::set_handler(move || signal.store(true, Ordering::Release))?;
    let status = wait_for_behavior_host(host, HOST_TIMEOUT, &stopped)?;
    ensure(status, "Wabou behavior test")
}

fn behavior_test_runtime(workspace: &Path) -> Result<PathBuf> {
    for path in [
        workspace.join("packages/test/src/index.ts"),
        workspace.join("vendor/wabou/packages/test/src/index.ts"),
    ] {
        if path.is_file() {
            return Ok(path);
        }
    }
    Err(format!(
        "cannot find @wabou/test source under {} or its vendor/wabou submodule",
        workspace.display()
    )
    .into())
}

fn is_wabou_app_directory(path: &Path) -> bool {
    path.is_dir()
        && path.join("package.json").is_file()
        && (path.join("Cargo.toml").is_file()
            || path.join("wabou.config.ts").is_file()
            || path.join("ui/index.tsx").is_file())
}

fn prepare_behavior_scenario(target: &Path, test_dir: &Path) -> Result<PathBuf> {
    if target.is_file() {
        return Ok(target.to_path_buf());
    }
    if !target.is_dir() {
        return Err(format!("test target {} does not exist", target.display()).into());
    }

    fn collect(directory: &Path, scenarios: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(directory)? {
            let path = entry?.path();
            if path.is_dir() {
                collect(&path, scenarios)?;
            } else if path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".behavior.ts"))
            {
                scenarios.push(path);
            }
        }
        Ok(())
    }

    let mut scenarios = Vec::new();
    collect(target, &mut scenarios)?;
    scenarios.sort();
    if scenarios.is_empty() {
        return Err(format!(
            "no *.behavior.ts scenarios found under {}",
            target.display()
        )
        .into());
    }

    let generated = test_dir.join("discovered-scenarios.ts");
    let source = scenarios
        .iter()
        .map(|scenario| {
            serde_json::to_string(&scenario.to_string_lossy())
                .map(|path| format!("import {path};\n"))
        })
        .collect::<std::result::Result<String, _>>()?;
    fs::write(&generated, source)?;
    Ok(generated)
}

fn has_behavior_scenarios(directory: &Path) -> Result<bool> {
    has_file_matching(directory, |name| name.ends_with(".behavior.ts"))
}

fn has_unit_tests(directory: &Path) -> Result<bool> {
    has_file_matching(directory, |name| {
        name.ends_with(".test.ts") || name.ends_with(".test.tsx")
    })
}

fn has_file_matching(directory: &Path, matches: impl Copy + Fn(&str) -> bool) -> Result<bool> {
    if !directory.is_dir() {
        return Ok(false);
    }
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            if has_file_matching(&path, matches)? {
                return Ok(true);
            }
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(matches)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn build_behavior_host(
    workspace: &Path,
    manifest: &str,
    binary: &str,
    features: &[String],
) -> Result<PathBuf> {
    let mut command = Command::new("cargo");
    command.current_dir(workspace).args([
        "build",
        "--manifest-path",
        manifest,
        "--bin",
        binary,
        "--message-format=json-render-diagnostics",
    ]);
    apply_cargo_features(&mut command, features);
    let output = command
        // Cargo progress remains visible while stdout is reserved for its
        // machine-readable artifact stream.
        .stderr(Stdio::inherit())
        .output()?;
    let executable = behavior_host_executable(&output.stdout, binary)?;
    ensure(output.status, "Wabou behavior host build")?;
    executable.ok_or_else(|| {
        format!("Cargo did not report an executable artifact for binary {binary:?}").into()
    })
}

fn behavior_host_executable(messages: &[u8], binary: &str) -> Result<Option<PathBuf>> {
    let mut executable = None;
    for (index, line) in messages.split(|byte| *byte == b'\n').enumerate() {
        if line.is_empty() {
            continue;
        }
        let message: Value = serde_json::from_slice(line).map_err(|error| {
            format!("invalid Cargo JSON message on line {}: {error}", index + 1)
        })?;
        if message["reason"] == "compiler-message"
            && let Some(rendered) = message["message"]["rendered"].as_str()
        {
            eprint!("{rendered}");
        }
        let is_binary = message["target"]["kind"]
            .as_array()
            .is_some_and(|kinds| kinds.iter().any(|kind| kind == "bin"));
        if message["reason"] == "compiler-artifact"
            && message["target"]["name"] == binary
            && is_binary
            && let Some(path) = message["executable"].as_str()
        {
            executable = Some(PathBuf::from(path));
        }
    }
    Ok(executable)
}

fn bindings(workspace: &Path, app: &App, mode: BindingsCommand) -> Result<()> {
    let target = app_bindings_target(workspace, app)?;
    let (mode, cargo_features) = match mode {
        BindingsCommand::Write { cargo_features, .. } => ("write", cargo_features),
        BindingsCommand::Check { cargo_features, .. } => ("check", cargo_features),
    };
    run_bindings_target(workspace, app, &target, mode, &cargo_features.values)
}

fn run_bindings_target(
    workspace: &Path,
    app: &App,
    target: &BindingsTarget,
    mode: &str,
    cargo_features: &[String],
) -> Result<()> {
    let manifest = manifest(app);
    let mut cargo = Command::new("cargo");
    cargo.current_dir(workspace).args([
        "run",
        "--quiet",
        "--manifest-path",
        &manifest,
        "--example",
        &target.name,
    ]);
    let mut features = target.required_features.clone();
    features.extend_from_slice(cargo_features);
    features.sort();
    features.dedup();
    apply_cargo_features(&mut cargo, &features);
    cargo.args(["--", mode]);
    ensure(cargo.status()?, "Wabou bindings generator")
}

fn load_launch_config(path: &Path) -> Result<wabou_launch::LaunchConfig> {
    let path = path.canonicalize()?;
    let mut config: wabou_launch::LaunchConfig = serde_json::from_slice(&fs::read(&path)?)?;
    config.resolve_paths(
        path.parent()
            .ok_or("configuration has no parent directory")?,
    );
    Ok(config)
}

fn app_launch_configuration(app: &mut App) -> Result<Option<wabou_launch::LaunchConfig>> {
    let configured = config::load_dev_config(app)?;
    if let Some(path) = configured.runtime_config {
        return load_launch_config(&path).map(Some);
    }
    if let Some(prepared) = ts_config::prepare(app)? {
        app.entry = prepared.entry;
        return load_launch_config(prepared.file.path()).map(Some);
    }
    Ok(None)
}

fn bundled_launch_command(
    workspace: &Path,
    app: &App,
    mut config: wabou_launch::LaunchConfig,
    profile: BuildProfile,
    features: &[String],
) -> Result<(Command, tempfile::NamedTempFile)> {
    let _ = features;
    let bundle = bundle_path(workspace, app, profile)?;
    if config.resource_directory.is_none() {
        config.resource_directory = bundle.parent().map(Path::to_path_buf);
    }
    config.source = wabou_launch::UiSource::Bundle { path: bundle };
    let file = tempfile::NamedTempFile::new()?;
    fs::write(file.path(), serde_json::to_vec(&config)?)?;
    let mut host = Command::new(env::current_exe()?);
    host.arg("launch").arg(file.path());
    Ok((host, file))
}

fn launch_application(configuration: &Path) -> Result<()> {
    #[cfg(feature = "embedded-renderer")]
    return wabou_runtime_library::run_config(load_launch_config(configuration)?)
        .map_err(Into::into);
    #[cfg(not(feature = "embedded-renderer"))]
    {
        let _ = configuration;
        Err("this CLI has no linked fallback host".into())
    }
}

struct DevOptions<'a> {
    port: u16,
    open_devtools: bool,
    hud: bool,
    runtime_config: Option<&'a Path>,
    mode: Option<&'a str>,
    cargo_features: &'a [String],
}

fn dev(workspace: &Path, app: App, options: DevOptions<'_>) -> Result<()> {
    ensure_javascript_dependencies(workspace, &app)?;
    ensure_workspace_package_exports(workspace)?;
    let vite_configuration = frontend::default_vite_config(&app)?;
    let port_text = options.port.to_string();
    let mut vite_command = Command::new("bun");
    vite_command
        .current_dir(&app.frontend)
        .args([
            "x",
            "vite",
            "--host",
            "127.0.0.1",
            "--port",
            &port_text,
            "--strictPort",
        ])
        .stdin(Stdio::null());
    if let Some(mode) = options.mode {
        vite_command.args(["--mode", mode]);
    }
    vite_command.env("WABOU_VITE_ENTRY", &app.entry);
    if let Some(configuration) = &vite_configuration {
        vite_command.arg("--config").arg(configuration.path());
    }
    let mut vite = ManagedChild::spawn(vite_command)?;
    let url = format!("http://127.0.0.1:{}", options.port);
    wait_for_vite(&url, vite.child.as_mut())?;

    // Keep the generated configuration alive for the supervised host process.
    let mut runtime_configuration = None;
    let mut host_command = if let Some(configuration) = options.runtime_config {
        if !options.cargo_features.is_empty() {
            return Err("--features applies to a linked host; configure module features in the runtime JSON".into());
        }
        let mut config = load_launch_config(configuration)?;
        config.source = wabou_launch::UiSource::Vite {
            url: url.clone(),
            entry: app.entry.clone(),
        };
        let file = tempfile::NamedTempFile::new()?;
        serde_json::to_writer(file.as_file(), &config)?;
        let mut command = Command::new(env::current_exe()?);
        command.arg("launch").arg(file.path());
        runtime_configuration = Some(file);
        command
    } else {
        let binary = app_binary(workspace, &app)?;
        let dev_features = app_dev_features(workspace, &app)?;
        let app_manifest = manifest(&app);
        let mut command = Command::new("cargo");
        command.current_dir(workspace).args([
            "run",
            "--manifest-path",
            &app_manifest,
            "--bin",
            &binary,
            "--features",
            &dev_features,
        ]);
        command
            .env("WABOU_VITE_URL", &url)
            .env("WABOU_VITE_ENTRY", &app.entry);
        apply_cargo_features(&mut command, options.cargo_features);
        command
    };
    if options.hud {
        host_command.env("WABOU_PERFORMANCE_HUD", "1");
    }
    let mut host = ManagedChild::spawn(host_command)?;

    let mut inspector = if options.open_devtools {
        let command = devtools::command(workspace)?;
        Some(ManagedChild::spawn(command)?)
    } else {
        None
    };

    println!("[wabou] dev server ready at {url}; press Ctrl-C to stop");
    let result = supervise(&mut host, &mut vite, inspector.as_mut());
    drop(runtime_configuration);
    result
}

fn package_executable(workspace: &Path, app: &App, release: bool) -> Result<()> {
    package_executable_for_target(workspace, app, release, None)
}

fn package_executable_for_target(
    workspace: &Path,
    app: &App,
    release: bool,
    target: Option<&str>,
) -> Result<()> {
    let metadata = cargo_metadata(workspace, app)?;
    let manifest_path = app.root.join("Cargo.toml").canonicalize()?;
    let (source, binary) =
        artifact_from_metadata_for_target(&metadata, &manifest_path, release, target)?;
    let destination_dir =
        profile_application_dir(workspace, app, BuildProfile::from_release(release))?;
    fs::create_dir_all(&destination_dir)?;
    let destination = destination_dir.join(&binary);
    fs::copy(&source, &destination).map_err(|error| {
        format!(
            "failed to package {} as {}: {error}",
            source.display(),
            destination.display()
        )
    })?;
    println!("[wabou] packaged {}", destination.display());
    Ok(())
}

fn ensure(status: ExitStatus, label: &str) -> Result<()> {
    if status.success() {
        Ok(())
    } else {
        Err(format!("{label} failed with {status}").into())
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    fn parse<'a>(
        arguments: &'a [&'a str],
    ) -> std::result::Result<Cli, usage_rs::Error<'static, 'a>> {
        let arguments = arguments.iter().map(OsStr::new).collect::<Vec<_>>();
        Cli::try_parse_from(&arguments)
    }

    #[test]
    fn embedded_launch_and_bundle_commands() {
        assert!(matches!(
            parse(&["wabou", "launch", "app.json"]).unwrap().command,
            Commands::Launch { .. }
        ));
        assert!(matches!(
            parse(&[
                "wabou",
                "bundle",
                "app",
                "--config",
                "app/app.json",
                "--out",
                "dist"
            ])
            .unwrap()
            .command,
            Commands::Bundle { .. }
        ));
    }
    #[test]
    fn fallback_host_config_can_override_project_configuration() {
        assert!(parse(&["wabou", "dev", "--runtime-config", "app.json"]).is_ok());
        let parsed = parse(&["wabou", "dev", "--runtime-config", "app.json"]).unwrap();
        assert!(matches!(
            parsed.command,
            Commands::Dev {
                runtime_config: Some(_),
                ..
            }
        ));
    }
    #[test]
    fn launch_configuration_resolves_bundle_paths_from_its_directory() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("app.json");
        std::fs::write(
            &path,
            r#"{
            "version":1,"title":"Example","width":800,"height":600,
            "source":{"kind":"bundle","path":"bundle.js"}
        }"#,
        )
        .unwrap();
        let config = super::load_launch_config(&path).unwrap();
        let root = directory.path().canonicalize().unwrap();
        let wabou_launch::UiSource::Bundle { path } = config.source else {
            panic!("expected bundle")
        };
        assert_eq!(path, root.join("bundle.js"));
    }
    #[cfg(unix)]
    use std::thread;
    #[cfg(unix)]
    use std::time::Instant;

    use crate::render::RenderAction;

    use super::*;

    #[test]
    fn parses_doctor_application_path() {
        let Cli {
            command: Commands::Doctor { app },
        } = parse(&["wabou", "doctor", "apps/gallery"]).unwrap()
        else {
            panic!("expected doctor command");
        };
        assert_eq!(app.as_deref(), Some(Path::new("apps/gallery")));
    }

    #[test]
    fn parses_application_check_options() {
        let Cli {
            command: Commands::Check {
                app, skip_behavior, ..
            },
        } = parse(&["wabou", "check", "apps/gallery", "--skip-behavior"]).unwrap()
        else {
            panic!("expected check command");
        };
        assert_eq!(app.as_deref(), Some(Path::new("apps/gallery")));
        assert!(skip_behavior);
    }

    #[test]
    fn parses_clean_scope() {
        let Cli {
            command: Commands::Clean { app, packages },
        } = parse(&["wabou", "clean", "apps/gallery", "--packages"]).unwrap()
        else {
            panic!("expected clean command");
        };
        assert_eq!(app.as_deref(), Some(Path::new("apps/gallery")));
        assert!(packages);
    }

    #[test]
    fn parses_explicit_behavior_host_environment() {
        let Cli {
            command: Commands::Test { host_env, .. },
        } = parse(&[
            "wabou",
            "test",
            "apps/pi-agent",
            "--env",
            "WABOU_PI_BIN=/tmp/fake-pi",
            "--env",
            "PI_TEST_MODE=deterministic",
        ])
        .unwrap()
        else {
            panic!("expected test command");
        };
        assert_eq!(
            host_env,
            ["WABOU_PI_BIN=/tmp/fake-pi", "PI_TEST_MODE=deterministic"]
        );
    }

    #[test]
    fn parses_and_forwards_repeated_application_features() {
        let Cli {
            command: Commands::Run { cargo_features, .. },
        } = parse(&[
            "wabou",
            "run",
            "apps/gallery",
            "--features",
            "diagnostics,telemetry",
            "--features",
            "experimental",
        ])
        .unwrap()
        else {
            panic!("expected run command");
        };
        assert_eq!(
            cargo_features.values,
            ["diagnostics", "telemetry", "experimental"]
        );

        let mut command = Command::new("cargo");
        apply_cargo_features(&mut command, &cargo_features.values);
        assert_eq!(
            command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            ["--features", "diagnostics,telemetry,experimental"]
        );
    }

    #[test]
    fn parses_new_project_git_source() {
        let Cli {
            command:
                Commands::New {
                    path,
                    wabou_repository,
                    wabou_ref,
                },
        } = parse(&[
            "wabou",
            "new",
            "hello-wabou",
            "--wabou-repository",
            "/tmp/wabou",
            "--wabou-ref",
            "abc123",
        ])
        .unwrap()
        else {
            panic!("expected new command");
        };
        assert_eq!(path, Path::new("hello-wabou"));
        assert_eq!(wabou_repository, "/tmp/wabou");
        assert_eq!(wabou_ref, "abc123");

        let Cli {
            command:
                Commands::New {
                    wabou_repository,
                    wabou_ref,
                    ..
                },
        } = parse(&["wabou", "new", "hello-wabou"]).unwrap()
        else {
            panic!("expected new command");
        };
        assert_eq!(wabou_repository, "https://github.com/SunDoge/wabou.git");
        assert_eq!(wabou_ref, "v0.1.0-alpha.4");
    }

    #[test]
    fn parses_vendor_submodule_source() {
        let Cli {
            command:
                Commands::Vendor {
                    app,
                    wabou_repository,
                    wabou_ref,
                },
        } = parse(&[
            "wabou",
            "vendor",
            "apps/desktop",
            "--wabou-repository",
            "/tmp/wabou",
            "--wabou-ref",
            "abc123",
        ])
        .unwrap()
        else {
            panic!("expected vendor command");
        };
        assert_eq!(app.as_deref(), Some(Path::new("apps/desktop")));
        assert_eq!(wabou_repository, "/tmp/wabou");
        assert_eq!(wabou_ref, "abc123");

        let Cli {
            command:
                Commands::Vendor {
                    app,
                    wabou_repository,
                    wabou_ref,
                },
        } = parse(&["wabou", "vendor"]).unwrap()
        else {
            panic!("expected vendor command");
        };
        assert_eq!(app, None);
        assert_eq!(wabou_repository, "https://github.com/SunDoge/wabou.git");
        assert_eq!(wabou_ref, "v0.1.0-alpha.4");
    }

    #[cfg(unix)]
    #[test]
    fn managed_child_terminates_descendants_after_group_leader_exits() {
        use std::io::{BufRead, BufReader};

        let mut command = Command::new("sh");
        command
            .args(["-c", "sleep 30 >/dev/null 2>&1 & echo $!"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut managed = ManagedChild::spawn(command).unwrap();
        let mut pid = String::new();
        BufReader::new(managed.child.stdout().take().unwrap())
            .read_line(&mut pid)
            .unwrap();
        managed.child.wait().unwrap();
        let is_alive = || {
            Command::new("kill")
                .args(["-0", pid.trim()])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .unwrap()
                .success()
        };
        assert!(is_alive());
        managed.terminate();
        // SIGKILL delivery and orphan reaping are asynchronous. `kill(2)`
        // returning successfully does not guarantee that `kill -0` has
        // stopped observing the descendant in the same scheduling turn.
        let deadline = Instant::now() + Duration::from_secs(1);
        while is_alive() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(!is_alive());
    }
    #[test]
    fn render_defaults_to_the_main_logical_window_and_accepts_an_override() {
        let Cli {
            command:
                Commands::Render {
                    window_id,
                    scale_factor,
                    color_scheme,
                    mode,
                    fixture,
                    skip_build,
                    with_host,
                    scenario,
                    wait_ms,
                    click,
                    wheel,
                    key,
                    ..
                },
        } = parse(&["wabou", "render", "--out", "capture.png"]).unwrap()
        else {
            panic!("expected render command");
        };
        assert_eq!(window_id, 1);
        assert_eq!(scale_factor, 1.0);
        assert_eq!(color_scheme, HeadlessColorScheme::Light);
        assert_eq!(mode, None);
        assert_eq!(fixture, None);
        assert!(!skip_build);
        assert!(!with_host);
        assert_eq!(scenario, None);
        assert_eq!(wait_ms, 1_000);
        assert!(click.is_empty());
        assert!(wheel.is_empty());
        assert!(key.is_empty());

        let Cli {
            command:
                Commands::Render {
                    window_id,
                    scale_factor,
                    color_scheme,
                    mode,
                    fixture,
                    skip_build,
                    with_host,
                    scenario,
                    wait_ms,
                    click,
                    wheel,
                    key,
                    ..
                },
        } = parse(&[
            "wabou",
            "render",
            "--out",
            "capture.png",
            "--window-id",
            "7",
            "--scale-factor",
            "2",
            "--color-scheme",
            "dark",
            "--mode",
            "ui-test",
            "--skip-build",
            "--with-host",
            "--scenario",
            "captures/downloads.ts",
            "--wait-ms",
            "250",
            "--click",
            "10",
            "20",
            "--click",
            "30",
            "40",
            "--wheel",
            "100",
            "200",
            "0",
            "360",
            "--key",
            "Enter",
            "--key",
            "Escape",
        ])
        .unwrap()
        else {
            panic!("expected render command");
        };
        assert_eq!(window_id, 7);
        assert_eq!(scale_factor, 2.0);
        assert_eq!(color_scheme, HeadlessColorScheme::Dark);
        assert_eq!(mode.as_deref(), Some("ui-test"));
        assert_eq!(fixture, None);
        assert!(skip_build);
        assert!(with_host);
        assert_eq!(scenario, Some(PathBuf::from("captures/downloads.ts")));
        assert_eq!(wait_ms, 250);
        assert_eq!(click, [10.0, 20.0, 30.0, 40.0]);
        assert_eq!(wheel, [100.0, 200.0, 0.0, 360.0]);
        assert_eq!(key, ["Enter", "Escape"]);
    }

    #[test]
    fn render_accepts_a_named_layout_fixture_and_rejects_viewport_overrides() {
        let Cli {
            command: Commands::Render { fixture, .. },
        } = parse(&[
            "wabou",
            "render",
            "apps/pi-agent",
            "--out",
            "fixture.png",
            "--fixture",
            "conversation/complete-turn",
        ])
        .unwrap()
        else {
            panic!("expected render command");
        };
        assert_eq!(fixture.as_deref(), Some("conversation/complete-turn"));

        assert!(
            parse(&[
                "wabou",
                "render",
                "apps/pi-agent",
                "--out",
                "fixture.png",
                "--fixture",
                "conversation/complete-turn",
                "--width",
                "800",
            ])
            .is_err()
        );
    }

    #[test]
    fn render_accepts_a_debug_snapshot_with_or_without_host() {
        let Cli {
            command: Commands::Render { snapshot, .. },
        } = parse(&[
            "wabou",
            "render",
            "apps/gallery",
            "--out",
            "capture.png",
            "--snapshot",
            "tree.json",
        ])
        .unwrap()
        else {
            panic!("expected render command");
        };
        assert_eq!(snapshot, Some(PathBuf::from("tree.json")));

        let Cli {
            command:
                Commands::Render {
                    snapshot,
                    with_host,
                    ..
                },
        } = parse(&[
            "wabou",
            "render",
            "apps/gallery",
            "--out",
            "capture.png",
            "--snapshot",
            "tree.json",
            "--with-host",
        ])
        .unwrap()
        else {
            panic!("expected render command");
        };
        assert!(with_host);
        assert_eq!(snapshot, Some(PathBuf::from("tree.json")));
    }

    #[test]
    fn layout_defaults_to_a_fast_snapshot_without_render_options() {
        let Cli {
            command:
                Commands::Layout {
                    out,
                    width,
                    height,
                    scale_factor,
                    wait_ms,
                    ..
                },
        } = parse(&["wabou", "layout", "apps/gallery", "--out", "layout.json"]).unwrap()
        else {
            panic!("expected layout command");
        };
        assert_eq!(out, PathBuf::from("layout.json"));
        assert_eq!((width, height), (1440, 900));
        assert_eq!(scale_factor, 1.0);
        assert_eq!(wait_ms, 0);
    }

    #[test]
    fn layout_accepts_a_single_runtime_fixture_batch() {
        let Cli {
            command: Commands::Layout { batch, mode, .. },
        } = parse(&[
            "wabou",
            "layout",
            "apps/gallery",
            "--out",
            "report.json",
            "--batch",
            "fixtures.json",
            "--mode",
            "layout-test",
        ])
        .unwrap()
        else {
            panic!("expected layout command");
        };
        assert_eq!(batch, Some(PathBuf::from("fixtures.json")));
        assert_eq!(mode.as_deref(), Some("layout-test"));
    }

    #[test]
    fn parses_bindings_write_and_check_commands() {
        for name in ["write", "check"] {
            let Cli {
                command: Commands::Bindings { command },
            } = parse(&["wabou", "bindings", name, "apps/gallery"]).unwrap()
            else {
                panic!("expected bindings command");
            };
            match command {
                BindingsCommand::Write { app, .. } if name == "write" => {
                    assert_eq!(app.as_deref(), Some(Path::new("apps/gallery")));
                }
                BindingsCommand::Check { app, .. } if name == "check" => {
                    assert_eq!(app.as_deref(), Some(Path::new("apps/gallery")));
                }
                _ => panic!("unexpected bindings command"),
            }
        }
    }

    #[test]
    fn parses_behavior_test_scenarios_and_replays() {
        let Cli {
            command:
                Commands::Test {
                    scenario,
                    app,
                    replay,
                    replay_test,
                    artifacts,
                    mode,
                    failure_screenshot,
                    native,
                    ..
                },
        } = parse(&[
            "wabou",
            "test",
            "--app",
            "apps/warden-desktop",
            "tests/close-to-tray.test.ts",
            "--artifacts",
            "artifacts",
            "--mode",
            "ui-test",
            "--failure-screenshot",
            "--native",
        ])
        .unwrap()
        else {
            panic!("expected test command");
        };
        assert_eq!(app.as_deref(), Some(Path::new("apps/warden-desktop")));
        assert_eq!(
            scenario.as_deref(),
            Some(Path::new("tests/close-to-tray.test.ts"))
        );
        assert!(replay.is_none());
        assert!(replay_test.is_none());
        assert_eq!(artifacts.as_deref(), Some(Path::new("artifacts")));
        assert_eq!(mode.as_deref(), Some("ui-test"));
        assert!(failure_screenshot);
        assert!(native);

        let Cli {
            command:
                Commands::Test {
                    scenario,
                    replay,
                    replay_test,
                    native,
                    ..
                },
        } = parse(&[
            "wabou",
            "test",
            "--replay",
            "report.json",
            "--replay-test",
            "submits form",
        ])
        .unwrap()
        else {
            panic!("expected test replay command");
        };
        assert!(scenario.is_none());
        assert_eq!(replay.as_deref(), Some(Path::new("report.json")));
        assert_eq!(replay_test.as_deref(), Some("submits form"));
        assert!(!native);
        assert!(parse(&["wabou", "test", "--replay-test", "orphan"]).is_err());

        let Cli {
            command: Commands::Test { scenario, .. },
        } = parse(&["wabou", "test"]).unwrap()
        else {
            panic!("expected test command");
        };
        assert!(scenario.is_none());
    }

    #[test]
    fn infers_the_application_owning_a_behavior_scenario() {
        let root = tempfile::tempdir().unwrap();
        let app = root.path().join("apps/motrix");
        let tests = app.join("tests/nested");
        fs::create_dir_all(&tests).unwrap();
        fs::write(app.join("Cargo.toml"), "[package]\nname = \"motrix\"\n").unwrap();
        fs::write(app.join("package.json"), "{}").unwrap();
        let scenario = tests.join("navigation.behavior.ts");
        fs::write(&scenario, "").unwrap();

        let (resolved_app, resolved_scenario) =
            resolve_behavior_test_target(None, Some(scenario.clone()));
        assert_eq!(resolved_app.as_deref(), Some(app.as_path()));
        assert_eq!(resolved_scenario.as_deref(), Some(scenario.as_path()));

        let explicit = root.path().join("apps/other");
        let (resolved_app, resolved_scenario) =
            resolve_behavior_test_target(Some(&explicit), Some(scenario.clone()));
        assert_eq!(resolved_app.as_deref(), Some(explicit.as_path()));
        assert_eq!(resolved_scenario.as_deref(), Some(scenario.as_path()));

        let (resolved_app, resolved_scenario) =
            resolve_behavior_test_target(None, Some(app.clone()));
        assert_eq!(resolved_app.as_deref(), Some(app.as_path()));
        assert!(resolved_scenario.is_none());
    }

    #[test]
    fn discovers_behavior_scenarios_recursively_without_an_aggregate_entry() {
        let root = tempfile::tempdir().unwrap();
        let tests = root.path().join("tests");
        let nested = tests.join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(tests.join("behavior.ts"), "import './one.behavior';").unwrap();
        fs::write(tests.join("one.behavior.ts"), "test('one', () => {});").unwrap();
        fs::write(nested.join("two.behavior.ts"), "test('two', () => {});").unwrap();
        fs::write(nested.join("helper.ts"), "export const helper = true;").unwrap();

        let generated = prepare_behavior_scenario(&tests, root.path()).unwrap();
        let source = fs::read_to_string(generated).unwrap();
        assert_eq!(source.matches("import ").count(), 2);
        assert!(source.contains("one.behavior.ts"));
        assert!(source.contains("two.behavior.ts"));
        assert!(!source.contains(
            &serde_json::to_string(&tests.join("behavior.ts").to_string_lossy()).unwrap()
        ));
        assert!(!source.contains("helper.ts"));
    }

    #[test]
    fn detects_optional_behavior_scenarios_recursively() {
        let temp = tempfile::tempdir().unwrap();
        let nested = temp.path().join("nested");
        fs::create_dir(&nested).unwrap();
        assert!(!has_behavior_scenarios(temp.path()).unwrap());
        fs::write(nested.join("input.behavior.ts"), "").unwrap();
        assert!(has_behavior_scenarios(temp.path()).unwrap());
    }

    #[test]
    fn detects_application_unit_tests_without_confusing_behavior_scenarios() {
        let temp = tempfile::tempdir().unwrap();
        let nested = temp.path().join("components");
        fs::create_dir(&nested).unwrap();
        fs::write(nested.join("input.behavior.ts"), "").unwrap();
        assert!(!has_unit_tests(temp.path()).unwrap());
        fs::write(nested.join("input.test.tsx"), "").unwrap();
        assert!(has_unit_tests(temp.path()).unwrap());
    }

    #[test]
    fn resolves_behavior_runtime_in_workspace_and_vendored_projects() {
        for relative in [
            "packages/test/src/index.ts",
            "vendor/wabou/packages/test/src/index.ts",
        ] {
            let temp = tempfile::tempdir().unwrap();
            let runtime = temp.path().join(relative);
            fs::create_dir_all(runtime.parent().unwrap()).unwrap();
            fs::write(&runtime, "").unwrap();
            assert_eq!(behavior_test_runtime(temp.path()).unwrap(), runtime);
        }
    }

    #[test]
    fn native_test_backend_removes_an_inherited_headless_override() {
        let mut command = Command::new("wabou-test-host");
        command.env("WABOU_TEST_HEADLESS", "inherited");
        configure_test_backend(&mut command, true);
        assert_eq!(
            command
                .get_envs()
                .find(|(key, _)| *key == "WABOU_TEST_HEADLESS")
                .and_then(|(_, value)| value),
            None
        );

        configure_test_backend(&mut command, false);
        assert_eq!(
            command
                .get_envs()
                .find(|(key, _)| *key == "WABOU_TEST_HEADLESS")
                .and_then(|(_, value)| value),
            Some(std::ffi::OsStr::new("1"))
        );
    }

    #[cfg(unix)]
    #[test]
    fn behavior_host_watchdog_terminates_the_managed_process_group() {
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 30"]);
        let started = Instant::now();
        let error =
            wait_for_managed_child(command, Duration::from_millis(20), &AtomicBool::new(false))
                .unwrap_err();

        assert!(error.to_string().contains("final"));
        assert!(error.to_string().contains("watchdog"));
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[cfg(unix)]
    #[test]
    fn behavior_host_wait_preserves_the_child_exit_status() {
        let mut command = Command::new("sh");
        command.args(["-c", "exit 7"]);
        let status =
            wait_for_managed_child(command, Duration::from_secs(1), &AtomicBool::new(false))
                .unwrap();

        assert_eq!(status.code(), Some(7));
    }

    #[test]
    fn behavior_host_uses_cargos_authoritative_executable_artifact() {
        let messages = br#"{"reason":"compiler-artifact","target":{"name":"helper","kind":["bin"]},"executable":"/custom/target/debug/helper"}
{"reason":"compiler-artifact","target":{"name":"gallery","kind":["lib"]},"executable":null}
{"reason":"compiler-artifact","target":{"name":"gallery","kind":["bin"]},"executable":"/custom/target/aarch64-unknown-linux-gnu/debug/gallery"}
{"reason":"build-finished","success":true}
"#;

        assert_eq!(
            behavior_host_executable(messages, "gallery").unwrap(),
            Some(PathBuf::from(
                "/custom/target/aarch64-unknown-linux-gnu/debug/gallery"
            ))
        );
        assert_eq!(behavior_host_executable(messages, "missing").unwrap(), None);
        assert!(
            behavior_host_executable(b"not json\n", "gallery")
                .unwrap_err()
                .to_string()
                .contains("line 1")
        );
    }

    #[test]
    fn parses_native_package_format_overrides() {
        let Cli {
            command:
                Commands::Package {
                    app,
                    format,
                    target,
                    ..
                },
        } = parse(&[
            "wabou",
            "package",
            "apps/warden-desktop",
            "--format",
            "appimage",
            "--format",
            "deb",
        ])
        .unwrap()
        else {
            panic!("expected package command");
        };
        assert_eq!(app.as_deref(), Some(Path::new("apps/warden-desktop")));
        assert_eq!(format, [PackageFormat::Appimage, PackageFormat::Deb]);
        assert_eq!(target, None);
    }

    #[test]
    fn parses_zig_package_target() {
        let Cli {
            command: Commands::Package { target, .. },
        } = parse(&[
            "wabou",
            "package",
            "apps/gallery",
            "--target",
            "x86_64-unknown-linux-gnu.2.28",
        ])
        .unwrap()
        else {
            panic!("expected package command");
        };
        assert_eq!(target.as_deref(), Some("x86_64-unknown-linux-gnu.2.28"));
    }

    #[test]
    fn rejects_the_removed_app_dir_flag() {
        assert!(parse(&["wabou", "run", "--app-dir", "apps/gallery"]).is_err());
    }

    #[test]
    fn package_configuration_is_strict_and_app_owned() {
        let root = env::temp_dir().join(format!("wabou-cli-package-config-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("wabou.toml"),
            r#"[package]
product-name = "Example"
identifier = "dev.wabou.example"
resources = ["assets"]
formats = ["deb"]

[build]
out-dir = "dist/resources"
"#,
        )
        .unwrap();
        let app = App {
            name: "example".into(),
            root: root.clone(),
            frontend: root.clone(),
            entry: "ui/index.tsx".into(),
        };
        let config = load_package_config(&app).unwrap();
        assert_eq!(config.product_name, "Example");
        assert_eq!(config.formats, [PackageFormat::Deb]);
        assert_eq!(config.resources, [PathBuf::from("assets")]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn finds_an_app_manifest_above_its_ui_directory() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/gallery");
        assert_eq!(find_app_root(&root.join("ui")), Some(root));
    }

    #[test]
    fn resolves_app_dir_relative_to_the_calling_directory() {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let app = load_app(&workspace, &workspace, Some(Path::new("apps/gallery"))).unwrap();
        assert_eq!(app.root, workspace.join("apps/gallery"));
        assert_eq!(app.entry, "ui/index.tsx");
    }

    #[test]
    fn standalone_package_is_its_own_workspace_root() {
        let root = env::temp_dir().join(format!("wabou-cli-standalone-{}", std::process::id()));
        let ui = root.join("ui");
        fs::create_dir_all(&ui).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"standalone\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        assert_eq!(find_workspace(&ui).unwrap(), root);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn derives_the_binary_artifact_from_cargo_metadata() {
        let metadata = serde_json::json!({
            "target_directory": "/workspace/target",
            "packages": [{
                "manifest_path": "/workspace/apps/gallery/Cargo.toml",
                "targets": [{"name": "gallery-host", "kind": ["bin"]}]
            }]
        });
        let (debug, name) = artifact_from_metadata(
            &metadata,
            Path::new("/workspace/apps/gallery/Cargo.toml"),
            false,
        )
        .unwrap();
        assert_eq!(
            debug,
            Path::new("/workspace/target/debug")
                .join(format!("gallery-host{}", env::consts::EXE_SUFFIX))
        );
        assert_eq!(name, format!("gallery-host{}", env::consts::EXE_SUFFIX));

        let (release, _) = artifact_from_metadata(
            &metadata,
            Path::new("/workspace/apps/gallery/Cargo.toml"),
            true,
        )
        .unwrap();
        assert!(release.starts_with("/workspace/target/release"));

        let (zig_release, _) = artifact_from_metadata_for_target(
            &metadata,
            Path::new("/workspace/apps/gallery/Cargo.toml"),
            true,
            Some("x86_64-unknown-linux-gnu.2.28"),
        )
        .unwrap();
        assert_eq!(
            zig_release,
            Path::new("/workspace/target/x86_64-unknown-linux-gnu/release")
                .join(format!("gallery-host{}", env::consts::EXE_SUFFIX))
        );
    }

    #[test]
    fn discovers_bindings_generator_by_source_instead_of_global_target_name() {
        let metadata = serde_json::json!({
            "packages": [{
                "manifest_path": "/workspace/apps/gallery/Cargo.toml",
                "targets": [
                    {
                        "name": "gallery",
                        "kind": ["bin"],
                        "src_path": "/workspace/apps/gallery/src/main.rs"
                    },
                    {
                        "name": "gallery-bindgen",
                        "kind": ["example"],
                        "src_path": "/workspace/apps/gallery/examples/wabou-bindgen.rs"
                    }
                ]
            }]
        });
        assert_eq!(
            bindings_target(&metadata, Path::new("/workspace/apps/gallery/Cargo.toml"))
                .and_then(|target| target["name"].as_str()),
            Some("gallery-bindgen")
        );
    }

    #[test]
    fn selects_facade_development_features_for_new_apps() {
        let metadata = serde_json::json!({
            "packages": [{
                "manifest_path": "/workspace/apps/gallery/Cargo.toml",
                "dependencies": [{"name": "wabou"}]
            }]
        });
        assert_eq!(
            dev_features(&metadata, Path::new("/workspace/apps/gallery/Cargo.toml")),
            Some("wabou/vite,wabou/devtools".into())
        );
    }

    #[test]
    fn selects_the_facade_profiling_feature_for_new_apps() {
        let metadata = serde_json::json!({
            "packages": [{
                "manifest_path": "/workspace/apps/gallery/Cargo.toml",
                "dependencies": [{"name": "wabou"}]
            }]
        });
        assert_eq!(
            framework_feature(
                &metadata,
                Path::new("/workspace/apps/gallery/Cargo.toml"),
                "profiling"
            ),
            Some("wabou/profiling".into())
        );
    }

    #[test]
    fn parses_an_explicit_profile_trace_path() {
        let Cli {
            command:
                Commands::Run {
                    profile_trace,
                    release,
                    ..
                },
        } = parse(&["wabou", "run", "--release", "--profile-trace", "trace.json"]).unwrap()
        else {
            panic!("expected run command");
        };
        assert!(release);
        assert_eq!(profile_trace.as_deref(), Some(Path::new("trace.json")));
    }

    #[test]
    fn parses_native_performance_hud_for_dev_and_run() {
        let Cli {
            command: Commands::Dev { hud: dev_hud, .. },
        } = parse(&["wabou", "dev", "--hud"]).unwrap()
        else {
            panic!("expected dev command");
        };
        let Cli {
            command: Commands::Run { hud: run_hud, .. },
        } = parse(&["wabou", "run", "--hud"]).unwrap()
        else {
            panic!("expected run command");
        };
        assert!(dev_hud);
        assert!(run_hud);
    }

    #[test]
    fn parses_source_map_overrides() {
        let Cli {
            command:
                Commands::Build {
                    source_map,
                    release,
                    ..
                },
        } = parse(&["wabou", "build", "--release", "--source-map"]).unwrap()
        else {
            panic!("expected build command");
        };
        assert!(release);
        assert_eq!(source_map, Some(true));

        let Cli {
            command: Commands::Build { source_map, .. },
        } = parse(&["wabou", "build", "--source-map=false"]).unwrap()
        else {
            panic!("expected build command");
        };
        assert_eq!(source_map, Some(false));
    }

    #[test]
    fn selects_direct_runtime_development_features() {
        let metadata = serde_json::json!({
            "packages": [{
                "manifest_path": "/workspace/apps/runtime/Cargo.toml",
                "dependencies": [{"name": "wabou-runtime"}]
            }]
        });
        assert_eq!(
            dev_features(&metadata, Path::new("/workspace/apps/runtime/Cargo.toml")),
            Some("wabou-runtime/vite,wabou-runtime/devtools".into())
        );
    }

    #[test]
    fn selects_the_package_named_binary_when_helpers_exist() {
        let metadata = serde_json::json!({
            "packages": [{
                "name": "warden-desktop",
                "manifest_path": "/workspace/apps/warden-desktop/Cargo.toml",
                "targets": [
                    {"name": "warden-live-crud", "kind": ["bin"]},
                    {"name": "warden-desktop", "kind": ["bin"]}
                ]
            }]
        });
        let target = binary_target(
            &metadata,
            Path::new("/workspace/apps/warden-desktop/Cargo.toml"),
        )
        .unwrap();
        assert_eq!(target["name"], "warden-desktop");
    }

    #[test]
    fn uses_the_conventional_dist_resource_path() {
        let workspace = Path::new("/workspace");
        let app = App {
            name: "gallery".into(),
            root: workspace.join("apps/gallery"),
            frontend: workspace.join("apps/gallery"),
            entry: "ui/index.tsx".into(),
        };
        assert_eq!(
            bundle_path(workspace, &app, BuildProfile::Debug).unwrap(),
            Path::new("/workspace/dist/gallery/debug/resources/bundle.js")
        );
        assert_eq!(
            bundle_path(workspace, &app, BuildProfile::Release).unwrap(),
            Path::new("/workspace/dist/gallery/release/resources/bundle.js")
        );
    }

    #[test]
    fn standalone_app_uses_a_direct_dist_resource_path() {
        let root = Path::new("/workspace/warden-desktop");
        let app = App {
            name: "warden-desktop".into(),
            root: root.into(),
            frontend: root.into(),
            entry: "ui/index.tsx".into(),
        };
        assert_eq!(
            bundle_path(root, &app, BuildProfile::Debug).unwrap(),
            Path::new("/workspace/warden-desktop/dist/debug/resources/bundle.js")
        );
    }

    #[test]
    fn workspace_package_preflight_reports_interrupted_dist_builds() {
        let root = tempfile::tempdir().unwrap();
        let package = root.path().join("packages/vite");
        fs::create_dir_all(&package).unwrap();
        fs::write(
            package.join("package.json"),
            r#"{
                "name": "@wabou/vite",
                "exports": {
                    ".": {
                        "wabou-source": "./src/index.ts",
                        "import": "./dist/index.mjs",
                        "types": "./dist/index.d.mts"
                    }
                }
            }"#,
        )
        .unwrap();

        let error = ensure_workspace_package_exports(root.path()).unwrap_err();
        let message = error.to_string();
        assert_eq!(
            missing_workspace_package_exports(root.path()).unwrap(),
            vec![package.join("dist/index.mjs")]
        );
        assert!(message.contains("bun run packages:build"));

        fs::create_dir_all(package.join("dist")).unwrap();
        fs::write(package.join("dist/index.mjs"), []).unwrap();
        ensure_workspace_package_exports(root.path()).unwrap();
    }

    #[test]
    fn workspace_package_preflight_is_a_noop_for_standalone_apps() {
        let root = tempfile::tempdir().unwrap();
        ensure_workspace_package_exports(root.path()).unwrap();
    }

    #[test]
    fn workspace_package_preflight_checks_vendored_wabou_packages() {
        let root = tempfile::tempdir().unwrap();
        let package = root.path().join("vendor/wabou/packages/vite");
        fs::create_dir_all(&package).unwrap();
        fs::write(
            package.join("package.json"),
            r#"{"name":"@wabou/vite","exports":"./dist/index.mjs"}"#,
        )
        .unwrap();

        assert_eq!(
            missing_workspace_package_exports(root.path()).unwrap(),
            vec![package.join("dist/index.mjs")]
        );
    }

    #[test]
    fn workspace_package_preflight_ignores_unrelated_packages() {
        let root = tempfile::tempdir().unwrap();
        let package = root.path().join("packages/application-library");
        fs::create_dir_all(&package).unwrap();
        fs::write(
            package.join("package.json"),
            r#"{"name":"@application/library","exports":"./dist/index.mjs"}"#,
        )
        .unwrap();

        ensure_workspace_package_exports(root.path()).unwrap();
    }

    #[test]
    fn manifest_build_output_is_the_bundle_path_source_of_truth() {
        let root = env::temp_dir().join(format!("wabou-cli-build-output-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("wabou.toml"),
            "[package]\nproduct-name = \"App\"\nidentifier = \"dev.wabou.app\"\n\n[build]\nout-dir = \"artifacts/ui\"\n",
        )
        .unwrap();
        let app = App {
            name: "app".into(),
            root: root.clone(),
            frontend: root.clone(),
            entry: "ui/index.tsx".into(),
        };
        assert_eq!(
            bundle_path(&root, &app, BuildProfile::Release).unwrap(),
            root.join("artifacts/release/ui/bundle.js")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn source_maps_follow_the_profile_unless_configured() {
        let root = tempfile::tempdir().unwrap();
        fs::write(
            root.path().join("wabou.toml"),
            "[package]\nproduct-name = \"App\"\nidentifier = \"dev.wabou.app\"\n\n[build]\nout-dir = \"dist/resources\"\n",
        )
        .unwrap();
        let app = App {
            name: "app".into(),
            root: root.path().into(),
            frontend: root.path().into(),
            entry: "ui/index.tsx".into(),
        };
        assert!(configured_source_map(&app, BuildProfile::Debug).unwrap());
        assert!(!configured_source_map(&app, BuildProfile::Release).unwrap());

        fs::write(
            root.path().join("wabou.toml"),
            "[package]\nproduct-name = \"App\"\nidentifier = \"dev.wabou.app\"\n\n[build]\nout-dir = \"dist/resources\"\nsource-map = true\n",
        )
        .unwrap();
        assert!(configured_source_map(&app, BuildProfile::Release).unwrap());
    }

    #[test]
    fn invalid_application_config_does_not_silently_select_default_outputs() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("wabou.toml"), "[build\nout-dir = 42").unwrap();
        let app = App {
            name: "app".into(),
            root: root.path().into(),
            frontend: root.path().into(),
            entry: "ui/index.tsx".into(),
        };

        let path_error = bundle_path(root.path(), &app, BuildProfile::Debug)
            .unwrap_err()
            .to_string();
        let source_map_error = configured_source_map(&app, BuildProfile::Debug)
            .unwrap_err()
            .to_string();
        assert!(path_error.contains("invalid"));
        assert!(source_map_error.contains("invalid"));
        assert!(path_error.contains("wabou.toml"));
    }

    #[test]
    fn render_actions_preserve_cross_option_command_line_order() {
        let arguments = [
            "wabou",
            "render",
            "apps/gallery",
            "--out",
            "/tmp/gallery.png",
            "--wheel",
            "120",
            "700",
            "0",
            "650",
            "--click",
            "75",
            "203",
            "--key",
            "Enter",
        ];

        assert_eq!(
            render::actions_from_argv(&arguments).unwrap(),
            vec![
                RenderAction::Wheel([120.0, 700.0, 0.0, 650.0]),
                RenderAction::Click([75.0, 203.0]),
                RenderAction::Key("Enter".into()),
            ]
        );
        assert_eq!(render::actions_from_argv(&["wabou", "run"]), None);
        assert_eq!(
            render::actions_from_argv(&["wabou", "render", "--", "--key"]),
            Some(Vec::new())
        );
    }
}
