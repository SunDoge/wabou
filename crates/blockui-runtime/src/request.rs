//! Stable-host pending reply storage for host and SDK actor request envelopes.
use crate::ActorId;
use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    time::Instant,
};

pub use blockui_abi::RequestError;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RequestId(pub u64);
enum ReplyValue {
    Scalar(i64),
    Text(String),
}
struct WaitState {
    result: Option<Result<ReplyValue, RequestError>>,
    waker: Option<Waker>,
    abandoned: bool,
    failure_detail: Option<String>,
}
struct Pending {
    text: bool,
    target: ActorId,
    lifecycle: u64,
    deadline: Instant,
    state: Arc<Mutex<WaitState>>,
}
/// Future owned and polled only by stable-host code. Never passed across the ABI.
pub struct Reply {
    id: RequestId,
    state: Arc<Mutex<WaitState>>,
}
impl Reply {
    pub fn id(&self) -> RequestId {
        self.id
    }
    /// Host-owned handler failure text, available after failure (also after polling).
    /// Status remains RequestError::HandlerFailed; this metadata never crosses the ABI.
    pub fn failure_detail(&self) -> Option<String> {
        self.state.lock().unwrap().failure_detail.clone()
    }
}
/// Owned UTF-8 reply future, held exclusively by stable-host code.
/// Uses the same bounded tracker, deadlines and lifecycle checks as scalar replies.
pub struct TextReply(Reply);
impl TextReply {
    pub fn id(&self) -> RequestId {
        self.0.id()
    }
    pub fn failure_detail(&self) -> Option<String> {
        self.0.failure_detail()
    }
}
impl Future for TextReply {
    type Output = Result<String, RequestError>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.poll_value(cx).map(|result| {
            result.and_then(|value| match value {
                ReplyValue::Text(text) => Ok(text),
                ReplyValue::Scalar(_) => Err(RequestError::HandlerFailed),
            })
        })
    }
}
impl Future for Reply {
    type Output = Result<i64, RequestError>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.poll_value(cx).map(|result| {
            result.and_then(|value| match value {
                ReplyValue::Scalar(value) => Ok(value),
                ReplyValue::Text(_) => Err(RequestError::HandlerFailed),
            })
        })
    }
}
impl Reply {
    fn poll_value(&mut self, cx: &mut Context<'_>) -> Poll<Result<ReplyValue, RequestError>> {
        // RawWaker clone/drop operations belong to host code and may re-enter.
        let candidate = cx.waker().clone();
        let mut state = self.state.lock().unwrap();
        if let Some(result) = state.result.take() {
            drop(state);
            drop(candidate);
            Poll::Ready(result)
        } else {
            let retired = if state
                .waker
                .as_ref()
                .is_none_or(|w| !w.will_wake(&candidate))
            {
                state.waker.replace(candidate)
            } else {
                Some(candidate)
            };
            drop(state);
            drop(retired);
            Poll::Pending
        }
    }
}
impl Drop for Reply {
    fn drop(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.abandoned = true;
        let retired = state.waker.take();
        drop(state);
        // Custom host waker destructors may re-enter reply state.
        drop(retired);
    }
}
/// Bounded low-level request bookkeeping; this does not send actor messages.
#[derive(Default)]
pub struct RequestTracker {
    next: u64,
    pending: BTreeMap<RequestId, Pending>,
}
impl RequestTracker {
    pub fn admit(
        &mut self,
        target: ActorId,
        lifecycle: u64,
        deadline: Instant,
    ) -> Result<(RequestId, Reply), &'static str> {
        self.admit_kind(target, lifecycle, deadline, false)
    }
    pub fn admit_text(
        &mut self,
        target: ActorId,
        lifecycle: u64,
        deadline: Instant,
    ) -> Result<(RequestId, TextReply), &'static str> {
        let (id, reply) = self.admit_kind(target, lifecycle, deadline, true)?;
        Ok((id, TextReply(reply)))
    }
    fn admit_kind(
        &mut self,
        target: ActorId,
        lifecycle: u64,
        deadline: Instant,
        text: bool,
    ) -> Result<(RequestId, Reply), &'static str> {
        self.reap();
        if target.0 == 0 || lifecycle == 0 {
            return Err("invalid request target lifecycle");
        }
        if self.pending.len() >= 256 {
            return Err("pending request limit reached");
        }
        let next = self
            .next
            .checked_add(1)
            .ok_or("request identity exhausted")?;
        let state = Arc::new(Mutex::new(WaitState {
            result: None,
            waker: None,
            abandoned: false,
            failure_detail: None,
        }));
        let id = RequestId(next);
        self.pending.insert(
            id,
            Pending {
                text,
                target,
                lifecycle,
                deadline,
                state: state.clone(),
            },
        );
        self.next = next;
        Ok((id, Reply { id, state }))
    }
    /// Validate lifecycle and deadline before accepting an owned committed reply.
    pub fn resolve(
        &mut self,
        id: RequestId,
        target: ActorId,
        lifecycle: u64,
        value: i64,
        now: Instant,
    ) -> bool {
        self.resolve_value(id, target, lifecycle, ReplyValue::Scalar(value), now)
    }
    /// Transfer a committed UTF-8 reply into stable-host ownership without copying.
    /// Oversized or wrong-kind replies leave the pending request unchanged.
    pub fn resolve_text(
        &mut self,
        id: RequestId,
        target: ActorId,
        lifecycle: u64,
        value: String,
        now: Instant,
    ) -> bool {
        if value.len() > blockui_abi::MAX_MESSAGE_TEXT_BYTES {
            return false;
        }
        self.resolve_value(id, target, lifecycle, ReplyValue::Text(value), now)
    }
    fn resolve_value(
        &mut self,
        id: RequestId,
        target: ActorId,
        lifecycle: u64,
        value: ReplyValue,
        now: Instant,
    ) -> bool {
        let Some(pending) = self.pending.get(&id) else {
            return false;
        };
        if pending.text != matches!(value, ReplyValue::Text(_))
            || pending.target != target
            || pending.lifecycle != lifecycle
        {
            return false;
        }
        let expired = now >= pending.deadline;
        let delivered = self.finish(
            id,
            if expired {
                Err(RequestError::Timeout)
            } else {
                Ok(value)
            },
        );
        !expired && delivered
    }
    pub(crate) fn is_pending(
        &self,
        id: RequestId,
        target: ActorId,
        lifecycle: u64,
        now: Instant,
    ) -> bool {
        self.pending.get(&id).is_some_and(|p| {
            p.target == target
                && p.lifecycle == lifecycle
                && now < p.deadline
                && !p.state.lock().unwrap().abandoned
        })
    }
    pub(crate) fn handler_failed(&mut self, id: RequestId, reason: &str) {
        if let Some(pending) = self.pending.get(&id) {
            let mut end = reason.len().min(4096);
            while !reason.is_char_boundary(end) {
                end -= 1;
            }
            pending.state.lock().unwrap().failure_detail = Some(reason[..end].to_owned());
        }
        self.finish(id, Err(RequestError::HandlerFailed));
    }
    pub fn cancel(&mut self, id: RequestId) -> bool {
        self.finish(id, Err(RequestError::Cancelled))
    }
    pub fn terminate(&mut self, target: ActorId, lifecycle: u64) {
        let ids: Vec<_> = self
            .pending
            .iter()
            .filter_map(|(id, p)| (p.target == target && p.lifecycle == lifecycle).then_some(*id))
            .collect();
        for id in ids {
            self.finish(id, Err(RequestError::TargetTerminated));
        }
    }
    /// The stable runtime must call this on its deadline tick.
    pub fn expire(&mut self, now: Instant) {
        self.reap();
        let ids: Vec<_> = self
            .pending
            .iter()
            .filter_map(|(id, p)| (now >= p.deadline).then_some(*id))
            .collect();
        for id in ids {
            self.finish(id, Err(RequestError::Timeout));
        }
    }
    pub fn next_deadline(&self) -> Option<Instant> {
        self.pending
            .values()
            .filter(|pending| !pending.state.lock().unwrap().abandoned)
            .map(|pending| pending.deadline)
            .min()
    }
    fn reap(&mut self) {
        self.pending
            .retain(|_, p| !p.state.lock().unwrap().abandoned);
    }
    fn finish(&mut self, id: RequestId, result: Result<ReplyValue, RequestError>) -> bool {
        let Some(pending) = self.pending.remove(&id) else {
            return false;
        };
        let waker = {
            let mut state = pending.state.lock().unwrap();
            if state.abandoned {
                return false;
            }
            state.result = Some(result);
            state.waker.take()
        };
        // No tracker/state lock is held across caller wake code.
        if let Some(waker) = waker {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| waker.wake()));
        }
        true
    }
}
impl Drop for RequestTracker {
    fn drop(&mut self) {
        while let Some(id) = self.pending.keys().next().copied() {
            self.finish(id, Err(RequestError::RuntimeStopped));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn poll(reply: &mut Reply) -> Poll<Result<i64, RequestError>> {
        Pin::new(reply).poll(&mut Context::from_waker(Waker::noop()))
    }
    fn poll_text(reply: &mut TextReply) -> Poll<Result<String, RequestError>> {
        Pin::new(reply).poll(&mut Context::from_waker(Waker::noop()))
    }
    #[test]
    fn text_replies_keep_owned_buffers_and_validate_kind_size_and_lifecycle() {
        let now = Instant::now();
        let deadline = now + std::time::Duration::from_secs(30);
        let mut tracker = RequestTracker::default();
        let (id, mut reply) = tracker.admit_text(ActorId(1), 7, deadline).unwrap();
        assert_eq!(poll_text(&mut reply), Poll::Pending);
        assert!(!tracker.resolve(id, ActorId(1), 7, 42, now));
        assert!(!tracker.resolve_text(id, ActorId(1), 8, "stale".into(), now));
        assert!(!tracker.resolve_text(id, ActorId(2), 7, "wrong actor".into(), now));
        assert!(!tracker.resolve_text(
            id,
            ActorId(1),
            7,
            "x".repeat(blockui_abi::MAX_MESSAGE_TEXT_BYTES + 1),
            now
        ));
        assert_eq!(poll_text(&mut reply), Poll::Pending);
        let text = String::from("資料😀");
        let allocation = text.as_ptr();
        assert!(tracker.resolve_text(id, ActorId(1), 7, text, now));
        drop(tracker);
        let Poll::Ready(Ok(text)) = poll_text(&mut reply) else {
            panic!("text reply")
        };
        assert_eq!(text, "資料😀");
        assert_eq!(text.as_ptr(), allocation);
        let mut tracker = RequestTracker::default();
        for text in [
            String::new(),
            "x".repeat(blockui_abi::MAX_MESSAGE_TEXT_BYTES),
        ] {
            let (id, mut reply) = tracker.admit_text(ActorId(1), 7, deadline).unwrap();
            assert!(tracker.resolve_text(id, ActorId(1), 7, text.clone(), now));
            assert_eq!(poll_text(&mut reply), Poll::Ready(Ok(text)));
        }
        let (id, mut scalar) = tracker.admit(ActorId(1), 7, deadline).unwrap();
        assert!(!tracker.resolve_text(id, ActorId(1), 7, "wrong kind".into(), now));
        assert!(tracker.resolve(id, ActorId(1), 7, 42, now));
        assert_eq!(poll(&mut scalar), Poll::Ready(Ok(42)));
    }
    #[test]
    fn text_waiters_share_capacity_and_all_terminal_paths() {
        let now = Instant::now();
        let deadline = now + std::time::Duration::from_secs(30);
        let mut tracker = RequestTracker::default();
        let mut replies = Vec::new();
        for _ in 0..255 {
            replies.push(tracker.admit_text(ActorId(1), 7, deadline).unwrap().1);
        }
        let scalar = tracker.admit(ActorId(1), 7, deadline).unwrap().1;
        assert!(tracker.admit_text(ActorId(1), 7, deadline).is_err());
        assert!(tracker.admit(ActorId(1), 7, deadline).is_err());
        drop(scalar);
        let (id, mut cancelled) = tracker.admit_text(ActorId(1), 7, deadline).unwrap();
        assert!(tracker.cancel(id));
        assert_eq!(
            poll_text(&mut cancelled),
            Poll::Ready(Err(RequestError::Cancelled))
        );
        tracker.terminate(ActorId(1), 8);
        assert_eq!(poll_text(&mut replies[0]), Poll::Pending);
        tracker.terminate(ActorId(1), 7);
        for reply in &mut replies {
            assert_eq!(
                poll_text(reply),
                Poll::Ready(Err(RequestError::TargetTerminated))
            );
        }
        let (id, mut timed_out) = tracker.admit_text(ActorId(1), 7, now).unwrap();
        assert!(!tracker.resolve_text(id, ActorId(1), 7, "late".into(), now));
        assert_eq!(
            poll_text(&mut timed_out),
            Poll::Ready(Err(RequestError::Timeout))
        );
        let (id, mut failed) = tracker.admit_text(ActorId(1), 7, deadline).unwrap();
        tracker.handler_failed(id, "文件已关闭");
        assert_eq!(
            poll_text(&mut failed),
            Poll::Ready(Err(RequestError::HandlerFailed))
        );
        assert_eq!(failed.failure_detail().as_deref(), Some("文件已关闭"));
        let (_, mut stopped) = tracker.admit_text(ActorId(1), 7, deadline).unwrap();
        drop(tracker);
        assert_eq!(
            poll_text(&mut stopped),
            Poll::Ready(Err(RequestError::RuntimeStopped))
        );
    }
    #[test]
    fn failed_reply_retains_bounded_owned_detail_after_poll_and_tracker_disposal() {
        let mut tracker = RequestTracker::default();
        let (id, mut reply) = tracker
            .admit(
                ActorId(1),
                1,
                Instant::now() + std::time::Duration::from_secs(30),
            )
            .unwrap();
        assert!(reply.failure_detail().is_none());
        let reason = format!("{}😀suffix", "x".repeat(4095));
        tracker.handler_failed(id, &reason);
        drop(reason);
        assert_eq!(
            poll(&mut reply),
            Poll::Ready(Err(RequestError::HandlerFailed))
        );
        drop(tracker);
        assert_eq!(reply.failure_detail(), Some("x".repeat(4095)));
    }
    #[test]
    fn abandoned_reply_drops_waker_without_holding_state_lock() {
        use std::sync::atomic::{AtomicBool, Ordering};
        struct Probe {
            state: Arc<Mutex<WaitState>>,
            unlocked: Arc<AtomicBool>,
        }
        // A custom destructor is the behavior under test.
        #[allow(clippy::manual_noop_waker)]
        impl std::task::Wake for Probe {
            fn wake(self: Arc<Self>) {}
        }
        impl Drop for Probe {
            fn drop(&mut self) {
                self.unlocked
                    .store(self.state.try_lock().is_ok(), Ordering::Release);
            }
        }
        let mut tracker = RequestTracker::default();
        let (_, mut reply) = tracker
            .admit(
                ActorId(1),
                1,
                Instant::now() + std::time::Duration::from_secs(2),
            )
            .unwrap();
        let unlocked = Arc::new(AtomicBool::new(false));
        let waker = Waker::from(Arc::new(Probe {
            state: reply.state.clone(),
            unlocked: unlocked.clone(),
        }));
        assert_eq!(
            Pin::new(&mut reply).poll(&mut Context::from_waker(&waker)),
            Poll::Pending
        );
        drop(waker);
        assert_eq!(poll(&mut reply), Poll::Pending);
        assert!(
            unlocked.load(Ordering::Acquire),
            "replaced waker destructor ran under reply state lock"
        );
        unlocked.store(false, Ordering::Release);
        let waker = Waker::from(Arc::new(Probe {
            state: reply.state.clone(),
            unlocked: unlocked.clone(),
        }));
        assert_eq!(
            Pin::new(&mut reply).poll(&mut Context::from_waker(&waker)),
            Poll::Pending
        );
        drop(waker);
        drop(reply);
        assert!(
            unlocked.load(Ordering::Acquire),
            "waker destructor ran under reply state lock"
        );
        tracker.expire(Instant::now());
        assert!(tracker.pending.is_empty());
    }

    #[test]
    fn deadlines_lifecycle_and_duplicate_replies() {
        let mut tracker = RequestTracker::default();
        let now = Instant::now();
        let (id, mut reply) = tracker
            .admit(ActorId(1), 5, now + std::time::Duration::from_secs(1))
            .unwrap();
        assert_eq!(poll(&mut reply), Poll::Pending);
        assert!(!tracker.resolve(id, ActorId(1), 6, 7, now));
        assert!(tracker.resolve(id, ActorId(1), 5, 7, now));
        assert!(!tracker.resolve(id, ActorId(1), 5, 8, now));
        assert_eq!(poll(&mut reply), Poll::Ready(Ok(7)));
        let (late, mut reply) = tracker.admit(ActorId(1), 6, now).unwrap();
        assert!(!tracker.resolve(late, ActorId(1), 6, 8, now));
        assert_eq!(poll(&mut reply), Poll::Ready(Err(RequestError::Timeout)));
    }
    #[test]
    fn cancellation_removal_shutdown_and_capacity() {
        let mut tracker = RequestTracker::default();
        let deadline = Instant::now() + std::time::Duration::from_secs(1);
        let mut replies = Vec::new();
        for _ in 0..256 {
            replies.push(tracker.admit(ActorId(1), 9, deadline).unwrap());
        }
        assert!(tracker.admit(ActorId(2), 1, deadline).is_err());
        let (id, mut reply) = replies.pop().unwrap();
        assert!(tracker.cancel(id));
        assert!(!tracker.cancel(id));
        assert_eq!(poll(&mut reply), Poll::Ready(Err(RequestError::Cancelled)));
        drop(replies.pop());
        let (_, mut fresh) = tracker.admit(ActorId(2), 10, deadline).unwrap();
        tracker.terminate(ActorId(1), 8);
        assert_eq!(poll(&mut replies[0].1), Poll::Pending);
        tracker.terminate(ActorId(1), 9);
        assert_eq!(
            poll(&mut replies[0].1),
            Poll::Ready(Err(RequestError::TargetTerminated))
        );
        drop(tracker);
        assert_eq!(
            poll(&mut fresh),
            Poll::Ready(Err(RequestError::RuntimeStopped))
        );
    }
    #[test]
    fn completion_wakes_waiter_and_abandoned_reply_is_not_delivered() {
        struct Count(std::sync::atomic::AtomicUsize);
        impl std::task::Wake for Count {
            fn wake(self: Arc<Self>) {
                self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
        let counter = Arc::new(Count(std::sync::atomic::AtomicUsize::new(0)));
        let waker = Waker::from(counter.clone());
        let mut tracker = RequestTracker::default();
        let now = Instant::now();
        let deadline = now + std::time::Duration::from_secs(1);
        let (id, mut reply) = tracker.admit(ActorId(1), 1, deadline).unwrap();
        assert_eq!(
            Pin::new(&mut reply).poll(&mut Context::from_waker(&waker)),
            Poll::Pending
        );
        assert!(tracker.resolve(id, ActorId(1), 1, 42, now));
        assert_eq!(counter.0.load(std::sync::atomic::Ordering::Relaxed), 1);
        assert_eq!(poll(&mut reply), Poll::Ready(Ok(42)));
        let (id, reply) = tracker.admit(ActorId(1), 1, deadline).unwrap();
        drop(reply);
        assert!(!tracker.resolve(id, ActorId(1), 1, 9, now));
    }
    #[test]
    fn abandoned_waiters_do_not_schedule_deadline_wakes() {
        let mut tracker = RequestTracker::default();
        let now = Instant::now();
        let later = now + std::time::Duration::from_secs(60);
        let (_, abandoned) = tracker.admit(ActorId(1), 1, now).unwrap();
        let (_, mut live) = tracker.admit(ActorId(2), 1, later).unwrap();
        assert_eq!(tracker.next_deadline(), Some(now));
        drop(abandoned);
        assert_eq!(tracker.next_deadline(), Some(later));
        tracker.expire(now);
        assert_eq!(poll(&mut live), Poll::Pending);
        tracker.expire(later);
        assert_eq!(poll(&mut live), Poll::Ready(Err(RequestError::Timeout)));
        assert_eq!(tracker.next_deadline(), None);
        let (_, final_waiter) = tracker.admit(ActorId(3), 1, later).unwrap();
        drop(final_waiter);
        assert_eq!(tracker.next_deadline(), None);
    }

    #[test]
    fn deadline_tick_and_identity_exhaustion() {
        let mut tracker = RequestTracker::default();
        let now = Instant::now();
        let (_, mut reply) = tracker.admit(ActorId(1), 1, now).unwrap();
        assert_eq!(tracker.next_deadline(), Some(now));
        tracker.expire(now);
        assert_eq!(poll(&mut reply), Poll::Ready(Err(RequestError::Timeout)));
        tracker.next = u64::MAX;
        assert!(tracker.admit(ActorId(1), 1, now).is_err());
    }
}
