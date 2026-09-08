use std::{
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant},
};

use rustic_core::{Progress, ProgressBars, ProgressType, RusticProgress};
use serde::{Deserialize, Serialize};

pub const OPERATION_PROGRESS_TOPIC: &str = "timestow:operation-progress";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OperationKind {
    Backup,
    Restore,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OperationProgressPhase {
    Running,
    PhaseComplete,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OperationProgressUnit {
    Spinner,
    Counter,
    Bytes,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OperationProgressEvent {
    pub profile_id: String,
    pub operation: OperationKind,
    pub operation_id: String,
    pub state: OperationProgressPhase,
    pub unit: OperationProgressUnit,
    pub title: String,
    pub current: u64,
    pub total: Option<u64>,
}

type ProgressCallback = dyn Fn(&OperationProgressEvent) + Send + Sync;

#[derive(Clone, Default)]
pub struct ProgressEmitter {
    callback: Arc<RwLock<Option<Arc<ProgressCallback>>>>,
}

impl ProgressEmitter {
    pub fn replace(&self, callback: impl Fn(&OperationProgressEvent) + Send + Sync + 'static) {
        if let Ok(mut current) = self.callback.write() {
            *current = Some(Arc::new(callback));
        }
    }

    pub fn emit(&self, event: &OperationProgressEvent) {
        let callback = self
            .callback
            .read()
            .ok()
            .and_then(|callback| callback.clone());
        if let Some(callback) = callback {
            callback(event);
        }
    }

    pub fn emit_state(
        &self,
        profile_id: &str,
        operation: OperationKind,
        operation_id: &str,
        state: OperationProgressPhase,
        title: &str,
    ) {
        self.emit(&OperationProgressEvent {
            profile_id: profile_id.to_string(),
            operation,
            operation_id: operation_id.to_string(),
            state,
            unit: OperationProgressUnit::Spinner,
            title: title.to_string(),
            current: u64::from(state == OperationProgressPhase::Completed),
            total: (state == OperationProgressPhase::Completed).then_some(1),
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
struct OperationProgress {
    profile_id: String,
    operation: OperationKind,
    operation_id: String,
    unit: OperationProgressUnit,
    state: Arc<Mutex<ProgressState>>,
    emitter: ProgressEmitter,
}

impl std::fmt::Debug for OperationProgress {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OperationProgress")
            .field("profile_id", &self.profile_id)
            .field("operation", &self.operation)
            .field("operation_id", &self.operation_id)
            .field("unit", &self.unit)
            .finish_non_exhaustive()
    }
}

impl OperationProgress {
    fn new(
        profile_id: String,
        operation: OperationKind,
        operation_id: String,
        unit: OperationProgressUnit,
        title: String,
        emitter: ProgressEmitter,
    ) -> Self {
        let progress = Self {
            profile_id,
            operation,
            operation_id,
            unit,
            state: Arc::new(Mutex::new(ProgressState {
                title,
                current: 0,
                total: None,
                last_emit: None,
            })),
            emitter,
        };
        progress.publish(OperationProgressPhase::Running, true);
        progress
    }

    fn publish(&self, phase: OperationProgressPhase, force: bool) {
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
            OperationProgressEvent {
                profile_id: self.profile_id.clone(),
                operation: self.operation,
                operation_id: self.operation_id.clone(),
                state: phase,
                unit: self.unit,
                title: state.title.clone(),
                current: state.current,
                total: state.total,
            }
        };
        self.emitter.emit(&event);
    }
}

impl RusticProgress for OperationProgress {
    fn is_hidden(&self) -> bool {
        false
    }

    fn set_length(&self, len: u64) {
        if let Ok(mut state) = self.state.lock() {
            state.total = Some(len);
        }
        self.publish(OperationProgressPhase::Running, true);
    }

    fn set_title(&self, title: &str) {
        if let Ok(mut state) = self.state.lock() {
            state.title = title.to_string();
        }
        self.publish(OperationProgressPhase::Running, true);
    }

    fn inc(&self, inc: u64) {
        if let Ok(mut state) = self.state.lock() {
            state.current = state.current.saturating_add(inc);
        }
        self.publish(OperationProgressPhase::Running, false);
    }

    fn finish(&self) {
        if let Ok(mut state) = self.state.lock()
            && let Some(total) = state.total
        {
            state.current = total;
        }
        self.publish(OperationProgressPhase::PhaseComplete, true);
    }
}

#[derive(Clone)]
pub struct OperationProgressBars {
    profile_id: String,
    operation: OperationKind,
    operation_id: String,
    emitter: ProgressEmitter,
}

impl OperationProgressBars {
    pub fn new(
        profile_id: String,
        operation: OperationKind,
        operation_id: String,
        emitter: ProgressEmitter,
    ) -> Self {
        Self {
            profile_id,
            operation,
            operation_id,
            emitter,
        }
    }
}

impl std::fmt::Debug for OperationProgressBars {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OperationProgressBars")
            .field("profile_id", &self.profile_id)
            .field("operation", &self.operation)
            .field("operation_id", &self.operation_id)
            .finish_non_exhaustive()
    }
}

impl ProgressBars for OperationProgressBars {
    fn progress(&self, progress_type: ProgressType, prefix: &str) -> Progress {
        let kind = match progress_type {
            ProgressType::Spinner => OperationProgressUnit::Spinner,
            ProgressType::Counter => OperationProgressUnit::Counter,
            ProgressType::Bytes => OperationProgressUnit::Bytes,
        };
        Progress::new(OperationProgress::new(
            self.profile_id.clone(),
            self.operation,
            self.operation_id.clone(),
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
        let events = Arc::new(Mutex::new(Vec::<OperationProgressEvent>::new()));
        let captured = events.clone();
        let emitter = ProgressEmitter::default();
        emitter.replace(move |event| {
            captured
                .lock()
                .expect("progress events")
                .push(event.clone());
        });

        let progress = OperationProgress::new(
            "photos".to_string(),
            OperationKind::Restore,
            "restore-7".to_string(),
            OperationProgressUnit::Bytes,
            "Writing data".to_string(),
            emitter,
        );
        progress.set_length(2_048);
        progress.inc(1_024);
        progress.finish();

        let events = events.lock().expect("progress events");
        assert!(events.iter().all(|event| event.profile_id == "photos"));
        assert!(events.iter().any(|event| {
            event.state == OperationProgressPhase::Running && event.total == Some(2_048)
        }));
        assert_eq!(
            events.last(),
            Some(&OperationProgressEvent {
                profile_id: "photos".to_string(),
                operation: OperationKind::Restore,
                operation_id: "restore-7".to_string(),
                state: OperationProgressPhase::PhaseComplete,
                unit: OperationProgressUnit::Bytes,
                title: "Writing data".to_string(),
                current: 2_048,
                total: Some(2_048),
            })
        );
    }
}
