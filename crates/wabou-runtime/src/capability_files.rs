//! Resident file handles and asynchronous reads, independent of module code.
use crate::{capability_resources::CapabilityResources, capability_tasks::CapabilityTasks};
use std::{path::PathBuf, sync::Arc};
use tokio::{io::AsyncReadExt, sync::Mutex};
use wabou_host_api::{FileKey, OpenFile};

/// File provider for one capability. Relative paths use its configured base;
/// this is path resolution, not a filesystem security sandbox.
#[derive(Clone)]
pub struct CapabilityFiles {
    base: PathBuf,
    files: CapabilityResources<OpenFile, Arc<Mutex<tokio::fs::File>>>,
    tasks: CapabilityTasks,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wabou_host_api::TaskKey;
    async fn result<T: serde::de::DeserializeOwned>(tasks: &CapabilityTasks, key: TaskKey) -> T {
        let request = serde_json::to_vec(&key).unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(3), async {
            loop {
                let value: serde_json::Value =
                    serde_json::from_slice(&tasks.call("task.poll", &request).unwrap()).unwrap();
                match value["status"].as_str().unwrap() {
                    "ready" => {
                        tasks.call("task.release", &request).unwrap();
                        return serde_json::from_value(value["value"].clone()).unwrap();
                    }
                    "failed" => panic!("file task failed: {value}"),
                    _ => tokio::task::yield_now().await,
                }
            }
        })
        .await
        .unwrap()
    }
    #[tokio::test]
    async fn resident_file_cursor_and_inflight_read_survive_handle_close() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("document.txt"), b"AB").unwrap();
        let tasks = CapabilityTasks::new(tokio::runtime::Handle::current());
        let files = CapabilityFiles::new(root.path().to_owned(), tasks.clone());
        let open = serde_json::from_slice(&files.call("file.open", br#""document.txt""#).unwrap())
            .unwrap();
        let file: FileKey = result(&tasks, open).await;
        let read = serde_json::to_vec(&serde_json::json!({"resource":file,"request":1})).unwrap();
        let task = serde_json::from_slice(&files.call("file.read", &read).unwrap()).unwrap();
        assert_eq!(result::<Vec<u8>>(&tasks, task).await, b"A");
        let second = serde_json::from_slice(&files.call("file.read", &read).unwrap()).unwrap();
        files
            .call("file.close", &serde_json::to_vec(&file).unwrap())
            .unwrap();
        assert_eq!(result::<Vec<u8>>(&tasks, second).await, b"B");
        assert!(files.call("file.read", &read).is_err());
        assert!(
            files
                .call("file.close", &serde_json::to_vec(&file).unwrap())
                .is_err()
        );
        let too_large =
            serde_json::to_vec(&serde_json::json!({"resource":file,"request":65537})).unwrap();
        assert!(
            files
                .call("file.read", &too_large)
                .unwrap_err()
                .contains("64 KiB")
        );
    }
}
impl CapabilityFiles {
    /// Use the same resident task service that modules poll for results.
    pub fn new(base: PathBuf, tasks: CapabilityTasks) -> Self {
        Self {
            base,
            files: CapabilityResources::default(),
            tasks,
        }
    }
    /// Open/read return task handles; close invalidates a file handle immediately.
    /// In-flight reads retain the file until completion. Reads serialize on each
    /// file and advance its resident cursor; chunks are capped at 64 KiB for JSON.
    pub fn call(&self, operation: &str, request: &[u8]) -> Result<Vec<u8>, String> {
        let response = match operation {
            "file.open" => {
                let path: PathBuf = serde_json::from_slice(request).map_err(|e| e.to_string())?;
                let path = if path.is_absolute() {
                    path
                } else {
                    self.base.join(path)
                };
                let files = self.files.clone();
                serde_json::to_value(self.tasks.spawn(async move {
                    let file = tokio::fs::File::open(path)
                        .await
                        .map_err(|e| e.to_string())?;
                    files.insert(Arc::new(Mutex::new(file)))
                })?)
                .map_err(|e| e.to_string())?
            }
            "file.read" => {
                let request: serde_json::Value =
                    serde_json::from_slice(request).map_err(|e| e.to_string())?;
                let key: FileKey = serde_json::from_value(request["resource"].clone())
                    .map_err(|e| e.to_string())?;
                let length = request["request"]
                    .as_u64()
                    .ok_or("file.read expects a byte count")?;
                if length > 65536 {
                    return Err("file.read chunk exceeds 64 KiB".into());
                }
                let file = self.files.with(key, |file| Ok(file.clone()))?;
                serde_json::to_value(self.tasks.spawn(async move {
                    let mut bytes = vec![0; length as usize];
                    let read = file
                        .lock()
                        .await
                        .read(&mut bytes)
                        .await
                        .map_err(|e| e.to_string())?;
                    bytes.truncate(read);
                    Ok(bytes)
                })?)
                .map_err(|e| e.to_string())?
            }
            "file.close" => {
                let key: FileKey = serde_json::from_slice(request).map_err(|e| e.to_string())?;
                drop(self.files.remove(key)?);
                serde_json::Value::Null
            }
            _ => return Err(format!("unknown file operation: {operation}")),
        };
        serde_json::to_vec(&response).map_err(|e| e.to_string())
    }
}
