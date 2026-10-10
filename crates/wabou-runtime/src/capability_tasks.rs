//! Resident Tokio tasks; replaceable code observes only copied results and keys.
use crate::capability_resources::CapabilityResources;
use std::{
    future::Future,
    sync::{Arc, Mutex},
};
use wabou_host_api::{BackgroundTask, TaskKey};

struct Job {
    abort: tokio::task::AbortHandle,
    result: Arc<Mutex<Option<Result<serde_json::Value, String>>>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    async fn terminal(tasks: &CapabilityTasks, key: TaskKey) -> serde_json::Value {
        let request = serde_json::to_vec(&key).unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(3), async {
            loop {
                let value: serde_json::Value =
                    serde_json::from_slice(&tasks.call("task.poll", &request).unwrap()).unwrap();
                if value["status"] != "pending" {
                    return value;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap()
    }
    #[tokio::test]
    async fn completed_results_repeat_until_release_and_panics_become_errors() {
        let tasks = CapabilityTasks::new(tokio::runtime::Handle::current());
        let key = tasks.spawn(async { Ok(42u32) }).unwrap();
        let first = terminal(&tasks, key).await;
        assert_eq!(first, serde_json::json!({"status":"ready", "value":42}));
        assert_eq!(terminal(&tasks, key).await, first);
        let request = serde_json::to_vec(&key).unwrap();
        tasks.call("task.release", &request).unwrap();
        let replacement = tasks
            .spawn(async { Err::<(), _>("network failure".into()) })
            .unwrap();
        assert_ne!(key, replacement);
        assert!(tasks.call("task.poll", &request).is_err());
        assert_eq!(
            terminal(&tasks, replacement).await["error"],
            "network failure"
        );
        let panic = tasks
            .spawn(async {
                panic!("host job panicked");
                #[allow(unreachable_code)]
                Ok(())
            })
            .unwrap();
        let value = terminal(&tasks, panic).await;
        assert_eq!(value["status"], "failed");
        assert!(value["error"].as_str().unwrap().contains("panicked"));
    }
    #[tokio::test]
    async fn resident_timer_uses_the_same_task_protocol() {
        let tasks = CapabilityTasks::new(tokio::runtime::Handle::current());
        let key: TaskKey =
            serde_json::from_slice(&tasks.call("task.sleep", b"0").unwrap()).unwrap();
        assert_eq!(
            terminal(&tasks, key).await,
            serde_json::json!({"status":"ready", "value":null})
        );
        tasks
            .call("task.release", &serde_json::to_vec(&key).unwrap())
            .unwrap();
        assert!(tasks.call("task.sleep", b"-1").is_err());
    }
    struct Dropped(Option<tokio::sync::oneshot::Sender<()>>);
    impl Drop for Dropped {
        fn drop(&mut self) {
            let _ = self.0.take().unwrap().send(());
        }
    }
    #[tokio::test]
    async fn releasing_or_dropping_owner_cancels_pending_host_futures() {
        for release in [true, false] {
            let tasks = CapabilityTasks::new(tokio::runtime::Handle::current());
            let (dropped, observed) = tokio::sync::oneshot::channel();
            let guard = Dropped(Some(dropped));
            let key = tasks
                .spawn(async move {
                    let _guard = guard;
                    std::future::pending::<()>().await;
                    Ok(())
                })
                .unwrap();
            tokio::task::yield_now().await;
            if release {
                tasks
                    .call("task.release", &serde_json::to_vec(&key).unwrap())
                    .unwrap();
            }
            drop(tasks);
            tokio::time::timeout(std::time::Duration::from_secs(3), observed)
                .await
                .unwrap()
                .unwrap();
        }
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        self.abort.abort();
    }
}

/// Scheduler for work implemented in resident host code. Futures, captured
/// resources and their destructors must not originate in replaceable libraries.
/// Clones share jobs. Dropping the last scheduler aborts remaining jobs; the
/// executor must continue running to finish cooperative cancellation.
#[derive(Clone)]
pub struct CapabilityTasks {
    executor: tokio::runtime::Handle,
    jobs: CapabilityResources<BackgroundTask, Job>,
}
impl CapabilityTasks {
    /// Use a resident executor whose lifetime covers this scheduler.
    pub fn new(executor: tokio::runtime::Handle) -> Self {
        Self {
            executor,
            jobs: CapabilityResources::default(),
        }
    }
    /// Start host work. Blocking operations require their own cancellation policy;
    /// Tokio abort cannot interrupt an already-running blocking OS operation.
    pub fn spawn<T, F>(&self, future: F) -> Result<TaskKey, String>
    where
        T: serde::Serialize + Send + 'static,
        F: Future<Output = Result<T, String>> + Send + 'static,
    {
        let result = Arc::new(Mutex::new(None));
        let output = result.clone();
        let task = self.executor.spawn(async move {
            future
                .await
                .and_then(|value| serde_json::to_value(value).map_err(|e| e.to_string()))
        });
        let abort = task.abort_handle();
        self.executor.spawn(async move {
            let value = task
                .await
                .unwrap_or_else(|error| Err(format!("background task failed: {error}")));
            if let Ok(mut output) = output.lock() {
                *output = Some(value);
            }
        });
        self.jobs.insert(Job { abort, result })
    }
    /// Poll or release a handle in this scheduler's namespace. Results are copied;
    /// polling is repeatable, and callers explicitly release finished jobs.
    pub fn call(&self, operation: &str, request: &[u8]) -> Result<Vec<u8>, String> {
        if operation == "task.sleep" {
            let milliseconds: u64 = serde_json::from_slice(request).map_err(|e| e.to_string())?;
            let key = self.spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(milliseconds)).await;
                Ok(())
            })?;
            return serde_json::to_vec(&key).map_err(|e| e.to_string());
        }
        let key: TaskKey = serde_json::from_slice(request).map_err(|e| e.to_string())?;
        let response = match operation {
            "task.poll" => self.jobs.with(key, |job| {
                let result = job.result.lock().map_err(|e| e.to_string())?;
                Ok(match result.as_ref() {
                    None => serde_json::json!({"status": "pending"}),
                    Some(Ok(value)) => serde_json::json!({"status": "ready", "value": value}),
                    Some(Err(error)) => serde_json::json!({"status": "failed", "error": error}),
                })
            })?,
            "task.release" => {
                drop(self.jobs.remove(key)?);
                serde_json::Value::Null
            }
            _ => return Err(format!("unknown task operation: {operation}")),
        };
        serde_json::to_vec(&response).map_err(|e| e.to_string())
    }
}
