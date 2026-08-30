//! A Wabou terminal widget backed by `rio-vt`.
//!
//! `rio-vt` owns terminal semantics (VT parsing, grid, cursor, scrollback and
//! PTY events). This crate is the frontend adapter: it translates Wabou input
//! to PTY bytes and pulls Rio's visible grid into a retained AnyRender scene.

extern crate wabou_backend_winit as wabou_shell;

use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyrender::{PaintScene, Scene};
use rio_vt::ansi::CursorShape;
use rio_vt::config::colors::term::{COUNT as TERMINAL_COLOR_COUNT, TermColors};
use rio_vt::config::colors::{AnsiColor, ColorRgb, NamedColor};
use rio_vt::crosswords::grid::Scroll;
use rio_vt::crosswords::grid::row::Row;
use rio_vt::crosswords::pos::{Column, CursorState, Line, Pos, Side};
use rio_vt::crosswords::square::{ContentTag, Extras, Square, Wide};
use rio_vt::crosswords::style::{Style, StyleFlags};
use rio_vt::crosswords::{Crosswords, CrosswordsSize, Mode};
use rio_vt::event::sync::FairMutex;
use rio_vt::event::{EventListener, Msg, ProgressState, RioEvent, TerminalDamage, WindowId};
use rio_vt::performer::Machine;
use rio_vt::performer::handler::Processor;
use rio_vt::selection::{Selection, SelectionRange, SelectionType};
use rustc_hash::FxHashMap;
use teletypewriter::{WinsizeBuilder, create_pty_with_spawn};
use vello::kurbo::{Affine, Rect, Stroke};
use vello::peniko::{Color, Fill};
#[cfg(test)]
use wabou_runtime::{Widget, WidgetEventResult, WidgetNodeEvent, WidgetStyle, event};
#[cfg(test)]
use wabou_shell::style::Paint;
use wabou_shell::text::{TextContext, layout_text_styled};
use wabou_shell_api::{
    HostAction, HostActionResult, ImeEvent, KeyPhase, Modifiers, PointerButton, PointerPhase,
    UiEvent, WHEEL_LINE_DELTA, WakeCallback,
};

mod box_drawing;
mod gpui_widget;
mod graphics;
mod input_encoding;
mod kitty_keyboard;
mod legacy_widget;
mod process;
mod rendering;
mod selection;
mod session;

pub use gpui_widget::gpui_terminal_factory;
use graphics::{KittyLayer, TerminalGraphics};
use input_encoding::*;
pub use legacy_widget::terminal_widget;
#[cfg(test)]
use process::quote_windows_command_arg;
use process::{
    LaunchConfig, default_shell_command, pty_spawn_parts, spawn_child_reaper,
    validate_launch_command,
};
use rendering::*;
pub use session::{TerminalEventKind, TerminalFrame, TerminalNodeEvent};

const DEFAULT_COLUMNS: usize = 80;
const DEFAULT_ROWS: usize = 24;
const DEFAULT_SCROLLBACK: usize = 10_000;
const DEFAULT_FONT_SIZE: f32 = 14.0;
const DEFAULT_LINE_HEIGHT: f32 = 18.0;
const DEFAULT_CELL_WIDTH: f32 = 8.4;
const DEFAULT_SELECTION_BACKGROUND: Color = Color::from_rgba8(59, 130, 246, 105);
const SELECTION_DRAG_THRESHOLD: f64 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq)]
enum CursorVisual {
    Filled(Rect),
    Hollow(Rect),
}

type Terminal = Crosswords<TerminalListener>;
type PtySend = Arc<dyn Fn(Msg) + Send + Sync>;
type ClipboardFormatter = Arc<dyn Fn(&str) -> String + Send + Sync + 'static>;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct TerminalSelectionSnapshot {
    text: Option<String>,
    kind: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq)]
struct PendingHyperlink {
    url: String,
    origin: (f64, f64),
    cancelled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WheelContext {
    Scrollback,
    Selection,
    MouseReport,
    AlternateScroll,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct WheelLineAccumulator {
    remainder: f64,
    context: Option<WheelContext>,
}

impl WheelLineAccumulator {
    fn push(&mut self, context: WheelContext, delta: f64) -> i32 {
        if delta == 0.0 {
            return 0;
        }
        if self.context != Some(context) {
            self.context = Some(context);
            self.remainder = 0.0;
        }
        // Do not make a direction reversal pay off an unrelated gesture's
        // residual delta. This keeps a small counter-scroll responsive.
        if self.remainder != 0.0 && self.remainder.signum() != delta.signum() {
            self.remainder = 0.0;
        }
        self.remainder += delta;
        let lines = (self.remainder / WHEEL_LINE_DELTA).trunc() as i32;
        self.remainder -= f64::from(lines) * WHEEL_LINE_DELTA;
        lines
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TerminalSize {
    columns: usize,
    rows: usize,
    pixel_width: u32,
    pixel_height: u32,
    cell_pixel_width: u32,
    cell_pixel_height: u32,
}

impl TerminalSize {
    fn from_viewport(
        width: f32,
        height: f32,
        cell_width: f32,
        line_height: f32,
        device_scale: f64,
    ) -> Self {
        let columns = (width / cell_width).floor().max(1.0) as usize;
        let rows = (height / line_height).floor().max(1.0) as usize;
        let scale = device_scale.max(f64::EPSILON);
        Self {
            columns,
            rows,
            pixel_width: scaled_pixels(width, scale),
            pixel_height: scaled_pixels(height, scale),
            cell_pixel_width: scaled_pixels(cell_width, scale),
            cell_pixel_height: scaled_pixels(line_height, scale),
        }
    }

    fn for_grid(columns: usize, rows: usize, cell_width: f32, line_height: f32) -> Self {
        Self::from_viewport(
            columns as f32 * cell_width,
            rows as f32 * line_height,
            cell_width,
            line_height,
            1.0,
        )
    }

    fn crosswords(self) -> CrosswordsSize {
        CrosswordsSize::new_with_dimensions(
            self.columns,
            self.rows,
            self.pixel_width,
            self.pixel_height,
            self.cell_pixel_width,
            self.cell_pixel_height,
        )
    }

    fn winsize(self) -> WinsizeBuilder {
        WinsizeBuilder {
            rows: saturating_u16(self.rows),
            cols: saturating_u16(self.columns),
            width: saturating_u16(self.pixel_width as usize),
            height: saturating_u16(self.pixel_height as usize),
        }
    }
}

fn scaled_pixels(value: f32, scale: f64) -> u32 {
    (f64::from(value.max(0.0)) * scale)
        .round()
        .clamp(0.0, u32::MAX as f64) as u32
}

fn saturating_u16(value: usize) -> u16 {
    value.min(u16::MAX as usize) as u16
}

fn terminal_primary_shortcut(modifiers: Modifiers) -> bool {
    modifiers.primary_shortcut()
}

fn terminal_clipboard_shortcut(modifiers: Modifiers) -> bool {
    terminal_primary_shortcut(modifiers) && (cfg!(target_os = "macos") || modifiers.shift())
}

#[derive(Clone, Default)]
struct TerminalListener {
    shared: Arc<ListenerShared>,
}

#[derive(Default)]
struct ListenerShared {
    dirty: AtomicBool,
    wake: Mutex<Option<WakeCallback>>,
    events: Mutex<Vec<RioEvent>>,
}

impl TerminalListener {
    fn set_wake(&self, wake: WakeCallback) {
        *self.shared.wake.lock().unwrap() = Some(wake);
    }

    fn wake(&self) {
        self.shared.dirty.store(true, Ordering::Release);
        if let Some(wake) = self.shared.wake.lock().unwrap().as_ref() {
            wake();
        }
    }

    fn take_dirty(&self) -> bool {
        self.shared.dirty.swap(false, Ordering::AcqRel)
    }

    fn drain_events(&self) -> Vec<RioEvent> {
        std::mem::take(&mut *self.shared.events.lock().unwrap())
    }
}

impl EventListener for TerminalListener {
    fn event(&self) -> (Option<RioEvent>, bool) {
        (None, false)
    }

    fn send_event(&self, event: RioEvent, _window: WindowId) {
        self.shared.events.lock().unwrap().push(event);
        self.wake();
    }

    fn send_event_with_high_priority(&self, event: RioEvent, window: WindowId) {
        self.send_event(event, window);
    }

    fn send_redraw(&self, _window: WindowId) {
        self.wake();
    }
}

/// Interactive terminal surface. Use [`TerminalWidget::spawn_default_shell`]
/// for a real PTY, or [`TerminalWidget::headless`] to feed deterministic VT
/// bytes in tests and previews.
pub struct TerminalWidget {
    terminal: Arc<FairMutex<Terminal>>,
    visible_rows: Vec<Row<Square>>,
    visible_styles: Vec<Style>,
    visible_extras: FxHashMap<u16, Extras>,
    listener: TerminalListener,
    parser: Processor,
    pty_send: Option<PtySend>,
    #[cfg(unix)]
    child_pid: Option<libc::pid_t>,
    launch: Option<LaunchConfig>,
    launch_started: bool,
    exit_reported: bool,
    launch_error: Option<String>,
    pending_input: Vec<u8>,
    pending_host_actions: VecDeque<HostAction>,
    pending_node_events: VecDeque<TerminalNodeEvent>,
    last_reported_selection: TerminalSelectionSnapshot,
    last_reported_directory: Option<PathBuf>,
    pending_clipboard_loads: HashMap<u64, ClipboardFormatter>,
    next_clipboard_request_id: u64,
    allow_clipboard_read: bool,
    sync_window_title: bool,
    size: TerminalSize,
    font_size: f32,
    line_height: f32,
    cell_width: f32,
    font_family: Arc<str>,
    metrics_dirty: bool,
    explicit_line_height: Option<f32>,
    focused: bool,
    cursor_on: bool,
    cursor_blink: Option<bool>,
    next_cursor_blink: Option<Instant>,
    spawn_error: Option<String>,
    selecting: bool,
    selection_pointer_origin: Option<(f64, f64)>,
    selection_dragged: bool,
    selection_background: Color,
    selection_foreground: Option<Color>,
    theme_foreground: Color,
    theme_background: Color,
    inherit_theme: bool,
    selection_drag_point: Option<(f64, f64)>,
    next_selection_scroll: Option<Instant>,
    pending_hyperlink: Option<PendingHyperlink>,
    remote_mouse_button: Option<PointerButton>,
    wheel_lines: WheelLineAccumulator,
    last_click: Option<(Instant, f32, f32, u8)>,
    graphics: TerminalGraphics,
}

impl Drop for TerminalWidget {
    fn drop(&mut self) {
        self.shutdown_pty();
    }
}

impl Default for TerminalWidget {
    fn default() -> Self {
        Self::headless(DEFAULT_COLUMNS, DEFAULT_ROWS)
    }
}

impl TerminalWidget {
    /// Construct a parser/grid without spawning a process.
    pub fn headless(columns: usize, rows: usize) -> Self {
        let columns = columns.max(1);
        let rows = rows.max(1);
        let size = TerminalSize::for_grid(columns, rows, DEFAULT_CELL_WIDTH, DEFAULT_LINE_HEIGHT);
        let listener = TerminalListener::default();
        let terminal = Crosswords::new(
            size.crosswords(),
            CursorShape::Block,
            listener.clone(),
            WindowId::from(0),
            0,
            DEFAULT_SCROLLBACK,
        );
        Self {
            terminal: Arc::new(FairMutex::new(terminal)),
            visible_rows: Vec::new(),
            visible_styles: Vec::new(),
            visible_extras: FxHashMap::default(),
            listener,
            parser: Processor::default(),
            pty_send: None,
            #[cfg(unix)]
            child_pid: None,
            launch: None,
            launch_started: false,
            exit_reported: false,
            launch_error: None,
            pending_input: Vec::new(),
            pending_host_actions: VecDeque::new(),
            pending_node_events: VecDeque::new(),
            last_reported_selection: TerminalSelectionSnapshot::default(),
            last_reported_directory: None,
            pending_clipboard_loads: HashMap::new(),
            next_clipboard_request_id: 1,
            allow_clipboard_read: false,
            sync_window_title: false,
            size,
            font_size: DEFAULT_FONT_SIZE,
            line_height: DEFAULT_LINE_HEIGHT,
            cell_width: DEFAULT_CELL_WIDTH,
            font_family: Arc::from("monospace"),
            metrics_dirty: true,
            explicit_line_height: None,
            focused: false,
            cursor_on: true,
            cursor_blink: None,
            next_cursor_blink: None,
            spawn_error: None,
            selecting: false,
            selection_pointer_origin: None,
            selection_dragged: false,
            selection_background: DEFAULT_SELECTION_BACKGROUND,
            selection_foreground: None,
            theme_foreground: named_color(NamedColor::Foreground, true),
            theme_background: named_color(NamedColor::Background, false),
            inherit_theme: false,
            selection_drag_point: None,
            next_selection_scroll: None,
            pending_hyperlink: None,
            remote_mouse_button: None,
            wheel_lines: WheelLineAccumulator::default(),
            last_click: None,
            graphics: TerminalGraphics::default(),
        }
    }

    /// Spawn the user's default shell in a Rio PTY reader thread.
    pub fn spawn_default_shell() -> Self {
        let mut widget = Self::default();
        widget.launch = Some(LaunchConfig::default_shell());
        widget.ensure_launched();
        widget
    }

    /// Spawn a command attached to the terminal's PTY.
    pub fn spawn(command: &str, args: Vec<String>, cwd: Option<String>) -> Self {
        let mut widget = Self::default();
        widget.launch = Some(LaunchConfig {
            command: command.to_owned(),
            args,
            cwd,
            login_shell: false,
        });
        widget.ensure_launched();
        widget
    }

    fn lazy_default_shell() -> Self {
        let mut widget = Self::default();
        widget.launch = Some(LaunchConfig::default_shell());
        widget
    }

    fn ensure_launched(&mut self) {
        if self.launch_started || self.launch_error.is_some() {
            return;
        }
        let Some(launch) = self.launch.clone() else {
            return;
        };
        self.launch_started = true;
        if let Err(error) = validate_launch_command(&launch.command)
            .and_then(|()| process::validate_working_directory(launch.cwd.as_deref()))
        {
            self.spawn_error = Some(error.to_string());
            return;
        }
        let winsize = self.size.winsize();
        let (command, args) = pty_spawn_parts(&launch);
        match create_pty_with_spawn(
            &command,
            args,
            &launch.cwd,
            winsize.cols,
            winsize.rows,
            winsize.width,
            winsize.height,
        )
        .and_then(|pty| {
            #[cfg(unix)]
            {
                self.child_pid = Some(*pty.child.pid);
            }
            Machine::new(
                self.terminal.clone(),
                pty,
                self.listener.clone(),
                WindowId::from(0),
                0,
            )
            .map_err(|error| std::io::Error::other(error.to_string()))
        }) {
            Ok(machine) => {
                let channel = machine.channel();
                self.pty_send = Some(Arc::new(move |message| {
                    let _ = channel.send(message);
                }));
                machine.spawn();
            }
            Err(error) => self.spawn_error = Some(error.to_string()),
        }
    }

    fn shutdown_pty(&mut self) {
        if let Some(send) = self.pty_send.take() {
            send(Msg::Shutdown);
        }
        #[cfg(unix)]
        if let Some(pid) = self.child_pid.take() {
            // The task starts waiting immediately. Rio owns the PTY until it
            // handles Shutdown; dropping the PTY sends SIGHUP, after which
            // this wait reaps the child instead of leaving a zombie behind.
            if let Err(error) = spawn_child_reaper(pid) {
                tracing::warn!(%error, pid, "failed to start PTY child reaper");
            }
        }
    }

    fn report_exit_once(&mut self) {
        if self.exit_reported {
            return;
        }
        self.exit_reported = true;
        self.pty_send = None;
        self.next_cursor_blink = None;
        self.pending_clipboard_loads.clear();
        self.pending_node_events.push_back(TerminalNodeEvent::json(
            TerminalEventKind::Exit,
            r#"{"reason":"exit"}"#,
        ));
    }

    /// Feed raw PTY output into the VT parser (useful for tests/replay).
    pub fn feed(&mut self, bytes: &[u8]) {
        self.parser.advance(&mut *self.terminal.lock(), bytes);
        self.listener.wake();
    }

    /// Input bytes captured by a headless terminal.
    pub fn take_input(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.pending_input)
    }

    /// Plain text for a visible row, primarily for deterministic tests.
    pub fn visible_line(&self, row: usize) -> String {
        let terminal = self.terminal.lock();
        let rows = terminal.visible_rows();
        rows.get(row).map_or_else(String::new, |line| {
            (0..terminal.columns())
                .map(|column| line[Column(column)].c())
                .collect()
        })
    }

    /// Current grid selection as plain text.
    pub fn selected_text(&self) -> Option<String> {
        self.terminal.lock().selection_to_string()
    }

    fn report_pointer(&mut self, pointer: &wabou_shell_api::PointerEvent) -> bool {
        if self.exit_reported || (pointer.modifiers.shift() && self.remote_mouse_button.is_none()) {
            return false;
        }
        let mode = self.terminal.lock().mode();
        if !mode.intersects(Mode::MOUSE_MODE) {
            if matches!(pointer.phase, PointerPhase::Up | PointerPhase::Cancel) {
                self.remote_mouse_button = None;
            }
            return false;
        }
        if pointer.phase == PointerPhase::Down {
            self.last_click = None;
        }
        let motion = pointer.phase == PointerPhase::Move;
        if motion
            && !mode.contains(Mode::MOUSE_MOTION)
            && !(mode.contains(Mode::MOUSE_DRAG) && pointer.buttons != 0)
        {
            return false;
        }
        let button = pointer.button.or(self.remote_mouse_button);
        let base = if motion {
            if pointer.buttons & 1 != 0 {
                0
            } else if pointer.buttons & 2 != 0 {
                1
            } else if pointer.buttons & 4 != 0 {
                2
            } else {
                3
            }
        } else {
            match button.unwrap_or(PointerButton::Primary) {
                PointerButton::Primary => 0,
                PointerButton::Auxiliary => 1,
                PointerButton::Secondary => 2,
                PointerButton::Other(_) => return false,
            }
        };
        let modifiers = u8::from(pointer.modifiers.shift()) * 4
            + u8::from(pointer.modifiers.alt()) * 8
            + u8::from(pointer.modifiers.control()) * 16;
        let code = base + modifiers + if motion { 32 } else { 0 };
        let release = matches!(pointer.phase, PointerPhase::Up | PointerPhase::Cancel);
        let (column, row) = self.mouse_grid_position(pointer.position.x, pointer.position.y);
        let bytes = if mode.contains(Mode::SGR_MOUSE) {
            format!(
                "\x1b[<{code};{column};{row}{}",
                if release { 'm' } else { 'M' }
            )
            .into_bytes()
        } else {
            normal_mouse_sequence(
                if release { 3 + modifiers } else { code },
                column,
                row,
                mode.contains(Mode::UTF8_MOUSE),
            )
            .unwrap_or_default()
        };
        if !bytes.is_empty() {
            self.send_bytes(bytes);
        }
        match pointer.phase {
            PointerPhase::Down => self.remote_mouse_button = button,
            PointerPhase::Up | PointerPhase::Cancel => self.remote_mouse_button = None,
            PointerPhase::Enter | PointerPhase::Move | PointerPhase::Leave => {}
        }
        true
    }

    fn report_wheel(&mut self, wheel: &wabou_shell_api::WheelEvent, lines: i32) -> bool {
        if self.exit_reported || wheel.modifiers.shift() {
            return false;
        }
        let mode = self.terminal.lock().mode();
        if !mode.intersects(Mode::MOUSE_MODE) {
            return false;
        }
        let modifiers =
            u8::from(wheel.modifiers.alt()) * 8 + u8::from(wheel.modifiers.control()) * 16;
        if lines == 0 {
            return true;
        }
        let code = (if lines < 0 { 64 } else { 65 }) + modifiers;
        let (column, row) = self.mouse_grid_position(wheel.position.x, wheel.position.y);
        let bytes = if mode.contains(Mode::SGR_MOUSE) {
            format!("\x1b[<{code};{column};{row}M").into_bytes()
        } else {
            normal_mouse_sequence(code, column, row, mode.contains(Mode::UTF8_MOUSE))
                .unwrap_or_default()
        };
        if !bytes.is_empty() {
            self.send_bytes(bytes.repeat(lines.unsigned_abs() as usize));
        }
        true
    }

    fn report_alternate_scroll(&mut self, lines: i32) -> bool {
        if self.exit_reported {
            return false;
        }
        let mode = self.terminal.lock().mode();
        if !mode.contains(Mode::ALT_SCREEN) || !mode.contains(Mode::ALTERNATE_SCROLL) {
            return false;
        }
        if lines == 0 {
            return true;
        }
        let sequence: &[u8] = if lines < 0 { b"\x1b[A" } else { b"\x1b[B" };
        self.send_bytes(sequence.repeat(lines.unsigned_abs() as usize));
        true
    }

    fn wheel_context(&self, wheel: &wabou_shell_api::WheelEvent) -> WheelContext {
        if self.selecting {
            return WheelContext::Selection;
        }
        if self.exit_reported {
            return WheelContext::Scrollback;
        }
        let mode = self.terminal.lock().mode();
        if !wheel.modifiers.shift() && mode.intersects(Mode::MOUSE_MODE) {
            WheelContext::MouseReport
        } else if mode.contains(Mode::ALT_SCREEN) && mode.contains(Mode::ALTERNATE_SCROLL) {
            WheelContext::AlternateScroll
        } else {
            WheelContext::Scrollback
        }
    }

    fn send_bytes(&mut self, bytes: Vec<u8>) {
        if self.exit_reported {
            return;
        }
        if let Some(send) = &self.pty_send {
            send(Msg::Input(Cow::Owned(bytes)));
        } else {
            self.pending_input.extend(bytes);
        }
    }

    fn begin_terminal_input(&mut self) {
        self.last_click = None;
        let mut terminal = self.terminal.lock();
        terminal.selection = None;
        terminal.scroll_display(Scroll::Bottom);
        drop(terminal);
        self.sync_selection_change();
    }

    fn resize(&mut self, width: f32, height: f32, device_scale: f64) {
        let size = TerminalSize::from_viewport(
            width,
            height,
            self.cell_width,
            self.line_height,
            device_scale,
        );
        if size == self.size {
            return;
        }
        self.size = size;
        self.terminal.lock().resize(size.crosswords());
        self.sync_selection_change();
        if let Some(send) = &self.pty_send {
            send(Msg::Resize(size.winsize()));
        }
    }

    fn update_font_metrics(&mut self, tcx: &mut TextContext) {
        if !self.metrics_dirty {
            return;
        }
        let layout = layout_text_styled(
            tcx,
            Arc::from("0"),
            self.font_size,
            400.0,
            false,
            None,
            Default::default(),
            [255, 255, 255, 255],
            Arc::from([]),
            Some(&self.font_family),
            None,
        );
        let advance = layout.width();
        if advance.is_finite() && advance > 0.0 {
            self.cell_width = advance;
        }
        self.line_height = self.explicit_line_height.map_or_else(
            || (layout.height() * 1.1).max(self.font_size),
            |line_height| line_height.max(self.font_size),
        );
        self.metrics_dirty = false;
    }

    fn handle_rio_events(&mut self) -> bool {
        let mut changed = self.listener.take_dirty();
        for event in self.listener.drain_events() {
            changed |= self.handle_rio_event(event);
        }
        if changed {
            self.terminal.lock().damage_event_in_flight = false;
        }
        self.sync_current_directory();
        self.sync_selection_change();
        changed
    }

    fn handle_rio_event(&mut self, event: RioEvent) -> bool {
        match event {
            RioEvent::PtyWrite(_, text) => self.send_bytes(text.into_bytes()),
            RioEvent::TextAreaSizeRequest(_, formatter) => {
                let reply = formatter(self.size.winsize());
                self.send_bytes(reply.into_bytes());
            }
            RioEvent::TerminalDamaged(_) | RioEvent::Render | RioEvent::RenderRoute(_) => {
                return true;
            }
            RioEvent::UpdateGraphics { queues, .. } => {
                self.graphics.apply_updates(queues);
                return true;
            }
            RioEvent::Title(title) => {
                self.report_terminal_title(Some(title), None);
            }
            RioEvent::TitleWithSubtitle(title, subtitle) => {
                self.report_terminal_title(Some(title), Some(subtitle));
            }
            RioEvent::ResetTitle => {
                self.report_terminal_title(None, None);
            }
            RioEvent::ClipboardStore(_, text) => self
                .pending_host_actions
                .push_back(HostAction::SetClipboard(text)),
            RioEvent::ClipboardLoad(_, _, formatter) => {
                if self.allow_clipboard_read {
                    let request_id = self.next_clipboard_request_id;
                    self.next_clipboard_request_id =
                        self.next_clipboard_request_id.wrapping_add(1).max(1);
                    self.pending_clipboard_loads.insert(request_id, formatter);
                    self.pending_host_actions
                        .push_back(HostAction::ReadClipboard { request_id });
                } else {
                    self.send_bytes(formatter("").into_bytes());
                }
            }
            RioEvent::ColorRequest(_, index, formatter) => {
                let colors = self.terminal.lock().colors;
                self.send_bytes(
                    formatter(terminal_color(
                        index,
                        &colors,
                        self.theme_foreground,
                        self.theme_background,
                    ))
                    .into_bytes(),
                );
            }
            RioEvent::CursorBlinkingChange | RioEvent::CursorBlinkingChangeOnRoute(_) => {
                let terminal_blinking = self.terminal.lock().blinking_cursor;
                self.schedule_cursor_blink(terminal_blinking);
                return true;
            }
            RioEvent::Bell => {
                self.pending_host_actions
                    .push_back(HostAction::RequestAttention);
                self.pending_node_events
                    .push_back(TerminalNodeEvent::json(TerminalEventKind::Bell, "{}"));
            }
            RioEvent::CloseTerminal(_) | RioEvent::Exit | RioEvent::Quit => {
                self.report_exit_once();
            }
            RioEvent::ProgressReport(report) => {
                let state = match report.state {
                    ProgressState::Remove => "remove",
                    ProgressState::Set => "set",
                    ProgressState::Error => "error",
                    ProgressState::Indeterminate => "indeterminate",
                    ProgressState::Pause => "pause",
                };
                self.pending_node_events.push_back(TerminalNodeEvent::json(
                    TerminalEventKind::Progress,
                    serde_json::json!({ "state": state, "progress": report.progress }).to_string(),
                ));
            }
            RioEvent::DesktopNotification { title, body } => {
                self.pending_node_events.push_back(TerminalNodeEvent::json(
                    TerminalEventKind::Notification,
                    serde_json::json!({ "title": title, "body": body }).to_string(),
                ))
            }
            _ => {}
        }
        false
    }

    fn report_terminal_title(&mut self, title: Option<String>, subtitle: Option<String>) {
        if self.sync_window_title {
            self.pending_host_actions
                .push_back(HostAction::SetWindowTitle(title.clone()));
        }
        self.pending_node_events.push_back(TerminalNodeEvent::json(
            TerminalEventKind::TitleChange,
            serde_json::json!({ "title": title, "subtitle": subtitle }).to_string(),
        ));
    }

    fn sync_current_directory(&mut self) {
        let directory = self.terminal.lock().current_directory.clone();
        if directory.is_none() || directory == self.last_reported_directory {
            return;
        }
        self.last_reported_directory = directory.clone();
        let encoded_path = directory
            .expect("checked as some")
            .to_string_lossy()
            .into_owned();
        let path = percent_encoding::percent_decode_str(&encoded_path)
            .decode_utf8_lossy()
            .into_owned();
        self.pending_node_events.push_back(TerminalNodeEvent::json(
            TerminalEventKind::CurrentDirectoryChange,
            serde_json::json!({ "path": path }).to_string(),
        ));
    }
}

#[cfg(test)]
mod tests;
