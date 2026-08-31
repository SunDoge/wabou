//! Backend-neutral Vite reload state shared by runtime hosts.

use std::sync::mpsc;

use wabou_shell::WakeCallback;

use crate::ui_inbox::{UiInbox, UiInboxSender};

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
    },
    /// Structured Vite transform/runtime diagnostic serialized as JSON.
    Error {
        /// Vite diagnostic payload retained for the JS developer overlay.
        diagnostic: String,
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
    /// Vite rejected the current source; the last-good UI remains mounted.
    Error {
        /// Structured diagnostic serialized as JSON.
        diagnostic: String,
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
    tx: UiInboxSender<ReloadMsg>,
}

impl ReloadHandle {
    /// Enqueue an HMR signal and wake an otherwise idle render loop.
    pub fn send(&self, message: ReloadMsg) -> Result<(), mpsc::SendError<ReloadMsg>> {
        match self.tx.try_send(message) {
            Ok(()) => Ok(()),
            Err(flume::TrySendError::Full(message))
            | Err(flume::TrySendError::Disconnected(message)) => Err(mpsc::SendError(message)),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct HmrBatch {
    pub(crate) full_reload: bool,
    pub(crate) full_reload_reason: Option<String>,
    pub(crate) js_updates: Vec<HmrJsUpdate>,
    pub(crate) css_paths: Vec<String>,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HmrJsUpdate {
    pub(crate) path: String,
    pub(crate) accepted_path: String,
    pub(crate) timestamp: u64,
    pub(crate) source: String,
}

pub(crate) struct ReloadState {
    // Retain one sender so the inbox remains connected even in non-Vite builds.
    _sender: UiInboxSender<ReloadMsg>,
    inbox: UiInbox<ReloadMsg>,
    #[cfg(feature = "vite")]
    vite_entry: Option<String>,
}

impl Default for ReloadState {
    fn default() -> Self {
        let (sender, inbox) = crate::ui_inbox::unbounded();
        Self {
            _sender: sender,
            inbox,
            #[cfg(feature = "vite")]
            vite_entry: None,
        }
    }
}

impl ReloadState {
    #[cfg(any(feature = "vite", test))]
    pub(super) fn handle(&mut self) -> ReloadHandle {
        ReloadHandle {
            tx: self._sender.clone(),
        }
    }

    pub(crate) fn drain(&self) -> Option<HmrBatch> {
        let messages = self.inbox.drain();
        (!messages.is_empty()).then(|| plan_hmr_batch(messages))
    }

    pub(crate) fn is_pending(&self) -> bool {
        self.inbox.has_pending()
    }

    pub(crate) fn set_wake(&self, wake: WakeCallback) {
        self.inbox.set_wake(wake);
    }

    #[cfg(feature = "vite")]
    pub(super) fn set_vite_entry(&mut self, entry: impl Into<String>) {
        self.vite_entry = Some(entry.into());
    }

    #[cfg(feature = "vite")]
    pub(crate) fn vite_entry(&self) -> Option<&str> {
        self.vite_entry.as_deref()
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
            ReloadMsg::CssUpdate { path } => batch.css_paths.push(path),
            ReloadMsg::Error { diagnostic } => batch.error = Some(diagnostic),
        }
    }
    batch
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{ReloadMsg, ReloadState};

    #[test]
    fn sending_wakes_an_idle_event_loop() {
        let mut state = ReloadState::default();
        let handle = state.handle();
        let wakes = std::sync::Arc::new(AtomicUsize::new(0));
        let callback_wakes = wakes.clone();
        state.set_wake(std::sync::Arc::new(move || {
            callback_wakes.fetch_add(1, Ordering::Relaxed);
        }));

        handle.send(ReloadMsg::FullReload).unwrap();

        assert!(state.is_pending());
        assert_eq!(wakes.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn draining_consumes_the_pending_signal_with_the_batch() {
        let mut state = ReloadState::default();
        let handle = state.handle();
        handle.send(ReloadMsg::FullReload).unwrap();

        let batch = state.drain().expect("queued HMR batch");

        assert!(batch.full_reload);
        assert!(!state.is_pending());
        assert!(state.drain().is_none());
    }

    #[test]
    fn latest_vite_error_is_coalesced_for_one_ui_frame() {
        let batch = super::plan_hmr_batch([
            ReloadMsg::Error {
                diagnostic: r#"{"message":"first"}"#.into(),
            },
            ReloadMsg::Error {
                diagnostic: r#"{"message":"latest"}"#.into(),
            },
        ]);

        assert_eq!(batch.error.as_deref(), Some(r#"{"message":"latest"}"#));
    }

    #[test]
    fn vite_error_takes_priority_over_a_same_frame_full_reload() {
        let batch = super::plan_hmr_batch([
            ReloadMsg::FullReload,
            ReloadMsg::Error {
                diagnostic: r#"{"message":"transform failed"}"#.into(),
            },
        ]);

        assert!(batch.full_reload);
        assert_eq!(
            batch.error.as_deref(),
            Some(r#"{"message":"transform failed"}"#)
        );
    }
}
