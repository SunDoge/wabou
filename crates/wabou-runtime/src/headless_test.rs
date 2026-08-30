use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use vello::peniko::Color;

use crate::test_report::finish_test_report;
use crate::{FrameSource, WindowMetrics, WindowOptions};

#[derive(Clone, Copy)]
pub(super) struct HeadlessViewport {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) scale_factor: f64,
    pub(super) window_index: usize,
    pub(super) color_scheme: gpui_shell::ColorScheme,
}

impl HeadlessViewport {
    pub(super) fn with_logical_size(self, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..self
        }
    }

    pub(super) fn from_environment() -> crate::Result<Self> {
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

        let width = parse("WABOU_TEST_VIEWPORT_WIDTH", 1100_u32)?;
        let height = parse("WABOU_TEST_VIEWPORT_HEIGHT", 720_u32)?;
        let scale_factor = parse("WABOU_TEST_SCALE_FACTOR", 1.0_f64)?;
        let window_id = parse("WABOU_TEST_CAPTURE_WINDOW_ID", 1_u32)?;
        let color_scheme = match std::env::var("WABOU_TEST_COLOR_SCHEME")
            .as_deref()
            .unwrap_or("light")
        {
            "light" => gpui_shell::ColorScheme::Light,
            "dark" => gpui_shell::ColorScheme::Dark,
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

    pub(super) fn physical_width(self) -> u32 {
        self.physical_width_for(self.width)
    }

    pub(super) fn physical_width_for(self, width: u32) -> u32 {
        (f64::from(width) * self.scale_factor)
            .round()
            .clamp(1.0, f64::from(u32::MAX)) as u32
    }

    pub(super) fn physical_height(self) -> u32 {
        self.physical_height_for(self.height)
    }

    pub(super) fn physical_height_for(self, height: u32) -> u32 {
        (f64::from(height) * self.scale_factor)
            .round()
            .clamp(1.0, f64::from(u32::MAX)) as u32
    }
}

pub(super) fn run_headless_test(
    controller: &crate::test_driver::TestController,
    sources: &mut [(Box<dyn FrameSource>, WindowOptions)],
    base_color: Color,
    #[cfg(feature = "devtools")] debug_state: Option<&wabou_devtools::SharedDebugState>,
) -> crate::Result<()> {
    let viewport = HeadlessViewport::from_environment()?;

    controller.initialize_headless(
        (0..sources.len()).map(gpui_shell::initial_window_resource_key),
        viewport.width,
        viewport.height,
    );
    // JavaScript reports individual test timeouts with the test name. This is
    // only a final safety net for a broken runtime or runner that cannot
    // produce a report at all. Keep this later than the 60-second JS budget.
    let deadline = Instant::now() + Duration::from_secs(65);
    let mut text = crate::TextContext::new();
    let mut last_nodes = vec![Vec::new(); sources.len()];
    let mut profilers = (0..sources.len())
        .map(|_| wabou_shell::headless::HeadlessFrameProfiler::default())
        .collect::<Vec<_>>();
    while !controller.has_report() && Instant::now() < deadline {
        for (index, (source, _)) in sources.iter_mut().enumerate() {
            let window_key = gpui_shell::initial_window_resource_key(index);
            let (width, height) = controller
                .headless_viewport(window_key)
                .unwrap_or((viewport.width, viewport.height));
            source.set_semantics_enabled(true);
            source.handle_event(gpui_shell::UiEvent::WindowMetrics(WindowMetrics {
                window_key,
                logical_width: width,
                logical_height: height,
                physical_width: viewport.physical_width_for(width),
                physical_height: viewport.physical_height_for(height),
                scale_factor: viewport.scale_factor,
                maximized: false,
                focused: true,
                outer_x: None,
                outer_y: None,
                occluded: false,
                color_scheme: Some(viewport.color_scheme),
            }));
            last_nodes[index] = profilers[index].build(
                source.as_mut(),
                &mut text,
                width,
                height,
                viewport.scale_factor,
                base_color,
            );
            controller.poll_headless_source(window_key, source.as_mut());
            drain_headless_effects(source.as_mut());
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    // A test can finish while its final JS mutation still needs projection.
    // Capture only after two additional host frames have settled that work.
    for _ in 0..2 {
        for (index, (source, _)) in sources.iter_mut().enumerate() {
            let window_key = gpui_shell::initial_window_resource_key(index);
            let (width, height) = controller
                .headless_viewport(window_key)
                .unwrap_or((viewport.width, viewport.height));
            last_nodes[index] = source.build_frame(&mut text, width, height);
        }
    }

    let capture_window = gpui_shell::initial_window_resource_key(viewport.window_index);
    let capture_viewport = controller
        .headless_viewport(capture_window)
        .map(|(width, height)| viewport.with_logical_size(width, height))
        .unwrap_or(viewport);
    // Every source publishes into the shared DevTools state. Build the selected
    // window last so its tree and the PNG describe the same final frame.
    if let Some((source, _)) = sources.get_mut(viewport.window_index) {
        last_nodes[viewport.window_index] =
            source.build_frame(&mut text, capture_viewport.width, capture_viewport.height);
    }
    let source_count = sources.len();
    let capture_source = sources
        .get_mut(viewport.window_index)
        .map(|(source, _)| source.as_mut())
        .ok_or_else(|| crate::Error::TestScenario {
            message: format!(
                "capture requested window {} but the application has {} window(s)",
                viewport.window_index + 1,
                source_count
            ),
        })?;
    if controller.report_passed() == Some(false) {
        render_headless_failure(
            capture_source,
            &last_nodes,
            &mut text,
            base_color,
            capture_viewport,
        )?;
    }
    if let Some(output) = std::env::var_os("WABOU_TEST_CAPTURE_PATH") {
        render_headless_capture(
            capture_source,
            &last_nodes,
            &mut text,
            base_color,
            capture_viewport,
            Path::new(&output),
        )?;
    }
    #[cfg(feature = "devtools")]
    if let (Some(output), Some(state)) = (std::env::var_os("WABOU_TEST_SNAPSHOT_PATH"), debug_state)
    {
        write_headless_snapshot(state, Path::new(&output))?;
    }
    finish_test_report(controller.clone())
}

#[cfg(feature = "devtools")]
fn write_headless_snapshot(
    state: &wabou_devtools::SharedDebugState,
    output: &Path,
) -> crate::Result<()> {
    let failure = |message: String| crate::error::Error::HeadlessSnapshot {
        path: output.to_owned(),
        message,
    };
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| failure(error.to_string()))?;
    }
    let snapshot = state
        .read()
        .map_err(|_| failure("DevTools snapshot lock was poisoned".to_owned()))?
        .snapshot()
        .clone();
    let bytes = serde_json::to_vec_pretty(&snapshot).map_err(|error| failure(error.to_string()))?;
    std::fs::write(output, bytes).map_err(|error| failure(error.to_string()))?;
    Ok(())
}

fn drain_headless_effects(source: &mut dyn FrameSource) {
    while let Some(request) = source.take_effect() {
        source.complete_effect(gpui_shell::EffectCompletion {
            id: request.id,
            op: request.payload.op(),
            result: gpui_shell::EffectResult::Error {
                code: gpui_shell::EffectErrorCode::Unsupported,
                message: format!(
                    "native effect {:?} has no deterministic test fixture",
                    request.payload.op()
                ),
            },
        });
    }
}

fn render_headless_failure(
    source: &mut dyn FrameSource,
    last_nodes: &[Vec<wabou_shell::layout::PlacedNode>],
    text: &mut crate::TextContext,
    base_color: Color,
    viewport: HeadlessViewport,
) -> crate::Result<()> {
    if !std::env::var("WABOU_TEST_FAILURE_SCREENSHOT").is_ok_and(|value| value != "0") {
        return Ok(());
    }
    let Some(directory) = std::env::var_os("WABOU_TEST_ARTIFACT_DIR").map(PathBuf::from) else {
        return Ok(());
    };
    std::fs::create_dir_all(&directory).map_err(|error| crate::Error::TestScenario {
        message: format!("cannot create failure artifact directory: {error}"),
    })?;
    render_headless_capture(
        source,
        last_nodes,
        text,
        base_color,
        viewport,
        &directory.join("failure.png"),
    )
}

fn render_headless_capture(
    source: &mut dyn FrameSource,
    last_nodes: &[Vec<wabou_shell::layout::PlacedNode>],
    text: &mut crate::TextContext,
    base_color: Color,
    viewport: HeadlessViewport,
    output: &Path,
) -> crate::Result<()> {
    let Some(nodes) = last_nodes.get(viewport.window_index) else {
        return Err(crate::Error::TestScenario {
            message: format!(
                "capture requested window {} but the application has {} window(s)",
                viewport.window_index + 1,
                last_nodes.len()
            ),
        });
    };
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|error| crate::Error::TestScenario {
            message: format!(
                "cannot create capture directory {}: {error}",
                parent.display()
            ),
        })?;
    }
    let mut scene = anyrender::Scene::new();
    wabou_shell::scene::build_scene_scaled(
        &mut scene,
        nodes,
        text,
        viewport.width,
        viewport.height,
        base_color,
        viewport.scale_factor,
    );
    source.paint_debug_overlay(&mut scene, nodes, text, viewport.scale_factor);
    wabou_shell::renderer::render_to_png(
        &scene,
        viewport.physical_width(),
        viewport.physical_height(),
        base_color,
        output.to_string_lossy().as_ref(),
    )
    .map_err(|error| crate::Error::TestScenario {
        message: format!("cannot render headless screenshot: {error:?}"),
    })
}

#[cfg(test)]
mod tests {
    use super::HeadlessViewport;

    #[test]
    fn converts_logical_dimensions_with_one_shared_scale_policy() {
        let viewport = HeadlessViewport {
            width: 801,
            height: 601,
            scale_factor: 1.5,
            window_index: 0,
            color_scheme: gpui_shell::ColorScheme::Dark,
        };

        assert_eq!(viewport.physical_width(), 1202);
        assert_eq!(viewport.physical_height(), 902);
        assert_eq!(viewport.physical_width_for(640), 960);
        assert_eq!(viewport.with_logical_size(640, 480).window_index, 0);
        assert_eq!(
            viewport.with_logical_size(640, 480).color_scheme,
            gpui_shell::ColorScheme::Dark
        );
    }
}
