use std::{ops::Range, sync::Arc};

use gpui::{AppContext as _, IntoElement as _, ParentElement as _, Styled as _};
use wabou_shell_api::{KeyEvent, KeyLocation, KeyPhase, Modifiers, UiEvent, WakeCallback};
use wabou_shell_gpui::{NativeWidgetContext, NativeWidgetMount, gpui};

use super::TerminalWidget;

struct GpuiTerminal {
    terminal: TerminalWidget,
    focus: gpui::FocusHandle,
    _wake_task: gpui::Task<()>,
}

impl GpuiTerminal {
    fn key_event(event: &gpui::KeyDownEvent) -> KeyEvent {
        let source = &event.keystroke.modifiers;
        let mut modifiers = Modifiers::empty();
        modifiers.set(Modifiers::SHIFT, source.shift);
        modifiers.set(Modifiers::CONTROL, source.control);
        modifiers.set(Modifiers::ALT, source.alt);
        modifiers.set(Modifiers::META, source.platform);
        KeyEvent {
            phase: KeyPhase::Down,
            key: event
                .keystroke
                .key_char
                .clone()
                .unwrap_or_else(|| event.keystroke.key.clone()),
            key_without_modifiers: event.keystroke.key.clone(),
            code: event.keystroke.key.clone(),
            text: event.keystroke.key_char.clone(),
            text_with_all_modifiers: event.keystroke.key_char.clone(),
            location: KeyLocation::Standard,
            modifiers,
            repeat: event.is_held,
            synthetic: false,
        }
    }

    fn update_attributes(&mut self, context: &NativeWidgetContext<'_>) {
        for (name, value) in context.attributes() {
            self.terminal.apply_native_attribute(name, value);
        }
    }
}

impl gpui::Focusable for GpuiTerminal {
    fn focus_handle(&self, _cx: &gpui::App) -> gpui::FocusHandle {
        self.focus.clone()
    }
}

impl gpui::EntityInputHandler for GpuiTerminal {
    fn text_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        _actual_range: &mut Option<Range<usize>>,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> Option<String> {
        Some(String::new())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> Option<gpui::UTF16Selection> {
        Some(gpui::UTF16Selection {
            range: 0..0,
            reversed: false,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> Option<Range<usize>> {
        None
    }

    fn unmark_text(&mut self, _window: &mut gpui::Window, _cx: &mut gpui::Context<Self>) {}

    fn replace_text_in_range(
        &mut self,
        _range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        if !new_text.is_empty() {
            let _ = self
                .terminal
                .dispatch_native_event(&UiEvent::TextInput(new_text.into()));
            cx.notify();
        }
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _new_selected_range_utf16: Option<Range<usize>>,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        self.replace_text_in_range(range_utf16, new_text, window, cx);
    }

    fn bounds_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> Option<gpui::Bounds<gpui::Pixels>> {
        Some(bounds)
    }

    fn character_index_for_point(
        &mut self,
        _point: gpui::Point<gpui::Pixels>,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> Option<usize> {
        Some(0)
    }
}

impl gpui::Render for GpuiTerminal {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        use gpui::{InteractiveElement as _, ParentElement as _, Styled as _};

        let entity = cx.entity();
        let focus = self.focus.clone();
        gpui::div()
            .id(("wabou-terminal", entity.entity_id()))
            .relative()
            .size_full()
            .overflow_hidden()
            .bg(gpui::rgb(0x111827))
            .text_color(gpui::rgb(0xe5e7eb))
            .font_family("monospace")
            .text_size(gpui::px(self.terminal.font_size))
            .line_height(gpui::px(self.terminal.line_height))
            .track_focus(&self.focus)
            .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                window.focus(&focus, cx);
                cx.stop_propagation();
            })
            .on_key_down({
                let entity = entity.clone();
                move |event, _, cx| {
                    let key = Self::key_event(event);
                    let has_text = key.text.as_ref().is_some_and(|text| !text.is_empty());
                    if !has_text
                        || key.modifiers.control()
                        || key.modifiers.alt()
                        || key.modifiers.meta()
                    {
                        entity.update(cx, |state, cx| {
                            if state.terminal.dispatch_native_event(&UiEvent::Key(key)) {
                                cx.notify();
                            }
                        });
                        cx.stop_propagation();
                    }
                }
            })
            .child(
                gpui::canvas(
                    {
                        let entity = entity.clone();
                        move |bounds, window, cx| {
                            entity.update(cx, |state, _cx| {
                                let lines = state.terminal.gpui_visible_text(
                                    bounds.size.width.into(),
                                    bounds.size.height.into(),
                                );
                                let style = gpui::TextStyle {
                                    color: gpui::rgb_to_hsla(gpui::rgb(0xe5e7eb)),
                                    font_family: "monospace".into(),
                                    ..Default::default()
                                };
                                let font_size = gpui::px(state.terminal.font_size);
                                let shaped = lines
                                    .into_iter()
                                    .map(|line| {
                                        let line: gpui::SharedString = line.into();
                                        let run = style.to_run(line.len());
                                        window.text_system().shape_line(
                                            line,
                                            font_size,
                                            &[run],
                                            None,
                                        )
                                    })
                                    .collect::<Vec<_>>();
                                (shaped, state.terminal.line_height)
                            })
                        }
                    },
                    {
                        let entity = entity.clone();
                        let focus = self.focus.clone();
                        move |bounds, (lines, line_height), window, cx| {
                            window.handle_input(
                                &focus,
                                gpui::ElementInputHandler::new(bounds, entity),
                                cx,
                            );
                            let line_height = gpui::px(line_height);
                            for (index, line) in lines.iter().enumerate() {
                                let origin = gpui::point(
                                    bounds.left(),
                                    bounds.top() + line_height * index as f32,
                                );
                                let _ = line.paint(
                                    origin,
                                    line_height,
                                    gpui::TextAlign::Left,
                                    None,
                                    window,
                                    cx,
                                );
                            }
                        }
                    },
                )
                .size_full(),
            )
    }
}

/// Factory for the GPUI-native terminal widget.
pub fn gpui_terminal_factory()
-> impl for<'a> Fn(NativeWidgetContext<'a>, &mut gpui::Window, &mut gpui::App) -> NativeWidgetMount
+ Send
+ Sync
+ 'static {
    move |context, _window, cx| {
        let entity = context.entity::<GpuiTerminal>().unwrap_or_else(|| {
            let (sender, receiver) = flume::bounded::<()>(1);
            cx.new(|entity_cx: &mut gpui::Context<GpuiTerminal>| {
                let mut terminal = TerminalWidget::lazy_default_shell();
                let wake: WakeCallback = Arc::new(move || {
                    let _ = sender.try_send(());
                });
                terminal.install_native_wake(wake);
                let task = entity_cx.spawn(async move |view, cx| {
                    while receiver.recv_async().await.is_ok() {
                        if view
                            .update(cx, |view, cx| {
                                let _ = view.terminal.poll_native_events();
                                cx.notify();
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                });
                GpuiTerminal {
                    terminal,
                    focus: entity_cx.focus_handle(),
                    _wake_task: task,
                }
            })
        });
        entity.update(cx, |state, _| state.update_attributes(&context));
        NativeWidgetMount::entity(
            entity.clone(),
            gpui::div().size_full().child(entity).into_any_element(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use gpui::TestAppContext;

    struct Harness;

    impl gpui::Render for Harness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut gpui::Context<Self>,
        ) -> impl gpui::IntoElement {
            gpui::div()
        }
    }

    #[gpui::test]
    fn factory_reuses_the_entity_retained_for_the_same_node(cx: &mut TestAppContext) {
        let factory = gpui_terminal_factory();
        let (_view, _cx) = cx.add_window_view(move |window, cx| {
            let attributes = BTreeMap::new();
            let first = factory(
                NativeWidgetContext::new(wabou_host_api::NodeKey::new(9, 2), &attributes, None),
                window,
                cx,
            );
            let (_, retained) = first.into_parts();
            let retained = retained.expect("terminal factory retains state");
            let first_id = retained.entity_id();
            let second = factory(
                NativeWidgetContext::new(
                    wabou_host_api::NodeKey::new(9, 2),
                    &attributes,
                    Some(&retained),
                ),
                window,
                cx,
            );
            let (_, retained) = second.into_parts();
            assert_eq!(
                retained.expect("reused terminal state").entity_id(),
                first_id
            );
            Harness
        });
    }
}
