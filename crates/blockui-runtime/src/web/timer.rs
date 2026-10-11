//! Single-thread browser scheduler. Polling publishes owned completion messages only.
use crate::{TimerCompletion, wake};
use std::{
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};
use web_time::Instant;
pub(crate) struct Spec {
    pub completion: TimerCompletion,
    pub duration: Duration,
    pub repeating: bool,
    pub cancelled: Arc<AtomicBool>,
    pub started: Arc<AtomicBool>,
}
struct Job {
    spec: Spec,
    sender: mpsc::SyncSender<TimerCompletion>,
    deadline: Mutex<Option<Instant>>,
    done: AtomicBool,
}
pub(crate) struct Handle(Arc<Job>);
impl Handle {
    pub(crate) fn set_deadline(&self, duration: Duration) {
        *self.0.deadline.lock().unwrap() = Instant::now().checked_add(duration);
    }
    pub(crate) fn acknowledge(&self) {
        if self.0.spec.repeating {
            self.set_deadline(self.0.spec.duration);
        }
    }
    pub(crate) fn unpark(&self) {}
    pub(crate) fn is_finished(&self) -> bool {
        self.0.done.load(Ordering::Relaxed)
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        self.0.spec.cancelled.store(true, Ordering::Relaxed);
    }
}
#[derive(Default)]
pub(crate) struct Pool {
    jobs: Mutex<Vec<Weak<Job>>>,
}
#[derive(Default)]
pub(crate) struct Executor {
    pool: Arc<Pool>,
}
impl Executor {
    pub(crate) fn use_pool(&mut self, pool: Arc<Pool>) {
        self.pool = pool;
    }
    pub(crate) fn prepare(
        &self,
        _: u64,
        spec: Spec,
        sender: mpsc::SyncSender<TimerCompletion>,
        _: Arc<wake::Slot>,
    ) -> Result<Handle, String> {
        let job = Arc::new(Job {
            spec,
            sender,
            deadline: Mutex::new(None),
            done: AtomicBool::new(false),
        });
        let mut jobs = self.pool.jobs.lock().unwrap();
        jobs.retain(|job| job.strong_count() != 0);
        jobs.push(Arc::downgrade(&job));
        Ok(Handle(job))
    }
    pub(crate) fn poll(&self) -> bool {
        let now = Instant::now();
        let mut delivered = false;
        self.pool.jobs.lock().unwrap().retain(|weak| {
            let Some(job) = weak.upgrade() else {
                return false;
            };
            if job.spec.cancelled.load(Ordering::Relaxed) || job.done.load(Ordering::Relaxed) {
                return false;
            }
            if !job.spec.started.load(Ordering::Relaxed) {
                return true;
            }
            let mut deadline = job.deadline.lock().unwrap();
            if deadline.is_some_and(|deadline| deadline <= now) {
                match job.sender.try_send(job.spec.completion.clone()) {
                    Ok(()) => {
                        delivered = true;
                        *deadline = None;
                        job.done.store(!job.spec.repeating, Ordering::Relaxed);
                    }
                    Err(mpsc::TrySendError::Full(_)) => {}
                    Err(mpsc::TrySendError::Disconnected(_)) => return false,
                }
            }
            true
        });
        delivered
    }
}
pub(crate) fn validate_deadline(duration: Duration) -> Result<(), String> {
    Instant::now()
        .checked_add(duration)
        .ok_or_else(|| "timer deadline out of range".into())
        .map(|_| ())
}
