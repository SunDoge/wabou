//! A bounded host worker pool. Only data and host-owned File handles enter jobs.
use super::{Context, Message, TaskCommand};
#[cfg(target_arch = "wasm32")]
use crate::web_files::FileExt;
#[cfg(windows)]
use crate::windows_files::FileExt;
use blockui_abi::{ERROR, OK};
#[cfg(unix)]
use std::os::unix::fs::{FileExt, OpenOptionsExt};
use std::{
    collections::{BTreeSet, VecDeque},
    ffi::c_void,
    fs::File,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};
pub(super) type ReadResult = Result<Vec<u8>, String>;
#[derive(Clone)]
pub(super) struct Recipe {
    pub file: Arc<File>,
    pub offset: u64,
    pub limit: usize,
    pub write: Option<Arc<[u8]>>,
    pub save_path: Option<std::path::PathBuf>,
    pub create_file: bool,
    pub action: u64,
    pub policy: u32,
}
pub(super) struct Job {
    pub sender: Option<mpsc::SyncSender<Completion>>,
    pub id: u64,
    pub generation: u64,
    pub recipe: Recipe,
    pub cancelled: Arc<AtomicBool>,
    pub wake: Arc<crate::wake::Slot>,
    pub message: Message,
}
pub(super) struct Completion {
    pub id: u64,
    pub generation: u64,
    pub result: ReadResult,
    pub message: Message,
}
struct Pending {
    sender: mpsc::SyncSender<Completion>,
    completion: Completion,
    cancelled: Arc<AtomicBool>,
    wake: Arc<crate::wake::Slot>,
}
#[derive(Default)]
struct Queue {
    pending: Vec<Pending>,
    jobs: VecDeque<Job>,
    shutdown: bool,
    active_saves: BTreeSet<std::path::PathBuf>,
}
impl Queue {
    fn retire_cancelled(&mut self, retired: &mut Vec<Job>) {
        if !self
            .jobs
            .iter()
            .any(|job| job.cancelled.load(Ordering::Acquire))
        {
            return;
        }
        let jobs = std::mem::take(&mut self.jobs);
        self.jobs.reserve(jobs.len());
        for job in jobs {
            if job.cancelled.load(Ordering::Acquire) {
                retired.push(job);
            } else {
                self.jobs.push_back(job);
            }
        }
    }
    fn pop_ready(&mut self, retired: &mut Vec<Job>) -> Option<Job> {
        self.retire_cancelled(retired);
        let index = self.jobs.iter().position(|job| {
            job.recipe
                .save_path
                .as_ref()
                .is_none_or(|path| !self.active_saves.contains(path))
        })?;
        let job = self.jobs.remove(index).unwrap();
        if let Some(path) = &job.recipe.save_path {
            self.active_saves.insert(path.clone());
        }
        Some(job)
    }
}
struct SavePermit {
    shared: Arc<Shared>,
    path: Option<std::path::PathBuf>,
}
impl Drop for SavePermit {
    fn drop(&mut self) {
        if let Some(path) = &self.path {
            self.shared.queue.lock().unwrap().active_saves.remove(path);
            self.shared.available.notify_all();
        }
    }
}
struct Shared {
    stopped: AtomicBool,
    queue: Mutex<Queue>,
    available: Condvar,
}
pub(super) struct Pool {
    shared: Arc<Shared>,
    workers: Mutex<Vec<thread::JoinHandle<()>>>,
}
pub(super) struct Executor {
    pool: Arc<Pool>,
    sender: mpsc::SyncSender<Completion>,
}
impl std::ops::Deref for Executor {
    type Target = Pool;
    fn deref(&self) -> &Pool {
        &self.pool
    }
}
impl Default for Pool {
    fn default() -> Self {
        Self {
            shared: Arc::new(Shared {
                stopped: AtomicBool::new(false),
                queue: Mutex::new(Queue::default()),
                available: Condvar::new(),
            }),
            workers: Mutex::new(Vec::new()),
        }
    }
}
impl Executor {
    pub fn use_pool(&mut self, pool: Arc<Pool>) {
        assert_eq!(
            self.worker_count(),
            0,
            "attach shared pool before actor admission"
        );
        self.pool = pool;
    }
    pub fn worker_count(&self) -> usize {
        self.workers.lock().unwrap().len()
    }
    pub fn queue_depth(&self) -> usize {
        self.shared.queue.lock().unwrap().jobs.len()
    }
    pub fn new(sender: mpsc::SyncSender<Completion>) -> Self {
        Self {
            pool: Arc::new(Pool::default()),
            sender,
        }
    }

    pub fn ensure_workers(&self) -> Result<(), String> {
        self.ensure_workers_with(|name, work| thread::Builder::new().name(name).spawn(work))
    }
    fn ensure_workers_with(
        &self,
        mut spawn: impl FnMut(
            String,
            Box<dyn FnOnce() + Send>,
        ) -> std::io::Result<thread::JoinHandle<()>>,
    ) -> Result<(), String> {
        let mut workers = self.workers.lock().unwrap();
        if !workers.is_empty() {
            return Ok(());
        }
        let mut prepared: Vec<thread::JoinHandle<()>> = Vec::new();
        for index in 0..2 {
            let shared = self.shared.clone();
            let work = Box::new(move || {
                loop {
                    // Move results out before notifying: host wake callbacks may
                    // re-enter the runtime and must never run under the queue lock.
                    let pending = {
                        let mut queue = shared.queue.lock().unwrap();
                        if queue.shutdown {
                            return;
                        }
                        std::mem::take(&mut queue.pending)
                    };
                    let mut retained = Vec::new();
                    for mut item in pending {
                        if shared.stopped.load(Ordering::Acquire)
                            || item.cancelled.load(Ordering::Acquire)
                        {
                            continue;
                        }
                        match item.sender.try_send(item.completion) {
                            Ok(()) => item.wake.notify(),
                            Err(mpsc::TrySendError::Full(value)) => {
                                item.completion = value;
                                // Initial parking already notified the host. An unchanged
                                // full sink must not produce a GUI wake on every retry.
                                retained.push(item);
                            }
                            Err(mpsc::TrySendError::Disconnected(_)) => {}
                        }
                    }
                    let mut retired = Vec::new();
                    let job = {
                        let mut queue = shared.queue.lock().unwrap();
                        if queue.shutdown {
                            return;
                        }
                        queue.pending.extend(retained);
                        if let Some(job) = queue.pop_ready(&mut retired) {
                            Some(job)
                        } else if !retired.is_empty() {
                            // Release canceled captures before parking; their drop
                            // may submit new work into this same pool.
                            None
                        } else {
                            if queue.pending.is_empty() {
                                drop(shared.available.wait(queue).unwrap());
                            } else {
                                drop(
                                    shared
                                        .available
                                        .wait_timeout(queue, Duration::from_millis(2))
                                        .unwrap(),
                                );
                            }
                            None
                        }
                    };
                    drop(retired);
                    let Some(job) = job else { continue };
                    let sender = job.sender.as_ref().expect("submitted job owns recipient");
                    let permit = SavePermit {
                        shared: shared.clone(),
                        path: job.recipe.save_path.clone(),
                    };
                    if job.cancelled.load(Ordering::Acquire) {
                        continue;
                    }
                    let mut bytes = vec![0; job.recipe.limit];
                    let result = if let Some(path) = &job.recipe.save_path {
                        let bytes = job.recipe.write.as_ref().expect("save owns bytes");
                        crate::save::publish_cancellable(
                            path,
                            bytes,
                            job.recipe.create_file,
                            || {
                                job.cancelled.load(Ordering::Acquire)
                                    || shared.stopped.load(Ordering::Acquire)
                            },
                        )
                        .map(|()| Vec::new())
                    } else if let Some(data) = &job.recipe.write {
                        job.recipe
                            .file
                            .write_all_at(data, job.recipe.offset)
                            .map(|()| Vec::new())
                            .map_err(|error| error.to_string())
                    } else {
                        job.recipe
                            .file
                            .read_at(&mut bytes, job.recipe.offset)
                            .map(|length| {
                                bytes.truncate(length);
                                // Parked completions retain only meaningful data, not
                                // a large caller-specified read limit after a short read.
                                if bytes.capacity() > length.saturating_mul(2).max(4096) {
                                    bytes.shrink_to_fit();
                                }
                                bytes
                            })
                            .map_err(|error| error.to_string())
                    };
                    drop(permit); // Release pathname before waiting for completion-channel space.
                    let completion = Completion {
                        id: job.id,
                        generation: job.generation,
                        result,
                        message: job.message,
                    };
                    if !job.cancelled.load(Ordering::Acquire)
                        && !shared.stopped.load(Ordering::Acquire)
                    {
                        match sender.try_send(completion) {
                            Ok(()) => job.wake.notify(),
                            Err(mpsc::TrySendError::Full(completion)) => {
                                job.wake.notify();
                                let mut queue = shared.queue.lock().unwrap();
                                if !queue.shutdown {
                                    queue.pending.push(Pending {
                                        sender: sender.clone(),
                                        completion,
                                        cancelled: job.cancelled,
                                        wake: job.wake,
                                    });
                                    shared.available.notify_one();
                                }
                            }
                            Err(mpsc::TrySendError::Disconnected(_)) => {}
                        }
                    }
                }
            });
            match spawn(format!("actor-file-read-{index}"), work) {
                Ok(worker) => prepared.push(worker),
                Err(error) => {
                    self.shared.queue.lock().unwrap().shutdown = true;
                    self.shared.available.notify_all();
                    for worker in prepared {
                        let _ = worker.join();
                    }
                    self.shared.queue.lock().unwrap().shutdown = false;
                    return Err(format!("could not start file-read pool: {error}"));
                }
            }
        }
        *workers = prepared;
        Ok(())
    }
    pub fn submit(&self, mut job: Job) {
        assert_eq!(
            self.worker_count(),
            2,
            "file workers prepared before turn commit"
        );
        let mut queue = self.shared.queue.lock().unwrap();
        let mut retired = Vec::new();
        queue.retire_cancelled(&mut retired);
        assert!(
            queue.jobs.len() < 64 * 64,
            "task admission bounds the executor queue"
        );
        job.sender = Some(self.sender.clone());
        queue.jobs.push_back(job);
        drop(queue);
        self.shared.available.notify_one();
        drop(retired);
    }
}
impl Drop for Pool {
    fn drop(&mut self) {
        self.shared.stopped.store(true, Ordering::Release);
        let mut queue = self.shared.queue.lock().unwrap();
        queue.shutdown = true;
        let jobs = std::mem::take(&mut queue.jobs);
        let pending = std::mem::take(&mut queue.pending);
        drop(queue);
        self.shared.available.notify_all();
        // Job wake captures are arbitrary host values; retire them unlocked.
        drop(jobs);
        drop(pending);
        // An in-flight filesystem syscall cannot be forcibly interrupted. Detach host
        // workers; they return after that syscall. No plugin code or actor borrow escapes.
    }
}
pub(super) unsafe extern "C" fn request(
    ctx: *mut c_void,
    id: u64,
    file: u64,
    offset: u64,
    limit: usize,
    action: u64,
    policy: u32,
) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    if turn.read_only
        || turn.journal.is_none()
        || id == 0
        || action == 0
        || policy > 2
        || limit > 1024 * 1024
        || turn.task_keys.len() >= 64
        || turn.task_keys.contains(&id)
        || turn.task_commands.len() >= 256
        || (turn.io_results.len() >= 64 && !turn.io_results.contains_key(&id))
    {
        return ERROR;
    }
    let Some(resource) = turn.files.get(&file) else {
        return ERROR;
    };
    let recipe = Recipe {
        file: resource.file.clone(),
        offset,
        limit,
        write: None,
        save_path: None,
        create_file: false,
        action,
        policy,
    };
    if !turn.reserve_task_generation() {
        return ERROR;
    }
    turn.task_keys.insert(id);
    turn.io_results.insert(id, None);
    turn.network_permits.remove(&id);
    turn.task_commands.push(TaskCommand::Read(id, recipe));
    OK
}
/// Copies plugin bytes during admission; no plugin pointer survives the call.
pub(super) unsafe extern "C" fn request_write(
    ctx: *mut c_void,
    id: u64,
    file: u64,
    offset: u64,
    bytes: blockui_abi::Bytes,
    action: u64,
    policy: u32,
) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    if policy > 1
        || bytes.len > 1024 * 1024
        || (bytes.len != 0 && bytes.ptr.is_null())
        || offset.checked_add(bytes.len as u64).is_none()
        || !turn
            .files
            .get(&file)
            .is_some_and(|resource| resource.access == 1)
    {
        return ERROR;
    }
    // Common admission reserves task identity and stages the host-owned recipe.
    if unsafe { request(ctx, id, file, offset, 0, action, policy) } != OK {
        return ERROR;
    }
    let data: Arc<[u8]> = if bytes.len == 0 {
        Arc::from([])
    } else {
        Arc::from(unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) })
    };
    let turn = unsafe { &mut *context.turn };
    let Some(TaskCommand::Read(_, recipe)) = turn.task_commands.last_mut() else {
        unreachable!()
    };
    recipe.write = Some(data);
    OK
}
pub(super) unsafe extern "C" fn request_save(
    ctx: *mut c_void,
    id: u64,
    file: u64,
    bytes: blockui_abi::Bytes,
    action: u64,
    policy: u32,
) -> i32 {
    if unsafe { request_write(ctx, id, file, 0, bytes, action, policy) } != OK {
        return ERROR;
    }
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    let path = turn.files[&file].path.clone();
    let Some(TaskCommand::Read(_, recipe)) = turn.task_commands.last_mut() else {
        unreachable!()
    };
    recipe.save_path = Some(path);
    OK
}
/// Create a missing destination after a successful turn; never overwrite an existing path.
pub(super) unsafe extern "C" fn request_create(
    ctx: *mut c_void,
    id: u64,
    path: blockui_abi::Bytes,
    bytes: blockui_abi::Bytes,
    action: u64,
    policy: u32,
) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    if turn.read_only
        || turn.journal.is_none()
        || id == 0
        || action == 0
        || policy > 1
        || turn.task_keys.len() >= 64
        || turn.task_keys.contains(&id)
        || turn.task_commands.len() >= 256
        || (turn.io_results.len() >= 64 && !turn.io_results.contains_key(&id))
        || path.len == 0
        || path.len > 4096
        || path.ptr.is_null()
        || bytes.len > 1024 * 1024
        || (bytes.len != 0 && bytes.ptr.is_null())
    {
        return ERROR;
    }
    let Ok(path) = (unsafe { crate::copy_bytes(path) }) else {
        return ERROR;
    };
    if path.ends_with('/') {
        return ERROR;
    }
    let path = std::path::Path::new(&path);
    let Some(name) = path.file_name() else {
        return ERROR;
    };
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(std::path::Path::new("."));
    let Ok(parent) = std::fs::canonicalize(parent) else {
        return ERROR;
    };
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_DIRECTORY | libc::O_NONBLOCK | libc::O_NOCTTY);
    let Ok(directory) = options.open(&parent) else {
        return ERROR;
    };
    if !directory.metadata().is_ok_and(|metadata| metadata.is_dir()) {
        return ERROR;
    }
    let data: Arc<[u8]> = if bytes.len == 0 {
        Arc::from([])
    } else {
        Arc::from(unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) })
    };
    if !turn.reserve_task_generation() {
        return ERROR;
    }
    let recipe = Recipe {
        file: Arc::new(directory),
        offset: 0,
        limit: 0,
        write: Some(data),
        save_path: Some(parent.join(name)),
        create_file: true,
        action,
        policy,
    };
    turn.task_keys.insert(id);
    turn.io_results.insert(id, None);
    turn.network_permits.remove(&id);
    turn.task_commands.push(TaskCommand::Read(id, recipe));
    OK
}

pub(super) unsafe extern "C" fn result(
    ctx: *mut c_void,
    id: u64,
    out: *mut u8,
    capacity: usize,
    status: *mut u32,
) -> isize {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &*context.turn };
    if turn.read_only || turn.journal.is_none() || status.is_null() {
        return ERROR as isize;
    }
    let Some(Some(result)) = turn.io_results.get(&id) else {
        unsafe {
            *status = 0;
        }
        return 0;
    };
    let bytes: &[u8] = match &**result {
        Ok(bytes) => {
            unsafe {
                *status = 1;
            }
            bytes
        }
        Err(error) => {
            unsafe {
                *status = 2;
            }
            error.as_bytes()
        }
    };
    if out.is_null() {
        return bytes.len() as isize;
    }
    if capacity < bytes.len() {
        return ERROR as isize;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len());
    }
    bytes.len() as isize
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::atomic::AtomicUsize, time::Instant};

    fn slot(callback: crate::wake::Callback) -> Arc<crate::wake::Slot> {
        let slot = Arc::new(crate::wake::Slot::default());
        slot.set(callback);
        slot
    }
    fn save_job(file: Arc<File>, path: std::path::PathBuf, id: u64) -> Job {
        Job {
            sender: None,
            id,
            generation: id,
            recipe: Recipe {
                file,
                offset: 0,
                limit: 0,
                write: Some(Arc::from(vec![id as u8])),
                save_path: Some(path),
                create_file: false,
                action: 1,
                policy: 0,
            },
            cancelled: Arc::new(AtomicBool::new(false)),
            wake: Arc::new(crate::wake::Slot::default()),
            message: Message {
                action: 1,
                value: id as i64,
            },
        }
    }
    #[test]
    fn cancelled_job_retirement_releases_captures_after_unlocking() {
        struct Probe {
            shared: Arc<Shared>,
            unlocked: Arc<AtomicBool>,
        }
        impl Drop for Probe {
            fn drop(&mut self) {
                self.unlocked
                    .store(self.shared.queue.try_lock().is_ok(), Ordering::Release);
            }
        }
        let pool = Pool::default();
        let unlocked = Arc::new(AtomicBool::new(false));
        let probe = Probe {
            shared: pool.shared.clone(),
            unlocked: unlocked.clone(),
        };
        let mut job = save_job(
            Arc::new(File::open("Cargo.toml").unwrap()),
            "unused".into(),
            1,
        );
        job.cancelled.store(true, Ordering::Release);
        job.wake = slot(Arc::new(move || {
            let _ = &probe;
        }));
        let mut retired = Vec::new();
        {
            let mut queue = pool.shared.queue.lock().unwrap();
            queue.jobs.push_back(job);
            assert!(queue.pop_ready(&mut retired).is_none());
            assert_eq!(retired.len(), 1);
            assert!(!unlocked.load(Ordering::Acquire));
        }
        drop(retired);
        assert!(unlocked.load(Ordering::Acquire));
    }

    #[test]
    fn shutdown_retires_queued_wake_captures_outside_queue_lock() {
        struct Probe {
            shared: std::sync::Weak<Shared>,
            unlocked: Arc<AtomicBool>,
        }
        impl Drop for Probe {
            fn drop(&mut self) {
                let shared = self.shared.upgrade().unwrap();
                self.unlocked
                    .store(shared.queue.try_lock().is_ok(), Ordering::Release);
            }
        }
        let pool = Pool::default();
        let unlocked = Arc::new(AtomicBool::new(false));
        let probe = Probe {
            shared: Arc::downgrade(&pool.shared),
            unlocked: unlocked.clone(),
        };
        let mut job = save_job(
            Arc::new(File::open("Cargo.toml").unwrap()),
            "unused".into(),
            1,
        );
        job.wake = slot(Arc::new(move || {
            let _ = &probe;
        }));
        pool.shared.queue.lock().unwrap().jobs.push_back(job);
        drop(pool);
        assert!(
            unlocked.load(Ordering::Acquire),
            "queued wake capture dropped under I/O queue lock"
        );
    }

    #[test]
    fn save_queue_serializes_same_path_and_admits_independent_paths() {
        let file = Arc::new(File::open("Cargo.toml").unwrap());
        let a = std::path::PathBuf::from("a");
        let b = std::path::PathBuf::from("b");
        let mut queue = Queue::default();
        queue.jobs.push_back(save_job(file.clone(), a.clone(), 1));
        queue.jobs.push_back(save_job(file.clone(), a.clone(), 2));
        queue.jobs.push_back(save_job(file, b.clone(), 3));
        assert_eq!(queue.pop_ready(&mut Vec::new()).unwrap().id, 1);
        assert_eq!(queue.pop_ready(&mut Vec::new()).unwrap().id, 3);
        assert!(queue.pop_ready(&mut Vec::new()).is_none());
        assert_eq!(queue.jobs.len(), 1);
        queue.active_saves.remove(&a);
        assert_eq!(queue.pop_ready(&mut Vec::new()).unwrap().id, 2);
    }
    #[test]
    fn cancelled_save_releases_path_and_does_not_block_replacement_job() {
        let file = Arc::new(File::open("Cargo.toml").unwrap());
        let path = std::path::PathBuf::from("cancelled");
        let shared = Arc::new(Shared {
            stopped: AtomicBool::new(false),
            queue: Mutex::new(Queue::default()),
            available: Condvar::new(),
        });
        let queued = save_job(file.clone(), path.clone(), 1);
        queued.cancelled.store(true, Ordering::Release);
        let mut queue = shared.queue.lock().unwrap();
        queue.jobs.push_back(queued);
        queue
            .jobs
            .push_back(save_job(file.clone(), path.clone(), 2));
        let selected = queue.pop_ready(&mut Vec::new()).unwrap();
        assert_eq!(selected.id, 2);
        selected.cancelled.store(true, Ordering::Release); // cancellation after selection
        let permit = SavePermit {
            shared: shared.clone(),
            path: Some(path.clone()),
        };
        queue.jobs.push_back(save_job(file, path.clone(), 3));
        assert!(queue.pop_ready(&mut Vec::new()).is_none());
        drop(queue);
        drop(permit);
        assert_eq!(
            shared
                .queue
                .lock()
                .unwrap()
                .pop_ready(&mut Vec::new())
                .unwrap()
                .id,
            3
        );
    }
    #[test]
    fn shutdown_from_completion_callback_discards_later_parked_results() {
        let (sender, first_receiver) = mpsc::sync_channel(1);
        let (later_sender, later_receiver) = mpsc::sync_channel(1);
        let executor = Executor::new(sender.clone());
        let shared = Arc::downgrade(&executor.shared);
        let owner = Arc::new(Mutex::new(Some(executor)));
        let callback_owner = owner.clone();
        let wake = slot(Arc::new(move || {
            drop(callback_owner.lock().unwrap().take());
        }));
        // Both completed results are taken together by one worker. The first
        // successful-delivery callback shuts down the entire pool.
        {
            let owner = owner.lock().unwrap();
            let mut queue = owner.as_ref().unwrap().shared.queue.lock().unwrap();
            for (id, sender, wake) in [
                (1, sender, wake),
                (2, later_sender, Arc::new(crate::wake::Slot::default())),
            ] {
                queue.pending.push(Pending {
                    sender,
                    completion: Completion {
                        id,
                        generation: id,
                        result: Ok(vec![id as u8]),
                        message: Message {
                            action: 7,
                            value: id as i64,
                        },
                    },
                    cancelled: Arc::new(AtomicBool::new(false)),
                    wake,
                });
            }
        }
        owner
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .ensure_workers()
            .unwrap();
        assert_eq!(
            first_receiver
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .id,
            1
        );
        let deadline = Instant::now() + Duration::from_secs(2);
        while shared.upgrade().is_some() {
            assert!(Instant::now() < deadline, "I/O pool did not stop");
            thread::yield_now();
        }
        assert!(
            later_receiver.try_recv().is_err(),
            "delivered result after pool stop"
        );
    }
    #[test]
    fn cancelled_parked_generations_release_owned_data_before_id_reuse() {
        let (sender, receiver) = mpsc::sync_channel(0);
        let executor = Executor::new(sender);
        executor.ensure_workers().unwrap();
        let (notifications, notified) = mpsc::channel();
        for generation in 1..=64 {
            let mut job = save_job(
                Arc::new(File::open("Cargo.toml").unwrap()),
                "unused".into(),
                1,
            );
            job.generation = generation;
            job.recipe.save_path = None;
            job.recipe.write = None;
            job.recipe.limit = 1024 * 1024;
            let cancelled = job.cancelled.clone();
            let notifications = notifications.clone();
            job.wake = slot(Arc::new(move || {
                let _ = notifications.send(());
            }));
            executor.submit(job);
            notified.recv_timeout(Duration::from_secs(2)).unwrap();
            cancelled.store(true, Ordering::Release);
            let deadline = Instant::now() + Duration::from_secs(2);
            // The worker releases the entire pending record, including its Vec,
            // recipient and wake callback, when its cancellation Arc is released.
            while Arc::strong_count(&cancelled) != 1 {
                assert!(Instant::now() < deadline, "cancelled generation retained");
                thread::sleep(Duration::from_millis(2));
            }
            assert!(receiver.try_recv().is_err());
        }
        assert!(executor.shared.queue.lock().unwrap().pending.is_empty());
        assert_eq!(executor.queue_depth(), 0);
    }
    #[test]
    fn parked_completion_does_not_repeatedly_wake_the_gui() {
        let (sender, receiver) = mpsc::sync_channel(0);
        let executor = Executor::new(sender);
        executor.ensure_workers().unwrap();
        let (notifications, notified) = mpsc::channel();
        let mut job = save_job(
            Arc::new(File::open("Cargo.toml").unwrap()),
            "unused".into(),
            1,
        );
        job.recipe.save_path = None;
        job.recipe.write = None;
        job.recipe.limit = 16;
        job.wake = slot(Arc::new(move || {
            let _ = notifications.send(());
        }));
        executor.submit(job);
        notified.recv_timeout(Duration::from_secs(2)).unwrap();
        // Cover multiple 2 ms completion retry intervals while the sink is full.
        assert!(
            notified.recv_timeout(Duration::from_millis(40)).is_err(),
            "unchanged completion backpressure repeatedly woke the GUI"
        );
        assert_eq!(receiver.recv_timeout(Duration::from_secs(2)).unwrap().id, 1);
        notified.recv_timeout(Duration::from_secs(2)).unwrap();
    }
    #[test]
    fn shared_pool_isolates_recipients_backpressure_and_executor_removal() {
        let pool = Arc::new(Pool::default());
        let (blocked_sender, blocked_receiver) = mpsc::sync_channel(0);
        let (healthy_sender, healthy_receiver) = mpsc::sync_channel(1);
        let mut blocked = Executor::new(blocked_sender);
        let mut healthy = Executor::new(healthy_sender);
        blocked.use_pool(pool.clone());
        healthy.use_pool(pool.clone());
        blocked.ensure_workers().unwrap();
        healthy.ensure_workers().unwrap();
        assert_eq!(healthy.worker_count(), 2);
        let read = |id| {
            let mut job = save_job(
                Arc::new(File::open("Cargo.toml").unwrap()),
                "unused".into(),
                id,
            );
            job.recipe.save_path = None;
            job.recipe.write = None;
            job.recipe.limit = 16;
            job
        };
        let mut cancellations = Vec::new();
        for id in 1..=3 {
            let job = read(id);
            cancellations.push(job.cancelled.clone());
            blocked.submit(job);
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        while pool.shared.queue.lock().unwrap().pending.len() < 3 {
            assert!(Instant::now() < deadline, "blocked results were not parked");
            thread::sleep(Duration::from_millis(2));
        }
        // IDs are actor-local: both recipients deliberately use ID 1.
        healthy.submit(read(1));
        let completion = healthy_receiver
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
        assert_eq!(completion.id, 1);
        assert_eq!(completion.result.unwrap().len(), 16);
        assert!(blocked_receiver.try_recv().is_err());
        for cancelled in cancellations {
            cancelled.store(true, Ordering::Release);
        }
        drop(blocked);
        drop(blocked_receiver);
        healthy.submit(read(2));
        assert_eq!(
            healthy_receiver
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .id,
            2
        );
        let deadline = Instant::now() + Duration::from_secs(2);
        while !pool.shared.queue.lock().unwrap().pending.is_empty() {
            assert!(Instant::now() < deadline, "cancelled results retained");
            thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(healthy.worker_count(), 2);
    }
    #[test]
    fn full_completion_channel_does_not_stop_file_work() {
        let root =
            std::env::temp_dir().join(format!("identity-save-backpressure-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("document");
        std::fs::write(&path, b"original").unwrap();
        let file = Arc::new(File::open(&path).unwrap());
        let (sender, receiver) = mpsc::sync_channel(0);
        let executor = Executor::new(sender);
        executor.ensure_workers().unwrap();
        for id in 1..=3 {
            executor.submit(save_job(file.clone(), path.clone(), id));
        }
        // No receiver consumes results until all three writes have executed.
        let deadline = Instant::now() + Duration::from_secs(2);
        while std::fs::read(&path).unwrap() != [3] && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(2));
        }
        let contents = std::fs::read(&path).unwrap();
        if contents != [3] {
            drop(executor);
            std::fs::remove_dir_all(root).unwrap();
            panic!("full completion channel stopped file work: {contents:?}");
        }
        let mut completed = BTreeSet::new();
        for _ in 0..3 {
            let result = receiver.recv_timeout(Duration::from_secs(2)).unwrap();
            assert!(result.result.unwrap().is_empty());
            assert!(completed.insert(result.id));
        }
        assert_eq!(completed, BTreeSet::from([1, 2, 3]));
        drop(executor);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn concurrent_save_pool_leaves_latest_admitted_contents() {
        let root = std::env::temp_dir().join(format!("identity-save-order-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("document");
        std::fs::write(&path, b"original").unwrap();
        let file = Arc::new(File::open(&path).unwrap());
        let (sender, receiver) = mpsc::sync_channel(32);
        let executor = Executor::new(sender);
        executor.ensure_workers().unwrap();
        for id in 1..=32 {
            executor.submit(save_job(file.clone(), path.clone(), id));
        }
        let mut completed = BTreeSet::new();
        for _ in 0..32 {
            let result = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
            assert!(result.result.unwrap().is_empty());
            assert!(completed.insert(result.id));
        }
        assert_eq!(std::fs::read(&path).unwrap(), [32]);
        assert!(
            executor
                .shared
                .queue
                .lock()
                .unwrap()
                .active_saves
                .is_empty()
        );
        // Windows may retain the old link as delete-pending while an old
        // resource handle or the last worker job still refers to that file.
        drop(file);
        #[cfg(windows)]
        {
            let deadline = Instant::now() + Duration::from_secs(1);
            while std::fs::read_dir(&root).unwrap().count() != 1 && Instant::now() < deadline {
                thread::yield_now();
            }
        }
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
        drop(executor);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn write_job_owns_bytes_and_preserves_trailing_contents() {
        let path = std::env::temp_dir().join(format!("identity-write-job-{}", std::process::id()));
        std::fs::write(&path, b"original tail").unwrap();
        let file = Arc::new(
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)
                .unwrap(),
        );
        let (sender, receiver) = mpsc::sync_channel(1);
        let executor = Executor::new(sender);
        executor.ensure_workers().unwrap();
        let mut source = b"new".to_vec();
        let bytes: Arc<[u8]> = Arc::from(source.as_slice());
        source.fill(b'x');
        executor.submit(Job {
            sender: None,
            id: 1,
            generation: 2,
            recipe: Recipe {
                file,
                offset: 0,
                limit: 0,
                write: Some(bytes),
                save_path: None,
                create_file: false,
                action: 7,
                policy: 0,
            },
            cancelled: Arc::new(AtomicBool::new(false)),
            wake: Arc::new(crate::wake::Slot::default()),
            message: Message {
                action: 7,
                value: 1,
            },
        });
        let completion = receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(completion.generation, 2);
        assert!(completion.result.unwrap().is_empty());
        assert_eq!(std::fs::read(&path).unwrap(), b"newginal tail");
        drop(executor);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn idle_pool_is_lazy_and_partial_worker_failure_is_retryable() {
        let (sender, _receiver) = mpsc::sync_channel(1);
        let executor = Executor::new(sender);
        assert_eq!(executor.worker_count(), 0);
        let result = executor.ensure_workers_with(|name, work| {
            if name.ends_with('1') {
                Err(std::io::Error::other("injected worker creation failure"))
            } else {
                thread::Builder::new().name(name).spawn(work)
            }
        });
        assert!(
            result
                .unwrap_err()
                .contains("injected worker creation failure")
        );
        assert_eq!(executor.worker_count(), 0);
        assert_eq!(executor.queue_depth(), 0);
        assert!(!executor.shared.queue.lock().unwrap().shutdown);
        executor.ensure_workers().unwrap();
        assert_eq!(executor.worker_count(), 2);
    }
    #[test]
    fn short_read_releases_excess_capacity_while_waiting_for_recipient() {
        let path = std::env::temp_dir().join(format!("identity-short-read-{}", std::process::id()));
        std::fs::write(&path, b"data").unwrap();
        let (sender, receiver) = mpsc::sync_channel(0);
        let executor = Executor::new(sender);
        executor.ensure_workers().unwrap();
        let (notified, notification) = mpsc::channel();
        executor.submit(Job {
            sender: None,
            id: 1,
            generation: 1,
            recipe: Recipe {
                file: Arc::new(File::open(&path).unwrap()),
                offset: 0,
                limit: 1024 * 1024,
                write: None,
                save_path: None,
                create_file: false,
                action: 7,
                policy: 0,
            },
            cancelled: Arc::new(AtomicBool::new(false)),
            wake: slot(Arc::new(move || {
                let _ = notified.send(());
            })),
            message: Message {
                action: 7,
                value: 0,
            },
        });
        notification.recv_timeout(Duration::from_secs(2)).unwrap();
        let result = receiver
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
            .result
            .unwrap();
        assert_eq!(result, b"data");
        assert!(
            result.capacity() <= 4096,
            "short read retained {} bytes",
            result.capacity()
        );
        drop(executor);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn submitted_read_uses_later_installed_callback() {
        let path =
            std::env::temp_dir().join(format!("identity-read-late-wake-{}", std::process::id()));
        std::fs::write(&path, b"data").unwrap();
        let file = Arc::new(File::open(&path).unwrap());
        let wake = Arc::new(crate::wake::Slot::default());
        let (sender, receiver) = mpsc::sync_channel(0);
        let executor = Executor::new(sender);
        executor.ensure_workers().unwrap();
        executor.submit(Job {
            sender: None,
            id: 1,
            generation: 1,
            recipe: Recipe {
                file,
                offset: 0,
                limit: 4,
                write: None,
                save_path: None,
                create_file: false,
                action: 7,
                policy: 0,
            },
            cancelled: Arc::new(AtomicBool::new(false)),
            wake: wake.clone(),
            message: Message {
                action: 7,
                value: 0,
            },
        });
        let (tx, rx) = mpsc::channel();
        wake.set(Arc::new(move || {
            let _ = tx.send(());
        }));
        rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(
            receiver
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .result
                .unwrap(),
            b"data"
        );
        drop(executor);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn panicking_notifications_do_not_terminate_executor_workers() {
        let path = std::env::temp_dir().join(format!("identity-io-wake-{}", std::process::id()));
        std::fs::write(&path, b"data").unwrap();
        let file = Arc::new(File::open(&path).unwrap());
        let (sender, receiver) = mpsc::sync_channel(4);
        let executor = Executor::new(sender);
        executor.ensure_workers().unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        for id in 1..=4 {
            let count = calls.clone();
            executor.submit(Job {
                sender: None,
                id,
                generation: 1,
                recipe: Recipe {
                    file: file.clone(),
                    offset: 0,
                    limit: 4,
                    write: None,
                    save_path: None,
                    create_file: false,
                    action: 1,
                    policy: 0,
                },
                cancelled: Arc::new(AtomicBool::new(false)),
                wake: slot(Arc::new(move || {
                    count.fetch_add(1, Ordering::Release);
                    panic!("host notification failure");
                })),
                message: Message {
                    action: 1,
                    value: 0,
                },
            });
            let completion = receiver.recv_timeout(Duration::from_secs(3)).unwrap();
            assert_eq!(completion.id, id);
            assert_eq!(completion.result.unwrap(), b"data");
        }
        let deadline = Instant::now() + Duration::from_secs(3);
        while calls.load(Ordering::Acquire) < 4 && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(calls.load(Ordering::Acquire), 4);
        drop(executor);
        drop(file);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn shutdown_releases_workers_with_full_completion_channel() {
        let path =
            std::env::temp_dir().join(format!("identity-io-shutdown-{}", std::process::id()));
        std::fs::write(&path, b"owned data").unwrap();
        let file = Arc::new(File::open(&path).unwrap());
        let (sender, receiver) = mpsc::sync_channel(0);
        let executor = Executor::new(sender);
        executor.ensure_workers().unwrap();
        let shared = Arc::downgrade(&executor.shared);
        let wakes = Arc::new(AtomicUsize::new(0));
        for id in 1..=2 {
            let wake_count = wakes.clone();
            let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
                wake_count.fetch_or(1 << (id - 1), Ordering::Release);
            });
            executor.submit(Job {
                sender: None,
                id,
                generation: 1,
                recipe: Recipe {
                    file: file.clone(),
                    offset: 0,
                    limit: 10,
                    write: None,
                    save_path: None,
                    create_file: false,
                    action: 1,
                    policy: 0,
                },
                cancelled: Arc::new(AtomicBool::new(false)),
                wake: slot(wake.clone()),
                message: Message {
                    action: 1,
                    value: 0,
                },
            });
        }
        let deadline = Instant::now() + Duration::from_secs(3);
        while wakes.load(Ordering::Acquire) != 3 && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(2));
        }
        assert!(
            wakes.load(Ordering::Acquire) == 3,
            "workers must encounter the full channel"
        );
        drop(executor); // receiver stays alive and neither job cancellation flag is set
        let deadline = Instant::now() + Duration::from_secs(3);
        while shared.upgrade().is_some() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(2));
        }
        assert!(
            shared.upgrade().is_none(),
            "detached workers must release their host state"
        );
        assert!(matches!(
            receiver.try_recv(),
            Err(mpsc::TryRecvError::Disconnected)
        ));
        drop(file);
        std::fs::remove_file(path).unwrap();
    }
}
