//! The frame-producer contract a host implements to drive a [`crate::Shell`].
//!
//! Each frame, the app calls `build_frame` to get the flattened layout list,
//! then renders+presents. `has_anim` gates the continuous-redraw loop: a
//! reactive source (e.g. the SolidJS applier) returns true while rAF callbacks
//! are queued; a static source returns false so the loop idles until a resize.

#![warn(missing_docs)]

use vello::peniko::Color;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use crate::layout::PlacedNode;
use crate::style::CursorStyle;
use crate::text::TextContext;
use anyrender::Scene;

pub use wabou_shell_api::event::*;
pub use wabou_shell_api::{WindowCommand, WindowInputMode, WindowLevel, WindowOptions};

/// Retained UI producer consumed by the native shell.
///
/// All methods run on the UI thread. Background work communicates through
/// [`WakeCallback`] plus the polling/drain methods; it must never call into
/// layout, widgets, or the renderer directly.
pub trait FrameSource {
    /// Inform the source of the physical-pixels-per-logical-pixel ratio before
    /// it builds widget scene fragments for this frame.
    fn set_device_scale(&mut self, _scale: f64) {}

    /// Commit state queued by a previous native event before another event is
    /// dispatched. This is distinct from painting: input ordering must remain
    /// correct even when several platform events arrive before the next vsync.
    fn prepare_for_event(&mut self, _tcx: &mut TextContext) {}

    /// Lay out for `width x height` and return the paint-ordered node list.
    /// Borrowed `tcx` is used for text measurement (parley).
    fn build_frame(&mut self, tcx: &mut TextContext, width: u32, height: u32) -> Vec<PlacedNode>;

    /// Enable semantic snapshot production while a platform accessibility
    /// client is active. Sources should avoid accessibility tree work when
    /// this is false.
    fn set_semantics_enabled(&mut self, _enabled: bool) {}

    /// Return the latest immutable accessibility snapshot.
    ///
    /// Bounds are expressed in logical window coordinates.
    fn semantic_snapshot(&self) -> Option<Arc<SemanticSnapshot>> {
        None
    }

    /// Route a platform accessibility action back into retained UI state.
    fn handle_semantic_action(&mut self, _action: SemanticAction) -> bool {
        false
    }

    /// Focused editor exclusion area for the platform IME candidate window,
    /// expressed in window-logical coordinates.
    fn ime_cursor_area(&self) -> Option<[f64; 4]> {
        None
    }

    /// Cursor requested by the node currently under the pointer.
    fn pointer_cursor(&self) -> CursorStyle {
        CursorStyle::Default
    }

    /// Paint optional diagnostics after the application scene. Decorations are
    /// deliberately outside the retained tree, so they cannot affect layout,
    /// clipping, or hit testing.
    fn paint_debug_overlay(
        &mut self,
        _scene: &mut Scene,
        _nodes: &[PlacedNode],
        _tcx: &mut TextContext,
        _device_scale: f64,
    ) {
    }

    /// Viewport background color.
    fn base_color(&self) -> Color;

    /// Whether to keep redrawing every vsync. Default `false` (static); a
    /// reactive source overrides to return true while it has pending rAF work.
    fn has_anim(&self) -> bool {
        false
    }

    /// Earliest deferred animation repaint. Unlike `has_anim`, this lets the
    /// event loop sleep until the next frame is actually needed.
    fn animation_deadline(&self) -> Option<Instant> {
        None
    }

    /// Install the host event-loop wake callback. Sources should call this
    /// only after background work has made progress that must be observed on
    /// the UI thread.
    fn set_wake_callback(&mut self, _wake: WakeCallback) {}

    /// Drain asynchronous completions after the event loop was woken. Returns
    /// whether the completion can have changed the next rendered frame.
    fn poll_async(&mut self) -> bool {
        false
    }

    /// Drain one native host action produced by asynchronous work.
    fn take_host_action(&mut self) -> Option<HostAction> {
        None
    }

    /// Deliver the completion of a host action to its original producer.
    fn complete_host_action(&mut self, _result: HostActionResult) {}

    /// Drain one typed desktop effect. Unlike render ops, effects represent
    /// OS interaction and may complete asynchronously at a later frame boundary.
    fn take_effect(&mut self) -> Option<crate::EffectRequest> {
        None
    }

    /// Deliver completion of a typed desktop effect at a frame boundary.
    fn complete_effect(&mut self, _completion: crate::EffectCompletion) {}

    /// Notify the source that the native window was asked to close.
    /// Return true to keep the window alive.
    fn close_requested(&mut self) -> bool {
        false
    }

    /// Deliver a native Wabou event to the source.
    fn handle_event(&mut self, _event: UiEvent) -> EventResponse {
        EventResponse::IGNORED
    }

    /// Receive the latest per-frame stage timings (EMA) for host-side perf
    /// tooling (e.g. a Host diagnostics overlay). Default: ignore.
    fn push_frame_stats(&mut self, _stats: &FrameStats) {}

    /// DevTools screenshot handshake. The shell renders its current scene to
    /// this path only when requested; normal frames pay no readback cost.
    fn take_screenshot_request(&mut self) -> Option<ScreenshotRequest> {
        None
    }

    /// Report completion of the last screenshot request.
    fn complete_screenshot(
        &mut self,
        _requested_path: &std::path::Path,
        _result: Result<PathBuf, String>,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::{Modifiers, WindowInputMode, WindowLevel, WindowOptions};

    #[test]
    fn modifier_flags_match_host_protocol_bits() {
        let modifiers = Modifiers::SHIFT | Modifiers::ALT | Modifiers::META;
        assert_eq!(modifiers.bits(), 0b1101);
        assert!(modifiers.shift());
        assert!(!modifiers.control());
        assert!(modifiers.alt());
        assert!(modifiers.meta());
    }

    #[test]
    fn window_options_distinguish_requested_size_from_live_metrics() {
        let options = WindowOptions::new()
            .title("Inspector")
            .initial_inner_size(1440, 900)
            .min_inner_size(960, 600)
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .window_level(WindowLevel::AlwaysOnTop)
            .input_mode(WindowInputMode::Passthrough);
        assert_eq!(options.title, "Inspector");
        assert_eq!(options.initial_inner_size, (1440, 900));
        assert_eq!(options.min_inner_size, Some((960, 600)));
        assert!(!options.resizable);
        assert!(!options.decorations);
        assert!(options.transparent);
        assert_eq!(options.window_level, WindowLevel::AlwaysOnTop);
        assert_eq!(options.input_mode, WindowInputMode::Passthrough);
    }

    #[test]
    fn primary_shortcut_uses_only_the_platform_modifier() {
        let expected = if cfg!(target_os = "macos") {
            Modifiers::META
        } else {
            Modifiers::CONTROL
        };
        let other = if cfg!(target_os = "macos") {
            Modifiers::CONTROL
        } else {
            Modifiers::META
        };

        assert!(expected.primary_shortcut());
        assert!((expected | Modifiers::SHIFT).primary_shortcut());
        assert!(!other.primary_shortcut());
        assert!(!(expected | other).primary_shortcut());
        assert!(!(expected | Modifiers::ALT).primary_shortcut());
    }
}
