//! Futures compiled into the stable host only. No plugin future crosses the ABI.
use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
    thread,
};

use crate::{AsyncMessage, Message, TimerCompletion};
use std::{
    collections::{BTreeMap, BTreeSet},
    pin::Pin,
    sync::{Condvar, Mutex, Weak, mpsc},
    time::Duration,
};
type HostFuture = Pin<Box<dyn Future<Output = AsyncMessage> + Send>>;
struct Delivery {
    wake: Arc<crate::wake::Slot>,
    sender: mpsc::SyncSender<TimerCompletion>,
}
struct Data {
    backpressure_notified: bool,
    delivery: Option<Delivery>,
    future: Option<HostFuture>,
    output: Option<AsyncMessage>,
}
struct Job {
    data: Mutex<Data>,
    scheduled: AtomicBool,
    cancelled: AtomicBool,
    done: Mutex<bool>,
    finished: Condvar,
    worker: thread::Thread,
    shared: Weak<Shared>,
    token: u64,
    id: u64,
    generation: u64,
    on_panic: Message,
}
impl Job {
    fn enqueue(&self) {
        // Serialize admission with retirement, including retained external wakers.
        let done = self.done.lock().unwrap();
        if *done {
            return;
        }
        if let Some(shared) = self.shared.upgrade() {
            self.scheduled.store(true, Ordering::Release);
            shared.ready.lock().unwrap().insert(self.token);
            self.worker.unpark();
        }
    }
}
struct JobWake(Weak<Job>);
impl Wake for JobWake {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        if let Some(job) = self.0.upgrade() {
            job.enqueue();
        }
    }
}
pub(crate) struct Handle(Arc<Job>);
impl Handle {
    pub(crate) fn is_finished(&self) -> bool {
        *self.0.done.lock().unwrap()
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn worker_id(&self) -> thread::ThreadId {
        self.0.worker.id()
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        self.0.cancelled.store(true, Ordering::Release);
        self.0.enqueue();
        if thread::current().id() == self.0.worker.id() {
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
    jobs: Mutex<BTreeMap<u64, Arc<Job>>>,
    ready: Mutex<BTreeSet<u64>>,
    stop: AtomicBool,
}
struct Worker {
    shared: Arc<Shared>,
    thread: thread::JoinHandle<()>,
}
#[derive(Default)]
struct Pool {
    worker: Mutex<Option<Worker>>,
    next: Mutex<u64>,
}
#[derive(Default)]
pub(crate) struct Executor {
    wake: Arc<crate::wake::Slot>,
    pool: Arc<Pool>,
}
impl Executor {
    pub(crate) fn with_wake(wake: Arc<crate::wake::Slot>) -> Self {
        Self {
            wake,
            pool: Arc::default(),
        }
    }
    pub(crate) fn sharing(&self, wake: Arc<crate::wake::Slot>) -> Self {
        Self {
            wake,
            pool: self.pool.clone(),
        }
    }
    pub(crate) fn set_wake(&self, wake: crate::wake::Callback) {
        self.wake.set(wake);
    }
    #[cfg(test)]
    pub(crate) fn spawn<F: Future<Output = Message> + Send + 'static>(
        &mut self,
        actor: u64,
        completion: TimerCompletion,
        future: F,
        sender: mpsc::SyncSender<TimerCompletion>,
        wake: Option<Arc<dyn Fn() + Send + Sync>>,
    ) -> Result<Handle, String> {
        self.spawn_message(
            actor,
            completion,
            async move { future.await.into() },
            sender,
            wake,
        )
    }
    pub(crate) fn spawn_message<F: Future<Output = AsyncMessage> + Send + 'static>(
        &mut self,
        actor: u64,
        completion: TimerCompletion,
        future: F,
        sender: mpsc::SyncSender<TimerCompletion>,
        wake: Option<Arc<dyn Fn() + Send + Sync>>,
    ) -> Result<Handle, String> {
        if let Some(wake) = wake {
            self.set_wake(wake);
        }
        let token = {
            let mut next = self.pool.next.lock().unwrap();
            let token = *next;
            *next = next
                .checked_add(1)
                .ok_or("host future identity exhausted")?;
            token
        };
        let mut pool_worker = self.pool.worker.lock().unwrap();
        if pool_worker.is_none() {
            let shared = Arc::new(Shared::default());
            let state = shared.clone();
            let thread = thread::Builder::new()
                .name(format!("actor-{actor}-futures"))
                .spawn(move || worker_loop(state))
                .map_err(|e| format!("could not start host future worker: {e}"))?;
            *pool_worker = Some(Worker { shared, thread });
        }
        let worker = pool_worker.as_ref().unwrap();
        let job = Arc::new(Job {
            data: Mutex::new(Data {
                backpressure_notified: false,
                future: Some(Box::pin(future)),
                output: None,
                delivery: Some(Delivery {
                    sender,
                    wake: self.wake.clone(),
                }),
            }),
            scheduled: AtomicBool::new(true),
            cancelled: AtomicBool::new(false),
            done: Mutex::new(false),
            finished: Condvar::new(),
            worker: worker.thread.thread().clone(),
            shared: Arc::downgrade(&worker.shared),
            token,
            id: completion.id,
            generation: completion.generation,
            on_panic: completion.message,
        });
        worker
            .shared
            .jobs
            .lock()
            .unwrap()
            .insert(token, job.clone());
        job.enqueue();
        Ok(Handle(job))
    }
}
impl Drop for Pool {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.get_mut().unwrap().take() {
            worker.shared.stop.store(true, Ordering::Release);
            worker.thread.thread().unpark();
            // A host wake callback may release the last owner on this worker.
            // Its stop signal is already published; joining itself would panic.
            if thread::current().id() != worker.thread.thread().id() {
                let _ = worker.thread.join();
            }
        }
    }
}
fn drop_future(data: &mut Data) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(data.future.take()))).is_err()
}
fn worker_loop(shared: Arc<Shared>) {
    loop {
        let stop = shared.stop.load(Ordering::Acquire);
        let jobs: Vec<_> = if stop {
            shared.jobs.lock().unwrap().values().cloned().collect()
        } else {
            let ready = std::mem::take(&mut *shared.ready.lock().unwrap());
            let jobs = shared.jobs.lock().unwrap();
            ready
                .iter()
                .filter_map(|token| jobs.get(token).cloned())
                .collect()
        };
        let mut retry = false;
        for job in jobs {
            let mut data = job.data.lock().unwrap();
            let mut notify = false;
            let mut done =
                shared.stop.load(Ordering::Acquire) || job.cancelled.load(Ordering::Acquire);
            if done {
                drop_future(&mut data);
            } else if data.future.is_some() && job.scheduled.swap(false, Ordering::AcqRel) {
                let waker = Waker::from(Arc::new(JobWake(Arc::downgrade(&job))));
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    data.future
                        .as_mut()
                        .unwrap()
                        .as_mut()
                        .poll(&mut Context::from_waker(&waker))
                }));
                match result {
                    Ok(Poll::Pending) => {}
                    Ok(Poll::Ready(message)) => {
                        let panicked = drop_future(&mut data);
                        data.output = Some(if panicked {
                            job.on_panic.into()
                        } else {
                            message
                        });
                    }
                    Err(_) => {
                        drop_future(&mut data);
                        data.output = Some(job.on_panic.into());
                    }
                }
            }
            // Polling and destroying a host future can re-enter task ownership
            // and release the last executor, or cancel this job. Recheck after
            // that user code before publishing a completion.
            if !done
                && (shared.stop.load(Ordering::Acquire) || job.cancelled.load(Ordering::Acquire))
            {
                done = true;
                drop_future(&mut data);
            }
            if !done && let Some(message) = data.output.as_ref() {
                match data
                    .delivery
                    .as_ref()
                    .unwrap()
                    .sender
                    .try_send(TimerCompletion {
                        id: job.id,
                        generation: job.generation,
                        message: message.message,
                        text: message.text.clone(),
                        network: message.network.clone(),
                    }) {
                    Ok(()) => {
                        done = true;
                        notify = true;
                    }
                    Err(mpsc::TrySendError::Disconnected(_)) => done = true,
                    Err(mpsc::TrySendError::Full(_)) => {
                        retry = true;
                        shared.ready.lock().unwrap().insert(job.token);
                        notify = !data.backpressure_notified;
                        data.backpressure_notified = true;
                    }
                }
            }
            let wake = notify.then(|| data.delivery.as_ref().unwrap().wake.clone());
            let retired_delivery = if done {
                data.output = None;
                data.delivery.take()
            } else {
                None
            };
            drop(data);
            if done {
                shared.jobs.lock().unwrap().remove(&job.token);
                {
                    let mut finished = job.done.lock().unwrap();
                    *finished = true;
                    // A synchronous wake during the final poll may already be
                    // queued. No future enqueue can pass the same finish lock.
                    shared.ready.lock().unwrap().remove(&job.token);
                }
                job.finished.notify_all();
            }
            // Releasing the final callback owner can run host destructors.
            // Retire delivery state outside locks and after publishing completion.
            drop(retired_delivery);
            // Host callbacks can re-enter stable-host task APIs. Never notify
            // under the task data lock, and only signal changed delivery state.
            if let Some(wake) = wake {
                wake.notify();
            }
        }
        if stop {
            return;
        }
        // A notification callback may consume the shutdown unpark token while
        // waiting internally. Recheck the durable stop flag before parking.
        if shared.stop.load(Ordering::Acquire) {
            continue;
        }
        if retry {
            thread::park_timeout(Duration::from_millis(10));
        } else if shared.ready.lock().unwrap().is_empty() {
            // A callback may wait internally and consume the worker's unpark
            // token. The ready registry is authoritative before sleeping.
            thread::park();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn completion(generation: u64) -> TimerCompletion {
        TimerCompletion {
            text: None,
            network: None,
            id: generation,
            generation,
            message: Message {
                action: 7,
                value: -1,
            },
        }
    }
    #[test]
    fn text_completion_parks_without_copy_and_cancellation_releases_payload() {
        for cancel in [false, true] {
            let mut executor = Executor::default();
            let (sender, receiver) = mpsc::sync_channel(1);
            sender.send(completion(90)).unwrap();
            let text: Arc<str> = Arc::from("网络响应\n🦀");
            let weak = Arc::downgrade(&text);
            let message = AsyncMessage {
                message: Message {
                    action: 7,
                    value: 0,
                },
                text: Some(text.clone()),
                network: None,
            };
            let (notified, wake) = mpsc::sync_channel(1);
            let handle = executor
                .spawn_message(
                    1,
                    completion(1),
                    async move { message },
                    sender,
                    Some(Arc::new(move || {
                        let _ = notified.try_send(());
                    })),
                )
                .unwrap();
            wake.recv_timeout(Duration::from_secs(2)).unwrap();
            assert!(!handle.is_finished()); // Scalar filler still occupies the sink.
            if cancel {
                drop(text);
                drop(handle); // Worker-side retirement must drop the parked output.
                assert!(weak.upgrade().is_none());
                assert_eq!(
                    receiver
                        .recv_timeout(Duration::from_secs(2))
                        .unwrap()
                        .generation,
                    90
                );
                assert!(receiver.try_recv().is_err());
            } else {
                assert_eq!(
                    receiver
                        .recv_timeout(Duration::from_secs(2))
                        .unwrap()
                        .generation,
                    90
                );
                let delivered = receiver.recv_timeout(Duration::from_secs(2)).unwrap();
                assert_eq!(delivered.message.action, 7);
                let delivered_text = delivered.text.unwrap();
                assert!(Arc::ptr_eq(&text, &delivered_text));
                assert_eq!(delivered_text.as_ref(), "网络响应\n🦀");
                drop(handle);
                drop(delivered_text);
                drop(text);
                assert!(weak.upgrade().is_none());
                assert!(receiver.try_recv().is_err());
            }
        }
    }

    #[test]
    fn shutdown_from_future_poll_or_drop_suppresses_its_completion() {
        struct ShutdownFuture {
            ready: Arc<AtomicBool>,
            initial: Option<mpsc::Sender<Waker>>,
            owner: Arc<Mutex<Option<Executor>>>,
            shutdown_in_drop: bool,
        }
        impl Future for ShutdownFuture {
            type Output = Message;
            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Message> {
                if let Some(sender) = self.initial.take() {
                    sender.send(cx.waker().clone()).unwrap();
                }
                if !self.ready.load(Ordering::Acquire) {
                    return Poll::Pending;
                }
                if !self.shutdown_in_drop {
                    drop(self.owner.lock().unwrap().take());
                }
                Poll::Ready(Message {
                    action: 7,
                    value: 42,
                })
            }
        }
        impl Drop for ShutdownFuture {
            fn drop(&mut self) {
                if self.shutdown_in_drop {
                    drop(self.owner.lock().unwrap().take());
                }
            }
        }
        for shutdown_in_drop in [false, true] {
            let mut executor = Executor::default();
            let owner = Arc::new(Mutex::new(None));
            let ready = Arc::new(AtomicBool::new(false));
            let (initial, polled) = mpsc::channel();
            let (sender, receiver) = mpsc::sync_channel(1);
            let handle = executor
                .spawn(
                    1,
                    completion(1),
                    ShutdownFuture {
                        ready: ready.clone(),
                        initial: Some(initial),
                        owner: owner.clone(),
                        shutdown_in_drop,
                    },
                    sender,
                    None,
                )
                .unwrap();
            let waker = polled.recv_timeout(Duration::from_secs(2)).unwrap();
            *owner.lock().unwrap() = Some(executor);
            ready.store(true, Ordering::Release);
            waker.wake();
            let until = std::time::Instant::now() + Duration::from_secs(2);
            while !handle.is_finished() {
                assert!(
                    std::time::Instant::now() < until,
                    "shutdown job did not finish"
                );
                thread::yield_now();
            }
            drop(handle);
            assert!(owner.lock().unwrap().is_none());
            assert!(
                matches!(
                    receiver.recv_timeout(Duration::from_secs(2)),
                    Err(mpsc::RecvTimeoutError::Disconnected)
                ),
                "completion delivered after shutdown from future (drop={shutdown_in_drop})"
            );
        }
    }

    #[test]
    fn cancellation_from_own_poll_suppresses_reply_without_stopping_shared_worker() {
        let mut executor = Executor::default();
        let owner: Arc<Mutex<Option<Handle>>> = Arc::default();
        let future_owner = owner.clone();
        let ready = Arc::new(AtomicBool::new(false));
        let future_ready = ready.clone();
        let (initial, polled) = mpsc::channel();
        let mut initial = Some(initial);
        let (sender, receiver) = mpsc::sync_channel(1);
        let handle = executor
            .spawn(
                1,
                completion(1),
                std::future::poll_fn(move |cx| {
                    if let Some(initial) = initial.take() {
                        initial.send(cx.waker().clone()).unwrap();
                    }
                    if !future_ready.load(Ordering::Acquire) {
                        return Poll::Pending;
                    }
                    drop(future_owner.lock().unwrap().take());
                    Poll::Ready(Message {
                        action: 7,
                        value: 42,
                    })
                }),
                sender,
                None,
            )
            .unwrap();
        let worker = handle.worker_id();
        let waker = polled.recv_timeout(Duration::from_secs(2)).unwrap();
        *owner.lock().unwrap() = Some(handle);
        ready.store(true, Ordering::Release);
        waker.wake();
        assert!(
            matches!(
                receiver.recv_timeout(Duration::from_secs(2)),
                Err(mpsc::RecvTimeoutError::Disconnected)
            ),
            "self-cancelled future published a completion"
        );
        assert!(owner.lock().unwrap().is_none());
        let (sender, receiver) = mpsc::sync_channel(1);
        let sibling = executor
            .spawn(
                2,
                completion(2),
                std::future::ready(Message {
                    action: 7,
                    value: 99,
                }),
                sender,
                None,
            )
            .unwrap();
        assert_eq!(sibling.worker_id(), worker);
        assert_eq!(
            receiver
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .message
                .value,
            99
        );
        drop(sibling);
    }

    #[test]
    fn finished_handle_does_not_retain_delivery_channel_or_actor_wake() {
        for completes in [false, true] {
            let mut executor = Executor::default();
            let wake = Arc::downgrade(&executor.wake);
            let (sender, receiver) = mpsc::sync_channel(1);
            let handle = executor
                .spawn(
                    1,
                    completion(1),
                    std::future::poll_fn(move |_| {
                        if completes {
                            Poll::Ready(Message {
                                action: 7,
                                value: 42,
                            })
                        } else {
                            Poll::Pending
                        }
                    }),
                    sender,
                    None,
                )
                .unwrap();
            if completes {
                assert_eq!(
                    receiver
                        .recv_timeout(Duration::from_secs(2))
                        .unwrap()
                        .message
                        .value,
                    42
                );
            }
            drop(executor); // Join worker; delivery/notification is no longer in flight.
            assert!(handle.is_finished());
            assert!(
                wake.upgrade().is_none(),
                "finished handle retained the actor wake slot"
            );
            assert!(
                matches!(
                    receiver.recv_timeout(Duration::from_secs(2)),
                    Err(mpsc::RecvTimeoutError::Disconnected)
                ),
                "finished handle retained its completion sender"
            );
            let data = handle.0.data.lock().unwrap();
            assert!(data.future.is_none());
            assert!(data.output.is_none());
            drop(data);
            drop(handle);
        }
    }

    #[test]
    fn stale_waker_and_finished_handle_drop_do_not_schedule_retired_jobs() {
        let mut executor = Executor::default();
        let (first_notified, first_notification) = mpsc::channel();
        executor.set_wake(Arc::new(move || {
            first_notified.send(()).unwrap();
        }));
        let (saved, waker) = mpsc::channel();
        let (sender, receiver) = mpsc::sync_channel(1);
        let mut saved = Some(saved);
        let handle = executor
            .spawn(
                1,
                completion(1),
                std::future::poll_fn(move |cx| {
                    if let Some(saved) = saved.take() {
                        saved.send(cx.waker().clone()).unwrap();
                    }
                    Poll::Ready(Message {
                        action: 7,
                        value: 42,
                    })
                }),
                sender,
                None,
            )
            .unwrap();
        let waker = waker.recv_timeout(Duration::from_secs(2)).unwrap();
        receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        first_notification
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
        let until = std::time::Instant::now() + Duration::from_secs(2);
        while !handle.is_finished() {
            assert!(std::time::Instant::now() < until);
            thread::yield_now();
        }
        let token = handle.0.token;
        let shared = handle.0.shared.upgrade().unwrap();
        let (entered, barrier) = mpsc::channel();
        let (release, released) = mpsc::channel();
        let released = Mutex::new(released);
        executor.set_wake(Arc::new(move || {
            entered.send(()).unwrap();
            released
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(2))
                .unwrap();
        }));
        let (sender, _receiver) = mpsc::sync_channel(1);
        let blocker = executor
            .spawn(
                2,
                completion(2),
                std::future::ready(Message {
                    action: 7,
                    value: 99,
                }),
                sender,
                None,
            )
            .unwrap();
        barrier.recv_timeout(Duration::from_secs(2)).unwrap();
        for _ in 0..1000 {
            waker.wake_by_ref();
        }
        let stale_wake = shared.ready.lock().unwrap().contains(&token);
        shared.ready.lock().unwrap().remove(&token);
        drop(handle);
        let stale_drop = shared.ready.lock().unwrap().contains(&token);
        release.send(()).unwrap();
        drop(blocker);
        drop(executor);
        assert!(!stale_wake, "stale Waker scheduled a retired job");
        assert!(!stale_drop, "finished Handle scheduled a retired job");
    }

    #[test]
    fn shutdown_survives_notification_consuming_its_unpark_token() {
        let mut executor = Executor::default();
        let owner: Arc<Mutex<Option<Executor>>> = Arc::default();
        let callback_owner = owner.clone();
        let (notified, notification) = mpsc::channel();
        executor.set_wake(Arc::new(move || {
            drop(callback_owner.lock().unwrap().take());
            // Pool::drop has unparked this worker. Consume that token exactly,
            // reproducing a callback that waits after publishing shutdown.
            thread::park();
            notified.send(()).unwrap();
        }));
        let ready = Arc::new(AtomicBool::new(false));
        let future_ready = ready.clone();
        let (initial, polled) = mpsc::channel();
        let mut initial = Some(initial);
        let (sender, receiver) = mpsc::sync_channel(1);
        let handle = executor
            .spawn(
                1,
                completion(1),
                std::future::poll_fn(move |cx| {
                    if let Some(initial) = initial.take() {
                        initial.send(cx.waker().clone()).unwrap();
                    }
                    if future_ready.load(Ordering::Acquire) {
                        Poll::Ready(Message {
                            action: 7,
                            value: 42,
                        })
                    } else {
                        Poll::Pending
                    }
                }),
                sender,
                None,
            )
            .unwrap();
        let shared = handle.0.shared.clone();
        let waker = polled.recv_timeout(Duration::from_secs(2)).unwrap();
        *owner.lock().unwrap() = Some(executor);
        ready.store(true, Ordering::Release);
        waker.wake();
        receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        notification.recv_timeout(Duration::from_secs(2)).unwrap();
        let until = std::time::Instant::now() + Duration::from_secs(2);
        while shared.upgrade().is_some() {
            assert!(
                std::time::Instant::now() < until,
                "worker parked after callback consumed shutdown wake"
            );
            thread::yield_now();
        }
        assert!(handle.is_finished());
        drop(handle);
    }

    #[test]
    fn wake_burst_resumes_one_of_many_pending_jobs_without_repolling_idle_jobs() {
        let mut executor = Executor::default();
        let (sender, receiver) = mpsc::sync_channel(1);
        let (polled_tx, polled_rx) = mpsc::channel();
        let ready = Arc::new(AtomicBool::new(false));
        let mut handles = Vec::new();
        let counts: Vec<_> = (0..64)
            .map(|_| Arc::new(std::sync::atomic::AtomicUsize::new(0)))
            .collect();
        for (index, count) in counts.iter().enumerate() {
            let count = count.clone();
            let ready = ready.clone();
            let mut initial = Some(polled_tx.clone());
            handles.push(
                executor
                    .spawn(
                        1,
                        completion(index as u64 + 1),
                        std::future::poll_fn(move |cx| {
                            count.fetch_add(1, Ordering::Relaxed);
                            if let Some(tx) = initial.take() {
                                tx.send((index, cx.waker().clone())).unwrap();
                            }
                            if index == 31 && ready.load(Ordering::Acquire) {
                                Poll::Ready(Message {
                                    action: 7,
                                    value: 42,
                                })
                            } else {
                                Poll::Pending
                            }
                        }),
                        sender.clone(),
                        None,
                    )
                    .unwrap(),
            );
        }
        let mut selected = None;
        for _ in 0..64 {
            let (index, waker) = polled_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            if index == 31 {
                selected = Some(waker);
            }
        }
        ready.store(true, Ordering::Release);
        let waker = selected.unwrap();
        for _ in 0..1000 {
            waker.wake_by_ref();
        }
        let completion = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(completion.id, 32);
        assert_eq!(completion.message.value, 42);
        for (index, count) in counts.iter().enumerate() {
            if index != 31 {
                assert_eq!(count.load(Ordering::Relaxed), 1);
            }
        }
        drop(handles); // Cancellation must enqueue idle jobs for worker-side destruction.
    }
    #[test]
    fn actor_executors_share_worker_with_independent_generations_queues_and_lifetimes() {
        let mut first = Executor::default();
        let mut second = first.sharing(Arc::default());
        let (a_tx, _a_rx) = mpsc::sync_channel(1);
        let (b_tx, b_rx) = mpsc::sync_channel(1);
        a_tx.send(completion(99)).unwrap(); // Actor A is backpressured.
        let a = first
            .spawn(
                1,
                completion(1),
                std::future::ready(Message {
                    action: 7,
                    value: 11,
                }),
                a_tx,
                None,
            )
            .unwrap();
        let b = second
            .spawn(
                2,
                completion(1),
                std::future::ready(Message {
                    action: 7,
                    value: 22,
                }),
                b_tx.clone(),
                None,
            )
            .unwrap();
        assert_eq!(a.worker_id(), b.worker_id());
        let received = b_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(received.generation, 1);
        assert_eq!(received.message.value, 22);
        drop(a); // Cancels only A, even though both actors used generation 1.
        drop(first); // B still owns the pool.
        let pending = second
            .spawn(
                2,
                completion(2),
                std::future::pending::<Message>(),
                b_tx.clone(),
                None,
            )
            .unwrap();
        assert_eq!(pending.worker_id(), b.worker_id());
        let next = second
            .spawn(
                2,
                completion(3),
                std::future::ready(Message {
                    action: 7,
                    value: 33,
                }),
                b_tx,
                None,
            )
            .unwrap();
        assert_eq!(
            b_rx.recv_timeout(Duration::from_secs(5))
                .unwrap()
                .message
                .value,
            33
        );
        drop(pending);
        drop(next);
        drop(b);
        drop(second);
    }
    #[test]
    fn pending_future_uses_later_installed_callback() {
        let mut executor = Executor::default();
        let (sender, receiver) = mpsc::sync_channel(1);
        let (pending_tx, pending_rx) = mpsc::channel();
        let gate = Arc::new(Mutex::new(None::<Waker>));
        let ready = Arc::new(AtomicBool::new(false));
        let future_gate = gate.clone();
        let future_ready = ready.clone();
        let mut pending_tx = Some(pending_tx);
        let handle = executor
            .spawn(
                1,
                completion(1),
                std::future::poll_fn(move |cx| {
                    if future_ready.load(Ordering::Acquire) {
                        return Poll::Ready(Message {
                            action: 7,
                            value: 42,
                        });
                    }
                    *future_gate.lock().unwrap() = Some(cx.waker().clone());
                    if let Some(tx) = pending_tx.take() {
                        tx.send(()).unwrap();
                    }
                    Poll::Pending
                }),
                sender,
                None,
            )
            .unwrap();
        pending_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let (notified_tx, notified_rx) = mpsc::channel();
        executor.set_wake(Arc::new(move || {
            notified_tx.send(()).unwrap();
        }));
        ready.store(true, Ordering::Release);
        gate.lock().unwrap().take().unwrap().wake();
        assert_eq!(
            receiver
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .message
                .value,
            42
        );
        notified_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        drop(handle);
    }
    #[test]
    fn executor_shutdown_drops_pending_work_and_releases_handles() {
        let mut executor = Executor::default();
        let (sender, _) = mpsc::sync_channel(1);
        let handle = executor
            .spawn(1, completion(1), std::future::pending(), sender, None)
            .unwrap();
        drop(executor); // joins the shared thread after dropping its registered futures
        assert!(handle.is_finished());
        drop(handle);
    }
    #[test]
    fn panicking_future_does_not_stop_other_jobs() {
        let mut executor = Executor::default();
        let (sender, receiver) = mpsc::sync_channel(2);
        let failed = executor
            .spawn(
                1,
                completion(1),
                async { panic!("future failure") },
                sender.clone(),
                None,
            )
            .unwrap();
        let healthy = executor
            .spawn(
                1,
                completion(2),
                std::future::ready(Message {
                    action: 7,
                    value: 42,
                }),
                sender,
                None,
            )
            .unwrap();
        let mut values = (0..2)
            .map(|_| {
                receiver
                    .recv_timeout(Duration::from_secs(2))
                    .unwrap()
                    .message
                    .value
            })
            .collect::<Vec<_>>();
        values.sort();
        assert_eq!(values, vec![-1, 42]);
        assert_eq!(failed.worker_id(), healthy.worker_id());
        drop(failed);
        drop(healthy);
    }
    #[test]
    fn synchronous_wake_is_not_lost_on_shared_worker() {
        let mut executor = Executor::default();
        let (sender, receiver) = mpsc::sync_channel(4);
        let mut first = true;
        let future = std::future::poll_fn(move |cx| {
            if first {
                first = false;
                cx.waker().wake_by_ref();
                Poll::Pending
            } else {
                Poll::Ready(Message {
                    action: 7,
                    value: 42,
                })
            }
        });
        let handle = executor
            .spawn(1, completion(1), future, sender, None)
            .unwrap();
        assert_eq!(
            receiver
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .message
                .value,
            42
        );
        drop(handle);
    }
    #[test]
    fn parked_future_notifies_once_and_delivers_after_backpressure() {
        let mut executor = Executor::default();
        let (sender, receiver) = mpsc::sync_channel(0);
        let (tx, rx) = mpsc::channel();
        executor.set_wake(Arc::new(move || {
            let _ = tx.send(());
        }));
        let handle = executor
            .spawn(
                1,
                completion(1),
                std::future::ready(Message {
                    action: 7,
                    value: 42,
                }),
                sender,
                None,
            )
            .unwrap();
        rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(
            rx.recv_timeout(Duration::from_millis(50)).is_err(),
            "unchanged future backpressure repeatedly woke the GUI"
        );
        assert_eq!(
            receiver
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .message
                .value,
            42
        );
        rx.recv_timeout(Duration::from_secs(2)).unwrap();
        drop(handle);
    }
    #[test]
    fn ready_job_added_during_waiting_notification_is_not_lost() {
        let mut executor = Executor::default();
        let (entered, enter) = mpsc::channel();
        let (release, released) = mpsc::channel();
        let released = Mutex::new(released);
        executor.set_wake(Arc::new(move || {
            entered.send(()).unwrap();
            released
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(2))
                .unwrap();
        }));
        let (sender, receiver) = mpsc::sync_channel(2);
        let first = executor
            .spawn(
                1,
                completion(1),
                std::future::ready(Message {
                    action: 7,
                    value: 1,
                }),
                sender.clone(),
                None,
            )
            .unwrap();
        enter.recv_timeout(Duration::from_secs(2)).unwrap();
        executor.set_wake(Arc::new(|| {}));
        let second = executor
            .spawn(
                1,
                completion(2),
                std::future::ready(Message {
                    action: 7,
                    value: 2,
                }),
                sender,
                None,
            )
            .unwrap();
        release.send(()).unwrap();
        assert_eq!(receiver.recv_timeout(Duration::from_secs(2)).unwrap().id, 1);
        assert_eq!(receiver.recv_timeout(Duration::from_secs(2)).unwrap().id, 2);
        drop((first, second));
    }
    #[test]
    fn shutdown_from_first_callback_does_not_poll_later_ready_jobs() {
        let owner = Arc::new(Mutex::new(None::<Executor>));
        let callback_owner = owner.clone();
        let mut executor = Executor::default();
        executor.set_wake(Arc::new(move || {
            drop(callback_owner.lock().unwrap().take());
        }));
        let ready = Arc::new(AtomicBool::new(false));
        let late = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let (polled, polls) = mpsc::channel();
        let (sender, receiver) = mpsc::sync_channel(2);
        let mut handles = Vec::new();
        for index in 0..2 {
            let ready = ready.clone();
            let late = late.clone();
            let polled = polled.clone();
            handles.push(
                executor
                    .spawn(
                        1,
                        completion(index + 1),
                        std::future::poll_fn(move |_| {
                            if ready.load(Ordering::Acquire) {
                                if index == 1 {
                                    late.fetch_add(1, Ordering::Relaxed);
                                }
                                Poll::Ready(Message {
                                    action: 7,
                                    value: index as i64,
                                })
                            } else {
                                polled.send(()).unwrap();
                                Poll::Pending
                            }
                        }),
                        sender.clone(),
                        None,
                    )
                    .unwrap(),
            );
        }
        for _ in 0..2 {
            polls.recv_timeout(Duration::from_secs(2)).unwrap();
        }
        let shared = executor
            .pool
            .worker
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .shared
            .clone();
        *owner.lock().unwrap() = Some(executor);
        // Admit both ready tokens atomically so the worker takes one ordered batch.
        {
            let mut queue = shared.ready.lock().unwrap();
            ready.store(true, Ordering::Release);
            for handle in &handles {
                handle.0.scheduled.store(true, Ordering::Release);
                queue.insert(handle.0.token);
            }
            handles[0].0.worker.unpark();
        }
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while !handles[1].is_finished() {
            assert!(std::time::Instant::now() < deadline);
            thread::yield_now();
        }
        assert_eq!(
            late.load(Ordering::Relaxed),
            0,
            "polled after stop was published"
        );
        assert_eq!(
            receiver
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .message
                .value,
            0
        );
        assert!(receiver.try_recv().is_err());
        assert!(shared.jobs.lock().unwrap().is_empty());
    }
    #[test]
    fn final_executor_can_drop_inside_worker_notification() {
        let owner = Arc::new(Mutex::new(None::<Executor>));
        let callback_owner = owner.clone();
        let (tx, rx) = mpsc::channel();
        let mut executor = Executor::default();
        executor.set_wake(Arc::new(move || {
            drop(callback_owner.lock().unwrap().take());
            tx.send(()).unwrap();
        }));
        let ready = Arc::new(AtomicBool::new(false));
        let gate = ready.clone();
        let (polled, poll_receiver) = mpsc::channel();
        let (sender, receiver) = mpsc::sync_channel(1);
        let handle = executor
            .spawn(
                1,
                completion(1),
                std::future::poll_fn(move |cx| {
                    if gate.load(Ordering::Acquire) {
                        Poll::Ready(Message {
                            action: 7,
                            value: 42,
                        })
                    } else {
                        polled.send(cx.waker().clone()).unwrap();
                        Poll::Pending
                    }
                }),
                sender,
                None,
            )
            .unwrap();
        let shared = handle.0.shared.clone();
        let waker = poll_receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        *owner.lock().unwrap() = Some(executor);
        ready.store(true, Ordering::Release);
        waker.wake();
        rx.recv_timeout(Duration::from_secs(2))
            .expect("executor self-drop panicked");
        assert_eq!(
            receiver
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .message
                .value,
            42
        );
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while shared.upgrade().is_some() {
            assert!(
                std::time::Instant::now() < deadline,
                "worker retained after self-shutdown"
            );
            thread::yield_now();
        }
        drop(handle);
    }
    #[test]
    fn host_notification_runs_without_holding_future_data_lock() {
        let mut executor = Executor::default();
        let (sender, receiver) = mpsc::sync_channel(1);
        let (polled_tx, polled_rx) = mpsc::channel();
        let (tx, rx) = mpsc::channel();
        let reference = Arc::new(Mutex::new(None::<Weak<Job>>));
        let access = reference.clone();
        executor.set_wake(Arc::new(move || {
            let job = access.lock().unwrap().as_ref().unwrap().upgrade().unwrap();
            tx.send(job.data.try_lock().is_ok()).unwrap();
        }));
        let ready = Arc::new(AtomicBool::new(false));
        let gate = ready.clone();
        let handle = executor
            .spawn(
                1,
                completion(1),
                std::future::poll_fn(move |cx| {
                    if gate.load(Ordering::Acquire) {
                        Poll::Ready(Message {
                            action: 7,
                            value: 42,
                        })
                    } else {
                        polled_tx.send(cx.waker().clone()).unwrap();
                        Poll::Pending
                    }
                }),
                sender,
                None,
            )
            .unwrap();
        *reference.lock().unwrap() = Some(Arc::downgrade(&handle.0));
        let waker = polled_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        ready.store(true, Ordering::Release);
        waker.wake();
        assert!(
            rx.recv_timeout(Duration::from_secs(2)).unwrap(),
            "wake callback ran under future data lock"
        );
        assert_eq!(
            receiver
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .message
                .value,
            42
        );
        drop(handle);
    }
    #[test]
    fn full_completion_queue_does_not_block_other_futures_or_cancellation() {
        let mut executor = Executor::default();
        let (sender, receiver) = mpsc::sync_channel(1);
        sender.send(completion(99)).unwrap();
        let first = executor
            .spawn(
                1,
                completion(1),
                std::future::ready(Message {
                    action: 7,
                    value: 1,
                }),
                sender.clone(),
                None,
            )
            .unwrap();
        let (polled_tx, polled_rx) = mpsc::channel();
        let mut tx = Some(polled_tx);
        let second = executor
            .spawn(
                1,
                completion(2),
                std::future::poll_fn(move |_| {
                    if let Some(tx) = tx.take() {
                        tx.send(thread::current().id()).unwrap();
                    }
                    Poll::Pending
                }),
                sender,
                None,
            )
            .unwrap();
        assert_eq!(
            polled_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
            first.worker_id()
        );
        assert_eq!(first.worker_id(), second.worker_id());
        drop(second);
        assert_eq!(receiver.recv().unwrap().generation, 99);
        assert_eq!(
            receiver
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .message
                .value,
            1
        );
        drop(first);
    }
    #[test]
    fn pending_future_drops_on_shared_worker() {
        struct Pending(mpsc::Sender<thread::ThreadId>);
        impl Future for Pending {
            type Output = Message;
            fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Message> {
                Poll::Pending
            }
        }
        impl Drop for Pending {
            fn drop(&mut self) {
                self.0.send(thread::current().id()).unwrap();
            }
        }
        let mut executor = Executor::default();
        let (sender, _) = mpsc::sync_channel(1);
        let (tx, rx) = mpsc::channel();
        let handle = executor
            .spawn(1, completion(1), Pending(tx), sender, None)
            .unwrap();
        let worker = handle.worker_id();
        drop(handle);
        assert_eq!(rx.recv().unwrap(), worker);
    }
}
