//! Browser builds cannot use raw TCP. Reject admission before starting host work.
use std::{
    future::Ready,
    io,
    net::{SocketAddr, TcpStream},
    sync::Arc,
    time::Duration,
};
#[derive(Clone)]
pub struct NetworkIo;
pub type SocketConnect = Ready<io::Result<TcpStream>>;
pub type SocketRead = Ready<io::Result<Vec<u8>>>;
pub type SocketWrite = Ready<Result<usize, SocketWriteError>>;
#[derive(Debug)]
pub struct SocketWriteError {
    pub sent: usize,
    pub error: io::Error,
}
impl std::fmt::Display for SocketWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}
impl std::error::Error for SocketWriteError {}
fn unsupported() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "raw TCP is unavailable in browser builds",
    )
}
impl NetworkIo {
    pub fn new() -> io::Result<Self> {
        Err(unsupported())
    }
    pub fn connect(&self, _: SocketAddr, _: Duration) -> io::Result<SocketConnect> {
        Err(unsupported())
    }
    pub fn read(&self, _: &TcpStream, _: usize, _: Duration) -> io::Result<SocketRead> {
        Err(unsupported())
    }
    pub fn write(&self, _: &TcpStream, _: &[u8], _: Duration) -> io::Result<SocketWrite> {
        Err(unsupported())
    }
    pub(crate) fn write_shared(
        &self,
        _: &TcpStream,
        _: Arc<[u8]>,
        _: Duration,
    ) -> io::Result<SocketWrite> {
        Err(unsupported())
    }
    pub fn waiting_count(&self) -> usize {
        0
    }
}
