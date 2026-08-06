//! A Wabou terminal widget backed by `rio-vt`.
//!
//! `rio-vt` owns terminal semantics (VT parsing, grid, cursor, scrollback and
//! PTY events). This crate is the frontend adapter: it translates Wabou input
//! to PTY bytes and pulls Rio's visible grid into a retained Vello scene.

use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rio_vt::ansi::CursorShape;
use rio_vt::config::colors::term::{COUNT as TERMINAL_COLOR_COUNT, TermColors};
use rio_vt::config::colors::{AnsiColor, ColorRgb, NamedColor};
use rio_vt::crosswords::grid::Scroll;
use rio_vt::crosswords::grid::row::Row;
use rio_vt::crosswords::pos::{Column, Line, Pos, Side};
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
use vello::Scene;
use vello::kurbo::{Affine, Rect, Stroke};
use vello::peniko::{Color, Fill};
use wabou_quick::protocol::event;
use wabou_quick::widget::WidgetEventResult;
use wabou_quick::{Widget, WidgetNodeEvent, WidgetStyle};
#[cfg(test)]
use wabou_shell::style::Paint;
use wabou_shell::text::{TextContext, layout_text_styled};
use wabou_shell::{
    HostAction, HostActionResult, KeyPhase, Modifiers, PointerButton, PointerPhase, UiEvent,
    WHEEL_LINE_DELTA, WakeCallback,
};

mod graphics;
mod kitty_keyboard;

use graphics::{KittyLayer, TerminalGraphics};

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct LaunchConfig {
    command: String,
    args: Vec<String>,
    cwd: Option<String>,
    login_shell: bool,
}

impl LaunchConfig {
    fn default_shell() -> Self {
        Self {
            command: default_shell_command(),
            args: Vec::new(),
            cwd: None,
            login_shell: true,
        }
    }
}

fn pty_spawn_parts(launch: &LaunchConfig) -> (String, Vec<String>) {
    #[cfg(windows)]
    {
        (
            quote_windows_command_arg(&launch.command),
            launch
                .args
                .iter()
                .map(|arg| quote_windows_command_arg(arg))
                .collect(),
        )
    }
    #[cfg(not(windows))]
    {
        let mut args = vec![
            "TERM=xterm-256color".into(),
            "COLORTERM=truecolor".into(),
            "TERM_PROGRAM=wabou".into(),
            format!("TERM_PROGRAM_VERSION={}", env!("CARGO_PKG_VERSION")),
            launch.command.clone(),
        ];
        #[cfg(target_os = "macos")]
        if launch.login_shell {
            args.push("-l".into());
        }
        args.extend(launch.args.iter().cloned());
        ("/usr/bin/env".into(), args)
    }
}

/// Quote one argv item for the Windows `CommandLineToArgvW` rules.
///
/// teletypewriter's ConPTY backend accepts a command-line string and joins its
/// argument vector with spaces, so its inputs must already preserve argument
/// boundaries. Backslashes are only special immediately before a quote or the
/// closing quote.
#[cfg(any(windows, test))]
fn quote_windows_command_arg(value: &str) -> String {
    if !value.is_empty()
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte == b'"')
    {
        return value.to_owned();
    }

    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    let mut backslashes = 0;
    for character in value.chars() {
        if character == '\\' {
            backslashes += 1;
            continue;
        }
        if character == '"' {
            quoted.extend(std::iter::repeat_n('\\', backslashes * 2 + 1));
        } else {
            quoted.extend(std::iter::repeat_n('\\', backslashes));
        }
        backslashes = 0;
        quoted.push(character);
    }
    quoted.extend(std::iter::repeat_n('\\', backslashes * 2));
    quoted.push('"');
    quoted
}

#[cfg(unix)]
fn validate_launch_command(command: &str) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let path = PathBuf::from(command);
    let candidates: Vec<PathBuf> = if path.components().count() > 1 {
        vec![path]
    } else {
        std::env::var_os("PATH")
            .map(|path| {
                std::env::split_paths(&path)
                    .map(|directory| directory.join(command))
                    .collect()
            })
            .unwrap_or_default()
    };
    let mut found_non_executable = false;
    for candidate in candidates {
        let Ok(metadata) = candidate.metadata() else {
            continue;
        };
        if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
            return Ok(());
        }
        found_non_executable = true;
    }
    let kind = if found_non_executable {
        std::io::ErrorKind::PermissionDenied
    } else {
        std::io::ErrorKind::NotFound
    };
    Err(std::io::Error::new(
        kind,
        format!("terminal command is not executable: {command}"),
    ))
}

#[cfg(not(unix))]
fn validate_launch_command(_command: &str) -> std::io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn spawn_child_reaper(
    pid: libc::pid_t,
) -> std::io::Result<std::thread::JoinHandle<std::io::Result<()>>> {
    const SHUTDOWN_GRACE: Duration = Duration::from_millis(500);
    std::thread::Builder::new()
        .name("PTY child reaper".into())
        .spawn(move || {
            let deadline = Instant::now() + SHUTDOWN_GRACE;
            let mut sent_hangup = false;
            loop {
                let mut status = 0;
                let result = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
                if result == pid {
                    return Ok(());
                }
                if result == 0 {
                    if !sent_hangup {
                        // create_pty_with_spawn calls setsid in the child, so
                        // its PID is also the process-group id. Signal the
                        // whole terminal job, including descendants.
                        unsafe { libc::kill(-pid, libc::SIGHUP) };
                        sent_hangup = true;
                    }
                    if Instant::now() >= deadline {
                        unsafe {
                            libc::kill(-pid, libc::SIGKILL);
                            // Fall back to the direct child for unusual PTY
                            // implementations without a separate group.
                            libc::kill(pid, libc::SIGKILL);
                        }
                        loop {
                            let result = unsafe { libc::waitpid(pid, &mut status, 0) };
                            if result == pid {
                                return Ok(());
                            }
                            let error = std::io::Error::last_os_error();
                            if error.kind() == std::io::ErrorKind::Interrupted {
                                continue;
                            }
                            if error.raw_os_error() == Some(libc::ECHILD) {
                                return Ok(());
                            }
                            return Err(error);
                        }
                    }
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                // Rio might have observed SIGCHLD and reaped a naturally
                // exiting process before widget teardown reached this task.
                if error.raw_os_error() == Some(libc::ECHILD) {
                    return Ok(());
                }
                return Err(error);
            }
        })
}

fn default_shell_command() -> String {
    #[cfg(windows)]
    {
        std::env::var("COMSPEC")
            .ok()
            .filter(|command| !command.is_empty())
            .unwrap_or_else(|| "cmd.exe".into())
    }
    #[cfg(not(windows))]
    {
        std::env::var("SHELL")
            .ok()
            .filter(|command| !command.is_empty())
            .unwrap_or_else(|| "/bin/sh".into())
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
    pending_node_events: VecDeque<WidgetNodeEvent>,
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
    next_cursor_blink: Option<Instant>,
    spawn_error: Option<String>,
    window_to_local: [f64; 6],
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
    device_scale: f64,
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
            next_cursor_blink: None,
            spawn_error: None,
            window_to_local: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
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
            device_scale: 1.0,
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
        if let Err(error) = validate_launch_command(&launch.command) {
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
        self.pending_node_events.push_back(WidgetNodeEvent::json(
            event::TERMINALEXIT,
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

    fn sync_selection_change(&mut self) {
        // Selection changes stay renderer-local while a pointer gesture is in
        // progress. PTY output, resize, and rio-vt side events can all call
        // this method mid-drag; only the gesture terminator publishes the
        // committed snapshot to Solid.
        if self.selecting {
            return;
        }
        let terminal = self.terminal.lock();
        let text = terminal.selection_to_string();
        let kind = text.as_ref().and_then(|_| {
            terminal
                .selection
                .as_ref()
                .map(|selection| match selection.ty {
                    SelectionType::Simple => "simple",
                    SelectionType::Block => "block",
                    SelectionType::Semantic => "word",
                    SelectionType::Lines => "line",
                })
        });
        drop(terminal);
        let selection = TerminalSelectionSnapshot { text, kind };
        if selection == self.last_reported_selection {
            return;
        }
        self.last_reported_selection = selection.clone();
        self.pending_node_events.push_back(WidgetNodeEvent::json(
            event::TERMINALSELECTIONCHANGE,
            serde_json::json!({ "text": selection.text, "kind": selection.kind }).to_string(),
        ));
    }

    fn local_point(&self, x: f64, y: f64) -> (f32, f32) {
        let [a, b, c, d, e, f] = self.window_to_local;
        ((a * x + c * y + e) as f32, (b * x + d * y + f) as f32)
    }

    fn pointer_cell(&self, x: f64, y: f64, display_offset: usize) -> (Pos, Side) {
        let (local_x, local_y) = self.local_point(x, y);
        let local_y = local_y.max(0.0);
        let raw_column = local_x.max(0.0) / self.cell_width;
        let column = (raw_column.floor() as usize).min(self.size.columns.saturating_sub(1));
        let viewport_row = (local_y / self.line_height).floor() as usize;
        let viewport_row = viewport_row.min(self.size.rows.saturating_sub(1));
        let line = Line(viewport_row as i32 - display_offset as i32);
        let viewport_width = self.size.columns as f32 * self.cell_width;
        let side = if local_x <= 0.0 {
            Side::Left
        } else if local_x >= viewport_width {
            Side::Right
        } else if raw_column.fract() < 0.5 {
            Side::Left
        } else {
            Side::Right
        };
        (Pos::new(line, Column(column)), side)
    }

    fn begin_selection(&mut self, x: f64, y: f64, block: bool) {
        let (local_x, local_y) = self.local_point(x, y);
        let now = Instant::now();
        let clicks = self.last_click.map_or(1, |(time, last_x, last_y, count)| {
            if now.duration_since(time) <= Duration::from_millis(400)
                && (local_x - last_x).abs() <= 4.0
                && (local_y - last_y).abs() <= 4.0
            {
                count % 3 + 1
            } else {
                1
            }
        });
        self.last_click = Some((now, local_x, local_y, clicks));
        let mut terminal = self.terminal.lock();
        let (point, side) = self.pointer_cell(x, y, terminal.display_offset());
        terminal.selection = Some(Selection::new(
            if block {
                SelectionType::Block
            } else {
                match clicks {
                    2 => SelectionType::Semantic,
                    3 => SelectionType::Lines,
                    _ => SelectionType::Simple,
                }
            },
            point,
            side,
        ));
        drop(terminal);
        self.selecting = true;
        self.selection_pointer_origin = Some((x, y));
        self.selection_dragged = false;
    }

    fn begin_or_extend_selection(&mut self, x: f64, y: f64, modifiers: Modifiers) {
        if modifiers.shift() && self.terminal.lock().selection.is_some() {
            self.selecting = true;
            self.selection_pointer_origin = Some((x, y));
            self.selection_dragged = false;
            self.last_click = None;
            self.update_selection(x, y);
        } else {
            self.begin_selection(x, y, modifiers.alt());
        }
    }

    fn update_selection(&mut self, x: f64, y: f64) {
        if let Some((origin_x, origin_y)) = self.selection_pointer_origin {
            self.selection_dragged |= (x - origin_x).hypot(y - origin_y) > SELECTION_DRAG_THRESHOLD;
        }
        let mut terminal = self.terminal.lock();
        let (_, local_y) = self.local_point(x, y);
        let viewport_height = self.size.rows as f32 * self.line_height;
        let outside = if local_y < 0.0 {
            -local_y
        } else if local_y >= viewport_height {
            local_y - viewport_height
        } else {
            0.0
        };
        if outside > 0.0 {
            let lines = (outside / self.line_height).ceil().clamp(1.0, 8.0) as i32;
            terminal.scroll_display(Scroll::Delta(if local_y < 0.0 { lines } else { -lines }));
        }
        let (point, side) = self.pointer_cell(x, y, terminal.display_offset());
        if let Some(selection) = terminal.selection.as_mut() {
            selection.update(point, side);
        }
        let above = local_y < 0.0;
        let below = local_y >= viewport_height;
        let can_continue_scrolling = if above {
            terminal.display_offset() < terminal.history_size()
        } else if below {
            terminal.display_offset() > 0
        } else {
            false
        };
        drop(terminal);
        self.selection_drag_point = Some((x, y));
        self.next_selection_scroll =
            can_continue_scrolling.then(|| Instant::now() + Duration::from_millis(50));
    }

    fn finish_selection_gesture(&mut self) {
        if self.selection_dragged {
            self.last_click = None;
        }
        self.selecting = false;
        self.selection_pointer_origin = None;
        self.selection_dragged = false;
        self.selection_drag_point = None;
        self.next_selection_scroll = None;
        self.sync_selection_change();
    }

    fn select_all(&mut self) {
        let mut terminal = self.terminal.lock();
        let start = Pos::new(Line(-(terminal.history_size() as i32)), Column(0));
        let end = Pos::new(
            terminal.bottommost_line(),
            Column(terminal.columns().saturating_sub(1)),
        );
        let mut selection = Selection::new(SelectionType::Lines, start, Side::Left);
        selection.update(end, Side::Right);
        terminal.selection = Some(selection);
        drop(terminal);

        self.selecting = false;
        self.selection_pointer_origin = None;
        self.selection_dragged = false;
        self.selection_drag_point = None;
        self.next_selection_scroll = None;
        self.last_click = None;
        self.sync_selection_change();
    }

    fn tick_selection_autoscroll(&mut self) {
        if !self.selecting
            || !self
                .next_selection_scroll
                .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return;
        }
        if let Some((x, y)) = self.selection_drag_point {
            self.update_selection(x, y);
        }
    }

    fn scroll_active_selection(&mut self, wheel: &wabou_shell::WheelEvent, lines: i32) {
        if lines == 0 {
            return;
        }
        let mut terminal = self.terminal.lock();
        terminal.scroll_display(Scroll::Delta(lines));
        let (point, side) = self.pointer_cell(
            wheel.position.x,
            wheel.position.y,
            terminal.display_offset(),
        );
        if let Some(selection) = terminal.selection.as_mut() {
            selection.update(point, side);
        }
        drop(terminal);
        self.selection_drag_point = Some((wheel.position.x, wheel.position.y));
    }

    fn mouse_grid_position(&self, x: f64, y: f64) -> (usize, usize) {
        let (local_x, local_y) = self.local_point(x, y);
        let column = ((local_x.max(0.0) / self.cell_width).floor() as usize)
            .min(self.size.columns.saturating_sub(1));
        let row = ((local_y.max(0.0) / self.line_height).floor() as usize)
            .min(self.size.rows.saturating_sub(1));
        (column + 1, row + 1)
    }

    fn hyperlink_at(&self, x: f64, y: f64) -> Option<String> {
        let terminal = self.terminal.lock();
        let (point, _) = self.pointer_cell(x, y, terminal.display_offset());
        let hyperlink = terminal.cell_hyperlink(point.row, point.col).or_else(|| {
            let square = terminal.grid[point];
            (square.wide() == Wide::Spacer && point.col.0 > 0)
                .then(|| terminal.cell_hyperlink(point.row, Column(point.col.0 - 1)))
                .flatten()
        });
        hyperlink.map(|link| link.uri().to_owned())
    }

    fn report_pointer(&mut self, pointer: &wabou_shell::PointerEvent) -> bool {
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
            PointerPhase::Move => {}
        }
        true
    }

    fn report_wheel(&mut self, wheel: &wabou_shell::WheelEvent, lines: i32) -> bool {
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

    fn wheel_context(&self, wheel: &wabou_shell::WheelEvent) -> WheelContext {
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

    fn resize(&mut self, width: f32, height: f32) {
        let size = TerminalSize::from_viewport(
            width,
            height,
            self.cell_width,
            self.line_height,
            self.device_scale,
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
            match event {
                RioEvent::PtyWrite(_, text) => self.send_bytes(text.into_bytes()),
                RioEvent::TextAreaSizeRequest(_, formatter) => {
                    let reply = formatter(self.size.winsize());
                    self.send_bytes(reply.into_bytes());
                }
                RioEvent::TerminalDamaged(_) | RioEvent::Render | RioEvent::RenderRoute(_) => {
                    changed = true;
                }
                RioEvent::UpdateGraphics { queues, .. } => {
                    self.graphics.apply_updates(queues);
                    changed = true;
                }
                RioEvent::Title(title) => {
                    if self.sync_window_title {
                        self.pending_host_actions
                            .push_back(HostAction::SetWindowTitle(Some(title.clone())));
                    }
                    self.pending_node_events.push_back(WidgetNodeEvent::json(
                        event::TERMINALTITLECHANGE,
                        serde_json::json!({ "title": title, "subtitle": null }).to_string(),
                    ));
                }
                RioEvent::TitleWithSubtitle(title, subtitle) => {
                    if self.sync_window_title {
                        self.pending_host_actions
                            .push_back(HostAction::SetWindowTitle(Some(title.clone())));
                    }
                    self.pending_node_events.push_back(WidgetNodeEvent::json(
                        event::TERMINALTITLECHANGE,
                        serde_json::json!({ "title": title, "subtitle": subtitle }).to_string(),
                    ));
                }
                RioEvent::ResetTitle => {
                    if self.sync_window_title {
                        self.pending_host_actions
                            .push_back(HostAction::SetWindowTitle(None));
                    }
                    self.pending_node_events.push_back(WidgetNodeEvent::json(
                        event::TERMINALTITLECHANGE,
                        serde_json::json!({ "title": null, "subtitle": null }).to_string(),
                    ));
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
                    self.cursor_on = true;
                    let blinking = self.terminal.lock().blinking_cursor;
                    self.next_cursor_blink = (self.focused && blinking)
                        .then(|| Instant::now() + Duration::from_millis(500));
                    changed = true;
                }
                RioEvent::Bell => {
                    self.pending_host_actions
                        .push_back(HostAction::RequestAttention);
                    self.pending_node_events
                        .push_back(WidgetNodeEvent::json(event::TERMINALBELL, "{}"));
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
                    self.pending_node_events.push_back(WidgetNodeEvent::json(
                        event::TERMINALPROGRESS,
                        serde_json::json!({ "state": state, "progress": report.progress })
                            .to_string(),
                    ));
                }
                RioEvent::DesktopNotification { title, body } => {
                    self.pending_node_events.push_back(WidgetNodeEvent::json(
                        event::TERMINALNOTIFICATION,
                        serde_json::json!({ "title": title, "body": body }).to_string(),
                    ))
                }
                _ => {}
            }
        }
        if changed {
            self.terminal.lock().damage_event_in_flight = false;
        }
        self.sync_current_directory();
        self.sync_selection_change();
        changed
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
        self.pending_node_events.push_back(WidgetNodeEvent::json(
            event::TERMINALCWDCHANGE,
            serde_json::json!({ "path": path }).to_string(),
        ));
    }
}

/// Factory suitable for `HostBuilder::widget("terminal", terminal_widget)`.
pub fn terminal_widget() -> Box<dyn Widget> {
    Box::new(TerminalWidget::lazy_default_shell())
}

impl Widget for TerminalWidget {
    fn measure(&mut self, tcx: &mut TextContext) -> Option<[f32; 2]> {
        self.update_font_metrics(tcx);
        self.intrinsic_size()
    }

    fn paint(&mut self, width: f32, height: f32, tcx: &mut TextContext) -> Scene {
        self.update_font_metrics(tcx);
        self.resize(width, height);
        self.ensure_launched();
        self.tick_selection_autoscroll();
        if self.focused
            && self
                .next_cursor_blink
                .is_some_and(|time| Instant::now() >= time)
        {
            self.cursor_on = !self.cursor_on;
            self.next_cursor_blink = Some(Instant::now() + Duration::from_millis(500));
        }

        let (
            cursor,
            selection,
            display_offset,
            colors,
            atlas_placements,
            kitty_placements,
            history_size,
        ) = {
            let mut terminal = self.terminal.lock();
            let damage = terminal.peek_damage_event().unwrap_or(TerminalDamage::Noop);
            terminal.snapshot_visible(
                &damage,
                self.size.columns,
                &mut self.visible_rows,
                &mut self.visible_styles,
                &mut self.visible_extras,
            );
            // `damage()` advances Rio's remembered cursor position; resetting
            // only the dirty lines would otherwise leave CursorOnly pending.
            {
                let _consumed = terminal.damage();
            }
            terminal.reset_damage();
            (
                terminal.cursor(),
                terminal
                    .selection
                    .as_ref()
                    .and_then(|selection| selection.to_range(&*terminal)),
                terminal.display_offset(),
                terminal.colors,
                terminal.graphics.atlas_placements.clone(),
                terminal
                    .graphics
                    .kitty_placements
                    .values()
                    .cloned()
                    .collect::<Vec<_>>(),
                terminal.lines_evicted() as i64 + terminal.history_size() as i64,
            )
        };
        let default_background = terminal_ansi_color(
            AnsiColor::Named(NamedColor::Background),
            false,
            &colors,
            self.theme_foreground,
            self.theme_background,
        );
        let mut scene = Scene::new();
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            default_background,
            None,
            &Rect::new(0.0, 0.0, width as f64, height as f64),
        );

        let scale = self.device_scale.max(f64::EPSILON);
        let viewport = rio_vt::ansi::graphics::OverlayViewport {
            cell_width: (f64::from(self.cell_width) * scale) as f32,
            cell_height: (f64::from(self.line_height) * scale) as f32,
            origin_x: 0.0,
            origin_y: 0.0,
            history_size,
            display_offset: display_offset as i64,
            screen_lines: self.size.rows as i64,
        };
        let clip = [
            0.0,
            0.0,
            self.size.columns as f32 * viewport.cell_width,
            self.size.rows as f32 * viewport.cell_height,
        ];
        self.graphics
            .draw_atlas(&mut scene, &atlas_placements, &viewport, clip, scale);
        self.graphics.draw_kitty(
            &mut scene,
            &kitty_placements,
            KittyLayer::BehindText,
            &viewport,
            clip,
            scale,
        );

        for (row_index, row) in self.visible_rows.iter().enumerate() {
            let y = row_index as f32 * self.line_height;
            for column in 0..self.size.columns.min(row.inner.len()) {
                let square = row[Column(column)];
                let point = Pos::new(
                    Line(row_index as i32 - display_offset as i32),
                    Column(column),
                );
                let selected = selection.is_some_and(|selection: SelectionRange| {
                    selection_contains_square(selection, point, square)
                });
                let (character, style) = match square.content_tag() {
                    ContentTag::Codepoint => (
                        square.c(),
                        self.visible_styles
                            .get(square.style_id() as usize)
                            .copied()
                            .unwrap_or_default(),
                    ),
                    ContentTag::BgPalette => {
                        let bg = terminal_indexed_color(square.bg_palette_index(), &colors);
                        fill_cell(
                            &mut scene,
                            column,
                            row_index,
                            self.cell_width,
                            self.line_height,
                            scale,
                            bg,
                        );
                        if selected {
                            fill_cell(
                                &mut scene,
                                column,
                                row_index,
                                self.cell_width,
                                self.line_height,
                                scale,
                                self.selection_background,
                            );
                        }
                        continue;
                    }
                    ContentTag::BgRgb => {
                        let (r, g, b) = square.bg_rgb();
                        fill_cell(
                            &mut scene,
                            column,
                            row_index,
                            self.cell_width,
                            self.line_height,
                            scale,
                            Color::from_rgb8(r, g, b),
                        );
                        if selected {
                            fill_cell(
                                &mut scene,
                                column,
                                row_index,
                                self.cell_width,
                                self.line_height,
                                scale,
                                self.selection_background,
                            );
                        }
                        continue;
                    }
                };
                let mut fg = terminal_ansi_color(
                    style.fg,
                    true,
                    &colors,
                    self.theme_foreground,
                    self.theme_background,
                );
                let mut bg = terminal_ansi_color(
                    style.bg,
                    false,
                    &colors,
                    self.theme_foreground,
                    self.theme_background,
                );
                if style.flags.contains(StyleFlags::INVERSE) {
                    std::mem::swap(&mut fg, &mut bg);
                }
                if bg != default_background {
                    fill_cell(
                        &mut scene,
                        column,
                        row_index,
                        self.cell_width,
                        self.line_height,
                        scale,
                        bg,
                    );
                }
                if selected {
                    fill_cell(
                        &mut scene,
                        column,
                        row_index,
                        self.cell_width,
                        self.line_height,
                        scale,
                        self.selection_background,
                    );
                }
                if style.flags.contains(StyleFlags::HIDDEN) {
                    continue;
                }
                if style.flags.contains(StyleFlags::DIM) {
                    fg = dim(fg);
                }
                if selected && let Some(selection_foreground) = self.selection_foreground {
                    fg = selection_foreground;
                }
                draw_cell_decorations(
                    &mut scene,
                    column,
                    y,
                    self.cell_width,
                    self.line_height,
                    style,
                    fg,
                    &colors,
                    self.theme_foreground,
                    self.theme_background,
                    selected.then_some(self.selection_foreground).flatten(),
                );
                if matches!(square.wide(), Wide::Spacer | Wide::LeadingSpacer)
                    || character == '\0'
                    || (character == ' ' && !square.has_extras())
                    || character.is_control()
                {
                    continue;
                }
                let cell_text = cell_text(
                    square,
                    square
                        .extras_id()
                        .and_then(|extras_id| self.visible_extras.get(&extras_id)),
                );
                let font_weight = if style.flags.contains(StyleFlags::BOLD) {
                    700.0
                } else {
                    400.0
                };
                let layout = layout_text_styled(
                    tcx,
                    Arc::from(cell_text),
                    self.font_size,
                    font_weight,
                    None,
                    Default::default(),
                    fg.to_rgba8().to_u8_array(),
                    Arc::from([]),
                    Some(&self.font_family),
                    None,
                );
                let glyph_scene = tcx.glyph_scene_scaled(&layout, self.device_scale);
                let x = column as f64 * self.cell_width as f64;
                let text_y =
                    y as f64 + ((self.line_height - layout.height()) * 0.5).max(0.0) as f64;
                let italic = style
                    .flags
                    .contains(StyleFlags::ITALIC)
                    .then(|| Affine::skew(-0.18, 0.0))
                    .unwrap_or(Affine::IDENTITY);
                scene.append(
                    &glyph_scene,
                    Some(
                        Affine::translate((x, text_y))
                            * italic
                            * Affine::scale(self.device_scale.recip()),
                    ),
                );
            }
        }

        if display_offset == 0 && cursor.is_visible() && cursor.pos.row >= 0 {
            let x = cursor.pos.col.0 as f64 * self.cell_width as f64;
            let y = cursor.pos.row.0 as f64 * self.line_height as f64;
            if let Some(visual) = cursor_visual(
                self.focused,
                self.cursor_on,
                cursor.content,
                x,
                y,
                self.cell_width as f64,
                self.line_height as f64,
            ) {
                let color = terminal_ansi_color(
                    AnsiColor::Named(NamedColor::Cursor),
                    true,
                    &colors,
                    self.theme_foreground,
                    self.theme_background,
                );
                match visual {
                    CursorVisual::Filled(rect) => scene.fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        color.with_alpha(0.43),
                        None,
                        &rect,
                    ),
                    CursorVisual::Hollow(rect) => {
                        scene.stroke(&Stroke::new(1.0), Affine::IDENTITY, color, None, &rect)
                    }
                }
            }
        }

        self.graphics.draw_kitty(
            &mut scene,
            &kitty_placements,
            KittyLayer::AboveText,
            &viewport,
            clip,
            scale,
        );

        if let Some(error) = &self.spawn_error {
            let layout = layout_text_styled(
                tcx,
                Arc::from(format!("terminal: {error}")),
                13.0,
                400.0,
                None,
                Default::default(),
                [248, 113, 113, 255],
                Arc::from([]),
                Some(&self.font_family),
                Some(width),
            );
            scene.append(
                &tcx.glyph_scene_scaled(&layout, self.device_scale),
                Some(Affine::translate((4.0, 4.0)) * Affine::scale(self.device_scale.recip())),
            );
        }
        scene
    }

    fn paint_scaled(
        &mut self,
        width: f32,
        height: f32,
        device_scale: f64,
        tcx: &mut TextContext,
    ) -> Scene {
        self.device_scale = device_scale.max(f64::EPSILON);
        self.paint(width, height, tcx)
    }

    fn handle_event(&mut self, event: &UiEvent) -> WidgetEventResult {
        if matches!(
            event,
            UiEvent::Pointer(pointer)
                if pointer.phase == PointerPhase::Down
                    && pointer.button != Some(PointerButton::Primary)
        ) {
            self.last_click = None;
        }
        match event {
            UiEvent::Pointer(pointer)
                if pointer.phase == PointerPhase::Down
                    && pointer.button == Some(PointerButton::Primary)
                    && terminal_primary_shortcut(pointer.modifiers)
                    && let Some(url) =
                        self.hyperlink_at(pointer.position.x, pointer.position.y) =>
            {
                self.last_click = None;
                self.pending_hyperlink = Some(PendingHyperlink {
                    url,
                    origin: (pointer.position.x, pointer.position.y),
                    cancelled: false,
                });
                WidgetEventResult::HANDLED
            }
            UiEvent::Pointer(pointer)
                if pointer.phase == PointerPhase::Move && self.pending_hyperlink.is_some() =>
            {
                let pending = self.pending_hyperlink.as_mut().unwrap();
                let distance = (pointer.position.x - pending.origin.0)
                    .hypot(pointer.position.y - pending.origin.1);
                pending.cancelled |= distance > SELECTION_DRAG_THRESHOLD;
                WidgetEventResult::HANDLED
            }
            UiEvent::Pointer(pointer)
                if pointer.phase == PointerPhase::Up && self.pending_hyperlink.is_some() =>
            {
                let pending = self.pending_hyperlink.take().unwrap();
                if !pending.cancelled
                    && self
                        .hyperlink_at(pointer.position.x, pointer.position.y)
                        .as_deref()
                        == Some(pending.url.as_str())
                {
                    self.pending_host_actions
                        .push_back(HostAction::OpenUrl(pending.url));
                }
                WidgetEventResult::HANDLED
            }
            UiEvent::Pointer(pointer)
                if pointer.phase == PointerPhase::Cancel && self.pending_hyperlink.is_some() =>
            {
                self.pending_hyperlink = None;
                WidgetEventResult::HANDLED
            }
            UiEvent::Pointer(pointer) if !self.selecting && self.report_pointer(pointer) => {
                WidgetEventResult::HANDLED
            }
            UiEvent::Pointer(pointer)
                if pointer.phase == PointerPhase::Down
                    && pointer.button == Some(PointerButton::Primary) =>
            {
                self.begin_or_extend_selection(
                    pointer.position.x,
                    pointer.position.y,
                    pointer.modifiers,
                );
                WidgetEventResult::HANDLED
            }
            UiEvent::Pointer(pointer) if pointer.phase == PointerPhase::Move && self.selecting => {
                self.update_selection(pointer.position.x, pointer.position.y);
                WidgetEventResult::HANDLED
            }
            UiEvent::Pointer(pointer) if pointer.phase == PointerPhase::Up && self.selecting => {
                self.update_selection(pointer.position.x, pointer.position.y);
                self.finish_selection_gesture();
                WidgetEventResult::HANDLED
            }
            UiEvent::Pointer(pointer)
                if pointer.phase == PointerPhase::Cancel && self.selecting =>
            {
                self.last_click = None;
                self.finish_selection_gesture();
                WidgetEventResult::HANDLED
            }
            UiEvent::TextInput(text) => {
                if self.exit_reported {
                    return WidgetEventResult::HANDLED;
                }
                self.begin_terminal_input();
                self.send_bytes(text.as_bytes().to_vec());
                WidgetEventResult::HANDLED
            }
            UiEvent::Paste(text) => {
                if self.exit_reported {
                    return WidgetEventResult::HANDLED;
                }
                let bracketed = self.terminal.lock().mode().contains(Mode::BRACKETED_PASTE);
                self.begin_terminal_input();
                self.send_bytes(encode_paste(text, bracketed));
                WidgetEventResult::HANDLED
            }
            UiEvent::Key(key) => {
                // Multi-click gestures must consist exclusively of pointer
                // clicks. Local terminal shortcuts (copy, paste, scrollback)
                // do not reach `begin_terminal_input`, but still separate two
                // clicks just like input forwarded to the PTY does.
                if key.phase == KeyPhase::Down {
                    self.last_click = None;
                }
                if terminal_clipboard_shortcut(key.modifiers) && key.key.eq_ignore_ascii_case("a") {
                    if key.phase == KeyPhase::Down {
                        self.select_all();
                    }
                    return WidgetEventResult::HANDLED;
                }
                if terminal_clipboard_shortcut(key.modifiers) && key.key.eq_ignore_ascii_case("c") {
                    return if key.phase == KeyPhase::Down {
                        self.selected_text()
                            .map_or(WidgetEventResult::HANDLED, WidgetEventResult::copy)
                    } else {
                        WidgetEventResult::HANDLED
                    };
                }
                if terminal_clipboard_shortcut(key.modifiers) && key.key.eq_ignore_ascii_case("v") {
                    return if self.exit_reported {
                        WidgetEventResult::HANDLED
                    } else if key.phase == KeyPhase::Down {
                        WidgetEventResult::paste()
                    } else {
                        WidgetEventResult::HANDLED
                    };
                }
                let mode = self.terminal.lock().mode();
                if key.phase == KeyPhase::Down
                    && key.modifiers == Modifiers::SHIFT
                    && !mode.contains(Mode::ALT_SCREEN)
                {
                    let scroll = match key.key.as_str() {
                        "Home" => Some(Scroll::Top),
                        "End" => Some(Scroll::Bottom),
                        "PageUp" => Some(Scroll::PageUp),
                        "PageDown" => Some(Scroll::PageDown),
                        _ => None,
                    };
                    if let Some(scroll) = scroll {
                        self.terminal.lock().scroll_display(scroll);
                        return WidgetEventResult::HANDLED;
                    }
                }
                if self.exit_reported {
                    return WidgetEventResult::HANDLED;
                }
                let bytes = self.key_bytes(key);
                if bytes.is_empty() {
                    WidgetEventResult::IGNORED
                } else {
                    self.begin_terminal_input();
                    self.send_bytes(bytes);
                    if key.phase == KeyPhase::Down {
                        WidgetEventResult::handled_consuming_key_text()
                    } else {
                        WidgetEventResult::HANDLED
                    }
                }
            }
            UiEvent::Wheel(wheel) => {
                self.last_click = None;
                let context = self.wheel_context(wheel);
                let lines = self.wheel_lines.push(context, wheel.delta_y);
                if self.selecting {
                    self.scroll_active_selection(wheel, lines);
                    return WidgetEventResult::HANDLED;
                }
                if self.report_wheel(wheel, lines) {
                    return WidgetEventResult::HANDLED;
                }
                if self.report_alternate_scroll(lines) {
                    return WidgetEventResult::HANDLED;
                }
                if lines == 0 {
                    // Keep fractional trackpad input owned by the terminal
                    // until it accumulates into a whole grid line.
                    return WidgetEventResult::HANDLED;
                }
                self.terminal.lock().scroll_display(Scroll::Delta(lines));
                WidgetEventResult::HANDLED
            }
            _ => WidgetEventResult::IGNORED,
        }
    }

    fn attribute_changed(&mut self, name: &str, value: &str) {
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
    }

    fn accepts_focus(&self) -> bool {
        true
    }

    fn style_changed(&mut self, style: &WidgetStyle) {
        if !self.inherit_theme {
            return;
        }
        self.theme_foreground = style.color;
        self.theme_background = style
            .background
            .unwrap_or_else(|| named_color(NamedColor::Background, false));
    }

    fn intrinsic_size(&self) -> Option<[f32; 2]> {
        Some([
            DEFAULT_COLUMNS as f32 * self.cell_width,
            DEFAULT_ROWS as f32 * self.line_height,
        ])
    }

    fn focus_changed(&mut self, focused: bool) {
        if !focused {
            self.pending_hyperlink = None;
            self.last_click = None;
        }
        if !focused && self.selecting {
            self.finish_selection_gesture();
        }
        self.focused = focused;
        self.cursor_on = true;
        let terminal = self.terminal.lock();
        self.next_cursor_blink = (focused && !self.exit_reported && terminal.blinking_cursor)
            .then(|| Instant::now() + Duration::from_millis(500));
        let report_focus = terminal.mode().contains(Mode::FOCUS_IN_OUT);
        drop(terminal);
        if report_focus && !self.exit_reported {
            self.send_bytes(if focused {
                b"\x1b[I".to_vec()
            } else {
                b"\x1b[O".to_vec()
            });
        }
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.window_to_local = [1.0, 0.0, 0.0, 1.0, -f64::from(x), -f64::from(y)];
    }

    fn set_window_to_local(&mut self, transform: [f64; 6]) {
        self.window_to_local = transform;
    }

    fn animation_deadline(&self) -> Option<Instant> {
        [self.next_cursor_blink, self.next_selection_scroll]
            .into_iter()
            .flatten()
            .min()
    }

    fn set_wake_callback(&mut self, wake: WakeCallback) {
        self.listener.set_wake(wake);
    }

    fn poll_async(&mut self) -> bool {
        self.handle_rio_events()
    }

    fn take_host_action(&mut self) -> Option<HostAction> {
        self.pending_host_actions.pop_front()
    }

    fn take_node_event(&mut self) -> Option<WidgetNodeEvent> {
        self.pending_node_events.pop_front()
    }

    fn complete_host_action(&mut self, result: HostActionResult) {
        match result {
            HostActionResult::Clipboard { request_id, text } => {
                if let Some(formatter) = self.pending_clipboard_loads.remove(&request_id) {
                    self.send_bytes(formatter(text.as_deref().unwrap_or("")).into_bytes());
                }
            }
        }
    }

    fn attribute_removed(&mut self, name: &str) {
        match name {
            "command" if !self.launch_started || self.spawn_error.is_some() => {
                self.attribute_changed("command", "");
            }
            "args" if !self.launch_started || self.spawn_error.is_some() => {
                self.attribute_changed("args", "[]");
            }
            "cwd" if !self.launch_started || self.spawn_error.is_some() => {
                self.attribute_changed("cwd", "");
            }
            "command" | "args" | "cwd" => {
                tracing::warn!(
                    attribute = name,
                    "ignored terminal launch option removal after PTY start"
                );
            }
            "allow-clipboard-read" => self.allow_clipboard_read = false,
            "sync-window-title" => {
                if self.sync_window_title {
                    self.pending_host_actions
                        .push_back(HostAction::SetWindowTitle(None));
                }
                self.sync_window_title = false;
            }
            "selection-background" => {
                self.selection_background = DEFAULT_SELECTION_BACKGROUND;
            }
            "selection-foreground" => self.selection_foreground = None,
            "inherit-theme" => {
                self.inherit_theme = false;
                self.theme_foreground = named_color(NamedColor::Foreground, true);
                self.theme_background = named_color(NamedColor::Background, false);
            }
            "font-size" => {
                self.font_size = DEFAULT_FONT_SIZE;
                self.metrics_dirty = true;
            }
            "line-height" => {
                self.explicit_line_height = None;
                self.metrics_dirty = true;
            }
            "font-family" => {
                self.font_family = Arc::from("monospace");
                self.metrics_dirty = true;
            }
            _ => {}
        }
    }

    fn unmount(&mut self) {
        self.shutdown_pty();
        if self.sync_window_title {
            self.sync_window_title = false;
            self.pending_host_actions
                .push_back(HostAction::SetWindowTitle(None));
        }
    }
}

impl TerminalWidget {
    fn key_bytes(&self, key: &wabou_shell::KeyEvent) -> Vec<u8> {
        let mode = self.terminal.lock().mode();
        if mode.intersects(Mode::KITTY_KEYBOARD_PROTOCOL) {
            let sequence = kitty_keyboard::build_key_sequence(key, mode);
            if !sequence.is_empty() || key.phase == KeyPhase::Up {
                return sequence;
            }
        } else if key.phase == KeyPhase::Up {
            return Vec::new();
        }
        if mode.contains(Mode::APP_KEYPAD)
            && let Some(sequence) = application_keypad_sequence(key)
        {
            return sequence;
        }
        if alt_graph_text(key).is_some() {
            return Vec::new();
        }
        if key.modifiers.control()
            && let Some(byte) = legacy_control_byte(&key.key)
        {
            let mut bytes = Vec::with_capacity(1 + usize::from(key.modifiers.alt()));
            if key.modifiers.alt() {
                bytes.push(0x1b);
            }
            bytes.push(byte);
            return bytes;
        }
        if key.modifiers.alt() && key.key.chars().count() == 1 {
            let mut bytes = vec![0x1b];
            bytes.extend_from_slice(key.key.as_bytes());
            return bytes;
        }

        let modifier = 1
            + u8::from(key.modifiers.shift())
            + u8::from(key.modifiers.alt()) * 2
            + u8::from(key.modifiers.control()) * 4;
        let modified = modifier != 1;
        let csi_key = |final_byte: char| format!("\x1b[1;{modifier}{final_byte}").into_bytes();

        match key.key.as_str() {
            "Enter" => b"\r".to_vec(),
            "Backspace" if key.modifiers.alt() => b"\x1b\x7f".to_vec(),
            "Backspace" => vec![0x7f],
            "Tab" if key.modifiers.shift() => b"\x1b[Z".to_vec(),
            "Tab" => b"\t".to_vec(),
            "Escape" => vec![0x1b],
            "ArrowUp" if modified => csi_key('A'),
            "ArrowDown" if modified => csi_key('B'),
            "ArrowRight" if modified => csi_key('C'),
            "ArrowLeft" if modified => csi_key('D'),
            "ArrowUp" if mode.contains(Mode::APP_CURSOR) => b"\x1bOA".to_vec(),
            "ArrowDown" if mode.contains(Mode::APP_CURSOR) => b"\x1bOB".to_vec(),
            "ArrowRight" if mode.contains(Mode::APP_CURSOR) => b"\x1bOC".to_vec(),
            "ArrowLeft" if mode.contains(Mode::APP_CURSOR) => b"\x1bOD".to_vec(),
            "ArrowUp" => b"\x1b[A".to_vec(),
            "ArrowDown" => b"\x1b[B".to_vec(),
            "ArrowRight" => b"\x1b[C".to_vec(),
            "ArrowLeft" => b"\x1b[D".to_vec(),
            "Home" if modified => csi_key('H'),
            "End" if modified => csi_key('F'),
            "Home" if mode.contains(Mode::APP_CURSOR) => b"\x1bOH".to_vec(),
            "End" if mode.contains(Mode::APP_CURSOR) => b"\x1bOF".to_vec(),
            "Home" => b"\x1b[H".to_vec(),
            "End" => b"\x1b[F".to_vec(),
            "Insert" => format!(
                "\x1b[2{}~",
                if modified {
                    format!(";{modifier}")
                } else {
                    String::new()
                }
            )
            .into_bytes(),
            "Delete" => format!(
                "\x1b[3{}~",
                if modified {
                    format!(";{modifier}")
                } else {
                    String::new()
                }
            )
            .into_bytes(),
            "PageUp" => format!(
                "\x1b[5{}~",
                if modified {
                    format!(";{modifier}")
                } else {
                    String::new()
                }
            )
            .into_bytes(),
            "PageDown" => format!(
                "\x1b[6{}~",
                if modified {
                    format!(";{modifier}")
                } else {
                    String::new()
                }
            )
            .into_bytes(),
            "F1" if modified => csi_key('P'),
            "F2" if modified => csi_key('Q'),
            "F3" if modified => csi_key('R'),
            "F4" if modified => csi_key('S'),
            "F1" => b"\x1bOP".to_vec(),
            "F2" => b"\x1bOQ".to_vec(),
            "F3" => b"\x1bOR".to_vec(),
            "F4" => b"\x1bOS".to_vec(),
            "F5" => function_key(15, modifier),
            "F6" => function_key(17, modifier),
            "F7" => function_key(18, modifier),
            "F8" => function_key(19, modifier),
            "F9" => function_key(20, modifier),
            "F10" => function_key(21, modifier),
            "F11" => function_key(23, modifier),
            "F12" => function_key(24, modifier),
            "F13" => csi_function_key('P', force_shift_modifier(modifier)),
            "F14" => csi_function_key('Q', force_shift_modifier(modifier)),
            "F15" => csi_function_key('R', force_shift_modifier(modifier)),
            "F16" => csi_function_key('S', force_shift_modifier(modifier)),
            "F17" => function_key(15, force_shift_modifier(modifier)),
            "F18" => function_key(17, force_shift_modifier(modifier)),
            "F19" => function_key(18, force_shift_modifier(modifier)),
            "F20" => function_key(19, force_shift_modifier(modifier)),
            "F21" => function_key(20, force_shift_modifier(modifier)),
            "F22" => function_key(21, force_shift_modifier(modifier)),
            "F23" => function_key(23, force_shift_modifier(modifier)),
            "F24" => function_key(24, force_shift_modifier(modifier)),
            "F25" => csi_function_key('P', force_control_modifier(modifier)),
            "F26" => csi_function_key('Q', force_control_modifier(modifier)),
            "F27" => csi_function_key('R', force_control_modifier(modifier)),
            "F28" => csi_function_key('S', force_control_modifier(modifier)),
            "F29" => function_key(15, force_control_modifier(modifier)),
            "F30" => function_key(17, force_control_modifier(modifier)),
            "F31" => function_key(18, force_control_modifier(modifier)),
            "F32" => function_key(19, force_control_modifier(modifier)),
            "F33" => function_key(20, force_control_modifier(modifier)),
            "F34" => function_key(21, force_control_modifier(modifier)),
            "F35" => function_key(23, force_control_modifier(modifier)),
            _ => Vec::new(),
        }
    }
}

fn legacy_control_byte(key: &str) -> Option<u8> {
    let byte = key.as_bytes().first().copied()?;
    if key.len() != 1 {
        return None;
    }
    match byte {
        b'@'..=b'_' => Some(byte & 0x1f),
        b'a'..=b'z' => Some(byte & 0x1f),
        b' ' | b'2' => Some(0x00),
        b'3'..=b'7' => Some(byte - b'3' + 0x1b),
        b'8' | b'?' => Some(0x7f),
        b'/' => Some(0x1f),
        _ => None,
    }
}

fn application_keypad_sequence(key: &wabou_shell::KeyEvent) -> Option<Vec<u8>> {
    if key.location != wabou_shell::KeyLocation::Numpad {
        return None;
    }
    let final_byte = if key.code.contains("Numpad0") {
        'p'
    } else if key.code.contains("Numpad1") {
        'q'
    } else if key.code.contains("Numpad2") {
        'r'
    } else if key.code.contains("Numpad3") {
        's'
    } else if key.code.contains("Numpad4") {
        't'
    } else if key.code.contains("Numpad5") {
        'u'
    } else if key.code.contains("Numpad6") {
        'v'
    } else if key.code.contains("Numpad7") {
        'w'
    } else if key.code.contains("Numpad8") {
        'x'
    } else if key.code.contains("Numpad9") {
        'y'
    } else if key.code.contains("NumpadDecimal") {
        'n'
    } else if key.code.contains("NumpadComma") {
        'l'
    } else if key.code.contains("NumpadAdd") {
        'k'
    } else if key.code.contains("NumpadSubtract") {
        'm'
    } else if key.code.contains("NumpadMultiply") {
        'j'
    } else if key.code.contains("NumpadDivide") {
        'o'
    } else if key.code.contains("NumpadEnter") {
        'M'
    } else if key.code.contains("NumpadEqual") {
        'X'
    } else {
        return None;
    };
    Some(vec![0x1b, b'O', final_byte as u8])
}

fn alt_graph_text(key: &wabou_shell::KeyEvent) -> Option<&str> {
    if !key.modifiers.control() || !key.modifiers.alt() || key.modifiers.meta() {
        return None;
    }
    let text = key.text_with_all_modifiers.as_deref()?;
    text.chars()
        .all(|character| !character.is_control())
        .then_some(text)
        .filter(|text| !text.is_empty())
}

fn function_key(number: u8, modifier: u8) -> Vec<u8> {
    if modifier == 1 {
        format!("\x1b[{number}~").into_bytes()
    } else {
        format!("\x1b[{number};{modifier}~").into_bytes()
    }
}

fn force_shift_modifier(modifier: u8) -> u8 {
    ((modifier - 1) | 1) + 1
}

fn force_control_modifier(modifier: u8) -> u8 {
    ((modifier - 1) | 4) + 1
}

fn csi_function_key(final_byte: char, modifier: u8) -> Vec<u8> {
    format!("\x1b[1;{modifier}{final_byte}").into_bytes()
}

fn encode_paste(text: &str, bracketed: bool) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(text.len() + usize::from(bracketed) * 12);
    if bracketed {
        encoded.extend_from_slice(b"\x1b[200~");
        for character in text
            .chars()
            .filter(|character| !character.is_control() || matches!(character, '\t' | '\n' | '\r'))
        {
            let mut utf8 = [0; 4];
            encoded.extend_from_slice(character.encode_utf8(&mut utf8).as_bytes());
        }
        encoded.extend_from_slice(b"\x1b[201~");
        return encoded;
    }

    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '\r' if characters.peek() == Some(&'\n') => {
                characters.next();
                encoded.push(b'\r');
            }
            '\n' | '\r' => encoded.push(b'\r'),
            '\t' => encoded.push(b'\t'),
            character if character.is_control() => {}
            character => {
                let mut utf8 = [0; 4];
                encoded.extend_from_slice(character.encode_utf8(&mut utf8).as_bytes());
            }
        }
    }
    encoded
}

fn normal_mouse_sequence(code: u8, column: usize, row: usize, utf8: bool) -> Option<Vec<u8>> {
    let max_coordinate = if utf8 { 2015 } else { 223 };
    if column > max_coordinate || row > max_coordinate {
        return None;
    }

    let mut sequence = vec![0x1b, b'[', b'M', code + 32];
    let mut encode_coordinate = |coordinate: usize| {
        let value = coordinate + 32;
        if utf8 && coordinate >= 96 {
            sequence.push((0xc0 + value / 64) as u8);
            sequence.push((0x80 + (value & 63)) as u8);
        } else {
            sequence.push(value as u8);
        }
    };
    encode_coordinate(column);
    encode_coordinate(row);
    Some(sequence)
}

fn cell_text(square: Square, extra: Option<&Extras>) -> String {
    let mut text = String::from(square.c());
    if let Some(extra) = extra {
        text.extend(extra.zerowidth.iter().copied());
    }
    text
}

fn selection_contains_square(selection: SelectionRange, point: Pos, square: Square) -> bool {
    if selection.contains(point) {
        return true;
    }

    match square.wide() {
        Wide::Wide => selection.contains(Pos::new(point.row, point.col + 1)),
        Wide::Spacer if point.col.0 > 0 => {
            selection.contains(Pos::new(point.row, Column(point.col.0 - 1)))
        }
        _ => false,
    }
}

fn cursor_visual(
    focused: bool,
    cursor_on: bool,
    shape: CursorShape,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Option<CursorVisual> {
    if shape == CursorShape::Hidden || (focused && !cursor_on) {
        return None;
    }
    if !focused {
        return Some(CursorVisual::Hollow(Rect::new(
            x + 0.5,
            y + 0.5,
            x + width - 0.5,
            y + height - 0.5,
        )));
    }

    let rect = match shape {
        CursorShape::Beam => Rect::new(x, y, x + 2.0, y + height),
        CursorShape::Underline => Rect::new(x, y + height - 2.0, x + width, y + height),
        CursorShape::Block => Rect::new(x, y, x + width, y + height),
        CursorShape::Hidden => unreachable!(),
    };
    Some(CursorVisual::Filled(rect))
}

fn fill_cell(
    scene: &mut Scene,
    column: usize,
    row: usize,
    cell_width: f32,
    line_height: f32,
    device_scale: f64,
    color: Color,
) {
    let rect = cell_fill_rect(column, row, cell_width, line_height, device_scale);
    scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, &rect);
}

fn cell_fill_rect(
    column: usize,
    row: usize,
    cell_width: f32,
    line_height: f32,
    device_scale: f64,
) -> Rect {
    let scale = device_scale.max(f64::EPSILON);
    let snap = |logical: f64| (logical * scale).round() / scale;
    let width = f64::from(cell_width);
    let height = f64::from(line_height);
    Rect::new(
        snap(column as f64 * width),
        snap(row as f64 * height),
        snap((column + 1) as f64 * width),
        snap((row + 1) as f64 * height),
    )
}

fn draw_cell_decorations(
    scene: &mut Scene,
    column: usize,
    y: f32,
    cell_width: f32,
    line_height: f32,
    style: rio_vt::crosswords::style::Style,
    foreground: Color,
    colors: &TermColors,
    theme_foreground: Color,
    theme_background: Color,
    decoration_override: Option<Color>,
) {
    let (strike_color, underline_color) = decoration_colors(
        style,
        foreground,
        colors,
        theme_foreground,
        theme_background,
        decoration_override,
    );
    let x = column as f32 * cell_width;
    let fill = |scene: &mut Scene, color: Color, x0: f32, y0: f32, x1: f32, y1: f32| {
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            color,
            None,
            &Rect::new(x0 as f64, y0 as f64, x1 as f64, y1 as f64),
        );
    };
    if style.flags.contains(StyleFlags::STRIKEOUT) {
        let strike_y = y + line_height * 0.52;
        fill(
            scene,
            strike_color,
            x,
            strike_y,
            x + cell_width,
            strike_y + 1.0,
        );
    }
    let underline_y = y + line_height - 2.0;
    if style.flags.contains(StyleFlags::DOUBLE_UNDERLINE) {
        fill(
            scene,
            underline_color,
            x,
            underline_y - 2.0,
            x + cell_width,
            underline_y - 1.0,
        );
        fill(
            scene,
            underline_color,
            x,
            underline_y,
            x + cell_width,
            underline_y + 1.0,
        );
    } else if style.flags.contains(StyleFlags::DOTTED_UNDERLINE) {
        let mut dot_x = x;
        while dot_x < x + cell_width {
            fill(
                scene,
                underline_color,
                dot_x,
                underline_y,
                (dot_x + 1.0).min(x + cell_width),
                underline_y + 1.0,
            );
            dot_x += 2.0;
        }
    } else if style.flags.contains(StyleFlags::DASHED_UNDERLINE) {
        let mut dash_x = x;
        while dash_x < x + cell_width {
            fill(
                scene,
                underline_color,
                dash_x,
                underline_y,
                (dash_x + 3.0).min(x + cell_width),
                underline_y + 1.0,
            );
            dash_x += 5.0;
        }
    } else if style.flags.contains(StyleFlags::UNDERCURL) {
        let mut curl_x = x;
        let mut high = true;
        while curl_x < x + cell_width {
            let curl_y = underline_y - if high { 1.0 } else { 0.0 };
            fill(
                scene,
                underline_color,
                curl_x,
                curl_y,
                (curl_x + 2.0).min(x + cell_width),
                curl_y + 1.0,
            );
            curl_x += 2.0;
            high = !high;
        }
    } else if style.flags.contains(StyleFlags::UNDERLINE) {
        fill(
            scene,
            underline_color,
            x,
            underline_y,
            x + cell_width,
            underline_y + 1.0,
        );
    }
}

fn decoration_colors(
    style: Style,
    foreground: Color,
    colors: &TermColors,
    theme_foreground: Color,
    theme_background: Color,
    decoration_override: Option<Color>,
) -> (Color, Color) {
    let strike = decoration_override.unwrap_or(foreground);
    let underline = decoration_override
        .or_else(|| {
            style.underline_color.map(|color| {
                terminal_ansi_color(color, true, colors, theme_foreground, theme_background)
            })
        })
        .unwrap_or(foreground);
    (strike, underline)
}

fn ansi_color(color: AnsiColor, foreground: bool) -> Color {
    match color {
        AnsiColor::Spec(ColorRgb { r, g, b }) => Color::from_rgb8(r, g, b),
        AnsiColor::Indexed(index) => indexed_color(index),
        AnsiColor::Named(name) => named_color(name, foreground),
    }
}

fn terminal_ansi_color(
    color: AnsiColor,
    foreground: bool,
    colors: &TermColors,
    theme_foreground: Color,
    theme_background: Color,
) -> Color {
    let override_index = match color {
        AnsiColor::Indexed(index) => Some(index as usize),
        AnsiColor::Named(name) => Some(name as usize),
        AnsiColor::Spec(_) => None,
    };
    override_index
        .and_then(|index| colors[index])
        .map(color_from_array)
        .unwrap_or_else(|| match color {
            AnsiColor::Named(
                NamedColor::Foreground | NamedColor::LightForeground | NamedColor::DimForeground,
            ) => theme_foreground,
            AnsiColor::Named(NamedColor::Background) => theme_background,
            _ => ansi_color(color, foreground),
        })
}

fn terminal_indexed_color(index: u8, colors: &TermColors) -> Color {
    colors[index as usize]
        .map(color_from_array)
        .unwrap_or_else(|| indexed_color(index))
}

fn color_from_array([r, g, b, a]: [f32; 4]) -> Color {
    Color::from_rgba8(
        (r * 255.0).round().clamp(0.0, 255.0) as u8,
        (g * 255.0).round().clamp(0.0, 255.0) as u8,
        (b * 255.0).round().clamp(0.0, 255.0) as u8,
        (a * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

fn named_color(color: NamedColor, foreground: bool) -> Color {
    let fallback = if foreground { 0xe2e8f0 } else { 0x0f172a };
    let rgb = match color {
        NamedColor::Black | NamedColor::DimBlack => 0x1e293b,
        NamedColor::Red | NamedColor::DimRed => 0xef4444,
        NamedColor::Green | NamedColor::DimGreen => 0x22c55e,
        NamedColor::Yellow | NamedColor::DimYellow => 0xeab308,
        NamedColor::Blue | NamedColor::DimBlue => 0x3b82f6,
        NamedColor::Magenta | NamedColor::DimMagenta => 0xd946ef,
        NamedColor::Cyan | NamedColor::DimCyan => 0x06b6d4,
        NamedColor::White | NamedColor::DimWhite => 0xcbd5e1,
        NamedColor::LightBlack => 0x64748b,
        NamedColor::LightRed => 0xf87171,
        NamedColor::LightGreen => 0x4ade80,
        NamedColor::LightYellow => 0xfacc15,
        NamedColor::LightBlue => 0x60a5fa,
        NamedColor::LightMagenta => 0xe879f9,
        NamedColor::LightCyan => 0x22d3ee,
        NamedColor::LightWhite => 0xf8fafc,
        NamedColor::Foreground | NamedColor::LightForeground | NamedColor::DimForeground => {
            fallback
        }
        NamedColor::Background => 0x0f172a,
        NamedColor::Cursor => 0xe2e8f0,
    };
    Color::from_rgb8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

fn indexed_color(index: u8) -> Color {
    if index < 16 {
        const TABLE: [u32; 16] = [
            0x1e293b, 0xef4444, 0x22c55e, 0xeab308, 0x3b82f6, 0xd946ef, 0x06b6d4, 0xcbd5e1,
            0x64748b, 0xf87171, 0x4ade80, 0xfacc15, 0x60a5fa, 0xe879f9, 0x22d3ee, 0xf8fafc,
        ];
        let rgb = TABLE[index as usize];
        return Color::from_rgb8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8);
    }
    if index < 232 {
        let value = index - 16;
        let component = |n: u8| if n == 0 { 0 } else { 55 + n * 40 };
        return Color::from_rgb8(
            component(value / 36),
            component((value / 6) % 6),
            component(value % 6),
        );
    }
    let gray = 8 + (index - 232) * 10;
    Color::from_rgb8(gray, gray, gray)
}

fn terminal_color(
    index: usize,
    colors: &TermColors,
    theme_foreground: Color,
    theme_background: Color,
) -> ColorRgb {
    let color = if let Some(color) = (index < TERMINAL_COLOR_COUNT)
        .then(|| colors[index])
        .flatten()
    {
        color_from_array(color)
    } else {
        match index {
            index if index < 256 => indexed_color(index as u8),
            index
                if index == NamedColor::Foreground as usize
                    || index == NamedColor::LightForeground as usize
                    || index == NamedColor::DimForeground as usize =>
            {
                theme_foreground
            }
            index if index == NamedColor::Background as usize => theme_background,
            index if index == NamedColor::Cursor as usize => named_color(NamedColor::Cursor, true),
            _ => named_color(NamedColor::Foreground, true),
        }
    };
    let [r, g, b, _] = color.to_rgba8().to_u8_array();
    ColorRgb { r, g, b }
}

fn dim(color: Color) -> Color {
    let [r, g, b, a] = color.to_rgba8().to_u8_array();
    Color::from_rgba8(
        (r as f32 * 0.66) as u8,
        (g as f32 * 0.66) as u8,
        (b as f32 * 0.66) as u8,
        a,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use wabou_shell::{ClipboardRequest, KeyEvent, KeyLocation, Modifiers, Point, PointerEvent};

    fn pointer(phase: PointerPhase, x: f64, y: f64, buttons: u32) -> UiEvent {
        pointer_with_modifiers(phase, x, y, buttons, Modifiers::default())
    }

    fn pointer_with_modifiers(
        phase: PointerPhase,
        x: f64,
        y: f64,
        buttons: u32,
        modifiers: Modifiers,
    ) -> UiEvent {
        UiEvent::Pointer(PointerEvent {
            phase,
            position: Point { x, y },
            button: Some(PointerButton::Primary),
            buttons,
            modifiers,
        })
    }

    #[test]
    fn rio_parser_preserves_text_and_sgr_cells() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"\x1b[31mred\x1b[0m plain\r\n\x1b[3;4;9mstyled\x1b[0m");
        assert!(widget.visible_line(0).starts_with("red plain"));

        let terminal = widget.terminal.lock();
        let rows = terminal.visible_rows();
        let style = terminal.grid.style_set.get(rows[0][Column(0)].style_id());
        assert_eq!(style.fg, AnsiColor::Named(NamedColor::Red));
        let decorated = terminal.grid.style_set.get(rows[1][Column(0)].style_id());
        assert!(decorated.flags.contains(StyleFlags::ITALIC));
        assert!(decorated.flags.contains(StyleFlags::UNDERLINE));
        assert!(decorated.flags.contains(StyleFlags::STRIKEOUT));
    }

    #[test]
    fn renderer_preserves_combining_grapheme_extras() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed("e\u{301}".as_bytes());

        let terminal = widget.terminal.lock();
        let rows = terminal.visible_rows();
        let square = rows[0][Column(0)];
        assert!(square.has_extras());
        assert_eq!(
            cell_text(
                square,
                square
                    .extras_id()
                    .and_then(|id| terminal.grid.extras_table.get(id))
            ),
            "e\u{301}"
        );
    }

    #[test]
    fn primary_click_on_osc8_link_opens_only_after_a_committed_click() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"\x1b]8;;https://example.com/docs\x1b\\link\x1b]8;;\x1b\\");

        let down = pointer_with_modifiers(PointerPhase::Down, 1.0, 1.0, 1, Modifiers::CONTROL);
        assert_eq!(widget.handle_event(&down), WidgetEventResult::HANDLED);
        assert_eq!(widget.take_host_action(), None);
        let up = pointer_with_modifiers(PointerPhase::Up, 1.0, 1.0, 0, Modifiers::CONTROL);
        assert_eq!(widget.handle_event(&up), WidgetEventResult::HANDLED);
        assert_eq!(
            widget.take_host_action(),
            Some(HostAction::OpenUrl("https://example.com/docs".into()))
        );
    }

    #[test]
    fn dragging_or_cancelling_an_osc8_link_does_not_open_it() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"\x1b]8;;https://example.com/docs\x1b\\link\x1b]8;;\x1b\\");

        for terminal_phase in [PointerPhase::Up, PointerPhase::Cancel] {
            widget.handle_event(&pointer_with_modifiers(
                PointerPhase::Down,
                1.0,
                1.0,
                1,
                Modifiers::CONTROL,
            ));
            widget.handle_event(&pointer_with_modifiers(
                PointerPhase::Move,
                20.0,
                1.0,
                1,
                Modifiers::CONTROL,
            ));
            widget.handle_event(&pointer_with_modifiers(
                terminal_phase,
                20.0,
                1.0,
                0,
                Modifiers::CONTROL,
            ));
            assert_eq!(widget.take_host_action(), None);
            assert!(widget.pending_hyperlink.is_none());
        }
    }

    #[test]
    fn osc8_links_resolve_from_wide_spacers_and_scrollback_rows() {
        let mut wide = TerminalWidget::headless(20, 4);
        wide.feed("\x1b]8;;https://example.com/wide\x1b\\你\x1b]8;;\x1b\\".as_bytes());
        assert_eq!(
            wide.hyperlink_at(f64::from(DEFAULT_CELL_WIDTH) * 1.5, 1.0),
            Some("https://example.com/wide".into())
        );

        let mut history = TerminalWidget::headless(20, 2);
        history
            .feed(b"\x1b]8;;https://example.com/history\x1b\\old\x1b]8;;\x1b\\\r\nnew\r\nlatest");
        history.terminal.lock().scroll_display(Scroll::Top);
        assert!(history.terminal.lock().display_offset() > 0);
        assert_eq!(
            history.hyperlink_at(1.0, 1.0),
            Some("https://example.com/history".into())
        );
    }

    #[test]
    fn selection_highlight_covers_both_cells_of_wide_glyphs() {
        let row = Line(0);
        let lead_point = Pos::new(row, Column(3));
        let spacer_point = Pos::new(row, Column(4));
        let mut lead = Square::default();
        lead.set_wide(Wide::Wide);
        let mut spacer = Square::default();
        spacer.set_wide(Wide::Spacer);

        let trailing_half = SelectionRange::new(spacer_point, spacer_point, false);
        assert!(selection_contains_square(trailing_half, lead_point, lead));
        assert!(selection_contains_square(
            trailing_half,
            spacer_point,
            spacer
        ));

        let leading_half = SelectionRange::new(lead_point, lead_point, false);
        assert!(selection_contains_square(leading_half, lead_point, lead));
        assert!(selection_contains_square(
            leading_half,
            spacer_point,
            spacer
        ));
    }

    #[test]
    fn wide_spacer_cells_keep_background_and_decorations() {
        let scene_paths = |bytes: &[u8]| {
            let mut widget = TerminalWidget::headless(10, 2);
            widget.feed(bytes);
            let mut tcx = TextContext::new();
            widget
                .paint(
                    DEFAULT_CELL_WIDTH * 10.0,
                    DEFAULT_LINE_HEIGHT * 2.0,
                    &mut tcx,
                )
                .encoding()
                .n_paths
        };

        let narrow = scene_paths(b"\x1b[41;4mA");
        let wide = scene_paths("\x1b[41;4m你".as_bytes());
        assert!(
            wide >= narrow + 2,
            "the wide spacer must contribute its background and underline"
        );
    }

    #[test]
    fn font_metrics_drive_cell_width_and_grid_resize() {
        let mut widget = TerminalWidget::headless(80, 24);
        widget.attribute_changed("font-size", "20px");
        assert!(widget.metrics_dirty);
        let mut tcx = TextContext::new();
        let family: Arc<str> = Arc::from("monospace");
        let expected = layout_text_styled(
            &mut tcx,
            Arc::from("0"),
            20.0,
            400.0,
            None,
            Default::default(),
            [255, 255, 255, 255],
            Arc::from([]),
            Some(&family),
            None,
        )
        .width();

        widget.paint(400.0, 200.0, &mut tcx);

        assert!((widget.cell_width - expected).abs() < 0.001);
        assert_eq!(widget.size.columns, (400.0 / expected).floor() as usize);
        assert!(!widget.metrics_dirty);

        widget.attribute_changed("line-height", "32px");
        widget.attribute_changed("font-size", "22px");
        widget.paint(400.0, 200.0, &mut tcx);
        assert_eq!(widget.line_height, 32.0);
        widget.attribute_removed("line-height");
        widget.paint(400.0, 200.0, &mut tcx);
        assert_ne!(widget.line_height, 32.0);

        widget.attribute_changed("line-height", "12px");
        widget.attribute_changed("font-size", "30px");
        widget.paint(400.0, 200.0, &mut tcx);
        assert_eq!(widget.line_height, 30.0);
        widget.attribute_changed("font-size", "10px");
        widget.paint(400.0, 200.0, &mut tcx);
        assert_eq!(
            widget.line_height, 12.0,
            "the authored line height must survive temporary font-size clamping"
        );
    }

    #[test]
    fn terminal_selection_colors_are_themeable_and_removable() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.attribute_changed("selection-background", "#11223380");
        widget.attribute_changed("selection-foreground", "rgb(240 241 242)");
        assert_eq!(
            widget.selection_background,
            Color::from_rgba8(0x11, 0x22, 0x33, 0x80)
        );
        assert_eq!(
            widget.selection_foreground,
            Some(Color::from_rgb8(240, 241, 242))
        );

        widget.attribute_changed("selection-background", "not-a-color");
        widget.attribute_changed("selection-foreground", "not-a-color");
        assert_eq!(
            widget.selection_background,
            Color::from_rgba8(0x11, 0x22, 0x33, 0x80)
        );
        assert_eq!(
            widget.selection_foreground,
            Some(Color::from_rgb8(240, 241, 242))
        );

        widget.attribute_removed("selection-background");
        widget.attribute_removed("selection-foreground");
        assert_eq!(widget.selection_background, DEFAULT_SELECTION_BACKGROUND);
        assert_eq!(widget.selection_foreground, None);
    }

    #[test]
    fn sgr_underline_color_does_not_recolor_strikeout() {
        let foreground = Color::from_rgb8(10, 20, 30);
        let underline = ColorRgb {
            r: 200,
            g: 40,
            b: 50,
        };
        let style = Style {
            underline_color: Some(AnsiColor::Spec(underline)),
            flags: StyleFlags::STRIKEOUT | StyleFlags::UNDERLINE,
            ..Style::default()
        };
        let colors = TermColors::default();
        let (strike_color, underline_color) = decoration_colors(
            style,
            foreground,
            &colors,
            named_color(NamedColor::Foreground, true),
            named_color(NamedColor::Background, false),
            None,
        );

        assert_eq!(strike_color, foreground);
        assert_eq!(underline_color, Color::from_rgb8(200, 40, 50));

        let selected = Color::from_rgb8(240, 241, 242);
        assert_eq!(
            decoration_colors(
                style,
                foreground,
                &colors,
                named_color(NamedColor::Foreground, true),
                named_color(NamedColor::Background, false),
                Some(selected),
            ),
            (selected, selected)
        );
    }

    #[test]
    fn headless_input_uses_terminal_escape_sequences() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.handle_event(&UiEvent::Key(KeyEvent {
            phase: KeyPhase::Down,
            key: "ArrowUp".into(),
            key_without_modifiers: "ArrowUp".into(),
            code: "ArrowUp".into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: Modifiers::default(),
            repeat: false,
        }));
        widget.handle_event(&UiEvent::TextInput("λ".into()));
        assert_eq!(widget.take_input(), b"\x1b[A\xce\xbb");
    }

    #[test]
    fn key_encoding_honors_terminal_modes_and_modifiers() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"\x1b[?1h");
        let key = |name: &str, modifiers: Modifiers| {
            UiEvent::Key(KeyEvent {
                phase: KeyPhase::Down,
                key: name.into(),
                key_without_modifiers: name.into(),
                code: name.into(),
                text: None,
                text_with_all_modifiers: None,
                location: Default::default(),
                modifiers,
                repeat: false,
            })
        };

        widget.handle_event(&key("ArrowUp", Modifiers::default()));
        widget.handle_event(&key("ArrowRight", Modifiers::SHIFT));
        let alt = widget.handle_event(&key("x", Modifiers::ALT));

        assert!(alt.consumes_key_text());
        assert_eq!(widget.take_input(), b"\x1bOA\x1b[1;2C\x1bx");
    }

    #[test]
    fn legacy_keyboard_encodes_extended_function_keys() {
        let mut widget = TerminalWidget::headless(20, 4);
        let key = |name: &str, modifiers: Modifiers| {
            UiEvent::Key(KeyEvent {
                phase: KeyPhase::Down,
                key: name.into(),
                key_without_modifiers: name.into(),
                code: name.into(),
                text: None,
                text_with_all_modifiers: None,
                location: KeyLocation::Standard,
                modifiers,
                repeat: false,
            })
        };

        for (name, sequence) in [
            ("F13", "\x1b[1;2P"),
            ("F14", "\x1b[1;2Q"),
            ("F15", "\x1b[1;2R"),
            ("F16", "\x1b[1;2S"),
            ("F17", "\x1b[15;2~"),
            ("F18", "\x1b[17;2~"),
            ("F19", "\x1b[18;2~"),
            ("F20", "\x1b[19;2~"),
            ("F21", "\x1b[20;2~"),
            ("F22", "\x1b[21;2~"),
            ("F23", "\x1b[23;2~"),
            ("F24", "\x1b[24;2~"),
            ("F25", "\x1b[1;5P"),
            ("F26", "\x1b[1;5Q"),
            ("F27", "\x1b[1;5R"),
            ("F28", "\x1b[1;5S"),
            ("F29", "\x1b[15;5~"),
            ("F30", "\x1b[17;5~"),
            ("F31", "\x1b[18;5~"),
            ("F32", "\x1b[19;5~"),
            ("F33", "\x1b[20;5~"),
            ("F34", "\x1b[21;5~"),
            ("F35", "\x1b[23;5~"),
        ] {
            let result = widget.handle_event(&key(name, Modifiers::empty()));
            assert!(result.consumes_key_text());
            assert_eq!(widget.take_input(), sequence.as_bytes(), "{name}");
        }

        widget.handle_event(&key("F15", Modifiers::SHIFT | Modifiers::CONTROL));
        assert_eq!(widget.take_input(), b"\x1b[1;6R");
        widget.handle_event(&key("F27", Modifiers::ALT));
        assert_eq!(widget.take_input(), b"\x1b[1;7R");

        widget.feed(b"\x1b[>8u");
        widget.handle_event(&key("F35", Modifiers::empty()));
        assert_eq!(
            widget.take_input(),
            b"\x1b[57398u",
            "Kitty mode must keep its dedicated F35 keycode"
        );
    }

    #[test]
    fn application_keypad_mode_encodes_physical_numpad_keys() {
        let mut widget = TerminalWidget::headless(20, 4);
        let key = |logical: &str, code: &str| {
            UiEvent::Key(KeyEvent {
                phase: KeyPhase::Down,
                key: logical.into(),
                key_without_modifiers: logical.into(),
                code: format!("Code({code})"),
                text: Some(logical.into()),
                text_with_all_modifiers: Some(logical.into()),
                location: KeyLocation::Numpad,
                modifiers: Modifiers::empty(),
                repeat: false,
            })
        };

        let numeric = widget.handle_event(&key("1", "Numpad1"));
        assert!(!numeric.is_handled());
        assert!(widget.take_input().is_empty());

        widget.feed(b"\x1b=");
        for (logical, code, sequence) in [
            ("0", "Numpad0", "\x1bOp"),
            ("1", "Numpad1", "\x1bOq"),
            ("5", "Numpad5", "\x1bOu"),
            ("9", "Numpad9", "\x1bOy"),
            (".", "NumpadDecimal", "\x1bOn"),
            (",", "NumpadComma", "\x1bOl"),
            ("+", "NumpadAdd", "\x1bOk"),
            ("-", "NumpadSubtract", "\x1bOm"),
            ("*", "NumpadMultiply", "\x1bOj"),
            ("/", "NumpadDivide", "\x1bOo"),
            ("Enter", "NumpadEnter", "\x1bOM"),
            ("=", "NumpadEqual", "\x1bOX"),
        ] {
            let result = widget.handle_event(&key(logical, code));
            assert!(result.consumes_key_text(), "{code}");
            assert_eq!(widget.take_input(), sequence.as_bytes(), "{code}");
        }

        widget.feed(b"\x1b>");
        assert!(!widget.handle_event(&key("1", "Numpad1")).is_handled());
        assert!(widget.take_input().is_empty());
    }

    #[test]
    fn negotiated_kitty_keyboard_reports_press_and_release_without_double_text() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"\x1b[>31u");
        let mut key = KeyEvent {
            phase: KeyPhase::Down,
            key: "x".into(),
            key_without_modifiers: "x".into(),
            code: "Code(KeyX)".into(),
            text: Some("x".into()),
            text_with_all_modifiers: Some("x".into()),
            location: KeyLocation::Standard,
            modifiers: Modifiers::empty(),
            repeat: false,
        };

        let pressed = widget.handle_event(&UiEvent::Key(key.clone()));
        assert!(pressed.is_handled());
        assert!(pressed.consumes_key_text());
        key.phase = KeyPhase::Up;
        let released = widget.handle_event(&UiEvent::Key(key));
        assert!(released.is_handled());
        assert_eq!(widget.take_input(), b"\x1b[120;1;120u\x1b[120;1:3u");
    }

    #[test]
    fn legacy_control_keys_cover_ascii_control_range() {
        let mut widget = TerminalWidget::headless(20, 4);
        let key = |name: &str, modifiers: Modifiers| {
            UiEvent::Key(KeyEvent {
                phase: KeyPhase::Down,
                key: name.into(),
                key_without_modifiers: name.into(),
                code: name.into(),
                text: None,
                text_with_all_modifiers: None,
                location: Default::default(),
                modifiers,
                repeat: false,
            })
        };

        for name in ["@", "a", "[", "\\", "]", "^", "_", "?", "3", "8"] {
            widget.handle_event(&key(name, Modifiers::CONTROL));
        }
        widget.handle_event(&key("a", Modifiers::CONTROL | Modifiers::ALT));

        assert_eq!(
            widget.take_input(),
            [
                0x00, 0x01, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x7f, 0x1b, 0x7f, 0x1b, 0x01
            ]
        );

        let alt_graph = widget.handle_event(&UiEvent::Key(KeyEvent {
            phase: KeyPhase::Down,
            key: "@".into(),
            key_without_modifiers: "q".into(),
            code: "KeyQ".into(),
            text: Some("@".into()),
            text_with_all_modifiers: Some("@".into()),
            location: Default::default(),
            modifiers: Modifiers::CONTROL | Modifiers::ALT,
            repeat: false,
        }));
        assert!(!alt_graph.is_handled());
        assert!(widget.take_input().is_empty());
        widget.handle_event(&UiEvent::TextInput("@".into()));
        assert_eq!(widget.take_input(), b"@");
    }

    #[test]
    fn scrollback_shortcuts_are_local_and_typing_returns_to_bottom() {
        let mut widget = TerminalWidget::headless(20, 2);
        widget.feed(b"one\r\ntwo\r\nthree\r\nfour");
        let key = |name: &str, modifiers: Modifiers| {
            UiEvent::Key(KeyEvent {
                phase: KeyPhase::Down,
                key: name.into(),
                key_without_modifiers: name.into(),
                code: name.into(),
                text: None,
                text_with_all_modifiers: None,
                location: Default::default(),
                modifiers,
                repeat: false,
            })
        };

        widget.handle_event(&key("PageUp", Modifiers::SHIFT));
        assert!(widget.terminal.lock().display_offset() > 0);
        assert!(widget.take_input().is_empty());
        widget.handle_event(&key("Home", Modifiers::SHIFT));
        let top = widget.terminal.lock().display_offset();
        assert!(top >= 2);

        widget.begin_selection(1.0, 1.0, false);
        widget.update_selection(f64::from(DEFAULT_CELL_WIDTH) * 2.75, 1.0);
        assert!(widget.selected_text().is_some());
        let ignored = widget.handle_event(&key("Unidentified", Modifiers::default()));
        assert!(!ignored.is_handled());
        assert!(widget.selected_text().is_some());
        assert_eq!(widget.terminal.lock().display_offset(), top);

        widget.handle_event(&key("ArrowUp", Modifiers::default()));
        assert_eq!(widget.take_input(), b"\x1b[A");
        assert_eq!(widget.terminal.lock().display_offset(), 0);
        assert!(widget.selected_text().is_none());

        widget.feed(b"\x1b[?1049h");
        widget.handle_event(&key("PageUp", Modifiers::SHIFT));
        assert_eq!(widget.take_input(), b"\x1b[5;2~");
    }

    #[test]
    fn paste_honors_bracketed_paste_mode() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"\x1b[?2004h");

        widget.handle_event(&UiEvent::Paste(
            "hello\n世界\tworld\x1b[201~unsafe\x03\0\u{7f}still pasted".into(),
        ));

        assert_eq!(
            widget.take_input(),
            "\x1b[200~hello\n世界\tworld[201~unsafestill pasted\x1b[201~".as_bytes()
        );
    }

    #[test]
    fn plain_paste_converts_line_breaks_to_enter_keys() {
        let mut widget = TerminalWidget::headless(20, 4);

        widget.handle_event(&UiEvent::Paste(
            "one\ntwo\r\n三\tthree\rfour\x1b[31m\x03\0\u{7f}".into(),
        ));

        assert_eq!(
            widget.take_input(),
            "one\rtwo\r三\tthree\rfour[31m".as_bytes()
        );
    }

    #[test]
    fn resizing_updates_rio_grid() {
        let mut widget = TerminalWidget::headless(80, 24);
        widget.resize(DEFAULT_CELL_WIDTH * 40.0, DEFAULT_LINE_HEIGHT * 10.0);
        let terminal = widget.terminal.lock();
        assert_eq!((terminal.columns(), terminal.screen_lines()), (40, 10));
    }

    #[test]
    fn lazy_terminal_collects_initial_process_attributes_before_launch() {
        let mut widget = TerminalWidget::lazy_default_shell();
        assert!(!widget.launch_started);
        assert!(widget.pty_send.is_none());

        widget.attribute_changed("command", "ssh");
        widget.attribute_changed("args", r#"["-t","example.com"]"#);
        widget.attribute_changed("cwd", "/tmp/project");

        assert_eq!(
            widget.launch,
            Some(LaunchConfig {
                command: "ssh".into(),
                args: vec!["-t".into(), "example.com".into()],
                cwd: Some("/tmp/project".into()),
                login_shell: false,
            })
        );
        assert!(!widget.launch_started);
        assert!(widget.spawn_error.is_none());
    }

    #[test]
    fn pty_spawn_injects_terminal_capabilities_without_mutating_parent_environment() {
        let launch = LaunchConfig::default_shell();
        let (command, args) = pty_spawn_parts(&launch);

        #[cfg(not(windows))]
        {
            assert_eq!(command, "/usr/bin/env");
            assert!(args.iter().any(|arg| arg == "TERM=xterm-256color"));
            assert!(args.iter().any(|arg| arg == "COLORTERM=truecolor"));
            assert!(args.iter().any(|arg| arg == "TERM_PROGRAM=wabou"));
            assert!(
                args.iter()
                    .any(|arg| arg == concat!("TERM_PROGRAM_VERSION=", env!("CARGO_PKG_VERSION")))
            );
            assert!(args.iter().any(|arg| arg == &launch.command));
            #[cfg(target_os = "macos")]
            assert_eq!(args.last().map(String::as_str), Some("-l"));
        }
        #[cfg(windows)]
        {
            assert_eq!(command, quote_windows_command_arg(&launch.command));
            assert_eq!(
                args,
                launch
                    .args
                    .iter()
                    .map(|arg| quote_windows_command_arg(arg))
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn windows_command_line_quoting_preserves_argument_boundaries() {
        assert_eq!(quote_windows_command_arg("plain"), "plain");
        assert_eq!(quote_windows_command_arg(""), r#""""#);
        assert_eq!(
            quote_windows_command_arg(r"C:\Program Files\tool.exe"),
            r#""C:\Program Files\tool.exe""#
        );
        assert_eq!(
            quote_windows_command_arg(r#"say "hello""#),
            r#""say \"hello\"""#
        );
        assert_eq!(
            quote_windows_command_arg("trailing slash \\"),
            "\"trailing slash \\\\\""
        );
    }

    #[cfg(unix)]
    #[test]
    fn child_reaper_waits_for_and_collects_exited_processes() {
        let child = std::process::Command::new("/bin/sh")
            .args(["-c", "exit 0"])
            .spawn()
            .unwrap();
        let pid = child.id() as libc::pid_t;
        // Dropping std::process::Child does not wait; ownership is handed to
        // the same reaper used after Rio drops a terminal PTY.
        drop(child);

        spawn_child_reaper(pid).unwrap().join().unwrap().unwrap();
        let mut status = 0;
        assert_eq!(
            unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) },
            -1
        );
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ECHILD)
        );
    }

    #[cfg(unix)]
    #[test]
    fn child_reaper_escalates_for_terminal_jobs_that_ignore_hangup() {
        use std::io::BufRead;
        use std::os::unix::process::CommandExt;

        let mut command = std::process::Command::new("/bin/sh");
        command
            .args(["-c", "trap '' HUP; echo ready; exec sleep 30"])
            .stdout(std::process::Stdio::piped());
        unsafe {
            command.pre_exec(|| {
                if libc::setsid() == -1 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(())
                }
            });
        }
        let mut child = command.spawn().unwrap();
        let pid = child.id() as libc::pid_t;
        let mut ready = String::new();
        std::io::BufReader::new(child.stdout.take().unwrap())
            .read_line(&mut ready)
            .unwrap();
        assert_eq!(ready, "ready\n");
        drop(child);

        let started = Instant::now();
        spawn_child_reaper(pid).unwrap().join().unwrap().unwrap();
        assert!(started.elapsed() >= Duration::from_millis(400));
        assert!(started.elapsed() < Duration::from_secs(3));
        let mut status = 0;
        assert_eq!(
            unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) },
            -1
        );
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ECHILD)
        );
    }

    #[cfg(unix)]
    #[test]
    fn unmount_shuts_down_and_reaps_the_terminal_process() {
        let mut widget =
            TerminalWidget::spawn("/bin/sh", vec!["-c".into(), "exec sleep 30".into()], None);
        let pid = widget.child_pid.expect("spawned terminal child");

        widget.unmount();

        assert!(widget.pty_send.is_none());
        assert!(widget.child_pid.is_none());
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let result = unsafe { libc::kill(pid, 0) };
            if result == -1 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "terminal child {pid} was not reaped"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn removing_unlaunched_process_attributes_restores_launch_defaults() {
        let mut widget = TerminalWidget::lazy_default_shell();
        widget.attribute_changed("command", "ssh");
        widget.attribute_changed("args", r#"["example.com"]"#);
        widget.attribute_changed("cwd", "/tmp/project");

        widget.attribute_removed("command");
        widget.attribute_removed("args");
        widget.attribute_removed("cwd");

        assert_eq!(widget.launch, Some(LaunchConfig::default_shell()));
        assert!(!widget.launch_started);
        assert!(widget.spawn_error.is_none());
    }

    #[test]
    fn invalid_lazy_terminal_args_block_launch_until_corrected() {
        let mut widget = TerminalWidget::lazy_default_shell();
        widget.attribute_changed("args", "not-json");
        widget.ensure_launched();
        assert!(!widget.launch_started);
        assert!(
            widget
                .spawn_error
                .as_deref()
                .is_some_and(|error| { error.starts_with("invalid terminal args JSON:") })
        );

        widget.attribute_changed("args", r#"["-l"]"#);
        assert!(widget.launch_error.is_none());
        assert!(widget.spawn_error.is_none());
        assert_eq!(widget.launch.as_ref().unwrap().args, ["-l"]);
    }

    #[test]
    fn failed_terminal_spawn_can_be_reconfigured_without_retrying_each_frame() {
        let mut widget = TerminalWidget::lazy_default_shell();
        widget.attribute_changed("command", "/wabou/definitely/missing-terminal-command");
        widget.ensure_launched();

        assert!(widget.launch_started);
        assert!(widget.spawn_error.is_some());
        widget.ensure_launched();
        assert!(widget.launch_started, "a failed launch remains attempted");

        widget.attribute_changed("command", "/bin/sh");
        assert!(!widget.launch_started);
        assert!(widget.spawn_error.is_none());
        assert_eq!(widget.launch.as_ref().unwrap().command, "/bin/sh");
    }

    #[test]
    fn terminal_reports_physical_text_area_and_cell_dimensions() {
        let mut widget = TerminalWidget::headless(40, 10);
        widget.device_scale = 2.0;
        widget.resize(340.0, 180.0);

        // 340 logical pixels still fit 40 columns, but the pixel width must
        // update independently from the grid and be reported in device pixels.
        assert_eq!((widget.size.columns, widget.size.rows), (40, 10));
        widget.feed(b"\x1b[14t\x1b[16t");
        assert!(widget.poll_async());

        assert_eq!(widget.take_input(), b"\x1b[4;360;680t\x1b[6;36;17t");
    }

    #[test]
    fn terminal_size_saturates_pty_fields_without_losing_rio_dimensions() {
        let size = TerminalSize::from_viewport(
            40_000.0,
            20_000.0,
            DEFAULT_CELL_WIDTH,
            DEFAULT_LINE_HEIGHT,
            4.0,
        );

        assert_eq!((size.pixel_width, size.pixel_height), (160_000, 80_000));
        assert_eq!(
            (size.winsize().width, size.winsize().height),
            (u16::MAX, u16::MAX)
        );
    }

    #[test]
    fn parser_damage_wakes_the_host_once_and_is_drained() {
        let mut widget = TerminalWidget::headless(20, 4);
        let wakes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let wake_count = wakes.clone();
        widget.set_wake_callback(Arc::new(move || {
            wake_count.fetch_add(1, Ordering::Relaxed);
        }));

        widget.feed(b"output");
        assert_eq!(wakes.load(Ordering::Relaxed), 1);
        assert!(widget.poll_async());
        assert!(!widget.poll_async());
    }

    #[test]
    fn paint_consumes_damage_and_reuses_visible_row_storage() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"first");
        assert!(widget.terminal.lock().peek_damage_event().is_some());
        let mut tcx = TextContext::new();

        widget.paint(
            DEFAULT_CELL_WIDTH * 20.0,
            DEFAULT_LINE_HEIGHT * 4.0,
            &mut tcx,
        );

        assert!(widget.terminal.lock().peek_damage_event().is_none());
        let outer_capacity = widget.visible_rows.capacity();
        let row_buffers = widget
            .visible_rows
            .iter()
            .map(|row| row.inner.as_ptr())
            .collect::<Vec<_>>();

        widget.feed(b" second");
        widget.paint(
            DEFAULT_CELL_WIDTH * 20.0,
            DEFAULT_LINE_HEIGHT * 4.0,
            &mut tcx,
        );

        assert_eq!(widget.visible_rows.capacity(), outer_capacity);
        assert_eq!(
            widget
                .visible_rows
                .iter()
                .map(|row| row.inner.as_ptr())
                .collect::<Vec<_>>(),
            row_buffers
        );
        assert!(widget.visible_line(0).starts_with("first second"));
        assert!(widget.terminal.lock().peek_damage_event().is_none());
    }

    #[test]
    fn kitty_graphic_updates_flow_from_vt_parser_into_renderer_cache() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"\x1b_Ga=T,f=32,s=1,v=1,i=42,p=9;/wAA/w==\x1b\\");

        assert!(widget.poll_async());
        assert!(widget.graphics.contains(rio_graphics::kitty_image_key(42)));
        let terminal = widget.terminal.lock();
        assert!(terminal.graphics.kitty_placements.contains_key(&(42, 9)));
    }

    #[test]
    fn osc_titles_are_node_scoped_while_clipboard_store_remains_a_host_action() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"\x1b]0;project shell\x07\x1b]52;c;aGVsbG8=\x07");

        assert!(widget.poll_async());
        assert_eq!(
            widget.take_host_action(),
            Some(HostAction::SetClipboard("hello".into()))
        );
        assert_eq!(widget.take_host_action(), None);
        let title = widget.take_node_event().expect("title node event");
        assert_eq!(title.event_code, event::TERMINALTITLECHANGE);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&title.json).unwrap(),
            serde_json::json!({ "title": "project shell", "subtitle": null })
        );

        let mut synced = TerminalWidget::headless(20, 4);
        synced.attribute_changed("sync-window-title", "true");
        synced.feed(b"\x1b]0;synced shell\x07");
        synced.poll_async();
        assert_eq!(
            synced.take_host_action(),
            Some(HostAction::SetWindowTitle(Some("synced shell".into())))
        );
        synced.attribute_removed("sync-window-title");
        assert!(!synced.sync_window_title);
        assert_eq!(
            synced.take_host_action(),
            Some(HostAction::SetWindowTitle(None))
        );
        synced.attribute_changed("sync-window-title", "true");
        synced.unmount();
        assert_eq!(
            synced.take_host_action(),
            Some(HostAction::SetWindowTitle(None))
        );
    }

    #[test]
    fn terminal_subtitle_and_title_reset_remain_scoped_to_the_widget_node() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.listener.send_event(
            RioEvent::TitleWithSubtitle("editor".into(), "README.md".into()),
            WindowId::from(0),
        );
        widget
            .listener
            .send_event(RioEvent::ResetTitle, WindowId::from(0));

        widget.poll_async();
        assert_eq!(widget.take_host_action(), None);
        let titled = widget.take_node_event().expect("title event");
        assert_eq!(titled.event_code, event::TERMINALTITLECHANGE);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&titled.json).unwrap(),
            serde_json::json!({ "title": "editor", "subtitle": "README.md" })
        );
        let reset = widget.take_node_event().expect("reset event");
        assert_eq!(reset.event_code, event::TERMINALTITLECHANGE);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&reset.json).unwrap(),
            serde_json::json!({ "title": null, "subtitle": null })
        );
    }

    #[test]
    fn osc_current_directory_is_scoped_and_deduplicated() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"\x1b]7;file://localhost/tmp/project%20dir\x07");
        widget.poll_async();

        let cwd = widget.take_node_event().expect("cwd event");
        assert_eq!(cwd.event_code, event::TERMINALCWDCHANGE);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&cwd.json).unwrap(),
            serde_json::json!({ "path": "/tmp/project dir" })
        );

        widget.feed(b"\x1b]7;file://localhost/tmp/project%20dir\x07");
        widget.poll_async();
        assert_eq!(widget.take_node_event(), None);

        widget.feed(b"\x1b]7;file://localhost/tmp/next\x07");
        widget.poll_async();
        let next = widget.take_node_event().expect("changed cwd event");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&next.json).unwrap(),
            serde_json::json!({ "path": "/tmp/next" })
        );
    }

    #[test]
    fn terminal_lifecycle_reports_are_exposed_as_node_events() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.listener.send_event(
            RioEvent::ProgressReport(rio_vt::event::ProgressReport {
                state: ProgressState::Set,
                progress: Some(42),
            }),
            WindowId::from(0),
        );
        widget.listener.send_event(
            RioEvent::DesktopNotification {
                title: "Build \"done\"".into(),
                body: "All tests passed".into(),
            },
            WindowId::from(0),
        );
        widget
            .listener
            .send_event(RioEvent::CloseTerminal(0), WindowId::from(0));
        widget
            .listener
            .send_event(RioEvent::Exit, WindowId::from(0));
        widget
            .listener
            .send_event(RioEvent::Quit, WindowId::from(0));

        widget.poll_async();
        let progress = widget.take_node_event().expect("progress event");
        assert_eq!(progress.event_code, event::TERMINALPROGRESS);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&progress.json).unwrap(),
            serde_json::json!({ "state": "set", "progress": 42 })
        );
        let notification = widget.take_node_event().expect("notification event");
        assert_eq!(notification.event_code, event::TERMINALNOTIFICATION);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&notification.json).unwrap(),
            serde_json::json!({
                "title": "Build \"done\"",
                "body": "All tests passed"
            })
        );
        assert_eq!(
            widget.take_node_event(),
            Some(WidgetNodeEvent::json(
                event::TERMINALEXIT,
                r#"{"reason":"exit"}"#
            ))
        );
        assert_eq!(widget.take_node_event(), None);
        assert!(widget.exit_reported);
        assert!(widget.next_cursor_blink.is_none());
    }

    #[test]
    fn exited_terminal_is_read_only_but_remains_selectable_and_copyable() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"abcdef\x1b[?1000h\x1b[?1006h\x1b[?1004h");
        widget.report_exit_once();
        let cell = f64::from(DEFAULT_CELL_WIDTH);

        // Stale application mouse mode must not prevent local selection once
        // there is no process left to receive mouse reports.
        widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Move, cell * 5.75, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Up, cell * 5.75, 1.0, 0));
        assert_eq!(widget.selected_text().as_deref(), Some("abcdef"));
        assert!(widget.take_input().is_empty());

        let modifiers = if cfg!(target_os = "macos") {
            Modifiers::META
        } else {
            Modifiers::CONTROL | Modifiers::SHIFT
        };
        let key = |name: &str| {
            UiEvent::Key(KeyEvent {
                phase: KeyPhase::Down,
                key: name.into(),
                key_without_modifiers: name.into(),
                code: format!("Key{}", name.to_ascii_uppercase()),
                text: None,
                text_with_all_modifiers: None,
                location: KeyLocation::Standard,
                modifiers,
                repeat: false,
            })
        };
        assert_eq!(
            widget.handle_event(&key("c")).clipboard_request(),
            Some(&ClipboardRequest::Write("abcdef".into()))
        );
        assert_eq!(widget.handle_event(&key("v")).clipboard_request(), None);

        widget.handle_event(&UiEvent::TextInput("x".into()));
        widget.handle_event(&UiEvent::Paste("paste".into()));
        widget.handle_event(&UiEvent::Key(KeyEvent {
            phase: KeyPhase::Down,
            key: "Enter".into(),
            key_without_modifiers: "Enter".into(),
            code: "Enter".into(),
            text: None,
            text_with_all_modifiers: None,
            location: KeyLocation::Standard,
            modifiers: Modifiers::empty(),
            repeat: false,
        }));
        widget.focus_changed(true);

        assert_eq!(widget.selected_text().as_deref(), Some("abcdef"));
        assert!(widget.take_input().is_empty());
        assert!(widget.next_cursor_blink.is_none());
    }

    #[test]
    fn osc_clipboard_read_requires_opt_in_and_formats_the_reply() {
        let mut denied = TerminalWidget::headless(20, 4);
        denied.feed(b"\x1b]52;c;?\x07");
        assert!(denied.poll_async());
        assert_eq!(denied.take_host_action(), None);
        assert_eq!(denied.take_input(), b"\x1b]52;c;\x07");

        let mut allowed = TerminalWidget::headless(20, 4);
        allowed.attribute_changed("allow-clipboard-read", "true");
        allowed.feed(b"\x1b]52;c;?\x07");
        assert!(allowed.poll_async());
        let request_id = match allowed.take_host_action() {
            Some(HostAction::ReadClipboard { request_id }) => request_id,
            action => panic!("expected clipboard read, got {action:?}"),
        };
        assert!(allowed.take_input().is_empty());
        allowed.complete_host_action(HostActionResult::Clipboard {
            request_id,
            text: Some("hello".into()),
        });
        assert_eq!(allowed.take_input(), b"\x1b]52;c;aGVsbG8=\x07");

        allowed.attribute_removed("allow-clipboard-read");
        assert!(!allowed.allow_clipboard_read);
    }

    #[test]
    fn color_queries_use_the_rendered_palette() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(
            b"\x1b]4;1;#010203\x07\x1b]4;1;?\x07\x1b]4;2;?\x07\
              \x1b]10;?\x07\x1b]11;?\x07\x1b]12;?\x07",
        );

        assert!(widget.poll_async());
        assert_eq!(
            widget.take_input(),
            b"\x1b]4;1;rgb:0101/0202/0303\x07\
              \x1b]4;2;rgb:2222/c5c5/5e5e\x07\
              \x1b]10;rgb:e2e2/e8e8/f0f0\x07\
              \x1b]11;rgb:0f0f/1717/2a2a\x07\
              \x1b]12;rgb:e2e2/e8e8/f0f0\x07"
        );
        let colors = widget.terminal.lock().colors;
        assert_eq!(
            terminal_indexed_color(1, &colors),
            Color::from_rgb8(1, 2, 3)
        );
    }

    #[test]
    fn host_theme_defaults_are_opt_in_and_osc_overrides_them() {
        let mut widget = TerminalWidget::headless(20, 4);
        let mut paint = Paint::default();
        paint.text_color = Color::from_rgb8(0x11, 0x22, 0x33);
        paint.background = Some(Color::from_rgb8(0x44, 0x55, 0x66));

        widget.style_changed(&WidgetStyle::from(&paint));
        assert_eq!(
            widget.theme_foreground,
            named_color(NamedColor::Foreground, true)
        );
        widget.attribute_changed("inherit-theme", "true");
        widget.style_changed(&WidgetStyle::from(&paint));
        assert_eq!(widget.theme_foreground, Color::from_rgb8(0x11, 0x22, 0x33));
        assert_eq!(widget.theme_background, Color::from_rgb8(0x44, 0x55, 0x66));

        widget.attribute_changed("inherit-theme", "false");
        assert_eq!(
            widget.theme_foreground,
            named_color(NamedColor::Foreground, true)
        );
        assert_eq!(
            widget.theme_background,
            named_color(NamedColor::Background, false)
        );
        widget.attribute_changed("inherit-theme", "true");
        widget.style_changed(&WidgetStyle::from(&paint));

        widget.feed(b"\x1b]10;#010203\x07\x1b]10;?\x07\x1b]11;?\x07");
        assert!(widget.poll_async());
        assert_eq!(
            widget.take_input(),
            b"\x1b]10;rgb:0101/0202/0303\x07\x1b]11;rgb:4444/5555/6666\x07"
        );
        let colors = widget.terminal.lock().colors;
        assert_eq!(
            terminal_ansi_color(
                AnsiColor::Named(NamedColor::Foreground),
                true,
                &colors,
                widget.theme_foreground,
                widget.theme_background,
            ),
            Color::from_rgb8(1, 2, 3)
        );
        widget.attribute_removed("inherit-theme");
        assert!(!widget.inherit_theme);
        assert_eq!(
            widget.theme_foreground,
            named_color(NamedColor::Foreground, true)
        );
    }

    #[test]
    fn cursor_blink_mode_and_bell_reach_the_frontend() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.focus_changed(true);
        assert!(
            widget.next_cursor_blink.is_none(),
            "default cursor is steady"
        );

        widget.feed(b"\x1b[1 q");
        assert!(widget.poll_async());
        assert!(widget.terminal.lock().blinking_cursor);
        assert!(widget.next_cursor_blink.is_some());

        widget.feed(b"\x1b[2 q\x07");
        assert!(widget.poll_async());
        assert!(!widget.terminal.lock().blinking_cursor);
        assert!(widget.next_cursor_blink.is_none());
        assert_eq!(
            widget.take_host_action(),
            Some(HostAction::RequestAttention)
        );
        assert_eq!(
            widget.take_node_event(),
            Some(WidgetNodeEvent::json(event::TERMINALBELL, "{}"))
        );
    }

    #[test]
    fn cursor_visual_stays_hollow_when_terminal_is_unfocused() {
        let cell = Rect::new(10.5, 20.5, 17.5, 37.5);
        assert_eq!(
            cursor_visual(false, false, CursorShape::Beam, 10.0, 20.0, 8.0, 18.0),
            Some(CursorVisual::Hollow(cell))
        );
        assert_eq!(
            cursor_visual(true, false, CursorShape::Block, 10.0, 20.0, 8.0, 18.0),
            None
        );
        assert_eq!(
            cursor_visual(true, true, CursorShape::Beam, 10.0, 20.0, 8.0, 18.0),
            Some(CursorVisual::Filled(Rect::new(10.0, 20.0, 12.0, 38.0)))
        );
        assert_eq!(
            cursor_visual(false, true, CursorShape::Hidden, 10.0, 20.0, 8.0, 18.0),
            None
        );
    }

    #[test]
    fn pointer_drag_selects_terminal_grid_text() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"hello world");
        widget.set_position(10.0, 20.0);

        widget.handle_event(&pointer(PointerPhase::Down, 10.1, 20.1, 1));
        widget.handle_event(&pointer(
            PointerPhase::Move,
            10.0 + f64::from(DEFAULT_CELL_WIDTH) * 4.75,
            20.1,
            1,
        ));
        widget.handle_event(&pointer(
            PointerPhase::Up,
            10.0 + f64::from(DEFAULT_CELL_WIDTH) * 4.75,
            20.1,
            0,
        ));

        assert_eq!(widget.selected_text().as_deref(), Some("hello"));
        assert!(!widget.selecting);

        let copy = widget.handle_event(&UiEvent::Key(KeyEvent {
            phase: KeyPhase::Down,
            key: "C".into(),
            key_without_modifiers: "c".into(),
            code: "KeyC".into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: if cfg!(target_os = "macos") {
                Modifiers::META
            } else {
                Modifiers::CONTROL | Modifiers::SHIFT
            },
            repeat: false,
        }));
        assert_eq!(
            copy.clipboard_request(),
            Some(&ClipboardRequest::Write("hello".into()))
        );

        let paste = widget.handle_event(&UiEvent::Key(KeyEvent {
            phase: KeyPhase::Down,
            key: "V".into(),
            key_without_modifiers: "v".into(),
            code: "KeyV".into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: if cfg!(target_os = "macos") {
                Modifiers::META
            } else {
                Modifiers::CONTROL | Modifiers::SHIFT
            },
            repeat: false,
        }));
        assert_eq!(paste.clipboard_request(), Some(&ClipboardRequest::Read));
    }

    #[test]
    fn selection_changes_are_deduplicated_and_exposed_to_solid() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"hello world");
        let cell = f64::from(DEFAULT_CELL_WIDTH);
        widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, 1.0, 1));
        assert!(widget.take_node_event().is_none());
        for index in 0..1_000 {
            let column = 1.25 + f64::from(index % 4);
            widget.handle_event(&pointer(PointerPhase::Move, cell * column, 1.0, 1));
        }
        widget.handle_event(&pointer(PointerPhase::Move, cell * 4.75, 1.0, 1));
        assert!(
            widget.take_node_event().is_none(),
            "dragging updates paint locally without serializing selection text"
        );
        widget.handle_event(&pointer(PointerPhase::Up, cell * 4.75, 1.0, 0));
        let selected = widget.take_node_event().expect("committed selection event");
        assert_eq!(selected.event_code, event::TERMINALSELECTIONCHANGE);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&selected.json).unwrap(),
            serde_json::json!({ "text": "hello", "kind": "simple" })
        );
        assert!(widget.take_node_event().is_none());

        widget.handle_event(&UiEvent::TextInput("x".into()));
        let cleared = widget.take_node_event().expect("selection clear event");
        assert_eq!(cleared.event_code, event::TERMINALSELECTIONCHANGE);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&cleared.json).unwrap(),
            serde_json::json!({ "text": null, "kind": null })
        );
        assert_eq!(widget.take_input(), b"x");
    }

    #[test]
    fn live_pty_output_cannot_publish_an_uncommitted_selection() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"hello world");
        widget.handle_rio_events();
        let cell = f64::from(DEFAULT_CELL_WIDTH);
        widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Move, cell * 4.75, 1.0, 1));

        widget.feed(b"!");
        widget.handle_rio_events();
        assert!(
            widget.take_node_event().is_none(),
            "PTY output must not bypass the pointer gesture's commit boundary"
        );

        widget.handle_event(&pointer(PointerPhase::Up, cell * 4.75, 1.0, 0));
        let selected = widget.take_node_event().expect("committed selection event");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&selected.json).unwrap(),
            serde_json::json!({ "text": "hello", "kind": "simple" })
        );
    }

    #[test]
    fn selection_event_kind_changes_even_when_text_is_identical() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"hello world");
        let mut simple = Selection::new(
            SelectionType::Simple,
            Pos::new(Line(0), Column(0)),
            Side::Left,
        );
        simple.update(Pos::new(Line(0), Column(4)), Side::Right);
        widget.terminal.lock().selection = Some(simple);
        widget.sync_selection_change();
        let simple = widget.take_node_event().unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&simple.json).unwrap(),
            serde_json::json!({ "text": "hello", "kind": "simple" })
        );

        widget.terminal.lock().selection = Some(Selection::new(
            SelectionType::Semantic,
            Pos::new(Line(0), Column(2)),
            Side::Left,
        ));
        widget.sync_selection_change();
        let word = widget.take_node_event().expect("selection kind changed");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&word.json).unwrap(),
            serde_json::json!({ "text": "hello", "kind": "word" })
        );
    }

    #[test]
    fn shift_click_extends_existing_terminal_selection_in_both_directions() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"hello world");
        let cell = f64::from(DEFAULT_CELL_WIDTH);

        widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Up, cell * 4.75, 1.0, 0));
        assert_eq!(widget.selected_text().as_deref(), Some("hello"));

        widget.handle_event(&pointer_with_modifiers(
            PointerPhase::Down,
            cell * 10.75,
            1.0,
            1,
            Modifiers::SHIFT,
        ));
        widget.handle_event(&pointer_with_modifiers(
            PointerPhase::Up,
            cell * 10.75,
            1.0,
            0,
            Modifiers::SHIFT,
        ));
        assert_eq!(widget.selected_text().as_deref(), Some("hello world"));

        widget.handle_event(&pointer(PointerPhase::Down, cell * 10.75, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Up, cell * 6.25, 1.0, 0));
        assert_eq!(widget.selected_text().as_deref(), Some("world"));

        widget.handle_event(&pointer_with_modifiers(
            PointerPhase::Down,
            cell * 0.25,
            1.0,
            1,
            Modifiers::SHIFT,
        ));
        widget.handle_event(&pointer_with_modifiers(
            PointerPhase::Up,
            cell * 0.25,
            1.0,
            0,
            Modifiers::SHIFT,
        ));
        assert_eq!(widget.selected_text().as_deref(), Some("hello world"));
    }

    #[test]
    fn triple_click_selects_a_soft_wrapped_logical_line() {
        let mut widget = TerminalWidget::headless(5, 4);
        widget.feed(b"abcdefghij\r\nnext");
        let x = f64::from(DEFAULT_CELL_WIDTH) * 1.25;
        let y = f64::from(DEFAULT_LINE_HEIGHT) * 1.25;

        for _ in 0..3 {
            widget.handle_event(&pointer(PointerPhase::Down, x, y, 1));
            widget.handle_event(&pointer(PointerPhase::Up, x, y, 0));
        }

        assert_eq!(widget.selected_text().as_deref(), Some("abcdefghij\n"));
    }

    #[test]
    fn reverse_drag_selection_preserves_visual_text_order() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"zero one two");
        let cell = f64::from(DEFAULT_CELL_WIDTH);

        widget.handle_event(&pointer(PointerPhase::Down, cell * 11.75, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Move, cell * 5.25, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Up, cell * 5.25, 1.0, 0));

        assert_eq!(widget.selected_text().as_deref(), Some("one two"));
    }

    #[test]
    fn selection_dragged_past_horizontal_edges_includes_boundary_cells() {
        let mut widget = TerminalWidget::headless(20, 2);
        widget.feed(b"0123456789abcdefghij");
        let cell = f64::from(DEFAULT_CELL_WIDTH);

        widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Move, cell * 25.0, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Up, cell * 25.0, 1.0, 0));
        assert_eq!(
            widget.selected_text().as_deref(),
            Some("0123456789abcdefghij")
        );

        widget.handle_event(&pointer(PointerPhase::Down, cell * 19.75, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Move, -cell * 5.0, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Up, -cell * 5.0, 1.0, 0));
        assert_eq!(
            widget.selected_text().as_deref(),
            Some("0123456789abcdefghij")
        );
    }

    #[test]
    fn alt_drag_keeps_rectangular_selection_after_modifier_release() {
        let mut widget = TerminalWidget::headless(10, 3);
        widget.feed(b"abcdef\r\nuvwxyz");
        let cell = f64::from(DEFAULT_CELL_WIDTH);
        let line = f64::from(DEFAULT_LINE_HEIGHT);

        widget.handle_event(&pointer_with_modifiers(
            PointerPhase::Down,
            cell * 1.25,
            line * 0.25,
            1,
            Modifiers::ALT,
        ));
        widget.handle_event(&pointer(PointerPhase::Move, cell * 3.75, line * 1.25, 1));
        widget.handle_event(&pointer(PointerPhase::Up, cell * 3.75, line * 1.25, 0));

        assert_eq!(widget.selected_text().as_deref(), Some("bcd\nvwx"));
    }

    #[test]
    fn rectangular_selection_never_copies_half_of_a_wide_glyph() {
        let mut widget = TerminalWidget::headless(10, 3);
        widget.feed("a你bc\r\n12345".as_bytes());
        let cell = f64::from(DEFAULT_CELL_WIDTH);
        let line = f64::from(DEFAULT_LINE_HEIGHT);

        widget.handle_event(&pointer_with_modifiers(
            PointerPhase::Down,
            cell * 2.25,
            line * 0.25,
            1,
            Modifiers::ALT,
        ));
        widget.handle_event(&pointer(PointerPhase::Move, cell * 3.75, line * 1.25, 1));
        widget.handle_event(&pointer(PointerPhase::Up, cell * 3.75, line * 1.25, 0));

        assert_eq!(widget.selected_text().as_deref(), Some("你b\n34"));
    }

    #[test]
    fn drag_gesture_does_not_seed_the_double_click_streak() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"hello world");
        let cell = f64::from(DEFAULT_CELL_WIDTH);

        widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Move, cell * 4.75, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Up, cell * 4.75, 1.0, 0));
        assert_eq!(widget.selected_text().as_deref(), Some("hello"));

        widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Up, cell * 0.25, 1.0, 0));
        assert_ne!(widget.selected_text().as_deref(), Some("hello"));
    }

    #[test]
    fn input_and_focus_loss_break_the_terminal_click_streak() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"hello world");
        let x = f64::from(DEFAULT_CELL_WIDTH) * 1.25;
        let click = |widget: &mut TerminalWidget| {
            widget.handle_event(&pointer(PointerPhase::Down, x, 1.0, 1));
            widget.handle_event(&pointer(PointerPhase::Up, x, 1.0, 0));
        };

        click(&mut widget);
        widget.handle_event(&UiEvent::TextInput("x".into()));
        click(&mut widget);
        assert_eq!(
            widget.terminal.lock().selection.as_ref().unwrap().ty,
            SelectionType::Simple,
            "keyboard input must terminate a pending multi-click sequence"
        );

        widget.focus_changed(false);
        widget.focus_changed(true);
        click(&mut widget);
        assert_eq!(
            widget.terminal.lock().selection.as_ref().unwrap().ty,
            SelectionType::Simple,
            "window focus sessions must not share a multi-click sequence"
        );

        let mut secondary = match pointer(PointerPhase::Down, x, 1.0, 4) {
            UiEvent::Pointer(pointer) => pointer,
            _ => unreachable!(),
        };
        secondary.button = Some(PointerButton::Secondary);
        widget.handle_event(&UiEvent::Pointer(secondary));
        click(&mut widget);
        assert_eq!(
            widget.terminal.lock().selection.as_ref().unwrap().ty,
            SelectionType::Simple,
            "another pointer button must break the primary click sequence"
        );
    }

    #[test]
    fn local_keyboard_shortcuts_break_the_terminal_click_streak() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"hello world");
        let x = f64::from(DEFAULT_CELL_WIDTH) * 1.25;
        let click = |widget: &mut TerminalWidget| {
            widget.handle_event(&pointer(PointerPhase::Down, x, 1.0, 1));
            widget.handle_event(&pointer(PointerPhase::Up, x, 1.0, 0));
        };
        let key = |name: &str, modifiers| {
            UiEvent::Key(KeyEvent {
                phase: KeyPhase::Down,
                key: name.into(),
                key_without_modifiers: name.into(),
                code: name.into(),
                text: None,
                text_with_all_modifiers: None,
                location: KeyLocation::Standard,
                modifiers,
                repeat: false,
            })
        };

        click(&mut widget);
        let copy_modifiers = if cfg!(target_os = "macos") {
            Modifiers::META
        } else {
            Modifiers::CONTROL | Modifiers::SHIFT
        };
        widget.handle_event(&key("c", copy_modifiers));
        click(&mut widget);
        assert_eq!(
            widget.terminal.lock().selection.as_ref().unwrap().ty,
            SelectionType::Simple,
            "a local clipboard shortcut must separate click gestures"
        );

        click(&mut widget);
        widget.handle_event(&key("PageUp", Modifiers::SHIFT));
        click(&mut widget);
        assert_eq!(
            widget.terminal.lock().selection.as_ref().unwrap().ty,
            SelectionType::Simple,
            "a local scrollback shortcut must separate click gestures"
        );
    }

    #[test]
    fn cancelled_terminal_click_does_not_seed_word_selection() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"hello world");
        let x = f64::from(DEFAULT_CELL_WIDTH) * 1.25;

        widget.handle_event(&pointer(PointerPhase::Down, x, 1.0, 1));
        let mut cancelled = match pointer(PointerPhase::Cancel, x, 1.0, 0) {
            UiEvent::Pointer(pointer) => pointer,
            _ => unreachable!(),
        };
        cancelled.button = None;
        widget.handle_event(&UiEvent::Pointer(cancelled));
        widget.handle_event(&pointer(PointerPhase::Down, x, 1.0, 1));

        assert_eq!(
            widget.terminal.lock().selection.as_ref().unwrap().ty,
            SelectionType::Simple
        );
    }

    #[test]
    fn terminal_clipboard_shortcuts_do_not_consume_control_process_input() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"selected text");
        widget.begin_selection(1.0, 1.0, false);
        widget.update_selection(f64::from(DEFAULT_CELL_WIDTH) * 7.75, 1.0);
        let key = |name: &str, modifiers| {
            UiEvent::Key(KeyEvent {
                phase: KeyPhase::Down,
                key: name.into(),
                key_without_modifiers: name.into(),
                code: format!("Key{}", name.to_ascii_uppercase()),
                text: None,
                text_with_all_modifiers: None,
                location: KeyLocation::Standard,
                modifiers,
                repeat: false,
            })
        };

        if cfg!(target_os = "macos") {
            let copy = widget.handle_event(&key("c", Modifiers::META));
            assert_eq!(
                copy.clipboard_request(),
                Some(&ClipboardRequest::Write("selected".into()))
            );
            let paste = widget.handle_event(&key("v", Modifiers::META));
            assert_eq!(paste.clipboard_request(), Some(&ClipboardRequest::Read));
        } else {
            let interrupt = widget.handle_event(&key("c", Modifiers::CONTROL));
            assert!(interrupt.is_handled());
            assert_eq!(interrupt.clipboard_request(), None);
            assert_eq!(widget.take_input(), [0x03]);
            assert!(widget.selected_text().is_none());

            widget.begin_selection(1.0, 1.0, false);
            widget.update_selection(f64::from(DEFAULT_CELL_WIDTH) * 7.75, 1.0);
            let copy = widget.handle_event(&key("c", Modifiers::CONTROL | Modifiers::SHIFT));
            assert_eq!(
                copy.clipboard_request(),
                Some(&ClipboardRequest::Write("selected".into()))
            );
            assert!(widget.take_input().is_empty());

            let quoted_insert = widget.handle_event(&key("v", Modifiers::CONTROL));
            assert!(quoted_insert.is_handled());
            assert_eq!(quoted_insert.clipboard_request(), None);
            assert_eq!(widget.take_input(), [0x16]);
            let paste = widget.handle_event(&key("v", Modifiers::CONTROL | Modifiers::SHIFT));
            assert_eq!(paste.clipboard_request(), Some(&ClipboardRequest::Read));
        }
    }

    #[test]
    fn terminal_select_all_covers_scrollback_and_publishes_a_line_selection() {
        let mut widget = TerminalWidget::headless(10, 3);
        widget.feed(b"one\r\ntwo\r\nthree\r\nfour\r\nfive");
        let modifiers = if cfg!(target_os = "macos") {
            Modifiers::META
        } else {
            Modifiers::CONTROL | Modifiers::SHIFT
        };
        let key = UiEvent::Key(KeyEvent {
            phase: KeyPhase::Down,
            key: "a".into(),
            key_without_modifiers: "a".into(),
            code: "KeyA".into(),
            text: None,
            text_with_all_modifiers: None,
            location: KeyLocation::Standard,
            modifiers,
            repeat: false,
        });

        let result = widget.handle_event(&key);

        assert!(result.is_handled());
        assert!(widget.terminal.lock().history_size() > 0);
        assert_eq!(
            widget.selected_text().as_deref(),
            Some("one\ntwo\nthree\nfour\nfive\n")
        );
        let event = widget.take_node_event().expect("select-all event");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&event.json).unwrap(),
            serde_json::json!({
                "text": "one\ntwo\nthree\nfour\nfive\n",
                "kind": "line"
            })
        );

        if !cfg!(target_os = "macos") {
            let interrupt = UiEvent::Key(KeyEvent {
                phase: KeyPhase::Down,
                key: "a".into(),
                key_without_modifiers: "a".into(),
                code: "KeyA".into(),
                text: None,
                text_with_all_modifiers: None,
                location: KeyLocation::Standard,
                modifiers: Modifiers::CONTROL,
                repeat: false,
            });
            widget.handle_event(&interrupt);
            assert_eq!(widget.take_input(), [0x01]);
            assert!(widget.selected_text().is_none());
        }
    }

    #[test]
    fn copy_shortcut_without_selection_never_reaches_the_pty() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"\x1b[>3u");
        let modifiers = if cfg!(target_os = "macos") {
            Modifiers::META
        } else {
            Modifiers::CONTROL | Modifiers::SHIFT
        };
        let mut key = KeyEvent {
            phase: KeyPhase::Down,
            key: "c".into(),
            key_without_modifiers: "c".into(),
            code: "KeyC".into(),
            text: None,
            text_with_all_modifiers: None,
            location: KeyLocation::Standard,
            modifiers,
            repeat: false,
        };
        let response = widget.handle_event(&UiEvent::Key(key.clone()));

        assert!(response.is_handled());
        assert_eq!(response.clipboard_request(), None);
        key.phase = KeyPhase::Up;
        assert!(widget.handle_event(&UiEvent::Key(key)).is_handled());
        assert!(widget.take_input().is_empty());
    }

    #[test]
    fn selecting_the_trailing_cell_copies_the_complete_wide_glyph() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed("你a".as_bytes());
        let y = 1.0;
        let left = f64::from(DEFAULT_CELL_WIDTH) * 1.25;
        let right = f64::from(DEFAULT_CELL_WIDTH) * 1.75;

        widget.handle_event(&pointer(PointerPhase::Down, left, y, 1));
        widget.handle_event(&pointer(PointerPhase::Up, right, y, 0));

        assert_eq!(widget.selected_text().as_deref(), Some("你"));
    }

    #[test]
    fn dragging_selection_outside_viewport_scrolls_history() {
        let mut widget = TerminalWidget::headless(20, 2);
        widget.feed(b"one\r\ntwo\r\nthree\r\nfour");
        assert_eq!(widget.terminal.lock().display_offset(), 0);

        widget.handle_event(&pointer(PointerPhase::Down, 1.0, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Move, 1.0, -5.0, 1));

        assert_eq!(widget.terminal.lock().display_offset(), 1);

        widget.next_selection_scroll = Some(Instant::now() - Duration::from_millis(1));
        let mut tcx = TextContext::new();
        widget.paint(
            DEFAULT_CELL_WIDTH * 20.0,
            DEFAULT_LINE_HEIGHT * 2.0,
            &mut tcx,
        );
        assert_eq!(widget.terminal.lock().display_offset(), 2);
    }

    #[test]
    fn selection_autoscroll_accelerates_far_outside_the_viewport() {
        let mut widget = TerminalWidget::headless(20, 2);
        for index in 0..20 {
            widget.feed(format!("line {index}\r\n").as_bytes());
        }
        widget.handle_event(&pointer(PointerPhase::Down, 1.0, 1.0, 1));

        widget.handle_event(&pointer(
            PointerPhase::Move,
            1.0,
            -f64::from(DEFAULT_LINE_HEIGHT) * 4.5,
            1,
        ));

        assert_eq!(widget.terminal.lock().display_offset(), 5);
    }

    #[test]
    fn selection_autoscroll_stops_at_scrollback_boundaries() {
        let mut widget = TerminalWidget::headless(20, 2);
        widget.feed(b"one\r\ntwo");
        widget.handle_event(&pointer(PointerPhase::Down, 1.0, 1.0, 1));

        widget.handle_event(&pointer(PointerPhase::Move, 1.0, -5.0, 1));
        assert_eq!(widget.terminal.lock().display_offset(), 0);
        assert!(widget.next_selection_scroll.is_none());

        widget.handle_event(&pointer(
            PointerPhase::Move,
            1.0,
            f64::from(DEFAULT_LINE_HEIGHT) * 3.0,
            1,
        ));
        assert_eq!(widget.terminal.lock().display_offset(), 0);
        assert!(widget.next_selection_scroll.is_none());
    }

    #[test]
    fn smooth_wheel_deltas_accumulate_into_terminal_lines() {
        let mut widget = TerminalWidget::headless(20, 2);
        widget.feed(b"one\r\ntwo\r\nthree\r\nfour");
        let wheel = |delta_y| {
            UiEvent::Wheel(wabou_shell::WheelEvent {
                position: wabou_shell::Point { x: 1.0, y: 1.0 },
                delta_x: 0.0,
                delta_y,
                modifiers: Modifiers::default(),
            })
        };

        for _ in 0..3 {
            assert!(widget.handle_event(&wheel(10.0)).is_handled());
            assert_eq!(widget.terminal.lock().display_offset(), 0);
        }
        widget.handle_event(&wheel(10.0));
        assert_eq!(widget.terminal.lock().display_offset(), 1);

        // A new gesture in the opposite direction is responsive instead of
        // first consuming a stale same-direction remainder.
        widget.handle_event(&wheel(30.0));
        widget.handle_event(&wheel(-40.0));
        assert_eq!(widget.terminal.lock().display_offset(), 0);
    }

    #[test]
    fn wheel_remainders_do_not_leak_between_terminal_input_owners() {
        let mut widget = TerminalWidget::headless(20, 2);
        widget.feed(b"one\r\ntwo\r\nthree\r\nfour");
        let wheel = |delta_y| {
            UiEvent::Wheel(wabou_shell::WheelEvent {
                position: wabou_shell::Point { x: 1.0, y: 1.0 },
                delta_x: 0.0,
                delta_y,
                modifiers: Modifiers::default(),
            })
        };

        widget.handle_event(&wheel(30.0));
        assert_eq!(widget.terminal.lock().display_offset(), 0);

        widget.feed(b"\x1b[?1000h\x1b[?1006h");
        widget.handle_event(&wheel(10.0));
        widget.handle_event(&wheel(20.0));
        assert!(widget.take_input().is_empty());

        widget.feed(b"\x1b[?1000l\x1b[?1006l");
        widget.handle_event(&wheel(10.0));
        assert_eq!(widget.terminal.lock().display_offset(), 0);
        widget.handle_event(&wheel(30.0));
        assert_eq!(widget.terminal.lock().display_offset(), 1);
        assert!(widget.take_input().is_empty());
    }

    #[test]
    fn selection_tracks_content_as_new_output_scrolls_the_grid() {
        let mut widget = TerminalWidget::headless(20, 3);
        widget.feed(b"one\r\ntwo\r\nthree");
        let cell = f64::from(DEFAULT_CELL_WIDTH);
        let row = f64::from(DEFAULT_LINE_HEIGHT) * 1.25;
        widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, row, 1));
        widget.handle_event(&pointer(PointerPhase::Move, cell * 2.75, row, 1));
        widget.handle_event(&pointer(PointerPhase::Up, cell * 2.75, row, 0));
        assert_eq!(widget.selected_text().as_deref(), Some("two"));

        widget.feed(b"\r\nfour");

        assert_eq!(widget.selected_text().as_deref(), Some("two"));
        assert!(widget.visible_line(0).starts_with("two"));
    }

    #[test]
    fn selection_survives_vertical_resize_without_reflow() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"one\r\ntwo\r\nthree");
        let cell = f64::from(DEFAULT_CELL_WIDTH);
        let row = f64::from(DEFAULT_LINE_HEIGHT) * 1.25;
        widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, row, 1));
        widget.handle_event(&pointer(PointerPhase::Move, cell * 2.75, row, 1));
        widget.handle_event(&pointer(PointerPhase::Up, cell * 2.75, row, 0));

        widget.resize(DEFAULT_CELL_WIDTH * 20.0, DEFAULT_LINE_HEIGHT * 2.0);

        assert_eq!(widget.selected_text().as_deref(), Some("two"));
    }

    #[test]
    fn scale_aware_paint_tracks_retina_density() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed("🚀".as_bytes());
        let mut tcx = TextContext::new();

        widget.paint_scaled(200.0, 80.0, 2.0, &mut tcx);

        assert_eq!(widget.device_scale, 2.0);
    }

    #[test]
    fn cell_backgrounds_share_device_pixel_aligned_edges() {
        let scale = 1.5;
        let first = cell_fill_rect(0, 0, 8.3, 19.7, scale);
        let second = cell_fill_rect(1, 0, 8.3, 19.7, scale);
        let next_row = cell_fill_rect(0, 1, 8.3, 19.7, scale);

        assert_eq!(first.x1, second.x0);
        assert_eq!(first.y1, next_row.y0);
        for edge in [first.x0, first.y0, first.x1, first.y1] {
            assert_eq!(edge * scale, (edge * scale).round());
        }
    }

    #[test]
    fn double_click_selects_semantic_word() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"hello world");
        let x = f64::from(DEFAULT_CELL_WIDTH) * 7.25;

        for _ in 0..2 {
            widget.handle_event(&pointer(PointerPhase::Down, x, 1.0, 1));
            widget.handle_event(&pointer(PointerPhase::Up, x, 1.0, 0));
        }

        assert_eq!(widget.selected_text().as_deref(), Some("world"));
    }

    #[test]
    fn sgr_mouse_mode_reports_to_pty_and_shift_bypasses_it() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"\x1b[?1000h\x1b[?1006h\r\nabcdef");
        let x = f64::from(DEFAULT_CELL_WIDTH) * 2.25;
        let y = f64::from(DEFAULT_LINE_HEIGHT) * 1.25;

        widget.handle_event(&pointer(PointerPhase::Down, x, y, 1));
        widget.handle_event(&pointer(PointerPhase::Up, x, y, 0));
        assert_eq!(widget.take_input(), b"\x1b[<0;3;2M\x1b[<0;3;2m");

        let mut shifted = match pointer(PointerPhase::Down, x, y, 1) {
            UiEvent::Pointer(pointer) => pointer,
            _ => unreachable!(),
        };
        shifted.modifiers.insert(Modifiers::SHIFT);
        widget.handle_event(&UiEvent::Pointer(shifted));
        assert!(widget.take_input().is_empty());
        assert!(widget.selecting);

        // Once local selection owns the gesture, releasing Shift must not
        // hand the remaining move/up back to the terminal application.
        widget.handle_event(&pointer(
            PointerPhase::Move,
            f64::from(DEFAULT_CELL_WIDTH) * 5.75,
            y,
            1,
        ));
        widget.handle_event(&pointer(
            PointerPhase::Up,
            f64::from(DEFAULT_CELL_WIDTH) * 5.75,
            y,
            0,
        ));
        assert!(widget.take_input().is_empty());
        assert!(!widget.selecting);
        assert_eq!(widget.selected_text().as_deref(), Some("cdef"));
    }

    #[test]
    fn pointer_cancel_releases_remote_mouse_buttons() {
        let x = f64::from(DEFAULT_CELL_WIDTH) * 2.25;
        let y = f64::from(DEFAULT_LINE_HEIGHT) * 1.25;
        let mut cancelled = match pointer(PointerPhase::Cancel, x, y, 0) {
            UiEvent::Pointer(pointer) => pointer,
            _ => unreachable!(),
        };
        cancelled.button = None;

        let mut sgr = TerminalWidget::headless(20, 4);
        sgr.feed(b"\x1b[?1000h\x1b[?1006h");
        sgr.handle_event(&pointer(PointerPhase::Down, x, y, 1));
        sgr.handle_event(&UiEvent::Pointer(cancelled.clone()));
        assert_eq!(sgr.take_input(), b"\x1b[<0;3;2M\x1b[<0;3;2m");

        let mut legacy = TerminalWidget::headless(20, 4);
        legacy.feed(b"\x1b[?1000h");
        legacy.handle_event(&pointer(PointerPhase::Down, x, y, 1));
        legacy.handle_event(&UiEvent::Pointer(cancelled));
        assert_eq!(legacy.take_input(), b"\x1b[M #\"\x1b[M##\"");
    }

    #[test]
    fn remote_mouse_gesture_keeps_ownership_and_button_identity() {
        let x = f64::from(DEFAULT_CELL_WIDTH) * 2.25;
        let y = f64::from(DEFAULT_LINE_HEIGHT) * 1.25;
        let mut down = match pointer(PointerPhase::Down, x, y, 4) {
            UiEvent::Pointer(pointer) => pointer,
            _ => unreachable!(),
        };
        down.button = Some(PointerButton::Secondary);
        let mut cancelled =
            match pointer_with_modifiers(PointerPhase::Cancel, x, y, 0, Modifiers::SHIFT) {
                UiEvent::Pointer(pointer) => pointer,
                _ => unreachable!(),
            };
        cancelled.button = None;

        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"\x1b[?1000h\x1b[?1006h");
        widget.handle_event(&UiEvent::Pointer(down));
        assert_eq!(widget.remote_mouse_button, Some(PointerButton::Secondary));
        widget.handle_event(&UiEvent::Pointer(cancelled));

        assert_eq!(widget.take_input(), b"\x1b[<2;3;2M\x1b[<6;3;2m");
        assert_eq!(widget.remote_mouse_button, None);
        assert!(widget.terminal.lock().selection.is_none());
    }

    #[test]
    fn focus_loss_finishes_an_in_progress_selection_gesture() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"abcdef");
        let cell = f64::from(DEFAULT_CELL_WIDTH);
        widget.handle_event(&pointer(PointerPhase::Down, cell * 0.25, 1.0, 1));
        widget.handle_event(&pointer(PointerPhase::Move, cell * 3.75, 1.0, 1));
        assert!(widget.selecting);
        assert!(widget.take_node_event().is_none());

        widget.focus_changed(false);

        assert!(!widget.selecting);
        assert!(widget.last_click.is_none());
        assert_eq!(widget.selected_text().as_deref(), Some("abcd"));
        let node_event = widget.take_node_event().expect("committed selection");
        assert_eq!(node_event.event_code, event::TERMINALSELECTIONCHANGE);
    }

    #[test]
    fn active_selection_owns_wheel_input_in_mouse_reporting_mode() {
        let mut widget = TerminalWidget::headless(20, 2);
        widget.feed(b"one\r\ntwo\r\nthree\r\nfour\x1b[?1000h\x1b[?1006h");
        let position = wabou_shell::Point {
            x: f64::from(DEFAULT_CELL_WIDTH) * 3.75,
            y: f64::from(DEFAULT_LINE_HEIGHT) * 1.25,
        };
        widget.handle_event(&pointer_with_modifiers(
            PointerPhase::Down,
            position.x,
            position.y,
            1,
            Modifiers::SHIFT,
        ));
        assert!(widget.selecting);

        widget.handle_event(&UiEvent::Wheel(wabou_shell::WheelEvent {
            position,
            delta_x: 0.0,
            delta_y: 40.0,
            modifiers: Modifiers::empty(),
        }));

        assert!(widget.terminal.lock().display_offset() > 0);
        assert!(widget.take_input().is_empty());
        assert!(widget.selecting);
        widget.handle_event(&pointer(PointerPhase::Up, position.x, position.y, 0));
        assert!(!widget.selecting);
        assert!(widget.selected_text().is_some());
    }

    #[test]
    fn mouse_reporting_sends_wheel_buttons_at_pointer_position() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"\x1b[?1000h\x1b[?1006h");
        let position = wabou_shell::Point {
            x: f64::from(DEFAULT_CELL_WIDTH) * 2.25,
            y: f64::from(DEFAULT_LINE_HEIGHT) * 1.25,
        };

        for delta_y in [-40.0, 40.0] {
            widget.handle_event(&UiEvent::Wheel(wabou_shell::WheelEvent {
                position,
                delta_x: 0.0,
                delta_y,
                modifiers: Modifiers::default(),
            }));
        }

        assert_eq!(widget.take_input(), b"\x1b[<64;3;2M\x1b[<65;3;2M");
        widget.handle_event(&UiEvent::Wheel(wabou_shell::WheelEvent {
            position,
            delta_x: 0.0,
            delta_y: -40.0,
            modifiers: Modifiers::ALT | Modifiers::CONTROL,
        }));
        assert_eq!(widget.take_input(), b"\x1b[<88;3;2M");
        assert_eq!(widget.terminal.lock().display_offset(), 0);
    }

    #[test]
    fn utf8_mouse_encodes_large_coordinates_without_legacy_clamping() {
        let mut widget = TerminalWidget::headless(300, 4);
        widget.feed(b"\x1b[?1000h\x1b[?1005h");
        let x = f64::from(DEFAULT_CELL_WIDTH) * 99.25;

        widget.handle_event(&pointer(PointerPhase::Down, x, 1.0, 1));

        assert_eq!(
            widget.take_input(),
            [b"\x1b[M ".as_slice(), &[0xc2, 0x84, 33]].concat()
        );

        let mut legacy = TerminalWidget::headless(300, 4);
        legacy.feed(b"\x1b[?1000h");
        let outside_legacy_range = f64::from(DEFAULT_CELL_WIDTH) * 223.25;
        let result =
            legacy.handle_event(&pointer(PointerPhase::Down, outside_legacy_range, 1.0, 1));
        assert!(result.is_handled());
        assert!(legacy.take_input().is_empty());
        assert!(!legacy.selecting);
    }

    #[test]
    fn focus_and_alternate_scroll_modes_report_to_pty() {
        let mut widget = TerminalWidget::headless(20, 4);
        widget.feed(b"\x1b[?1004h");
        widget.focus_changed(true);
        widget.focus_changed(false);
        assert_eq!(widget.take_input(), b"\x1b[I\x1b[O");

        widget.feed(b"\x1b[?1049h\x1b[?1007h");
        widget.handle_event(&UiEvent::Wheel(wabou_shell::WheelEvent {
            position: wabou_shell::Point { x: 0.0, y: 0.0 },
            delta_x: 0.0,
            delta_y: -48.0,
            modifiers: Modifiers::default(),
        }));
        assert_eq!(widget.take_input(), b"\x1b[A");
    }
}
