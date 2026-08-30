//! GPUI view owning one Wabou JavaScript runtime.

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
        _cx: &mut Context<Self>,
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

        self.applier
            .gpui_element()
            .expect("the canonical Wabou root remains retained")
    }
}
