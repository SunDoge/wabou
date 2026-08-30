//! Backend-neutral results produced by the terminal session state machine.

use super::*;

/// Backend-neutral snapshot of the terminal viewport.
///
/// Renderers own shaping and painting. The terminal session only determines
/// which logical rows are visible and the metrics used to size the PTY grid.
#[derive(Debug, Clone, PartialEq)]
pub struct TerminalFrame {
    pub lines: Vec<String>,
    pub font_size: f32,
    pub line_height: f32,
    pub cell_width: f32,
}

/// An event emitted by the terminal session toward its JavaScript owner.
///
/// The session owns this DTO so it does not depend on a particular native
/// widget backend. Adapters translate it into their backend event envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalNodeEvent {
    pub kind: TerminalEventKind,
    pub json: String,
}

impl TerminalNodeEvent {
    pub(super) fn json(kind: TerminalEventKind, json: impl Into<String>) -> Self {
        Self {
            kind,
            json: json.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalEventKind {
    Exit,
    Progress,
    Notification,
    TitleChange,
    CurrentDirectoryChange,
    SelectionChange,
    Bell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TerminalInputResult {
    Ignored,
    Handled,
    HandledConsumingText,
    Clipboard(wabou_shell_api::ClipboardRequest),
}

impl TerminalInputResult {
    pub(super) const fn is_handled(&self) -> bool {
        !matches!(self, Self::Ignored)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct TerminalInvalidation {
    pub(super) measure: bool,
    pub(super) redraw: bool,
}

impl TerminalInvalidation {
    pub(super) const REDRAW: Self = Self {
        measure: false,
        redraw: true,
    };
    pub(super) const MEASURE_AND_REDRAW: Self = Self {
        measure: true,
        redraw: true,
    };
}

impl TerminalWidget {
    /// Update renderer-measured cell metrics without exposing a backend text
    /// context to the terminal session.
    pub fn set_font_metrics(&mut self, cell_width: f32, line_height: f32) {
        if cell_width.is_finite() && cell_width > 0.0 {
            self.cell_width = cell_width;
        }
        if line_height.is_finite() && line_height > 0.0 {
            self.line_height = line_height.max(self.font_size);
        }
        self.metrics_dirty = false;
    }

    pub(super) fn apply_native_attribute(
        &mut self,
        name: &str,
        value: &str,
    ) -> TerminalInvalidation {
        match name {
            "command" if !self.launch_started || self.spawn_error.is_some() => {
                self.launch_started = false;
                self.spawn_error = None;
                let launch = self.launch.get_or_insert_with(LaunchConfig::default_shell);
                launch.login_shell = value.is_empty();
                launch.command = if value.is_empty() {
                    default_shell_command()
                } else {
                    value.to_owned()
                };
            }
            "args" if !self.launch_started || self.spawn_error.is_some() => {
                match serde_json::from_str::<Vec<String>>(value) {
                    Ok(args) => {
                        self.launch_started = false;
                        self.launch
                            .get_or_insert_with(LaunchConfig::default_shell)
                            .args = args;
                        self.launch_error = None;
                        self.spawn_error = None;
                    }
                    Err(error) => {
                        let message = format!("invalid terminal args JSON: {error}");
                        self.launch_error = Some(message.clone());
                        self.spawn_error = Some(message);
                    }
                }
            }
            "cwd" if !self.launch_started || self.spawn_error.is_some() => {
                self.launch_started = false;
                self.spawn_error = None;
                self.launch
                    .get_or_insert_with(LaunchConfig::default_shell)
                    .cwd = (!value.is_empty()).then(|| value.to_owned());
            }
            "command" | "args" | "cwd" => {
                tracing::warn!(
                    attribute = name,
                    "ignored terminal launch option after PTY start"
                );
            }
            "font-size" => {
                if let Ok(size) = value.trim_end_matches("px").parse::<f32>() {
                    self.font_size = size.max(6.0);
                    self.metrics_dirty = true;
                }
            }
            "line-height" => {
                if let Ok(height) = value.trim_end_matches("px").parse::<f32>() {
                    self.explicit_line_height = Some(height.max(0.0));
                    self.metrics_dirty = true;
                }
            }
            "font-family" => {
                self.font_family = Arc::from(value);
                self.metrics_dirty = true;
            }
            "allow-clipboard-read" => {
                self.allow_clipboard_read = matches!(value, "" | "true" | "1");
            }
            "cursor-blink" => {
                self.cursor_blink = Some(matches!(value, "" | "true" | "1"));
                let terminal_blinking = self.terminal.lock().blinking_cursor;
                self.schedule_cursor_blink(terminal_blinking);
            }
            "sync-window-title" => {
                let enabled = matches!(value, "" | "true" | "1");
                if self.sync_window_title && !enabled {
                    self.pending_host_actions
                        .push_back(HostAction::SetWindowTitle(None));
                }
                self.sync_window_title = enabled;
            }
            "selection-background" => {
                if let Some(color) = wabou_shell::style::parse_color(value) {
                    self.selection_background = color;
                }
            }
            "selection-foreground" => {
                if let Some(color) = wabou_shell::style::parse_color(value) {
                    self.selection_foreground = Some(color);
                }
            }
            "inherit-theme" => {
                self.inherit_theme = matches!(value, "" | "true" | "1");
                if !self.inherit_theme {
                    self.theme_foreground = named_color(NamedColor::Foreground, true);
                    self.theme_background = named_color(NamedColor::Background, false);
                }
            }
            _ => {}
        }
        match name {
            "font-size" | "line-height" | "font-family" => TerminalInvalidation::MEASURE_AND_REDRAW,
            "command"
            | "args"
            | "cwd"
            | "selection-background"
            | "selection-foreground"
            | "inherit-theme"
            | "cursor-blink" => TerminalInvalidation::REDRAW,
            _ => TerminalInvalidation::default(),
        }
    }

    pub(super) fn dispatch_native_event(&mut self, event: &UiEvent) -> TerminalInputResult {
        self.handle_native_event(event)
    }

    pub(super) fn install_native_wake(&mut self, wake: WakeCallback) {
        self.listener.set_wake(wake);
    }

    pub(super) fn poll_native_events(&mut self) -> bool {
        self.handle_rio_events()
    }

    pub(super) fn handle_native_event(&mut self, event: &UiEvent) -> TerminalInputResult {
        match event {
            UiEvent::Pointer(pointer) => self.handle_pointer_event(pointer),
            UiEvent::TextInput(text) | UiEvent::Ime(ImeEvent::Commit(text)) => {
                if self.exit_reported {
                    return TerminalInputResult::Handled;
                }
                self.begin_terminal_input();
                self.send_bytes(text.as_bytes().to_vec());
                TerminalInputResult::Handled
            }
            UiEvent::Paste(text) => {
                if self.exit_reported {
                    return TerminalInputResult::Handled;
                }
                let bracketed = self.terminal.lock().mode().contains(Mode::BRACKETED_PASTE);
                self.begin_terminal_input();
                self.send_bytes(encode_paste(text, bracketed));
                TerminalInputResult::Handled
            }
            UiEvent::Key(key) => self.handle_key_event(key),
            UiEvent::Wheel(wheel) => self.handle_wheel_event(wheel),
            _ => TerminalInputResult::Ignored,
        }
    }

    pub fn snapshot_frame(&mut self, width: f32, height: f32, device_scale: f64) -> TerminalFrame {
        self.resize(width, height, device_scale);
        self.ensure_launched();
        self.update_cursor_blink();
        let mut terminal = self.terminal.lock();
        let damage = terminal.peek_damage_event().unwrap_or(TerminalDamage::Noop);
        terminal.snapshot_visible(
            &damage,
            self.size.columns,
            &mut self.visible_rows,
            &mut self.visible_styles,
            &mut self.visible_extras,
        );
        let _ = terminal.damage();
        terminal.reset_damage();
        let lines = self
            .visible_rows
            .iter()
            .map(|row| {
                let mut text = String::with_capacity(self.size.columns);
                for column in 0..self.size.columns.min(row.inner.len()) {
                    let square = row[Column(column)];
                    if !matches!(square.wide(), Wide::Spacer | Wide::LeadingSpacer) {
                        let character = square.c();
                        text.push(if character.is_control() {
                            ' '
                        } else {
                            character
                        });
                    }
                }
                text.trim_end().to_owned()
            })
            .collect();
        TerminalFrame {
            lines,
            font_size: self.font_size,
            line_height: self.line_height,
            cell_width: self.cell_width,
        }
    }

    fn cursor_blinking(&self, terminal_blinking: bool) -> bool {
        self.cursor_blink.unwrap_or(terminal_blinking)
    }

    pub(super) fn schedule_cursor_blink(&mut self, terminal_blinking: bool) {
        let blinking =
            self.focused && !self.exit_reported && self.cursor_blinking(terminal_blinking);
        self.cursor_on = true;
        self.next_cursor_blink = blinking.then(|| Instant::now() + Duration::from_millis(500));
    }

    pub(super) fn update_cursor_blink(&mut self) {
        if self.focused
            && self
                .next_cursor_blink
                .is_some_and(|time| Instant::now() >= time)
        {
            self.cursor_on = !self.cursor_on;
            self.next_cursor_blink = Some(Instant::now() + Duration::from_millis(500));
        }
    }

    fn handle_pointer_event(
        &mut self,
        pointer: &wabou_shell_api::PointerEvent,
    ) -> TerminalInputResult {
        if pointer.phase == PointerPhase::Down && pointer.button != Some(PointerButton::Primary) {
            self.last_click = None;
        }
        if pointer.phase == PointerPhase::Down
            && pointer.button == Some(PointerButton::Primary)
            && terminal_primary_shortcut(pointer.modifiers)
            && let Some(url) = self.hyperlink_at(pointer.position.x, pointer.position.y)
        {
            self.last_click = None;
            self.pending_hyperlink = Some(PendingHyperlink {
                url,
                origin: (pointer.position.x, pointer.position.y),
                cancelled: false,
            });
            return TerminalInputResult::Handled;
        }
        if let Some(pending) = self.pending_hyperlink.as_mut()
            && pointer.phase == PointerPhase::Move
        {
            let distance = (pointer.position.x - pending.origin.0)
                .hypot(pointer.position.y - pending.origin.1);
            pending.cancelled |= distance > SELECTION_DRAG_THRESHOLD;
            return TerminalInputResult::Handled;
        }
        if pointer.phase == PointerPhase::Up
            && let Some(pending) = self.pending_hyperlink.take()
        {
            if !pending.cancelled
                && self
                    .hyperlink_at(pointer.position.x, pointer.position.y)
                    .as_deref()
                    == Some(pending.url.as_str())
            {
                self.pending_host_actions
                    .push_back(HostAction::OpenUrl(pending.url));
            }
            return TerminalInputResult::Handled;
        }
        if pointer.phase == PointerPhase::Cancel && self.pending_hyperlink.take().is_some() {
            return TerminalInputResult::Handled;
        }
        if !self.selecting && self.report_pointer(pointer) {
            return TerminalInputResult::Handled;
        }
        match (pointer.phase, pointer.button, self.selecting) {
            (PointerPhase::Down, Some(PointerButton::Primary), _) => {
                self.begin_or_extend_selection(
                    pointer.position.x,
                    pointer.position.y,
                    pointer.modifiers,
                );
                TerminalInputResult::Handled
            }
            (PointerPhase::Move, _, true) => {
                self.update_selection(pointer.position.x, pointer.position.y);
                TerminalInputResult::Handled
            }
            (PointerPhase::Up, _, true) => {
                self.update_selection(pointer.position.x, pointer.position.y);
                self.finish_selection_gesture();
                TerminalInputResult::Handled
            }
            (PointerPhase::Cancel, _, true) => {
                self.last_click = None;
                self.finish_selection_gesture();
                TerminalInputResult::Handled
            }
            _ => TerminalInputResult::Ignored,
        }
    }

    fn handle_key_event(&mut self, key: &wabou_shell_api::KeyEvent) -> TerminalInputResult {
        if key.phase == KeyPhase::Down {
            self.last_click = None;
        }
        if terminal_clipboard_shortcut(key.modifiers) && key.key.eq_ignore_ascii_case("a") {
            if key.phase == KeyPhase::Down {
                self.select_all();
            }
            return TerminalInputResult::Handled;
        }
        if terminal_clipboard_shortcut(key.modifiers) && key.key.eq_ignore_ascii_case("c") {
            return if key.phase == KeyPhase::Down {
                self.selected_text()
                    .map_or(TerminalInputResult::Handled, |text| {
                        TerminalInputResult::Clipboard(wabou_shell_api::ClipboardRequest::Write(
                            text,
                        ))
                    })
            } else {
                TerminalInputResult::Handled
            };
        }
        if terminal_clipboard_shortcut(key.modifiers) && key.key.eq_ignore_ascii_case("v") {
            return if self.exit_reported {
                TerminalInputResult::Handled
            } else if key.phase == KeyPhase::Down {
                TerminalInputResult::Clipboard(wabou_shell_api::ClipboardRequest::Read)
            } else {
                TerminalInputResult::Handled
            };
        }
        let mode = self.terminal.lock().mode();
        if key.phase == KeyPhase::Down
            && key.modifiers == Modifiers::SHIFT
            && !mode.contains(Mode::ALT_SCREEN)
            && let Some(scroll) = scrollback_key(&key.key)
        {
            self.terminal.lock().scroll_display(scroll);
            return TerminalInputResult::Handled;
        }
        if self.exit_reported {
            return TerminalInputResult::Handled;
        }
        let bytes = self.key_bytes(key);
        if bytes.is_empty() {
            return TerminalInputResult::Ignored;
        }
        self.begin_terminal_input();
        self.send_bytes(bytes);
        if key.phase == KeyPhase::Down {
            TerminalInputResult::HandledConsumingText
        } else {
            TerminalInputResult::Handled
        }
    }

    fn handle_wheel_event(&mut self, wheel: &wabou_shell_api::WheelEvent) -> TerminalInputResult {
        self.last_click = None;
        let context = self.wheel_context(wheel);
        let lines = self.wheel_lines.push(context, wheel.delta_y);
        if self.selecting {
            self.scroll_active_selection(wheel, lines);
        } else if !self.report_wheel(wheel, lines)
            && !self.report_alternate_scroll(lines)
            && lines != 0
        {
            self.terminal.lock().scroll_display(Scroll::Delta(lines));
        }
        // Fractional trackpad input remains terminal-owned until it reaches a
        // complete grid line.
        TerminalInputResult::Handled
    }
}

fn scrollback_key(key: &str) -> Option<Scroll> {
    match key {
        "Home" => Some(Scroll::Top),
        "End" => Some(Scroll::Bottom),
        "PageUp" => Some(Scroll::PageUp),
        "PageDown" => Some(Scroll::PageDown),
        _ => None,
    }
}
