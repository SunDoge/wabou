//! Stable-host sockets. Connections contain no implementation-owned callback.
use std::{collections::BTreeMap, net::TcpStream, sync::Arc};

/// Reload policy for a socket registered by the stable executable.
/// Reconnecting needs application-specific protocol setup and is never implicit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SocketReloadPolicy {
    #[default]
    Preserve,
    Cancel,
}
#[derive(Clone)]
struct Socket {
    stream: Arc<TcpStream>,
    policy: SocketReloadPolicy,
}
#[derive(Clone, Default)]
pub(crate) struct Sockets(BTreeMap<u64, Socket>);
impl Sockets {
    pub fn insert(
        &mut self,
        id: u64,
        stream: TcpStream,
        policy: SocketReloadPolicy,
    ) -> Result<(), String> {
        stream
            .set_nonblocking(true)
            .map_err(|error| error.to_string())?;
        self.insert_arc(id, Arc::new(stream), policy)
    }
    pub fn insert_arc(
        &mut self,
        id: u64,
        stream: Arc<TcpStream>,
        policy: SocketReloadPolicy,
    ) -> Result<(), String> {
        if id == 0 {
            return Err("socket identity must be nonzero".into());
        }
        if self.0.contains_key(&id) {
            return Err("duplicate socket identity".into());
        }
        if self.0.len() >= 64 {
            return Err("socket resource limit reached".into());
        }
        self.0.insert(id, Socket { stream, policy });
        Ok(())
    }
    pub fn get(&self, id: u64) -> Option<&TcpStream> {
        self.0.get(&id).map(|socket| socket.stream.as_ref())
    }
    pub fn handle(&self, id: u64) -> Option<Arc<TcpStream>> {
        self.0.get(&id).map(|socket| socket.stream.clone())
    }
    pub fn remove(&mut self, id: u64) -> Option<Arc<TcpStream>> {
        self.0.remove(&id).map(|socket| socket.stream)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn commit_reload(&mut self) {
        self.0
            .retain(|_, socket| socket.policy == SocketReloadPolicy::Preserve);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Read, net::TcpListener, time::Duration};
    #[allow(clippy::unnecessary_cast)] // RawSocket is usize-sized on supported Windows targets.
    fn raw(stream: &TcpStream) -> u64 {
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            stream.as_raw_fd() as u64
        }
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawSocket;
            stream.as_raw_socket() as u64
        }
    }
    fn pair() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        (client, listener.accept().unwrap().0)
    }
    #[test]
    fn identities_and_limits_preserve_existing_connections() {
        let mut sockets = Sockets::default();
        let (stream, _peer) = pair();
        let fd = raw(&stream);
        sockets
            .insert(1, stream, SocketReloadPolicy::Preserve)
            .unwrap();
        assert!(
            sockets
                .insert(0, pair().0, SocketReloadPolicy::Preserve)
                .is_err()
        );
        assert!(
            sockets
                .insert(1, pair().0, SocketReloadPolicy::Cancel)
                .is_err()
        );
        assert_eq!(raw(sockets.get(1).unwrap()), fd);
        let source = sockets.get(1).unwrap().try_clone().unwrap();
        for id in 2..=64 {
            sockets
                .insert(
                    id,
                    source.try_clone().unwrap(),
                    SocketReloadPolicy::Preserve,
                )
                .unwrap();
        }
        assert!(
            sockets
                .insert(
                    65,
                    source.try_clone().unwrap(),
                    SocketReloadPolicy::Preserve
                )
                .is_err()
        );
        assert_eq!(sockets.len(), 64);
        drop(sockets.remove(1).unwrap());
        sockets
            .insert(65, source, SocketReloadPolicy::Preserve)
            .unwrap();
        assert_eq!(sockets.len(), 64);
    }
    #[test]
    fn commit_drops_cancel_connections_and_preserves_descriptors() {
        let mut sockets = Sockets::default();
        let (preserved, _peer) = pair();
        let fd = raw(&preserved);
        let (cancelled, mut peer) = pair();
        peer.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        sockets
            .insert(1, preserved, SocketReloadPolicy::Preserve)
            .unwrap();
        sockets
            .insert(2, cancelled, SocketReloadPolicy::Cancel)
            .unwrap();
        sockets.commit_reload();
        assert_eq!(sockets.len(), 1);
        assert_eq!(raw(sockets.get(1).unwrap()), fd);
        assert!(sockets.get(2).is_none());
        assert_eq!(peer.read(&mut [0; 1]).unwrap(), 0);
    }
}
