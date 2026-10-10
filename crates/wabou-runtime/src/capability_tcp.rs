//! Resident TCP resources; modules exchange typed keys and copied chunks.
use crate::{capability_resources::CapabilityResources, capability_tasks::CapabilityTasks};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::Mutex,
};
use tokio_util::sync::CancellationToken;
use wabou_host_api::{ConnectionKey, TcpConnection};

struct Connection {
    read: Arc<Mutex<OwnedReadHalf>>,
    write: Arc<Mutex<OwnedWriteHalf>>,
    closed: CancellationToken,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wabou_host_api::TaskKey;
    async fn terminal(tasks: &CapabilityTasks, key: TaskKey) -> serde_json::Value {
        let request = serde_json::to_vec(&key).unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(3), async {
            loop {
                let value: serde_json::Value =
                    serde_json::from_slice(&tasks.call("task.poll", &request).unwrap()).unwrap();
                if value["status"] != "pending" {
                    tasks.call("task.release", &request).unwrap();
                    return value;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap()
    }
    #[tokio::test]
    async fn connection_can_write_while_read_waits_and_close_cancels_pending_io() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let (finish, finished) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut byte = [0];
            stream.read_exact(&mut byte).await.unwrap();
            stream.write_all(&byte).await.unwrap();
            finished.await.unwrap();
        });
        let tasks = CapabilityTasks::new(tokio::runtime::Handle::current());
        let tcp = CapabilityTcp::new(tasks.clone());
        let connect: TaskKey = serde_json::from_slice(
            &tcp.call("tcp.connect", &serde_json::to_vec(&address).unwrap())
                .unwrap(),
        )
        .unwrap();
        let response = terminal(&tasks, connect).await;
        assert_eq!(response["status"], "ready");
        let connection: ConnectionKey = serde_json::from_value(response["value"].clone()).unwrap();
        let request =
            serde_json::to_vec(&serde_json::json!({"resource":connection,"request":1})).unwrap();
        let read = serde_json::from_slice(&tcp.call("tcp.read", &request).unwrap()).unwrap();
        let write_request =
            serde_json::to_vec(&serde_json::json!({"resource":connection,"request":[65]})).unwrap();
        let write =
            serde_json::from_slice(&tcp.call("tcp.write", &write_request).unwrap()).unwrap();
        assert_eq!(terminal(&tasks, write).await["value"], 1);
        assert_eq!(
            terminal(&tasks, read).await["value"],
            serde_json::json!([65])
        );
        let pending = serde_json::from_slice(&tcp.call("tcp.read", &request).unwrap()).unwrap();
        tcp.call("tcp.close", &serde_json::to_vec(&connection).unwrap())
            .unwrap();
        let cancelled = terminal(&tasks, pending).await;
        assert_eq!(cancelled["status"], "failed");
        assert_eq!(cancelled["error"], "connection closed");
        assert!(tcp.call("tcp.read", &request).is_err());
        finish.send(()).unwrap();
        server.await.unwrap();
    }
}
impl Drop for Connection {
    fn drop(&mut self) {
        self.closed.cancel();
    }
}
/// One namespace of connections. Read/write halves lock independently, allowing
/// writes while a read waits. Dropping the final provider closes its connections
/// and signals pending I/O; the resident executor performs cooperative cleanup.
#[derive(Clone)]
pub struct CapabilityTcp {
    connections: CapabilityResources<TcpConnection, Connection>,
    tasks: CapabilityTasks,
}
impl CapabilityTcp {
    /// Share the resident task service used to query I/O results.
    pub fn new(tasks: CapabilityTasks) -> Self {
        Self {
            connections: CapabilityResources::default(),
            tasks,
        }
    }
    /// Connect/read/write return task handles. Close invalidates a connection
    /// and cancels pending I/O. Reads/writes can be partial; retry only the
    /// remaining bytes after a successful write result. Chunks are at most 64 KiB.
    pub fn call(&self, operation: &str, request: &[u8]) -> Result<Vec<u8>, String> {
        let value = match operation {
            "tcp.connect" => {
                let address: String = serde_json::from_slice(request).map_err(|e| e.to_string())?;
                let connections = self.connections.clone();
                serde_json::to_value(self.tasks.spawn(async move {
                    let stream = TcpStream::connect(address)
                        .await
                        .map_err(|e| e.to_string())?;
                    let (read, write) = stream.into_split();
                    connections.insert(Connection {
                        read: Arc::new(Mutex::new(read)),
                        write: Arc::new(Mutex::new(write)),
                        closed: CancellationToken::new(),
                    })
                })?)
                .map_err(|e| e.to_string())?
            }
            "tcp.read" | "tcp.write" => {
                let request: serde_json::Value =
                    serde_json::from_slice(request).map_err(|e| e.to_string())?;
                let key: ConnectionKey = serde_json::from_value(request["resource"].clone())
                    .map_err(|e| e.to_string())?;
                let (read, write, closed) = self.connections.with(key, |connection| {
                    Ok((
                        connection.read.clone(),
                        connection.write.clone(),
                        connection.closed.clone(),
                    ))
                })?;
                let key = if operation == "tcp.read" {
                    let length = request["request"]
                        .as_u64()
                        .ok_or("tcp.read expects a byte count")?;
                    if length > 65536 {
                        return Err("tcp.read chunk exceeds 64 KiB".into());
                    }
                    self.tasks.spawn(async move {
                        tokio::select! {
                            biased;
                            _ = closed.cancelled() => Err("connection closed".into()),
                            result = async {
                                let mut bytes = vec![0; length as usize];
                                let read = read.lock().await.read(&mut bytes).await.map_err(|e| e.to_string())?;
                                bytes.truncate(read);
                                Ok(bytes)
                            } => result,
                        }
                    })?
                } else {
                    let bytes: Vec<u8> = serde_json::from_value(request["request"].clone())
                        .map_err(|e| e.to_string())?;
                    if bytes.len() > 65536 {
                        return Err("tcp.write chunk exceeds 64 KiB".into());
                    }
                    self.tasks.spawn(async move {
                        tokio::select! {
                            biased;
                            _ = closed.cancelled() => Err("connection closed".into()),
                            result = async { write.lock().await.write(&bytes).await.map_err(|e| e.to_string()) } => result,
                        }
                    })?
                };
                serde_json::to_value(key).map_err(|e| e.to_string())?
            }
            "tcp.close" => {
                let key: ConnectionKey =
                    serde_json::from_slice(request).map_err(|e| e.to_string())?;
                drop(self.connections.remove(key)?);
                serde_json::Value::Null
            }
            _ => return Err(format!("unknown TCP operation: {operation}")),
        };
        serde_json::to_vec(&value).map_err(|e| e.to_string())
    }
}
