//! Command-line entry point for developing, building, and testing Wabou apps.

use std::env;
use std::error::Error;
use std::fs;
use std::fs::OpenOptions;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use process_wrap::std::JobObject;
#[cfg(unix)]
use process_wrap::std::ProcessGroup;
use process_wrap::std::{ChildWrapper, CommandWrap};

use clap::{ArgMatches, CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};
use fs4::fs_std::FileExt as _;
use serde::Deserialize;
use serde_json::{Value, json};
use vello::Scene;
use wabou_devtools::{DebugCaptureCase, call, discover_socket, empty_params, request};
use wabou_runtime::{AppConfig, Applier, JsRuntime, PasswordInput, SecretStore};
use wabou_shell::layout::PlacedNode;
use wabou_shell::renderer::render_to_png;
use wabou_shell::scene as scene_builder;
use wabou_shell::{
    FrameSource, KeyEvent, KeyLocation, KeyPhase, Modifiers, Point, PointerButton, PointerEvent,
    PointerPhase, TextContext, UiEvent, WheelEvent,
};

mod behavior_test;
mod packaging;
mod scaffold;

use behavior_test::{default_artifact_dir, prepare_artifact_dir, replay_actions};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[command(name = "wabou", version, about = "Build and run Wabou applications")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a standalone Wabou application pinned to a Git revision.
    New {
        #[arg(value_name = "PATH")]
        path: PathBuf,
        /// Git repository used for the embedded Wabou submodule.
        #[arg(long, default_value = "https://github.com/SunDoge/wabou.git")]
        wabou_repository: String,
        /// Tag or commit used for the embedded Wabou submodule.
        #[arg(long, default_value = scaffold::DEFAULT_REVISION)]
        wabou_ref: String,
    },
    /// Start Vite, the Rust host, and live HMR.
    Dev {
        #[arg(value_name = "APP")]
        app: Option<PathBuf>,
        #[arg(long, default_value_t = 5173)]
        port: u16,
        #[arg(long)]
        devtools: bool,
        /// Vite mode used to select an application-owned development entry.
        #[arg(long)]
        mode: Option<String>,
    },
    /// Build the frontend bundle and Rust host.
    Build {
        #[arg(value_name = "APP")]
        app: Option<PathBuf>,
        #[arg(long)]
        release: bool,
        /// Generate a source map even for a release build.
        #[arg(long, value_name = "BOOL", num_args = 0..=1, default_missing_value = "true")]
        source_map: Option<bool>,
    },
    /// Build a release application and create native installers or bundles.
    Package {
        #[arg(value_name = "APP")]
        app: Option<PathBuf>,
        /// Override the formats declared in wabou.toml.
        #[arg(long, value_enum, action = clap::ArgAction::Append)]
        format: Vec<PackageFormat>,
    },
    /// Build the frontend bundle and run the Rust host.
    Run {
        #[arg(value_name = "APP")]
        app: Option<PathBuf>,
        #[arg(long)]
        release: bool,
        /// Generate a source map even for a release build.
        #[arg(long, value_name = "BOOL", num_args = 0..=1, default_missing_value = "true")]
        source_map: Option<bool>,
        /// Write an opt-in performance trace for Perfetto/Chrome tracing.
        #[arg(long, value_name = "JSON")]
        profile_trace: Option<PathBuf>,
    },
    /// Run a bundled TypeScript behavior scenario against the native host.
    Test {
        #[arg(value_name = "SCENARIO", required_unless_present = "replay")]
        scenario: Option<PathBuf>,
        /// Run the scenario against an application outside the current directory.
        #[arg(long, value_name = "PATH")]
        app: Option<PathBuf>,
        /// Replay a JSON action trace produced by an earlier test run.
        #[arg(long, value_name = "TRACE_OR_REPORT", conflicts_with = "scenario")]
        replay: Option<PathBuf>,
        /// Replay through the named test in a report, including prior state-building actions.
        #[arg(long, value_name = "NAME", requires = "replay")]
        replay_test: Option<String>,
        /// Directory for the JSON report and replayable action trace.
        #[arg(long, value_name = "DIR")]
        artifacts: Option<PathBuf>,
        /// Vite mode used to select an application-owned test fixture.
        #[arg(long)]
        mode: Option<String>,
        /// Render a PNG after failure; requires an available wgpu backend.
        #[arg(long)]
        failure_screenshot: bool,
        /// Use the real platform event loop instead of the deterministic backend.
        #[arg(long)]
        native: bool,
    },
    /// Generate or verify Rust-owned TypeScript capability bindings.
    Bindings {
        #[command(subcommand)]
        command: BindingsCommand,
    },
    /// Render an application to a PNG without opening a native window.
    Render {
        #[arg(value_name = "APP")]
        app: Option<PathBuf>,
        #[arg(long, value_name = "PNG")]
        out: PathBuf,
        #[arg(long, default_value_t = 1440)]
        width: u32,
        #[arg(long, default_value_t = 900)]
        height: u32,
        /// Logical window id exposed to the application during boot.
        #[arg(long, default_value_t = 1)]
        window_id: u64,
        /// Device scale used for widget encoding and physical PNG dimensions.
        #[arg(long, default_value_t = 1.0)]
        scale_factor: f64,
        /// Vite mode used to select an application-owned render fixture.
        #[arg(long)]
        mode: Option<String>,
        /// Keep driving asynchronous JavaScript work before capture.
        #[arg(long, default_value_t = 0)]
        wait_ms: u64,
        /// Dispatch a primary click at X Y before capture.
        #[arg(
            long,
            num_args = 2,
            value_names = ["X", "Y"],
            action = clap::ArgAction::Append
        )]
        click: Vec<f64>,
        /// Dispatch a wheel gesture at X Y with horizontal and vertical deltas.
        #[arg(
            long,
            num_args = 4,
            value_names = ["X", "Y", "DX", "DY"],
            action = clap::ArgAction::Append
        )]
        wheel: Vec<f64>,
        /// Dispatch a named key press before capture, in command-line action order.
        #[arg(long, value_name = "KEY", action = clap::ArgAction::Append)]
        key: Vec<String>,
        /// Commit text before capture, in command-line action order.
        #[arg(long, requires = "click")]
        text: Option<String>,
    },
    /// Open the native Wabou inspector.
    Devtools,
    /// Inspect a running Wabou application from the terminal.
    Inspect {
        #[arg(long, env = "WABOU_DEVTOOLS_SOCKET")]
        socket: Option<PathBuf>,
        #[command(subcommand)]
        command: InspectCommand,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Subcommand)]
enum BindingsCommand {
    /// Rewrite the committed TypeScript declarations.
    Write {
        #[arg(value_name = "APP")]
        app: Option<PathBuf>,
    },
    /// Fail when the committed declarations differ from Rust types.
    Check {
        #[arg(value_name = "APP")]
        app: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, ValueEnum)]
#[serde(rename_all = "lowercase")]
enum PackageFormat {
    App,
    Dmg,
    Nsis,
    Wix,
    Deb,
    Appimage,
    Pacman,
}

impl PackageFormat {
    fn as_str(self) -> &'static str {
        match self {
            Self::App => "app",
            Self::Dmg => "dmg",
            Self::Nsis => "nsis",
            Self::Wix => "wix",
            Self::Deb => "deb",
            Self::Appimage => "appimage",
            Self::Pacman => "pacman",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WabouPackageFile {
    package: PackageConfig,
    #[serde(default)]
    build: Option<BuildConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct BuildConfig {
    out_dir: PathBuf,
    #[serde(default)]
    source_map: Option<bool>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BuildProfile {
    Debug,
    Release,
}

impl BuildProfile {
    fn from_release(release: bool) -> Self {
        if release { Self::Release } else { Self::Debug }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Release => "release",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct PackageConfig {
    product_name: String,
    identifier: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    authors: Vec<String>,
    #[serde(default)]
    copyright: Option<String>,
    #[serde(default)]
    license_file: Option<PathBuf>,
    #[serde(default)]
    icons: Vec<String>,
    #[serde(default)]
    resources: Vec<PathBuf>,
    #[serde(default)]
    formats: Vec<PackageFormat>,
}

#[derive(Subcommand)]
enum InspectCommand {
    Status,
    Query {
        query: String,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    Node {
        id: u32,
    },
    At {
        x: f32,
        y: f32,
    },
    Frames {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    Screenshot,
    Capture {
        #[arg(long)]
        x: Option<f32>,
        #[arg(long)]
        y: Option<f32>,
        #[arg(long, value_name = "DIR")]
        output: PathBuf,
    },
}

struct App {
    name: String,
    root: PathBuf,
    frontend: PathBuf,
    entry: String,
}

struct RenderOptions {
    out: PathBuf,
    width: u32,
    height: u32,
    window_id: u64,
    scale_factor: f64,
    mode: Option<String>,
    wait_ms: u64,
    actions: Vec<RenderAction>,
}

#[derive(Clone, Debug, PartialEq)]
enum RenderAction {
    Click([f64; 2]),
    Wheel([f64; 4]),
    Text(String),
    Key(String),
}

struct TestOptions {
    scenario: Option<PathBuf>,
    replay: Option<PathBuf>,
    replay_test: Option<String>,
    artifacts: Option<PathBuf>,
    mode: Option<String>,
    failure_screenshot: bool,
    native: bool,
}

fn main() -> Result<()> {
    let matches = Cli::command().get_matches();
    let render_actions = render_actions_from_matches(&matches);
    let cli = Cli::from_arg_matches(&matches)?;
    let cwd = env::current_dir()?;
    let resolve_app = |app_path: Option<&Path>| {
        let workspace = find_workspace(&cwd)?;
        let app = load_app(&workspace, &cwd, app_path)?;
        Ok::<_, Box<dyn Error>>((workspace, app))
    };
    match cli.command {
        Commands::New {
            path,
            wabou_repository,
            wabou_ref,
        } => scaffold::create(&cwd.join(path), &wabou_repository, &wabou_ref),
        Commands::Dev {
            app,
            port,
            devtools,
            mode,
        } => {
            let (workspace, app) = resolve_app(app.as_deref())?;
            dev(&workspace, app, port, devtools, mode.as_deref())
        }
        Commands::Build {
            app,
            release,
            source_map,
        } => {
            let (workspace, app) = resolve_app(app.as_deref())?;
            build(&workspace, &app, release, source_map)
        }
        Commands::Package { app, format } => {
            let (workspace, app) = resolve_app(app.as_deref())?;
            package(&workspace, &app, &format)
        }
        Commands::Run {
            app,
            release,
            source_map,
            profile_trace,
        } => {
            let (workspace, app) = resolve_app(app.as_deref())?;
            let profile_trace = profile_trace.map(|path| cwd.join(path));
            run(
                &workspace,
                &app,
                release,
                source_map,
                profile_trace.as_deref(),
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
        } => {
            let (workspace, app) = resolve_app(app.as_deref())?;
            let options = TestOptions {
                scenario: scenario.map(|path| cwd.join(path)),
                replay: replay.map(|path| cwd.join(path)),
                replay_test,
                artifacts,
                mode,
                failure_screenshot,
                native,
            };
            test_scenario(&workspace, &app, &options)
        }
        Commands::Bindings { command } => {
            let app_path = match &command {
                BindingsCommand::Write { app } | BindingsCommand::Check { app } => app.as_deref(),
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
            mode,
            wait_ms,
            click,
            wheel,
            key,
            text,
        } => {
            let (workspace, app) = resolve_app(app.as_deref())?;
            render(
                &workspace,
                &app,
                &RenderOptions {
                    out,
                    width,
                    height,
                    window_id,
                    scale_factor,
                    mode,
                    wait_ms,
                    actions: render_actions
                        .unwrap_or_else(|| legacy_render_actions(click, wheel, text, key)),
                },
            )
        }
        Commands::Devtools => run_devtools(&find_workspace(&cwd).unwrap_or(cwd)),
        Commands::Inspect { socket, command } => inspect(socket, command),
    }
}

fn legacy_render_actions(
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

fn render_actions_from_matches(matches: &ArgMatches) -> Option<Vec<RenderAction>> {
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

fn find_workspace(start: &Path) -> Result<PathBuf> {
    for dir in start.ancestors() {
        let manifest = dir.join("Cargo.toml");
        if fs::read_to_string(&manifest).is_ok_and(|text| text.contains("[workspace]")) {
            return Ok(dir.to_path_buf());
        }
    }
    find_app_root(start).ok_or_else(|| "not inside a Wabou Cargo project".into())
}

fn load_app(workspace: &Path, cwd: &Path, app_path: Option<&Path>) -> Result<App> {
    let root = match app_path {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => cwd.join(path),
        None => find_app_root(cwd).unwrap_or_else(|| workspace.join("apps/gallery")),
    };
    if !root.join("Cargo.toml").is_file() {
        return Err(format!(
            "{} is not a Wabou app: Cargo.toml is missing",
            root.display()
        )
        .into());
    }
    if !root.join("package.json").is_file() {
        return Err(format!(
            "{} is not a Wabou app: package.json is missing",
            root.display()
        )
        .into());
    }
    let entry = if root.join("ui/index.tsx").is_file() {
        "ui/index.tsx"
    } else {
        return Err(format!("{} has no conventional ui/index.tsx entry", root.display()).into());
    };
    Ok(App {
        name: root
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("app directory must have a UTF-8 name")?
            .to_string(),
        frontend: root.clone(),
        root,
        entry: entry.to_string(),
    })
}

fn find_app_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| {
            fs::read_to_string(dir.join("Cargo.toml")).is_ok_and(|text| text.contains("[package]"))
        })
        .map(Path::to_path_buf)
}

fn manifest(app: &App) -> String {
    app.root.join("Cargo.toml").to_string_lossy().into_owned()
}

fn configured_resource_dir(workspace: &Path, app: &App) -> PathBuf {
    if let Ok(source) = fs::read_to_string(app.root.join("wabou.toml"))
        && let Ok(file) = toml::from_str::<WabouPackageFile>(&source)
        && let Some(build) = file.build
    {
        return app.root.join(build.out_dir);
    }
    let dist = workspace.join("dist");
    if workspace == app.root {
        dist.join("resources")
    } else {
        dist.join(&app.name).join("resources")
    }
}

fn profile_resource_dir(workspace: &Path, app: &App, profile: BuildProfile) -> PathBuf {
    let configured = configured_resource_dir(workspace, app);
    let name = configured.file_name().unwrap_or_default();
    configured
        .parent()
        .unwrap_or(&configured)
        .join(profile.as_str())
        .join(name)
}

fn distribution_root(workspace: &Path, app: &App) -> PathBuf {
    let resources = configured_resource_dir(workspace, app);
    resources.parent().unwrap_or(&resources).to_path_buf()
}

fn profile_application_dir(workspace: &Path, app: &App, profile: BuildProfile) -> PathBuf {
    distribution_root(workspace, app).join(profile.as_str())
}

fn bundle_path(workspace: &Path, app: &App, profile: BuildProfile) -> PathBuf {
    profile_resource_dir(workspace, app, profile).join("bundle.js")
}

fn configured_source_map(app: &App, profile: BuildProfile) -> bool {
    let setting = fs::read_to_string(app.root.join("wabou.toml"))
        .ok()
        .and_then(|source| toml::from_str::<WabouPackageFile>(&source).ok())
        .and_then(|file| file.build)
        .and_then(|build| build.source_map);
    setting.unwrap_or(profile == BuildProfile::Debug)
}

fn frontend_build_lock(workspace: &Path, app: &App) -> Result<fs::File> {
    let directory = workspace.join("target/wabou/frontend").join(&app.name);
    fs::create_dir_all(&directory)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(directory.join("frontend.lock"))?;
    file.lock_exclusive()?;
    Ok(file)
}

fn collect_dist_exports(value: &Value, output: &mut Vec<String>) {
    match value {
        Value::String(path) if path.starts_with("./dist/") => output.push(path[2..].to_owned()),
        Value::Array(values) => {
            for value in values {
                collect_dist_exports(value, output);
            }
        }
        Value::Object(values) => {
            for (condition, value) in values {
                if condition != "types" && condition != "wabou-source" {
                    collect_dist_exports(value, output);
                }
            }
        }
        _ => {}
    }
}

/// Catch interrupted workspace package builds before Vite turns a missing
/// tracked entrypoint into an opaque `externalize-deps` resolution failure.
fn ensure_workspace_package_exports(workspace: &Path) -> Result<()> {
    let packages = workspace.join("packages");
    if !packages.is_dir() {
        return Ok(());
    }
    let entries = fs::read_dir(&packages)?;
    let mut missing = Vec::new();
    for entry in entries {
        let package = entry?.path();
        let manifest_path = package.join("package.json");
        if !manifest_path.is_file() {
            continue;
        }
        let manifest: Value =
            serde_json::from_str(&fs::read_to_string(&manifest_path)?).map_err(|error| {
                format!(
                    "invalid package manifest {}: {error}",
                    manifest_path.display()
                )
            })?;
        let mut exports = Vec::new();
        if let Some(value) = manifest.get("exports") {
            collect_dist_exports(value, &mut exports);
        }
        exports.sort();
        exports.dedup();
        missing.extend(
            exports
                .into_iter()
                .map(|path| package.join(path))
                .filter(|path| !path.is_file()),
        );
    }
    if missing.is_empty() {
        return Ok(());
    }
    missing.sort();
    let paths = missing
        .iter()
        .map(|path| {
            path.strip_prefix(workspace)
                .unwrap_or(path)
                .display()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n  - ");
    Err(format!(
        "Wabou workspace JavaScript package artifacts are missing:\n  - {paths}\nrun `bun run packages:build` from {} and retry",
        workspace.display()
    )
    .into())
}

fn frontend_unlocked(
    workspace: &Path,
    app: &App,
    script: &str,
    args: &[&str],
    profile: BuildProfile,
    source_map: bool,
) -> Result<ExitStatus> {
    ensure_workspace_package_exports(workspace)?;
    let mut command = Command::new("bun");
    command.current_dir(&app.frontend).args(["run", script]);
    command
        .env("WABOU_BUILD_PROFILE", profile.as_str())
        .env(
            "WABOU_ENV_DEBUG",
            if profile == BuildProfile::Debug {
                "true"
            } else {
                "false"
            },
        )
        .env(
            "WABOU_SOURCE_MAP",
            if source_map { "true" } else { "false" },
        )
        .env(
            "WABOU_OUT_DIR",
            profile_resource_dir(workspace, app, profile),
        );
    if !args.is_empty() {
        command.arg("--").args(args);
    }
    command.status().map_err(|error| {
        format!(
            "failed to run `bun run {script}` in {}: {error}; install Bun or run Wabou through `mise exec --`",
            app.frontend.display()
        )
        .into()
    })
}

fn frontend(
    workspace: &Path,
    app: &App,
    script: &str,
    args: &[&str],
    profile: BuildProfile,
    source_map: bool,
) -> Result<ExitStatus> {
    let _lock = frontend_build_lock(workspace, app)?;
    frontend_unlocked(workspace, app, script, args, profile, source_map)
}

fn build(
    workspace: &Path,
    app: &App,
    release: bool,
    source_map_override: Option<bool>,
) -> Result<()> {
    let profile = BuildProfile::from_release(release);
    let source_map = source_map_override.unwrap_or_else(|| configured_source_map(app, profile));
    let manifest = manifest(app);
    let mut cargo = Command::new("cargo");
    cargo
        .current_dir(workspace)
        .args(["build", "--manifest-path", &manifest]);
    if release {
        cargo.arg("--release");
    }
    ensure(cargo.status()?, "Cargo build")?;
    ensure(
        frontend(workspace, app, "build", &[], profile, source_map)?,
        "Vite build",
    )?;
    package_executable(workspace, app, release)
}

fn package(workspace: &Path, app: &App, format_override: &[PackageFormat]) -> Result<()> {
    let config = load_package_config(app)?;
    build(workspace, app, true, None)?;
    let (stage, binary) = stage_application(workspace, app, &config)?;
    let metadata = cargo_metadata(workspace, app)?;
    let manifest_path = app.root.join("Cargo.toml").canonicalize()?;
    let version = package_metadata(&metadata, &manifest_path)
        .and_then(|package| package["version"].as_str())
        .ok_or("Cargo metadata has no application version")?;
    let formats = if format_override.is_empty() {
        &config.formats
    } else {
        format_override
    };
    if formats.is_empty() {
        return Err("wabou.toml must declare at least one package format".into());
    }

    let package_root = distribution_root(workspace, app);
    let bundles = package_root.join("bundles");
    fs::create_dir_all(&bundles)?;
    let packager_config = package_root.join("packager.json");
    let resources = stage.join("resources");
    let icons = config
        .icons
        .iter()
        .map(|path| app.root.join(path).to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let license_file = config
        .license_file
        .as_ref()
        .map(|path| app.root.join(path).to_string_lossy().into_owned());
    let generated = json!({
        "name": app.name,
        "productName": config.product_name,
        "version": version,
        "identifier": config.identifier,
        "description": config.description,
        "authors": config.authors,
        "copyright": config.copyright,
        "licenseFile": license_file,
        "icons": icons,
        "binaries": [{ "path": binary, "main": true }],
        "binariesDir": stage,
        "resources": [{ "src": resources, "target": "resources" }],
        "formats": formats.iter().map(|format| format.as_str()).collect::<Vec<_>>(),
        "outDir": bundles,
    });
    let outputs = packaging::package(&generated, &packager_config)?;
    for output in outputs {
        println!("[wabou] packaged {}", output.display());
    }
    Ok(())
}

fn load_package_config(app: &App) -> Result<PackageConfig> {
    let path = app.root.join("wabou.toml");
    let source = fs::read_to_string(&path).map_err(|error| {
        format!(
            "cannot read package configuration {}: {error}",
            path.display()
        )
    })?;
    let file: WabouPackageFile =
        toml::from_str(&source).map_err(|error| format!("invalid {}: {error}", path.display()))?;
    if file.package.product_name.trim().is_empty() {
        return Err("package.product-name cannot be empty".into());
    }
    let identifier = file.package.identifier.as_str();
    if !identifier.contains('.')
        || !identifier
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
    {
        return Err("package.identifier must be a reverse-domain identifier".into());
    }
    Ok(file.package)
}

fn stage_application(
    workspace: &Path,
    app: &App,
    config: &PackageConfig,
) -> Result<(PathBuf, String)> {
    let package_root = distribution_root(workspace, app);
    let stage = package_root.join("stage");
    if stage.is_dir() {
        fs::remove_dir_all(&stage)?;
    }
    let resources = stage.join("resources");
    fs::create_dir_all(&resources)?;
    let binary = app_binary(workspace, app)?;
    let release_root = package_root.join(BuildProfile::Release.as_str());
    fs::copy(release_root.join(&binary), stage.join(&binary))?;
    fs::copy(
        release_root.join("resources/bundle.js"),
        resources.join("bundle.js"),
    )?;

    let app_root = app.root.canonicalize()?;
    for relative in &config.resources {
        let source = app.root.join(relative).canonicalize().map_err(|error| {
            format!(
                "cannot stage package resource {}: {error}",
                relative.display()
            )
        })?;
        if !source.starts_with(&app_root) {
            return Err(format!(
                "package resource {} escapes the application directory",
                relative.display()
            )
            .into());
        }
        let name = source
            .file_name()
            .ok_or("package resource must have a file name")?;
        copy_resource(&source, &resources.join(name))?;
    }
    Ok((stage, binary))
}

fn copy_resource(source: &Path, destination: &Path) -> Result<()> {
    if fs::symlink_metadata(source)?.file_type().is_symlink() {
        return Err(format!(
            "package resources cannot contain symbolic links: {}",
            source.display()
        )
        .into());
    }
    if source.is_dir() {
        fs::create_dir_all(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_resource(&entry.path(), &destination.join(entry.file_name()))?;
        }
    } else if source.is_file() {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, destination)?;
    } else {
        return Err(format!("unsupported package resource {}", source.display()).into());
    }
    Ok(())
}

fn run(
    workspace: &Path,
    app: &App,
    release: bool,
    source_map_override: Option<bool>,
    profile_trace: Option<&Path>,
) -> Result<()> {
    let profile = BuildProfile::from_release(release);
    let source_map = source_map_override.unwrap_or_else(|| configured_source_map(app, profile));
    ensure(
        frontend(workspace, app, "build", &[], profile, source_map)?,
        "Vite build",
    )?;
    let manifest = manifest(app);
    let binary = app_binary(workspace, app)?;
    let mut cargo = Command::new("cargo");
    cargo
        .current_dir(workspace)
        .args(["run", "--manifest-path", &manifest, "--bin", &binary]);
    if release {
        cargo.arg("--release");
    }
    if let Some(path) = profile_trace {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        cargo
            .args(["--features", &app_profiling_feature(workspace, app)?])
            .env("WABOU_PROFILE_TRACE", path);
    }
    cargo.env("WABOU_BUNDLE_PATH", bundle_path(workspace, app, profile));
    ensure(cargo.status()?, "Rust host")
}

fn test_scenario(workspace: &Path, app: &App, options: &TestOptions) -> Result<()> {
    const HOST_TIMEOUT: Duration = Duration::from_secs(70);
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
                serde_json::to_string(
                    &workspace
                        .join("packages/test/src/index.ts")
                        .to_string_lossy()
                )?,
                serde_json::to_string(&parsed)?
            ),
        )?;
        generated_replay.as_path()
    } else {
        options
            .scenario
            .as_deref()
            .ok_or("a scenario or --replay trace is required")?
    };
    if !scenario.is_file() {
        return Err(format!("test scenario {} does not exist", scenario.display()).into());
    }

    let mode_args = options.mode.as_deref().map(|mode| ["--mode", mode]);
    ensure(
        frontend(
            workspace,
            app,
            "build",
            mode_args.as_ref().map_or(&[], |args| args),
            BuildProfile::Debug,
            true,
        )?,
        "Vite build",
    )?;

    let scenario_bundle = test_dir.join("scenario.js");
    let mut bun = Command::new("bun");
    bun.current_dir(workspace).args([
        "build",
        &scenario.to_string_lossy(),
        "--target=browser",
        "--format=iife",
        &format!("--outfile={}", scenario_bundle.display()),
    ]);
    ensure(bun.status()?, "test scenario build")?;

    let manifest = manifest(app);
    let binary = app_binary(workspace, app)?;
    let test_data = tempfile::tempdir_in(&test_dir)?;
    let executable = build_behavior_host(workspace, &manifest, &binary)?;
    let mut host = Command::new(executable);
    host.current_dir(workspace)
        .env(
            "WABOU_BUNDLE_PATH",
            bundle_path(workspace, app, BuildProfile::Debug),
        )
        .env("WABOU_TEST_SCRIPT", scenario_bundle)
        .env("WABOU_TEST_ARTIFACT_DIR", artifact_dir)
        .env("WABOU_TEST_APP_DATA_ROOT", test_data.path())
        // Also isolate libraries that use the XDG convention directly rather
        // than resolving paths through Wabou's AppDirectories API.
        .env("XDG_CONFIG_HOME", test_data.path().join("xdg-config"))
        .env("XDG_DATA_HOME", test_data.path().join("xdg-data"))
        .env("XDG_CACHE_HOME", test_data.path().join("xdg-cache"));
    configure_test_backend(&mut host, options.native);
    if options.failure_screenshot {
        host.env("WABOU_TEST_FAILURE_SCREENSHOT", "1");
    }
    let stopped = Arc::new(AtomicBool::new(false));
    let signal = stopped.clone();
    ctrlc::set_handler(move || signal.store(true, Ordering::Release))?;
    let status = wait_for_managed_child(host, HOST_TIMEOUT, &stopped)?;
    ensure(status, "Wabou behavior test")
}

fn build_behavior_host(workspace: &Path, manifest: &str, binary: &str) -> Result<PathBuf> {
    let output = Command::new("cargo")
        .current_dir(workspace)
        .args([
            "build",
            "--manifest-path",
            manifest,
            "--bin",
            binary,
            "--message-format=json-render-diagnostics",
        ])
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

fn configure_test_backend(command: &mut Command, native: bool) {
    if native {
        // `--native` must win over an inherited shell/CI variable.
        command.env_remove("WABOU_TEST_HEADLESS");
    } else {
        command.env("WABOU_TEST_HEADLESS", "1");
    }
}

fn wait_for_managed_child(
    command: Command,
    timeout: Duration,
    stopped: &AtomicBool,
) -> Result<ExitStatus> {
    let mut child = ManagedChild::spawn(command)?;
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.child.try_wait()? {
            return Ok(status);
        }
        if stopped.load(Ordering::Acquire) {
            child.terminate();
            return Err("Wabou behavior test interrupted".into());
        }
        if Instant::now() >= deadline {
            child.terminate();
            return Err(format!(
                "Wabou behavior test host exceeded its final {}s watchdog",
                timeout.as_secs()
            )
            .into());
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn bindings(workspace: &Path, app: &App, mode: BindingsCommand) -> Result<()> {
    let manifest = manifest(app);
    let mode = match mode {
        BindingsCommand::Write { .. } => "write",
        BindingsCommand::Check { .. } => "check",
    };
    let mut cargo = Command::new("cargo");
    cargo.current_dir(workspace).args([
        "run",
        "--quiet",
        "--manifest-path",
        &manifest,
        "--example",
        "wabou-bindgen",
        "--",
        mode,
    ]);
    ensure(cargo.status()?, "Wabou bindings generator")
}

fn settle_render_actions(
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

fn apply_render_actions(
    applier: &mut Applier,
    text_context: &mut TextContext,
    nodes: &mut Vec<PlacedNode>,
    options: &RenderOptions,
) {
    let mut settle = |applier: &mut Applier| {
        settle_render_actions(applier, text_context, nodes, options.width, options.height);
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

fn render(workspace: &Path, app: &App, options: &RenderOptions) -> Result<()> {
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
    // Vite replaces the shared bundle and source map while building. Keep the
    // lock through the bundle read so concurrent renders cannot observe a
    // partially replaced frontend artifact.
    let frontend_lock = frontend_build_lock(workspace, app)?;
    let mode_args = mode.as_deref().map(|mode| ["--mode", mode]);
    ensure(
        frontend_unlocked(
            workspace,
            app,
            "build",
            mode_args.as_ref().map_or(&[], |args| args),
            BuildProfile::Debug,
            true,
        )?,
        "Vite build",
    )?;
    let path = bundle_path(workspace, app, BuildProfile::Debug);
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
    // Install host globals before evaluating the bundle. Hooks such as
    // useWindow() read the logical id during module initialization, so booting
    // first would permanently expose the fallback window id 0 in screenshots.
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

    if *wait_ms > 0 {
        let deadline = Instant::now() + Duration::from_millis(*wait_ms);
        while Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
            nodes = applier.build_frame(&mut text_context, *width, *height);
        }
    }
    apply_render_actions(&mut applier, &mut text_context, &mut nodes, options);

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

fn dev(
    workspace: &Path,
    app: App,
    port: u16,
    open_devtools: bool,
    mode: Option<&str>,
) -> Result<()> {
    ensure_workspace_package_exports(workspace)?;
    let port_text = port.to_string();
    let mut vite_command = Command::new("bun");
    vite_command
        .current_dir(&app.frontend)
        .args(["run", "dev", "--", "--port", &port_text, "--strictPort"])
        .stdin(Stdio::null());
    if let Some(mode) = mode {
        vite_command.args(["--mode", mode]);
    }
    let mut vite = ManagedChild::spawn(vite_command)?;
    let url = format!("http://127.0.0.1:{port}");
    wait_for_vite(&url, vite.child.as_mut())?;

    let app_manifest = manifest(&app);
    let binary = app_binary(workspace, &app)?;
    let vite_feature = app_vite_feature(workspace, &app)?;
    let mut host_command = Command::new("cargo");
    host_command
        .current_dir(workspace)
        .args([
            "run",
            "--manifest-path",
            &app_manifest,
            "--bin",
            &binary,
            "--features",
            &vite_feature,
        ])
        .env("WABOU_VITE_URL", &url)
        .env("WABOU_VITE_ENTRY", &app.entry);
    let mut host = ManagedChild::spawn(host_command)?;

    let mut inspector = if open_devtools {
        let command = devtools_command(workspace)?;
        Some(ManagedChild::spawn(command)?)
    } else {
        None
    };

    println!("[wabou] dev server ready at {url}; press Ctrl-C to stop");
    supervise(&mut host, &mut vite, inspector.as_mut())
}

fn run_devtools(workspace: &Path) -> Result<()> {
    ensure(devtools_command(workspace)?.status()?, "Wabou DevTools")
}

/// Resolve the GUI as a helper executable. Inside the Wabou source workspace we
/// build its frontend and host directly; installed CLIs find a sibling binary
/// first and then use PATH.
fn devtools_command(workspace: &Path) -> Result<Command> {
    let source = workspace.join("apps/devtools");
    if source.join("Cargo.toml").is_file() && source.join("package.json").is_file() {
        let app = load_app(workspace, workspace, Some(&source))?;
        ensure(
            frontend(workspace, &app, "build", &[], BuildProfile::Debug, true)?,
            "DevTools Vite build",
        )?;
        let manifest = manifest(&app);
        let mut cargo = Command::new("cargo");
        cargo
            .current_dir(workspace)
            .args(["build", "--manifest-path", &manifest]);
        ensure(cargo.status()?, "DevTools Rust build")?;
        let executable = built_executable(workspace, &app, false)?;
        let mut command = Command::new(executable);
        command.env(
            "WABOU_BUNDLE_PATH",
            bundle_path(workspace, &app, BuildProfile::Debug),
        );
        return Ok(command);
    }

    let executable = find_helper(
        &env::current_exe()?,
        env::var_os("PATH").as_deref(),
        "wabou-devtools",
    )
    .ok_or(
        "wabou-devtools was not found next to wabou or on PATH; install the Wabou DevTools package",
    )?;
    Ok(Command::new(executable))
}

fn built_executable(workspace: &Path, app: &App, release: bool) -> Result<PathBuf> {
    let manifest = manifest(app);
    let output = Command::new("cargo")
        .current_dir(workspace)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--manifest-path",
            &manifest,
        ])
        .output()?;
    ensure(output.status, "Cargo metadata")?;
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    artifact_from_metadata(
        &metadata,
        &app.root.join("Cargo.toml").canonicalize()?,
        release,
    )
    .map(|(path, _)| path)
}

fn find_helper(current_exe: &Path, path: Option<&std::ffi::OsStr>, name: &str) -> Option<PathBuf> {
    let filename = format!("{name}{}", env::consts::EXE_SUFFIX);
    if let Some(parent) = current_exe.parent() {
        let sibling = parent.join(&filename);
        if sibling.is_file() {
            return Some(sibling);
        }
    }
    path.and_then(|path| {
        env::split_paths(path)
            .map(|dir| dir.join(&filename))
            .find(|candidate| candidate.is_file())
    })
}

fn inspect(socket: Option<PathBuf>, command: InspectCommand) -> Result<()> {
    let path = socket.map_or_else(discover_socket, Ok)?;
    let capture_output = match &command {
        InspectCommand::Capture { output, .. } => Some(output.clone()),
        _ => None,
    };
    let (method, params): (&str, Value) = match command {
        InspectCommand::Status => ("status", empty_params()),
        InspectCommand::Query { query, limit } => {
            ("queryNodes", json!({ "query": query, "limit": limit }))
        }
        InspectCommand::Node { id } => ("inspectNode", json!({ "id": id })),
        InspectCommand::At { x, y } => ("inspectAtPoint", json!({ "x": x, "y": y })),
        InspectCommand::Frames { limit } => ("recentFrames", json!({ "limit": limit })),
        InspectCommand::Screenshot => ("captureScreenshot", empty_params()),
        InspectCommand::Capture { x, y, .. } => {
            let params = match (x, y) {
                (Some(x), Some(y)) => json!({ "x": x, "y": y }),
                (None, None) => empty_params(),
                _ => return Err("--x and --y must be provided together".into()),
            };
            ("captureCase", params)
        }
    };
    let response = call(&path, &request(1, method, params))
        .map_err(|error| format!("cannot connect to {}: {error}", path.display()))?;
    if let Some(error) = response.error {
        return Err(error.into());
    }
    let result = response.result.unwrap_or(Value::Null);
    if let Some(output) = capture_output {
        let capture: DebugCaptureCase = serde_json::from_value(result)?;
        write_capture_case(&output, capture)?;
        println!("{}", output.display());
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }
    Ok(())
}

fn write_capture_case(output: &Path, mut capture: DebugCaptureCase) -> Result<()> {
    fs::create_dir_all(output)?;
    let screenshot = output.join("screenshot.png");
    fs::copy(&capture.screenshot_path, screenshot)?;
    capture.screenshot_path = PathBuf::from("screenshot.png");
    fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(&capture)?,
    )?;
    fs::write(
        output.join("tree.json"),
        serde_json::to_vec_pretty(&capture.snapshot)?,
    )?;
    if let Some(node) = capture.point.as_ref().and_then(|point| point.node.as_ref()) {
        fs::write(
            output.join("selected-node.json"),
            serde_json::to_vec_pretty(node)?,
        )?;
    }
    Ok(())
}

fn package_executable(workspace: &Path, app: &App, release: bool) -> Result<()> {
    let metadata = cargo_metadata(workspace, app)?;
    let manifest_path = app.root.join("Cargo.toml").canonicalize()?;
    let (source, binary) = artifact_from_metadata(&metadata, &manifest_path, release)?;
    let destination_dir =
        profile_application_dir(workspace, app, BuildProfile::from_release(release));
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

fn app_binary(workspace: &Path, app: &App) -> Result<String> {
    let metadata = cargo_metadata(workspace, app)?;
    let manifest_path = app.root.join("Cargo.toml").canonicalize()?;
    binary_target(&metadata, &manifest_path)
        .and_then(|target| target["name"].as_str())
        .map(str::to_owned)
        .ok_or_else(|| "application binary target has no name".into())
}

fn app_vite_feature(workspace: &Path, app: &App) -> Result<String> {
    let metadata = cargo_metadata(workspace, app)?;
    let manifest_path = app.root.join("Cargo.toml").canonicalize()?;
    vite_feature(&metadata, &manifest_path)
        .map(str::to_owned)
        .ok_or_else(|| "application must depend on `wabou` or `wabou-runtime`".into())
}

fn app_profiling_feature(workspace: &Path, app: &App) -> Result<String> {
    let metadata = cargo_metadata(workspace, app)?;
    let manifest_path = app.root.join("Cargo.toml").canonicalize()?;
    framework_feature(&metadata, &manifest_path, "profiling")
        .ok_or_else(|| "application must depend on `wabou` or `wabou-runtime`".into())
}

fn framework_feature(metadata: &Value, manifest_path: &Path, feature: &str) -> Option<String> {
    let dependencies = package_metadata(metadata, manifest_path)?["dependencies"].as_array()?;
    if dependencies
        .iter()
        .any(|dependency| dependency["name"] == "wabou")
    {
        Some(format!("wabou/{feature}"))
    } else if dependencies
        .iter()
        .any(|dependency| dependency["name"] == "wabou-runtime")
    {
        Some(format!("wabou-runtime/{feature}"))
    } else {
        None
    }
}

fn vite_feature<'a>(metadata: &'a Value, manifest_path: &Path) -> Option<&'a str> {
    let dependencies = package_metadata(metadata, manifest_path)?["dependencies"].as_array()?;
    if dependencies
        .iter()
        .any(|dependency| dependency["name"] == "wabou")
    {
        Some("wabou/vite")
    } else if dependencies
        .iter()
        .any(|dependency| dependency["name"] == "wabou-runtime")
    {
        Some("wabou-runtime/vite")
    } else {
        None
    }
}

fn cargo_metadata(workspace: &Path, app: &App) -> Result<Value> {
    let manifest = manifest(app);
    let output = Command::new("cargo")
        .current_dir(workspace)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--manifest-path",
            &manifest,
        ])
        .output()?;
    ensure(output.status, "Cargo metadata")?;
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn binary_target<'a>(metadata: &'a Value, manifest_path: &Path) -> Option<&'a Value> {
    let package = package_metadata(metadata, manifest_path)?;
    let binaries = package["targets"]
        .as_array()?
        .iter()
        .filter(|target| {
            target["kind"]
                .as_array()
                .is_some_and(|kinds| kinds.iter().any(|kind| kind == "bin"))
        })
        .collect::<Vec<_>>();
    let package_name = package["name"].as_str();
    let named = binaries
        .iter()
        .copied()
        .find(|target| target["name"].as_str() == package_name);
    named.or_else(|| {
        if binaries.len() == 1 {
            Some(binaries[0])
        } else {
            None
        }
    })
}

fn package_metadata<'a>(metadata: &'a Value, manifest_path: &Path) -> Option<&'a Value> {
    metadata["packages"].as_array()?.iter().find(|package| {
        package["manifest_path"]
            .as_str()
            .is_some_and(|path| Path::new(path) == manifest_path)
    })
}

fn artifact_from_metadata(
    metadata: &serde_json::Value,
    manifest_path: &Path,
    release: bool,
) -> Result<(PathBuf, String)> {
    let binary = binary_target(metadata, manifest_path)
        .and_then(|target| target["name"].as_str())
        .ok_or("app package has no unambiguous primary binary target")?;
    let target_dir = metadata["target_directory"]
        .as_str()
        .ok_or("Cargo metadata has no target directory")?;
    let profile = if release { "release" } else { "debug" };
    let filename = format!("{binary}{}", env::consts::EXE_SUFFIX);
    Ok((
        Path::new(target_dir).join(profile).join(&filename),
        filename,
    ))
}

fn wait_for_vite(url: &str, child: &mut dyn ChildWrapper) -> Result<()> {
    let authority = url.trim_start_matches("http://");
    let address = authority
        .to_socket_addrs()?
        .next()
        .ok_or("Vite address did not resolve")?;
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait()? {
            return Err(format!("Vite exited before startup: {status}").into());
        }
        if TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err("timed out waiting for Vite".into())
}

fn supervise(
    host: &mut ManagedChild,
    vite: &mut ManagedChild,
    inspector: Option<&mut ManagedChild>,
) -> Result<()> {
    let stopped = Arc::new(AtomicBool::new(false));
    let signal = stopped.clone();
    ctrlc::set_handler(move || signal.store(true, Ordering::Release))?;
    let mut inspector = inspector;
    let result = loop {
        if stopped.load(Ordering::Acquire) {
            break Ok(());
        }
        if let Some(status) = host.child.try_wait()? {
            break ensure(status, "Rust host");
        }
        if let Some(status) = vite.child.try_wait()? {
            break ensure(status, "Vite dev server");
        }
        if let Some(child) = inspector.as_mut()
            && let Some(status) = child.child.try_wait()?
        {
            eprintln!("[wabou] DevTools exited: {status}");
            inspector = None;
        }
        thread::sleep(Duration::from_millis(50));
    };
    host.terminate();
    vite.terminate();
    if let Some(child) = inspector {
        child.terminate();
    }
    result
}

struct ManagedChild {
    child: Box<dyn ChildWrapper>,
}

impl ManagedChild {
    fn spawn(command: Command) -> std::io::Result<Self> {
        let mut command = CommandWrap::from(command);
        #[cfg(unix)]
        command.wrap(ProcessGroup::leader());
        #[cfg(windows)]
        command.wrap(JobObject);
        command.spawn().map(|child| Self { child })
    }

    fn terminate(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for ManagedChild {
    fn drop(&mut self) {
        self.terminate();
    }
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
    use super::*;

    #[test]
    fn parses_new_project_git_source() {
        let Cli {
            command:
                Commands::New {
                    path,
                    wabou_repository,
                    wabou_ref,
                },
        } = Cli::try_parse_from([
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
        } = Cli::try_parse_from(["wabou", "new", "hello-wabou"]).unwrap()
        else {
            panic!("expected new command");
        };
        assert_eq!(wabou_repository, "https://github.com/SunDoge/wabou.git");
        assert_eq!(wabou_ref, "v0.1.0-alpha.1");
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
    use wabou_devtools::{DebugNode, DebugPointInspection, DebugSnapshot};

    #[test]
    fn render_defaults_to_the_main_logical_window_and_accepts_an_override() {
        let Cli {
            command:
                Commands::Render {
                    window_id,
                    scale_factor,
                    mode,
                    click,
                    wheel,
                    key,
                    ..
                },
        } = Cli::try_parse_from(["wabou", "render", "--out", "capture.png"]).unwrap()
        else {
            panic!("expected render command");
        };
        assert_eq!(window_id, 1);
        assert_eq!(scale_factor, 1.0);
        assert_eq!(mode, None);
        assert!(click.is_empty());
        assert!(wheel.is_empty());
        assert!(key.is_empty());

        let Cli {
            command:
                Commands::Render {
                    window_id,
                    scale_factor,
                    mode,
                    click,
                    wheel,
                    key,
                    ..
                },
        } = Cli::try_parse_from([
            "wabou",
            "render",
            "--out",
            "capture.png",
            "--window-id",
            "7",
            "--scale-factor",
            "2",
            "--mode",
            "ui-test",
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
        assert_eq!(mode.as_deref(), Some("ui-test"));
        assert_eq!(click, [10.0, 20.0, 30.0, 40.0]);
        assert_eq!(wheel, [100.0, 200.0, 0.0, 360.0]);
        assert_eq!(key, ["Enter", "Escape"]);
    }

    #[test]
    fn parses_bindings_write_and_check_commands() {
        for name in ["write", "check"] {
            let Cli {
                command: Commands::Bindings { command },
            } = Cli::try_parse_from(["wabou", "bindings", name, "apps/gallery"]).unwrap()
            else {
                panic!("expected bindings command");
            };
            match command {
                BindingsCommand::Write { app } if name == "write" => {
                    assert_eq!(app.as_deref(), Some(Path::new("apps/gallery")));
                }
                BindingsCommand::Check { app } if name == "check" => {
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
                },
        } = Cli::try_parse_from([
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
        } = Cli::try_parse_from([
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
        assert!(Cli::try_parse_from(["wabou", "test", "--replay-test", "orphan"]).is_err());
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
            command: Commands::Package { app, format },
        } = Cli::try_parse_from([
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
    }

    #[test]
    fn rejects_the_removed_app_dir_flag() {
        assert!(Cli::try_parse_from(["wabou", "run", "--app-dir", "apps/gallery"]).is_err());
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
    fn recursively_stages_application_resources() {
        let root =
            env::temp_dir().join(format!("wabou-cli-package-resource-{}", std::process::id()));
        let source = root.join("assets/nested");
        let destination = root.join("stage/assets");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("fixture.txt"), "staged").unwrap();
        copy_resource(&root.join("assets"), &destination).unwrap();
        assert_eq!(
            fs::read_to_string(destination.join("nested/fixture.txt")).unwrap(),
            "staged"
        );
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
    }

    #[test]
    fn selects_the_facade_vite_feature_for_new_apps() {
        let metadata = serde_json::json!({
            "packages": [{
                "manifest_path": "/workspace/apps/gallery/Cargo.toml",
                "dependencies": [{"name": "wabou"}]
            }]
        });
        assert_eq!(
            vite_feature(&metadata, Path::new("/workspace/apps/gallery/Cargo.toml")),
            Some("wabou/vite")
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
        } = Cli::try_parse_from(["wabou", "run", "--release", "--profile-trace", "trace.json"])
            .unwrap()
        else {
            panic!("expected run command");
        };
        assert!(release);
        assert_eq!(profile_trace.as_deref(), Some(Path::new("trace.json")));
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
        } = Cli::try_parse_from(["wabou", "build", "--release", "--source-map"]).unwrap()
        else {
            panic!("expected build command");
        };
        assert!(release);
        assert_eq!(source_map, Some(true));

        let Cli {
            command: Commands::Build { source_map, .. },
        } = Cli::try_parse_from(["wabou", "build", "--source-map=false"]).unwrap()
        else {
            panic!("expected build command");
        };
        assert_eq!(source_map, Some(false));
    }

    #[test]
    fn selects_the_direct_runtime_vite_feature() {
        let metadata = serde_json::json!({
            "packages": [{
                "manifest_path": "/workspace/apps/runtime/Cargo.toml",
                "dependencies": [{"name": "wabou-runtime"}]
            }]
        });
        assert_eq!(
            vite_feature(&metadata, Path::new("/workspace/apps/runtime/Cargo.toml")),
            Some("wabou-runtime/vite")
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
            bundle_path(workspace, &app, BuildProfile::Debug),
            Path::new("/workspace/dist/gallery/debug/resources/bundle.js")
        );
        assert_eq!(
            bundle_path(workspace, &app, BuildProfile::Release),
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
            bundle_path(root, &app, BuildProfile::Debug),
            Path::new("/workspace/warden-desktop/dist/debug/resources/bundle.js")
        );
    }

    #[test]
    fn frontend_build_lock_excludes_a_second_process_handle() {
        let root = tempfile::tempdir().unwrap();
        let app = App {
            name: "app".into(),
            root: root.path().into(),
            frontend: root.path().into(),
            entry: "ui/index.tsx".into(),
        };
        let first = frontend_build_lock(root.path(), &app).unwrap();
        let second = OpenOptions::new()
            .read(true)
            .write(true)
            .open(root.path().join("target/wabou/frontend/app/frontend.lock"))
            .unwrap();
        assert!(!second.try_lock_exclusive().unwrap());
        drop(first);
        assert!(second.try_lock_exclusive().unwrap());
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
        assert!(message.contains("packages/vite/dist/index.mjs"));
        assert!(!message.contains("packages/vite/dist/index.d.mts"));
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
            bundle_path(&root, &app, BuildProfile::Release),
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
        assert!(configured_source_map(&app, BuildProfile::Debug));
        assert!(!configured_source_map(&app, BuildProfile::Release));

        fs::write(
            root.path().join("wabou.toml"),
            "[package]\nproduct-name = \"App\"\nidentifier = \"dev.wabou.app\"\n\n[build]\nout-dir = \"dist/resources\"\nsource-map = true\n",
        )
        .unwrap();
        assert!(configured_source_map(&app, BuildProfile::Release));
    }

    #[test]
    fn finds_a_sibling_devtools_helper_before_path() {
        let root = env::temp_dir().join(format!("wabou-cli-helper-{}", std::process::id()));
        let bin = root.join("bin");
        let path_bin = root.join("path-bin");
        fs::create_dir_all(&bin).unwrap();
        fs::create_dir_all(&path_bin).unwrap();
        let helper_name = format!("wabou-devtools{}", env::consts::EXE_SUFFIX);
        let sibling = bin.join(&helper_name);
        let path_helper = path_bin.join(&helper_name);
        fs::write(&sibling, []).unwrap();
        fs::write(&path_helper, []).unwrap();

        assert_eq!(
            find_helper(
                &bin.join(format!("wabou{}", env::consts::EXE_SUFFIX)),
                Some(path_bin.as_os_str()),
                "wabou-devtools",
            ),
            Some(sibling)
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn falls_back_to_path_for_the_devtools_helper() {
        let root = env::temp_dir().join(format!("wabou-cli-path-helper-{}", std::process::id()));
        let path_bin = root.join("path-bin");
        fs::create_dir_all(&path_bin).unwrap();
        let helper = path_bin.join(format!("wabou-devtools{}", env::consts::EXE_SUFFIX));
        fs::write(&helper, []).unwrap();

        assert_eq!(
            find_helper(
                &root.join("missing/wabou"),
                Some(path_bin.as_os_str()),
                "wabou-devtools",
            ),
            Some(helper)
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn capture_case_writer_emits_frame_matched_bundle() {
        let root = env::temp_dir().join(format!("wabou-cli-capture-{}", std::process::id()));
        let source = root.with_extension("png");
        fs::write(&source, b"png").unwrap();
        let capture = DebugCaptureCase {
            screenshot_path: source.clone(),
            snapshot: DebugSnapshot::default(),
            point: Some(DebugPointInspection {
                x: 1.0,
                y: 2.0,
                node: Some(DebugNode {
                    id: 42,
                    ..Default::default()
                }),
                ancestors: Vec::new(),
            }),
            frames: Vec::new(),
        };

        write_capture_case(&root, capture).unwrap();

        assert_eq!(fs::read(root.join("screenshot.png")).unwrap(), b"png");
        assert!(root.join("manifest.json").is_file());
        assert!(root.join("tree.json").is_file());
        assert!(root.join("selected-node.json").is_file());
        fs::remove_dir_all(root).unwrap();
        fs::remove_file(source).unwrap();
    }

    #[test]
    fn render_actions_preserve_cross_option_command_line_order() {
        let matches = Cli::command()
            .try_get_matches_from([
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
            ])
            .unwrap();

        assert_eq!(
            render_actions_from_matches(&matches).unwrap(),
            vec![
                RenderAction::Wheel([120.0, 700.0, 0.0, 650.0]),
                RenderAction::Click([75.0, 203.0]),
                RenderAction::Key("Enter".into()),
            ]
        );
    }
}
