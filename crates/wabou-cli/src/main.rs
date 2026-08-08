use std::env;
use std::error::Error;
use std::fs;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use clap::{Parser, Subcommand};
use serde_json::{Value, json};
use vello::Scene;
use wabou_devtools::{DebugCaptureCase, call, discover_socket, empty_params, request};
use wabou_quick::{AppConfig, Applier, JsRuntime, PasswordInput, SecretStore};
use wabou_shell::renderer::render_to_png;
use wabou_shell::scene as scene_builder;
use wabou_shell::{
    FrameSource, Modifiers, Point, PointerButton, PointerEvent, PointerPhase, TextContext, UiEvent,
    WheelEvent,
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[command(name = "wabou", version, about = "Build and run Wabou applications")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start Vite, the Rust host, and live HMR.
    Dev {
        #[arg(long, value_name = "PATH")]
        app_dir: Option<PathBuf>,
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
        #[arg(long, value_name = "PATH")]
        app_dir: Option<PathBuf>,
        #[arg(long)]
        release: bool,
    },
    /// Build the frontend bundle and run the Rust host.
    Run {
        #[arg(long, value_name = "PATH")]
        app_dir: Option<PathBuf>,
        #[arg(long)]
        release: bool,
    },
    /// Generate or verify Rust-owned TypeScript capability bindings.
    Bindings {
        #[arg(long, value_name = "PATH")]
        app_dir: Option<PathBuf>,
        #[command(subcommand)]
        command: BindingsCommand,
    },
    /// Render an application to a PNG without opening a native window.
    Render {
        #[arg(long, value_name = "PATH")]
        app_dir: Option<PathBuf>,
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
        /// Commit text to the element focused by the final --click before capture.
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Subcommand)]
enum BindingsCommand {
    /// Rewrite the committed TypeScript declarations.
    Write,
    /// Fail when the committed declarations differ from Rust types.
    Check,
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = env::current_dir()?;
    match cli.command {
        Commands::Dev {
            app_dir,
            port,
            devtools,
            mode,
        } => {
            let workspace = find_workspace(&cwd)?;
            dev(
                &workspace,
                load_app(&workspace, &cwd, app_dir.as_deref())?,
                port,
                devtools,
                mode.as_deref(),
            )
        }
        Commands::Build { app_dir, release } => {
            let workspace = find_workspace(&cwd)?;
            build(
                &workspace,
                &load_app(&workspace, &cwd, app_dir.as_deref())?,
                release,
            )
        }
        Commands::Run { app_dir, release } => {
            let workspace = find_workspace(&cwd)?;
            run(
                &workspace,
                &load_app(&workspace, &cwd, app_dir.as_deref())?,
                release,
            )
        }
        Commands::Bindings { app_dir, command } => {
            let workspace = find_workspace(&cwd)?;
            bindings(
                &workspace,
                &load_app(&workspace, &cwd, app_dir.as_deref())?,
                command,
            )
        }
        Commands::Render {
            app_dir,
            out,
            width,
            height,
            window_id,
            scale_factor,
            mode,
            wait_ms,
            click,
            wheel,
            text,
        } => {
            let workspace = find_workspace(&cwd)?;
            render(
                &workspace,
                &load_app(&workspace, &cwd, app_dir.as_deref())?,
                &out,
                width,
                height,
                window_id,
                scale_factor,
                mode.as_deref(),
                wait_ms,
                &click,
                &wheel,
                text.as_deref(),
            )
        }
        Commands::Devtools => run_devtools(&find_workspace(&cwd).unwrap_or(cwd)),
        Commands::Inspect { socket, command } => inspect(socket, command),
    }
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

fn load_app(workspace: &Path, cwd: &Path, app_dir: Option<&Path>) -> Result<App> {
    let root = match app_dir {
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

fn bundle_path(workspace: &Path, app: &App) -> PathBuf {
    workspace
        .join("dist")
        .join(&app.name)
        .join("resources/bundle.js")
}

fn frontend(app: &App, script: &str, args: &[&str]) -> Result<ExitStatus> {
    let mut command = Command::new("bun");
    command.current_dir(&app.frontend).args(["run", script]);
    if !args.is_empty() {
        command.arg("--").args(args);
    }
    Ok(command.status()?)
}

fn build(workspace: &Path, app: &App, release: bool) -> Result<()> {
    let manifest = manifest(app);
    let mut cargo = Command::new("cargo");
    cargo
        .current_dir(workspace)
        .args(["build", "--manifest-path", &manifest]);
    if release {
        cargo.arg("--release");
    }
    ensure(cargo.status()?, "Cargo build")?;
    ensure(frontend(app, "build", &[])?, "Vite build")?;
    package_executable(workspace, app, release)
}

fn run(workspace: &Path, app: &App, release: bool) -> Result<()> {
    ensure(frontend(app, "build", &[])?, "Vite build")?;
    let manifest = manifest(app);
    let binary = app_binary(workspace, app)?;
    let mut cargo = Command::new("cargo");
    cargo
        .current_dir(workspace)
        .args(["run", "--manifest-path", &manifest, "--bin", &binary]);
    if release {
        cargo.arg("--release");
    }
    cargo.env("WABOU_BUNDLE_PATH", bundle_path(workspace, app));
    ensure(cargo.status()?, "Rust host")
}

fn bindings(workspace: &Path, app: &App, mode: BindingsCommand) -> Result<()> {
    let manifest = manifest(app);
    let mode = match mode {
        BindingsCommand::Write => "write",
        BindingsCommand::Check => "check",
    };
    let mut cargo = Command::new("cargo");
    cargo.current_dir(workspace).args([
        "run",
        "--quiet",
        "--manifest-path",
        &manifest,
        "--example",
        "wabou-bindings",
        "--",
        mode,
    ]);
    ensure(cargo.status()?, "Wabou bindings generator")
}

#[allow(clippy::too_many_arguments)]
fn render(
    workspace: &Path,
    app: &App,
    out: &Path,
    width: u32,
    height: u32,
    window_id: u64,
    scale_factor: f64,
    mode: Option<&str>,
    wait_ms: u64,
    clicks: &[f64],
    wheels: &[f64],
    text: Option<&str>,
) -> Result<()> {
    if !scale_factor.is_finite() || scale_factor <= 0.0 {
        return Err("--scale-factor must be a finite number greater than zero".into());
    }
    let mode_args = mode.map(|mode| ["--mode", mode]);
    ensure(
        frontend(app, "build", mode_args.as_ref().map_or(&[], |args| args))?,
        "Vite build",
    )?;
    let path = bundle_path(workspace, app);
    let source = fs::read_to_string(&path).map_err(|error| {
        format!(
            "failed to read JavaScript bundle {}: {error}",
            path.display()
        )
    })?;
    let js =
        JsRuntime::new().map_err(|error| format!("cannot create JavaScript runtime: {error:?}"))?;

    let base_color = AppConfig::new("").base_color;
    // Install host globals before evaluating the bundle. Hooks such as
    // useWindow() read the logical id during module initialization, so booting
    // first would permanently expose the fallback window id 0 in screenshots.
    let mut factories = wabou_quick::widget::builtin_factories();
    factories.insert(
        "password-input".into(),
        Arc::new(|| Box::new(PasswordInput::new(SecretStore::default()))),
    );
    let mut applier =
        Applier::from_runtime_with_factories_and_window(js, factories, base_color, window_id);
    applier
        .boot(&source)
        .map_err(|error| format!("cannot boot JavaScript bundle: {error:?}"))?;
    applier.set_device_scale(scale_factor);
    let physical_width = (f64::from(width) * scale_factor)
        .round()
        .clamp(1.0, f64::from(u32::MAX)) as u32;
    let physical_height = (f64::from(height) * scale_factor)
        .round()
        .clamp(1.0, f64::from(u32::MAX)) as u32;
    applier.handle_event(UiEvent::WindowMetrics(wabou_shell::WindowMetrics {
        window_id,
        logical_width: width,
        logical_height: height,
        physical_width,
        physical_height,
        scale_factor,
        maximized: false,
        focused: true,
    }));
    let mut text_context = TextContext::new();
    let mut nodes = applier.build_frame(&mut text_context, width, height);

    if wait_ms > 0 {
        let deadline = Instant::now() + Duration::from_millis(wait_ms);
        while Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
            nodes = applier.build_frame(&mut text_context, width, height);
        }
    }

    for position in clicks.chunks_exact(2) {
        let point = Point {
            x: position[0],
            y: position[1],
        };
        applier.handle_event(UiEvent::Pointer(PointerEvent {
            phase: PointerPhase::Down,
            position: point,
            button: Some(PointerButton::Primary),
            buttons: 1,
            modifiers: Modifiers::default(),
        }));
        applier.handle_event(UiEvent::Pointer(PointerEvent {
            phase: PointerPhase::Up,
            position: point,
            button: Some(PointerButton::Primary),
            buttons: 0,
            modifiers: Modifiers::default(),
        }));
        for _ in 0..4 {
            nodes = applier.build_frame(&mut text_context, width, height);
        }
    }
    for gesture in wheels.chunks_exact(4) {
        applier.handle_event(UiEvent::Wheel(WheelEvent {
            position: Point {
                x: gesture[0],
                y: gesture[1],
            },
            delta_x: gesture[2],
            delta_y: gesture[3],
            modifiers: Modifiers::default(),
        }));
        for _ in 0..4 {
            nodes = applier.build_frame(&mut text_context, width, height);
        }
    }
    if let Some(text) = text {
        applier.handle_event(UiEvent::TextInput(text.to_owned()));
        for _ in 0..4 {
            nodes = applier.build_frame(&mut text_context, width, height);
        }
    }

    if let Some(parent) = out.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let mut scene = Scene::new();
    scene_builder::build_scene_scaled(
        &mut scene,
        &nodes,
        &mut text_context,
        width,
        height,
        base_color,
        scale_factor,
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
    let port_text = port.to_string();
    let mut vite_command = Command::new("bun");
    vite_command
        .current_dir(&app.frontend)
        .args(["run", "dev", "--", "--port", &port_text, "--strictPort"])
        .stdin(Stdio::null());
    if let Some(mode) = mode {
        vite_command.args(["--mode", mode]);
    }
    let mut vite = vite_command.spawn()?;
    let url = format!("http://127.0.0.1:{port}");
    wait_for_vite(&url, &mut vite)?;

    let app_manifest = manifest(&app);
    let binary = app_binary(workspace, &app)?;
    let mut host = Command::new("cargo")
        .current_dir(workspace)
        .args([
            "run",
            "--manifest-path",
            &app_manifest,
            "--bin",
            &binary,
            "--features",
            "wabou-quick/vite",
        ])
        .env("WABOU_VITE_URL", &url)
        .env("WABOU_VITE_ENTRY", &app.entry)
        .spawn()?;

    let mut inspector = if open_devtools {
        Some(devtools_command(workspace)?.spawn()?)
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
        ensure(frontend(&app, "build", &[])?, "DevTools Vite build")?;
        let manifest = manifest(&app);
        let mut cargo = Command::new("cargo");
        cargo
            .current_dir(workspace)
            .args(["build", "--manifest-path", &manifest]);
        ensure(cargo.status()?, "DevTools Rust build")?;
        let executable = built_executable(workspace, &app, false)?;
        let mut command = Command::new(executable);
        command.env("WABOU_BUNDLE_PATH", bundle_path(workspace, &app));
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
    let destination_dir = workspace.join("dist").join(&app.name);
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
    let package = metadata["packages"].as_array()?.iter().find(|package| {
        package["manifest_path"]
            .as_str()
            .is_some_and(|path| Path::new(path) == manifest_path)
    })?;
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

fn wait_for_vite(url: &str, child: &mut Child) -> Result<()> {
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

fn supervise(host: &mut Child, vite: &mut Child, inspector: Option<&mut Child>) -> Result<()> {
    let stopped = Arc::new(AtomicBool::new(false));
    let signal = stopped.clone();
    ctrlc::set_handler(move || signal.store(true, Ordering::Release))?;
    let mut inspector = inspector;
    let result = loop {
        if stopped.load(Ordering::Acquire) {
            break Ok(());
        }
        if let Some(status) = host.try_wait()? {
            break ensure(status, "Rust host");
        }
        if let Some(status) = vite.try_wait()? {
            break ensure(status, "Vite dev server");
        }
        if let Some(child) = inspector.as_deref_mut()
            && let Some(status) = child.try_wait()?
        {
            eprintln!("[wabou] DevTools exited: {status}");
            inspector = None;
        }
        thread::sleep(Duration::from_millis(50));
    };
    terminate(host);
    terminate(vite);
    if let Some(child) = inspector {
        terminate(child);
    }
    result
}

fn terminate(child: &mut Child) {
    if child.try_wait().ok().flatten().is_none() {
        let _ = child.kill();
        let _ = child.wait();
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

        let Cli {
            command:
                Commands::Render {
                    window_id,
                    scale_factor,
                    mode,
                    click,
                    wheel,
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
    }

    #[test]
    fn parses_bindings_write_and_check_commands() {
        for (name, expected) in [
            ("write", BindingsCommand::Write),
            ("check", BindingsCommand::Check),
        ] {
            let Cli {
                command: Commands::Bindings { app_dir, command },
            } = Cli::try_parse_from(["wabou", "bindings", "--app-dir", "apps/gallery", name])
                .unwrap()
            else {
                panic!("expected bindings command");
            };
            assert_eq!(app_dir.as_deref(), Some(Path::new("apps/gallery")));
            assert_eq!(command, expected);
        }
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
    fn resolves_the_real_multi_binary_application() {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let app = load_app(
            &workspace,
            &workspace,
            Some(Path::new("apps/warden-desktop")),
        )
        .unwrap();
        assert_eq!(app_binary(&workspace, &app).unwrap(), "warden-desktop");
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
            bundle_path(workspace, &app),
            Path::new("/workspace/dist/gallery/resources/bundle.js")
        );
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
}
