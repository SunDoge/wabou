//! Readiness-driven I/O compiled into the stable host. No plugin callbacks.
#[cfg(windows)]
use std::os::windows::io::AsRawSocket;
#[cfg(unix)]
use std::os::{
    fd::{AsRawFd, FromRawFd},
    unix::net::UnixStream as WakeStream,
};
use std::{
    collections::BTreeMap,
    future::Future,
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream},
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    task::{Context, Poll, Waker},
    thread,
    time::{Duration, Instant},
};
#[cfg(windows)]
type WakeStream = TcpStream;
#[cfg(unix)]
use libc as readiness;
#[cfg(windows)]
use windows_sys::Win32::Networking::WinSock as readiness;
#[cfg(unix)]
fn wake_pair() -> io::Result<(WakeStream, WakeStream)> {
    WakeStream::pair()
}
#[cfg(windows)]
fn wake_pair() -> io::Result<(WakeStream, WakeStream)> {
    let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))?;
    let sender = TcpStream::connect(listener.local_addr()?)?;
    let (receiver, _) = listener.accept()?;
    Ok((sender, receiver))
}
const MAX_BYTES: usize = 1024 * 1024;
const MAX_WAITERS: usize = 256;
struct Registration {
    stream: Arc<TcpStream>,
    events: i16,
    deadline: Instant,
    waker: Waker,
}
struct State {
    registrations: BTreeMap<u64, Registration>,
    failure: Option<String>,
}
struct Shared {
    state: Mutex<State>,
    wake: WakeStream,
    stopped: AtomicBool,
    next: AtomicU64,
}
impl Shared {
    fn notify(&self) {
        let _ = (&self.wake).write(&[1]);
    }
    fn remove(&self, id: u64) {
        let removed = self.state.lock().unwrap().registrations.remove(&id);
        if removed.is_some() {
            self.notify();
        }
        // Wakers may run destructors that re-enter the registry; drop unlocked.
        drop(removed);
    }
}
struct Inner {
    shared: Arc<Shared>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Drop for Inner {
    fn drop(&mut self) {
        self.shared.stopped.store(true, Ordering::Release);
        self.shared.notify();
        if let Some(thread) = self.thread.take()
            && thread.thread().id() != thread::current().id()
        {
            let _ = thread.join();
        }
    }
}
/// A shareable readiness worker for TCP connect/read/write in the stable host.
/// Construction starts one worker; clones and pending reads/writes share it.
/// Futures use nonblocking I/O and may be awaited inside `spawn_async_message`.
/// Cloned socket descriptors share nonblocking flags with the original socket.
#[derive(Clone)]
pub struct NetworkIo(Arc<Inner>);
impl NetworkIo {
    pub fn new() -> io::Result<Self> {
        let (wake, control) = wake_pair()?;
        wake.set_nonblocking(true)?;
        control.set_nonblocking(true)?;
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                registrations: BTreeMap::new(),
                failure: None,
            }),
            wake,
            stopped: AtomicBool::new(false),
            next: AtomicU64::new(1),
        });
        let worker = shared.clone();
        let thread = thread::Builder::new()
            .name("actor-network-ready".into())
            .spawn(move || run(worker, control))?;
        Ok(Self(Arc::new(Inner {
            shared,
            thread: Some(thread),
        })))
    }
    fn operation(&self, stream: &TcpStream, timeout: Duration) -> io::Result<Operation> {
        validate_timeout(timeout)?;
        let stream = stream.try_clone()?;
        stream.set_nonblocking(true)?;
        let id = self
            .0
            .shared
            .next
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| io::Error::other("network identity exhausted"))?;
        Ok(Operation {
            inner: self.0.clone(),
            stream: Arc::new(stream),
            id,
            timeout,
            deadline: None,
        })
    }
    /// Connect to a concrete IP address without DNS or blocking the caller.
    /// The timeout starts at first poll, after any transaction preparation delay.
    /// Connection initiation occurs at first poll; drop closes the local socket.
    /// The peer may already have observed a handshake before cancellation.
    pub fn connect(&self, address: SocketAddr, timeout: Duration) -> io::Result<SocketConnect> {
        validate_timeout(timeout)?;
        #[cfg(unix)]
        let stream = {
            let family = if address.is_ipv4() {
                libc::AF_INET
            } else {
                libc::AF_INET6
            };
            #[cfg(any(target_os = "linux", target_os = "android"))]
            let flags = libc::SOCK_STREAM | libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC;
            #[cfg(not(any(target_os = "linux", target_os = "android")))]
            let flags = libc::SOCK_STREAM;
            let fd = unsafe { libc::socket(family, flags, 0) };
            if fd < 0 {
                return Err(io::Error::last_os_error());
            }
            let stream = unsafe { TcpStream::from_raw_fd(fd) };
            #[cfg(not(any(target_os = "linux", target_os = "android")))]
            if unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) } < 0 {
                return Err(io::Error::last_os_error());
            }
            stream
        };
        #[cfg(windows)]
        let stream: TcpStream = socket2::Socket::new(
            socket2::Domain::for_address(address),
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        )?
        .into();
        Ok(SocketConnect {
            operation: Some(self.operation(&stream, timeout)?),
            address: Some(address),
        })
    }
    /// Read one available TCP chunk (not a framed message). Empty means EOF.
    /// Dropping before completion unregisters readiness; consumed bytes cannot be undone.
    pub fn read(
        &self,
        stream: &TcpStream,
        limit: usize,
        timeout: Duration,
    ) -> io::Result<SocketRead> {
        if limit == 0 || limit > MAX_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "network read limit must be within 1..=1MiB",
            ));
        }
        Ok(SocketRead {
            operation: Some(self.operation(stream, timeout)?),
            bytes: vec![0; limit],
        })
    }
    /// Send up to 1 MiB completely. Cancellation/errors can follow partial sends.
    /// Use one reader and one writer per connection; framing/protocol ownership is external.
    pub fn write(
        &self,
        stream: &TcpStream,
        bytes: &[u8],
        timeout: Duration,
    ) -> io::Result<SocketWrite> {
        if bytes.len() > MAX_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "network write limit exceeded",
            ));
        }
        self.write_shared(stream, Arc::from(bytes), timeout)
    }
    /// Reuse host-owned recipe storage without copying the payload again.
    pub(crate) fn write_shared(
        &self,
        stream: &TcpStream,
        bytes: Arc<[u8]>,
        timeout: Duration,
    ) -> io::Result<SocketWrite> {
        if bytes.len() > MAX_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "network write limit exceeded",
            ));
        }
        Ok(SocketWrite {
            operation: Some(self.operation(stream, timeout)?),
            bytes,
            sent: 0,
        })
    }
    pub fn waiting_count(&self) -> usize {
        self.0.shared.state.lock().unwrap().registrations.len()
    }
}
fn validate_timeout(timeout: Duration) -> io::Result<()> {
    if timeout.is_zero() || timeout > Duration::from_secs(120) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "network timeout must be within (0, 120s]",
        ));
    }
    Ok(())
}
#[cfg(unix)]
fn initiate(stream: &TcpStream, address: SocketAddr) -> io::Result<()> {
    let result = match address {
        SocketAddr::V4(address) => {
            let mut raw: libc::sockaddr_in = unsafe { std::mem::zeroed() };
            raw.sin_family = libc::AF_INET as libc::sa_family_t;
            raw.sin_port = address.port().to_be();
            raw.sin_addr.s_addr = u32::from_ne_bytes(address.ip().octets());
            #[cfg(any(
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd",
                target_os = "dragonfly"
            ))]
            {
                raw.sin_len = std::mem::size_of_val(&raw) as u8;
            }
            unsafe {
                libc::connect(
                    stream.as_raw_fd(),
                    (&raw as *const libc::sockaddr_in).cast(),
                    std::mem::size_of_val(&raw) as libc::socklen_t,
                )
            }
        }
        SocketAddr::V6(address) => {
            let mut raw: libc::sockaddr_in6 = unsafe { std::mem::zeroed() };
            raw.sin6_family = libc::AF_INET6 as libc::sa_family_t;
            raw.sin6_port = address.port().to_be();
            raw.sin6_addr.s6_addr = address.ip().octets();
            // Match Rust SocketAddrV6 native representation (flowinfo is opaque).
            raw.sin6_flowinfo = address.flowinfo();
            raw.sin6_scope_id = address.scope_id();
            #[cfg(any(
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd",
                target_os = "dragonfly"
            ))]
            {
                raw.sin6_len = std::mem::size_of_val(&raw) as u8;
            }
            unsafe {
                libc::connect(
                    stream.as_raw_fd(),
                    (&raw as *const libc::sockaddr_in6).cast(),
                    std::mem::size_of_val(&raw) as libc::socklen_t,
                )
            }
        }
    };
    if result == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    if matches!(error.raw_os_error(), Some(libc::EINPROGRESS | libc::EINTR)) {
        Ok(())
    } else {
        Err(error)
    }
}
#[cfg(windows)]
fn initiate(stream: &TcpStream, address: SocketAddr) -> io::Result<()> {
    match socket2::SockRef::from(stream).connect(&address.into()) {
        Ok(()) => Ok(()),
        Err(error)
            if matches!(
                error.raw_os_error(),
                Some(
                    readiness::WSAEWOULDBLOCK | readiness::WSAEINPROGRESS | readiness::WSAEALREADY
                )
            ) =>
        {
            Ok(())
        }
        Err(error) => Err(error),
    }
}
fn connect_pending(error: &io::Error) -> bool {
    #[cfg(unix)]
    {
        error.raw_os_error() == Some(libc::ENOTCONN)
    }
    #[cfg(windows)]
    {
        error.raw_os_error() == Some(readiness::WSAENOTCONN)
    }
}
#[cfg(unix)]
type PollFd = libc::pollfd;
#[cfg(windows)]
type PollFd = readiness::WSAPOLLFD;
#[cfg(unix)]
fn descriptor(stream: &impl AsRawFd, events: i16) -> PollFd {
    PollFd {
        fd: stream.as_raw_fd(),
        events,
        revents: 0,
    }
}
#[cfg(windows)]
fn descriptor(stream: &impl AsRawSocket, events: i16) -> PollFd {
    PollFd {
        fd: stream.as_raw_socket() as usize,
        events,
        revents: 0,
    }
}
fn poll_ready(descriptors: &mut [PollFd], timeout: i32) -> io::Result<()> {
    #[cfg(unix)]
    let result = unsafe {
        libc::poll(
            descriptors.as_mut_ptr(),
            descriptors.len() as libc::nfds_t,
            timeout,
        )
    };
    #[cfg(windows)]
    let result =
        unsafe { readiness::WSAPoll(descriptors.as_mut_ptr(), descriptors.len() as u32, timeout) };
    if result < 0 {
        #[cfg(unix)]
        {
            Err(io::Error::last_os_error())
        }
        #[cfg(windows)]
        {
            Err(io::Error::from_raw_os_error(unsafe {
                readiness::WSAGetLastError()
            }))
        }
    } else {
        Ok(())
    }
}
/// A pending nonblocking connection, using the same readiness worker as I/O.
/// Successful output remains nonblocking and is owned by the stable executable.
pub struct SocketConnect {
    operation: Option<Operation>,
    address: Option<SocketAddr>,
}
impl Future for SocketConnect {
    type Output = io::Result<TcpStream>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let Some(operation) = &mut this.operation else {
            return Poll::Ready(Err(io::Error::other(
                "connect future polled after completion",
            )));
        };
        let result = (|| {
            operation.check()?;
            if let Some(address) = this.address.take() {
                initiate(&operation.stream, address)?;
            }
            if let Some(error) = operation.stream.take_error()? {
                return Err(error);
            }
            match operation.stream.peer_addr() {
                Ok(_) => Ok(Poll::Ready(operation.stream.try_clone()?)),
                Err(error) if connect_pending(&error) => {
                    operation.wait(cx, readiness::POLLOUT)?;
                    Ok(Poll::Pending)
                }
                Err(error) => Err(error),
            }
        })();
        match result {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready(stream)) => {
                drop(this.operation.take());
                Poll::Ready(Ok(stream))
            }
            Err(error) => {
                // Close pending connection ownership even when the caller keeps
                // the completed future. Snapshot ownership retires after notify.
                drop(this.operation.take());
                Poll::Ready(Err(error))
            }
        }
    }
}
struct Operation {
    inner: Arc<Inner>,
    stream: Arc<TcpStream>,
    id: u64,
    timeout: Duration,
    deadline: Option<Instant>,
}
impl Operation {
    fn check(&mut self) -> io::Result<()> {
        let now = Instant::now();
        let deadline = match self.deadline {
            Some(deadline) => deadline,
            None => {
                let deadline = now
                    .checked_add(self.timeout)
                    .ok_or_else(|| io::Error::other("network deadline overflow"))?;
                self.deadline = Some(deadline);
                deadline
            }
        };
        if now >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "network operation timed out",
            ));
        }
        if let Some(failure) = &self.inner.shared.state.lock().unwrap().failure {
            return Err(io::Error::other(failure.clone()));
        }
        Ok(())
    }
    fn wait(&self, cx: &Context<'_>, events: i16) -> io::Result<()> {
        let shared = &self.inner.shared;
        // RawWaker clone is user code too: it may panic or enter the registry.
        // Build the owned registration before locking, including its waker.
        let registration = Registration {
            stream: self.stream.clone(),
            events,
            deadline: self
                .deadline
                .expect("network check starts deadline before waiting"),
            waker: cx.waker().clone(),
        };
        let retired = {
            let mut state = shared.state.lock().unwrap();
            if let Some(failure) = &state.failure {
                return Err(io::Error::other(failure.clone()));
            }
            if state.registrations.len() >= MAX_WAITERS
                && !state.registrations.contains_key(&self.id)
            {
                return Err(io::Error::other("network waiter limit reached"));
            }
            state.registrations.insert(self.id, registration)
        };
        shared.notify();
        drop(retired);
        Ok(())
    }
}
impl Drop for Operation {
    fn drop(&mut self) {
        self.inner.shared.remove(self.id);
    }
}
/// One pending nonblocking read. Its ownership and drop glue belong to the host.
pub struct SocketRead {
    operation: Option<Operation>,
    bytes: Vec<u8>,
}
impl Future for SocketRead {
    type Output = io::Result<Vec<u8>>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let Some(operation) = &mut this.operation else {
            return Poll::Ready(Err(io::Error::other("read future polled after completion")));
        };
        let result = (|| {
            operation.check()?;
            match (&*operation.stream).read(&mut this.bytes) {
                Ok(length) => {
                    this.bytes.truncate(length);
                    this.bytes.shrink_to_fit();
                    Ok(Poll::Ready(std::mem::take(&mut this.bytes)))
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    operation.wait(cx, readiness::POLLIN)?;
                    Ok(Poll::Pending)
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                    cx.waker().wake_by_ref();
                    Ok(Poll::Pending)
                }
                Err(error) => Err(error),
            }
        })();
        match result {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready(bytes)) => {
                drop(this.operation.take());
                Poll::Ready(Ok(bytes))
            }
            Err(error) => {
                drop(this.operation.take());
                this.bytes = Vec::new();
                Poll::Ready(Err(error))
            }
        }
    }
}
/// An error after zero or more bytes have reached the kernel send buffer.
#[derive(Debug)]
pub struct SocketWriteError {
    pub sent: usize,
    pub error: io::Error,
}
impl std::fmt::Display for SocketWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "network write failed after {} bytes: {}",
            self.sent, self.error
        )
    }
}
impl std::error::Error for SocketWriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}
pub struct SocketWrite {
    operation: Option<Operation>,
    bytes: Arc<[u8]>,
    sent: usize,
}
impl Future for SocketWrite {
    type Output = Result<usize, SocketWriteError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let Some(operation) = &mut this.operation else {
            return Poll::Ready(Err(SocketWriteError {
                sent: this.sent,
                error: io::Error::other("write future polled after completion"),
            }));
        };
        let result = (|| {
            operation.check()?;
            for _ in 0..32 {
                if this.sent == this.bytes.len() {
                    return Ok(Poll::Ready(this.sent));
                }
                match (&*operation.stream).write(&this.bytes[this.sent..]) {
                    Ok(0) => {
                        return Err(io::Error::new(
                            io::ErrorKind::WriteZero,
                            "socket wrote zero bytes",
                        ));
                    }
                    Ok(length) => this.sent += length,
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        operation.wait(cx, readiness::POLLOUT)?;
                        return Ok(Poll::Pending);
                    }
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                    Err(error) => return Err(error),
                }
            }
            cx.waker().wake_by_ref();
            Ok(Poll::Pending)
        })();
        match result {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready(sent)) => {
                drop(this.operation.take());
                this.bytes = Arc::from([]);
                Poll::Ready(Ok(sent))
            }
            Err(error) => {
                drop(this.operation.take());
                this.bytes = Arc::from([]);
                Poll::Ready(Err(SocketWriteError {
                    sent: this.sent,
                    error,
                }))
            }
        }
    }
}
fn wake_registration(registration: Registration) {
    // Custom stable-host wakers can panic or re-enter resource ownership.
    // One failing wake must not strand unrelated sockets on this worker.
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| registration.waker.wake()));
}
fn run(shared: Arc<Shared>, mut control: WakeStream) {
    while !shared.stopped.load(Ordering::Acquire) {
        let snapshot: Vec<_> = shared
            .state
            .lock()
            .unwrap()
            .registrations
            .iter()
            .map(|(id, registration)| {
                (
                    *id,
                    registration.stream.clone(),
                    registration.events,
                    registration.deadline,
                )
            })
            .collect();
        let mut descriptors = Vec::with_capacity(snapshot.len() + 1);
        descriptors.push(descriptor(&control, readiness::POLLIN));
        descriptors.extend(
            snapshot
                .iter()
                .map(|(_, stream, events, _)| descriptor(&**stream, *events)),
        );
        let now = Instant::now();
        let timeout = snapshot
            .iter()
            .map(|(_, _, _, deadline)| deadline.saturating_duration_since(now))
            .min()
            .map_or(-1, |duration| {
                duration.as_millis().saturating_add(1).min(i32::MAX as u128) as i32
            });
        if let Err(error) = poll_ready(&mut descriptors, timeout) {
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            let retired = {
                let mut state = shared.state.lock().unwrap();
                state.failure = Some(error.to_string());
                std::mem::take(&mut state.registrations)
            };
            for (_, registration) in retired {
                wake_registration(registration);
            }
            return;
        }
        if descriptors[0].revents != 0 {
            let mut bytes = [0; 256];
            for _ in 0..32 {
                if !control.read(&mut bytes).is_ok_and(|length| length != 0) {
                    break;
                }
            }
        }
        let now = Instant::now();
        let retired: Vec<_> = {
            let mut state = shared.state.lock().unwrap();
            snapshot
                .iter()
                .zip(&descriptors[1..])
                .filter_map(|((id, _, _, deadline), descriptor)| {
                    if descriptor.revents != 0 || *deadline <= now {
                        state.registrations.remove(id)
                    } else {
                        None
                    }
                })
                .collect()
        };
        for registration in retired {
            wake_registration(registration);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{net::TcpListener, sync::mpsc, task::Wake};
    struct Notify(mpsc::Sender<()>);
    impl Wake for Notify {
        fn wake(self: Arc<Self>) {
            let _ = self.0.send(());
        }
        fn wake_by_ref(self: &Arc<Self>) {
            let _ = self.0.send(());
        }
    }
    fn pair() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let stream = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let peer = listener.accept().unwrap().0;
        peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        (stream, peer)
    }
    fn complete<F: Future + Unpin>(future: &mut F) -> F::Output {
        let (tx, rx) = mpsc::channel();
        let waker = Waker::from(Arc::new(Notify(tx)));
        let mut cx = Context::from_waker(&waker);
        loop {
            match Pin::new(&mut *future).poll(&mut cx) {
                Poll::Ready(result) => return result,
                Poll::Pending => rx.recv_timeout(Duration::from_secs(3)).unwrap(),
            }
        }
    }
    #[test]
    fn shared_write_payload_survives_sender_drop_and_releases_on_completion_or_cancel() {
        let network = NetworkIo::new().unwrap();
        let (stream, mut peer) = pair();
        let bytes: Arc<[u8]> = Arc::from(&b"shared payload"[..]);
        let owner = Arc::downgrade(&bytes);
        let mut write = network
            .write_shared(&stream, bytes.clone(), Duration::from_secs(2))
            .unwrap();
        assert!(Arc::ptr_eq(&bytes, &write.bytes));
        drop(bytes);
        assert!(owner.upgrade().is_some());
        assert_eq!(complete(&mut write).unwrap(), 14);
        let mut received = [0; 14];
        peer.read_exact(&mut received).unwrap();
        assert_eq!(&received, b"shared payload");
        assert!(owner.upgrade().is_none());

        let bytes: Arc<[u8]> = Arc::from(&b"cancelled"[..]);
        let owner = Arc::downgrade(&bytes);
        let write = network
            .write_shared(&stream, bytes, Duration::from_secs(2))
            .unwrap();
        drop(write);
        assert!(owner.upgrade().is_none());
    }
    #[test]
    fn dormant_network_operations_start_timeout_at_first_poll() {
        let network = NetworkIo::new().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut connect = network
            .connect(listener.local_addr().unwrap(), Duration::from_millis(100))
            .unwrap();
        let (stream, mut peer) = pair();
        peer.write_all(b"ready").unwrap();
        let mut read = network
            .read(&stream, 5, Duration::from_millis(100))
            .unwrap();
        let mut write = network
            .write(&stream, b"sent", Duration::from_millis(100))
            .unwrap();
        // Models a prepared operation waiting for its actor turn to commit.
        thread::sleep(Duration::from_millis(150));
        assert_eq!(network.waiting_count(), 0);
        assert!(complete(&mut connect).is_ok());
        assert_eq!(complete(&mut read).unwrap(), b"ready");
        assert_eq!(complete(&mut write).unwrap(), 4);
        let mut bytes = [0; 4];
        peer.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"sent");
        assert_eq!(network.waiting_count(), 0);
    }
    #[test]
    fn lazy_ipv4_ipv6_connect_returns_nonblocking_cloexec_socket() {
        let network = NetworkIo::new().unwrap();
        for address in ["127.0.0.1:0", "[::1]:0"] {
            let listener = TcpListener::bind(address).unwrap();
            listener.set_nonblocking(true).unwrap();
            let target = listener.local_addr().unwrap();
            let mut connect = network.connect(target, Duration::from_secs(2)).unwrap();
            assert_eq!(
                listener.accept().unwrap_err().kind(),
                io::ErrorKind::WouldBlock
            );
            let stream = complete(&mut connect).unwrap();
            assert_eq!(stream.peer_addr().unwrap(), target);
            let (mut peer, _) = listener.accept().unwrap();
            peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
            #[cfg(unix)]
            assert_ne!(
                unsafe { libc::fcntl(stream.as_raw_fd(), libc::F_GETFL) } & libc::O_NONBLOCK,
                0
            );
            #[cfg(unix)]
            assert_ne!(
                unsafe { libc::fcntl(stream.as_raw_fd(), libc::F_GETFD) } & libc::FD_CLOEXEC,
                0
            );
            let mut send = network
                .write(&stream, b"connected", Duration::from_secs(2))
                .unwrap();
            assert_eq!(complete(&mut send).unwrap(), 9);
            let mut bytes = [0; 9];
            peer.read_exact(&mut bytes).unwrap();
            assert_eq!(&bytes, b"connected");
            assert!(connect.operation.is_none());
            let unpolled = network.connect(target, Duration::from_secs(2)).unwrap();
            drop(unpolled);
            assert_eq!(
                listener.accept().unwrap_err().kind(),
                io::ErrorKind::WouldBlock
            );
        }
    }
    #[test]
    fn refused_connection_and_invalid_timeout_release_admission() {
        let network = NetworkIo::new().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let target = listener.local_addr().unwrap();
        drop(listener);
        assert!(network.connect(target, Duration::ZERO).is_err());
        assert!(network.connect(target, Duration::from_secs(121)).is_err());
        let mut connect = network.connect(target, Duration::from_secs(2)).unwrap();
        assert_eq!(
            complete(&mut connect).unwrap_err().kind(),
            io::ErrorKind::ConnectionRefused
        );
        assert!(connect.operation.is_none());
        assert_eq!(network.waiting_count(), 0);
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn pending_connect_times_out_cancels_and_recovers_when_backlog_opens() {
        let network = NetworkIo::new().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        assert_eq!(unsafe { libc::listen(listener.as_raw_fd(), 0) }, 0);
        let target = listener.local_addr().unwrap();
        let _filler = TcpStream::connect(target).unwrap();
        let (tx, _rx) = mpsc::channel();
        let waker = Waker::from(Arc::new(Notify(tx)));
        let mut cx = Context::from_waker(&waker);
        let mut connect = network.connect(target, Duration::from_secs(3)).unwrap();
        assert!(Pin::new(&mut connect).poll(&mut cx).is_pending());
        assert_eq!(network.waiting_count(), 1);
        drop(connect);
        assert_eq!(network.waiting_count(), 0);
        let mut timeout = network.connect(target, Duration::from_millis(30)).unwrap();
        assert_eq!(
            complete(&mut timeout).unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
        assert!(timeout.operation.is_none());
        assert_eq!(network.waiting_count(), 0);
        let mut accepted = network.connect(target, Duration::from_secs(3)).unwrap();
        assert!(Pin::new(&mut accepted).poll(&mut cx).is_pending());
        let _first = listener.accept().unwrap();
        let stream = complete(&mut accepted).unwrap();
        let (mut peer, _) = listener.accept().unwrap();
        peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let mut send = network
            .write(&stream, b"ok", Duration::from_secs(2))
            .unwrap();
        complete(&mut send).unwrap();
        let mut bytes = [0; 2];
        peer.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"ok");
        assert_eq!(network.waiting_count(), 0);
    }

    #[test]
    fn readiness_deadlines_eof_and_cancel_are_nonblocking() {
        let network = NetworkIo::new().unwrap();
        let (stream, mut peer) = pair();
        let mut read = network.read(&stream, 1024, Duration::from_secs(2)).unwrap();
        let (tx, rx) = mpsc::channel();
        let waker = Waker::from(Arc::new(Notify(tx)));
        let mut cx = Context::from_waker(&waker);
        assert!(Pin::new(&mut read).poll(&mut cx).is_pending());
        assert_eq!(network.waiting_count(), 1);
        peer.write_all("响应😀".as_bytes()).unwrap();
        rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(complete(&mut read).unwrap(), "响应😀".as_bytes());
        assert_eq!(network.waiting_count(), 0);
        let mut timeout = network
            .read(&stream, 128, Duration::from_millis(30))
            .unwrap();
        assert_eq!(
            complete(&mut timeout).unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
        assert_eq!(network.waiting_count(), 0);
        let mut cancelled = network.read(&stream, 128, Duration::from_secs(2)).unwrap();
        assert!(Pin::new(&mut cancelled).poll(&mut cx).is_pending());
        drop(cancelled);
        assert_eq!(network.waiting_count(), 0);
        drop(peer);
        let mut eof = network.read(&stream, 128, Duration::from_secs(2)).unwrap();
        assert!(complete(&mut eof).unwrap().is_empty());
    }
    #[test]
    fn custom_waker_clone_is_unlocked_and_fault_does_not_poison_shared_service() {
        use std::task::{RawWaker, RawWakerVTable};
        struct Probe {
            shared: std::sync::Weak<Shared>,
            fail: bool,
        }
        unsafe fn clone(data: *const ()) -> RawWaker {
            let probe = unsafe { &*data.cast::<Probe>() };
            let shared = probe.shared.upgrade().unwrap();
            // Other threads may briefly own the lock. Repeated attempts distinguish
            // that contention from polling while holding the same lock ourselves.
            let deadline = Instant::now() + Duration::from_secs(1);
            loop {
                if shared.state.try_lock().is_ok() {
                    break;
                }
                assert!(
                    Instant::now() < deadline,
                    "waker clone runs under registry lock"
                );
                std::thread::yield_now();
            }
            assert!(!probe.fail, "injected waker clone fault");
            unsafe { Arc::increment_strong_count(data.cast::<Probe>()) };
            RawWaker::new(data, &VTABLE)
        }
        unsafe fn release(data: *const ()) {
            drop(unsafe { Arc::from_raw(data.cast::<Probe>()) });
        }
        unsafe fn wake_by_ref(_: *const ()) {}
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, release, wake_by_ref, release);
        for (fail, replacing) in [(true, false), (true, true), (false, true)] {
            let network = NetworkIo::new().unwrap();
            let (stream, mut peer) = pair();
            let probe = Arc::new(Probe {
                shared: Arc::downgrade(&network.0.shared),
                fail,
            });
            // One Arc ownership share per raw waker; clone adds and drop/wake releases it.
            let waker =
                unsafe { Waker::from_raw(RawWaker::new(Arc::into_raw(probe).cast(), &VTABLE)) };
            let mut read = network.read(&stream, 64, Duration::from_secs(2)).unwrap();
            if replacing {
                let (sender, _receiver) = mpsc::channel();
                let old = Waker::from(Arc::new(Notify(sender)));
                assert!(
                    Pin::new(&mut read)
                        .poll(&mut Context::from_waker(&old))
                        .is_pending()
                );
                assert_eq!(network.waiting_count(), 1);
            }
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                Pin::new(&mut read).poll(&mut Context::from_waker(&waker))
            }));
            let healthy = network.0.shared.state.lock().is_ok();
            // Let the failing pre-fix regression dispose its futures without a
            // second destructor panic. Production code does not clear poisoning.
            if !healthy {
                network.0.shared.state.clear_poison();
            }
            assert!(healthy, "clone fault poisoned shared readiness service");
            if fail {
                assert!(result.is_err());
            } else {
                assert!(result.unwrap().is_pending());
            }
            drop(read);
            assert_eq!(network.waiting_count(), 0);
            // An unrelated connection still progresses on the same reactor.
            let (other, mut other_peer) = pair();
            let mut next = network.read(&other, 64, Duration::from_secs(2)).unwrap();
            other_peer.write_all(b"healthy").unwrap();
            assert_eq!(complete(&mut next).unwrap(), b"healthy");
            peer.write_all(b"retry").unwrap();
            let mut retry = network.read(&stream, 64, Duration::from_secs(2)).unwrap();
            assert_eq!(complete(&mut retry).unwrap(), b"retry");
        }
    }
    #[test]
    fn retained_completed_write_releases_buffer_socket_and_reactor() {
        let network = NetworkIo::new().unwrap();
        let weak = Arc::downgrade(&network.0);
        let (stream, mut peer) = pair();
        let mut write = network
            .write(&stream, b"done", Duration::from_secs(2))
            .unwrap();
        drop(stream);
        assert_eq!(complete(&mut write).unwrap(), 4);
        assert_eq!(write.bytes.len(), 0);
        drop(network);
        assert!(
            weak.upgrade().is_none(),
            "completed future retains readiness worker"
        );
        let mut bytes = [0; 4];
        peer.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"done");
        assert_eq!(peer.read(&mut bytes).unwrap(), 0);
        assert!(complete(&mut write).is_err());
    }
    #[test]
    fn retained_completed_reads_release_socket_reactor_and_error_buffer() {
        for mode in 0..3 {
            let network = NetworkIo::new().unwrap();
            let weak = Arc::downgrade(&network.0);
            let (stream, mut peer) = pair();
            let mut read = network
                .read(
                    &stream,
                    MAX_BYTES,
                    if mode == 1 {
                        Duration::from_millis(30)
                    } else {
                        Duration::from_secs(2)
                    },
                )
                .unwrap();
            drop(stream);
            if mode == 1 {
                assert_eq!(
                    complete(&mut read).unwrap_err().kind(),
                    io::ErrorKind::TimedOut
                );
            } else if mode == 2 {
                peer.shutdown(std::net::Shutdown::Write).unwrap();
                assert!(complete(&mut read).unwrap().is_empty());
            } else {
                peer.write_all(b"ok").unwrap();
                assert_eq!(complete(&mut read).unwrap(), b"ok");
            }
            assert_eq!(read.bytes.capacity(), 0);
            drop(network);
            assert!(
                weak.upgrade().is_none(),
                "completed read retains readiness worker"
            );
            let mut bytes = [0];
            assert_eq!(peer.read(&mut bytes).unwrap(), 0);
            assert!(complete(&mut read).is_err());
        }
    }
    #[test]
    fn sends_complete_bytes_and_reports_partial_progress_on_timeout() {
        let network = NetworkIo::new().unwrap();
        #[cfg(unix)]
        let (stream, mut peer) = pair();
        #[cfg(windows)]
        let (stream, mut peer) = {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            socket2::SockRef::from(&listener)
                .set_recv_buffer_size(4096)
                .unwrap();
            let stream = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
            let peer = listener.accept().unwrap().0;
            peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
            (stream, peer)
        };
        let mut write = network
            .write(&stream, "hello\n🦀".as_bytes(), Duration::from_secs(2))
            .unwrap();
        assert_eq!(complete(&mut write).unwrap(), "hello\n🦀".len());
        let mut bytes = vec![0; "hello\n🦀".len()];
        peer.read_exact(&mut bytes).unwrap();
        assert_eq!(bytes, "hello\n🦀".as_bytes());
        // Keep the peer alive without consuming its bytes; make saturation deterministic.
        #[cfg(unix)]
        {
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
        }
        #[cfg(windows)]
        socket2::SockRef::from(&stream)
            .set_send_buffer_size(4096)
            .unwrap();
        #[cfg(windows)]
        {
            // Winsock may accept a complete large write despite a small SO_SNDBUF.
            // Fill the actual send queue before testing the timeout contract.
            stream.set_nonblocking(true).unwrap();
            let mut saturated = false;
            for _ in 0..1024 {
                match (&stream).write(&[0; 65536]) {
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        saturated = true;
                        break;
                    }
                    Ok(_) => {}
                    Err(error) => panic!("send queue preparation failed: {error}"),
                }
            }
            assert!(saturated, "send queue did not saturate within 64 MiB");
        }
        let mut blocked = network
            .write(&stream, &vec![1; MAX_BYTES], Duration::from_millis(50))
            .unwrap();
        let payload = Arc::downgrade(&blocked.bytes);
        let error = complete(&mut blocked).unwrap_err();
        assert_eq!(error.error.kind(), io::ErrorKind::TimedOut);
        assert!(error.sent < MAX_BYTES);
        #[cfg(unix)]
        assert!(error.sent > 0);
        assert_eq!(blocked.bytes.len(), 0);
        assert!(payload.upgrade().is_none(), "timeout retains write payload");
        assert!(blocked.operation.is_none());
        assert_eq!(network.waiting_count(), 0);
    }
    #[test]
    fn independent_connections_share_readiness_and_isolate_panicking_wakers() {
        struct Panics;
        impl Wake for Panics {
            fn wake(self: Arc<Self>) {
                panic!("injected host waker fault");
            }
        }
        let network = NetworkIo::new().unwrap();
        let (stream, mut peer) = pair();
        let mut faulty = network.read(&stream, 1, Duration::from_secs(2)).unwrap();
        let waker = Waker::from(Arc::new(Panics));
        assert!(
            Pin::new(&mut faulty)
                .poll(&mut Context::from_waker(&waker))
                .is_pending()
        );
        peer.write_all(b"x").unwrap();
        // Wait for retirement to ensure the failure precedes the other sockets.
        let deadline = Instant::now() + Duration::from_secs(2);
        while network.waiting_count() != 0 && Instant::now() < deadline {
            thread::yield_now();
        }
        assert_eq!(network.waiting_count(), 0);
        drop(faulty);
        let clone = network.clone();
        assert_eq!(
            network.0.thread.as_ref().unwrap().thread().id(),
            clone.0.thread.as_ref().unwrap().thread().id()
        );
        let (tx, _rx) = mpsc::channel();
        let waker = Waker::from(Arc::new(Notify(tx)));
        let mut cx = Context::from_waker(&waker);
        let mut connections = Vec::new();
        for index in 0..64u8 {
            let (stream, peer) = pair();
            let mut read = clone.read(&stream, 1, Duration::from_secs(2)).unwrap();
            assert!(Pin::new(&mut read).poll(&mut cx).is_pending());
            connections.push((index, read, peer));
        }
        assert_eq!(network.waiting_count(), 64);
        for (index, _, peer) in &mut connections {
            peer.write_all(&[*index]).unwrap();
        }
        for (index, read, _) in &mut connections {
            assert_eq!(complete(read).unwrap(), vec![*index]);
        }
        assert_eq!(network.waiting_count(), 0);
    }

    #[test]
    fn bounded_admission_shared_worker_and_prompt_shutdown() {
        let network = NetworkIo::new().unwrap();
        let (stream, _peer) = pair();
        assert!(network.read(&stream, 0, Duration::from_secs(1)).is_err());
        assert!(
            network
                .read(&stream, MAX_BYTES + 1, Duration::from_secs(1))
                .is_err()
        );
        assert!(network.read(&stream, 1, Duration::ZERO).is_err());
        assert!(
            network
                .write(&stream, b"x", Duration::from_secs(121))
                .is_err()
        );
        assert!(
            network
                .write(&stream, &vec![0; MAX_BYTES + 1], Duration::from_secs(1))
                .is_err()
        );
        let clone = network.clone();
        let worker = network.0.thread.as_ref().unwrap().thread().id();
        assert_eq!(clone.0.thread.as_ref().unwrap().thread().id(), worker);
        let (tx, _rx) = mpsc::channel();
        let waker = Waker::from(Arc::new(Notify(tx)));
        let mut cx = Context::from_waker(&waker);
        let mut reads = Vec::new();
        for _ in 0..MAX_WAITERS {
            let mut read = clone.read(&stream, 1, Duration::from_secs(2)).unwrap();
            assert!(Pin::new(&mut read).poll(&mut cx).is_pending());
            reads.push(read);
        }
        let mut extra = clone.read(&stream, 1, Duration::from_secs(2)).unwrap();
        assert!(matches!(
            Pin::new(&mut extra).poll(&mut cx),
            Poll::Ready(Err(_))
        ));
        assert_eq!(network.waiting_count(), MAX_WAITERS);
        drop(reads);
        assert_eq!(network.waiting_count(), 0);
        drop(extra);
        drop(clone);
        let shutdown = Instant::now();
        drop(network);
        assert!(shutdown.elapsed() < Duration::from_secs(1));
    }
}
