//! Window mode: a winit event loop driving a [`Shell`] from a [`FrameSource`].

use snafu::ResultExt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use vello::peniko::Color;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ButtonSource, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, KeyLocation as WinitKeyLocation, ModifiersState};
use winit::window::{
    ImeCapabilities, ImeEnableRequest, ImeHint, ImePurpose, ImeRequest, ImeRequestData,
    UserAttentionType, WindowId,
};

use crate::scene as scene_builder;
use crate::shell::Shell;
use crate::source::{
    ClipboardRequest, EventResponse, FrameSource, FrameStats, HostAction, HostActionResult,
    KeyEvent, KeyLocation, KeyPhase, Modifiers, Point, PointerButton, PointerEvent, PointerPhase,
    UiEvent, WakeCallback, WheelEvent, WindowCommand, WindowMetrics, WindowOptions,
};

fn ms(d: std::time::Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn allowed_external_url(raw: &str) -> Option<url::Url> {
    let url = url::Url::parse(raw).ok()?;
    matches!(url.scheme(), "http" | "https").then_some(url)
}

fn update_present_retry(presented: bool, retry_pending: &mut bool) -> bool {
    if presented {
        *retry_pending = false;
        false
    } else if !*retry_pending {
        *retry_pending = true;
        true
    } else {
        false
    }
}

/// The winit application. Owns a [`FrameSource`] (the frame producer) and a
/// lazily-created [`Shell`] (window + GPU surface + renderer). Created in
/// `can_create_surfaces` once the event loop is ready.
pub struct App {
    logical_window_id: u64,
    source: Box<dyn FrameSource>,
    state: Option<Shell>,
    window_options: WindowOptions,
    window_metrics: WindowMetrics,
    modifiers: Modifiers,
    pointer_buttons: u32,
    pointer_position: Point,
    ime_enabled: bool,
    startup_error: Arc<Mutex<Option<crate::Error>>>,
    /// EMA per-frame stage timings, reported to the source for a perf overlay.
    frame_stats: FrameStats,
    /// A configure can make the first acquire transiently fail. Static apps do
    /// not otherwise produce another frame, so allow exactly one recovery
    /// redraw without spinning forever while a window is occluded.
    present_retry_pending: bool,
    clipboard: Option<arboard::Clipboard>,
    pending_windows: Vec<(u64, WindowOptions)>,
    pending_window_commands: Vec<(u64, WindowCommand)>,
    close_requested: bool,
}

impl App {
    fn unconsumed_key_text(text: Option<String>, response: &EventResponse) -> Option<String> {
        text.filter(|_| !response.consume_key_text)
    }

    fn dispatch_focus_change(&mut self, focused: bool) {
        if !focused {
            if self.pointer_buttons != 0 {
                self.pointer_buttons = 0;
                self.dispatch_event(UiEvent::Pointer(PointerEvent {
                    phase: PointerPhase::Cancel,
                    position: self.pointer_position,
                    button: None,
                    buttons: 0,
                    modifiers: self.modifiers,
                }));
            }
            // Some platforms do not send matching key/modifier releases after
            // deactivation. Never carry those physical states into the next
            // focus session.
            self.modifiers = Modifiers::default();
        }
        self.dispatch_event(UiEvent::Focus(focused));
    }

    pub fn new(source: Box<dyn FrameSource>) -> Self {
        Self::with_options(source, WindowOptions::default())
    }

    pub fn with_size(source: Box<dyn FrameSource>, width: u32, height: u32) -> Self {
        Self::with_options(
            source,
            WindowOptions::new().initial_inner_size(width, height),
        )
    }

    pub fn with_options(source: Box<dyn FrameSource>, window_options: WindowOptions) -> Self {
        Self {
            logical_window_id: 1,
            source,
            state: None,
            window_options,
            window_metrics: WindowMetrics::default(),
            modifiers: Modifiers::default(),
            pointer_buttons: 0,
            pointer_position: Point { x: 0.0, y: 0.0 },
            ime_enabled: false,
            startup_error: Arc::new(Mutex::new(None)),
            frame_stats: FrameStats::default(),
            present_retry_pending: false,
            clipboard: None,
            pending_windows: Vec::new(),
            pending_window_commands: Vec::new(),
            close_requested: false,
        }
    }

    fn sync_window_metrics(&mut self) {
        let Some(shell) = self.state.as_ref() else {
            return;
        };
        let (physical_width, physical_height) = shell.size();
        let (logical_width, logical_height) = shell.logical_size();
        let next = WindowMetrics {
            window_id: self.logical_window_id,
            logical_width,
            logical_height,
            physical_width,
            physical_height,
            scale_factor: shell.scale_factor(),
            maximized: shell.window().is_maximized(),
            focused: shell.window().has_focus(),
        };
        if next != self.window_metrics {
            self.window_metrics = next;
            self.dispatch_event(UiEvent::WindowMetrics(next));
        }
    }

    /// Build one frame into `shell.scene` from the source. Returns
    /// `(node_count, build_frame_ms, scene_ms)` so the caller can fold the
    /// timings (plus its own present timing) into the EMA perf stats.
    fn build(
        shell: &mut Shell,
        source: &mut dyn FrameSource,
        base_color: Color,
    ) -> (usize, f64, f64) {
        let (w, h) = shell.logical_size();
        let scale = shell.scale_factor();
        source.set_device_scale(scale);
        let t0 = Instant::now();
        let nodes = source.build_frame(&mut shell.tcx, w, h);
        let t1 = Instant::now();
        scene_builder::build_scene_scaled(
            &mut shell.scene,
            &nodes,
            &mut shell.tcx,
            w,
            h,
            base_color,
            scale,
        );
        source.paint_debug_overlay(&mut shell.scene, &nodes, &mut shell.tcx, scale);
        let t2 = Instant::now();
        (nodes.len(), ms(t1 - t0), ms(t2 - t1))
    }

    fn pointer_button(button: &ButtonSource) -> PointerButton {
        match button {
            ButtonSource::Mouse(MouseButton::Left) => PointerButton::Primary,
            ButtonSource::Mouse(MouseButton::Middle) => PointerButton::Auxiliary,
            ButtonSource::Mouse(MouseButton::Right) => PointerButton::Secondary,
            ButtonSource::Mouse(MouseButton::Back) => PointerButton::Other(3),
            ButtonSource::Mouse(MouseButton::Forward) => PointerButton::Other(4),
            _ => PointerButton::Primary,
        }
    }

    fn button_mask(button: PointerButton) -> u32 {
        match button {
            PointerButton::Primary => 1,
            PointerButton::Auxiliary => 2,
            PointerButton::Secondary => 4,
            PointerButton::Other(index) if index < u32::BITS as u16 => 1 << index,
            PointerButton::Other(_) => 0,
        }
    }

    fn modifiers(state: ModifiersState) -> Modifiers {
        let mut modifiers = Modifiers::empty();
        modifiers.set(Modifiers::SHIFT, state.shift_key());
        modifiers.set(Modifiers::CONTROL, state.control_key());
        modifiers.set(Modifiers::ALT, state.alt_key());
        modifiers.set(Modifiers::META, state.meta_key());
        modifiers
    }

    fn logical_key_name(key: &Key) -> String {
        match key {
            Key::Named(named) => format!("{named:?}"),
            other => other
                .to_text()
                .map_or_else(|| format!("{other:?}"), str::to_owned),
        }
    }

    fn key_location(location: WinitKeyLocation) -> KeyLocation {
        match location {
            WinitKeyLocation::Standard => KeyLocation::Standard,
            WinitKeyLocation::Left => KeyLocation::Left,
            WinitKeyLocation::Right => KeyLocation::Right,
            WinitKeyLocation::Numpad => KeyLocation::Numpad,
        }
    }

    fn printable_key_text(text: &str) -> Option<String> {
        let text: String = text
            .chars()
            .filter(|character| !character.is_control())
            .collect();
        (!text.is_empty()).then_some(text)
    }

    fn committed_key_text(
        text: Option<&str>,
        text_with_all_modifiers: Option<&str>,
        modifiers: Modifiers,
    ) -> Option<String> {
        if modifiers.control() && modifiers.alt() && !modifiers.meta() {
            text_with_all_modifiers.and_then(Self::printable_key_text)
        } else if !modifiers.control() && !modifiers.meta() {
            text.and_then(Self::printable_key_text)
        } else {
            None
        }
    }

    fn dispatch_event(&mut self, event: UiEvent) -> EventResponse {
        let mut response = self.source.handle_event(event);
        if let Some(request) = response.clipboard.take() {
            if self.clipboard.is_none() {
                self.clipboard = arboard::Clipboard::new().ok();
            }
            match request {
                ClipboardRequest::Write(text) => {
                    if let Some(clipboard) = self.clipboard.as_mut() {
                        let _ = clipboard.set_text(text);
                    }
                }
                ClipboardRequest::Read => {
                    let text = self
                        .clipboard
                        .as_mut()
                        .and_then(|clipboard| clipboard.get_text().ok());
                    if let Some(text) = text.filter(|text| !text.is_empty()) {
                        let pasted = self.source.handle_event(UiEvent::Paste(text));
                        response.handled |= pasted.handled;
                        response.request_redraw |= pasted.request_redraw;
                        response.consume_key_text |= pasted.consume_key_text;
                    }
                }
            }
        }
        let host_action = self.drain_host_actions();
        response.handled |= host_action;
        response.request_redraw |= host_action;
        if let Some(allowed) = response.text_input
            && allowed != self.ime_enabled
            && let Some(shell) = self.state.as_ref()
        {
            let request = if allowed {
                let capabilities = ImeCapabilities::new().with_hint_and_purpose();
                let data = ImeRequestData::default()
                    .with_hint_and_purpose(ImeHint::NONE, ImePurpose::Normal);
                ImeRequest::Enable(
                    ImeEnableRequest::new(capabilities, data)
                        .expect("IME capabilities and initial data must match"),
                )
            } else {
                ImeRequest::Disable
            };
            if shell.window().request_ime_update(request).is_ok() {
                self.ime_enabled = allowed;
            }
        }
        if response.request_redraw
            && let Some(shell) = self.state.as_ref()
        {
            shell.window().request_redraw();
        }
        response
    }

    fn drain_host_actions(&mut self) -> bool {
        let mut handled = false;
        while let Some(action) = self.source.take_host_action() {
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
                    if self.clipboard.is_none() {
                        self.clipboard = arboard::Clipboard::new().ok();
                    }
                    if let Some(clipboard) = self.clipboard.as_mut() {
                        let _ = clipboard.set_text(text);
                    }
                }
                HostAction::WriteClipboard { request_id, text } => {
                    if self.clipboard.is_none() {
                        self.clipboard = arboard::Clipboard::new().ok();
                    }
                    let success = self
                        .clipboard
                        .as_mut()
                        .is_some_and(|clipboard| clipboard.set_text(text).is_ok());
                    self.source
                        .complete_host_action(HostActionResult::ClipboardWrite {
                            request_id,
                            success,
                        });
                }
                HostAction::ReadClipboard { request_id } => {
                    if self.clipboard.is_none() {
                        self.clipboard = arboard::Clipboard::new().ok();
                    }
                    let text = self
                        .clipboard
                        .as_mut()
                        .and_then(|clipboard| clipboard.get_text().ok());
                    self.source
                        .complete_host_action(HostActionResult::Clipboard { request_id, text });
                }
                HostAction::SetWindowTitle(title) => {
                    if let Some(shell) = self.state.as_ref() {
                        shell
                            .window()
                            .set_title(title.as_deref().unwrap_or(&self.window_options.title));
                    }
                }
                HostAction::RequestAttention => {
                    if let Some(shell) = self.state.as_ref() {
                        shell
                            .window()
                            .request_user_attention(Some(UserAttentionType::Informational));
                    }
                }
                HostAction::CreateWindow { window_id, options } => {
                    self.pending_windows.push((window_id, options));
                }
                HostAction::ControlWindow { window_id, command } => {
                    self.pending_window_commands.push((window_id, command));
                }
            }
        }
        handled
    }
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        // Initial size: prefer the source's notion of viewport if it exposes one;
        // otherwise default. For now we read none from the source, so use a sane
        // default that the first resize will correct.
        let shell = match Shell::create(event_loop, &self.window_options) {
            Ok(shell) => shell,
            Err(error) => {
                *self
                    .startup_error
                    .lock()
                    .expect("startup error mutex poisoned") = Some(error);
                event_loop.exit();
                return;
            }
        };
        let window = shell.window().clone();
        self.state = Some(shell);
        self.sync_window_metrics();
        window.request_redraw();
    }

    fn destroy_surfaces(&mut self, _event_loop: &dyn ActiveEventLoop) {
        self.state = None;
    }

    fn proxy_wake_up(&mut self, _event_loop: &dyn ActiveEventLoop) {
        let changed = self.source.poll_async();
        let host_action = self.drain_host_actions();
        if (changed || host_action)
            && let Some(shell) = self.state.as_ref()
        {
            shell.window().request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(shell) = self.state.as_mut() {
            shell
                .accessibility
                .process_window_event(shell.window.as_ref(), &event);
        }
        match event {
            WindowEvent::CloseRequested => {
                self.state = None;
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let Some(shell) = self.state.as_mut() else {
                    return;
                };
                let semantics_enabled = shell.accessibility.prepare_frame();
                self.source.set_semantics_enabled(semantics_enabled);
                for action in shell.accessibility.take_actions() {
                    self.source.handle_semantic_action(action);
                }
                let base_color = self.source.base_color();
                let (node_count, build_frame_ms, scene_ms) =
                    Self::build(shell, self.source.as_mut(), base_color);
                shell
                    .accessibility
                    .set_snapshot(self.source.semantic_snapshot());
                if let Some(path) = self.source.take_screenshot_request() {
                    let (width, height) = shell.size();
                    let result = crate::renderer::render_to_png(
                        &shell.scene,
                        width,
                        height,
                        base_color,
                        &path.to_string_lossy(),
                    )
                    .map(|_| path.clone())
                    .map_err(|error| error.to_string());
                    self.source.complete_screenshot(result);
                }
                let t2 = Instant::now();
                let presented = shell.present(base_color);
                let present_ms = ms(Instant::now() - t2);
                self.frame_stats
                    .update(build_frame_ms, scene_ms, present_ms, node_count);
                self.source.push_frame_stats(&self.frame_stats);
                if update_present_retry(presented, &mut self.present_retry_pending) {
                    shell.window().request_redraw();
                }
            }
            WindowEvent::SurfaceResized(PhysicalSize { width, height }) => {
                let Some(shell) = self.state.as_mut() else {
                    return;
                };
                // Match Blitz's native path: configure immediately. Only its
                // WASM backend debounces resize because configuring clears the
                // browser canvas. Ignore zero-sized/minimized surfaces.
                shell.resize(width, height);
                self.present_retry_pending = false;
                self.sync_window_metrics();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                self.sync_window_metrics();
                if let Some(shell) = self.state.as_ref() {
                    shell.window().request_redraw();
                }
            }
            WindowEvent::Occluded(false) => {
                self.present_retry_pending = false;
                if let Some(shell) = self.state.as_ref() {
                    shell.window().request_redraw();
                }
            }
            WindowEvent::PointerMoved { position, .. } => {
                let scale = self
                    .state
                    .as_ref()
                    .map_or(1.0, |shell| shell.scale_factor());
                self.pointer_position = Point {
                    x: position.x / scale,
                    y: position.y / scale,
                };
                self.dispatch_event(UiEvent::Pointer(PointerEvent {
                    phase: PointerPhase::Move,
                    position: self.pointer_position,
                    button: None,
                    buttons: self.pointer_buttons,
                    modifiers: self.modifiers,
                }));
            }
            WindowEvent::PointerButton {
                position,
                button,
                state,
                ..
            } => {
                let scale = self
                    .state
                    .as_ref()
                    .map_or(1.0, |shell| shell.scale_factor());
                let button = Self::pointer_button(&button);
                self.pointer_position = Point {
                    x: position.x / scale,
                    y: position.y / scale,
                };
                let phase = match state {
                    ElementState::Pressed => {
                        self.pointer_buttons |= Self::button_mask(button);
                        PointerPhase::Down
                    }
                    ElementState::Released => {
                        self.pointer_buttons &= !Self::button_mask(button);
                        PointerPhase::Up
                    }
                };
                self.dispatch_event(UiEvent::Pointer(PointerEvent {
                    phase,
                    position: self.pointer_position,
                    button: Some(button),
                    buttons: self.pointer_buttons,
                    modifiers: self.modifiers,
                }));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (delta_x, delta_y) = match delta {
                    // Winit reports the direction content should move; the
                    // DOM/Wabou event reports the scroll-position delta.
                    MouseScrollDelta::LineDelta(x, y) => (
                        -x as f64 * crate::WHEEL_LINE_DELTA,
                        -y as f64 * crate::WHEEL_LINE_DELTA,
                    ),
                    MouseScrollDelta::PixelDelta(p) => {
                        let scale = self
                            .state
                            .as_ref()
                            .map_or(1.0, |shell| shell.scale_factor());
                        (-p.x / scale, -p.y / scale)
                    }
                };
                // Winit's wheel event has no position, so expose the latest
                // logical pointer position as a stable framework invariant.
                self.dispatch_event(UiEvent::Wheel(WheelEvent {
                    position: self.pointer_position,
                    delta_x,
                    delta_y,
                    modifiers: self.modifiers,
                }));
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let phase = match event.state {
                    ElementState::Pressed => KeyPhase::Down,
                    ElementState::Released => KeyPhase::Up,
                };
                // Winit reports ordinary composed text on KeyboardInput. IME
                // composition uses the separate Ime::Commit path below.
                let text = (event.state == ElementState::Pressed)
                    .then(|| {
                        Self::committed_key_text(
                            event.text.as_deref(),
                            event.text_with_all_modifiers.as_deref(),
                            self.modifiers,
                        )
                    })
                    .flatten();
                let response = self.dispatch_event(UiEvent::Key(KeyEvent {
                    phase,
                    key: Self::logical_key_name(&event.logical_key),
                    key_without_modifiers: Self::logical_key_name(&event.key_without_modifiers),
                    code: format!("{:?}", event.physical_key),
                    text: event.text.as_deref().map(str::to_owned),
                    text_with_all_modifiers: event
                        .text_with_all_modifiers
                        .as_deref()
                        .map(str::to_owned),
                    location: Self::key_location(event.location),
                    modifiers: self.modifiers,
                    repeat: event.repeat,
                }));
                if let Some(text) = Self::unconsumed_key_text(text, &response) {
                    self.dispatch_event(UiEvent::TextInput(text));
                }
            }
            WindowEvent::ModifiersChanged(state) => {
                self.modifiers = Self::modifiers(state.state());
            }
            WindowEvent::Ime(winit::event::Ime::Commit(text)) => {
                self.dispatch_event(UiEvent::TextInput(text));
            }
            WindowEvent::Focused(focused) => {
                self.dispatch_focus_change(focused);
                self.sync_window_metrics();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        // A reactive source keeps the redraw loop spinning at vsync while it has
        // pending rAF work; a static source idles (ControlFlow::Wait) until a
        // resize/close.
        if self.source.has_anim() {
            event_loop.set_control_flow(ControlFlow::Wait);
            if let Some(shell) = self.state.as_ref() {
                shell.window().request_redraw();
            }
        } else if let Some(deadline) = self.source.animation_deadline() {
            if deadline <= std::time::Instant::now() {
                event_loop.set_control_flow(ControlFlow::Wait);
                if let Some(shell) = self.state.as_ref() {
                    shell.window().request_redraw();
                }
            } else {
                event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
            }
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}

/// One winit event loop routing events to independent native windows.
/// Each window owns its own frame source, input state, surface and renderer.
struct MultiWindowApp {
    pending: Vec<App>,
    windows: HashMap<WindowId, App>,
    startup_errors: Vec<Arc<Mutex<Option<crate::Error>>>>,
    factory: Option<FrameSourceFactory>,
    wake: WakeCallback,
}

pub type FrameSourceFactory =
    Arc<dyn Fn(u64, &WindowOptions) -> Result<Box<dyn FrameSource>, String>>;

fn find_window_by_logical_id<'a>(
    windows: impl Iterator<Item = &'a mut App>,
    window_id: u64,
) -> Option<&'a mut App> {
    windows
        .into_iter()
        .find(|app| app.logical_window_id == window_id)
}

fn apply_window_command(app: &mut App, command: WindowCommand) {
    match command {
        WindowCommand::Close => app.close_requested = true,
        WindowCommand::SetMaximized(maximized) => {
            if let Some(shell) = app.state.as_ref() {
                shell.window().set_maximized(maximized);
            }
        }
        WindowCommand::SetTitle(title) => {
            if let Some(shell) = app.state.as_ref() {
                shell.window().set_title(&title);
            }
        }
    }
}

impl MultiWindowApp {
    fn new(apps: Vec<App>, factory: Option<FrameSourceFactory>, wake: WakeCallback) -> Self {
        let startup_errors = apps.iter().map(|app| app.startup_error.clone()).collect();
        Self {
            pending: apps,
            windows: HashMap::new(),
            startup_errors,
            factory,
            wake,
        }
    }

    fn apply_window_requests(&mut self, event_loop: &dyn ActiveEventLoop) {
        let requests = self
            .windows
            .values_mut()
            .flat_map(|app| std::mem::take(&mut app.pending_windows))
            .collect::<Vec<_>>();
        if let Some(factory) = self.factory.clone() {
            for (window_id, options) in requests {
                match factory(window_id, &options) {
                    Ok(mut source) => {
                        source.set_wake_callback(self.wake.clone());
                        let mut app = App::with_options(source, options);
                        app.logical_window_id = window_id;
                        app.can_create_surfaces(event_loop);
                        if let Some(id) = app.state.as_ref().map(|shell| shell.window().id()) {
                            self.startup_errors.push(app.startup_error.clone());
                            self.windows.insert(id, app);
                        }
                    }
                    Err(error) => {
                        tracing::error!(window_id, %error, "failed to create window runtime")
                    }
                }
            }
        }
        let commands = self
            .windows
            .values_mut()
            .flat_map(|app| std::mem::take(&mut app.pending_window_commands))
            .collect::<Vec<_>>();
        for (window_id, command) in commands {
            let Some(app) = find_window_by_logical_id(self.windows.values_mut(), window_id) else {
                tracing::warn!(window_id, "ignored command for unknown window");
                continue;
            };
            apply_window_command(app, command);
        }
        let closed = self
            .windows
            .iter()
            .filter_map(|(id, app)| app.close_requested.then_some(*id))
            .collect::<Vec<_>>();
        for id in closed {
            self.windows.remove(&id);
        }
        if self.windows.is_empty() {
            event_loop.exit();
        }
    }
}

impl ApplicationHandler for MultiWindowApp {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        for (index, mut app) in self.pending.drain(..).enumerate() {
            app.logical_window_id = index as u64 + 1;
            app.can_create_surfaces(event_loop);
            if let Some(id) = app.state.as_ref().map(|shell| shell.window().id()) {
                self.windows.insert(id, app);
            }
        }
        // Initial render may synchronously call createWindow()/close(). The
        // wake callback is not installed until the source enters this event
        // loop, so explicitly drain those boot-time requests here.
        self.apply_window_requests(event_loop);
        if self.windows.is_empty() {
            event_loop.exit();
        }
    }

    fn destroy_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        for app in self.windows.values_mut() {
            app.destroy_surfaces(event_loop);
        }
        self.windows.clear();
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        for app in self.windows.values_mut() {
            app.proxy_wake_up(event_loop);
        }
        self.apply_window_requests(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if matches!(event, WindowEvent::CloseRequested) {
            self.windows.remove(&window_id);
            if self.windows.is_empty() {
                event_loop.exit();
            }
            return;
        }
        if let Some(app) = self.windows.get_mut(&window_id) {
            app.window_event(event_loop, window_id, event);
        }
        self.apply_window_requests(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        let now = Instant::now();
        let mut earliest = None;
        for app in self.windows.values_mut() {
            if app.source.has_anim() {
                if let Some(shell) = app.state.as_ref() {
                    shell.window().request_redraw();
                }
                continue;
            }
            if let Some(deadline) = app.source.animation_deadline() {
                if deadline <= now {
                    if let Some(shell) = app.state.as_ref() {
                        shell.window().request_redraw();
                    }
                } else {
                    earliest = Some(earliest.map_or(deadline, |old: Instant| old.min(deadline)));
                }
            }
        }
        event_loop.set_control_flow(earliest.map_or(ControlFlow::Wait, ControlFlow::WaitUntil));
    }
}

pub fn run_window(source: Box<dyn FrameSource>) -> crate::Result<()> {
    run_window_with_size(source, 800, 600)
}

pub fn run_window_with_size(
    source: Box<dyn FrameSource>,
    width: u32,
    height: u32,
) -> crate::Result<()> {
    run_window_with_options(
        source,
        WindowOptions::new().initial_inner_size(width, height),
    )
}

pub fn run_window_with_options(
    source: Box<dyn FrameSource>,
    options: WindowOptions,
) -> crate::Result<()> {
    let event_loop: EventLoop = EventLoop::new().context(crate::error::CreateEventLoopSnafu)?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let proxy = event_loop.create_proxy();
    let mut source = source;
    source.set_wake_callback(std::sync::Arc::new(move || proxy.wake_up()));
    let app = App::with_options(source, options);
    let startup_error = app.startup_error.clone();
    event_loop
        .run_app(app)
        .context(crate::error::RunEventLoopSnafu)?;
    if let Some(error) = startup_error
        .lock()
        .expect("startup error mutex poisoned")
        .take()
    {
        return Err(error);
    }
    Ok(())
}

/// Run independent frame sources as native windows on one platform event loop.
pub fn run_windows(windows: Vec<(Box<dyn FrameSource>, WindowOptions)>) -> crate::Result<()> {
    run_windows_with_factory(windows, None)
}

pub fn run_windows_with_factory(
    mut windows: Vec<(Box<dyn FrameSource>, WindowOptions)>,
    factory: Option<FrameSourceFactory>,
) -> crate::Result<()> {
    if windows.is_empty() {
        return Ok(());
    }
    let event_loop: EventLoop = EventLoop::new().context(crate::error::CreateEventLoopSnafu)?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let proxy = event_loop.create_proxy();
    let wake: WakeCallback = std::sync::Arc::new(move || proxy.wake_up());
    let apps = windows
        .drain(..)
        .map(|(mut source, options)| {
            source.set_wake_callback(wake.clone());
            App::with_options(source, options)
        })
        .collect();
    let app = MultiWindowApp::new(apps, factory, wake);
    let errors = app.startup_errors.clone();
    event_loop
        .run_app(app)
        .context(crate::error::RunEventLoopSnafu)?;
    for error in errors {
        if let Some(error) = error.lock().expect("startup error mutex poisoned").take() {
            return Err(error);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
