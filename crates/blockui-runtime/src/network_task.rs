//! Transactional SDK network recipes; futures, buffers and sockets stay in host code.
use crate::{
    Actor, AsyncMessage, Context, Message, SocketReloadPolicy, Task, TaskCommand, TimerCompletion,
    Turn, network::NetworkIo,
};
use blockui_abi::{Bytes, ERROR, OK};
use std::{
    ffi::c_void,
    future::Future,
    net::{SocketAddr, TcpStream},
    sync::{Arc, Mutex},
    task::{Poll, Waker},
    time::Duration,
};
const MAX_BUFFER_BYTES: usize = 16 * 1024 * 1024;
#[derive(Default)]
struct PoolState {
    io: Option<NetworkIo>,
    bytes: usize,
}
#[derive(Clone, Default)]
pub(crate) struct Pool(Arc<Mutex<PoolState>>);
#[derive(Debug)]
pub(crate) struct Permit {
    pool: Pool,
    bytes: Mutex<usize>,
}
impl std::fmt::Debug for Pool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NetworkPool")
    }
}
impl Permit {
    fn shrink(&self, bytes: usize) {
        let mut old = self.bytes.lock().unwrap();
        assert!(bytes <= *old);
        self.pool.0.lock().unwrap().bytes -= *old - bytes;
        *old = bytes;
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        self.pool.0.lock().unwrap().bytes -= *self.bytes.get_mut().unwrap();
    }
}
impl Pool {
    fn get(&self) -> Result<NetworkIo, String> {
        let mut value = self.0.lock().unwrap();
        if value.io.is_none() {
            value.io = Some(NetworkIo::new().map_err(|error| error.to_string())?);
        }
        Ok(value.io.as_ref().unwrap().clone())
    }
    fn buffer(&self, bytes: usize) -> Result<Arc<Permit>, String> {
        let mut value = self.0.lock().unwrap();
        if bytes > MAX_BUFFER_BYTES.saturating_sub(value.bytes) {
            return Err("network buffer budget exhausted".into());
        }
        value.bytes += bytes;
        Ok(Arc::new(Permit {
            pool: self.clone(),
            bytes: Mutex::new(bytes),
        }))
    }
    pub fn buffer_bytes(&self) -> usize {
        self.0.lock().unwrap().bytes
    }
    pub fn worker_count(&self) -> usize {
        usize::from(self.0.lock().unwrap().io.is_some())
    }
    pub fn waiting_count(&self) -> usize {
        self.0
            .lock()
            .unwrap()
            .io
            .as_ref()
            .map_or(0, NetworkIo::waiting_count)
    }
}
#[derive(Clone)]
pub(crate) struct Recipe {
    action: u64,
    policy: u32,
    timeout: Duration,
    operation: Operation,
}
#[derive(Clone)]
enum Operation {
    Connect {
        resource: u64,
        address: SocketAddr,
        policy: SocketReloadPolicy,
    },
    Read {
        stream: Arc<TcpStream>,
        limit: usize,
    },
    Write {
        stream: Arc<TcpStream>,
        bytes: Arc<[u8]>,
    },
}
#[derive(Debug)]
pub(crate) enum Output {
    Bytes {
        result: Arc<crate::io::ReadResult>,
        permit: Arc<Permit>,
    },
    Connect {
        resource: u64,
        policy: SocketReloadPolicy,
        result: Result<Arc<TcpStream>, String>,
    },
}
#[derive(Default)]
struct Gate {
    started: bool,
    waker: Option<Waker>,
}
pub(crate) struct Prepared {
    pub task: Task,
    gate: Arc<Mutex<Gate>>,
}
impl Prepared {
    pub fn activate(self) -> Task {
        let wake = {
            let mut gate = self.gate.lock().unwrap();
            gate.started = true;
            gate.waker.take()
        };
        if let Some(wake) = wake {
            wake.wake();
        }
        self.task
    }
}
async fn dormant<F: Future<Output = Output> + Send + 'static>(
    gate: Arc<Mutex<Gate>>,
    future: F,
    message: Message,
) -> AsyncMessage {
    std::future::poll_fn(move |cx| {
        let retired = {
            let mut state = gate.lock().unwrap();
            if state.started {
                return Poll::Ready(());
            }
            state.waker.replace(cx.waker().clone())
        };
        drop(retired);
        Poll::Pending
    })
    .await;
    AsyncMessage {
        message,
        text: None,
        network: Some(Arc::new(future.await)),
    }
}

pub(crate) fn prepare(
    actor: &mut Actor,
    id: u64,
    generation: u64,
    recipe: Recipe,
) -> Result<Prepared, String> {
    let network = actor.network_pool.get()?;
    let future: std::pin::Pin<Box<dyn Future<Output = Output> + Send>> = match recipe.operation {
        Operation::Connect {
            resource,
            address,
            policy,
        } => {
            let connect = network
                .connect(address, recipe.timeout)
                .map_err(|error| error.to_string())?;
            Box::pin(async move {
                Output::Connect {
                    resource,
                    policy,
                    result: connect
                        .await
                        .map(Arc::new)
                        .map_err(|error| error.to_string()),
                }
            })
        }
        Operation::Read { stream, limit } => {
            let permit = actor.network_pool.buffer(limit)?;
            let read = network
                .read(&stream, limit, recipe.timeout)
                .map_err(|error| error.to_string())?;
            Box::pin(async move {
                let result = read.await.map_err(|error| error.to_string());
                permit.shrink(result.as_ref().map_or(0, Vec::len));
                Output::Bytes {
                    result: Arc::new(result),
                    permit,
                }
            })
        }
        Operation::Write { stream, bytes } => {
            let permit = actor.network_pool.buffer(bytes.len())?;
            let write = network
                .write_shared(&stream, bytes, recipe.timeout)
                .map_err(|error| error.to_string())?;
            Box::pin(async move {
                let result = write
                    .await
                    .map(|_| Vec::new())
                    .map_err(|error| error.to_string());
                permit.shrink(0);
                Output::Bytes {
                    result: Arc::new(result),
                    permit,
                }
            })
        }
    };
    let message = Message {
        action: recipe.action,
        value: id as i64,
    };
    let gate = Arc::new(Mutex::new(Gate::default()));
    let handle = actor.future_executor.spawn_message(
        actor.id,
        TimerCompletion {
            id,
            generation,
            message,
            text: None,
            network: None,
        },
        dormant(gate.clone(), future, message),
        actor.sender.clone(),
        actor.wake.clone(),
    )?;
    Ok(Prepared {
        task: Task {
            cancelled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            future: Some(handle),
            timer: None,
            policy: recipe.policy,
            duration: Duration::ZERO,
            repeating: false,
            message,
            io: None,
            network: true,
            generation,
        },
        gate,
    })
}
fn can_stage(turn: &Turn, id: u64, action: u64, policy: u32, timeout: Duration) -> bool {
    !turn.read_only
        && turn.journal.is_some()
        && id != 0
        && action != 0
        && policy <= 1
        && !timeout.is_zero()
        && timeout <= Duration::from_secs(120)
        && turn.task_keys.len() < 64
        && !turn.task_keys.contains(&id)
        && turn.task_commands.len() < 256
        && (turn.io_results.len() < 64 || turn.io_results.contains_key(&id))
}
fn stage(turn: &mut Turn, id: u64, recipe: Recipe) -> i32 {
    if !can_stage(turn, id, recipe.action, recipe.policy, recipe.timeout) {
        return ERROR;
    }
    if !turn.reserve_task_generation() {
        return ERROR;
    }
    if let Operation::Connect { resource, .. } = recipe.operation {
        turn.socket_pending.insert(id, resource);
    }
    turn.task_keys.insert(id);
    turn.io_results.insert(id, None);
    turn.network_permits.remove(&id);
    turn.task_commands.push(TaskCommand::Network(id, recipe));
    OK
}
pub(crate) unsafe extern "C" fn connect(
    ctx: *mut c_void,
    id: u64,
    resource: u64,
    address: Bytes,
    millis: u64,
    action: u64,
    policy: u32,
    resource_policy: u32,
) -> i32 {
    let turn = unsafe { &mut *(*ctx.cast::<Context>()).turn };
    if resource == 0
        || resource_policy > 1
        || address.len == 0
        || address.len > 128
        || address.ptr.is_null()
        || turn.sockets.len() + turn.socket_pending.len() >= 64
        || turn.sockets.get(resource).is_some()
        || turn
            .socket_pending
            .values()
            .any(|pending| *pending == resource)
    {
        return ERROR;
    }
    let Ok(address) =
        std::str::from_utf8(unsafe { std::slice::from_raw_parts(address.ptr, address.len) })
    else {
        return ERROR;
    };
    let Ok(address) = address.parse::<SocketAddr>() else {
        return ERROR;
    };
    if address.port() == 0 {
        return ERROR;
    }
    stage(
        turn,
        id,
        Recipe {
            action,
            policy,
            timeout: Duration::from_millis(millis),
            operation: Operation::Connect {
                resource,
                address,
                policy: if resource_policy == 0 {
                    SocketReloadPolicy::Preserve
                } else {
                    SocketReloadPolicy::Cancel
                },
            },
        },
    )
}
pub(crate) unsafe extern "C" fn read(
    ctx: *mut c_void,
    id: u64,
    resource: u64,
    limit: usize,
    millis: u64,
    action: u64,
    policy: u32,
) -> i32 {
    let turn = unsafe { &mut *(*ctx.cast::<Context>()).turn };
    if limit == 0 || limit > 1024 * 1024 {
        return ERROR;
    }
    let Some(stream) = turn.sockets.handle(resource) else {
        return ERROR;
    };
    stage(
        turn,
        id,
        Recipe {
            action,
            policy,
            timeout: Duration::from_millis(millis),
            operation: Operation::Read { stream, limit },
        },
    )
}
pub(crate) unsafe extern "C" fn write(
    ctx: *mut c_void,
    id: u64,
    resource: u64,
    bytes: Bytes,
    millis: u64,
    action: u64,
    policy: u32,
) -> i32 {
    let turn = unsafe { &mut *(*ctx.cast::<Context>()).turn };
    if bytes.len > 1024 * 1024 || (bytes.len != 0 && bytes.ptr.is_null()) {
        return ERROR;
    }
    if !can_stage(turn, id, action, policy, Duration::from_millis(millis)) {
        return ERROR;
    }
    // Bound transaction-owned copies before touching the plugin's payload.
    // Prepared/live futures are separately admitted by the shared pool.
    let staged: usize = turn
        .task_commands
        .iter()
        .filter_map(|command| match command {
            TaskCommand::Network(
                _,
                Recipe {
                    operation: Operation::Write { bytes, .. },
                    ..
                },
            ) => Some(bytes.len()),
            _ => None,
        })
        .sum();
    if bytes.len > MAX_BUFFER_BYTES.saturating_sub(staged) {
        return ERROR;
    }
    let Some(stream) = turn.sockets.handle(resource) else {
        return ERROR;
    };
    let bytes: Arc<[u8]> = if bytes.len == 0 {
        Arc::from([])
    } else {
        Arc::from(unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) })
    };
    stage(
        turn,
        id,
        Recipe {
            action,
            policy,
            timeout: Duration::from_millis(millis),
            operation: Operation::Write { stream, bytes },
        },
    )
}
pub(crate) unsafe extern "C" fn close(ctx: *mut c_void, resource: u64) -> i32 {
    let turn = unsafe { &mut *(*ctx.cast::<Context>()).turn };
    if turn.read_only
        || turn.journal.is_none()
        || resource == 0
        || turn
            .socket_pending
            .values()
            .any(|pending| *pending == resource)
    {
        return ERROR;
    }
    turn.sockets.remove(resource);
    OK
}
pub(crate) fn complete(
    turn: &mut Turn,
    id: u64,
    output: Option<Arc<Output>>,
) -> Arc<crate::io::ReadResult> {
    let reserved = turn.socket_pending.remove(&id);
    let result = match output.as_deref() {
        Some(Output::Bytes { result, permit }) => {
            turn.network_permits.insert(id, permit.clone());
            result.clone()
        }
        Some(Output::Connect {
            resource,
            policy,
            result,
        }) => {
            let result = if reserved != Some(*resource) {
                Err("socket reservation no longer belongs to task".into())
            } else {
                match result {
                    Ok(stream) => turn
                        .sockets
                        .insert_arc(*resource, stream.clone(), *policy)
                        .map(|()| Vec::new()),
                    Err(error) => Err(error.clone()),
                }
            };
            Arc::new(result)
        }
        None => Arc::new(Err("host network task panicked".into())),
    };
    turn.io_results.insert(id, Some(result.clone()));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        path::Path,
        time::Instant,
    };
    fn gate_polled(prepared: &Prepared) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while prepared.gate.lock().unwrap().waker.is_none() && Instant::now() < deadline {
            std::thread::yield_now();
        }
        assert!(prepared.gate.lock().unwrap().waker.is_some());
    }
    #[test]
    fn zero_port_host_requests_do_not_stage_tasks_or_start_workers() {
        let mut actor = Actor::new(649, Path::new(".runtime")).unwrap();
        for address in ["127.0.0.1:0", "[::1]:0"] {
            actor
                .transaction(|actor| {
                    let code = crate::with_api(&mut actor.turn, std::ptr::null(), |api| unsafe {
                        (api.connect_socket_async)(
                            api.context,
                            1,
                            2,
                            Bytes::new(address),
                            1000,
                            3,
                            0,
                            0,
                        )
                    });
                    assert_eq!(code, ERROR);
                    assert!(actor.turn.task_commands.is_empty());
                    assert!(actor.turn.socket_pending.is_empty());
                    assert!(actor.turn.task_keys.is_empty());
                    assert!(actor.turn.io_results.is_empty());
                    Ok(None)
                })
                .unwrap();
            assert_eq!(actor.network_worker_count(), 0);
            assert_eq!(actor.network_waiting_count(), 0);
            assert_eq!(actor.socket_count(), 0);
        }
    }
    #[test]
    fn dormant_connect_read_write_only_run_after_activation() {
        let mut actor = Actor::new(648, Path::new(".runtime")).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let prepared = prepare(
            &mut actor,
            1,
            1,
            Recipe {
                action: 1,
                policy: 0,
                timeout: Duration::from_secs(2),
                operation: Operation::Connect {
                    resource: 1,
                    address: listener.local_addr().unwrap(),
                    policy: SocketReloadPolicy::Preserve,
                },
            },
        )
        .unwrap();
        gate_polled(&prepared);
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
        assert_eq!(actor.network_waiting_count(), 0);
        drop(prepared);
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
        let stream = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (mut peer, _) = listener.accept().unwrap();
        peer.set_nonblocking(true).unwrap();
        stream.set_nonblocking(true).unwrap();
        let stream = Arc::new(stream);
        peer.write_all(b"raw").unwrap();
        let prepared = prepare(
            &mut actor,
            2,
            2,
            Recipe {
                action: 1,
                policy: 0,
                timeout: Duration::from_secs(2),
                operation: Operation::Read {
                    stream: stream.clone(),
                    limit: 128,
                },
            },
        )
        .unwrap();
        gate_polled(&prepared);
        assert_eq!(actor.network_buffer_bytes(), 128);
        let mut bytes = [0; 3];
        assert_eq!(stream.peek(&mut bytes).unwrap(), 3);
        drop(prepared);
        assert_eq!(actor.network_buffer_bytes(), 0);
        assert_eq!(stream.peek(&mut bytes).unwrap(), 3);
        let prepared = prepare(
            &mut actor,
            3,
            3,
            Recipe {
                action: 1,
                policy: 0,
                timeout: Duration::from_secs(2),
                operation: Operation::Write {
                    stream: stream.clone(),
                    bytes: Arc::from(&b"send"[..]),
                },
            },
        )
        .unwrap();
        gate_polled(&prepared);
        assert_eq!(
            peer.read(&mut [0; 4]).unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
        let _task = prepared.activate();
        let completion = actor.receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(completion.generation, 3);
        assert!(
            matches!(completion.network.as_deref(), Some(Output::Bytes { result, .. }) if result.as_ref() == &Ok(Vec::new()))
        );
        peer.set_nonblocking(false).unwrap();
        peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let mut bytes = [0; 4];
        peer.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"send");
        assert_eq!(actor.network_buffer_bytes(), 0);
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn async_write_timeout_reports_partial_progress_and_releases_budget() {
        use std::os::fd::AsRawFd;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let stream = Arc::new(TcpStream::connect(listener.local_addr().unwrap()).unwrap());
        let (mut peer, _) = listener.accept().unwrap();
        let size: libc::c_int = 4096;
        assert_eq!(
            unsafe {
                libc::setsockopt(
                    stream.as_raw_fd(),
                    libc::SOL_SOCKET,
                    libc::SO_SNDBUF,
                    (&size as *const libc::c_int).cast(),
                    std::mem::size_of_val(&size) as libc::socklen_t,
                )
            },
            0
        );
        let mut actor = Actor::new(652, Path::new(".runtime")).unwrap();
        let bytes: Arc<[u8]> = Arc::from(vec![42; 1024 * 1024]);
        let owner = Arc::downgrade(&bytes);
        let prepared = prepare(
            &mut actor,
            1,
            1,
            Recipe {
                action: 3,
                policy: 0,
                timeout: Duration::from_millis(30),
                operation: Operation::Write {
                    stream: stream.clone(),
                    bytes,
                },
            },
        )
        .unwrap();
        gate_polled(&prepared);
        assert_eq!(actor.network_buffer_bytes(), 1024 * 1024);
        let _task = prepared.activate();
        let completion = actor.receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        let Some(Output::Bytes { result, .. }) = completion.network.as_deref() else {
            panic!("write completion lost its network result");
        };
        let error = result.as_ref().as_ref().unwrap_err();
        let sent: usize = error
            .strip_prefix("network write failed after ")
            .unwrap()
            .split_once(" bytes: ")
            .unwrap()
            .0
            .parse()
            .unwrap();
        assert!(sent > 0 && sent < 1024 * 1024, "{error}");
        assert!(owner.upgrade().is_none());
        assert_eq!(actor.network_buffer_bytes(), 0);
        peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let mut received = vec![0; sent];
        peer.read_exact(&mut received).unwrap();
        assert!(received.iter().all(|byte| *byte == 42));
        // Retain the old completion while the same task identity has a new generation.
        let prepared = prepare(
            &mut actor,
            1,
            2,
            Recipe {
                action: 3,
                policy: 0,
                timeout: Duration::from_secs(2),
                operation: Operation::Write {
                    stream: stream.clone(),
                    bytes: Arc::from(&b"recovered"[..]),
                },
            },
        )
        .unwrap();
        let _next_task = prepared.activate();
        let next = actor.receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(completion.generation, 1);
        assert_eq!(next.generation, 2);
        assert!(
            matches!(next.network.as_deref(), Some(Output::Bytes { result, .. }) if result.as_ref() == &Ok(Vec::new()))
        );
        assert!(
            matches!(completion.network.as_deref(), Some(Output::Bytes { result, .. }) if result.is_err())
        );
        let mut recovered = [0; 9];
        peer.read_exact(&mut recovered).unwrap();
        assert_eq!(&recovered, b"recovered");
        assert_eq!(actor.network_buffer_bytes(), 0);
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut trailing = Vec::new();
        peer.read_to_end(&mut trailing).unwrap();
        assert!(
            trailing.is_empty(),
            "failed payload was unexpectedly replayed"
        );
    }

    #[test]
    fn staged_writes_reject_over_budget_and_rollback_releases_payloads() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let stream = Arc::new(TcpStream::connect(listener.local_addr().unwrap()).unwrap());
        let _peer = listener.accept().unwrap().0;
        let mut actor = Actor::new(651, Path::new(".runtime")).unwrap();
        let payload = vec![42; 1024 * 1024];
        let mut owners = Vec::new();
        let result = actor.transaction(|actor| {
            actor
                .turn
                .sockets
                .insert_arc(7, stream.clone(), SocketReloadPolicy::Preserve)?;
            for id in 1..=16 {
                let code = crate::with_api(&mut actor.turn, std::ptr::null(), |api| unsafe {
                    (api.write_socket_async)(
                        api.context,
                        id,
                        7,
                        Bytes {
                            ptr: payload.as_ptr(),
                            len: payload.len(),
                        },
                        1000,
                        3,
                        0,
                    )
                });
                assert_eq!(code, OK);
            }
            for command in &actor.turn.task_commands {
                if let TaskCommand::Network(
                    _,
                    Recipe {
                        operation: Operation::Write { bytes, .. },
                        ..
                    },
                ) = command
                {
                    owners.push(Arc::downgrade(bytes));
                }
            }
            assert_eq!(owners.len(), 16);
            let code = crate::with_api(&mut actor.turn, std::ptr::null(), |api| unsafe {
                (api.write_socket_async)(
                    api.context,
                    17,
                    7,
                    Bytes {
                        ptr: payload.as_ptr(),
                        len: 1,
                    },
                    1000,
                    3,
                    0,
                )
            });
            assert_eq!(code, ERROR);
            assert_eq!(actor.turn.task_commands.len(), 16);
            assert!(!actor.turn.task_keys.contains(&17));
            assert!(!actor.turn.io_results.contains_key(&17));
            Err("rollback staged writes".into())
        });
        assert!(result.is_err());
        assert!(owners.iter().all(|owner| owner.upgrade().is_none()));
        assert!(actor.turn.task_commands.is_empty());
        assert_eq!(actor.network_worker_count(), 0);
        assert_eq!(actor.network_buffer_bytes(), 0);
    }
    #[test]
    fn payload_budget_is_shared_rejects_without_growth_and_releases_on_last_owner() {
        let pool = Pool::default();
        let first = pool.buffer(MAX_BUFFER_BYTES).unwrap();
        assert!(pool.buffer(1).is_err());
        let snapshot = first.clone();
        first.shrink(3);
        assert_eq!(pool.buffer_bytes(), 3);
        let other = pool.clone();
        let rest = other.buffer(MAX_BUFFER_BYTES - 3).unwrap();
        assert!(pool.buffer(1).is_err());
        assert_eq!(pool.buffer_bytes(), MAX_BUFFER_BYTES);
        drop(first);
        assert_eq!(pool.buffer_bytes(), MAX_BUFFER_BYTES);
        drop(snapshot);
        assert_eq!(other.buffer_bytes(), MAX_BUFFER_BYTES - 3);
        drop(rest);
        assert_eq!(pool.buffer_bytes(), 0);
        assert_eq!(pool.worker_count(), 0);
    }
}
