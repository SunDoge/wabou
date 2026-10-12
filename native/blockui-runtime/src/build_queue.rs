//! One replaceable result per watcher; obsolete DLLs are disposed on replacement.
use super::native::BuildResult;
use std::sync::{Arc, Mutex, mpsc};

#[derive(Default)]
struct Slot {
    closed: bool,
    result: Option<BuildResult>,
}
#[derive(Clone)]
pub(super) struct Sender(Arc<Mutex<Slot>>);
pub(super) enum Receiver {
    Latest(Sender),
    Channel(mpsc::Receiver<BuildResult>),
}
fn dispose(result: Option<BuildResult>) {
    if let Some(BuildResult {
        artifact: Some(path),
        ..
    }) = result
    {
        let _ = std::fs::remove_file(path);
    }
}
pub(super) fn channel() -> (Sender, Receiver) {
    let sender = Sender(Arc::new(Mutex::new(Slot::default())));
    (sender.clone(), Receiver::Latest(sender))
}
impl Sender {
    pub fn clear(&self) {
        dispose(self.0.lock().unwrap().result.take());
    }
    pub fn publish(&self, result: BuildResult) -> bool {
        let mut slot = self.0.lock().unwrap();
        if slot.closed {
            dispose(Some(result));
            return false;
        }
        dispose(slot.result.replace(result));
        true
    }
}
impl From<mpsc::Receiver<BuildResult>> for Receiver {
    fn from(value: mpsc::Receiver<BuildResult>) -> Self {
        Self::Channel(value)
    }
}
impl Receiver {
    pub fn try_recv(&self) -> Result<BuildResult, mpsc::TryRecvError> {
        match self {
            Self::Channel(receiver) => receiver.try_recv(),
            Self::Latest(sender) => sender
                .0
                .lock()
                .unwrap()
                .result
                .take()
                .ok_or(mpsc::TryRecvError::Empty),
        }
    }
    pub fn recv_timeout(
        &self,
        duration: std::time::Duration,
    ) -> Result<BuildResult, mpsc::RecvTimeoutError> {
        if let Self::Channel(receiver) = self {
            return receiver.recv_timeout(duration);
        }
        let deadline = std::time::Instant::now() + duration;
        loop {
            match self.try_recv() {
                Ok(result) => return Ok(result),
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(mpsc::RecvTimeoutError::Disconnected);
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                return Err(mpsc::RecvTimeoutError::Timeout);
            }
            std::thread::sleep(remaining.min(std::time::Duration::from_millis(10)));
        }
    }
    pub fn try_iter(&self) -> impl Iterator<Item = BuildResult> + '_ {
        std::iter::from_fn(|| self.try_recv().ok())
    }
}
impl Drop for Receiver {
    fn drop(&mut self) {
        if let Self::Latest(sender) = self {
            let mut slot = sender.0.lock().unwrap();
            slot.closed = true;
            dispose(slot.result.take());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replaces_and_disposes_artifacts_without_consumer_progress() {
        let root = std::env::temp_dir().join(format!("identity-latest-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let (sender, receiver) = channel();
        let paths: Vec<_> = (0..3)
            .map(|index| root.join(format!("{index}.so")))
            .collect();
        for (index, path) in paths.iter().enumerate() {
            std::fs::write(path, b"immutable artifact").unwrap();
            assert!(sender.publish(BuildResult {
                artifact: Some(path.clone()),
                message: index.to_string()
            }));
            assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
        }
        let result = receiver.try_recv().unwrap();
        assert_eq!(result.message, "2");
        assert!(receiver.try_recv().is_err());
        dispose(Some(result));
        let diagnostic = BuildResult {
            artifact: None,
            message: "compiler error".into(),
        };
        assert!(sender.publish(diagnostic));
        sender.clear();
        assert!(receiver.try_recv().is_err());
        std::fs::write(&paths[1], b"unconsumed artifact").unwrap();
        assert!(sender.publish(BuildResult {
            artifact: Some(paths[1].clone()),
            message: "pending".into()
        }));
        drop(receiver);
        assert!(!paths[1].exists());
        std::fs::write(&paths[0], b"late artifact").unwrap();
        assert!(!sender.publish(BuildResult {
            artifact: Some(paths[0].clone()),
            message: "late".into()
        }));
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
        std::fs::remove_dir(root).unwrap();
    }
}
