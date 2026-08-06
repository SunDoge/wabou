//! wabou-quick CLI: run a SolidJS bundle under the wabou renderer.
//!
//! Static (prod) path: `--js bundle.js`. The bundle carries its own Style IR
//! (via the `virtual:wabou-stylesheet` module → the private host ABI).
//! Dev (vite) path (feature `vite`): `--vite http://localhost:5173 --entry src/index.tsx`.

use clap::{Parser, ValueEnum};
use snafu::{OptionExt, ResultExt};
use std::path::PathBuf;
use vello::Scene;
use wabou_quick::{AppConfig, Applier, JsRuntime};
use wabou_shell::renderer::render_to_png;
use wabou_shell::scene as scene_builder;
use wabou_shell::{
    FrameSource, Modifiers, Point, PointerButton, PointerEvent, PointerPhase, TextContext, UiEvent,
    run_window_with_size,
};

#[derive(Parser)]
#[command(
    name = "wabou-quick",
    about = "Run a SolidJS bundle under the wabou renderer"
)]
struct Args {
    /// Static-bundle JS path (prod mode). Mutually exclusive with --vite.
    #[arg(long)]
    js: Option<String>,

    /// Vite dev-server URL (dev mode). Requires the `vite` feature.
    #[cfg(feature = "vite")]
    #[arg(long)]
    vite: Option<String>,

    /// Vite entry module (dev mode), e.g. `src/index.tsx`.
    #[cfg(feature = "vite")]
    #[arg(long, default_value = "src/index.tsx")]
    entry: String,

    #[arg(long, value_enum, default_value_t = Mode::Window)]
    mode: Mode,

    #[arg(long, default_value = "out.png")]
    out: String,

    #[arg(long)]
    width: Option<u32>,

    #[arg(long)]
    height: Option<u32>,

    /// Keep driving async JS work before capturing a PNG (useful for fetch).
    #[arg(long, default_value_t = 0)]
    wait_ms: u64,

    /// Dispatch a primary click at X Y after the optional async wait.
    #[arg(long, num_args = 2, value_names = ["X", "Y"])]
    click: Option<Vec<f64>>,

    /// Commit text to the element focused by --click before capture.
    #[arg(long, requires = "click")]
    text: Option<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum Mode {
    Window,
    Png,
}

fn read_bundle(path: &str) -> wabou_quick::Result<String> {
    std::fs::read_to_string(path).context(wabou_quick::error::ReadFileSnafu {
        kind: "JavaScript bundle",
        path: PathBuf::from(path),
    })
}

#[snafu::report]
fn main() -> wabou_quick::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init()
        .ok();

    let args = Args::parse();
    let (w, h) = (args.width.unwrap_or(800), args.height.unwrap_or(600));

    // Build the JsRuntime + boot it, then hand it to the Applier.
    #[cfg(feature = "vite")]
    let mut applier = if let Some(vite_url) = args.vite.as_deref() {
        let mut js =
            JsRuntime::new_vite(vite_url).context(wabou_quick::error::JavaScriptSnafu {
                operation: "create Vite JavaScript runtime",
            })?;
        js.boot_vite(&args.entry)
            .context(wabou_quick::error::JavaScriptSnafu {
                operation: "boot Vite entry module",
            })?;
        let mut a = Applier::from_runtime(js, AppConfig::new("").base_color);
        a.set_vite_entry(&args.entry);
        let reload = a.reload_handle();
        let _hmr = wabou_quick::start_hmr_client(vite_url, reload)
            .context(wabou_quick::error::ViteSnafu)?;
        a
    } else {
        let js_path = args
            .js
            .as_deref()
            .context(wabou_quick::error::MissingArgumentSnafu {
                argument: "either --vite or --js",
            })?;
        let js_src = read_bundle(js_path)?;
        let mut js = JsRuntime::new().context(wabou_quick::error::JavaScriptSnafu {
            operation: "create JavaScript runtime",
        })?;
        js.boot(&js_src)
            .context(wabou_quick::error::JavaScriptSnafu {
                operation: "boot JavaScript bundle",
            })?;
        Applier::from_runtime(js, AppConfig::new("").base_color)
    };
    #[cfg(not(feature = "vite"))]
    let mut applier = {
        let js_path = args
            .js
            .as_deref()
            .context(wabou_quick::error::MissingArgumentSnafu {
                argument: "--js <path>",
            })?;
        let js_src = read_bundle(js_path)?;
        let mut js = JsRuntime::new().context(wabou_quick::error::JavaScriptSnafu {
            operation: "create JavaScript runtime",
        })?;
        js.boot(&js_src)
            .context(wabou_quick::error::JavaScriptSnafu {
                operation: "boot JavaScript bundle",
            })?;
        Applier::from_runtime(js, AppConfig::new("").base_color)
    };

    let base_color = AppConfig::new("").base_color;

    match args.mode {
        Mode::Window => {
            println!("[wabou-quick] window mode: {w}x{h}");
            run_window_with_size(Box::new(applier), w, h)
                .context(wabou_quick::error::ShellSnafu)?;
        }
        Mode::Png => {
            println!("[wabou-quick] png mode: {w}x{h} -> {}", args.out);
            let mut tcx = TextContext::new();
            let mut nodes = applier.build_frame(&mut tcx, w, h);
            if args.wait_ms > 0 {
                let deadline =
                    std::time::Instant::now() + std::time::Duration::from_millis(args.wait_ms);
                while std::time::Instant::now() < deadline {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    nodes = applier.build_frame(&mut tcx, w, h);
                }
            }
            if let Some(position) = &args.click {
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
                if let Some(text) = &args.text {
                    applier.handle_event(UiEvent::TextInput(text.clone()));
                }
                // Event handlers update Solid synchronously; the following
                // ticks flush protocol mutations and any router microtasks.
                for _ in 0..4 {
                    nodes = applier.build_frame(&mut tcx, w, h);
                }
            }
            let mut scene = Scene::new();
            scene_builder::build_scene(&mut scene, &nodes, &mut tcx, w, h, base_color);
            render_to_png(&scene, w, h, base_color, &args.out)
                .context(wabou_quick::error::ShellSnafu)?;
            println!("[wabou-quick] wrote {}", args.out);
        }
    }
    Ok(())
}
