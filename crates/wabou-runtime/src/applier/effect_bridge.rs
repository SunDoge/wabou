use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::effect_trace::{EffectTrace, TraceSubmission};
use crate::jsrt::JsRuntime;

use gpui_shell::WakeCallback;
use gpui_shell::{
    AppDirectories, EffectCompletion, EffectId, EffectOp, EffectPayload, EffectRequest,
    EffectResult, EffectScope,
};

static NEXT_EFFECT_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Clone)]
pub(crate) struct EffectBridge {
    effects: Rc<RefCell<VecDeque<EffectRequest>>>,
    action_wake: Rc<RefCell<Option<WakeCallback>>>,
    pending: Rc<RefCell<HashSet<u32>>>,
    trace: Rc<RefCell<Option<EffectTrace>>>,
    replay_completions: Rc<RefCell<VecDeque<EffectCompletion>>>,
    app_directories: Rc<RefCell<Option<AppDirectories>>>,
}

impl EffectBridge {
    pub(crate) fn install(js: &JsRuntime, window_key: gpui_shell::WindowResourceKey) -> Self {
        let bridge = Self {
            effects: Rc::new(RefCell::new(VecDeque::new())),
            action_wake: Rc::new(RefCell::new(None)),
            pending: Rc::new(RefCell::new(HashSet::new())),
            trace: Rc::new(RefCell::new(None)),
            replay_completions: Rc::new(RefCell::new(VecDeque::new())),
            app_directories: Rc::new(RefCell::new(None)),
        };
        bridge.install_functions(js, window_key);
        bridge
    }

    fn install_functions(&self, js: &JsRuntime, window_key: gpui_shell::WindowResourceKey) {
        let submit_bridge = self.clone();
        js.with(|ctx| -> rquickjs::Result<()> {
            ctx.globals().set(
                "__wabou_effect_submit",
                rquickjs::Function::new(
                    ctx.clone(),
                    move |capability: u32, method: u16, payload_json: String| -> u32 {
                        let id = NEXT_EFFECT_ID.fetch_add(1, Ordering::Relaxed);
                        assert_ne!(id, 0, "effect request id space exhausted for this process");
                        let op = EffectOp::new(capability, method);
                        let payload = decode_effect_payload(
                            op,
                            window_key,
                            payload_json,
                            submit_bridge.app_directories.borrow().as_ref(),
                        );
                        submit_bridge.pending.borrow_mut().insert(id);
                        let request = EffectRequest {
                            id: EffectId(id),
                            scope: EffectScope::Window(window_key),
                            payload,
                        };
                        #[cfg(feature = "profiling")]
                        tracing::trace!(
                            target: "wabou::perf",
                            effect_id = id,
                            capability,
                            method,
                            "native_effect.submit"
                        );
                        let submission = submit_bridge
                            .trace
                            .borrow()
                            .as_ref()
                            .map(|trace| trace.submit(&request));
                        match submission {
                            Some(TraceSubmission::Replay(completions)) => submit_bridge
                                .replay_completions
                                .borrow_mut()
                                .extend(completions),
                            Some(TraceSubmission::Live) | None => {
                                submit_bridge.effects.borrow_mut().push_back(request);
                            }
                        }
                        if let Some(wake) = submit_bridge.action_wake.borrow().as_ref() {
                            wake();
                        }
                        id
                    },
                )?,
            )?;
            ctx.globals()
                .set("__wabou_effect_abi", legacy_shell::EFFECT_ABI_VERSION)?;
            ctx.globals().set("__wabou_window_id_lo", window_key.lo())?;
            ctx.globals().set("__wabou_window_id_hi", window_key.hi())?;
            Ok(())
        })
        .expect("install effect host functions");
    }

    pub(super) fn set_wake_callback(&self, wake: WakeCallback) {
        *self.action_wake.borrow_mut() = Some(wake);
    }

    pub(crate) fn set_trace(&self, trace: EffectTrace) {
        *self.trace.borrow_mut() = Some(trace);
    }

    pub(crate) fn set_app_directories(&self, directories: AppDirectories) {
        *self.app_directories.borrow_mut() = Some(directories);
    }

    pub(super) fn take(&self, js: &JsRuntime) -> Option<EffectRequest> {
        let mut delivered_replay = false;
        while let Some(completion) = self.replay_completions.borrow_mut().pop_front() {
            self.deliver_if_pending(js, &completion);
            delivered_replay = true;
        }
        if delivered_replay {
            // Replay resolves the same Promise as a live native completion,
            // but does so synchronously while the shell drains effects. Run a
            // bounded microtask checkpoint before the next test action can
            // inspect state, then wake the shell for rendering or any work
            // left by the scheduler budget.
            js.poll_async_runtime();
            if let Some(wake) = self.action_wake.borrow().as_ref() {
                wake();
            }
        }
        self.effects.borrow_mut().pop_front()
    }

    pub(super) fn complete(&self, js: &JsRuntime, completion: EffectCompletion) {
        if let Some(trace) = self.trace.borrow().as_ref() {
            trace.complete(&completion);
        }
        self.deliver_if_pending(js, &completion);
    }

    fn deliver_if_pending(&self, js: &JsRuntime, completion: &EffectCompletion) {
        if self.pending.borrow_mut().remove(&completion.id.0) {
            complete_js_effect(js, completion);
        }
    }
}

pub(super) fn decode_effect_payload(
    op: EffectOp,
    window_key: gpui_shell::WindowResourceKey,
    payload_json: String,
    app_directories: Option<&AppDirectories>,
) -> EffectPayload {
    let invalid = |message: String| EffectPayload::Invalid { op, message };
    match op {
        gpui_shell::effect::builtin::CLIPBOARD_READ => EffectPayload::ClipboardRead,
        gpui_shell::effect::builtin::CLIPBOARD_WRITE => {
            #[derive(serde::Deserialize)]
            struct Request {
                text: String,
            }
            serde_json::from_str::<Request>(&payload_json)
                .map(|request| EffectPayload::ClipboardWrite { text: request.text })
                .unwrap_or_else(|error| invalid(error.to_string()))
        }
        gpui_shell::effect::builtin::WINDOW_CREATE => {
            let value: serde_json::Value = serde_json::from_str(&payload_json).unwrap_or_default();
            let mut options = gpui_shell::WindowOptions::new();
            if let Some(title) = value.get("title").and_then(|value| value.as_str()) {
                options = options.title(title);
            }
            options = options.initial_inner_size(
                value
                    .get("width")
                    .and_then(|value| value.as_u64())
                    .unwrap_or(800) as u32,
                value
                    .get("height")
                    .and_then(|value| value.as_u64())
                    .unwrap_or(600) as u32,
            );
            if let Some(resizable) = value.get("resizable").and_then(|value| value.as_bool()) {
                options = options.resizable(resizable);
            }
            if let Some(decorations) = value.get("decorations").and_then(|value| value.as_bool()) {
                options = options.decorations(decorations);
            }
            if let Some(transparent) = value.get("transparent").and_then(|value| value.as_bool()) {
                options = options.transparent(transparent);
            }
            if let Some(window_level) = value.get("windowLevel").and_then(|value| value.as_str()) {
                options = options.window_level(match window_level {
                    "alwaysOnBottom" => gpui_shell::WindowLevel::AlwaysOnBottom,
                    "alwaysOnTop" => gpui_shell::WindowLevel::AlwaysOnTop,
                    _ => gpui_shell::WindowLevel::Normal,
                });
            }
            if value.get("inputMode").and_then(|value| value.as_str()) == Some("passthrough") {
                options = options.input_mode(gpui_shell::WindowInputMode::Passthrough);
            }
            if let (Some(width), Some(height)) = (
                value.get("minWidth").and_then(|value| value.as_u64()),
                value.get("minHeight").and_then(|value| value.as_u64()),
            ) {
                options = options.min_inner_size(width as u32, height as u32);
            }
            EffectPayload::WindowCreate(gpui_shell::effect::WindowCreateRequest { options })
        }
        gpui_shell::effect::builtin::WINDOW_CLOSE
        | gpui_shell::effect::builtin::WINDOW_SET_MAXIMIZED
        | gpui_shell::effect::builtin::WINDOW_SET_TITLE
        | gpui_shell::effect::builtin::WINDOW_MINIMIZE
        | gpui_shell::effect::builtin::WINDOW_START_DRAGGING
        | gpui_shell::effect::builtin::WINDOW_SHOW => {
            let value: serde_json::Value = serde_json::from_str(&payload_json).unwrap_or_default();
            let target = value
                .get("windowId")
                .and_then(|value| {
                    serde_json::from_value::<gpui_shell::WindowResourceKey>(value.clone()).ok()
                })
                .unwrap_or(window_key);
            let command = if op == gpui_shell::effect::builtin::WINDOW_CLOSE {
                gpui_shell::WindowCommand::Close
            } else if op == gpui_shell::effect::builtin::WINDOW_MINIMIZE {
                gpui_shell::WindowCommand::Minimize
            } else if op == gpui_shell::effect::builtin::WINDOW_START_DRAGGING {
                gpui_shell::WindowCommand::StartDragging
            } else if op == gpui_shell::effect::builtin::WINDOW_SET_MAXIMIZED {
                gpui_shell::WindowCommand::SetMaximized(
                    value
                        .get("value")
                        .and_then(|value| value.as_bool())
                        .unwrap_or(false),
                )
            } else if op == gpui_shell::effect::builtin::WINDOW_SHOW {
                gpui_shell::WindowCommand::Show
            } else {
                gpui_shell::WindowCommand::SetTitle(
                    value
                        .get("title")
                        .and_then(|value| value.as_str())
                        .unwrap_or_default()
                        .to_owned(),
                )
            };
            EffectPayload::WindowControl {
                window_id: target,
                command,
            }
        }
        gpui_shell::effect::builtin::CONTEXT_MENU_SHOW => {
            serde_json::from_str::<gpui_shell::ContextMenuRequest>(&payload_json)
                .map(EffectPayload::ContextMenuShow)
                .unwrap_or_else(|error| invalid(error.to_string()))
        }
        gpui_shell::effect::builtin::APP_DIRS_RESOLVE => app_directories
            .cloned()
            .map(EffectPayload::AppDirsResolve)
            .unwrap_or_else(|| invalid("application directories are not configured".into())),
        gpui_shell::effect::builtin::DIALOG_OPEN => {
            serde_json::from_str::<gpui_shell::OpenDialogRequest>(&payload_json)
                .map(EffectPayload::DialogOpen)
                .unwrap_or_else(|error| invalid(error.to_string()))
        }
        gpui_shell::effect::builtin::DIALOG_SAVE => {
            serde_json::from_str::<gpui_shell::SaveDialogRequest>(&payload_json)
                .map(EffectPayload::DialogSave)
                .unwrap_or_else(|error| invalid(error.to_string()))
        }
        gpui_shell::effect::builtin::DIALOG_PICK_DIRECTORY => {
            serde_json::from_str::<gpui_shell::PickDirectoryRequest>(&payload_json)
                .map(EffectPayload::DialogPickDirectory)
                .unwrap_or_else(|error| invalid(error.to_string()))
        }
        gpui_shell::effect::builtin::DIALOG_MESSAGE => {
            serde_json::from_str::<gpui_shell::MessageDialogRequest>(&payload_json)
                .map(EffectPayload::DialogMessage)
                .unwrap_or_else(|error| invalid(error.to_string()))
        }
        gpui_shell::effect::builtin::NOTIFICATION_SHOW => {
            serde_json::from_str::<gpui_shell::NotificationRequest>(&payload_json)
                .map(EffectPayload::NotificationShow)
                .unwrap_or_else(|error| invalid(error.to_string()))
        }
        gpui_shell::effect::builtin::APPLICATION_EXIT => EffectPayload::ApplicationExit,
        gpui_shell::effect::builtin::APPLICATION_RELAUNCH => EffectPayload::ApplicationRelaunch,
        _ => EffectPayload::Extension {
            op,
            bytes: payload_json.into_bytes(),
        },
    }
}

fn complete_js_effect(js: &JsRuntime, completion: &EffectCompletion) {
    #[cfg(feature = "profiling")]
    tracing::trace!(
        target: "wabou::perf",
        effect_id = completion.id.0,
        capability = completion.op.capability.0,
        method = completion.op.method.0,
        "native_effect.complete"
    );
    let (status, payload) = match &completion.result {
        EffectResult::Unit => (0_u8, "null".to_owned()),
        EffectResult::ClipboardText(text) => (
            0,
            serde_json::to_string(text).unwrap_or_else(|_| "null".into()),
        ),
        EffectResult::ContextMenuSelection(selection) => (
            0,
            serde_json::to_string(selection).unwrap_or_else(|_| "null".into()),
        ),
        EffectResult::AppDirectories(directories) => (
            0,
            serde_json::to_string(directories).unwrap_or_else(|_| "null".into()),
        ),
        EffectResult::Window(window) => (
            0,
            serde_json::to_string(window).unwrap_or_else(|_| "null".into()),
        ),
        EffectResult::DialogPaths(paths) => (
            0,
            serde_json::to_string(paths).unwrap_or_else(|_| "null".into()),
        ),
        EffectResult::DialogMessage(result) => (
            0,
            serde_json::to_string(result).unwrap_or_else(|_| "null".into()),
        ),
        EffectResult::Cancelled => (1, "null".to_owned()),
        EffectResult::Error { code, message } => (
            2,
            serde_json::json!({ "code": code, "message": message }).to_string(),
        ),
    };
    let result = js.with(|ctx| -> rquickjs::Result<()> {
        let callback: rquickjs::Function = ctx.globals().get("__wabou_effect_complete")?;
        callback.call::<_, ()>((
            completion.id.0,
            completion.op.capability.0,
            completion.op.method.0,
            status,
            payload,
        ))
    });
    if let Err(error) = result {
        tracing::warn!(
            ?error,
            effect_id = completion.id.0,
            "effect completion callback failed"
        );
    }
}
