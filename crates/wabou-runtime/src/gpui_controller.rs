//! GPUI-owned retained state for one JavaScript runtime.

use crate::{
    ImageResourceHandle, ImageResourceStore, protocol::Frame, runtime_session::RuntimeSession,
};
use gpui_shell::{GpuiProjection, ProjectionError};
use wabou_protocol::AtomPool;

/// Renderer-side state consumed exclusively by the GPUI application runtime.
///
/// This controller intentionally does not know about Taffy, Parley, Vello,
/// winit widgets, or the legacy document. It is the extraction boundary that
/// will remain in `wabou-runtime` after the old behavior oracle moves out.
pub struct GpuiController {
    pub(crate) runtime: RuntimeSession,
    projection: GpuiProjection,
    image_resources: ImageResourceStore,
}

impl GpuiController {
    pub(crate) fn new(runtime: RuntimeSession) -> Self {
        Self {
            runtime,
            projection: GpuiProjection::new(),
            image_resources: ImageResourceStore::default(),
        }
    }
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

    /// Monotonically increasing count of non-empty JS-to-host frames.
    pub fn protocol_revision(&self) -> u64 {
        self.runtime.protocol_revision
    }

    /// Boot the application after host bridges have been installed.
    pub fn boot(&mut self, source: &str) -> rquickjs::Result<()> {
        self.runtime.js.boot(source)
    }

    pub(crate) fn boot_with_source_map(
        &mut self,
        source: &str,
        source_map: Option<&[u8]>,
    ) -> rquickjs::Result<()> {
        self.runtime.js.boot_with_source_map(source, source_map)
    }

    /// Evaluate an additional script in the booted application realm.
    pub fn eval_script(&self, source: &str) -> rquickjs::Result<()> {
        self.runtime.js.eval_script(source)
    }

    /// Evaluate a test script and preserve mapped guest diagnostics.
    pub fn eval_script_diagnostic(&self, source: &str) -> Result<(), String> {
        self.runtime.js.eval_script_diagnostic(source)
    }

    /// Evaluate an expression and return its string value.
    pub fn eval_string(&self, source: &str) -> rquickjs::Result<String> {
        self.runtime.js.eval_string(source)
    }

    #[cfg(feature = "vite")]
    /// Boot an application entry module through Vite.
    pub fn boot_vite(&mut self, entry: &str) -> rquickjs::Result<()> {
        self.runtime.js.boot_vite(entry)
    }

    pub(crate) fn set_effect_trace(&mut self, trace: crate::effect_trace::EffectTrace) {
        self.runtime.effect_bridge.set_trace(trace);
    }

    /// Publish application-private directories to native effects.
    pub fn set_app_directories(&mut self, directories: gpui_shell::AppDirectories) {
        self.runtime.effect_bridge.set_app_directories(directories);
    }

    /// Cloneable producer handle for application Rust-to-JavaScript messages.
    pub fn host_message_handle(&self) -> crate::HostMessageHandle {
        self.runtime.host_message_handle.clone()
    }

    pub(crate) fn host_message_context(
        &self,
        window_key: gpui_shell::WindowResourceKey,
    ) -> crate::HostMessageContext {
        crate::HostMessageContext::new(
            window_key,
            self.host_message_handle(),
            self.runtime.host_message_cancellation.clone(),
            self.runtime.js.tokio_handle(),
            self.runtime.host_tasks.clone(),
        )
    }
}
