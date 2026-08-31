//! Native execution of animation timelines declared by the JS motion layer.

use std::time::Instant;

use serde::Deserialize;

/// A repeating timeline DTO shared by retained native widgets.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeLoopAnimation {
    /// Discriminator kept explicit for future timeline variants.
    kind: NativeLoopKind,
    /// Duration of one iteration in seconds.
    duration: f32,
    /// Playback-rate multiplier.
    speed: f32,
    /// Explicit application pause state.
    paused: bool,
    /// JS-side reduced-motion policy.
    #[serde(rename = "reducedMotion")]
    reduced_motion: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum NativeLoopKind {
    Loop,
}

impl Default for NativeLoopAnimation {
    fn default() -> Self {
        Self {
            kind: NativeLoopKind::Loop,
            duration: 1.0,
            speed: 1.0,
            paused: false,
            reduced_motion: false,
        }
    }
}

impl NativeLoopAnimation {
    fn normalized(self) -> Self {
        Self {
            duration: positive_finite(self.duration, 1.0),
            speed: positive_finite(self.speed, 1.0),
            ..self
        }
    }
}

fn positive_finite(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

/// Retained clock which samples a JS-authored loop without per-frame bridge traffic.
pub(crate) struct NativeLoopTimeline {
    animation: NativeLoopAnimation,
    started: Instant,
}

impl NativeLoopTimeline {
    pub(crate) fn new(animation: NativeLoopAnimation) -> Self {
        Self {
            animation: animation.normalized(),
            started: Instant::now(),
        }
    }

    /// Update authored policy. The phase restarts only when timing changes.
    pub(crate) fn synchronize(&mut self, animation: NativeLoopAnimation) -> bool {
        let animation = animation.normalized();
        if self.animation == animation {
            return false;
        }
        if self.animation.duration != animation.duration || self.animation.speed != animation.speed
        {
            self.started = Instant::now();
        }
        self.animation = animation;
        true
    }

    pub(crate) fn is_running(&self, platform_reduced_motion: bool) -> bool {
        !self.animation.paused && !self.animation.reduced_motion && !platform_reduced_motion
    }

    pub(crate) fn phase(&self, platform_reduced_motion: bool) -> f32 {
        if !self.is_running(platform_reduced_motion) {
            return 0.0;
        }
        (self.started.elapsed().as_secs_f32() * self.animation.speed / self.animation.duration)
            .fract()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_loop_normalizes_untrusted_timing_and_obeys_motion_policy() {
        let animation: NativeLoopAnimation = serde_json::from_str(
            r#"{"kind":"loop","duration":0,"speed":-2,"paused":false,"reducedMotion":false}"#,
        )
        .unwrap();
        let timeline = NativeLoopTimeline::new(animation);
        assert!(timeline.is_running(false));
        assert!(!timeline.is_running(true));
        assert_eq!(timeline.phase(true), 0.0);
        assert_eq!(timeline.animation.duration, 1.0);
        assert_eq!(timeline.animation.speed, 1.0);
    }
}
