//! GPUI view owning one Wabou JavaScript runtime.

use std::{rc::Rc, sync::Arc};

use wabou_shell_gpui::WakeCallback;
use wabou_shell_gpui::gpui::{
    ClipboardItem, Context, FocusHandle, PathPromptOptions, PromptButton, PromptLevel, Render,
    SystemNotification, Task, Window,
};

use crate::{Applier, FrameSource};
use wabou_shell::{
    ClipboardRequest, EffectCompletion, EffectErrorCode, EffectPayload, EffectRequest,
    EffectResult, HostAction, HostActionResult, UiEvent, WindowCommand,
};

/// A coarse GPUI entity for one Solid application runtime.
///
/// Solid retains individual UI nodes and emits one mutation batch per flush;
/// this entity advances that runtime once per GPUI frame and materializes the
/// resulting retained projection. It intentionally does not create one GPUI
/// entity per Solid node.
pub struct GpuiRuntimeView {
    applier: Applier,
    // Retaining the task ties the async bridge to the lifetime of this view.
    // The task itself only owns a weak entity handle, so this is not a cycle.
    _wake_task: Task<()>,
    focus: FocusHandle,
    default_title: String,
}

impl GpuiRuntimeView {
    /// Wrap an already configured and booted Wabou runtime.
    #[must_use]
    pub fn new(
        mut applier: Applier,
        default_title: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let (wake, receiver) = gpui_wake_channel();
        FrameSource::set_wake_callback(&mut applier, wake);

        let wake_task = cx.spawn(async move |view, cx| {
            while receiver.recv_async().await.is_ok() {
                if view
                    .update(cx, |view, cx| {
                        // Drain work immediately instead of waiting for an unrelated
                        // input event or animation frame. The following render pass
                        // publishes any resulting Solid mutation batch to GPUI.
                        let _ = FrameSource::poll_async(&mut view.applier);
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        });

        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Self {
            applier,
            _wake_task: wake_task,
            focus,
            default_title,
        }
    }

    /// Borrow the underlying runtime for host integration during migration.
    #[must_use]
    pub fn applier(&self) -> &Applier {
        &self.applier
    }

    /// Mutably borrow the underlying runtime for host integration.
    pub fn applier_mut(&mut self) -> &mut Applier {
        &mut self.applier
    }

    fn handle_input(
        &mut self,
        event: wabou_shell_gpui::ProjectedInputEvent,
        cx: &mut Context<Self>,
    ) {
        let mut response = self.applier.handle_gpui_input(event);
        if let Some(request) = response.clipboard.take() {
            match request {
                ClipboardRequest::Write(text) => {
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                }
                ClipboardRequest::Read => {
                    let text = cx
                        .read_from_clipboard()
                        .and_then(|clipboard| clipboard.text())
                        .filter(|text| !text.is_empty());
                    if let Some(text) = text {
                        let pasted =
                            FrameSource::handle_event(&mut self.applier, UiEvent::Paste(text));
                        response.handled |= pasted.handled;
                        response.request_redraw |= pasted.request_redraw;
                    }
                }
            }
        }
        if response.request_redraw {
            cx.notify();
        }
    }

    fn drain_host_actions(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let mut handled = false;
        while let Some(action) = FrameSource::take_host_action(&mut self.applier) {
            handled = true;
            match action {
                HostAction::OpenUrl(url) => {
                    if let Some(url) = allowed_external_url(&url) {
                        let _ = open::that_detached(url.as_str());
                    } else {
                        tracing::warn!(url, "refused unsafe external URL");
                    }
                }
                HostAction::SetClipboard(text) => {
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                }
                HostAction::WriteClipboard { request_id, text } => {
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                    FrameSource::complete_host_action(
                        &mut self.applier,
                        HostActionResult::ClipboardWrite {
                            request_id,
                            success: true,
                        },
                    );
                }
                HostAction::ReadClipboard { request_id } => {
                    let text = cx
                        .read_from_clipboard()
                        .and_then(|clipboard| clipboard.text());
                    FrameSource::complete_host_action(
                        &mut self.applier,
                        HostActionResult::Clipboard { request_id, text },
                    );
                }
                HostAction::SetWindowTitle(title) => {
                    window.set_window_title(title.as_deref().unwrap_or(&self.default_title));
                }
                HostAction::RequestAttention => window.request_attention(),
            }
        }
        handled
    }

    fn drain_effects(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let mut handled = false;
        while let Some(request) = FrameSource::take_effect(&mut self.applier) {
            handled = true;
            let completion = self.execute_effect(request, window, cx);
            if let Some(completion) = completion {
                FrameSource::complete_effect(&mut self.applier, completion);
            }
        }
        handled
    }

    fn execute_effect(
        &mut self,
        request: EffectRequest,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<EffectCompletion> {
        let id = request.id;
        let op = request.payload.op();
        match request.payload {
            EffectPayload::DialogOpen(request) => {
                if request.directory.is_some() || !request.filters.is_empty() {
                    tracing::warn!(
                        "GPUI-CE does not yet expose initial-directory or file-filter options"
                    );
                }
                let receiver = cx.prompt_for_paths(PathPromptOptions {
                    files: true,
                    directories: false,
                    multiple: request.multiple,
                    prompt: request.title.map(Into::into),
                });
                self.complete_path_prompt(id, op, receiver, cx);
                None
            }
            EffectPayload::DialogPickDirectory(request) => {
                if request.directory.is_some() {
                    tracing::warn!(
                        "GPUI-CE does not yet expose an initial-directory option for path prompts"
                    );
                }
                let receiver = cx.prompt_for_paths(PathPromptOptions {
                    files: false,
                    directories: true,
                    multiple: false,
                    prompt: request.title.map(Into::into),
                });
                self.complete_path_prompt(id, op, receiver, cx);
                None
            }
            EffectPayload::DialogSave(request) => {
                if request.title.is_some() || !request.filters.is_empty() {
                    tracing::warn!(
                        "GPUI-CE does not yet expose title or file-filter options for save prompts"
                    );
                }
                let directory = request
                    .directory
                    .map(std::path::PathBuf::from)
                    .or_else(|| std::env::current_dir().ok())
                    .unwrap_or_default();
                let receiver = cx.prompt_for_new_path(&directory, request.default_name.as_deref());
                self.complete_save_prompt(id, op, receiver, cx);
                None
            }
            EffectPayload::DialogMessage(request) => {
                let (buttons, results) = message_prompt_buttons(request.buttons);
                let receiver = window.prompt(
                    message_prompt_level(request.level),
                    request.title.as_deref().unwrap_or("Wabou"),
                    Some(&request.message),
                    &buttons,
                    cx,
                );
                self.complete_message_prompt(id, op, receiver, results, cx);
                None
            }
            payload => self.execute_synchronous_effect(id, op, payload, window, cx),
        }
    }

    fn execute_synchronous_effect(
        &mut self,
        id: wabou_shell::EffectId,
        op: wabou_shell::EffectOp,
        payload: EffectPayload,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<EffectCompletion> {
        let result = match payload {
            EffectPayload::ClipboardRead => EffectResult::ClipboardText(
                cx.read_from_clipboard()
                    .and_then(|clipboard| clipboard.text()),
            ),
            EffectPayload::ClipboardWrite { text } => {
                cx.write_to_clipboard(ClipboardItem::new_string(text));
                EffectResult::Unit
            }
            EffectPayload::AppDirsResolve(directories) => EffectResult::AppDirectories(directories),
            EffectPayload::WindowControl { command, .. } => {
                match command {
                    WindowCommand::Close => window.remove_window(),
                    WindowCommand::Minimize => window.minimize_window(),
                    WindowCommand::SetMaximized(maximized) => {
                        if window.is_maximized() != maximized {
                            window.zoom_window();
                        }
                    }
                    WindowCommand::SetTitle(title) => window.set_window_title(&title),
                    WindowCommand::StartDragging => window.start_window_move(),
                    WindowCommand::Show => window.activate_window(),
                }
                EffectResult::Unit
            }
            EffectPayload::NotificationShow(notification) => {
                if notification.title.trim().is_empty() {
                    EffectResult::Error {
                        code: EffectErrorCode::InvalidRequest,
                        message: "notification title must not be empty".into(),
                    }
                } else {
                    cx.show_system_notification(SystemNotification {
                        tag: format!("wabou-effect-{}", id.0).into(),
                        title: notification.title.into(),
                        body: notification.body.unwrap_or_default().into(),
                        actions: Vec::new(),
                    });
                    EffectResult::Unit
                }
            }
            EffectPayload::ApplicationExit => {
                cx.quit();
                EffectResult::Unit
            }
            EffectPayload::ApplicationRelaunch => {
                let result = match crate::host::relaunch_current_process() {
                    Ok(()) => EffectResult::Unit,
                    Err(error) => EffectResult::Error {
                        code: EffectErrorCode::PlatformFailure,
                        message: error.to_string(),
                    },
                };
                if matches!(result, EffectResult::Unit) {
                    cx.quit();
                }
                result
            }
            EffectPayload::Invalid { message, .. } => EffectResult::Error {
                code: EffectErrorCode::InvalidRequest,
                message,
            },
            EffectPayload::WindowCreate(_)
            | EffectPayload::ContextMenuShow(_)
            | EffectPayload::DialogOpen(_)
            | EffectPayload::DialogSave(_)
            | EffectPayload::DialogPickDirectory(_)
            | EffectPayload::DialogMessage(_)
            | EffectPayload::Extension { .. } => EffectResult::Error {
                code: EffectErrorCode::Unsupported,
                message: format!("effect `{op:?}` is not implemented by the GPUI shell yet"),
            },
        };
        Some(EffectCompletion { id, op, result })
    }

    fn complete_path_prompt(
        &self,
        id: wabou_shell::EffectId,
        op: wabou_shell::EffectOp,
        receiver: futures_channel::oneshot::Receiver<
            anyhow::Result<Option<Vec<std::path::PathBuf>>>,
        >,
        cx: &mut Context<Self>,
    ) {
        cx.spawn(async move |view, cx| {
            let result = match receiver.await {
                Ok(Ok(paths)) => EffectResult::DialogPaths(paths.map(paths_to_strings)),
                Ok(Err(error)) => platform_effect_error(error),
                Err(error) => platform_effect_error(error),
            };
            let _ = view.update(cx, |view, cx| {
                FrameSource::complete_effect(
                    &mut view.applier,
                    EffectCompletion { id, op, result },
                );
                cx.notify();
            });
        })
        .detach();
    }

    fn complete_save_prompt(
        &self,
        id: wabou_shell::EffectId,
        op: wabou_shell::EffectOp,
        receiver: futures_channel::oneshot::Receiver<anyhow::Result<Option<std::path::PathBuf>>>,
        cx: &mut Context<Self>,
    ) {
        cx.spawn(async move |view, cx| {
            let result = match receiver.await {
                Ok(Ok(path)) => {
                    EffectResult::DialogPaths(path.map(|path| paths_to_strings([path])))
                }
                Ok(Err(error)) => platform_effect_error(error),
                Err(error) => platform_effect_error(error),
            };
            let _ = view.update(cx, |view, cx| {
                FrameSource::complete_effect(
                    &mut view.applier,
                    EffectCompletion { id, op, result },
                );
                cx.notify();
            });
        })
        .detach();
    }

    fn complete_message_prompt(
        &self,
        id: wabou_shell::EffectId,
        op: wabou_shell::EffectOp,
        receiver: futures_channel::oneshot::Receiver<usize>,
        results: Vec<&'static str>,
        cx: &mut Context<Self>,
    ) {
        cx.spawn(async move |view, cx| {
            let result = match receiver.await {
                Ok(index) => EffectResult::DialogMessage(
                    results.get(index).copied().unwrap_or("custom").into(),
                ),
                Err(error) => platform_effect_error(error),
            };
            let _ = view.update(cx, |view, cx| {
                FrameSource::complete_effect(
                    &mut view.applier,
                    EffectCompletion { id, op, result },
                );
                cx.notify();
            });
        })
        .detach();
    }
}

fn paths_to_strings(paths: impl IntoIterator<Item = std::path::PathBuf>) -> Vec<String> {
    paths
        .into_iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect()
}

fn platform_effect_error(error: impl std::fmt::Display) -> EffectResult {
    EffectResult::Error {
        code: EffectErrorCode::PlatformFailure,
        message: error.to_string(),
    }
}

fn message_prompt_level(level: wabou_shell::MessageDialogLevel) -> PromptLevel {
    match level {
        wabou_shell::MessageDialogLevel::Info => PromptLevel::Info,
        wabou_shell::MessageDialogLevel::Warning => PromptLevel::Warning,
        wabou_shell::MessageDialogLevel::Error => PromptLevel::Critical,
    }
}

fn message_prompt_buttons(
    buttons: wabou_shell::MessageDialogButtons,
) -> (Vec<PromptButton>, Vec<&'static str>) {
    match buttons {
        wabou_shell::MessageDialogButtons::Ok => (vec![PromptButton::ok("OK")], vec!["ok"]),
        wabou_shell::MessageDialogButtons::OkCancel => (
            vec![PromptButton::ok("OK"), PromptButton::cancel("Cancel")],
            vec!["ok", "cancel"],
        ),
        wabou_shell::MessageDialogButtons::YesNo => (
            vec![PromptButton::ok("Yes"), PromptButton::cancel("No")],
            vec!["yes", "no"],
        ),
        wabou_shell::MessageDialogButtons::YesNoCancel => (
            vec![
                PromptButton::ok("Yes"),
                PromptButton::new("No"),
                PromptButton::cancel("Cancel"),
            ],
            vec!["yes", "no", "cancel"],
        ),
    }
}

fn allowed_external_url(value: &str) -> Option<url::Url> {
    let url = url::Url::parse(value).ok()?;
    matches!(url.scheme(), "http" | "https" | "mailto").then_some(url)
}

fn gpui_wake_channel() -> (WakeCallback, flume::Receiver<()>) {
    // One queued token is sufficient: poll_async drains every source and the next
    // completed Solid flush is committed atomically. This prevents message bursts
    // from scheduling an unbounded number of redundant GPUI updates.
    let (sender, receiver) = flume::bounded(1);
    let wake = Arc::new(move || {
        let _ = sender.try_send(());
    });
    (wake, receiver)
}

impl Render for GpuiRuntimeView {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl wabou_shell_gpui::gpui::IntoElement {
        let viewport = window.viewport_size();
        let _ = self
            .applier
            .build_gpui_frame(viewport.width.into(), viewport.height.into());
        if self.drain_host_actions(window, cx) {
            cx.notify();
        }
        if self.drain_effects(window, cx) {
            cx.notify();
        }

        if FrameSource::has_anim(&self.applier) {
            // GPUI associates this request with the currently rendering view
            // and notifies only that entity on the next platform frame.
            window.request_animation_frame();
        }

        let view = cx.weak_entity();
        let input = Rc::new(move |event, cx: &mut wabou_shell_gpui::gpui::App| {
            let _ = view.update(cx, |view, cx| {
                view.handle_input(event, cx);
            });
        });
        let text_input = self.applier.gpui_text_input_state();
        self.applier
            .gpui_interactive_element(input, self.focus.clone(), text_input)
            .expect("the canonical Wabou root remains retained")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::JsRuntime;
    use wabou_host_api::NodeKey;
    use wabou_shell::{EffectId, EffectScope, OpenDialogRequest};
    use wabou_shell_gpui::gpui::TestAppContext;

    #[test]
    fn gpui_wakes_are_coalesced_until_the_ui_task_drains_them() {
        let (wake, receiver) = gpui_wake_channel();

        wake();
        wake();
        wake();

        assert_eq!(receiver.try_recv(), Ok(()));
        assert!(matches!(
            receiver.try_recv(),
            Err(flume::TryRecvError::Empty)
        ));

        wake();
        assert_eq!(receiver.try_recv(), Ok(()));
    }

    #[test]
    fn real_solid_writer_frame_materializes_as_a_gpui_tree() {
        let runtime = JsRuntime::new().expect("QuickJS runtime");
        let mut applier = Applier::from_runtime(runtime, vello::peniko::Color::TRANSPARENT);
        applier
            .boot(include_str!("gen/test-runtime.js"))
            .expect("boot generated Solid runtime fixture");

        assert!(applier.build_gpui_frame(800, 600));
        assert_eq!(applier.protocol_revision(), 1);
        assert!(
            applier.gpui_contains(NodeKey::new(2, 1)),
            "the fixture's mounted <main> must cross the binary writer boundary"
        );
        let _root = applier
            .gpui_element()
            .expect("the completed Solid tree must materialize for GPUI");
    }

    #[wabou_shell_gpui::gpui::test]
    fn gpui_effect_executor_uses_the_platform_clipboard(cx: &mut TestAppContext) {
        let runtime = JsRuntime::new().expect("QuickJS runtime");
        let applier = Applier::from_runtime(runtime, vello::peniko::Color::TRANSPARENT);
        let (view, cx) = cx.add_window_view(move |window, cx| {
            GpuiRuntimeView::new(applier, "Clipboard test".into(), window, cx)
        });

        let write = cx.update(|window, app| {
            view.update(app, |view, cx| {
                view.execute_effect(
                    EffectRequest {
                        id: EffectId(1),
                        scope: EffectScope::Runtime,
                        payload: EffectPayload::ClipboardWrite {
                            text: "copied through GPUI".into(),
                        },
                    },
                    window,
                    cx,
                )
                .expect("synchronous clipboard completion")
            })
        });
        assert_eq!(write.result, EffectResult::Unit);
        assert_eq!(
            cx.read_from_clipboard().and_then(|item| item.text()),
            Some("copied through GPUI".into())
        );

        let read = cx.update(|window, app| {
            view.update(app, |view, cx| {
                view.execute_effect(
                    EffectRequest {
                        id: EffectId(2),
                        scope: EffectScope::Runtime,
                        payload: EffectPayload::ClipboardRead,
                    },
                    window,
                    cx,
                )
                .expect("synchronous clipboard completion")
            })
        });
        assert_eq!(
            read.result,
            EffectResult::ClipboardText(Some("copied through GPUI".into()))
        );
    }

    #[wabou_shell_gpui::gpui::test]
    fn gpui_effect_executor_uses_the_platform_path_prompt(cx: &mut TestAppContext) {
        let runtime = JsRuntime::new().expect("QuickJS runtime");
        let applier = Applier::from_runtime(runtime, vello::peniko::Color::TRANSPARENT);
        let (view, cx) = cx.add_window_view(move |window, cx| {
            GpuiRuntimeView::new(applier, "Dialog test".into(), window, cx)
        });

        let completion = cx.update(|window, app| {
            view.update(app, |view, cx| {
                view.execute_effect(
                    EffectRequest {
                        id: EffectId(3),
                        scope: EffectScope::Runtime,
                        payload: EffectPayload::DialogOpen(OpenDialogRequest {
                            title: Some("Choose source".into()),
                            multiple: true,
                            ..Default::default()
                        }),
                    },
                    window,
                    cx,
                )
            })
        });
        assert!(completion.is_none(), "path prompts complete asynchronously");
        assert!(cx.did_prompt_for_paths());

        cx.simulate_path_prompt_response(|options| {
            assert!(options.files);
            assert!(!options.directories);
            assert!(options.multiple);
            assert_eq!(options.prompt.as_deref(), Some("Choose source"));
            Some(vec!["/tmp/a.txt".into(), "/tmp/b.txt".into()])
        });
        cx.run_until_parked();
        assert!(!cx.did_prompt_for_paths());
    }

    #[test]
    fn message_prompt_buttons_preserve_wabou_result_names() {
        let (_, results) = message_prompt_buttons(wabou_shell::MessageDialogButtons::YesNoCancel);
        assert_eq!(results, ["yes", "no", "cancel"]);
        assert!(matches!(
            message_prompt_level(wabou_shell::MessageDialogLevel::Error),
            PromptLevel::Critical
        ));
    }
}
