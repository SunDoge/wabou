//! GPUI view owning one Wabou JavaScript runtime.

use std::rc::Rc;

use wabou_shell_gpui::gpui::{Context, Render, Window};

use crate::{Applier, FrameSource};

/// A coarse GPUI entity for one Solid application runtime.
///
/// Solid retains individual UI nodes and emits one mutation batch per flush;
/// this entity advances that runtime once per GPUI frame and materializes the
/// resulting retained projection. It intentionally does not create one GPUI
/// entity per Solid node.
pub struct GpuiRuntimeView {
    applier: Applier,
}

impl GpuiRuntimeView {
    /// Wrap an already configured and booted Wabou runtime.
    #[must_use]
    pub fn new(applier: Applier) -> Self {
        Self { applier }
    }

    /// Borrow the underlying runtime for host integration during migration.
    #[must_use]
    pub fn applier(&self) -> &Applier {
        &self.applier
    }

    /// Mutably borrow the underlying runtime for host integration.
    pub fn applier_mut(&mut self) -> &mut Applier {
        &mut self.applier
    }
}

impl Render for GpuiRuntimeView {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl wabou_shell_gpui::gpui::IntoElement {
        let viewport = window.viewport_size();
        let _ = self
            .applier
            .build_gpui_frame(viewport.width.into(), viewport.height.into());

        if FrameSource::has_anim(&self.applier) {
            // GPUI associates this request with the currently rendering view
            // and notifies only that entity on the next platform frame.
            window.request_animation_frame();
        }

        let view = cx.weak_entity();
        let input = Rc::new(move |event, cx: &mut wabou_shell_gpui::gpui::App| {
            let _ = view.update(cx, |view, cx| {
                let response = view.applier.handle_gpui_pointer(event);
                if response.request_redraw {
                    cx.notify();
                }
            });
        });
        self.applier
            .gpui_interactive_element(input)
            .expect("the canonical Wabou root remains retained")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::JsRuntime;
    use wabou_host_api::NodeKey;

    #[test]
    fn real_solid_writer_frame_materializes_as_a_gpui_tree() {
        let runtime = JsRuntime::new().expect("QuickJS runtime");
        let mut applier = Applier::from_runtime(runtime, vello::peniko::Color::TRANSPARENT);
        applier
            .boot(include_str!("gen/test-runtime.js"))
            .expect("boot generated Solid runtime fixture");

        assert!(applier.build_gpui_frame(800, 600));
        assert_eq!(applier.protocol_revision(), 1);
        assert!(
            applier.gpui_contains(NodeKey::new(2, 1)),
            "the fixture's mounted <main> must cross the binary writer boundary"
        );
        let _root = applier
            .gpui_element()
            .expect("the completed Solid tree must materialize for GPUI");
    }
}
