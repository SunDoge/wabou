//! Deterministic GPUI harness for bundle-level layout and interaction tests.

use std::{collections::HashMap, path::Path, rc::Rc, sync::Arc, time::Duration};

use snafu::ResultExt as _;
use wabou_shell::gpui::{
    AppContext as _, HeadlessAppContext, Keystroke, Modifiers, MouseButton, MouseDownEvent,
    MouseUpEvent, PlatformInput, ScrollDelta, ScrollWheelEvent, TouchPhase, point, px,
};

use crate::{JsRuntime, WindowOptions, gpui_view::GpuiRuntimeView};

const HEADLESS_FRAME_DURATION: Duration = Duration::from_micros(16_667);

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
    pub layout: Vec<wabou_shell::GpuiLayoutNode>,
    /// Number of Solid protocol revisions committed before the snapshot.
    pub protocol_revision: u64,
    /// Logical viewport width used by GPUI layout.
    pub viewport_width: u32,
    /// Logical viewport height used by GPUI layout.
    pub viewport_height: u32,
    /// Device scale reported by the GPUI test platform.
    pub scale_factor: f64,
}

/// One observable retained boundary in a deterministic GPUI checkpoint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GpuiProjectionBoundaryCheckpoint {
    /// Generational protocol identity of the boundary root.
    pub root: wabou_shell::NodeKey,
    /// Independent structure, layout, and paint revision clocks.
    pub revision: wabou_shell::ProjectionBoundaryRevision,
    /// Number of times GPUI requested this boundary's Render implementation.
    pub materializations: u64,
    /// Retained protocol nodes recursively owned by this boundary.
    pub owned_nodes: usize,
}

/// Incremental-work checkpoint from the real QuickJS → GPUI projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GpuiProjectionCheckpoint {
    /// Last completed Solid protocol revision.
    pub protocol_revision: u64,
    /// Stable boundary observations sorted by root NodeKey.
    pub boundaries: Vec<GpuiProjectionBoundaryCheckpoint>,
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
    window: wabou_shell::gpui::WindowHandle<GpuiRuntimeView>,
    runtime_clock: Option<Arc<crate::clock::ManualClock>>,
    _native_close_subscription: wabou_shell::gpui::Subscription,
}

impl GpuiHeadlessHarness {
    /// Boot a production bundle in QuickJS and mount it into a hidden GPUI window.
    pub fn boot(
        source: impl Into<Arc<str>>,
        source_map: Option<impl Into<Arc<[u8]>>>,
        options: GpuiHeadlessOptions,
    ) -> crate::Result<Self> {
        Self::boot_with_native_widgets(source, source_map, options, HashMap::new())
    }

    /// Boot a production bundle with the same native-widget registry used by an application.
    ///
    /// This keeps component and layout tests on the real GPUI materialization path instead of
    /// replacing application-owned widgets with test-only stand-ins. Factories are shared with
    /// every window created by the tested bundle.
    pub fn boot_with_native_widgets(
        source: impl Into<Arc<str>>,
        source_map: Option<impl Into<Arc<[u8]>>>,
        options: GpuiHeadlessOptions,
        native_widget_factories: HashMap<String, wabou_shell::NativeWidgetFactory>,
    ) -> crate::Result<Self> {
        let source = source.into();
        let source_map = source_map.map(Into::into);
        let dynamic_source = source.clone();
        let dynamic_source_map = source_map.clone();
        let dynamic_native_widget_factories = native_widget_factories.clone();
        let runtime_clock = Arc::new(crate::clock::ManualClock::default());
        let dynamic_runtime_clock = runtime_clock.clone();
        let window_host = crate::gpui_windows::GpuiApplicationWindows::new(
            Rc::new(move |key, window_options| {
                create_controller_with_clock(
                    key,
                    window_options,
                    &dynamic_source,
                    dynamic_source_map.as_deref(),
                    dynamic_runtime_clock.clone(),
                )
                .map_err(|error| error.to_string())
            }),
            dynamic_native_widget_factories,
            None,
        );
        let window_key = window_host.reserve();
        debug_assert_eq!(window_key, wabou_shell::initial_window_resource_key(0));
        let controller = create_controller_with_clock(
            window_key,
            &options.window,
            &source,
            source_map.as_deref(),
            runtime_clock.clone(),
        )?;
        let mut harness = Self::boot_application_with_clock(
            vec![(window_key, controller, options.window.clone())],
            window_host,
            Some(runtime_clock),
        )?;
        harness.settle(options.settle_frames.max(1))?;
        Ok(harness)
    }

    pub(crate) fn boot_application(
        windows: Vec<(
            wabou_shell::WindowResourceKey,
            crate::gpui_controller::GpuiController,
            WindowOptions,
        )>,
        window_host: Rc<crate::gpui_windows::GpuiApplicationWindows>,
    ) -> crate::Result<Self> {
        Self::boot_application_with_clock(windows, window_host, None)
    }

    fn boot_application_with_clock(
        windows: Vec<(
            wabou_shell::WindowResourceKey,
            crate::gpui_controller::GpuiController,
            WindowOptions,
        )>,
        window_host: Rc<crate::gpui_windows::GpuiApplicationWindows>,
        runtime_clock: Option<Arc<crate::clock::ManualClock>>,
    ) -> crate::Result<Self> {
        let platform = gpui_platform::current_platform(true);
        let mut context = HeadlessAppContext::with_platform(
            platform.text_system(),
            Arc::new(()),
            gpui_platform::current_headless_renderer,
        );
        let native_close_subscription = context.update(|cx| {
            gpui_base::init(cx);
            gpui_base::Theme::global_mut(cx).scrollbar = gpui_base::ScrollbarTheme::new()
                .with_motion(
                    gpui_base::ScrollbarMotion::default()
                        .with_idle(Duration::from_millis(500))
                        .with_exit(Duration::from_millis(200)),
                );
            window_host.observe_native_closes(cx)
        });
        let mut primary = None;
        for (index, (window_key, controller, options)) in windows.into_iter().enumerate() {
            let handle = context
                .update(|cx| window_host.open_controller(window_key, controller, options, None, cx))
                .map_err(|error| crate::Error::GpuiShell {
                    message: format!("failed to open hidden GPUI window: {error}"),
                })?;
            if index == 0 {
                primary = handle.downcast::<GpuiRuntimeView>();
            }
        }
        let window = primary.ok_or_else(|| crate::Error::GpuiShell {
            message: "headless GPUI application did not open a primary runtime window".into(),
        })?;
        Ok(Self {
            context,
            window,
            runtime_clock,
            _native_close_subscription: native_close_subscription,
        })
    }

    /// Run pending JavaScript work and force GPUI layout/prepaint cycles.
    pub fn settle(&mut self, frames: usize) -> crate::Result<()> {
        for _ in 0..frames {
            if let Some(clock) = &self.runtime_clock {
                clock.advance(HEADLESS_FRAME_DURATION);
                self.context.advance_clock(HEADLESS_FRAME_DURATION);
            }
            self.settle_frame()?;
        }
        Ok(())
    }

    /// Advance pending runtime work without forcing GPUI to discard cached views.
    ///
    /// Use this for incremental projection tests. [`Self::settle`] deliberately
    /// calls `Window::refresh` so layout and pixel fixtures always repaint, but
    /// GPUI defines that operation as bypassing every cached `View`.
    pub fn settle_incremental(&mut self, frames: usize) -> crate::Result<()> {
        for _ in 0..frames {
            if let Some(clock) = &self.runtime_clock {
                clock.advance(HEADLESS_FRAME_DURATION);
                self.context.advance_clock(HEADLESS_FRAME_DURATION);
            }
            self.draw_incremental()?;
        }
        Ok(())
    }

    /// Set the platform motion preference exposed to GPUI and JavaScript.
    ///
    /// Layout contracts normally enable reduced motion so geometry can settle
    /// without turning a component test into a multi-frame paint benchmark.
    /// Dedicated animation tests leave it disabled and advance frames
    /// explicitly.
    pub fn set_reduced_motion(&mut self, reduced_motion: bool) -> crate::Result<()> {
        self.context.update(|app| {
            app.set_reduce_motion(reduced_motion);
        });
        self.settle_frame()
    }

    fn settle_frame(&mut self) -> crate::Result<()> {
        // `run_until_parked` cannot be used while JavaScript owns an active
        // requestAnimationFrame loop: the callback schedules the next frame,
        // but GPUI's deterministic clock does not advance while the scheduler
        // is draining, so even a finite animation spins forever at one time.
        // Do not drain GPUI's executor here. One ready task can itself enter a
        // continuous refresh chain when JavaScript owns requestAnimationFrame,
        // so even a one-task bound does not define one deterministic frame.
        // The explicit draw below advances the runtime exactly once; callers
        // that need more progress settle more frames.
        // `GpuiRuntimeView::render` publishes the previous completed
        // prepaint bounds before advancing JavaScript. Keep that live
        // ordering intact: publishing outside render would consume the
        // resulting projection invalidation before the boundary decides
        // whether it must rebuild.
        self.context
            .update_window(self.window.into(), |_, window, app| {
                window.refresh();
                let _ = window.draw(app);
            })
            .map_err(|error| crate::Error::GpuiShell {
                message: format!("failed to draw hidden GPUI window: {error}"),
            })?;
        Ok(())
    }

    fn draw_incremental(&mut self) -> crate::Result<()> {
        self.context
            .update_window(self.window.into(), |_, window, app| {
                let _ = window.draw(app);
            })
            .map_err(|error| crate::Error::GpuiShell {
                message: format!("failed to incrementally draw hidden GPUI window: {error}"),
            })?;
        Ok(())
    }

    /// Advance the deterministic GPUI clock and publish the resulting frame.
    pub fn advance_time(&mut self, duration: Duration) -> crate::Result<()> {
        if let Some(clock) = &self.runtime_clock {
            clock.advance(duration);
        }
        self.context.advance_clock(duration);
        self.settle_frame()
    }

    /// Dispatch a native GPUI primary-button click in logical window coordinates.
    pub fn click(&mut self, x: f32, y: f32) -> crate::Result<()> {
        let position = point(px(x), px(y));
        self.context
            .update_window(self.window.into(), |_, window, app| {
                window.dispatch_event(
                    PlatformInput::MouseDown(MouseDownEvent {
                        position,
                        button: MouseButton::Left,
                        modifiers: Modifiers::default(),
                        click_count: 1,
                        first_mouse: false,
                    }),
                    app,
                );
                window.dispatch_event(
                    PlatformInput::MouseUp(MouseUpEvent {
                        position,
                        button: MouseButton::Left,
                        modifiers: Modifiers::default(),
                        click_count: 1,
                    }),
                    app,
                );
            })
            .map_err(|error| crate::Error::GpuiShell {
                message: format!("failed to dispatch headless GPUI click: {error}"),
            })?;
        self.settle(1)
    }

    /// Dispatch a native GPUI pixel-wheel event in logical window coordinates.
    pub fn wheel(&mut self, x: f32, y: f32, delta_x: f32, delta_y: f32) -> crate::Result<()> {
        self.context
            .update_window(self.window.into(), |_, window, app| {
                window.dispatch_event(
                    PlatformInput::ScrollWheel(ScrollWheelEvent {
                        position: point(px(x), px(y)),
                        delta: ScrollDelta::Pixels(point(px(delta_x), px(delta_y))),
                        modifiers: Modifiers::default(),
                        touch_phase: TouchPhase::Moved,
                    }),
                    app,
                );
            })
            .map_err(|error| crate::Error::GpuiShell {
                message: format!("failed to dispatch headless GPUI wheel event: {error}"),
            })?;
        self.settle(1)
    }

    /// Type text through GPUI's simulated platform/IME path.
    pub fn type_text(&mut self, text: &str) -> crate::Result<()> {
        for character in text.chars() {
            let key = character.to_string();
            self.dispatch_keystroke(Keystroke {
                modifiers: Modifiers::default(),
                key: key.clone(),
                key_char: Some(key),
            })?;
        }
        Ok(())
    }

    /// Dispatch one GPUI keystroke using GPUI's canonical key syntax.
    pub fn key(&mut self, key: &str) -> crate::Result<()> {
        let keystroke = Keystroke::parse(key).map_err(|error| crate::Error::GpuiShell {
            message: format!("invalid GPUI keystroke `{key}`: {error}"),
        })?;
        self.dispatch_keystroke(keystroke)
    }

    fn dispatch_keystroke(&mut self, keystroke: Keystroke) -> crate::Result<()> {
        self.context
            .update_window(self.window.into(), |_, window, app| {
                window.dispatch_keystroke(keystroke, app);
            })
            .map_err(|error| crate::Error::GpuiShell {
                message: format!("failed to dispatch headless GPUI keystroke: {error}"),
            })?;
        self.settle(1)
    }

    /// Evaluate fixture setup code inside the owned QuickJS runtime.
    pub fn eval_script(&mut self, source: &str) -> crate::Result<()> {
        let root = self.root()?;
        self.context
            .update_entity(&root, |view, _| view.eval_script_diagnostic(source))
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

    /// Observe incremental projection work without constructing a second test
    /// renderer. Call this before and after `eval_script` + `settle` to prove
    /// which real GPUI boundary was invalidated and materialized.
    pub fn projection_checkpoint(&mut self) -> crate::Result<GpuiProjectionCheckpoint> {
        let root = self.root()?;
        Ok(self
            .context
            .read_entity(&root, |view, cx| view.projection_checkpoint(cx)))
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

    fn root(&mut self) -> crate::Result<wabou_shell::gpui::Entity<GpuiRuntimeView>> {
        self.window
            .root(&mut self.context)
            .map_err(|error| crate::Error::GpuiShell {
                message: format!("failed to read hidden GPUI root: {error}"),
            })
    }
}

fn create_controller_with_clock(
    window_key: wabou_shell::WindowResourceKey,
    window_options: &WindowOptions,
    source: &str,
    source_map: Option<&[u8]>,
    clock: Arc<crate::clock::ManualClock>,
) -> crate::Result<crate::gpui_controller::GpuiController> {
    create_controller_with_runtime(
        window_key,
        window_options,
        source,
        source_map,
        JsRuntime::new_with_clock(clock),
    )
}

fn create_controller_with_runtime(
    window_key: wabou_shell::WindowResourceKey,
    window_options: &WindowOptions,
    source: &str,
    source_map: Option<&[u8]>,
    js: rquickjs::Result<JsRuntime>,
) -> crate::Result<crate::gpui_controller::GpuiController> {
    let js = js.context(crate::error::JavaScriptSnafu {
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    use wabou_host_api::NodeKey;
    use wabou_shell::gpui::{
        InteractiveElement as _, IntoElement as _, ParentElement as _, Styled as _, div, size,
    };

    #[derive(Default)]
    struct InputCounts {
        clicks: AtomicUsize,
        keys: AtomicUsize,
        wheels: AtomicUsize,
    }

    struct InputProbe {
        counts: Arc<InputCounts>,
        focus: wabou_shell::gpui::FocusHandle,
    }

    impl wabou_shell::gpui::Render for InputProbe {
        fn render(
            &mut self,
            _window: &mut wabou_shell::gpui::Window,
            _cx: &mut wabou_shell::gpui::Context<Self>,
        ) -> impl wabou_shell::gpui::IntoElement {
            let focus = self.focus.clone();
            let click_counts = self.counts.clone();
            let wheel_counts = self.counts.clone();
            let key_counts = self.counts.clone();
            div()
                .size_full()
                .track_focus(&self.focus)
                .on_mouse_down(
                    wabou_shell::gpui::MouseButton::Left,
                    move |_, window, cx| {
                        click_counts.clicks.fetch_add(1, Ordering::Relaxed);
                        window.focus(&focus, cx);
                    },
                )
                .on_scroll_wheel(move |_, _, _| {
                    wheel_counts.wheels.fetch_add(1, Ordering::Relaxed);
                })
                .on_key_down(move |_, _, _| {
                    key_counts.keys.fetch_add(1, Ordering::Relaxed);
                })
        }
    }

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

        let stats = harness
            .eval_string("__wabou_frame_stats()")
            .expect("read GPUI frame diagnostics");
        let stats: wabou_host_api::FrameStats =
            serde_json::from_str(&stats).expect("frame diagnostics are typed JSON");
        assert_eq!((stats.viewport_w, stats.viewport_h), (800, 600));
        assert!(stats.node_count >= 2);
        assert!(stats.build_frame_ms >= stats.js_tick_ms);
        assert!(stats.scene_ms >= 0.0);
        assert_eq!(stats.present_ms, 0.0);

        let layout = harness
            .eval_string(
                r#"(() => {
                  const ids = new Uint32Array([1, 1]);
                  const required = __wabou_layout_snapshot(ids, undefined);
                  const values = new Float64Array(required);
                  __wabou_layout_snapshot(ids, values);
                  return JSON.stringify(Array.from(values));
                })()"#,
            )
            .expect("read completed layout through the synchronous JS host API");
        let layout: Vec<f64> = serde_json::from_str(&layout).expect("layout ABI is numeric");
        assert_eq!(layout[0], 1.0, "layout ABI version");
        assert!(layout[1] > 0.0, "completed layout revision is published");
        assert_eq!((layout[3], layout[4]), (0.0, 0.0));
        assert_eq!((layout[5], layout[6]), (800.0, 600.0));
        assert_eq!(layout[7], 1.0, "requested projected node is present");
        assert!(layout[12] > 0.0 && layout[13] > 0.0);
    }

    #[test]
    fn advancing_headless_time_advances_quickjs_and_animation_frames_together() {
        let mut harness = GpuiHeadlessHarness::boot(
            include_str!("gen/test-runtime.js"),
            None::<Arc<[u8]>>,
            GpuiHeadlessOptions {
                window: WindowOptions::new().initial_inner_size(320, 200),
                settle_frames: 1,
            },
        )
        .expect("boot deterministic GPUI runtime");
        let before = harness
            .eval_string("String(performance.now())")
            .expect("read initial deterministic clock")
            .parse::<f64>()
            .expect("initial clock is numeric");
        harness
            .eval_script(
                "globalThis.__clock_probe = -1; requestAnimationFrame((time) => { globalThis.__clock_probe = time; });",
            )
            .expect("schedule animation frame");
        harness
            .advance_time(Duration::from_millis(220))
            .expect("advance the shared headless clock");
        let values = harness
            .eval_string("JSON.stringify([performance.now(), globalThis.__clock_probe])")
            .expect("read deterministic clock values");
        let values: Vec<f64> = serde_json::from_str(&values).expect("clock values are JSON");
        assert_eq!(values[0], values[1]);
        assert!((values[0] - before - 220.0).abs() < 0.001);
    }

    #[test]
    fn settling_headless_frames_completes_a_finite_animation_loop() {
        let mut harness = GpuiHeadlessHarness::boot(
            include_str!("gen/test-runtime.js"),
            None::<Arc<[u8]>>,
            GpuiHeadlessOptions {
                window: WindowOptions::new().initial_inner_size(320, 200),
                settle_frames: 1,
            },
        )
        .expect("boot deterministic GPUI runtime");
        harness
            .eval_script(
                r#"
                globalThis.__finite_frames = 0;
                const startedAt = performance.now();
                function finiteFrame(now) {
                  globalThis.__finite_frames += 1;
                  if (now - startedAt < 40) requestAnimationFrame(finiteFrame);
                }
                requestAnimationFrame(finiteFrame);
                "#,
            )
            .expect("schedule finite animation");

        harness.settle(4).expect("settle finite animation frames");

        let values = harness
            .eval_string(
                "JSON.stringify([globalThis.__finite_frames, globalThis.__wabou_has_raf()])",
            )
            .expect("read finite animation state");
        let values: (u32, bool) = serde_json::from_str(&values).expect("animation state is JSON");
        assert!(
            values.0 >= 3,
            "finite animation observed advancing frames: {values:?}"
        );
        assert!(!values.1, "finite animation released the frame clock");
    }

    #[test]
    fn completed_gpui_layout_notifies_javascript_resize_observers() {
        let mut harness = GpuiHeadlessHarness::boot(
            include_str!("gen/test-runtime.js"),
            None::<Arc<[u8]>>,
            GpuiHeadlessOptions {
                window: WindowOptions::new().initial_inner_size(800, 600),
                settle_frames: 2,
            },
        )
        .expect("boot GPUI headless fixture");
        harness
            .eval_script(
                r#"
                globalThis.__wabou_resize_result = null;
                new ResizeObserver(([entry]) => {
                  globalThis.__wabou_resize_result = [
                    entry.contentRect.width,
                    entry.contentRect.height,
                  ];
                }).observe({ id: { lo: 1, hi: 1 } });
                "#,
            )
            .expect("observe a projected node");
        harness.settle(2).expect("publish completed GPUI layout");

        let result = harness
            .eval_string("JSON.stringify(globalThis.__wabou_resize_result)")
            .expect("read resize observation");
        let result: Option<(f32, f32)> =
            serde_json::from_str(&result).expect("resize result is typed JSON");
        let (width, height) = result.expect("resize observer received completed geometry");
        assert!(width > 0.0 && height > 0.0);
    }

    #[test]
    fn native_widget_factories_run_on_the_real_headless_gpui_path() {
        let invocations = Arc::new(AtomicUsize::new(0));
        let observed_invocations = invocations.clone();
        let factory: wabou_shell::NativeWidgetFactory = Arc::new(move |context, _, _| {
            assert_eq!(context.key(), NodeKey::new(2, 1));
            observed_invocations.fetch_add(1, Ordering::Relaxed);
            wabou_shell::NativeWidgetMount::stateless(div().size_full().into_any_element())
        });

        let mut harness = GpuiHeadlessHarness::boot_with_native_widgets(
            include_str!("gen/test-runtime.js"),
            None::<Arc<[u8]>>,
            GpuiHeadlessOptions {
                window: WindowOptions::new().initial_inner_size(800, 600),
                settle_frames: 2,
            },
            HashMap::from([(String::from("main"), factory)]),
        )
        .expect("boot GPUI headless fixture with native widget");

        let output = harness.snapshot().expect("read GPUI layout");
        assert_eq!(output.protocol_revision, 1);
        assert!(
            invocations.load(Ordering::Relaxed) > 0,
            "registered native widget factory must participate in GPUI rendering"
        );
    }

    #[test]
    fn native_input_replay_uses_gpui_hit_testing_focus_and_wheel_dispatch() {
        let counts = Arc::new(InputCounts::default());
        let observed = counts.clone();
        let factory: wabou_shell::NativeWidgetFactory = Arc::new(move |context, _, cx| {
            let counts = observed.clone();
            let entity = context.entity::<InputProbe>().unwrap_or_else(|| {
                cx.new(|cx| InputProbe {
                    counts,
                    focus: cx.focus_handle(),
                })
            });
            wabou_shell::NativeWidgetMount::entity(
                entity.clone(),
                div()
                    .w(px(800.0))
                    .h(px(600.0))
                    .child(entity)
                    .into_any_element(),
            )
        });
        let mut harness = GpuiHeadlessHarness::boot_with_native_widgets(
            include_str!("gen/test-runtime.js"),
            None::<Arc<[u8]>>,
            GpuiHeadlessOptions {
                window: WindowOptions::new().initial_inner_size(800, 600),
                settle_frames: 2,
            },
            HashMap::from([(String::from("main"), factory)]),
        )
        .expect("boot interactive GPUI fixture");

        harness.click(400.0, 300.0).expect("click GPUI fixture");
        harness
            .wheel(400.0, 300.0, 0.0, -32.0)
            .expect("wheel GPUI fixture");
        harness
            .key("enter")
            .expect("type into focused GPUI fixture");
        harness
            .type_text("文a")
            .expect("type Unicode text through GPUI");

        assert_eq!(counts.clicks.load(Ordering::Relaxed), 1);
        assert_eq!(counts.wheels.load(Ordering::Relaxed), 1);
        assert_eq!(counts.keys.load(Ordering::Relaxed), 3);
    }
}
