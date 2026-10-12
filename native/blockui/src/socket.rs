//! Data-only socket/task descriptors. Native handles and futures stay in the host.
use crate::{
    Action, BindState, BoundResource, Cx, Error, ReloadPolicy, Resource, Result, Task, abi,
};
/// Runtime-owned TCP connection identity; never a native handle in actor state.
pub enum Socket {}
impl BindState for Resource<Socket> {
    type Bound<'cx, 'host>
        = BoundResource<'cx, 'host, Socket>
    where
        'host: 'cx;
    fn bind<'cx, 'host>(&self, cx: &'cx Cx<'host>) -> Result<Self::Bound<'cx, 'host>> {
        if matches!(self.policy, ReloadPolicy::Restart) {
            return Err(Error::Application(
                "socket resources do not support Restart".into(),
            ));
        }
        Ok(BoundResource::new(*self, cx))
    }
}
impl Resource<Socket> {
    /// Close registry ownership after a successful turn. Failed turns retain it.
    /// Independently admitted tasks retain their captured connection; cancel them
    /// explicitly to stop their I/O. A pending connect must be cancelled by Task.
    pub fn close(self, cx: &Cx<'_>) -> Result<()> {
        let code = unsafe { (cx.api.close_socket)(cx.api.context, self.id) };
        status(code)
    }
}
fn status(code: i32) -> Result<()> {
    if code == abi::OK {
        Ok(())
    } else {
        Err(code.into())
    }
}
fn millis(timeout: std::time::Duration) -> Result<u64> {
    if timeout.is_zero() || timeout > std::time::Duration::from_secs(120) {
        return Err(Error::Host(abi::ERROR));
    }
    crate::timer_millis(timeout)
}
impl Task {
    /// Connect to a concrete IPv4/IPv6 address with a nonzero port after commit.
    /// No DNS, native handle or Rust future crosses the implementation boundary.
    /// The timeout starts when the committed operation is first polled, not while
    /// its transaction is being prepared.
    /// Both task and resource support Preserve/Cancel; Restart is rejected.
    /// Completion receives this task's key; `result` is empty on connection success
    /// and the resource is registered before invoking the current-code callback.
    pub fn connect_socket(
        self,
        cx: &Cx<'_>,
        socket: Resource<Socket>,
        address: &str,
        timeout: std::time::Duration,
        completion: Action,
    ) -> Result<()> {
        if matches!(self.policy, ReloadPolicy::Restart)
            || matches!(socket.policy, ReloadPolicy::Restart)
        {
            return Err(Error::Host(abi::ERROR));
        }
        let millis = millis(timeout)?;
        let code = unsafe {
            (cx.api.connect_socket_async)(
                cx.api.context,
                self.id,
                socket.id,
                abi::Bytes::new(address),
                millis,
                completion.id,
                self.policy as u32,
                socket.policy as u32,
            )
        };
        status(code)
    }
    /// Read one TCP byte chunk after commit (1..=1MiB); empty result means EOF.
    /// Framing/UTF-8 decoding belongs to the application. Already received bytes
    /// cannot be restored by cancellation or a rejecting completion handler.
    pub fn read_socket(
        self,
        cx: &Cx<'_>,
        socket: Resource<Socket>,
        limit: usize,
        timeout: std::time::Duration,
        completion: Action,
    ) -> Result<()> {
        if matches!(self.policy, ReloadPolicy::Restart) {
            return Err(Error::Host(abi::ERROR));
        }
        let code = unsafe {
            (cx.api.read_socket_async)(
                cx.api.context,
                self.id,
                socket.id,
                limit,
                millis(timeout)?,
                completion.id,
                self.policy as u32,
            )
        };
        status(code)
    }
    /// Send the complete admitted buffer after commit (at most 1MiB).
    /// Success is an empty task result. Errors can follow partial sends; the host
    /// error text includes the sent count. Network effects are not rolled back.
    /// The count records bytes accepted by the local socket, not remote application
    /// acknowledgement. It is diagnostic text, not a structured retry protocol.
    /// Staged write payloads are bounded to 16MiB per turn, before copying;
    /// prepared/live buffers have a separate shared budget.
    pub fn write_socket(
        self,
        cx: &Cx<'_>,
        socket: Resource<Socket>,
        bytes: &[u8],
        timeout: std::time::Duration,
        completion: Action,
    ) -> Result<()> {
        if matches!(self.policy, ReloadPolicy::Restart) {
            return Err(Error::Host(abi::ERROR));
        }
        let code = unsafe {
            (cx.api.write_socket_async)(
                cx.api.context,
                self.id,
                socket.id,
                abi::Bytes {
                    ptr: bytes.as_ptr(),
                    len: bytes.len(),
                },
                millis(timeout)?,
                completion.id,
                self.policy as u32,
            )
        };
        status(code)
    }
}
impl BoundResource<'_, '_, Socket> {
    pub fn close(&self) -> Result<()> {
        self.resource.close(self.cx)
    }
}
