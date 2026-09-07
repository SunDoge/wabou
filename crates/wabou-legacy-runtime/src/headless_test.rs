//! Deterministic application-host loop for the Vello Hybrid backend.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use runtime_api::test_driver::{NativeTestHost, TestController};
use vello::peniko::Color;

use legacy_shell::{FrameSource, WindowOptions};

#[derive(Clone, Copy)]
struct HeadlessViewport {
    width: u32,
    height: u32,
    scale_factor: f64,
    window_index: usize,
    color_scheme: legacy_shell::ColorScheme,
}

impl HeadlessViewport {
    fn from_environment() -> crate::Result<Self> {
        fn parse<T>(name: &'static str, default: T) -> crate::Result<T>
        where
            T: std::str::FromStr,
            T::Err: std::fmt::Display,
        {
            let Some(value) = std::env::var_os(name) else {
                return Ok(default);
            };
            value
                .to_string_lossy()
                .parse()
                .map_err(|error| crate::Error::TestScenario {
                    message: format!("invalid {name}: {error}"),
                })
        }

        let width = parse("WABOU_TEST_VIEWPORT_WIDTH", 1_100_u32)?;
        let height = parse("WABOU_TEST_VIEWPORT_HEIGHT", 720_u32)?;
        let scale_factor = parse("WABOU_TEST_SCALE_FACTOR", 1.0_f64)?;
        let window_id = parse("WABOU_TEST_CAPTURE_WINDOW_ID", 1_u32)?;
        let color_scheme = match std::env::var("WABOU_TEST_COLOR_SCHEME")
            .as_deref()
            .unwrap_or("light")
        {
            "light" => legacy_shell::ColorScheme::Light,
            "dark" => legacy_shell::ColorScheme::Dark,
            value => {
                return Err(crate::Error::TestScenario {
                    message: format!(
                        "invalid WABOU_TEST_COLOR_SCHEME {value:?}; expected light or dark"
                    ),
                });
            }
        };
        if width == 0 || height == 0 {
            return Err(crate::Error::TestScenario {
                message: "headless viewport dimensions must be greater than zero".into(),
            });
        }
        if !scale_factor.is_finite() || scale_factor <= 0.0 {
            return Err(crate::Error::TestScenario {
                message: "headless scale factor must be finite and greater than zero".into(),
            });
        }
        let window_index = window_id
            .checked_sub(1)
            .ok_or_else(|| crate::Error::TestScenario {
                message: "headless capture window id must be greater than zero".into(),
            })? as usize;
        Ok(Self {
            width,
            height,
            scale_factor,
            window_index,
            color_scheme,
        })
    }

    fn physical(self, logical: u32) -> u32 {
        (f64::from(logical) * self.scale_factor)
            .round()
            .clamp(1.0, f64::from(u32::MAX)) as u32
    }
}

struct HeadlessNativeHost<'a> {
    source: &'a mut dyn FrameSource,
    viewport: &'a mut (u32, u32),
    visible: &'a mut bool,
}

impl NativeTestHost for HeadlessNativeHost<'_> {
    fn semantic_snapshot(
        &mut self,
        _window_key: legacy_shell::WindowResourceKey,
    ) -> Option<Arc<legacy_shell::SemanticSnapshot>> {
        self.source.semantic_snapshot()
    }

    fn dispatch_event(
        &mut self,
        _window_key: legacy_shell::WindowResourceKey,
        event: legacy_shell::UiEvent,
    ) -> bool {
        self.source.handle_event(event).handled
    }

    fn dispatch_semantic_action(
        &mut self,
        _window_key: legacy_shell::WindowResourceKey,
        action: legacy_shell::SemanticAction,
    ) -> bool {
        self.source.handle_semantic_action(action)
    }

    fn hide_window(
        &mut self,
        _window_key: legacy_shell::WindowResourceKey,
        _mutable_visibility: bool,
    ) -> bool {
        *self.visible = false;
        true
    }

    fn show_window(&mut self, _window_key: legacy_shell::WindowResourceKey) -> bool {
        *self.visible = true;
        true
    }

    fn resize_window(
        &mut self,
        _window_key: legacy_shell::WindowResourceKey,
        width: u32,
        height: u32,
    ) -> bool {
        *self.viewport = (width, height);
        true
    }

    fn window_viewport(&self, _window_key: legacy_shell::WindowResourceKey) -> Option<(u32, u32)> {
        Some(*self.viewport)
    }
}

pub(super) fn run(
    controller: &TestController,
    sources: &mut [(Box<dyn FrameSource>, WindowOptions)],
    base_color: Color,
    #[cfg(feature = "devtools")] debug_state: Option<&wabou_devtools::SharedDebugState>,
) -> crate::Result<()> {
    let viewport = HeadlessViewport::from_environment()?;
    let window_keys = (0..sources.len())
        .map(legacy_shell::initial_window_resource_key)
        .collect::<Vec<_>>();
    controller.connect_native_windows(window_keys.iter().copied(), Arc::new(|| {}));

    let mut viewports = vec![(viewport.width, viewport.height); sources.len()];
    let mut visible = vec![true; sources.len()];
    let mut text = legacy_shell::TextContext::new();
    let mut last_nodes = vec![Vec::new(); sources.len()];
    // Match the behavior runner's maximum suite budget. Authored scenarios can
    // register several tests, so a short capture-specific watchdog would turn
    // a healthy application host into a platform-dependent timeout.
    let deadline = Instant::now() + Duration::from_secs(305);

    while !controller.has_report() && Instant::now() < deadline {
        for (index, (source, _)) in sources.iter_mut().enumerate() {
            let window_key = window_keys[index];
            let (width, height) = viewports[index];
            source.set_semantics_enabled(true);
            source.set_device_scale(viewport.scale_factor);
            source.handle_event(legacy_shell::UiEvent::WindowMetrics(
                legacy_shell::WindowMetrics {
                    window_key,
                    logical_width: width,
                    logical_height: height,
                    physical_width: viewport.physical(width),
                    physical_height: viewport.physical(height),
                    scale_factor: viewport.scale_factor,
                    maximized: false,
                    focused: visible[index],
                    outer_x: None,
                    outer_y: None,
                    occluded: !visible[index],
                    reduced_motion: false,
                    color_scheme: Some(viewport.color_scheme),
                },
            ));
            last_nodes[index] = source.build_frame(&mut text, width, height);
            let mut host = HeadlessNativeHost {
                source: source.as_mut(),
                viewport: &mut viewports[index],
                visible: &mut visible[index],
            };
            let _ = controller.poll_native_host(window_key, &mut host);
            drain_effects(source.as_mut());
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    for _ in 0..2 {
        for (index, (source, _)) in sources.iter_mut().enumerate() {
            let (width, height) = viewports[index];
            last_nodes[index] = source.build_frame(&mut text, width, height);
        }
    }

    let source_count = sources.len();
    let (capture_source, _) =
        sources
            .get_mut(viewport.window_index)
            .ok_or_else(|| crate::Error::TestScenario {
                message: format!(
                    "capture requested window {} but the application has {source_count} window(s)",
                    viewport.window_index + 1
                ),
            })?;
    let (width, height) = viewports[viewport.window_index];
    last_nodes[viewport.window_index] = capture_source.build_frame(&mut text, width, height);

    if controller.report_passed() == Some(false) {
        render_failure(
            capture_source.as_mut(),
            &last_nodes[viewport.window_index],
            &mut text,
            base_color,
            viewport,
            width,
            height,
        )?;
    }
    if let Some(output) = std::env::var_os("WABOU_TEST_CAPTURE_PATH") {
        render_capture(
            capture_source.as_mut(),
            &last_nodes[viewport.window_index],
            &mut text,
            base_color,
            viewport,
            width,
            height,
            Path::new(&output),
        )?;
    }
    #[cfg(feature = "devtools")]
    if let (Some(output), Some(state)) = (std::env::var_os("WABOU_TEST_SNAPSHOT_PATH"), debug_state)
    {
        write_snapshot(state, Path::new(&output))?;
    }
    Ok(())
}

fn drain_effects(source: &mut dyn FrameSource) {
    while let Some(request) = source.take_effect() {
        source.complete_effect(legacy_shell::EffectCompletion {
            id: request.id,
            op: request.payload.op(),
            result: legacy_shell::EffectResult::Error {
                code: legacy_shell::EffectErrorCode::Unsupported,
                message: format!(
                    "native effect {:?} has no deterministic test fixture",
                    request.payload.op()
                ),
            },
        });
    }
}

fn render_failure(
    source: &mut dyn FrameSource,
    nodes: &[legacy_shell::layout::PlacedNode],
    text: &mut legacy_shell::TextContext,
    base_color: Color,
    viewport: HeadlessViewport,
    width: u32,
    height: u32,
) -> crate::Result<()> {
    if !std::env::var("WABOU_TEST_FAILURE_SCREENSHOT").is_ok_and(|value| value != "0") {
        return Ok(());
    }
    let Some(directory) = std::env::var_os("WABOU_TEST_ARTIFACT_DIR").map(PathBuf::from) else {
        return Ok(());
    };
    render_capture(
        source,
        nodes,
        text,
        base_color,
        viewport,
        width,
        height,
        &directory.join("failure.png"),
    )
}

#[allow(clippy::too_many_arguments)]
fn render_capture(
    source: &mut dyn FrameSource,
    nodes: &[legacy_shell::layout::PlacedNode],
    text: &mut legacy_shell::TextContext,
    base_color: Color,
    viewport: HeadlessViewport,
    width: u32,
    height: u32,
    output: &Path,
) -> crate::Result<()> {
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|error| crate::Error::TestScenario {
            message: format!(
                "cannot create capture directory {}: {error}",
                parent.display()
            ),
        })?;
    }
    let mut scene = anyrender::Scene::new();
    legacy_shell::scene::build_scene_scaled(
        &mut scene,
        nodes,
        text,
        width,
        height,
        base_color,
        viewport.scale_factor,
    );
    source.paint_debug_overlay(&mut scene, nodes, text, viewport.scale_factor);
    legacy_shell::renderer::render_to_png_with_backend(
        &scene,
        viewport.physical(width),
        viewport.physical(height),
        base_color,
        legacy_shell::RendererBackend::VelloHybrid,
        output.to_string_lossy().as_ref(),
    )
    .map_err(|error| crate::Error::TestScenario {
        message: format!("cannot render headless screenshot: {error:?}"),
    })
}

#[cfg(feature = "devtools")]
fn write_snapshot(state: &wabou_devtools::SharedDebugState, output: &Path) -> crate::Result<()> {
    let failure = |message: String| crate::Error::HeadlessSnapshot {
        path: output.to_owned(),
        message,
    };
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|error| failure(error.to_string()))?;
    }
    let snapshot = state
        .read()
        .map_err(|_| failure("DevTools snapshot lock was poisoned".into()))?
        .snapshot()
        .clone();
    let bytes = serde_json::to_vec_pretty(&snapshot).map_err(|error| failure(error.to_string()))?;
    std::fs::write(output, bytes).map_err(|error| failure(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::HeadlessViewport;

    #[test]
    fn physical_dimensions_round_once_at_the_surface_boundary() {
        let viewport = HeadlessViewport {
            width: 800,
            height: 600,
            scale_factor: 1.5,
            window_index: 0,
            color_scheme: legacy_shell::ColorScheme::Light,
        };

        assert_eq!(viewport.physical(viewport.width), 1_200);
        assert_eq!(viewport.physical(viewport.height), 900);
        assert_eq!(viewport.physical(1), 2);
    }
}
