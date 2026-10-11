//! Stable-host dormant timers, sharing one lazy scheduler per runtime.
use crate::{TimerCompletion, wake};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};
pub(crate) struct Spec {
    pub completion: TimerCompletion,
    pub duration: Duration,
    pub repeating: bool,
    pub cancelled: Arc<AtomicBool>,
    pub started: Arc<AtomicBool>,
}
struct Delivery {
    sender: mpsc::SyncSender<TimerCompletion>,
    wake: Arc<wake::Slot>,
}
struct Job {
    shared: std::sync::Weak<Shared>,
    backpressure_notified: AtomicBool,
    key: u64,
    delivery: Mutex<Option<Delivery>>,
    spec: Spec,
    deadline: Mutex<Option<Instant>>,
    #[cfg(feature = "testing")]
    test_deadline: Mutex<Option<Duration>>,
    done: Mutex<bool>,
    finished: Condvar,
    worker: thread::Thread,
}
pub(crate) struct Handle(Arc<Job>);
impl Handle {
    pub(crate) fn set_deadline(&self, duration: Duration) {
        let done = self.0.done.lock().unwrap();
        if *done {
            return;
        }
        *self.0.deadline.lock().unwrap() = Some(
            Instant::now()
                .checked_add(duration)
                .expect("bounded timer delay validated during preparation"),
        );
        #[cfg(feature = "testing")]
        if let Some(shared) = self.0.shared.upgrade().filter(|shared| shared.manual) {
            let now = *shared.test_now.lock().unwrap();
            *self.0.test_deadline.lock().unwrap() = Some(now.saturating_add(duration));
        }
    }
    #[cfg(feature = "testing")]
    pub(crate) fn complete_if_due_for_test(&self, now: Duration) -> Result<bool, String> {
        let due = self
            .0
            .test_deadline
            .lock()
            .unwrap()
            .is_some_and(|due| due <= now);
        if !due {
            return Ok(false);
        }
        self.complete_for_test()?;
        Ok(true)
    }
    #[cfg(feature = "testing")]
    pub(crate) fn complete_for_test(&self) -> Result<(), String> {
        let done = self.0.done.lock().unwrap();
        let mut deadline = self.0.deadline.lock().unwrap();
        if *done || deadline.is_none() || !self.0.spec.started.load(Ordering::Acquire) {
            return Err("timer has no pending completion".into());
        }
        let delivery_guard = self.0.delivery.lock().unwrap();
        let delivery = delivery_guard.as_ref().ok_or("timer delivery retired")?;
        delivery
            .sender
            .try_send(self.0.spec.completion.clone())
            .map_err(|error| format!("could not admit test timer completion: {error}"))?;
        *deadline = None;
        *self.0.test_deadline.lock().unwrap() = None;
        let wake = delivery.wake.clone();
        drop(delivery_guard);
        drop(deadline);
        drop(done);
        self.unpark();
        wake.notify();
        Ok(())
    }
    pub(crate) fn acknowledge(&self) {
        if self.0.spec.repeating {
            self.set_deadline(self.0.spec.duration);
            self.unpark();
        }
    }
    pub(crate) fn unpark(&self) {
        let done = self.0.done.lock().unwrap();
        if *done {
            return;
        }
        if let Some(shared) = self.0.shared.upgrade() {
            shared.changed.store(true, Ordering::Release);
        }
        self.0.worker.unpark();
    }
    pub(crate) fn worker_id(&self) -> thread::ThreadId {
        self.0.worker.id()
    }
    pub(crate) fn is_finished(&self) -> bool {
        *self.0.done.lock().unwrap()
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        self.0.spec.cancelled.store(true, Ordering::Release);
        self.unpark();
        if thread::current().id() == self.worker_id() {
            return;
        }
        let mut done = self.0.done.lock().unwrap();
        while !*done {
            done = self.0.finished.wait(done).unwrap();
        }
    }
}
#[derive(Default)]
struct Shared {
    manual: bool,
    #[cfg(feature = "testing")]
    test_now: Arc<Mutex<Duration>>,
    changed: AtomicBool,
    jobs: Mutex<BTreeMap<u64, Arc<Job>>>,
    stop: AtomicBool,
}
struct Worker {
    next_key: u64,
    shared: Arc<Shared>,
    thread: thread::JoinHandle<()>,
}
#[derive(Default)]
pub(crate) struct Pool {
    #[cfg(feature = "testing")]
    manual: AtomicBool,
    #[cfg(feature = "testing")]
    test_now: Arc<Mutex<Duration>>,
    worker: Mutex<Option<Worker>>,
}
#[derive(Default)]
pub(crate) struct Executor {
    pool: Arc<Pool>,
}
impl Executor {
    #[cfg(feature = "testing")]
    pub(crate) fn use_manual_timers_for_test(&self) -> Result<(), String> {
        if Arc::strong_count(&self.pool) != 1 {
            return Err("manual test timers require an isolated actor scheduler".into());
        }
        let worker = self.pool.worker.lock().unwrap();
        if worker.is_some() {
            return Err("configure manual timers before timer admission".into());
        }
        self.pool.manual.store(true, Ordering::Release);
        Ok(())
    }
    #[cfg(feature = "testing")]
    pub(crate) fn advance_for_test(&self, elapsed: Duration) -> Result<Duration, String> {
        if !self.pool.manual.load(Ordering::Acquire) {
            return Err("configure manual timers before advancing timer time".into());
        }
        let mut now = self.pool.test_now.lock().unwrap();
        *now = now.checked_add(elapsed).ok_or("test timer time overflow")?;
        Ok(*now)
    }
    pub(crate) fn use_pool(&mut self, pool: Arc<Pool>) {
        assert!(
            self.pool.worker.lock().unwrap().is_none(),
            "attach before admission"
        );
        self.pool = pool;
    }
    pub(crate) fn prepare(
        &self,
        actor: u64,
        spec: Spec,
        sender: mpsc::SyncSender<TimerCompletion>,
        wake: Arc<wake::Slot>,
    ) -> Result<Handle, String> {
        let mut worker = self.pool.worker.lock().unwrap();
        if worker.is_none() {
            let shared = Arc::new(Shared {
                #[cfg(feature = "testing")]
                manual: self.pool.manual.load(Ordering::Acquire),
                #[cfg(feature = "testing")]
                test_now: self.pool.test_now.clone(),
                ..Shared::default()
            });
            let state = shared.clone();
            let thread = thread::Builder::new()
                .name(format!("actor-{actor}-timers"))
                .spawn(move || run(state))
                .map_err(|e| format!("could not start timer scheduler: {e}"))?;
            *worker = Some(Worker {
                next_key: 1,
                shared,
                thread,
            });
        }
        let worker = worker.as_mut().unwrap();
        let key = worker.next_key;
        worker.next_key = key.checked_add(1).ok_or("timer pool identity exhausted")?;
        let initial_deadline = if spec.started.load(Ordering::Acquire) {
            Instant::now().checked_add(spec.duration)
        } else {
            None
        };
        let job = Arc::new(Job {
            shared: Arc::downgrade(&worker.shared),
            backpressure_notified: AtomicBool::new(false),
            key,
            delivery: Mutex::new(Some(Delivery { sender, wake })),
            deadline: Mutex::new(initial_deadline),
            #[cfg(feature = "testing")]
            test_deadline: Mutex::new(
                if worker.shared.manual && spec.started.load(Ordering::Acquire) {
                    Some(
                        self.pool
                            .test_now
                            .lock()
                            .unwrap()
                            .saturating_add(spec.duration),
                    )
                } else {
                    None
                },
            ),
            spec,
            done: Mutex::new(false),
            finished: Condvar::new(),
            worker: worker.thread.thread().clone(),
        });
        worker.shared.jobs.lock().unwrap().insert(key, job.clone());
        let handle = Handle(job);
        handle.unpark();
        Ok(handle)
    }
}
impl Drop for Pool {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.get_mut().unwrap().take() {
            worker.shared.stop.store(true, Ordering::Release);
            worker.shared.changed.store(true, Ordering::Release);
            worker.thread.thread().unpark();
            // A host wake callback may release the last owner on this worker.
            // Its stop signal is already published; joining itself would panic.
            if thread::current().id() != worker.thread.thread().id() {
                let _ = worker.thread.join();
            }
        }
    }
}
fn run(shared: Arc<Shared>) {
    loop {
        shared.changed.store(false, Ordering::Release);
        let stop = shared.stop.load(Ordering::Acquire);
        let jobs: Vec<_> = shared.jobs.lock().unwrap().values().cloned().collect();
        let mut next: Option<Instant> = None;
        for job in jobs {
            let mut done =
                shared.stop.load(Ordering::Acquire) || job.spec.cancelled.load(Ordering::Acquire);
            let mut notify = false;
            if !done && job.spec.started.load(Ordering::Acquire) {
                let now = Instant::now();
                {
                    let mut deadline = job.deadline.lock().unwrap();
                    if let Some(due) = *deadline {
                        let candidate = if due <= now && !shared.manual {
                            match job
                                .delivery
                                .lock()
                                .unwrap()
                                .as_ref()
                                .unwrap()
                                .sender
                                .try_send(job.spec.completion.clone())
                            {
                                Ok(()) => {
                                    job.backpressure_notified.store(false, Ordering::Release);
                                    done = !job.spec.repeating;
                                    if job.spec.repeating {
                                        *deadline = None;
                                    }
                                    notify = true;
                                }
                                Err(mpsc::TrySendError::Disconnected(_)) => done = true,
                                Err(mpsc::TrySendError::Full(_)) => {
                                    notify =
                                        !job.backpressure_notified.swap(true, Ordering::AcqRel);
                                }
                            }
                            now + Duration::from_millis(10)
                        } else {
                            due
                        };
                        if !done && deadline.is_some() && !shared.manual {
                            next = Some(next.map_or(candidate, |old| old.min(candidate)));
                        }
                    } else if !job.spec.repeating {
                        done = true;
                    }
                }
            }
            let wake = notify.then(|| job.delivery.lock().unwrap().as_ref().unwrap().wake.clone());
            if done {
                let retired = job.delivery.lock().unwrap().take();
                shared.jobs.lock().unwrap().remove(&job.key);
                *job.done.lock().unwrap() = true;
                job.finished.notify_all();
                // Callback captures may have host destructors; retire outside
                // locks, after marking the task finished.
                drop(retired);
            }
            // A reentrant wake may acknowledge an interval or cancel its own
            // handle. No scheduler/task locks are held during host callbacks.
            if let Some(wake) = wake {
                wake.notify();
            }
        }
        if stop {
            return;
        }
        // Callbacks may consume thread park tokens internally. Keep a durable
        // change flag so insertions/activation/cancellation cannot be lost.
        if shared.changed.load(Ordering::Acquire) {
            continue;
        }
        match next {
            Some(deadline) => {
                thread::park_timeout(deadline.saturating_duration_since(Instant::now()))
            }
            None => thread::park(),
        }
    }
}

pub(crate) fn validate_deadline(duration: Duration) -> Result<(), String> {
    Instant::now()
        .checked_add(duration)
        .ok_or_else(|| "timer deadline out of range".into())
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn spec(id: u64, duration: Duration) -> Spec {
        Spec {
            completion: TimerCompletion {
                text: None,
                network: None,
                id,
                generation: id,
                message: crate::Message {
                    action: 7,
                    value: id as i64,
                },
            },
            duration,
            repeating: false,
            cancelled: Arc::new(AtomicBool::new(false)),
            started: Arc::new(AtomicBool::new(true)),
        }
    }
    #[cfg(feature = "testing")]
    #[test]
    fn virtual_due_delivery_retries_backpressure_and_clock_overflow_is_atomic() {
        let executor = Executor::default();
        executor.use_manual_timers_for_test().unwrap();
        let (sender, receiver) = mpsc::sync_channel(1);
        sender
            .try_send(spec(99, Duration::ZERO).completion)
            .unwrap();
        let handle = executor
            .prepare(1, spec(1, Duration::from_secs(2)), sender, Arc::default())
            .unwrap();
        let now = executor.advance_for_test(Duration::from_secs(1)).unwrap();
        assert!(!handle.complete_if_due_for_test(now).unwrap());
        let now = executor.advance_for_test(Duration::from_secs(1)).unwrap();
        assert!(handle.complete_if_due_for_test(now).is_err());
        assert_eq!(receiver.try_recv().unwrap().id, 99);
        assert!(handle.complete_if_due_for_test(now).unwrap());
        assert_eq!(receiver.try_recv().unwrap().id, 1);
        assert!(!handle.complete_if_due_for_test(now).unwrap());
        assert!(executor.advance_for_test(Duration::MAX).is_err());
        assert_eq!(executor.advance_for_test(Duration::ZERO).unwrap(), now);
    }

    #[cfg(feature = "testing")]
    #[test]
    fn manual_configuration_rejects_shared_pool_without_changing_other_actors() {
        let pool = Arc::new(Pool::default());
        let mut first = Executor::default();
        let mut second = Executor::default();
        first.use_pool(pool.clone());
        second.use_pool(pool.clone());
        assert!(first.use_manual_timers_for_test().is_err());
        assert!(!pool.manual.load(Ordering::Acquire));
        let (sender, receiver) = mpsc::sync_channel(1);
        let _handle = second
            .prepare(2, spec(1, Duration::ZERO), sender, Arc::default())
            .unwrap();
        assert_eq!(receiver.recv_timeout(Duration::from_secs(2)).unwrap().id, 1);
    }

    #[cfg(feature = "testing")]
    #[test]
    fn manual_scheduler_does_not_deliver_already_due_deadlines() {
        for repeating in [false, true] {
            let executor = Executor::default();
            executor.use_manual_timers_for_test().unwrap();
            let (sender, receiver) = mpsc::sync_channel(1);
            let mut specification = spec(1, Duration::ZERO);
            specification.repeating = repeating;
            let handle = executor
                .prepare(1, specification, sender, Arc::default())
                .unwrap();
            assert!(matches!(
                receiver.recv_timeout(Duration::from_millis(20)),
                Err(mpsc::RecvTimeoutError::Timeout)
            ));
            handle.complete_for_test().unwrap();
            assert_eq!(receiver.try_recv().unwrap().id, 1);
            if repeating {
                handle.acknowledge();
                assert!(receiver.try_recv().is_err());
                handle.complete_for_test().unwrap();
                assert_eq!(receiver.try_recv().unwrap().id, 1);
            }
        }
    }

    #[cfg(feature = "testing")]
    #[test]
    fn explicit_completion_backpressure_preserves_timer_for_retry() {
        for repeating in [false, true] {
            let executor = Executor::default();
            let (sender, receiver) = mpsc::sync_channel(1);
            let mut specification = spec(1, Duration::from_secs(86_400));
            specification.repeating = repeating;
            sender
                .try_send(spec(99, Duration::ZERO).completion)
                .unwrap();
            let handle = executor
                .prepare(1, specification, sender, Arc::default())
                .unwrap();
            let deadline = *handle.0.deadline.lock().unwrap();
            assert!(handle.complete_for_test().is_err());
            assert_eq!(*handle.0.deadline.lock().unwrap(), deadline);
            assert_eq!(receiver.try_recv().unwrap().id, 99);
            handle.complete_for_test().unwrap();
            assert_eq!(receiver.try_recv().unwrap().id, 1);
            assert!(handle.complete_for_test().is_err());
            assert!(receiver.try_recv().is_err());
            if repeating {
                handle.acknowledge();
                handle.complete_for_test().unwrap();
                assert_eq!(receiver.try_recv().unwrap().id, 1);
            }
        }
    }

    #[cfg(feature = "testing")]
    #[test]
    fn explicit_completion_rejects_dormant_or_disconnected_timer() {
        let executor = Executor::default();
        let (sender, receiver) = mpsc::sync_channel(1);
        let specification = spec(1, Duration::from_secs(86_400));
        specification.started.store(false, Ordering::Release);
        let handle = executor
            .prepare(1, specification, sender, Arc::default())
            .unwrap();
        assert!(handle.complete_for_test().is_err());
        assert!(receiver.try_recv().is_err());
        handle.set_deadline(Duration::from_secs(86_400));
        handle.0.spec.started.store(true, Ordering::Release);
        drop(receiver);
        let deadline = *handle.0.deadline.lock().unwrap();
        assert!(handle.complete_for_test().is_err());
        assert_eq!(*handle.0.deadline.lock().unwrap(), deadline);
    }

    #[test]
    fn finished_timer_handle_releases_delivery_resources() {
        for completes in [false, true] {
            let executor = Executor::default();
            let wake = Arc::new(wake::Slot::default());
            let weak = Arc::downgrade(&wake);
            let (sender, receiver) = mpsc::sync_channel(1);
            let handle = executor
                .prepare(
                    1,
                    spec(
                        1,
                        if completes {
                            Duration::ZERO
                        } else {
                            Duration::from_secs(3600)
                        },
                    ),
                    sender,
                    wake,
                )
                .unwrap();
            if completes {
                assert_eq!(receiver.recv_timeout(Duration::from_secs(2)).unwrap().id, 1);
            }
            drop(executor); // Join the scheduler while keeping the task handle.
            assert!(handle.is_finished());
            assert!(
                weak.upgrade().is_none(),
                "finished timer retained actor wake slot"
            );
            assert!(
                matches!(
                    receiver.recv_timeout(Duration::from_secs(2)),
                    Err(mpsc::RecvTimeoutError::Disconnected)
                ),
                "finished timer retained completion sender"
            );
            drop(handle);
        }
    }

    #[test]
    fn notification_can_observe_and_drop_its_finished_timer_handle() {
        let executor = Executor::default();
        let owner: Arc<Mutex<Option<Handle>>> = Arc::default();
        let callback_owner = owner.clone();
        let wake = Arc::new(wake::Slot::default());
        let (observed, observation) = mpsc::channel();
        wake.set(Arc::new(move || {
            let handle = callback_owner.lock().unwrap().take().unwrap();
            observed.send(handle.is_finished()).unwrap();
            drop(handle);
        }));
        let (sender, receiver) = mpsc::sync_channel(1);
        let handle = executor
            .prepare(1, spec(1, Duration::from_secs(3600)), sender, wake)
            .unwrap();
        *owner.lock().unwrap() = Some(handle);
        {
            let guard = owner.lock().unwrap();
            let handle = guard.as_ref().unwrap();
            handle.set_deadline(Duration::ZERO);
            handle.unpark();
        }
        assert_eq!(receiver.recv_timeout(Duration::from_secs(2)).unwrap().id, 1);
        assert!(observation.recv_timeout(Duration::from_secs(2)).unwrap());
        drop(executor);
        assert!(owner.lock().unwrap().is_none());
        assert!(matches!(
            receiver.recv_timeout(Duration::from_secs(2)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
    }

    #[test]
    fn retired_handles_do_not_rearm_or_wake_shared_scheduler() {
        for repeating in [false, true] {
            let executor = Executor::default();
            let (sender, receiver) = mpsc::sync_channel(1);
            let mut request = spec(1, Duration::ZERO);
            request.repeating = repeating;
            let cancelled = request.cancelled.clone();
            let handle = executor
                .prepare(1, request, sender, Arc::default())
                .unwrap();
            receiver.recv_timeout(Duration::from_secs(2)).unwrap();
            if repeating {
                cancelled.store(true, Ordering::Release);
                handle.unpark();
            }
            let until = Instant::now() + Duration::from_secs(2);
            while !handle.is_finished() {
                assert!(Instant::now() < until);
                thread::yield_now();
            }
            let shared = handle.0.shared.upgrade().unwrap();
            let before = *handle.0.deadline.lock().unwrap();
            let (entered, barrier) = mpsc::channel();
            let (release, released) = mpsc::channel();
            let released = Mutex::new(released);
            let wake = Arc::new(wake::Slot::default());
            wake.set(Arc::new(move || {
                entered.send(()).unwrap();
                released
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(2))
                    .unwrap();
            }));
            let (sender, _receiver) = mpsc::sync_channel(1);
            let blocker = executor
                .prepare(2, spec(2, Duration::ZERO), sender, wake)
                .unwrap();
            barrier.recv_timeout(Duration::from_secs(2)).unwrap();
            shared.changed.store(false, Ordering::Release);
            handle.acknowledge();
            handle.set_deadline(Duration::from_secs(3600));
            for _ in 0..1000 {
                handle.unpark();
            }
            let changed = shared.changed.swap(false, Ordering::AcqRel);
            let deadline = *handle.0.deadline.lock().unwrap();
            drop(handle);
            let dropped = shared.changed.load(Ordering::Acquire);
            release.send(()).unwrap();
            drop(blocker);
            drop(executor);
            assert_eq!(deadline, before, "retired handle rewrote its deadline");
            assert!(!changed, "retired handle woke shared scheduler");
            assert!(!dropped, "dropping retired handle woke shared scheduler");
        }
    }

    #[test]
    fn shared_scheduler_isolates_local_generations_backpressure_and_removal() {
        let pool = Arc::new(Pool::default());
        let mut first = Executor::default();
        let mut second = Executor::default();
        first.use_pool(pool.clone());
        second.use_pool(pool);
        let (blocked, blocked_receiver) = mpsc::sync_channel(0);
        let (healthy, healthy_receiver) = mpsc::sync_channel(1);
        let (notifier, notified) = mpsc::channel();
        let wake = Arc::new(wake::Slot::default());
        wake.set(Arc::new(move || {
            let _ = notifier.send(());
        }));
        let a = first
            .prepare(1, spec(1, Duration::ZERO), blocked, wake)
            .unwrap();
        notified.recv_timeout(Duration::from_secs(2)).unwrap();
        // Same actor-local ID/generation must not overwrite the first timer.
        let b = second
            .prepare(
                2,
                spec(1, Duration::ZERO),
                healthy.clone(),
                Arc::new(wake::Slot::default()),
            )
            .unwrap();
        assert_eq!(a.worker_id(), b.worker_id());
        assert_eq!(
            healthy_receiver
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .id,
            1
        );
        assert!(!a.is_finished());
        assert!(blocked_receiver.try_recv().is_err());
        drop(a);
        drop(first);
        let c = second
            .prepare(
                2,
                spec(2, Duration::ZERO),
                healthy,
                Arc::new(wake::Slot::default()),
            )
            .unwrap();
        assert_eq!(b.worker_id(), c.worker_id());
        assert_eq!(
            healthy_receiver
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .id,
            2
        );
    }
    #[test]
    fn interval_has_one_pending_completion_and_rearms_only_on_acknowledgment() {
        let executor = Executor::default();
        let (sender, receiver) = mpsc::sync_channel(8);
        let mut request = spec(1, Duration::from_millis(5));
        request.repeating = true;
        let handle = executor
            .prepare(1, request, sender, Arc::new(wake::Slot::default()))
            .unwrap();
        receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        thread::sleep(Duration::from_millis(20));
        assert!(receiver.try_recv().is_err());
        assert!(!handle.is_finished());
        let before = Instant::now();
        handle.acknowledge();
        receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(before.elapsed() >= Duration::from_millis(5));
        assert!(receiver.try_recv().is_err());
        drop(handle); // cancellation wakes a scheduler dormant on pending delivery
    }

    #[test]
    fn dormant_timer_deadline_begins_on_activation_and_never_fires_early() {
        let executor = Executor::default();
        let (sender, receiver) = mpsc::sync_channel(1);
        let request = spec(1, Duration::from_millis(20));
        request.started.store(false, Ordering::Release);
        let timer = executor
            .prepare(1, request, sender, Arc::new(wake::Slot::default()))
            .unwrap();
        assert!(receiver.try_recv().is_err());
        let activated = Instant::now();
        timer.set_deadline(Duration::from_millis(20));
        let due = timer.0.deadline.lock().unwrap().unwrap();
        assert!(due >= activated + Duration::from_millis(20));
        assert!(due <= Instant::now() + Duration::from_millis(20));
        timer.0.spec.started.store(true, Ordering::Release);
        timer.unpark();
        receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(activated.elapsed() >= Duration::from_millis(20));
        drop(timer);
    }
    #[test]
    fn earlier_timer_wakes_shared_scheduler_and_does_not_wait_for_old_deadline() {
        let executor = Executor::default();
        let (sender, receiver) = mpsc::sync_channel(2);
        let wake = Arc::new(wake::Slot::default());
        let long = executor
            .prepare(
                1,
                spec(1, Duration::from_secs(3600)),
                sender.clone(),
                wake.clone(),
            )
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while long.0.deadline.lock().unwrap().is_none() {
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
        let short = executor
            .prepare(1, spec(2, Duration::ZERO), sender, wake)
            .unwrap();
        assert_eq!(long.worker_id(), short.worker_id());
        assert_eq!(receiver.recv_timeout(Duration::from_secs(2)).unwrap().id, 2);
        assert!(!long.is_finished());
        drop(short);
        drop(long);
    }
    #[test]
    fn shutdown_from_timer_callback_skips_later_due_jobs() {
        let executor = Executor::default();
        let owner = Arc::new(Mutex::new(None::<Executor>));
        let callback_owner = owner.clone();
        let (entered, enter) = mpsc::channel();
        let (release, released) = mpsc::channel();
        let released = Mutex::new(released);
        let barrier = Arc::new(wake::Slot::default());
        barrier.set(Arc::new(move || {
            entered.send(()).unwrap();
            released
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(2))
                .unwrap();
        }));
        let (sender, receiver) = mpsc::sync_channel(3);
        let first = executor
            .prepare(1, spec(1, Duration::ZERO), sender.clone(), barrier)
            .unwrap();
        enter.recv_timeout(Duration::from_secs(2)).unwrap();
        let shutdown = Arc::new(wake::Slot::default());
        shutdown.set(Arc::new(move || {
            drop(callback_owner.lock().unwrap().take());
        }));
        let second = executor
            .prepare(1, spec(2, Duration::ZERO), sender.clone(), shutdown)
            .unwrap();
        let third = executor
            .prepare(
                1,
                spec(3, Duration::ZERO),
                sender,
                Arc::new(wake::Slot::default()),
            )
            .unwrap();
        *owner.lock().unwrap() = Some(executor);
        release.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !first.is_finished() {
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
        while !third.is_finished() {
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
        assert_eq!(receiver.recv_timeout(Duration::from_secs(2)).unwrap().id, 1);
        assert_eq!(receiver.recv_timeout(Duration::from_secs(2)).unwrap().id, 2);
        assert!(
            receiver.try_recv().is_err(),
            "delivered timer after stop was published"
        );
        drop((first, second, third));
    }
    #[test]
    fn final_scheduler_can_drop_inside_worker_notification() {
        let owner = Arc::new(Mutex::new(None::<Executor>));
        let callback_owner = owner.clone();
        let (tx, rx) = mpsc::channel();
        let wake = Arc::new(wake::Slot::default());
        wake.set(Arc::new(move || {
            drop(callback_owner.lock().unwrap().take());
            tx.send(()).unwrap();
        }));
        let executor = Executor::default();
        let request = spec(1, Duration::ZERO);
        request.started.store(false, Ordering::Release);
        let (sender, receiver) = mpsc::sync_channel(1);
        let handle = executor.prepare(1, request, sender, wake).unwrap();
        let shared = Arc::downgrade(
            &executor
                .pool
                .worker
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .shared,
        );
        *owner.lock().unwrap() = Some(executor);
        handle.set_deadline(Duration::ZERO);
        handle.0.spec.started.store(true, Ordering::Release);
        handle.unpark();
        rx.recv_timeout(Duration::from_secs(2))
            .expect("scheduler self-drop panicked");
        assert_eq!(receiver.recv_timeout(Duration::from_secs(2)).unwrap().id, 1);
        let deadline = Instant::now() + Duration::from_secs(2);
        while shared.upgrade().is_some() {
            assert!(
                Instant::now() < deadline,
                "scheduler retained after self-shutdown"
            );
            thread::yield_now();
        }
        assert!(handle.is_finished());
        drop(handle);
    }
    #[test]
    fn blocked_timer_notifies_once_then_again_on_successful_delivery() {
        let executor = Executor::default();
        let (sender, receiver) = mpsc::sync_channel(0);
        let (tx, rx) = mpsc::channel();
        let wake = Arc::new(wake::Slot::default());
        wake.set(Arc::new(move || {
            let _ = tx.send(());
        }));
        let mut request = spec(1, Duration::ZERO);
        request.repeating = true;
        let timer = executor.prepare(1, request, sender, wake).unwrap();
        for _ in 0..2 {
            rx.recv_timeout(Duration::from_secs(2)).unwrap();
            // Cover several authored 10 ms backpressure retry intervals.
            assert!(
                rx.recv_timeout(Duration::from_millis(50)).is_err(),
                "unchanged timer backpressure repeatedly woke the GUI"
            );
            assert_eq!(receiver.recv_timeout(Duration::from_secs(2)).unwrap().id, 1);
            rx.recv_timeout(Duration::from_secs(2)).unwrap();
            timer.acknowledge();
        }
        drop(timer);
    }
    #[test]
    fn blocked_completion_does_not_prevent_timer_cancellation() {
        let executor = Executor::default();
        let (sender, _receiver) = mpsc::sync_channel(0);
        let wake = Arc::new(wake::Slot::default());
        let (tx, rx) = mpsc::channel();
        wake.set(Arc::new(move || {
            let _ = tx.send(());
        }));
        let timer = executor
            .prepare(1, spec(1, Duration::ZERO), sender, wake)
            .unwrap();
        rx.recv_timeout(Duration::from_secs(2)).unwrap();
        drop(timer); // cancellation waits for acknowledgement without draining completion
        assert!(
            executor
                .pool
                .worker
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .shared
                .jobs
                .lock()
                .unwrap()
                .is_empty()
        );
    }
}
