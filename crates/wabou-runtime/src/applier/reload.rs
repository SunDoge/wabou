use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};

/// A Vite HMR signal forwarded from the background HMR client to the applier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReloadMsg {
    /// Updated Vite module accepted by an HMR boundary.
    HmrUpdate {
        /// Module path reported by Vite.
        path: String,
        /// Boundary module that accepted the update.
        accepted_path: String,
        /// Vite update timestamp.
        timestamp: u64,
        /// Updated JavaScript module source.
        source: String,
    },
    /// Native Vite CSS channel. Wabou styles flow through
    /// `virtual:wabou-stylesheet` → `__wabou_set_stylesheet` (Style IR) instead;
    /// these messages are acknowledged and logged, not applied as CSSOM.
    CssUpdate {
        /// CSS module path reported by Vite.
        path: String,
        /// CSS source retained only for diagnostics.
        source: String,
    },
    /// Vite requested a complete entry re-import.
    FullReload,
}

/// Result of draining the HMR queue for one frame (for tests / diagnostics).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HmrDrainResult {
    /// No queued update changed the runtime.
    Idle,
    /// One or more JS modules were accepted; Style IR may also have updated
    /// via `pending_css` in the same frame.
    Applied {
        /// Number of JavaScript modules applied in arrival order.
        js_updates: usize,
    },
    /// Entry was (or should be) fully re-imported.
    FullReload {
        /// Diagnostic explaining why partial HMR was not possible.
        reason: String,
    },
}

/// Sendable handle the HMR client holds to push [`ReloadMsg`]s into the applier.
#[derive(Clone)]
pub struct ReloadHandle {
    tx: mpsc::Sender<ReloadMsg>,
    pending: Arc<AtomicBool>,
}

impl ReloadHandle {
    /// Enqueue an HMR signal and wake an otherwise idle render loop.
    pub fn send(&self, message: ReloadMsg) -> Result<(), mpsc::SendError<ReloadMsg>> {
        self.tx.send(message)?;
        self.pending.store(true, Ordering::Release);
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct HmrBatch {
    pub(super) full_reload: bool,
    pub(super) full_reload_reason: Option<String>,
    pub(super) js_updates: Vec<HmrJsUpdate>,
    pub(super) css_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct HmrJsUpdate {
    pub(super) path: String,
    pub(super) accepted_path: String,
    pub(super) timestamp: u64,
    pub(super) source: String,
}

pub(super) struct ReloadState {
    receiver: Option<mpsc::Receiver<ReloadMsg>>,
    pending: Arc<AtomicBool>,
    vite_entry: Option<String>,
    last_result: HmrDrainResult,
}

impl Default for ReloadState {
    fn default() -> Self {
        Self {
            receiver: None,
            pending: Arc::new(AtomicBool::new(false)),
            vite_entry: None,
            last_result: HmrDrainResult::Idle,
        }
    }
}

impl ReloadState {
    pub(super) fn handle(&mut self) -> ReloadHandle {
        let (tx, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        ReloadHandle {
            tx,
            pending: self.pending.clone(),
        }
    }

    pub(super) fn drain(&self) -> Option<HmrBatch> {
        let receiver = self.receiver.as_ref()?;
        let messages: Vec<_> = receiver.try_iter().collect();
        (!messages.is_empty()).then(|| plan_hmr_batch(messages))
    }

    pub(super) fn is_pending(&self) -> bool {
        self.pending.load(Ordering::Acquire)
    }

    pub(super) fn clear_pending(&self) {
        self.pending.store(false, Ordering::Release);
    }

    pub(super) fn set_vite_entry(&mut self, entry: impl Into<String>) {
        self.vite_entry = Some(entry.into());
    }

    #[cfg(feature = "vite")]
    pub(super) fn vite_entry(&self) -> Option<&str> {
        self.vite_entry.as_deref()
    }

    pub(super) fn last_result(&self) -> &HmrDrainResult {
        &self.last_result
    }

    pub(super) fn record_result(&mut self, result: HmrDrainResult) {
        self.last_result = result;
    }
}

/// Coalesce a burst of websocket messages into one ordered batch.
pub(super) fn plan_hmr_batch(msgs: impl IntoIterator<Item = ReloadMsg>) -> HmrBatch {
    let mut batch = HmrBatch::default();
    for msg in msgs {
        match msg {
            ReloadMsg::FullReload => {
                batch.full_reload = true;
                batch.full_reload_reason = Some("vite full-reload payload".to_string());
            }
            ReloadMsg::HmrUpdate {
                path,
                accepted_path,
                timestamp,
                source,
            } => batch.js_updates.push(HmrJsUpdate {
                path,
                accepted_path,
                timestamp,
                source,
            }),
            ReloadMsg::CssUpdate { path, .. } => batch.css_paths.push(path),
        }
    }
    batch
}
