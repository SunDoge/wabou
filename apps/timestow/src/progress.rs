use std::{
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant},
};

use rustic_core::{Progress, ProgressBars, ProgressType, RusticProgress};
use serde::{Deserialize, Serialize};

pub const BACKUP_PROGRESS_TOPIC: &str = "timestow:backup-progress";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BackupProgressPhase {
    Running,
    PhaseComplete,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BackupProgressKind {
    Spinner,
    Counter,
    Bytes,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BackupProgressEvent {
    pub profile_id: String,
    pub state: BackupProgressPhase,
    pub kind: BackupProgressKind,
    pub title: String,
    pub current: u64,
    pub total: Option<u64>,
}

type ProgressCallback = dyn Fn(&BackupProgressEvent) + Send + Sync;

#[derive(Clone, Default)]
pub struct ProgressEmitter {
    callback: Arc<RwLock<Option<Arc<ProgressCallback>>>>,
}

impl ProgressEmitter {
    pub fn replace(&self, callback: impl Fn(&BackupProgressEvent) + Send + Sync + 'static) {
        if let Ok(mut current) = self.callback.write() {
            *current = Some(Arc::new(callback));
        }
    }

    pub fn emit(&self, event: &BackupProgressEvent) {
        let callback = self
            .callback
            .read()
            .ok()
            .and_then(|callback| callback.clone());
        if let Some(callback) = callback {
            callback(event);
        }
    }

    pub fn emit_state(&self, profile_id: &str, state: BackupProgressPhase, title: &str) {
        self.emit(&BackupProgressEvent {
            profile_id: profile_id.to_string(),
            state,
            kind: BackupProgressKind::Spinner,
            title: title.to_string(),
            current: u64::from(state == BackupProgressPhase::Completed),
            total: (state == BackupProgressPhase::Completed).then_some(1),
        });
    }
}

#[derive(Debug)]
struct ProgressState {
    title: String,
    current: u64,
    total: Option<u64>,
    last_emit: Option<Instant>,
}

#[derive(Clone)]
struct BackupProgress {
    profile_id: String,
    kind: BackupProgressKind,
    state: Arc<Mutex<ProgressState>>,
    emitter: ProgressEmitter,
}

impl std::fmt::Debug for BackupProgress {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BackupProgress")
            .field("profile_id", &self.profile_id)
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl BackupProgress {
    fn new(
        profile_id: String,
        kind: BackupProgressKind,
        title: String,
        emitter: ProgressEmitter,
    ) -> Self {
        let progress = Self {
            profile_id,
            kind,
            state: Arc::new(Mutex::new(ProgressState {
                title,
                current: 0,
                total: None,
                last_emit: None,
            })),
            emitter,
        };
        progress.publish(BackupProgressPhase::Running, true);
        progress
    }

    fn publish(&self, phase: BackupProgressPhase, force: bool) {
        let event = {
            let Ok(mut state) = self.state.lock() else {
                return;
            };
            let now = Instant::now();
            if !force
                && state
                    .last_emit
                    .is_some_and(|last| now.duration_since(last) < Duration::from_millis(75))
                && state.total.is_none_or(|total| state.current < total)
            {
                return;
            }
            state.last_emit = Some(now);
            BackupProgressEvent {
                profile_id: self.profile_id.clone(),
                state: phase,
                kind: self.kind,
                title: state.title.clone(),
                current: state.current,
                total: state.total,
            }
        };
        self.emitter.emit(&event);
    }
}

impl RusticProgress for BackupProgress {
    fn is_hidden(&self) -> bool {
        false
    }

    fn set_length(&self, len: u64) {
        if let Ok(mut state) = self.state.lock() {
            state.total = Some(len);
        }
        self.publish(BackupProgressPhase::Running, true);
    }

    fn set_title(&self, title: &str) {
        if let Ok(mut state) = self.state.lock() {
            state.title = title.to_string();
        }
        self.publish(BackupProgressPhase::Running, true);
    }

    fn inc(&self, inc: u64) {
        if let Ok(mut state) = self.state.lock() {
            state.current = state.current.saturating_add(inc);
        }
        self.publish(BackupProgressPhase::Running, false);
    }

    fn finish(&self) {
        if let Ok(mut state) = self.state.lock()
            && let Some(total) = state.total
        {
            state.current = total;
        }
        self.publish(BackupProgressPhase::PhaseComplete, true);
    }
}

#[derive(Clone)]
pub struct BackupProgressBars {
    profile_id: String,
    emitter: ProgressEmitter,
}

impl BackupProgressBars {
    pub fn new(profile_id: String, emitter: ProgressEmitter) -> Self {
        Self {
            profile_id,
            emitter,
        }
    }
}

impl std::fmt::Debug for BackupProgressBars {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BackupProgressBars")
            .field("profile_id", &self.profile_id)
            .finish_non_exhaustive()
    }
}

impl ProgressBars for BackupProgressBars {
    fn progress(&self, progress_type: ProgressType, prefix: &str) -> Progress {
        let kind = match progress_type {
            ProgressType::Spinner => BackupProgressKind::Spinner,
            ProgressType::Counter => BackupProgressKind::Counter,
            ProgressType::Bytes => BackupProgressKind::Bytes,
        };
        Progress::new(BackupProgress::new(
            self.profile_id.clone(),
            kind,
            prefix.trim().trim_end_matches('.').to_string(),
            self.emitter.clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_real_counts_and_completion() {
        let events = Arc::new(Mutex::new(Vec::<BackupProgressEvent>::new()));
        let captured = events.clone();
        let emitter = ProgressEmitter::default();
        emitter.replace(move |event| {
            captured
                .lock()
                .expect("progress events")
                .push(event.clone());
        });

        let progress = BackupProgress::new(
            "photos".to_string(),
            BackupProgressKind::Bytes,
            "Writing data".to_string(),
            emitter,
        );
        progress.set_length(2_048);
        progress.inc(1_024);
        progress.finish();

        let events = events.lock().expect("progress events");
        assert!(events.iter().all(|event| event.profile_id == "photos"));
        assert!(events.iter().any(|event| {
            event.state == BackupProgressPhase::Running && event.total == Some(2_048)
        }));
        assert_eq!(
            events.last(),
            Some(&BackupProgressEvent {
                profile_id: "photos".to_string(),
                state: BackupProgressPhase::PhaseComplete,
                kind: BackupProgressKind::Bytes,
                title: "Writing data".to_string(),
                current: 2_048,
                total: Some(2_048),
            })
        );
    }
}
