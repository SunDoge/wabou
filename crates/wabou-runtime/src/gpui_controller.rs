//! GPUI-owned retained state for one JavaScript runtime.

use crate::{ImageResourceHandle, ImageResourceStore, protocol::Frame};
use gpui_shell::{GpuiProjection, ProjectionError};
use wabou_protocol::AtomPool;

/// Renderer-side state consumed exclusively by the GPUI application runtime.
///
/// This controller intentionally does not know about Taffy, Parley, Vello,
/// winit widgets, or the legacy document. It is the extraction boundary that
/// will remain in `wabou-runtime` after the old behavior oracle moves out.
pub(crate) struct GpuiController {
    projection: GpuiProjection,
    image_resources: ImageResourceStore,
}

impl Default for GpuiController {
    fn default() -> Self {
        Self {
            projection: GpuiProjection::new(),
            image_resources: ImageResourceStore::default(),
        }
    }
}

impl GpuiController {
    pub(crate) fn set_image_resources(&mut self, resources: ImageResourceStore) {
        self.image_resources = resources;
    }

    pub(crate) fn apply_frame(
        &mut self,
        frame: &Frame<'_>,
        atoms: &AtomPool,
    ) -> Result<(), ProjectionError> {
        self.projection.apply_ops(frame, atoms, |source| {
            let (lo, hi) = source.split_once(':')?;
            let handle = ImageResourceHandle {
                lo: lo.parse().ok()?,
                hi: hi.parse().ok()?,
            };
            self.image_resources
                .get(handle)
                .map(|resource| resource.gpui_image())
        })
    }

    pub(crate) fn projection(&self) -> &GpuiProjection {
        &self.projection
    }

    pub(crate) fn projection_mut(&mut self) -> &mut GpuiProjection {
        &mut self.projection
    }
}
