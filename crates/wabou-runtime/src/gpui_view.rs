//! GPUI view owning one Wabou JavaScript runtime.

use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
    sync::Arc,
};

use gpui_base::input::{Input, InputEvent, InputState, Textarea, TextareaState};
use gpui_shell::WakeCallback;
use gpui_shell::gpui::{
    AppContext as _, ClipboardItem, Context, Entity, FocusHandle, IntoElement as _,
    ParentElement as _, PathPromptOptions, PromptButton, PromptLevel, Render, Styled as _,
    Subscription, SystemNotification, Task, Window, div,
};

use crate::Applier;
use gpui_shell::{
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
    text_controls: BTreeMap<wabou_host_api::NodeKey, GpuiTextControlState>,
    native_widget_entities: BTreeMap<wabou_host_api::NodeKey, gpui_shell::gpui::AnyEntity>,
    window_size_persistence: Option<gpui_shell::WindowSizePersistence>,
    native_widget_factories: HashMap<String, gpui_shell::NativeWidgetFactory>,
}

enum GpuiTextControlState {
    Input {
        state: Entity<InputState>,
        _subscription: Subscription,
    },
    Textarea {
        state: Entity<TextareaState>,
        _subscription: Subscription,
    },
}

impl GpuiTextControlState {
    fn is_multiline(&self) -> bool {
        matches!(self, Self::Textarea { .. })
    }

    fn synchronize(
        &self,
        descriptor: &gpui_shell::GpuiTextControl,
        window: &mut Window,
        cx: &mut Context<GpuiRuntimeView>,
    ) {
        macro_rules! synchronize {
            ($state:expr) => {
                $state.update(cx, |state, cx| {
                    if state.value().as_ref() != descriptor.value {
                        state.set_value(descriptor.value.clone(), window, cx);
                    }
                    if state.presentation().placeholder().as_ref() != descriptor.placeholder {
                        state.set_placeholder(descriptor.placeholder.clone(), window, cx);
                    }
                    state.set_disabled(descriptor.disabled, cx);
                    state.set_readonly(descriptor.readonly, cx);
                })
            };
        }
        match self {
            Self::Input { state, .. } => synchronize!(state),
            Self::Textarea { state, .. } => synchronize!(state),
        }
    }

    fn element(&self) -> gpui_shell::gpui::AnyElement {
        match self {
            Self::Input { state, .. } => div()
                .size_full()
                .child(Input::new(state))
                .into_any_element(),
            Self::Textarea { state, .. } => div()
                .size_full()
                .child(Textarea::new(state))
                .into_any_element(),
        }
    }
}

impl GpuiRuntimeView {
    /// Wrap an already configured and booted Wabou runtime.
    #[must_use]
    pub fn new(
        mut applier: Applier,
        default_title: String,
        window_size_persistence: Option<gpui_shell::WindowSizePersistence>,
        native_widget_factories: HashMap<String, gpui_shell::NativeWidgetFactory>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let (wake, receiver) = gpui_wake_channel();
        applier.install_runtime_wake(wake);

        let wake_task = cx.spawn(async move |view, cx| {
            while receiver.recv_async().await.is_ok() {
                if view
                    .update(cx, |view, cx| {
                        // Drain work immediately instead of waiting for an unrelated
                        // input event or animation frame. The following render pass
                        // publishes any resulting Solid mutation batch to GPUI.
                        let _ = view.applier.poll_runtime();
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
            text_controls: BTreeMap::new(),
            native_widget_entities: BTreeMap::new(),
            window_size_persistence,
            native_widget_factories,
        }
    }

    fn synchronize_text_controls(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let descriptors = self.applier.gpui_text_controls();
        self.text_controls
            .retain(|key, _| descriptors.iter().any(|descriptor| descriptor.key == *key));

        for descriptor in descriptors {
            let needs_recreate = self
                .text_controls
                .get(&descriptor.key)
                .is_some_and(|state| state.is_multiline() != descriptor.multiline);
            if needs_recreate {
                self.text_controls.remove(&descriptor.key);
            }
            if let std::collections::btree_map::Entry::Vacant(entry) =
                self.text_controls.entry(descriptor.key)
            {
                let key = descriptor.key;
                let control =
                    if descriptor.multiline {
                        let state = cx.new(|cx| TextareaState::new(window, cx));
                        let subscription = cx.subscribe(&state, move |view, state, event, cx| {
                            let value = matches!(event, InputEvent::Change)
                                .then(|| state.read(cx).value().to_string());
                            let focused = match event {
                                InputEvent::Focus => Some(true),
                                InputEvent::Blur => Some(false),
                                _ => None,
                            };
                            let changed = value.as_deref().is_some_and(|value| {
                                view.applier.gpui_commit_text_value(key, value)
                            }) | focused.is_some_and(|focused| {
                                view.applier.gpui_set_text_focus(key, focused)
                            });
                            if changed {
                                cx.notify();
                            }
                        });
                        GpuiTextControlState::Textarea {
                            state,
                            _subscription: subscription,
                        }
                    } else {
                        let state = cx.new(|cx| InputState::new(window, cx));
                        let subscription = cx.subscribe(&state, move |view, state, event, cx| {
                            let value = matches!(event, InputEvent::Change)
                                .then(|| state.read(cx).value().to_string());
                            let focused = match event {
                                InputEvent::Focus => Some(true),
                                InputEvent::Blur => Some(false),
                                _ => None,
                            };
                            let changed = value.as_deref().is_some_and(|value| {
                                view.applier.gpui_commit_text_value(key, value)
                            }) | focused.is_some_and(|focused| {
                                view.applier.gpui_set_text_focus(key, focused)
                            });
                            if changed {
                                cx.notify();
                            }
                        });
                        GpuiTextControlState::Input {
                            state,
                            _subscription: subscription,
                        }
                    };
                entry.insert(control);
            }
            self.text_controls[&descriptor.key].synchronize(&descriptor, window, cx);
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

    fn handle_input(&mut self, event: gpui_shell::ProjectedInputEvent, cx: &mut Context<Self>) {
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
                        let pasted = self.applier.dispatch_runtime_event(UiEvent::Paste(text));
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
        while let Some(action) = self.applier.take_runtime_host_action() {
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
                    self.applier
                        .complete_runtime_host_action(HostActionResult::ClipboardWrite {
                            request_id,
                            success: true,
                        });
                }
                HostAction::ReadClipboard { request_id } => {
                    let text = cx
                        .read_from_clipboard()
                        .and_then(|clipboard| clipboard.text());
                    self.applier
                        .complete_runtime_host_action(HostActionResult::Clipboard {
                            request_id,
                            text,
                        });
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
        while let Some(request) = self.applier.take_runtime_effect() {
            handled = true;
            let completion = self.execute_effect(request, window, cx);
            if let Some(completion) = completion {
                self.applier.complete_runtime_effect(completion);
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
        id: gpui_shell::EffectId,
        op: gpui_shell::EffectOp,
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
        id: gpui_shell::EffectId,
        op: gpui_shell::EffectOp,
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
                view.applier
                    .complete_runtime_effect(EffectCompletion { id, op, result });
                cx.notify();
            });
        })
        .detach();
    }

    fn complete_save_prompt(
        &self,
        id: gpui_shell::EffectId,
        op: gpui_shell::EffectOp,
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
                view.applier
                    .complete_runtime_effect(EffectCompletion { id, op, result });
                cx.notify();
            });
        })
        .detach();
    }

    fn complete_message_prompt(
        &self,
        id: gpui_shell::EffectId,
        op: gpui_shell::EffectOp,
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
                view.applier
                    .complete_runtime_effect(EffectCompletion { id, op, result });
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

fn message_prompt_level(level: gpui_shell::MessageDialogLevel) -> PromptLevel {
    match level {
        gpui_shell::MessageDialogLevel::Info => PromptLevel::Info,
        gpui_shell::MessageDialogLevel::Warning => PromptLevel::Warning,
        gpui_shell::MessageDialogLevel::Error => PromptLevel::Critical,
    }
}

fn message_prompt_buttons(
    buttons: gpui_shell::MessageDialogButtons,
) -> (Vec<PromptButton>, Vec<&'static str>) {
    match buttons {
        gpui_shell::MessageDialogButtons::Ok => (vec![PromptButton::ok("OK")], vec!["ok"]),
        gpui_shell::MessageDialogButtons::OkCancel => (
            vec![PromptButton::ok("OK"), PromptButton::cancel("Cancel")],
            vec!["ok", "cancel"],
        ),
        gpui_shell::MessageDialogButtons::YesNo => (
            vec![PromptButton::ok("Yes"), PromptButton::cancel("No")],
            vec!["yes", "no"],
        ),
        gpui_shell::MessageDialogButtons::YesNoCancel => (
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
    ) -> impl gpui_shell::gpui::IntoElement {
        let viewport = window.viewport_size();
        if let Some(persistence) = &mut self.window_size_persistence {
            let width: f32 = viewport.width.into();
            let height: f32 = viewport.height.into();
            persistence.observe(
                width.round().max(1.0) as u32,
                height.round().max(1.0) as u32,
                window.is_maximized(),
            );
        }
        let _ = self
            .applier
            .build_gpui_frame(viewport.width.into(), viewport.height.into());
        self.synchronize_text_controls(window, cx);
        if self.drain_host_actions(window, cx) {
            cx.notify();
        }
        if self.drain_effects(window, cx) {
            cx.notify();
        }

        if self.applier.runtime_has_animation() {
            // GPUI associates this request with the currently rendering view
            // and notifies only that entity on the next platform frame.
            window.request_animation_frame();
        }

        let view = cx.weak_entity();
        let input = Rc::new(move |event, cx: &mut gpui_shell::gpui::App| {
            let _ = view.update(cx, |view, cx| {
                view.handle_input(event, cx);
            });
        });
        let mut text_input = self.applier.gpui_text_input_state();
        if self
            .applier
            .gpui_focused_target()
            .is_some_and(|target| self.text_controls.contains_key(&target))
        {
            // The child InputState owns the platform input handler. Keeping the
            // transitional root handler active would commit IME text twice.
            text_input.accepts_text = false;
        }
        let mut native_controls = self
            .text_controls
            .iter()
            .map(|(key, state)| (*key, state.element()))
            .collect::<BTreeMap<_, _>>();
        let widgets = self
            .applier
            .gpui_native_widgets(|tag| self.native_widget_factories.contains_key(tag));
        self.native_widget_entities
            .retain(|key, _| widgets.iter().any(|widget| widget.key == *key));
        for widget in &widgets {
            let factory = self
                .native_widget_factories
                .get(widget.tag.as_ref())
                .expect("native widget descriptors are filtered by the registry");
            let mount = factory(
                gpui_shell::NativeWidgetContext::new(
                    widget.key,
                    &widget.attributes,
                    self.native_widget_entities.get(&widget.key),
                ),
                window,
                cx,
            );
            let (element, entity) = mount.into_parts();
            if let Some(entity) = entity {
                self.native_widget_entities.insert(widget.key, entity);
            } else {
                self.native_widget_entities.remove(&widget.key);
            }
            native_controls.insert(widget.key, element);
        }
        let native_controls = Rc::new(std::cell::RefCell::new(native_controls));
        let native: gpui_shell::ProjectedNativeElementFactory =
            Rc::new(move |key| native_controls.borrow_mut().remove(&key));
        self.applier
            .gpui_interactive_element(input, self.focus.clone(), text_input, Some(native))
            .expect("the canonical Wabou root remains retained")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::JsRuntime;
    use gpui_shell::gpui::{HeadlessAppContext, TestAppContext, px, size};
    use gpui_shell::{EffectId, EffectScope, OpenDialogRequest};
    use wabou_host_api::NodeKey;

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

    #[test]
    fn solid_frame_draws_in_a_real_platform_headless_window() {
        let platform = gpui_platform::current_platform(true);
        let mut cx = HeadlessAppContext::new(platform.text_system());
        let handle = cx
            .open_window(size(px(800.0), px(600.0)), |window, app| {
                let runtime = JsRuntime::new().expect("QuickJS runtime");
                let mut applier = Applier::from_runtime(runtime, vello::peniko::Color::TRANSPARENT);
                applier
                    .boot(include_str!("gen/test-runtime.js"))
                    .expect("boot generated Solid runtime fixture");
                assert!(applier.build_gpui_frame(800, 600));
                app.new(|view_cx| {
                    GpuiRuntimeView::new(
                        applier,
                        "Headless GPUI fixture".into(),
                        None,
                        HashMap::new(),
                        window,
                        view_cx,
                    )
                })
            })
            .expect("open a hidden GPUI window with the platform text system");

        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, app| {
            let _ = window.draw(app);
            assert_eq!(window.bounds().size, size(px(800.0), px(600.0)));
        })
        .expect("layout and draw the projected Solid frame");
    }

    #[gpui_shell::gpui::test]
    fn gpui_effect_executor_uses_the_platform_clipboard(cx: &mut TestAppContext) {
        let runtime = JsRuntime::new().expect("QuickJS runtime");
        let applier = Applier::from_runtime(runtime, vello::peniko::Color::TRANSPARENT);
        let (view, cx) = cx.add_window_view(move |window, cx| {
            GpuiRuntimeView::new(
                applier,
                "Clipboard test".into(),
                None,
                HashMap::new(),
                window,
                cx,
            )
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

    #[gpui_shell::gpui::test]
    fn gpui_effect_executor_uses_the_platform_path_prompt(cx: &mut TestAppContext) {
        let runtime = JsRuntime::new().expect("QuickJS runtime");
        let applier = Applier::from_runtime(runtime, vello::peniko::Color::TRANSPARENT);
        let (view, cx) = cx.add_window_view(move |window, cx| {
            GpuiRuntimeView::new(
                applier,
                "Dialog test".into(),
                None,
                HashMap::new(),
                window,
                cx,
            )
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
        let (_, results) = message_prompt_buttons(gpui_shell::MessageDialogButtons::YesNoCancel);
        assert_eq!(results, ["yes", "no", "cancel"]);
        assert!(matches!(
            message_prompt_level(gpui_shell::MessageDialogLevel::Error),
            PromptLevel::Critical
        ));
    }
}
