use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use wabou_shell::{
    EFFECT_ABI_VERSION, EffectCompletion, EffectErrorCode, EffectOp, EffectRequest, EffectResult,
    EffectTapeEntry,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EffectTapeFile {
    abi_version: u16,
    entries: Vec<EffectTapeEntry>,
}

enum TraceMode {
    Record {
        entries: Vec<EffectTapeEntry>,
        record_all: bool,
        recorded_ids: HashSet<u32>,
    },
    Replay {
        entries: VecDeque<EffectTapeEntry>,
        recorded_ops: HashSet<EffectOp>,
        request_ids: HashMap<u32, u32>,
    },
    Fixtures {
        entries: VecDeque<(EffectOp, EffectResult)>,
    },
}

#[derive(Clone)]
pub(crate) struct EffectTrace(Arc<Mutex<TraceMode>>);

pub(crate) enum TraceSubmission {
    Live,
    Replay(Vec<EffectCompletion>),
}

impl EffectTrace {
    pub(crate) fn fixtures() -> Self {
        Self(Arc::new(Mutex::new(TraceMode::Fixtures {
            entries: VecDeque::new(),
        })))
    }

    pub(crate) fn enqueue_fixture(&self, op: EffectOp, result: EffectResult) -> Result<(), String> {
        let mut mode = self.0.lock().map_err(|_| "effect fixture mutex poisoned")?;
        let TraceMode::Fixtures { entries } = &mut *mode else {
            return Err(
                "effect fixtures are unavailable while recording or replaying an effect tape"
                    .to_owned(),
            );
        };
        entries.push_back((op, result));
        Ok(())
    }

    pub(crate) fn take_pending_fixtures(&self) -> Vec<EffectOp> {
        let Ok(mut mode) = self.0.lock() else {
            return Vec::new();
        };
        match &mut *mode {
            TraceMode::Fixtures { entries } => entries.drain(..).map(|(op, _)| op).collect(),
            _ => Vec::new(),
        }
    }

    pub(crate) fn record(record_all: bool) -> Self {
        Self(Arc::new(Mutex::new(TraceMode::Record {
            entries: Vec::new(),
            record_all,
            recorded_ids: HashSet::new(),
        })))
    }

    pub(crate) fn replay(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
        let tape: EffectTapeFile =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if tape.abi_version != EFFECT_ABI_VERSION {
            return Err(format!(
                "effect tape ABI {} does not match host ABI {}",
                tape.abi_version, EFFECT_ABI_VERSION
            ));
        }
        let recorded_ops = tape
            .entries
            .iter()
            .filter_map(|entry| match entry {
                EffectTapeEntry::Request(request) => Some(request.payload.op()),
                EffectTapeEntry::Completion(_) => None,
            })
            .collect();
        Ok(Self(Arc::new(Mutex::new(TraceMode::Replay {
            entries: tape.entries.into(),
            recorded_ops,
            request_ids: HashMap::new(),
        }))))
    }

    pub(crate) fn submit(&self, request: &EffectRequest) -> TraceSubmission {
        let mut mode = self.0.lock().expect("effect trace mutex poisoned");
        match &mut *mode {
            TraceMode::Record {
                entries,
                record_all,
                recorded_ids,
            } => {
                if *record_all || safe_to_record(request.payload.op()) {
                    entries.push(EffectTapeEntry::Request(request.clone()));
                    recorded_ids.insert(request.id.0);
                }
                TraceSubmission::Live
            }
            TraceMode::Replay {
                entries,
                recorded_ops,
                request_ids,
            } => {
                if !recorded_ops.contains(&request.payload.op()) {
                    return TraceSubmission::Live;
                }
                let expected = entries.pop_front();
                let Some(EffectTapeEntry::Request(expected_request)) = expected else {
                    return TraceSubmission::Replay(vec![EffectCompletion {
                        id: request.id,
                        op: request.payload.op(),
                        result: EffectResult::Error {
                            code: EffectErrorCode::ReplayDiverged,
                            message: format!(
                                "effect replay diverged: expected {expected:?}, received {request:?}"
                            ),
                        },
                    }]);
                };
                if !same_request_ignoring_id(&expected_request, request) {
                    return TraceSubmission::Replay(vec![EffectCompletion {
                        id: request.id,
                        op: request.payload.op(),
                        result: EffectResult::Error {
                            code: EffectErrorCode::ReplayDiverged,
                            message: format!(
                                "effect replay diverged: expected {expected_request:?}, received {request:?}"
                            ),
                        },
                    }]);
                }
                request_ids.insert(expected_request.id.0, request.id.0);
                let mut completions = Vec::new();
                while matches!(entries.front(), Some(EffectTapeEntry::Completion(_))) {
                    let Some(EffectTapeEntry::Completion(mut completion)) = entries.pop_front()
                    else {
                        unreachable!()
                    };
                    if let Some(id) = request_ids.remove(&completion.id.0) {
                        completion.id.0 = id;
                    }
                    completions.push(completion);
                }
                TraceSubmission::Replay(completions)
            }
            TraceMode::Fixtures { entries } => {
                let Some((expected_op, _)) = entries.front() else {
                    return TraceSubmission::Live;
                };
                if *expected_op == request.payload.op() {
                    let (_, result) = entries.pop_front().expect("fixture entry disappeared");
                    TraceSubmission::Replay(vec![EffectCompletion {
                        id: request.id,
                        op: request.payload.op(),
                        result,
                    }])
                } else if entries.iter().any(|(op, _)| *op == request.payload.op()) {
                    TraceSubmission::Replay(vec![EffectCompletion {
                        id: request.id,
                        op: request.payload.op(),
                        result: EffectResult::Error {
                            code: EffectErrorCode::ReplayDiverged,
                            message: format!(
                                "effect fixture order diverged: expected {expected_op:?}, received {:?}",
                                request.payload.op()
                            ),
                        },
                    }])
                } else {
                    TraceSubmission::Live
                }
            }
        }
    }

    pub(crate) fn complete(&self, completion: &EffectCompletion) {
        let mut mode = self.0.lock().expect("effect trace mutex poisoned");
        if let TraceMode::Record {
            entries,
            recorded_ids,
            ..
        } = &mut *mode
            && recorded_ids.remove(&completion.id.0)
        {
            entries.push(EffectTapeEntry::Completion(completion.clone()));
        }
    }

    pub(crate) fn write(&self, path: &Path) -> Result<(), String> {
        let mode = self.0.lock().expect("effect trace mutex poisoned");
        let TraceMode::Record { entries, .. } = &*mode else {
            return Ok(());
        };
        let bytes = serde_json::to_vec_pretty(&EffectTapeFile {
            abi_version: EFFECT_ABI_VERSION,
            entries: entries.clone(),
        })
        .map_err(|error| error.to_string())?;
        let mut options = std::fs::OpenOptions::new();
        options.create(true).truncate(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(path).map_err(|error| error.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))
                .map_err(|error| error.to_string())?;
        }
        file.write_all(&bytes).map_err(|error| error.to_string())
    }
}

fn safe_to_record(op: EffectOp) -> bool {
    op == wabou_shell::effect::builtin::WINDOW_CLOSE
        || op == wabou_shell::effect::builtin::WINDOW_SET_MAXIMIZED
}

fn same_request_ignoring_id(expected: &EffectRequest, actual: &EffectRequest) -> bool {
    expected.scope == actual.scope && expected.payload == actual.payload
}

#[cfg(test)]
mod tests {
    use super::*;
    use wabou_shell::{EffectId, EffectPayload, EffectScope, WindowCommand};

    fn title(id: u32, title: &str) -> EffectRequest {
        let window_key = wabou_shell::initial_window_resource_key(0);
        EffectRequest {
            id: EffectId(id),
            scope: EffectScope::Window(window_key),
            payload: EffectPayload::WindowControl {
                window_id: window_key,
                command: WindowCommand::SetTitle(title.into()),
            },
        }
    }

    fn completion(request: &EffectRequest) -> EffectCompletion {
        EffectCompletion {
            id: request.id,
            op: request.payload.op(),
            result: EffectResult::Unit,
        }
    }

    #[test]
    fn replay_preserves_async_completion_order_and_remaps_request_ids() {
        let first = title(1, "first");
        let second = title(2, "second");
        let trace = EffectTrace::record(true);
        assert!(matches!(trace.submit(&first), TraceSubmission::Live));
        assert!(matches!(trace.submit(&second), TraceSubmission::Live));
        trace.complete(&completion(&second));
        trace.complete(&completion(&first));

        let entries = match &*trace.0.lock().unwrap() {
            TraceMode::Record { entries, .. } => entries.clone(),
            TraceMode::Replay { .. } | TraceMode::Fixtures { .. } => unreachable!(),
        };
        let recorded_ops = entries
            .iter()
            .filter_map(|entry| match entry {
                EffectTapeEntry::Request(request) => Some(request.payload.op()),
                EffectTapeEntry::Completion(_) => None,
            })
            .collect();
        let replay = EffectTrace(Arc::new(Mutex::new(TraceMode::Replay {
            entries: entries.into(),
            recorded_ops,
            request_ids: HashMap::new(),
        })));

        let replay_first = title(10, "first");
        let replay_second = title(11, "second");
        assert!(
            matches!(replay.submit(&replay_first), TraceSubmission::Replay(items) if items.is_empty())
        );
        let TraceSubmission::Replay(items) = replay.submit(&replay_second) else {
            panic!("recorded effect must be replayed")
        };
        assert_eq!(
            items.iter().map(|item| item.id.0).collect::<Vec<_>>(),
            vec![11, 10]
        );
    }

    #[test]
    fn safe_recording_excludes_clipboard_payloads() {
        let trace = EffectTrace::record(false);
        let request = EffectRequest {
            id: EffectId(1),
            scope: EffectScope::Window(wabou_shell::initial_window_resource_key(0)),
            payload: EffectPayload::ClipboardWrite {
                text: "secret".into(),
            },
        };
        assert!(matches!(trace.submit(&request), TraceSubmission::Live));
        trace.complete(&completion(&request));
        let is_empty = match &*trace.0.lock().unwrap() {
            TraceMode::Record { entries, .. } => entries.is_empty(),
            TraceMode::Replay { .. } | TraceMode::Fixtures { .. } => unreachable!(),
        };
        assert!(is_empty);
    }

    #[test]
    fn fixtures_complete_matching_effects_without_dispatching_native_ui() {
        let trace = EffectTrace::fixtures();
        trace
            .enqueue_fixture(
                wabou_shell::effect::builtin::DIALOG_PICK_DIRECTORY,
                EffectResult::DialogPaths(Some(vec!["/tmp/downloads".to_owned()])),
            )
            .unwrap();
        let request = EffectRequest {
            id: EffectId(41),
            scope: EffectScope::Window(wabou_shell::initial_window_resource_key(0)),
            payload: wabou_shell::EffectPayload::DialogPickDirectory(
                wabou_shell::PickDirectoryRequest {
                    title: Some("Downloads".to_owned()),
                    directory: None,
                },
            ),
        };

        let TraceSubmission::Replay(completions) = trace.submit(&request) else {
            panic!("fixture must intercept the matching native effect")
        };
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].id, request.id);
        assert_eq!(
            completions[0].result,
            EffectResult::DialogPaths(Some(vec!["/tmp/downloads".to_owned()]))
        );
        assert!(trace.take_pending_fixtures().is_empty());
    }

    #[test]
    fn tape_file_round_trips_through_the_versioned_json_format() {
        let path =
            std::env::temp_dir().join(format!("wabou-effect-tape-{}.json", std::process::id()));
        let request = title(1, "recorded");
        let trace = EffectTrace::record(true);
        trace.submit(&request);
        trace.complete(&completion(&request));
        trace.write(&path).unwrap();
        let tape_json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(
            tape_json["entries"][0]["value"]["scope"]["id"],
            serde_json::json!({ "lo": 1, "hi": 1 }),
            "serialized window identities must remain lossless outside Rust",
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }

        let replay = EffectTrace::replay(&path).unwrap();
        let replay_request = title(99, "recorded");
        let TraceSubmission::Replay(completions) = replay.submit(&replay_request) else {
            panic!("effect from tape must be replayed")
        };
        assert_eq!(completions[0].id, replay_request.id);
        std::fs::remove_file(path).unwrap();
    }
}
