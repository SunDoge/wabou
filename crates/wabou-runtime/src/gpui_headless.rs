//! Deterministic GPUI harness for bundle-level layout and interaction tests.

use std::{collections::HashMap, path::Path, rc::Rc, sync::Arc, time::Duration};

use gpui_shell::gpui::{AppContext as _, HeadlessAppContext, px, size};
use snafu::ResultExt as _;

use crate::{GpuiRuntimeView, JsRuntime, WindowOptions};

/// Configuration for one hidden GPUI test window.
#[derive(Clone, Debug)]
pub struct GpuiHeadlessOptions {
    /// Native window options visible to JavaScript during boot.
    pub window: WindowOptions,
    /// Maximum number of GPUI draw cycles used to settle initial effects.
    pub settle_frames: usize,
}

impl Default for GpuiHeadlessOptions {
    fn default() -> Self {
        Self {
            window: WindowOptions::new()
                .title("Wabou headless GPUI fixture")
                .initial_inner_size(1_440, 900),
            settle_frames: 8,
        }
    }
}

/// Result published after the hidden window completed GPUI layout.
#[derive(Debug)]
pub struct GpuiHeadlessOutput {
    /// Bounds and retained metadata produced by GPUI's real prepaint pass.
    pub layout: Vec<gpui_shell::GpuiLayoutNode>,
    /// Number of Solid protocol revisions committed before the snapshot.
    pub protocol_revision: u64,
    /// Logical viewport width used by GPUI layout.
    pub viewport_width: u32,
    /// Logical viewport height used by GPUI layout.
    pub viewport_height: u32,
    /// Device scale reported by the GPUI test platform.
    pub scale_factor: f64,
}

/// Outcome of a platform pixel-capture request.
#[derive(Debug)]
pub enum GpuiHeadlessScreenshot {
    /// RGBA pixels rendered by the platform's GPUI headless renderer.
    Image(image::RgbaImage),
    /// This GPUI platform does not currently expose a headless renderer.
    Unsupported,
}

/// A single-bundle GPUI runtime used by CLI and component fixtures.
pub struct GpuiHeadlessHarness {
    context: HeadlessAppContext,
    window: gpui_shell::gpui::WindowHandle<GpuiRuntimeView>,
}

impl GpuiHeadlessHarness {
    /// Boot a production bundle in QuickJS and mount it into a hidden GPUI window.
    pub fn boot(
        source: impl Into<Arc<str>>,
        source_map: Option<impl Into<Arc<[u8]>>>,
        options: GpuiHeadlessOptions,
    ) -> crate::Result<Self> {
        let source = source.into();
        let source_map = source_map.map(Into::into);
        let platform = gpui_platform::current_platform(true);
        let mut context = HeadlessAppContext::with_platform(
            platform.text_system(),
            Arc::new(()),
            gpui_platform::current_headless_renderer,
        );
        context.update(gpui_base::init);

        let window_key = gpui_shell::initial_window_resource_key(0);
        let controller =
            create_controller(window_key, &options.window, &source, source_map.as_deref())?;
        let dynamic_source = source.clone();
        let dynamic_source_map = source_map.clone();
        let window_host = crate::gpui_windows::GpuiApplicationWindows::new(
            Rc::new(move |key, window_options| {
                create_controller(
                    key,
                    window_options,
                    &dynamic_source,
                    dynamic_source_map.as_deref(),
                )
                .map_err(|error| error.to_string())
            }),
            HashMap::new(),
            None,
        );
        let view_window_host = window_host.clone();
        let title = options.window.title.clone();
        let window = context
            .open_window(
                size(
                    px(options.window.initial_inner_size.0 as f32),
                    px(options.window.initial_inner_size.1 as f32),
                ),
                move |window, app| {
                    app.new(|cx| {
                        GpuiRuntimeView::new(
                            controller,
                            crate::gpui_view::GpuiRuntimeViewOptions {
                                default_title: title,
                                window_size_persistence: None,
                                native_widget_factories: HashMap::new(),
                                test_controller: None,
                                window_key,
                                window_host: view_window_host,
                            },
                            window,
                            cx,
                        )
                    })
                },
            )
            .map_err(|error| crate::Error::GpuiShell {
                message: format!("failed to open hidden GPUI window: {error}"),
            })?;
        let _ = window_host.attach(window_key, window.into());

        let mut harness = Self { context, window };
        harness.settle(options.settle_frames.max(1))?;
        Ok(harness)
    }

    /// Run pending JavaScript work and force GPUI layout/prepaint cycles.
    pub fn settle(&mut self, frames: usize) -> crate::Result<()> {
        for _ in 0..frames {
            self.context.run_until_parked();
            self.context
                .update_window(self.window.into(), |_, window, app| {
                    let _ = window.draw(app);
                })
                .map_err(|error| crate::Error::GpuiShell {
                    message: format!("failed to draw hidden GPUI window: {error}"),
                })?;
        }
        Ok(())
    }

    /// Advance the deterministic GPUI clock and publish the resulting frame.
    pub fn advance_time(&mut self, duration: Duration) -> crate::Result<()> {
        self.context.advance_clock(duration);
        self.settle(1)
    }

    /// Evaluate fixture setup code inside the owned QuickJS runtime.
    pub fn eval_script(&mut self, source: &str) -> crate::Result<()> {
        let root = self.root()?;
        self.context
            .read_entity(&root, |view, _| view.eval_script_diagnostic(source))
            .map_err(|message| crate::Error::GpuiShell { message })
    }

    /// Evaluate JavaScript and return its string result.
    pub fn eval_string(&mut self, source: &str) -> crate::Result<String> {
        let root = self.root()?;
        self.context
            .read_entity(&root, |view, _| view.eval_string(source))
            .context(crate::error::JavaScriptSnafu {
                operation: "evaluate headless JavaScript",
            })
    }

    /// Read layout generated by GPUI's latest prepaint pass.
    pub fn snapshot(&mut self) -> crate::Result<GpuiHeadlessOutput> {
        let root = self.root()?;
        let (viewport_width, viewport_height, scale_factor) = self
            .context
            .update_window(self.window.into(), |_, window, _| {
                let viewport = window.viewport_size();
                (
                    f32::from(viewport.width).round() as u32,
                    f32::from(viewport.height).round() as u32,
                    f64::from(window.scale_factor()),
                )
            })
            .map_err(|error| crate::Error::GpuiShell {
                message: format!("failed to read hidden GPUI viewport: {error}"),
            })?;
        Ok(self
            .context
            .read_entity(&root, |view, _| GpuiHeadlessOutput {
                layout: view.layout_snapshot(),
                protocol_revision: view.protocol_revision(),
                viewport_width,
                viewport_height,
                scale_factor,
            }))
    }

    /// Capture pixels when GPUI supplies a renderer for the current platform.
    pub fn screenshot(&mut self) -> crate::Result<GpuiHeadlessScreenshot> {
        match self.context.capture_screenshot(self.window.into()) {
            Ok(image) => Ok(GpuiHeadlessScreenshot::Image(image)),
            Err(error) if error.to_string().contains("no HeadlessRenderer configured") => {
                Ok(GpuiHeadlessScreenshot::Unsupported)
            }
            Err(error) => Err(crate::Error::GpuiShell {
                message: format!("failed to capture hidden GPUI window: {error}"),
            }),
        }
    }

    /// Save a PNG when pixel capture is supported.
    pub fn save_png(&mut self, path: impl AsRef<Path>) -> crate::Result<bool> {
        let GpuiHeadlessScreenshot::Image(image) = self.screenshot()? else {
            return Ok(false);
        };
        image
            .save(path.as_ref())
            .map_err(|error| crate::Error::GpuiShell {
                message: format!(
                    "failed to save GPUI screenshot {}: {error}",
                    path.as_ref().display()
                ),
            })?;
        Ok(true)
    }

    fn root(&mut self) -> crate::Result<gpui_shell::gpui::Entity<GpuiRuntimeView>> {
        self.window
            .root(&mut self.context)
            .map_err(|error| crate::Error::GpuiShell {
                message: format!("failed to read hidden GPUI root: {error}"),
            })
    }
}

fn create_controller(
    window_key: gpui_shell::WindowResourceKey,
    window_options: &WindowOptions,
    source: &str,
    source_map: Option<&[u8]>,
) -> crate::Result<crate::gpui_controller::GpuiController> {
    let js = JsRuntime::new().context(crate::error::JavaScriptSnafu {
        operation: "create headless JavaScript runtime",
    })?;
    let serialized = serde_json::to_string(window_options).expect("WindowOptions is serializable");
    js.with(|ctx| ctx.globals().set("__wabou_window_options_json", serialized))
        .context(crate::error::JavaScriptSnafu {
            operation: "install headless window options",
        })?;
    let mut controller = crate::gpui_controller::GpuiController::new(
        crate::runtime_session::RuntimeSession::new(js, window_key),
    );
    controller
        .boot_with_source_map(source, source_map)
        .context(crate::error::JavaScriptSnafu {
            operation: "boot headless JavaScript bundle",
        })?;
    Ok(controller)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wabou_host_api::NodeKey;

    #[test]
    fn production_protocol_bundle_reaches_real_gpui_layout() {
        let mut harness = GpuiHeadlessHarness::boot(
            include_str!("gen/test-runtime.js"),
            None::<Arc<[u8]>>,
            GpuiHeadlessOptions {
                window: WindowOptions::new().initial_inner_size(800, 600),
                settle_frames: 2,
            },
        )
        .expect("boot GPUI headless fixture");
        let output = harness.snapshot().expect("read GPUI layout");
        assert_eq!(output.protocol_revision, 1);
        assert!(
            output
                .layout
                .iter()
                .any(|node| node.key == NodeKey::new(2, 1))
        );
        let root = output
            .layout
            .iter()
            .find(|node| node.key == NodeKey::ROOT)
            .expect("synthetic root has layout bounds");
        assert_eq!(root.bounds.size, size(px(800.0), px(600.0)));
    }
}
