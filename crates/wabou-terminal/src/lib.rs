use std::{ops::Range, sync::Arc};

use gpui::{AppContext as _, IntoElement as _, ParentElement as _, Styled as _};
use wabou_shell::{NativeWidgetContext, NativeWidgetInput, NativeWidgetMount, gpui};
use wabou_shell_api::{KeyEvent, KeyLocation, KeyPhase, Modifiers, UiEvent, WakeCallback};

use wabou_terminal_core::{TerminalColor, TerminalInputResult, TerminalWidget};

#[cfg(target_os = "macos")]
const PLATFORM_MONOSPACE_FONT: &str = "Menlo";
#[cfg(target_os = "windows")]
const PLATFORM_MONOSPACE_FONT: &str = "Consolas";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const PLATFORM_MONOSPACE_FONT: &str = "DejaVu Sans Mono";

fn terminal_font_family(value: &str) -> &str {
    match value.trim() {
        "" | "monospace" | "ui-monospace" => PLATFORM_MONOSPACE_FONT,
        _ => value,
    }
}

fn gpui_color(color: TerminalColor) -> gpui::Rgba {
    let [r, g, b, a] = color.components();
    gpui::rgba((u32::from(r) << 24) | (u32::from(g) << 16) | (u32::from(b) << 8) | u32::from(a))
}

struct GpuiTerminal {
    terminal: TerminalWidget,
    focus: gpui::FocusHandle,
    _wake_task: gpui::Task<()>,
}

impl NativeWidgetInput for GpuiTerminal {
    fn handle_native_input(
        &mut self,
        event: &UiEvent,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> bool {
        let result = self.terminal.dispatch_native_event(event);
        self.apply_input_result(result, cx)
    }
}

impl GpuiTerminal {
    fn apply_input_result(
        &mut self,
        result: TerminalInputResult,
        cx: &mut gpui::Context<Self>,
    ) -> bool {
        match result {
            TerminalInputResult::Clipboard(wabou_shell_api::ClipboardRequest::Write(text)) => {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
                true
            }
            TerminalInputResult::Clipboard(wabou_shell_api::ClipboardRequest::Read) => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    let _ = self.terminal.dispatch_native_event(&UiEvent::Paste(text));
                }
                true
            }
            result => result.is_handled(),
        }
    }

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
        if !context.attributes_changed() {
            return;
        }
        for (name, value) in context.attributes() {
            if name == "font-family" {
                self.terminal
                    .apply_native_attribute(name, terminal_font_family(value));
            } else {
                self.terminal.apply_native_attribute(name, value);
            }
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
            .bg(gpui::transparent_black())
            .text_size(gpui::px(self.terminal.font_size()))
            .line_height(gpui::px(self.terminal.line_height()))
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
                            let result = state.terminal.dispatch_native_event(&UiEvent::Key(key));
                            if state.apply_input_result(result, cx) {
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
                                let style = gpui::TextStyle {
                                    color: gpui::rgb_to_hsla(gpui::rgb(0xe5e7eb)),
                                    font_family: state.terminal.font_family().to_owned().into(),
                                    ..Default::default()
                                };
                                let font_size = gpui::px(state.terminal.font_size());
                                let sample: gpui::SharedString = "0".into();
                                let sample_run = style.to_run(sample.len());
                                let sample = window.text_system().shape_line(
                                    sample,
                                    font_size,
                                    &[sample_run],
                                    None,
                                );
                                state.terminal.set_font_metrics(
                                    sample.width().into(),
                                    state.terminal.line_height(),
                                );
                                let frame = state.terminal.snapshot_frame(
                                    bounds.size.width.into(),
                                    bounds.size.height.into(),
                                    window.scale_factor() as f64,
                                );
                                let font_size = gpui::px(frame.font_size);
                                let mut shaped = Vec::new();
                                for (row, terminal_row) in frame.rows.into_iter().enumerate() {
                                    for cell in terminal_row.cells {
                                        let text: gpui::SharedString = cell.text.into();
                                        let mut style = gpui::TextStyle {
                                            color: gpui::rgb_to_hsla(gpui_color(cell.foreground)),
                                            font_family: frame.font_family.to_string().into(),
                                            ..Default::default()
                                        };
                                        style.font_weight = if cell.bold {
                                            gpui::FontWeight::BOLD
                                        } else {
                                            gpui::FontWeight::NORMAL
                                        };
                                        style.font_style = if cell.italic {
                                            gpui::FontStyle::Italic
                                        } else {
                                            gpui::FontStyle::Normal
                                        };
                                        let run = style.to_run(text.len());
                                        let line = window.text_system().shape_line(
                                            text,
                                            font_size,
                                            &[run],
                                            None,
                                        );
                                        shaped.push((row, cell.column, cell.background, line));
                                    }
                                }
                                (
                                    shaped,
                                    frame.background,
                                    frame.cell_width,
                                    frame.line_height,
                                )
                            })
                        }
                    },
                    {
                        let entity = entity.clone();
                        let focus = self.focus.clone();
                        move |bounds,
                              (cells, frame_background, cell_width, line_height),
                              window,
                              cx| {
                            window.handle_input(
                                &focus,
                                gpui::ElementInputHandler::new(bounds, entity),
                                cx,
                            );
                            window.paint_quad(gpui::fill(bounds, gpui_color(frame_background)));
                            let cell_width = gpui::px(cell_width);
                            let line_height = gpui::px(line_height);
                            for (row, column, cell_background, line) in cells {
                                let cell_bounds = gpui::Bounds {
                                    origin: gpui::point(
                                        bounds.left() + cell_width * column as f32,
                                        bounds.top() + line_height * row as f32,
                                    ),
                                    size: gpui::size(cell_width, line_height),
                                };
                                if cell_background != frame_background {
                                    window.paint_quad(gpui::fill(
                                        cell_bounds,
                                        gpui_color(cell_background),
                                    ));
                                }
                                let origin = gpui::point(cell_bounds.left(), cell_bounds.top());
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
                terminal.apply_native_attribute("font-family", PLATFORM_MONOSPACE_FONT);
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
        NativeWidgetMount::interactive_entity(
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

    #[test]
    fn generic_monospace_alias_uses_a_platform_font() {
        assert_eq!(terminal_font_family("monospace"), PLATFORM_MONOSPACE_FONT);
        assert_eq!(
            terminal_font_family("ui-monospace"),
            PLATFORM_MONOSPACE_FONT
        );
        assert_eq!(terminal_font_family("Hack"), "Hack");
    }

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
            let mut attributes = BTreeMap::new();
            attributes.insert("font-family".into(), "monospace".into());
            let input = std::rc::Rc::new(|_, _: &mut gpui::App| {});
            let first = factory(
                NativeWidgetContext::new(
                    wabou_host_api::NodeKey::new(9, 2),
                    &attributes,
                    None,
                    None,
                    input.clone(),
                ),
                window,
                cx,
            );
            let (_, retained, native_input) = first.into_parts();
            let retained = retained.expect("terminal factory retains state");
            assert!(
                native_input.is_some(),
                "terminal factory retains an input path"
            );
            let terminal = retained
                .clone()
                .downcast::<GpuiTerminal>()
                .expect("terminal entity type");
            assert_eq!(
                terminal.read(cx).terminal.font_family(),
                PLATFORM_MONOSPACE_FONT,
                "the native terminal must shape text with a real platform monospace family"
            );
            let first_id = retained.entity_id();
            let second = factory(
                NativeWidgetContext::new(
                    wabou_host_api::NodeKey::new(9, 2),
                    &attributes,
                    None,
                    Some(&retained),
                    input,
                ),
                window,
                cx,
            );
            let (_, retained, native_input) = second.into_parts();
            assert!(
                native_input.is_some(),
                "reused terminal retains an input path"
            );
            assert_eq!(
                retained.expect("reused terminal state").entity_id(),
                first_id
            );
            Harness
        });
    }
}
