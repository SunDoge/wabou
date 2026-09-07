//! Backend-neutral QuickJS session and host-bridge lifetime.

use std::{cell::RefCell, collections::VecDeque, rc::Rc, sync::Arc};

use tokio_util::sync::CancellationToken;

use crate::{
    effect_bridge::EffectBridge,
    host_message::{
        DEFAULT_HOST_MESSAGE_CAPACITY, HostMessageHandle, HostMessageInbox, host_message_channel,
    },
    jsrt::JsRuntime,
    reload::ReloadState,
    style_ir::StylesheetUpdate,
};
use wabou_protocol::AtomPool;
use wabou_shell_api::{FrameStats, HostAction, WakeCallback, WindowResourceKey};

/// QuickJS and host-bridge state with one shared cancellation lifetime.
///
/// This layer deliberately owns no layout tree, renderer object, or platform
/// event-loop state. GPUI, Winit/Vello Hybrid, and their headless harnesses can
/// therefore compose it without making the JavaScript host backend-aware.
#[doc(hidden)]
pub struct RuntimeSession {
    /// QuickJS runtime owned by this window.
    pub js: JsRuntime,
    /// Shared protocol atom table.
    pub atoms: Rc<RefCell<AtomPool>>,
    /// Whether JavaScript requested another animation frame.
    pub has_raf: bool,
    /// Last applied Solid protocol revision.
    pub protocol_revision: u64,
    /// Coalesced Vite reload state.
    pub reload: ReloadState,
    /// Pending Style IR update published by JavaScript.
    pub pending_css: Option<Rc<RefCell<Option<StylesheetUpdate>>>>,
    /// Pending semantic color-theme name.
    pub pending_color_theme: Option<Rc<RefCell<Option<String>>>>,
    /// Pending packed semantic color palette.
    pub pending_color_palette: Option<Rc<RefCell<Option<Vec<u32>>>>>,
    /// Application font byte buffers awaiting backend registration.
    pub pending_fonts: Option<Rc<RefCell<Vec<Vec<u8>>>>>,
    /// Latest frame statistics exposed to JavaScript diagnostics.
    pub frame_stats: Option<Rc<RefCell<Option<FrameStats>>>>,
    /// Backend-neutral widget/host actions awaiting native execution.
    pub pending_host_actions: Rc<RefCell<VecDeque<HostAction>>>,
    /// Native effect request/completion bridge.
    pub effect_bridge: EffectBridge,
    /// Current native event-loop wake callback.
    pub wake_callback: Option<WakeCallback>,
    /// Rust-to-JavaScript message inbox.
    pub host_message_inbox: HostMessageInbox,
    /// Producer handle paired with `host_message_inbox`.
    pub host_message_handle: HostMessageHandle,
    /// Cancellation lifetime shared by message producers.
    pub host_message_cancellation: CancellationToken,
    /// Tracks producer tasks during graceful shutdown.
    pub host_tasks: Arc<crate::host_message::HostTaskTracker>,
}

impl RuntimeSession {
    /// Compose all backend-neutral state for one native window.
    pub fn new(js: JsRuntime, window_key: WindowResourceKey) -> Self {
        let atoms = js.atom_pool_handle();
        let pending_css = js.pending_css_handle();
        let pending_color_theme = js.pending_color_theme_handle();
        let pending_color_palette = js.pending_color_palette_handle();
        let pending_fonts = js.pending_fonts_handle();
        let frame_stats = js.frame_stats_handle();
        let pending_host_actions = Rc::new(RefCell::new(VecDeque::new()));
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
            pending_host_actions,
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
