//! Retained custom WGSL surface driven by Wabou's native frame clock.

use std::time::{Duration, Instant};

use serde::Deserialize;
use wabou_shell::{PaintContext, Widget, WidgetChanges, decode_widget_config};
use wabou_shell_vello::{ShaderEffectId, ShaderEffectSource};

const FRAME_INTERVAL: Duration = Duration::from_millis(16);
const MAX_VALUES: usize = 16;

fn default_animated() -> bool {
    true
}

fn default_speed() -> f32 {
    1.0
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ShaderLayerConfig {
    source: String,
    #[serde(default)]
    values: Vec<f32>,
    #[serde(default)]
    time: f32,
    #[serde(default = "default_speed")]
    speed: f32,
    #[serde(default = "default_animated")]
    animated: bool,
    #[serde(default)]
    paused: bool,
}

struct PreparedConfig {
    authored: ShaderLayerConfig,
    source: ShaderEffectSource,
}

/// Native widget that renders one validated custom WGSL effect.
pub struct ShaderLayer {
    id: ShaderEffectId,
    config: Option<PreparedConfig>,
    started: Instant,
}

impl ShaderLayer {
    /// Construct an unconfigured shader surface.
    pub fn new() -> Self {
        Self {
            id: ShaderEffectId::new(),
            config: None,
            started: Instant::now(),
        }
    }

    fn running(&self) -> bool {
        self.config
            .as_ref()
            .is_some_and(|config| config.authored.animated && !config.authored.paused)
    }
}

impl Default for ShaderLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ShaderLayer {
    fn paint(&mut self, cx: &mut PaintContext<'_>) {
        let Some(config) = &self.config else {
            return;
        };
        let elapsed = if self.running() {
            self.started.elapsed().as_secs_f32() * config.authored.speed
        } else {
            0.0
        };
        cx.draw_shader_effect(
            self.id,
            config.source.clone(),
            config.authored.time + elapsed,
            &config.authored.values,
        );
    }

    fn config_changed(&mut self, json: &str) -> Result<WidgetChanges, String> {
        let authored = decode_widget_config::<ShaderLayerConfig>(json)?;
        if authored.values.len() > MAX_VALUES {
            return Err(format!(
                "shader layer accepts at most {MAX_VALUES} scalar values"
            ));
        }
        if !authored.time.is_finite()
            || !authored.speed.is_finite()
            || authored.speed < 0.0
            || authored.values.iter().any(|value| !value.is_finite())
        {
            return Err(
                "shader layer time, speed, and values must be finite; speed must be non-negative"
                    .into(),
            );
        }
        if self
            .config
            .as_ref()
            .is_some_and(|config| config.authored == authored)
        {
            return Ok(WidgetChanges::empty());
        }
        let source = ShaderEffectSource::new(&authored.source)?;
        self.config = Some(PreparedConfig { authored, source });
        self.started = Instant::now();
        Ok(WidgetChanges::REDRAW)
    }

    fn config_removed(&mut self) -> WidgetChanges {
        self.config = None;
        WidgetChanges::REDRAW
    }

    fn intrinsic_size(&self) -> Option<[f32; 2]> {
        Some([320.0, 180.0])
    }

    fn animation_deadline(&self) -> Option<Instant> {
        self.running().then(|| Instant::now() + FRAME_INTERVAL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wabou_shell::{PaintCommand, WidgetGeometry, WidgetHarness};

    const SOURCE: &str =
        "fn wabou_effect(uv: vec2<f32>) -> vec4<f32> { return vec4<f32>(uv, 0.0, 1.0); }";

    #[test]
    fn validates_config_and_records_physical_geometry() {
        let mut harness = WidgetHarness::new(ShaderLayer::new());
        harness.layout(WidgetGeometry {
            content_size: [80.0, 40.0],
            device_scale: 2.0,
            ..WidgetGeometry::default()
        });
        harness
            .widget_mut()
            .config_changed(&format!(
                r#"{{"source":{source:?},"animated":false,"values":[0.4]}}"#,
                source = SOURCE
            ))
            .unwrap();
        let scene = harness.paint();
        let PaintCommand::Shader { effect, .. } = &scene.commands[0] else {
            panic!("expected a shader paint command")
        };
        assert_eq!(effect.physical_size(), [160, 80]);
        assert_eq!(effect.logical_size(), [80.0, 40.0]);
        assert!(harness.widget().animation_deadline().is_none());
    }

    #[test]
    fn rejects_invalid_source_and_excess_parameters() {
        let mut layer = ShaderLayer::new();
        assert!(
            layer
                .config_changed(r#"{"source":"fn nope() {}"}"#)
                .is_err()
        );
        let values = vec!["1"; 17].join(",");
        assert!(
            layer
                .config_changed(&format!(
                    r#"{{"source":{source:?},"values":[{values}]}}"#,
                    source = SOURCE
                ))
                .is_err()
        );
    }

    #[test]
    fn running_effect_requests_native_frames() {
        let mut layer = ShaderLayer::new();
        layer
            .config_changed(&format!(r#"{{"source":{SOURCE:?}}}"#))
            .unwrap();
        assert!(layer.animation_deadline().is_some());
    }
}
