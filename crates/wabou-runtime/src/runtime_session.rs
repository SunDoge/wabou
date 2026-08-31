//! Backend-neutral QuickJS session and host-bridge lifetime.

use std::{cell::RefCell, rc::Rc, sync::Arc};

use tokio_util::sync::CancellationToken;

use crate::{
    atom::AtomPool,
    effect_bridge::EffectBridge,
    host_message::{
        DEFAULT_HOST_MESSAGE_CAPACITY, HostMessageHandle, HostMessageInbox, host_message_channel,
    },
    jsrt::JsRuntime,
    reload::ReloadState,
    style_ir::StylesheetUpdate,
};
use wabou_shell::{FrameStats, WakeCallback, WindowResourceKey};

/// QuickJS and host-bridge state with one shared cancellation lifetime.
///
/// This layer deliberately owns no layout tree, renderer object, or platform
/// event-loop state. The GPUI runtime and temporary legacy test runtime can
/// therefore compose it without making the JavaScript host backend-aware.
pub(crate) struct RuntimeSession {
    pub(crate) js: JsRuntime,
    pub(crate) atoms: Rc<RefCell<AtomPool>>,
    pub(crate) has_raf: bool,
    pub(crate) protocol_revision: u64,
    pub(crate) reload: ReloadState,
    pub(crate) pending_css: Option<Rc<RefCell<Option<StylesheetUpdate>>>>,
    pub(crate) pending_color_theme: Option<Rc<RefCell<Option<String>>>>,
    pub(crate) pending_color_palette: Option<Rc<RefCell<Option<Vec<u32>>>>>,
    pub(crate) pending_fonts: Option<Rc<RefCell<Vec<Vec<u8>>>>>,
    pub(crate) frame_stats: Option<Rc<RefCell<Option<FrameStats>>>>,
    pub(crate) effect_bridge: EffectBridge,
    pub(crate) wake_callback: Option<WakeCallback>,
    pub(crate) host_message_inbox: HostMessageInbox,
    pub(crate) host_message_handle: HostMessageHandle,
    pub(crate) host_message_cancellation: CancellationToken,
    pub(crate) host_tasks: Arc<crate::host_message::HostTaskTracker>,
}

impl RuntimeSession {
    pub(crate) fn new(js: JsRuntime, window_key: WindowResourceKey) -> Self {
        let atoms = js.atom_pool_handle();
        let pending_css = js.pending_css_handle();
        let pending_color_theme = js.pending_color_theme_handle();
        let pending_color_palette = js.pending_color_palette_handle();
        let pending_fonts = js.pending_fonts_handle();
        let frame_stats = js.frame_stats_handle();
        let effect_bridge = EffectBridge::install(&js, window_key);
        let (host_message_handle, host_message_inbox) =
            host_message_channel(DEFAULT_HOST_MESSAGE_CAPACITY);
        Self {
            js,
            atoms,
            has_raf: true,
            protocol_revision: 0,
            reload: ReloadState::default(),
            pending_css: Some(pending_css),
            pending_color_theme: Some(pending_color_theme),
            pending_color_palette: Some(pending_color_palette),
            pending_fonts: Some(pending_fonts),
            frame_stats: Some(frame_stats),
            effect_bridge,
            wake_callback: None,
            host_message_inbox,
            host_message_handle,
            host_message_cancellation: CancellationToken::new(),
            host_tasks: Arc::new(crate::host_message::HostTaskTracker::default()),
        }
    }
}

impl Drop for RuntimeSession {
    fn drop(&mut self) {
        self.host_message_cancellation.cancel();
        if !self
            .host_tasks
            .wait_for_idle(std::time::Duration::from_secs(1))
        {
            tracing::warn!("host message producers did not stop before runtime shutdown");
        }
    }
}
