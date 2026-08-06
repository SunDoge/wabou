//! Host → JS application messages.
//!
//! Background tasks push [`HostMsg`] into a bounded queue without touching
//! QuickJS. The applier drains on the UI thread and encodes each item as an
//! application record in the unified HostEventFrame.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use wabou_shell::WakeCallback;

/// Default bound: producers `try_send` and get [`HostMsgError::Full`] when the
/// UI thread is not draining fast enough.
pub const DEFAULT_HOST_MSG_CAPACITY: usize = 1024;

/// Max messages forwarded to JS in a single frame.
pub const MAX_HOST_MSG_PER_FRAME: usize = 128;

/// Max UTF-8 topic length (u16).
pub const MAX_TOPIC_BYTES: usize = 0xffff;

/// Max string payload length (u16).
pub const MAX_STR_PAYLOAD_BYTES: usize = 0xffff;

/// Max binary payload length.
pub const MAX_BYTES_PAYLOAD: usize = 1024 * 1024;

/// Payload kinds in an application HostEventFrame record.
pub mod kind {
    pub const NULL: u8 = 0;
    pub const BOOL: u8 = 1;
    pub const I32: u8 = 2;
    pub const F64: u8 = 3;
    pub const STR: u8 = 4;
    pub const BYTES: u8 = 5;
}

/// Typed payload — no JSON on the hot path.
#[derive(Debug, Clone, PartialEq)]
pub enum HostPayload {
    Null,
    Bool(bool),
    I32(i32),
    F64(f64),
    Str(String),
    Bytes(Vec<u8>),
}

/// One application-level notification from Rust to JS.
#[derive(Debug, Clone, PartialEq)]
pub struct HostMsg {
    pub topic: String,
    pub payload: HostPayload,
}

impl HostMsg {
    pub fn null(topic: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            payload: HostPayload::Null,
        }
    }

    pub fn bool(topic: impl Into<String>, value: bool) -> Self {
        Self {
            topic: topic.into(),
            payload: HostPayload::Bool(value),
        }
    }

    pub fn i32(topic: impl Into<String>, value: i32) -> Self {
        Self {
            topic: topic.into(),
            payload: HostPayload::I32(value),
        }
    }

    pub fn f64(topic: impl Into<String>, value: f64) -> Self {
        Self {
            topic: topic.into(),
            payload: HostPayload::F64(value),
        }
    }

    pub fn str(topic: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            payload: HostPayload::Str(value.into()),
        }
    }

    pub fn bytes(topic: impl Into<String>, value: impl Into<Vec<u8>>) -> Self {
        Self {
            topic: topic.into(),
            payload: HostPayload::Bytes(value.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostMsgError {
    Full,
    Disconnected,
    /// Topic or string payload exceeds length limits.
    TooLarge,
}

/// Cloneable, thread-safe handle for enqueueing host messages.
#[derive(Clone)]
pub struct HostMsgHandle {
    tx: SyncSender<HostMsg>,
    pending: Arc<AtomicBool>,
    wake: Arc<Mutex<Option<WakeCallback>>>,
}

impl HostMsgHandle {
    /// Non-blocking send. Safe from any thread.
    pub fn send(&self, msg: HostMsg) -> Result<(), HostMsgError> {
        if msg.topic.len() > MAX_TOPIC_BYTES {
            return Err(HostMsgError::TooLarge);
        }
        match &msg.payload {
            HostPayload::Str(s) if s.len() > MAX_STR_PAYLOAD_BYTES => {
                return Err(HostMsgError::TooLarge);
            }
            HostPayload::Bytes(b) if b.len() > MAX_BYTES_PAYLOAD => {
                return Err(HostMsgError::TooLarge);
            }
            _ => {}
        }
        match self.tx.try_send(msg) {
            Ok(()) => {
                self.pending.store(true, Ordering::Release);
                if let Ok(slot) = self.wake.lock()
                    && let Some(wake) = slot.as_ref()
                {
                    wake();
                }
                Ok(())
            }
            Err(TrySendError::Full(_)) => Err(HostMsgError::Full),
            Err(TrySendError::Disconnected(_)) => Err(HostMsgError::Disconnected),
        }
    }

    pub fn emit_null(&self, topic: impl Into<String>) -> Result<(), HostMsgError> {
        self.send(HostMsg::null(topic))
    }

    pub fn emit_bool(&self, topic: impl Into<String>, value: bool) -> Result<(), HostMsgError> {
        self.send(HostMsg::bool(topic, value))
    }

    pub fn emit_i32(&self, topic: impl Into<String>, value: i32) -> Result<(), HostMsgError> {
        self.send(HostMsg::i32(topic, value))
    }

    pub fn emit_f64(&self, topic: impl Into<String>, value: f64) -> Result<(), HostMsgError> {
        self.send(HostMsg::f64(topic, value))
    }

    pub fn emit_str(
        &self,
        topic: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), HostMsgError> {
        self.send(HostMsg::str(topic, value))
    }

    pub fn emit_bytes(
        &self,
        topic: impl Into<String>,
        value: impl Into<Vec<u8>>,
    ) -> Result<(), HostMsgError> {
        self.send(HostMsg::bytes(topic, value))
    }

    /// Retry `try_send` until `timeout`.
    pub fn send_timeout(&self, msg: HostMsg, timeout: Duration) -> Result<(), HostMsgError> {
        let start = std::time::Instant::now();
        loop {
            match self.send(msg.clone()) {
                Ok(()) => return Ok(()),
                Err(HostMsgError::Full) if start.elapsed() < timeout => {
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(e) => return Err(e),
            }
        }
    }
}

/// Receiver half owned by the applier (UI thread).
pub(crate) struct HostMsgInbox {
    rx: Receiver<HostMsg>,
    pending: Arc<AtomicBool>,
    wake: Arc<Mutex<Option<WakeCallback>>>,
}

impl HostMsgInbox {
    pub(crate) fn set_wake(&self, wake: WakeCallback) {
        if let Ok(mut slot) = self.wake.lock() {
            *slot = Some(wake);
        }
    }

    pub(crate) fn has_pending(&self) -> bool {
        self.pending.load(Ordering::Acquire)
    }

    pub(crate) fn drain_batch(&self) -> Vec<HostMsg> {
        let mut batch = Vec::new();
        while batch.len() < MAX_HOST_MSG_PER_FRAME {
            match self.rx.try_recv() {
                Ok(msg) => batch.push(msg),
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }
        if batch.len() >= MAX_HOST_MSG_PER_FRAME {
            self.pending.store(true, Ordering::Release);
        } else {
            self.pending.store(false, Ordering::Release);
        }
        batch
    }
}

pub(crate) fn host_msg_channel(capacity: usize) -> (HostMsgHandle, HostMsgInbox) {
    let capacity = capacity.max(1);
    let (tx, rx) = mpsc::sync_channel(capacity);
    let pending = Arc::new(AtomicBool::new(false));
    let wake = Arc::new(Mutex::new(None));
    (
        HostMsgHandle {
            tx,
            pending: pending.clone(),
            wake: wake.clone(),
        },
        HostMsgInbox { rx, pending, wake },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_send_and_drain_respects_frame_cap() {
        let (tx, rx) = host_msg_channel(8);
        for i in 0..5 {
            tx.emit_i32("t", i).unwrap();
        }
        assert!(rx.has_pending());
        let batch = rx.drain_batch();
        assert_eq!(batch.len(), 5);
        assert!(!rx.has_pending());
    }

    #[test]
    fn full_queue_returns_full() {
        let (tx, _rx) = host_msg_channel(2);
        tx.emit_i32("a", 1).unwrap();
        tx.emit_i32("a", 2).unwrap();
        assert_eq!(tx.emit_i32("a", 3), Err(HostMsgError::Full));
    }

    #[test]
    fn per_frame_cap_keeps_pending() {
        let (tx, rx) = host_msg_channel(MAX_HOST_MSG_PER_FRAME + 10);
        for i in 0..(MAX_HOST_MSG_PER_FRAME + 3) {
            tx.emit_i32("t", i as i32).unwrap();
        }
        let batch = rx.drain_batch();
        assert_eq!(batch.len(), MAX_HOST_MSG_PER_FRAME);
        assert!(rx.has_pending());
        let rest = rx.drain_batch();
        assert_eq!(rest.len(), 3);
        assert!(!rx.has_pending());
    }
}
