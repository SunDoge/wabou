use crate::{Action, ActionToken, Cx, File, Key, ReloadPolicy, Resource, Result, Socket, Task};

/// A task descriptor borrowed for one actor turn. Only its data-only identity and
/// host recipe persist; this context never enters the host task or completion.
///
/// ```compile_fail
/// use blockui::{BindState, BoundTask, Cx, Task};
/// fn escape(cx: &Cx<'_>) -> BoundTask<'static, 'static> {
///     Task::new("clock").bind(cx).unwrap()
/// }
/// ```
/// ```compile_fail
/// use blockui::{BindState, Cx, Task};
/// fn send(cx: &Cx<'_>) {
///     let task = Task::new("clock").bind(cx).unwrap();
///     std::thread::scope(|scope| { scope.spawn(move || task.cancel()); });
/// }
/// ```
pub struct BoundTask<'cx, 'host> {
    task: Task,
    cx: &'cx Cx<'host>,
}
impl<'cx, 'host> BoundTask<'cx, 'host> {
    pub(crate) fn new(task: Task, cx: &'cx Cx<'host>) -> Self {
        Self { task, cx }
    }
    /// Data-only descriptor suitable for an implementation-owned capture.
    pub fn handle(&self) -> Task {
        self.task
    }
    pub fn key(&self) -> u64 {
        self.task.key()
    }
    pub fn scoped(&self, scope: Key) -> Self {
        Self::new(self.task.scoped(scope), self.cx)
    }
    pub fn on_reload(&self, policy: ReloadPolicy) -> Self {
        Self::new(self.task.on_reload(policy), self.cx)
    }
    /// See [`Task::delay`]. Queued work starts only after successful turn commit.
    pub fn delay<A: ActionToken>(
        &self,
        duration: std::time::Duration,
        completion: A,
        value: A::Payload,
    ) -> Result<()> {
        self.task.delay(self.cx, duration, completion, value)
    }
    /// See [`Task::interval`]. Completion uses the currently installed action.
    pub fn interval<A: ActionToken>(
        &self,
        period: std::time::Duration,
        completion: A,
        value: A::Payload,
    ) -> Result<()> {
        self.task.interval(self.cx, period, completion, value)
    }
    /// See [`Task::cancel`]. Already delivered mailbox messages remain.
    pub fn cancel(&self) -> Result<()> {
        self.task.cancel(self.cx)
    }
    /// See [`Task::result`]. Copies a bounded host-owned result into this turn.
    pub fn result(&self) -> Result<Option<Vec<u8>>> {
        self.task.result(self.cx)
    }
    /// See [`Task::read_file`].
    pub fn read_file(
        &self,
        file: impl Into<Resource<File>>,
        offset: u64,
        limit: usize,
        completion: Action,
    ) -> Result<()> {
        self.task
            .read_file(self.cx, file.into(), offset, limit, completion)
    }
    /// See [`Task::write_file`], including cancellation and replay restrictions.
    pub fn write_file(
        &self,
        file: impl Into<Resource<File>>,
        offset: u64,
        bytes: &[u8],
        completion: Action,
    ) -> Result<()> {
        self.task
            .write_file(self.cx, file.into(), offset, bytes, completion)
    }
    /// See [`Task::create_file`], including publication and sync semantics.
    pub fn create_file(&self, path: &str, bytes: &[u8], completion: Action) -> Result<()> {
        self.task.create_file(self.cx, path, bytes, completion)
    }
    /// Connect after commit; completion uses the current actor implementation.
    pub fn connect_socket(
        &self,
        socket: impl Into<Resource<Socket>>,
        address: &str,
        timeout: std::time::Duration,
        completion: Action,
    ) -> Result<()> {
        self.task
            .connect_socket(self.cx, socket.into(), address, timeout, completion)
    }
    pub fn read_socket(
        &self,
        socket: impl Into<Resource<Socket>>,
        limit: usize,
        timeout: std::time::Duration,
        completion: Action,
    ) -> Result<()> {
        self.task
            .read_socket(self.cx, socket.into(), limit, timeout, completion)
    }
    pub fn write_socket(
        &self,
        socket: impl Into<Resource<Socket>>,
        bytes: &[u8],
        timeout: std::time::Duration,
        completion: Action,
    ) -> Result<()> {
        self.task
            .write_socket(self.cx, socket.into(), bytes, timeout, completion)
    }
    /// See [`Task::save_file`], including inode and cancellation semantics.
    pub fn save_file(
        &self,
        file: impl Into<Resource<File>>,
        bytes: &[u8],
        completion: Action,
    ) -> Result<()> {
        self.task.save_file(self.cx, file.into(), bytes, completion)
    }
}
/// Extract the durable task descriptor without retaining its borrowed context.
impl From<&BoundTask<'_, '_>> for Task {
    fn from(value: &BoundTask<'_, '_>) -> Self {
        value.task
    }
}
