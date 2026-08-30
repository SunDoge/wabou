//! GPUI view owning one Wabou JavaScript runtime.

use std::{rc::Rc, sync::Arc};

use wabou_shell_gpui::WakeCallback;
use wabou_shell_gpui::gpui::{ClipboardItem, Context, FocusHandle, Render, Task, Window};

use crate::{Applier, FrameSource};
use wabou_shell::{ClipboardRequest, HostAction, HostActionResult, UiEvent};

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
}
