use std::error::Error;
use std::fs;
use std::path::Path;
use std::time::Instant;

use serde_json::json;
use vello::Scene;
use vello::peniko::Color;
use wabou_runtime::{Applier, FrameSource};
use wabou_shell::TextContext;
use wabou_shell::scene;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[middle - 1] + values[middle]) * 0.5
    } else {
        values[middle]
    }
}

pub(crate) struct RenderMetricsOptions<'a> {
    pub path: &'a Path,
    pub application: &'a str,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
    pub samples: usize,
    pub base_color: Color,
}

pub(crate) fn write(
    options: RenderMetricsOptions<'_>,
    applier: &mut Applier,
    text: &mut TextContext,
) -> Result<()> {
    if options.samples == 0 {
        return Err("--samples must be greater than zero".into());
    }
    let mut build_ms = Vec::with_capacity(options.samples);
    let mut scene_ms = Vec::with_capacity(options.samples);
    let mut node_count = 0;
    for _ in 0..options.samples {
        let started = Instant::now();
        let nodes = applier.build_frame(text, options.width, options.height);
        build_ms.push(started.elapsed().as_secs_f64() * 1_000.0);
        node_count = nodes.len();
        let started = Instant::now();
        let mut output = Scene::new();
        scene::build_scene_scaled(
            &mut output,
            &nodes,
            text,
            options.width,
            options.height,
            options.base_color,
            options.scale_factor,
        );
        scene_ms.push(started.elapsed().as_secs_f64() * 1_000.0);
    }
    let report = json!({
        "version": 1,
        "kind": "headless",
        "application": options.application,
        "samples": options.samples,
        "viewport": {
            "width": options.width,
            "height": options.height,
            "scaleFactor": options.scale_factor
        },
        "nodeCount": node_count,
        "medianMs": {
            "build": median(&mut build_ms),
            "scene": median(&mut scene_ms)
        },
        "sampleMs": { "build": build_ms, "scene": scene_ms },
        "limitations": "Headless diagnostics exclude native surface presentation and are not an FPS claim."
    });
    if let Some(parent) = options
        .path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    fs::write(options.path, serde_json::to_vec_pretty(&report)?)?;
    println!(
        "[wabou] wrote headless performance metrics {}",
        options.path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::median;

    #[test]
    fn median_orders_odd_and_even_samples() {
        assert_eq!(median(&mut [3.0, 1.0, 2.0]), 2.0);
        assert_eq!(median(&mut [4.0, 1.0, 3.0, 2.0]), 2.5);
    }
}
