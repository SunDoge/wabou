//! GPUI view owning one Wabou JavaScript runtime.

use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    rc::Rc,
    sync::Arc,
};

use gpui_base::input::{
    Editor, EditorState, Input, InputBaseState, InputEditorStyle, InputEvent, InputModeKind,
    InputState, Redo, Textarea, TextareaState, Undo,
};
use gpui_base::{TextSelectionHandle, TextSelectionLayer};
use wabou_shell::WakeCallback;
use wabou_shell::gpui::{
    AppContext as _, ClipboardItem, Context, DragMoveEvent, Entity, EntityInputHandler,
    ExternalPaths, FocusHandle, Focusable as _, InteractiveElement as _, IntoElement as _,
    ParentElement as _, PathPromptOptions, PromptButton, PromptLevel, Render, StyleRefinement,
    Styled as _, Subscription, SystemNotification, Task, Window, div,
};

use crate::gpui_controller::GpuiController;
use crate::gpui_performance_hud::{GpuiPerformanceHud, performance_hud_enabled};
use crate::gpui_projection_boundary::{
    GpuiProjectionBoundary, GpuiProjectionBoundaryState, NativeElementBuilder,
};
use wabou_shell::{
    ClipboardRequest, EffectCompletion, EffectErrorCode, EffectPayload, EffectRequest,
    EffectResult, WindowCommand,
};

/// Window coordinator for one Solid application runtime.
///
/// Solid retains individual UI nodes and emits one mutation batch per flush.
/// The expensive element materialization belongs to an explicitly cached
/// [`GpuiProjectionBoundary`]; this root remains responsible for native window
/// services and frame coordination without creating one Entity per JSX node.
pub(crate) struct GpuiRuntimeView {
    controller: GpuiController,
    // Retaining the task ties the async bridge to the lifetime of this view.
    // The task itself only owns a weak entity handle, so this is not a cycle.
    _wake_task: Task<()>,
    runtime_wake: WakeCallback,
    runtime_work_pending: bool,
    focus: FocusHandle,
    text_controls: BTreeMap<wabou_host_api::NodeKey, GpuiTextControlState>,
    text_selections: BTreeMap<wabou_host_api::NodeKey, GpuiTextSelectionState>,
    projection_boundary: Option<Entity<GpuiProjectionBoundary>>,
    projection_subtrees: BTreeMap<wabou_host_api::NodeKey, Entity<GpuiProjectionBoundary>>,
    projection_boundary_revision: u64,
    projection_force_revision: u64,
    observed_projection_revision: u64,
    performance_hud: Option<Entity<GpuiPerformanceHud>>,
    window_size_persistence: Option<wabou_shell::WindowSizePersistence>,
    native_widget_factories: HashMap<String, wabou_shell::NativeWidgetFactory>,
    test_controller: Option<crate::test_driver::TestController>,
    window_key: wabou_shell::WindowResourceKey,
    window_host: std::rc::Rc<crate::gpui_windows::GpuiApplicationWindows>,
    file_drag_paths: Vec<std::path::PathBuf>,
    file_drag_position: Option<wabou_shell::Point>,
    projected_base_theme: Option<wabou_shell::GpuiThemeSnapshot>,
}

pub(crate) struct GpuiRuntimeViewOptions {
    pub(crate) window_size_persistence: Option<wabou_shell::WindowSizePersistence>,
    pub(crate) native_widget_factories: HashMap<String, wabou_shell::NativeWidgetFactory>,
    pub(crate) test_controller: Option<crate::test_driver::TestController>,
    pub(crate) window_key: wabou_shell::WindowResourceKey,
    pub(crate) window_host: std::rc::Rc<crate::gpui_windows::GpuiApplicationWindows>,
}

enum GpuiTextControlState {
    Input {
        state: Entity<InputState>,
        _subscriptions: [Subscription; 2],
    },
    Textarea {
        state: Entity<TextareaState>,
        _subscriptions: [Subscription; 2],
    },
    Editor {
        state: Entity<EditorState>,
        language: Option<String>,
        _subscriptions: [Subscription; 2],
    },
}

struct GpuiTextSelectionState {
    handle: TextSelectionHandle,
    select_all: bool,
    _subscription: Subscription,
}

fn utf16_to_utf8_offset(value: &str, target: u32) -> usize {
    let target = target as usize;
    let mut utf16 = 0;
    for (byte, character) in value.char_indices() {
        if utf16 >= target {
            return byte;
        }
        let next = utf16 + character.len_utf16();
        if next > target {
            return byte;
        }
        utf16 = next;
    }
    value.len()
}

fn utf8_to_utf16_offset(value: &str, byte: usize) -> u32 {
    value
        .get(..byte.min(value.len()))
        .unwrap_or_default()
        .encode_utf16()
        .count() as u32
}

/// Synchronize an authored controlled value without destroying an active
/// platform composition.
///
/// GPUI intentionally keeps IME preedit text native and does not emit
/// `InputEvent::Change` until the candidate is committed. During that interval
/// Solid's `value` is necessarily one edit behind. Applying that stale value
/// with `set_value` would clear GPUI's marked range and reset a multiline
/// selection to `0..0`.
fn synchronize_controlled_text_value<M: InputModeKind>(
    state: &mut InputBaseState<M>,
    authored_value: &str,
    window: &mut Window,
    cx: &mut Context<InputBaseState<M>>,
) {
    if state.value().as_ref() == authored_value {
        return;
    }
    if EntityInputHandler::marked_text_range(state, window, cx).is_some() {
        return;
    }
    state.set_value(authored_value.to_owned(), window, cx);
}

impl GpuiTextControlState {
    fn kind(&self) -> wabou_shell::GpuiTextControlKind {
        match self {
            Self::Input { .. } => wabou_shell::GpuiTextControlKind::Input,
            Self::Textarea { .. } => wabou_shell::GpuiTextControlKind::Textarea,
            Self::Editor { .. } => wabou_shell::GpuiTextControlKind::Editor,
        }
    }

    fn synchronize(
        &mut self,
        descriptor: &wabou_shell::GpuiTextControl,
        window: &mut Window,
        cx: &mut Context<GpuiRuntimeView>,
    ) {
        macro_rules! synchronize {
            ($state:expr) => {
                $state.update(cx, |state, cx| {
                    synchronize_controlled_text_value(state, &descriptor.value, window, cx);
                    if state.presentation().placeholder().as_ref() != descriptor.placeholder {
                        state.set_placeholder(descriptor.placeholder.clone(), window, cx);
                    }
                    state.set_disabled(descriptor.disabled, cx);
                    state.set_readonly(descriptor.readonly, cx);
                    state.set_submit_on_enter(descriptor.submit_on_enter, cx);
                    state.set_editor_style(InputEditorStyle {
                        foreground: descriptor.style.foreground,
                        muted_foreground: descriptor.style.muted_foreground,
                        background: descriptor.style.background,
                        border: descriptor.style.border,
                        selection: descriptor.style.selection,
                        caret: descriptor.style.caret,
                        ..InputEditorStyle::default()
                    });
                })
            };
        }
        match self {
            Self::Input { state, .. } => synchronize!(state),
            Self::Textarea { state, .. } => synchronize!(state),
            Self::Editor {
                state, language, ..
            } => {
                synchronize!(state);
                if language != &descriptor.language {
                    state.update(cx, |state, state_cx| {
                        state.set_highlighter(
                            descriptor.language.clone().unwrap_or_default(),
                            state_cx,
                        );
                    });
                    *language = descriptor.language.clone();
                }
            }
        }
    }

    fn element_builder(&self) -> NativeElementBuilder {
        match self {
            Self::Input { state, .. } => {
                let state = state.clone();
                Rc::new(move || {
                    div()
                        // `InputState` requests a line-height-sized child.
                        // Center it inside the authored editor surface.
                        .size_full()
                        .flex()
                        .items_center()
                        .child(Input::new(&state))
                        .into_any_element()
                })
            }
            Self::Textarea { state, .. } => {
                let state = state.clone();
                Rc::new(move || {
                    div()
                        .size_full()
                        .child(Textarea::new(&state))
                        .into_any_element()
                })
            }
            Self::Editor { state, .. } => {
                let state = state.clone();
                Rc::new(move || {
                    div()
                        .size_full()
                        .child(Editor::new(&state))
                        .into_any_element()
                })
            }
        }
    }

    fn focus_handle(&self, cx: &wabou_shell::gpui::App) -> FocusHandle {
        match self {
            Self::Input { state, .. } => state.focus_handle(cx),
            Self::Textarea { state, .. } => state.focus_handle(cx),
            Self::Editor { state, .. } => state.focus_handle(cx),
        }
    }

    fn set_selection_utf16(&self, anchor: u32, head: u32, cx: &mut Context<GpuiRuntimeView>) {
        macro_rules! set_selection {
            ($state:expr) => {{
                $state.update(cx, |state, state_cx| {
                    let value = state.value();
                    let anchor = utf16_to_utf8_offset(&value, anchor);
                    let head = utf16_to_utf8_offset(&value, head);
                    state.set_selected_range(anchor.min(head)..anchor.max(head), state_cx);
                })
            }};
        }
        match self {
            Self::Input { state, .. } => set_selection!(state),
            Self::Textarea { state, .. } => set_selection!(state),
            Self::Editor { state, .. } => set_selection!(state),
        }
    }

    fn select_all(&self, window: &mut Window, cx: &mut Context<GpuiRuntimeView>) {
        macro_rules! select_all {
            ($state:expr) => {{ $state.update(cx, |state, state_cx| state.select_all(window, state_cx)) }};
        }
        match self {
            Self::Input { state, .. } => select_all!(state),
            Self::Textarea { state, .. } => select_all!(state),
            Self::Editor { state, .. } => select_all!(state),
        }
    }
}

fn project_base_theme(
    snapshot: &wabou_shell::GpuiThemeSnapshot,
    mut theme: gpui_base::Theme,
) -> gpui_base::Theme {
    let colors = &snapshot.colors;
    let color = |names: &[&str], fallback: wabou_shell::gpui::Hsla| {
        names
            .iter()
            .find_map(|name| colors.get(*name).copied())
            .map(wabou_shell::gpui::rgba)
            .map(wabou_shell::gpui::rgb_to_hsla)
            .unwrap_or(fallback)
    };
    let tokens = &mut theme.tokens.colors;
    tokens.background = color(&["canvas"], tokens.background);
    tokens.foreground = color(&["primary"], tokens.foreground);
    tokens.surface = color(&["surface"], tokens.surface);
    tokens.surface_foreground = color(&["primary"], tokens.surface_foreground);
    tokens.primary = color(&["accent"], tokens.primary);
    tokens.primary_foreground = color(&["on-accent"], tokens.primary_foreground);
    tokens.secondary = color(&["control", "surface-muted"], tokens.secondary);
    tokens.secondary_foreground = color(&["primary", "secondary"], tokens.secondary_foreground);
    tokens.muted = color(&["surface-muted", "control"], tokens.muted);
    tokens.muted_foreground = color(&["muted"], tokens.muted_foreground);
    tokens.accent = color(&["selected", "accent"], tokens.accent);
    tokens.accent_foreground = color(&["primary"], tokens.accent_foreground);
    tokens.destructive = color(&["danger"], tokens.destructive);
    tokens.destructive_foreground = color(
        &["danger-primary", "on-accent"],
        tokens.destructive_foreground,
    );
    tokens.border = color(&["subtle", "strong"], tokens.border);
    tokens.input = color(&["input"], tokens.input);
    tokens.ring = color(&["focus", "accent"], tokens.ring);
    theme.appearance = if snapshot.dark {
        gpui_base::ThemeAppearance::Dark
    } else {
        gpui_base::ThemeAppearance::Light
    };
    theme
}

impl GpuiRuntimeView {
    /// Wrap an already configured and booted Wabou runtime.
    #[must_use]
    pub(crate) fn new(
        mut controller: GpuiController,
        options: GpuiRuntimeViewOptions,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let (wake, receiver) = gpui_wake_channel();
        controller.install_runtime_wake(wake.clone());
        if let Some(test_controller) = &options.test_controller {
            test_controller.connect_gpui_window(options.window_key, wake.clone());
        }

        let wake_task = cx.spawn(async move |view, cx| {
            while receiver.recv_async().await.is_ok() {
                if view
                    .update(cx, |view, cx| {
                        // Drain work immediately instead of waiting for an unrelated
                        // input event or animation frame. The following render pass
                        // publishes any resulting Solid mutation batch to GPUI.
                        let _ = view.controller.poll_runtime();
                        view.runtime_work_pending = true;
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
        let performance_hud =
            performance_hud_enabled().then(|| cx.new(|_| GpuiPerformanceHud::new()));
        Self {
            controller,
            _wake_task: wake_task,
            runtime_wake: wake,
            runtime_work_pending: true,
            focus,
            text_controls: BTreeMap::new(),
            text_selections: BTreeMap::new(),
            projection_boundary: None,
            projection_subtrees: BTreeMap::new(),
            projection_boundary_revision: 0,
            projection_force_revision: 0,
            observed_projection_revision: 0,
            performance_hud,
            window_size_persistence: options.window_size_persistence,
            native_widget_factories: options.native_widget_factories,
            test_controller: options.test_controller,
            window_key: options.window_key,
            window_host: options.window_host,
            file_drag_paths: Vec::new(),
            file_drag_position: None,
            projected_base_theme: None,
        }
    }

    fn synchronize_base_theme(&mut self, cx: &mut Context<Self>) {
        let Some(snapshot) = self.controller.active_theme_snapshot() else {
            return;
        };
        if self.projected_base_theme.as_ref() == Some(&snapshot) {
            return;
        }
        let fallback = gpui_base::Theme::global(cx);
        let projected = project_base_theme(&snapshot, fallback);
        let theme = gpui_base::Theme::global_mut(cx);
        theme.appearance = projected.appearance;
        theme.tokens = projected.tokens;
        self.projected_base_theme = Some(snapshot);
    }

    fn synchronize_text_selections(
        &mut self,
        window: &Window,
        cx: &mut Context<GpuiRuntimeView>,
    ) -> Rc<BTreeMap<wabou_host_api::NodeKey, wabou_shell::ProjectedTextSelection>> {
        let descriptors = self.controller.selectable_texts();
        self.text_selections
            .retain(|key, _| descriptors.iter().any(|text| text.key == *key));
        let projected = descriptors
            .into_iter()
            .map(|text| {
                if self
                    .text_selections
                    .get(&text.key)
                    .is_some_and(|state| state.select_all != text.select_all)
                {
                    self.text_selections.remove(&text.key);
                }
                let state = self.text_selections.entry(text.key).or_insert_with(|| {
                    let handle = TextSelectionHandle::new(text.text.to_string(), cx);
                    if text.select_all {
                        let copy_text = text.text.to_string();
                        handle.copy_with(move |_| copy_text.clone(), cx);
                    }
                    let subscription = handle.refresh_window_on_change(window, cx);
                    GpuiTextSelectionState {
                        handle,
                        select_all: text.select_all,
                        _subscription: subscription,
                    }
                });
                state
                    .handle
                    .set_fallback_copy_text(text.text.to_string(), cx);
                if text.select_all {
                    let copy_text = text.text.to_string();
                    state.handle.copy_with(move |_| copy_text.clone(), cx);
                }
                (
                    text.key,
                    wabou_shell::ProjectedTextSelection::new(
                        state.handle.clone(),
                        text.document_order,
                        text.select_all,
                    ),
                )
            })
            .collect();
        Rc::new(projected)
    }

    fn handle_file_drag_move(
        &mut self,
        paths: &[std::path::PathBuf],
        position: wabou_shell::Point,
    ) {
        let phase = if self.file_drag_paths.is_empty() {
            self.file_drag_paths = paths.to_vec();
            wabou_shell::FileDropPhase::Entered
        } else {
            wabou_shell::FileDropPhase::Moved
        };
        self.file_drag_position = Some(position);
        let _ = self
            .controller
            .dispatch_file_drop(wabou_shell::FileDropEvent {
                phase,
                paths: self.file_drag_paths.clone(),
                position: Some(position),
            });
    }

    fn finish_file_drag(&mut self, dropped_paths: Option<&[std::path::PathBuf]>) {
        if self.file_drag_paths.is_empty() && dropped_paths.is_none() {
            return;
        }
        let phase = if dropped_paths.is_some() {
            wabou_shell::FileDropPhase::Dropped
        } else {
            wabou_shell::FileDropPhase::Left
        };
        let paths = dropped_paths
            .map(<[std::path::PathBuf]>::to_vec)
            .unwrap_or_else(|| std::mem::take(&mut self.file_drag_paths));
        self.file_drag_paths.clear();
        let position = self.file_drag_position.take();
        let _ = self
            .controller
            .dispatch_file_drop(wabou_shell::FileDropEvent {
                phase,
                paths,
                position,
            });
    }

    fn window_metrics(
        &self,
        window: &Window,
        cx: &wabou_shell::gpui::App,
    ) -> wabou_shell::WindowMetrics {
        let viewport = window.viewport_size();
        let logical_width = f32::from(viewport.width).round().max(1.0) as u32;
        let logical_height = f32::from(viewport.height).round().max(1.0) as u32;
        let scale_factor = f64::from(window.scale_factor()).max(f64::EPSILON);
        let outer = window.bounds().origin;
        let color_scheme = match window.appearance() {
            wabou_shell::gpui::WindowAppearance::Light
            | wabou_shell::gpui::WindowAppearance::VibrantLight => wabou_shell::ColorScheme::Light,
            wabou_shell::gpui::WindowAppearance::Dark
            | wabou_shell::gpui::WindowAppearance::VibrantDark => wabou_shell::ColorScheme::Dark,
        };
        wabou_shell::WindowMetrics {
            window_key: self.window_key,
            logical_width,
            logical_height,
            physical_width: (f64::from(logical_width) * scale_factor).round() as u32,
            physical_height: (f64::from(logical_height) * scale_factor).round() as u32,
            scale_factor,
            maximized: window.is_maximized(),
            focused: window.is_window_active(),
            outer_x: Some(f32::from(outer.x).round() as i32),
            outer_y: Some(f32::from(outer.y).round() as i32),
            occluded: false,
            color_scheme: Some(color_scheme),
            reduced_motion: cx.reduce_motion(),
        }
    }

    fn synchronize_text_controls(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        macro_rules! subscribe_text_control {
            ($state:expr, $key:expr) => {{
                let events = cx.subscribe(&$state, move |view, state, event, cx| {
                    let value = matches!(event, InputEvent::Change)
                        .then(|| state.read(cx).value().to_string());
                    let focused = match event {
                        InputEvent::Focus => Some(true),
                        InputEvent::Blur => Some(false),
                        _ => None,
                    };
                    if let Some(value) = value {
                        view.handle_input(
                            wabou_shell::ProjectedInputEvent::TextChange {
                                target: $key,
                                value,
                            },
                            cx,
                        );
                    }
                    if let Some(focused) = focused {
                        view.handle_input(
                            wabou_shell::ProjectedInputEvent::FocusChange {
                                target: $key,
                                focused,
                            },
                            cx,
                        );
                    }
                    if let InputEvent::PressEnter { secondary, shift } = event {
                        view.handle_input(
                            wabou_shell::ProjectedInputEvent::Submit {
                                target: $key,
                                secondary: *secondary,
                                shift: *shift,
                            },
                            cx,
                        );
                    }
                });
                let previous = Rc::new(std::cell::Cell::new(None));
                let selection = cx.observe(&$state, move |view, state, cx| {
                    let state = state.read(cx);
                    let range = state.selected_range();
                    let head = state.cursor();
                    let anchor = if head == range.start {
                        range.end
                    } else {
                        range.start
                    };
                    let value = state.value();
                    let next = (
                        utf8_to_utf16_offset(&value, anchor),
                        utf8_to_utf16_offset(&value, head),
                    );
                    if previous.replace(Some(next)) != Some(next) {
                        view.handle_input(
                            wabou_shell::ProjectedInputEvent::TextSelectionChange {
                                target: $key,
                                anchor: next.0,
                                head: next.1,
                            },
                            cx,
                        );
                    }
                });
                [events, selection]
            }};
        }
        let descriptors = self.controller.text_controls();
        self.text_controls
            .retain(|key, _| descriptors.iter().any(|descriptor| descriptor.key == *key));

        for descriptor in descriptors {
            let needs_recreate = self
                .text_controls
                .get(&descriptor.key)
                .is_some_and(|state| state.kind() != descriptor.kind);
            if needs_recreate {
                self.text_controls.remove(&descriptor.key);
            }
            if let std::collections::btree_map::Entry::Vacant(entry) =
                self.text_controls.entry(descriptor.key)
            {
                let key = descriptor.key;
                let control = match descriptor.kind {
                    wabou_shell::GpuiTextControlKind::Input => {
                        let state = cx.new(|cx| InputState::new(window, cx));
                        let subscriptions = subscribe_text_control!(state, key);
                        GpuiTextControlState::Input {
                            state,
                            _subscriptions: subscriptions,
                        }
                    }
                    wabou_shell::GpuiTextControlKind::Textarea => {
                        let state = cx.new(|cx| TextareaState::new(window, cx));
                        let subscriptions = subscribe_text_control!(state, key);
                        GpuiTextControlState::Textarea {
                            state,
                            _subscriptions: subscriptions,
                        }
                    }
                    wabou_shell::GpuiTextControlKind::Editor => {
                        let language = descriptor.language.clone();
                        let initial_language = language.clone().unwrap_or_default();
                        let state =
                            cx.new(|cx| EditorState::new(window, cx).language(initial_language));
                        let subscriptions = subscribe_text_control!(state, key);
                        GpuiTextControlState::Editor {
                            state,
                            language,
                            _subscriptions: subscriptions,
                        }
                    }
                };
                entry.insert(control);
            }
            self.text_controls
                .get_mut(&descriptor.key)
                .expect("text control was retained or created")
                .synchronize(&descriptor, window, cx);
        }
    }

    pub(crate) fn layout_snapshot(&self) -> Vec<wabou_shell::GpuiLayoutNode> {
        self.controller
            .layout_snapshot()
            .into_iter()
            .filter(|node| node.attached)
            .collect()
    }

    #[cfg(feature = "headless")]
    pub(crate) fn protocol_revision(&self) -> u64 {
        self.controller.protocol_revision()
    }

    #[cfg(feature = "headless")]
    pub(crate) fn projection_checkpoint(
        &self,
        cx: &wabou_shell::gpui::App,
    ) -> crate::gpui_headless::GpuiProjectionCheckpoint {
        let snapshot = self.controller.projection_render_snapshot();
        let revisions = self.controller.projection_boundary_revisions();
        let boundaries = revisions
            .iter()
            .filter_map(|(root, revision)| {
                let entity = if *root == wabou_shell::NodeKey::ROOT {
                    self.projection_boundary.as_ref()
                } else {
                    self.projection_subtrees.get(root)
                }?;
                Some(crate::gpui_headless::GpuiProjectionBoundaryCheckpoint {
                    root: *root,
                    revision: *revision,
                    materializations: entity.read(cx).materialization_count(),
                    owned_nodes: snapshot.projection_boundary_node_count(*root),
                })
            })
            .collect();
        crate::gpui_headless::GpuiProjectionCheckpoint {
            protocol_revision: self.controller.protocol_revision(),
            boundaries,
        }
    }

    #[cfg(feature = "headless")]
    pub(crate) fn eval_script_diagnostic(&mut self, source: &str) -> Result<(), String> {
        self.controller.eval_script_diagnostic(source)?;
        // Headless fixture setup runs outside a GPUI render turn. Mark the
        // runtime dirty so rAF scheduled by the evaluated component receives
        // its first native frame instead of waiting for unrelated input.
        self.runtime_work_pending = true;
        Ok(())
    }

    #[cfg(feature = "headless")]
    pub(crate) fn eval_string(&self, source: &str) -> rquickjs::Result<String> {
        self.controller.eval_string(source)
    }

    fn handle_input(&mut self, event: wabou_shell::ProjectedInputEvent, cx: &mut Context<Self>) {
        let mut response = self.controller.handle_input(event);
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
                        let pasted = self.controller.dispatch_paste(text);
                        response.handled |= pasted.handled;
                        response.request_redraw |= pasted.request_redraw;
                    }
                }
            }
        }
        if response.request_redraw {
            // A native event may synchronously enqueue Solid mutations in the
            // QuickJS protocol writer. Merely notifying the GPUI Entity is not
            // sufficient: render() is allowed to skip the runtime when no RAF
            // or external wake is pending, which would strand mutations from
            // transition completion, text controls, and native widgets until
            // an unrelated input arrives.
            self.runtime_work_pending = true;
            cx.notify();
        }
    }

    fn drain_effects(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let mut handled = false;
        while let Some(request) = self.controller.take_runtime_effect() {
            handled = true;
            let completion = self.execute_effect(request, window, cx);
            if let Some(completion) = completion {
                self.controller.complete_runtime_effect(completion);
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
            EffectPayload::WindowCreate(request) => {
                match self.window_host.create(request.options, cx) {
                    Ok(window) => EffectResult::Window(window),
                    Err(crate::gpui_windows::GpuiWindowError::Unsupported(message)) => {
                        EffectResult::Error {
                            code: EffectErrorCode::Unsupported,
                            message,
                        }
                    }
                    Err(crate::gpui_windows::GpuiWindowError::Platform(message)) => {
                        EffectResult::Error {
                            code: EffectErrorCode::PlatformFailure,
                            message,
                        }
                    }
                }
            }
            EffectPayload::WindowControl { window_id, command } => {
                self.execute_window_command(window_id, command, window, cx)
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
            EffectPayload::ContextMenuShow(_)
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

    fn execute_window_command(
        &mut self,
        target: wabou_shell::WindowResourceKey,
        command: WindowCommand,
        current_window: &mut Window,
        cx: &mut Context<Self>,
    ) -> EffectResult {
        let closes = matches!(command, WindowCommand::Close);
        if target == self.window_key {
            apply_window_command(current_window, command);
            if closes {
                self.window_host.remove(target);
            }
            return EffectResult::Unit;
        }

        let Some(handle) = self.window_host.resolve(target) else {
            return EffectResult::Error {
                code: EffectErrorCode::InvalidRequest,
                message: format!("window `{target}` is not live"),
            };
        };
        match handle.update(cx, move |_, window, _| {
            apply_window_command(window, command)
        }) {
            Ok(()) => {
                if closes {
                    self.window_host.remove(target);
                }
                EffectResult::Unit
            }
            Err(error) => {
                self.window_host.remove(target);
                EffectResult::Error {
                    code: EffectErrorCode::PlatformFailure,
                    message: format!("failed to control window `{target}`: {error}"),
                }
            }
        }
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
                view.controller
                    .complete_runtime_effect(EffectCompletion { id, op, result });
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
                view.controller
                    .complete_runtime_effect(EffectCompletion { id, op, result });
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
                view.controller
                    .complete_runtime_effect(EffectCompletion { id, op, result });
                cx.notify();
            });
        })
        .detach();
    }
}

fn apply_window_command(window: &mut Window, command: WindowCommand) {
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
    ) -> impl wabou_shell::gpui::IntoElement {
        let frame_started = std::time::Instant::now();
        let viewport = window.viewport_size();
        let viewport_width = f32::from(viewport.width).round().max(1.0) as u32;
        let viewport_height = f32::from(viewport.height).round().max(1.0) as u32;
        // At the start of a render, the shared bounds still describe the
        // previous completed GPUI prepaint pass. Publish them before running
        // JavaScript work for this frame, so synchronous layout reads never
        // observe a half-built projection.
        let completed_layout_changed = self.controller.has_window_metrics()
            && self.controller.completed_layout_needs_publish()
            && self.controller.publish_completed_layout();
        let metrics = self.window_metrics(window, cx);
        self.controller.update_window_metrics(metrics);
        if let Some(persistence) = &mut self.window_size_persistence {
            let width: f32 = viewport.width.into();
            let height: f32 = viewport.height.into();
            persistence.observe(
                width.round().max(1.0) as u32,
                height.round().max(1.0) as u32,
                window.is_maximized(),
            );
        }
        let advance_runtime = self.runtime_work_pending
            || self.projection_boundary.is_none()
            || self.controller.has_animation();
        self.runtime_work_pending = false;
        let (advanced_projection, needs_runtime_followup, frame_timing) = if advance_runtime {
            self.controller.advance_ready_work_profiled(8)
        } else {
            (
                false,
                false,
                crate::gpui_controller::GpuiFrameTiming::default(),
            )
        };
        let committed_projection_revision = self.controller.projection_revision();
        let projection_changed = advanced_projection
            || committed_projection_revision != self.observed_projection_revision;
        if projection_changed {
            self.observed_projection_revision = committed_projection_revision;
        }
        let changed_boundaries = self.controller.take_projection_boundary_changes();
        if needs_runtime_followup {
            // Solid 2 and asynchronously loaded Vite modules may enqueue their
            // retained mutations after this turn's protocol writer was flushed.
            // Re-enter through the GPUI-managed pump used by IO and host
            // messages. This guarantees a later Entity update instead of a
            // notification coalesced into the render already in progress.
            (self.runtime_wake)();
        }
        let mut boundary_dirty =
            completed_layout_changed || projection_changed || self.projection_boundary.is_none();
        let mut force_boundary_sync = self.projection_boundary.is_none();
        self.synchronize_base_theme(cx);
        let fonts = self.controller.take_pending_fonts();
        if !fonts.is_empty() {
            let count = fonts.len();
            if let Err(error) = window
                .text_system()
                .add_fonts(fonts.into_iter().map(Cow::Owned).collect())
            {
                tracing::warn!(%error, count, "failed to register application fonts with GPUI");
            } else {
                // Font availability changes shaping and therefore layout even
                // when the retained Solid tree itself did not mutate.
                boundary_dirty = true;
                force_boundary_sync = true;
                cx.notify();
            }
        }
        // Native text controls must exist before imperative focus/selection
        // commands below are dispatched. A projection flush may create a
        // control and focus it in the same Solid transaction.
        let mut text_controls_synchronized = false;
        if projection_changed || self.projection_boundary.is_none() {
            self.synchronize_text_controls(window, cx);
            text_controls_synchronized = true;
        }
        let projection_commands = self.controller.take_projection_commands();
        boundary_dirty |= !projection_commands.is_empty();
        if !projection_commands.is_empty() {
            // Focus, selection, and imperative text commands change native
            // state outside the retained style/tree revision clocks. They are
            // intentionally rare and require refreshing every boundary that
            // may own the active native input handler.
            force_boundary_sync = true;
        }
        for command in projection_commands {
            match command {
                wabou_shell::GpuiCommand::Focus { id } => {
                    let focus = self
                        .text_controls
                        .get(&id)
                        .map(|control| control.focus_handle(cx))
                        .unwrap_or_else(|| self.focus.clone());
                    focus.focus(window, cx);
                    if self.controller.set_text_focus(id, true) {
                        cx.notify();
                    }
                }
                wabou_shell::GpuiCommand::Blur { id } => {
                    if self.controller.set_text_focus(id, false) {
                        window.blur();
                        cx.notify();
                    }
                }
                wabou_shell::GpuiCommand::SetTextSelection { id, anchor, head } => {
                    if let Some(control) = self.text_controls.get(&id) {
                        control.set_selection_utf16(anchor, head, cx);
                        cx.notify();
                    }
                }
                wabou_shell::GpuiCommand::Text { id, command } => {
                    if let Some(control) = self.text_controls.get(&id) {
                        match command {
                            wabou_shell::GpuiTextCommand::SelectAll => {
                                control.select_all(window, cx);
                            }
                            wabou_shell::GpuiTextCommand::Undo => {
                                control.focus_handle(cx).focus(window, cx);
                                window.dispatch_action(Box::new(Undo), cx);
                            }
                            wabou_shell::GpuiTextCommand::Redo => {
                                control.focus_handle(cx).focus(window, cx);
                                window.dispatch_action(Box::new(Redo), cx);
                            }
                        }
                        cx.notify();
                    }
                }
                command @ (wabou_shell::GpuiCommand::ScrollTo { .. }
                | wabou_shell::GpuiCommand::ScrollBy { .. }) => {
                    let _ = self.controller.apply_projection_scroll(command);
                    cx.notify();
                }
            }
        }
        if let Some(test_controller) = self.test_controller.clone() {
            test_controller.record_gpui_viewport(self.window_key, viewport_width, viewport_height);
            let window_action =
                test_controller.poll_gpui_window_action(self.window_key, |command| match command {
                    crate::test_driver::GpuiWindowTestCommand::Hide { mutable_visibility } => {
                        if mutable_visibility {
                            cx.hide();
                            true
                        } else {
                            false
                        }
                    }
                    crate::test_driver::GpuiWindowTestCommand::Show => {
                        cx.activate(true);
                        window.activate_window();
                        true
                    }
                    crate::test_driver::GpuiWindowTestCommand::Resize { width, height } => {
                        window.resize(wabou_shell::gpui::size(
                            wabou_shell::gpui::px(width as f32),
                            wabou_shell::gpui::px(height as f32),
                        ));
                        true
                    }
                });
            let input_snapshot = self.controller.projection_render_snapshot();
            let root_projection_boundary = self.projection_boundary.clone();
            let projection_subtrees = self.projection_subtrees.clone();
            let source_action = test_controller.poll_gpui_source(
                self.window_key,
                &self.layout_snapshot(),
                &mut self.controller,
                |key, event| {
                    let boundary =
                        input_snapshot
                            .nearest_projection_boundary(key)
                            .and_then(|owner| {
                                if owner == wabou_host_api::NodeKey::ROOT {
                                    root_projection_boundary.as_ref()
                                } else {
                                    projection_subtrees.get(&owner)
                                }
                            });
                    boundary.is_some_and(|boundary| {
                        boundary.update(cx, |boundary, cx| {
                            boundary.dispatch_native_input(key, event, window, cx)
                        })
                    })
                },
            );
            if source_action {
                self.synchronize_text_controls(window, cx);
                text_controls_synchronized = true;
                boundary_dirty = true;
                force_boundary_sync = true;
            }
            if window_action || source_action {
                cx.notify();
            }
        }
        if self.drain_effects(window, cx) {
            cx.notify();
        }

        if self.controller.has_animation() {
            // GPUI associates this request with the currently rendering view
            // and notifies only that entity on the next platform frame.
            window.request_animation_frame();
        }

        if boundary_dirty {
            self.projection_boundary_revision =
                self.projection_boundary_revision.wrapping_add(1).max(1);
        }
        if force_boundary_sync || !changed_boundaries.is_empty() {
            let snapshot = self.controller.projection_render_snapshot();
            let boundary_keys = snapshot
                .projection_boundaries()
                .into_iter()
                .filter(|key| *key != wabou_host_api::NodeKey::ROOT)
                .collect::<std::collections::BTreeSet<_>>();
            let retained_boundary_keys = self
                .projection_subtrees
                .keys()
                .copied()
                .collect::<std::collections::BTreeSet<_>>();
            let topology_changed = boundary_keys != retained_boundary_keys;
            force_boundary_sync |= topology_changed;
            if force_boundary_sync {
                self.projection_force_revision =
                    self.projection_force_revision.wrapping_add(1).max(1);
            }
            self.projection_subtrees
                .retain(|key, _| boundary_keys.contains(key));

            let mut affected_boundaries = if force_boundary_sync {
                let mut affected = boundary_keys.clone();
                affected.insert(wabou_host_api::NodeKey::ROOT);
                affected
            } else {
                changed_boundaries
                    .keys()
                    .copied()
                    .filter(|key| {
                        *key == wabou_host_api::NodeKey::ROOT || boundary_keys.contains(key)
                    })
                    .collect::<std::collections::BTreeSet<_>>()
            };

            // A completed GPUI layout pass updates shared geometry in place;
            // it does not require rebuilding any retained Solid boundary. Only
            // prepare native state when an exact changed owner (or an explicit
            // force condition above) requires synchronization.
            if !affected_boundaries.is_empty() {
                if !text_controls_synchronized {
                    self.synchronize_text_controls(window, cx);
                }
                let view = cx.weak_entity();
                let input: wabou_shell::ProjectedInputSink = Rc::new(move |event, app| {
                    let _ = view.update(app, |view, cx| {
                        view.handle_input(event, cx);
                    });
                });
                let mut text_input = self.controller.text_input_state();
                if self
                    .controller
                    .focused_target()
                    .is_some_and(|target| self.text_controls.contains_key(&target))
                {
                    // The child InputState owns the platform input handler. Keeping
                    // the transitional root handler active would commit IME twice.
                    text_input.accepts_text = false;
                }
                let all_widgets = self
                    .controller
                    .native_widgets(|tag| self.native_widget_factories.contains_key(tag));
                let text_selections = self.synchronize_text_selections(window, cx);

                // Synchronize only changed retained entities. Leaves go first
                // so a changed parent always references an already-current
                // child Entity, while stable siblings are never prepared.
                let mut pending = affected_boundaries
                    .iter()
                    .copied()
                    .filter(|key| *key != wabou_host_api::NodeKey::ROOT)
                    .collect::<std::collections::BTreeSet<_>>();
                while !pending.is_empty() {
                    let ready = pending
                        .iter()
                        .copied()
                        .find(|key| {
                            snapshot
                                .direct_projection_boundaries(*key)
                                .iter()
                                .all(|child| !pending.contains(child))
                        })
                        .expect("projection boundaries form an acyclic retained tree");
                    let subtree_builders = snapshot
                        .direct_projection_boundaries(ready)
                        .into_iter()
                        .filter_map(|key| {
                            self.projection_subtrees
                                .get(&key)
                                .cloned()
                                .map(|entity| (key, entity))
                        })
                        .map(|(key, entity)| {
                            let layout = snapshot
                                .projection_boundary_layout(key)
                                .expect("retained projection boundary has a style");
                            let build: NativeElementBuilder = Rc::new(move || {
                                entity.clone().cached(layout.clone()).into_any_element()
                            });
                            (key, build)
                        })
                        .collect();
                    let native_builders = self
                        .text_controls
                        .iter()
                        .filter(|(key, _)| {
                            snapshot.nearest_projection_boundary(**key) == Some(ready)
                        })
                        .map(|(key, state)| (*key, state.element_builder()))
                        .collect();
                    let widgets = all_widgets
                        .iter()
                        .filter(|widget| {
                            snapshot.nearest_projection_boundary(widget.key) == Some(ready)
                        })
                        .cloned()
                        .collect();
                    let state = GpuiProjectionBoundaryState {
                        root: ready,
                        revision: (
                            self.controller.projection_boundary_revision(ready),
                            self.projection_force_revision,
                        ),
                        snapshot: snapshot.clone(),
                        input: input.clone(),
                        focus: self.focus.clone(),
                        text_input: text_input.clone(),
                        native_builders,
                        subtree_builders,
                        text_selections: text_selections.clone(),
                        widgets,
                        native_widget_factories: self.native_widget_factories.clone(),
                    };
                    if let Some(boundary) = self.projection_subtrees.get(&ready) {
                        boundary.update(cx, |boundary, boundary_cx| {
                            boundary.synchronize(state, boundary_cx);
                        });
                    } else {
                        self.projection_subtrees
                            .insert(ready, cx.new(|_| GpuiProjectionBoundary::new(state)));
                    }
                    pending.remove(&ready);
                }

                if affected_boundaries.remove(&wabou_host_api::NodeKey::ROOT) {
                    let subtree_builders = snapshot
                        .direct_projection_boundaries(wabou_host_api::NodeKey::ROOT)
                        .into_iter()
                        .filter_map(|key| {
                            self.projection_subtrees
                                .get(&key)
                                .cloned()
                                .map(|entity| (key, entity))
                        })
                        .map(|(key, entity)| {
                            let layout = snapshot
                                .projection_boundary_layout(key)
                                .expect("retained projection boundary has a style");
                            let build: NativeElementBuilder = Rc::new(move || {
                                entity.clone().cached(layout.clone()).into_any_element()
                            });
                            (key, build)
                        })
                        .collect();
                    let native_builders = self
                        .text_controls
                        .iter()
                        .filter(|(key, _)| {
                            snapshot.nearest_projection_boundary(**key)
                                == Some(wabou_host_api::NodeKey::ROOT)
                        })
                        .map(|(key, state)| (*key, state.element_builder()))
                        .collect();
                    let root_widgets = all_widgets
                        .into_iter()
                        .filter(|widget| {
                            snapshot.nearest_projection_boundary(widget.key)
                                == Some(wabou_host_api::NodeKey::ROOT)
                        })
                        .collect();
                    let state = GpuiProjectionBoundaryState {
                        root: wabou_host_api::NodeKey::ROOT,
                        revision: (
                            self.controller
                                .projection_boundary_revision(wabou_host_api::NodeKey::ROOT),
                            self.projection_force_revision,
                        ),
                        snapshot,
                        input,
                        focus: self.focus.clone(),
                        text_input,
                        native_builders,
                        subtree_builders,
                        text_selections,
                        widgets: root_widgets,
                        native_widget_factories: self.native_widget_factories.clone(),
                    };
                    if let Some(boundary) = &self.projection_boundary {
                        boundary.update(cx, |boundary, boundary_cx| {
                            boundary.synchronize(state, boundary_cx);
                        });
                    } else {
                        self.projection_boundary =
                            Some(cx.new(|_| GpuiProjectionBoundary::new(state)));
                    }
                }
            }
        }
        let projection_boundary = self
            .projection_boundary
            .as_ref()
            .expect("projection boundary initialized before root composition")
            .clone();
        let projection_materializations = self.performance_hud.as_ref().map(|_| {
            projection_boundary.read(cx).materialization_count()
                + self
                    .projection_subtrees
                    .values()
                    .map(|boundary| boundary.read(cx).materialization_count())
                    .sum::<u64>()
        });
        let projected = if self.projection_subtrees.is_empty() {
            projection_boundary
                .cached(StyleRefinement::default().size_full())
                .into_any_element()
        } else {
            // GPUI's outer cached View freezes nested cached views as one unit.
            // Keep this lightweight composition root live when explicit child
            // boundaries exist; each child remains an independently retained
            // Entity and the root revision clocks still avoid structural work.
            projection_boundary.into_any_element()
        };
        let drag_view = cx.weak_entity();
        let drop_view = drag_view.clone();
        let leave_view = drag_view.clone();
        let mut root = div()
            .size_full()
            .relative()
            .on_drag_move(move |event: &DragMoveEvent<ExternalPaths>, _, cx| {
                let paths = event.drag(cx).paths().to_vec();
                let position = wabou_shell::Point {
                    x: event.event.position.x.into(),
                    y: event.event.position.y.into(),
                };
                let _ = drag_view.update(cx, |view, cx| {
                    view.handle_file_drag_move(&paths, position);
                    cx.notify();
                });
            })
            .on_drop(move |paths: &ExternalPaths, _, cx| {
                let paths = paths.paths().to_vec();
                let _ = drop_view.update(cx, |view, cx| {
                    view.finish_file_drag(Some(&paths));
                    cx.notify();
                });
            })
            .on_mouse_exit(move |_, _, cx| {
                let _ = leave_view.update(cx, |view, cx| {
                    view.finish_file_drag(None);
                    cx.notify();
                });
            })
            // The layer is intentionally painted last. Projected nodes stop
            // native bubbling after dispatching the exact JS target; placing
            // the window coordinator last lets its bubble listener observe a
            // drag first without changing Wabou's event-target contract.
            .child(projected)
            .child(TextSelectionLayer);
        if advance_runtime || boundary_dirty {
            self.controller.publish_frame_stats(
                frame_timing,
                frame_started.elapsed().as_secs_f64() * 1_000.0,
                (viewport_width, viewport_height),
            );
        }
        if let Some(hud) = &self.performance_hud {
            let stats = self.controller.frame_stats();
            hud.update(cx, |hud, hud_cx| {
                hud.update(
                    stats,
                    self.projection_boundary_revision,
                    projection_materializations.unwrap_or_default(),
                    hud_cx,
                );
            });
            root = root.child(hud.clone());
        }
        #[cfg(feature = "devtools")]
        if self.controller.debug_snapshot_needs_publish() {
            // Make structure and status observable immediately. Bounds are
            // finalized below by GPUI, so this provisional publication does
            // not retire the pending revision.
            self.controller.publish_provisional_debug_snapshot();
            // GPUI only finalizes element bounds after `Render::render`
            // returns. Publish on the following platform-frame boundary so
            // structure, resolved layout, focus, and timing all describe the
            // same completed retained revision.
            let view = cx.weak_entity();
            window.on_next_frame(move |_, cx| {
                let _ = view.update(cx, |view, _| {
                    view.controller.publish_debug_snapshot();
                });
            });
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{JsRuntime, runtime_session::RuntimeSession};
    use wabou_host_api::NodeKey;
    use wabou_protocol::{Frame, Op};
    use wabou_shell::gpui::{HeadlessAppContext, TestAppContext, VisualTestContext, px, size};
    use wabou_shell::{
        EffectId, EffectScope, OpenDialogRequest, WindowCreateRequest, WindowOptions,
    };

    fn test_controller() -> GpuiController {
        GpuiController::new(RuntimeSession::new(
            JsRuntime::new().expect("QuickJS runtime"),
            wabou_shell::initial_window_resource_key(0),
        ))
    }

    struct TextareaTestRoot(Entity<TextareaState>);

    impl Render for TextareaTestRoot {
        fn render(
            &mut self,
            _: &mut Window,
            _: &mut Context<Self>,
        ) -> impl wabou_shell::gpui::IntoElement {
            div().size_full().child(Textarea::new(&self.0))
        }
    }

    #[wabou_shell::gpui::test]
    fn controlled_text_sync_preserves_native_ime_composition(cx: &mut TestAppContext) {
        cx.update(gpui_base::init);
        let state_slot = Rc::new(std::cell::RefCell::new(None));
        let state_for_window = state_slot.clone();
        let window = cx.update(|cx| {
            cx.open_window(Default::default(), move |window, cx| {
                let state = cx.new(|cx| TextareaState::new(window, cx));
                state_for_window.replace(Some(state.clone()));
                cx.new(|_| TextareaTestRoot(state))
            })
            .expect("open textarea IME fixture")
        });
        let state = state_slot
            .borrow()
            .clone()
            .expect("textarea state created with the fixture window");
        let mut cx = VisualTestContext::from_window(window.into(), cx);

        cx.update(|window, cx| {
            state.update(cx, |state, cx| {
                state.set_value("hello ", window, cx);
                state.set_selected_range(6..6, cx);

                EntityInputHandler::replace_and_mark_text_in_range(
                    state,
                    None,
                    "n",
                    Some(1..1),
                    window,
                    cx,
                );
                assert_eq!(state.value(), "hello n");
                assert_eq!(state.selected_range(), 7..7);

                // Solid still owns the last committed value while macOS owns
                // the marked candidate. The stale controlled value must not
                // reset the native selection to the beginning.
                synchronize_controlled_text_value(state, "hello ", window, cx);
                assert_eq!(state.value(), "hello n");
                assert_eq!(state.selected_range(), 7..7);
                assert_eq!(
                    EntityInputHandler::marked_text_range(state, window, cx),
                    Some(6..7)
                );

                EntityInputHandler::replace_and_mark_text_in_range(
                    state,
                    None,
                    "ni",
                    Some(2..2),
                    window,
                    cx,
                );
                synchronize_controlled_text_value(state, "hello ", window, cx);
                assert_eq!(state.value(), "hello ni");
                assert_eq!(state.selected_range(), 8..8);

                EntityInputHandler::replace_text_in_range(state, None, "你", window, cx);
                synchronize_controlled_text_value(state, "hello 你", window, cx);
                assert_eq!(state.value(), "hello 你");
                assert_eq!(state.selected_range(), 9..9);
                assert_eq!(
                    EntityInputHandler::marked_text_range(state, window, cx),
                    None
                );
            });
        });
    }

    #[test]
    fn native_event_mutations_are_drained_by_the_notified_render() {
        let platform = gpui_platform::current_platform(true);
        let mut cx = HeadlessAppContext::new(platform.text_system());
        let target = NodeKey::new(2, 1);
        let handle = cx
            .open_window(size(px(320.0), px(200.0)), |window, app| {
                let mut controller = test_controller();
                controller
                    .boot(include_str!("gen/test-runtime.js"))
                    .expect("boot generated Solid runtime fixture");
                controller
                    .apply_frame(&Frame {
                        seq: 1,
                        ops: vec![
                            Op::CreateText {
                                id: target,
                                text: "before",
                            },
                            Op::AppendChild {
                                parent: NodeKey::ROOT,
                                child: target,
                            },
                            Op::AddEventListener {
                                id: target,
                                event_type: wabou_protocol::event::TRANSITIONEND,
                            },
                        ],
                    })
                    .expect("project transition target");
                assert!(controller.advance_frame(), "initial tree must commit");

                // seq=2, one SET_TEXT operation for target 2:1 and inline
                // string "after". The host-frame callback returns this frame
                // synchronously from the native transition completion.
                let protocol_frame = [
                    2,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    0,
                    wabou_protocol::op::SET_TEXT,
                    2,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    0,
                    5,
                    0,
                    b'a',
                    b'f',
                    b't',
                    b'e',
                    b'r',
                ];
                let bytes = protocol_frame
                    .iter()
                    .map(u8::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                controller
                    .eval_script_diagnostic(&format!(
                        r#"
                        globalThis.__wabou_dispatch_host_frame = () => ({{
                          needsTick: false,
                          preventedEventIds: new Uint32Array(),
                          protocolFrame: new Uint8Array([{bytes}]),
                        }});
                        "#
                    ))
                    .expect("install synchronous transition mutation");

                app.new(|view_cx| {
                    GpuiRuntimeView::new(
                        controller,
                        GpuiRuntimeViewOptions {
                            window_size_persistence: None,
                            native_widget_factories: HashMap::new(),
                            test_controller: None,
                            window_key: wabou_shell::initial_window_resource_key(0),
                            window_host: test_window_host(),
                        },
                        window,
                        view_cx,
                    )
                })
            })
            .expect("open native event fixture");
        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, app| {
            let _ = window.draw(app);
        })
        .expect("draw initial native event fixture");

        let root = handle.root(&mut cx).expect("GPUI runtime root entity");
        let revision_before = cx.read_entity(&root, |view, _| view.controller.protocol_revision());
        cx.update_entity(&root, |view, view_cx| {
            assert!(!view.runtime_work_pending);
            view.handle_input(
                wabou_shell::ProjectedInputEvent::TransitionEnd {
                    target,
                    generation: 7,
                },
                view_cx,
            );
            assert!(
                view.runtime_work_pending,
                "a handled native event must make its synchronous JS work visible to render"
            );
        });
        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, app| {
            let _ = window.draw(app);
        })
        .expect("draw the event-triggered Solid mutation");

        assert!(
            cx.read_entity(&root, |view, _| view.controller.protocol_revision()) > revision_before,
            "the notified render must commit the event's protocol frame without another input"
        );
    }

    #[test]
    fn explicit_projection_boundary_materializes_independently_from_root() {
        let platform = gpui_platform::current_platform(true);
        let mut cx = HeadlessAppContext::new(platform.text_system());
        let boundary_key = NodeKey::new(100, 1);
        let sibling_key = NodeKey::new(101, 1);
        let handle = cx
            .open_window(size(px(800.0), px(600.0)), |window, app| {
                let mut controller = test_controller();
                controller
                    .boot(include_str!("gen/test-runtime.js"))
                    .expect("boot generated Solid runtime fixture");
                assert!(controller.advance_frame());
                controller
                    .apply_frame(&Frame {
                        seq: 2,
                        ops: vec![
                            Op::CreateText {
                                id: boundary_key,
                                text: "before",
                            },
                            Op::AppendChild {
                                parent: NodeKey::ROOT,
                                child: boundary_key,
                            },
                            Op::SetProjectionBoundary {
                                id: boundary_key,
                                enabled: true,
                            },
                            Op::CreateText {
                                id: sibling_key,
                                text: "stable sibling",
                            },
                            Op::AppendChild {
                                parent: NodeKey::ROOT,
                                child: sibling_key,
                            },
                            Op::SetProjectionBoundary {
                                id: sibling_key,
                                enabled: true,
                            },
                        ],
                    })
                    .expect("create explicit projection boundary");
                app.new(|view_cx| {
                    GpuiRuntimeView::new(
                        controller,
                        GpuiRuntimeViewOptions {
                            window_size_persistence: None,
                            native_widget_factories: HashMap::new(),
                            test_controller: None,
                            window_key: wabou_shell::initial_window_resource_key(0),
                            window_host: test_window_host(),
                        },
                        window,
                        view_cx,
                    )
                })
            })
            .expect("open retained boundary fixture");
        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, app| {
            let _ = window.draw(app);
        })
        .expect("draw initial retained boundaries");

        let root = handle.root(&mut cx).expect("GPUI runtime root entity");
        let (root_boundary, child_boundary, sibling_boundary) = cx.read_entity(&root, |view, _| {
            (
                view.projection_boundary.clone().expect("root boundary"),
                view.projection_subtrees
                    .get(&boundary_key)
                    .cloned()
                    .expect("child boundary"),
                view.projection_subtrees
                    .get(&sibling_key)
                    .cloned()
                    .expect("sibling boundary"),
            )
        });
        let root_revision_before =
            cx.read_entity(&root_boundary, |boundary, _| boundary.revision());
        let (child_before, child_revision_before) = cx
            .read_entity(&child_boundary, |boundary, _| {
                (boundary.materialization_count(), boundary.revision())
            });
        let sibling_before = cx.read_entity(&sibling_boundary, |boundary, _| {
            boundary.materialization_count()
        });

        cx.update_entity(&root, |_, view_cx| view_cx.notify());
        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, app| {
            let _ = window.draw(app);
        })
        .expect("draw an unrelated root invalidation");
        assert_eq!(
            cx.read_entity(&child_boundary, |boundary, _| boundary
                .materialization_count()),
            child_before,
            "an unrelated root draw must reuse the cached child boundary"
        );

        cx.update_entity(&root, |view, view_cx| {
            view.controller
                .apply_frame(&Frame {
                    seq: 3,
                    ops: vec![Op::SetText {
                        id: boundary_key,
                        text: "after",
                    }],
                })
                .expect("mutate child boundary");
            assert!(view.controller.finish_projection_frame());
            view_cx.notify();
        });
        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, app| {
            let _ = window.draw(app);
        })
        .expect("synchronize updated retained boundary");
        cx.update_window(handle.into(), |_, window, app| {
            window.simulate_next_frame(app)
        })
        .expect("commit child boundary state at the GPUI frame boundary");
        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, app| {
            let _ = window.draw(app);
        })
        .expect("draw updated retained boundary");

        assert_eq!(
            cx.read_entity(&root_boundary, |boundary, _| boundary
                .revision()
                .0
                .structure),
            root_revision_before.0.structure,
            "child text mutation must not rebuild root structure"
        );
        assert!(
            cx.read_entity(&child_boundary, |boundary, _| {
                assert_ne!(boundary.revision(), child_revision_before);
                boundary.materialization_count()
            }) > child_before,
            "the owning child boundary must rematerialize"
        );
        assert_eq!(
            cx.read_entity(&sibling_boundary, |boundary, _| {
                boundary.materialization_count()
            }),
            sibling_before,
            "a local child mutation must not materialize a sibling boundary"
        );
    }

    #[test]
    fn native_editor_selection_offsets_round_trip_through_javascript_utf16() {
        let value = "a😀中b";
        for utf16 in [0, 1, 3, 4, 5] {
            let utf8 = utf16_to_utf8_offset(value, utf16);
            assert_eq!(utf8_to_utf16_offset(value, utf8), utf16);
        }
        assert_eq!(utf16_to_utf8_offset(value, 2), 1);
    }

    #[test]
    fn wabou_palette_projects_into_gpui_base_semantic_tokens() {
        let fallback = gpui_base::Theme::default();
        let snapshot = wabou_shell::GpuiThemeSnapshot {
            dark: true,
            colors: HashMap::from([
                ("canvas".into(), 0x1214_18ff),
                ("surface".into(), 0x1a1d_22ff),
                ("primary".into(), 0xf2f4_f7ff),
                ("control".into(), 0x2428_2fff),
                ("muted".into(), 0x8e97_a4ff),
                ("accent".into(), 0x4c8d_ffff),
                ("on-accent".into(), 0x1214_18ff),
                ("subtle".into(), 0x3035_3dff),
                ("input".into(), 0x2024_2aff),
                ("focus".into(), 0x74a8_ffff),
            ]),
        };

        let projected = project_base_theme(&snapshot, fallback);
        let hsla = |rgba| wabou_shell::gpui::rgb_to_hsla(wabou_shell::gpui::rgba(rgba));
        assert_eq!(projected.appearance, gpui_base::ThemeAppearance::Dark);
        assert_eq!(projected.tokens.colors.background, hsla(0x1214_18ff));
        assert_eq!(projected.tokens.colors.surface, hsla(0x1a1d_22ff));
        assert_eq!(projected.tokens.colors.foreground, hsla(0xf2f4_f7ff));
        assert_eq!(projected.tokens.colors.primary, hsla(0x4c8d_ffff));
        assert_eq!(
            projected.tokens.colors.primary_foreground,
            hsla(0x1214_18ff)
        );
        assert_eq!(projected.tokens.colors.secondary, hsla(0x2428_2fff));
        assert_eq!(projected.tokens.colors.muted_foreground, hsla(0x8e97_a4ff));
        assert_eq!(projected.tokens.colors.border, hsla(0x3035_3dff));
        assert_eq!(projected.tokens.colors.input, hsla(0x2024_2aff));
        assert_eq!(projected.tokens.colors.ring, hsla(0x74a8_ffff));
    }

    fn test_window_host() -> std::rc::Rc<crate::gpui_windows::GpuiApplicationWindows> {
        crate::gpui_windows::GpuiApplicationWindows::new(
            std::rc::Rc::new(|_, _| Ok(test_controller())),
            HashMap::new(),
            None,
        )
    }

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
        let mut controller = test_controller();
        controller
            .boot(include_str!("gen/test-runtime.js"))
            .expect("boot generated Solid runtime fixture");

        assert!(controller.advance_frame());
        assert_eq!(controller.protocol_revision(), 1);
        assert!(
            controller.contains(NodeKey::new(2, 1)),
            "the fixture's mounted <main> must cross the binary writer boundary"
        );
        let _root = controller
            .projection()
            .tree_element(NodeKey::ROOT)
            .expect("the completed Solid tree must materialize for GPUI");
    }

    #[test]
    fn solid_frame_draws_in_a_real_platform_headless_window() {
        let platform = gpui_platform::current_platform(true);
        let mut cx = HeadlessAppContext::new(platform.text_system());
        #[cfg(feature = "devtools")]
        let debug_state = wabou_devtools::DebugState::shared();
        #[cfg(feature = "devtools")]
        let view_debug_state = debug_state.clone();
        let handle = cx
            .open_window(size(px(800.0), px(600.0)), |window, app| {
                let mut controller = test_controller();
                #[cfg(feature = "devtools")]
                controller.set_debug_state(view_debug_state);
                controller
                    .boot(include_str!("gen/test-runtime.js"))
                    .expect("boot generated Solid runtime fixture");
                assert!(controller.advance_frame());
                app.new(|view_cx| {
                    GpuiRuntimeView::new(
                        controller,
                        GpuiRuntimeViewOptions {
                            window_size_persistence: None,
                            native_widget_factories: HashMap::new(),
                            test_controller: None,
                            window_key: wabou_shell::initial_window_resource_key(0),
                            window_host: test_window_host(),
                        },
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
            #[cfg(feature = "devtools")]
            assert!(window.simulate_next_frame(app) >= 1);
        })
        .expect("layout and draw the projected Solid frame");
        let root = handle.root(&mut cx).expect("GPUI runtime root entity");
        let snapshot = cx.read_entity(&root, |view, _| view.layout_snapshot());
        assert!(
            snapshot.iter().any(|node| node.key == NodeKey::new(2, 1)),
            "the real GPUI prepaint pass must publish bounds for the Solid fixture"
        );
        #[cfg(feature = "devtools")]
        {
            let state = debug_state.read().expect("debug state");
            let snapshot = state.snapshot();
            assert!(snapshot.status.revision > 1);
            assert_eq!(snapshot.status.node_count, snapshot.nodes.len());
            assert!(
                snapshot
                    .nodes
                    .iter()
                    .any(|node| { node.id == NodeKey::new(2, 1) && node.rect.width > 0.0 })
            );
        }
    }

    #[test]
    fn animation_only_frames_reuse_the_cached_solid_projection_boundary() {
        let platform = gpui_platform::current_platform(true);
        let mut cx = HeadlessAppContext::new(platform.text_system());
        let handle = cx
            .open_window(size(px(800.0), px(600.0)), |window, app| {
                let mut controller = test_controller();
                controller
                    .boot(include_str!("gen/test-runtime.js"))
                    .expect("boot generated Solid runtime fixture");
                assert!(controller.advance_frame());
                controller
                    .eval_script_diagnostic(
                        "requestAnimationFrame(function tick() { requestAnimationFrame(tick); })",
                    )
                    .expect("schedule an animation with no Solid mutations");
                app.new(|view_cx| {
                    GpuiRuntimeView::new(
                        controller,
                        GpuiRuntimeViewOptions {
                            window_size_persistence: None,
                            native_widget_factories: HashMap::new(),
                            test_controller: None,
                            window_key: wabou_shell::initial_window_resource_key(0),
                            window_host: test_window_host(),
                        },
                        window,
                        view_cx,
                    )
                })
            })
            .expect("open animated GPUI projection");
        cx.run_until_parked();

        let root = handle.root(&mut cx).expect("GPUI runtime root entity");
        let boundary = cx.read_entity(&root, |view, _| {
            view.projection_boundary
                .clone()
                .expect("projection boundary created by the first render")
        });
        let initial = cx.read_entity(&boundary, |boundary, _| boundary.materialization_count());
        assert_eq!(initial, 1);

        for _ in 0..3 {
            let callbacks = cx
                .update_window(handle.into(), |_, window, app| {
                    window.simulate_next_frame(app)
                })
                .expect("simulate one native animation frame");
            assert!(callbacks >= 1);
            cx.run_until_parked();
        }

        assert_eq!(
            cx.read_entity(&boundary, |boundary, _| {
                boundary.materialization_count()
            }),
            initial,
            "animation frames without Solid mutations must reuse the cached projection"
        );
    }

    #[test]
    fn performance_hud_updates_do_not_materialize_the_solid_projection() {
        let platform = gpui_platform::current_platform(true);
        let mut cx = HeadlessAppContext::new(platform.text_system());
        let handle = cx
            .open_window(size(px(800.0), px(600.0)), |window, app| {
                let mut controller = test_controller();
                controller
                    .boot(include_str!("gen/test-runtime.js"))
                    .expect("boot generated Solid runtime fixture");
                assert!(controller.advance_frame());
                controller
                    .eval_script_diagnostic(
                        r#"
                        globalThis.__wabou_test_tick_count = 0;
                        const originalTick = globalThis.__wabou_tick;
                        globalThis.__wabou_tick = (time) => {
                          globalThis.__wabou_test_tick_count += 1;
                          return originalTick(time);
                        };
                        "#,
                    )
                    .expect("instrument QuickJS animation ticks");
                app.new(|view_cx| {
                    let mut view = GpuiRuntimeView::new(
                        controller,
                        GpuiRuntimeViewOptions {
                            window_size_persistence: None,
                            native_widget_factories: HashMap::new(),
                            test_controller: None,
                            window_key: wabou_shell::initial_window_resource_key(0),
                            window_host: test_window_host(),
                        },
                        window,
                        view_cx,
                    );
                    view.performance_hud = Some(view_cx.new(|_| GpuiPerformanceHud::new()));
                    view
                })
            })
            .expect("open GPUI projection with performance HUD");
        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, app| {
            let _ = window.draw(app);
        })
        .expect("draw initial projected frame");

        let root = handle.root(&mut cx).expect("GPUI runtime root entity");
        let (boundary, hud) = cx.read_entity(&root, |view, _| {
            (
                view.projection_boundary
                    .clone()
                    .expect("projection boundary"),
                view.performance_hud.clone().expect("performance HUD"),
            )
        });
        let initial = cx.read_entity(&boundary, |boundary, _| boundary.materialization_count());
        let initial_hud_render_count = cx.read_entity(&hud, |hud, _| hud.render_count());
        let initial_tick_count = cx.read_entity(&root, |view, _| {
            view.controller
                .eval_string("String(globalThis.__wabou_test_tick_count)")
                .expect("read initial QuickJS tick count")
        });
        cx.update_entity(&hud, |hud, hud_cx| {
            hud.update(
                Some(wabou_shell::FrameStats {
                    node_count: 3,
                    viewport_w: 800,
                    viewport_h: 600,
                    ..wabou_shell::FrameStats::default()
                }),
                2,
                initial,
                hud_cx,
            );
        });
        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, app| {
            let _ = window.draw(app);
        })
        .expect("draw updated HUD frame");
        let callbacks = cx
            .update_window(handle.into(), |_, window, app| {
                window.simulate_next_frame(app)
            })
            .expect("simulate HUD-owned animation frame");
        assert!(
            callbacks >= 1,
            "the native HUD must schedule one post-draw sample"
        );
        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, app| {
            let _ = window.draw(app);
        })
        .expect("draw HUD-owned animation frame");
        assert!(
            !cx.read_entity(&hud, |hud, _| hud.sample_after_draw_pending()),
            "the native HUD must consume its one post-mount sample request"
        );

        assert_eq!(
            cx.read_entity(&boundary, |boundary, _| {
                boundary.materialization_count()
            }),
            initial,
            "native HUD updates must stay outside the Solid projection boundary"
        );
        assert!(
            cx.read_entity(&hud, |hud, _| hud.render_count()) > initial_hud_render_count,
            "the native HUD must render again on its own frame clock"
        );
        assert_eq!(
            cx.read_entity(&root, |view, _| {
                view.controller
                    .eval_string("String(globalThis.__wabou_test_tick_count)")
                    .expect("read final QuickJS tick count")
            }),
            initial_tick_count,
            "native HUD frames must not poll an idle QuickJS runtime"
        );
    }

    #[test]
    fn gpui_window_commands_are_routed_to_the_registered_target() {
        let platform = gpui_platform::current_platform(true);
        let mut cx = HeadlessAppContext::new(platform.text_system());
        let window_host = test_window_host();
        let first_key = window_host.reserve();
        let second_key = window_host.reserve();

        let first_window_host = window_host.clone();
        let first = cx
            .open_window(size(px(800.0), px(600.0)), move |window, app| {
                app.new(|view_cx| {
                    GpuiRuntimeView::new(
                        test_controller(),
                        GpuiRuntimeViewOptions {
                            window_size_persistence: None,
                            native_widget_factories: HashMap::new(),
                            test_controller: None,
                            window_key: first_key,
                            window_host: first_window_host,
                        },
                        window,
                        view_cx,
                    )
                })
            })
            .expect("open first GPUI window");
        assert!(window_host.attach(first_key, first.into()));

        let second_window_host = window_host.clone();
        let second = cx
            .open_window(size(px(640.0), px(480.0)), move |window, app| {
                app.new(|view_cx| {
                    GpuiRuntimeView::new(
                        test_controller(),
                        GpuiRuntimeViewOptions {
                            window_size_persistence: None,
                            native_widget_factories: HashMap::new(),
                            test_controller: None,
                            window_key: second_key,
                            window_host: second_window_host,
                        },
                        window,
                        view_cx,
                    )
                })
            })
            .expect("open second GPUI window");
        assert!(window_host.attach(second_key, second.into()));

        let first_root = first.root(&mut cx).expect("first GPUI runtime root");
        let result = cx
            .update_window(first.into(), |_, window, app| {
                first_root.update(app, |view, view_cx| {
                    view.execute_window_command(second_key, WindowCommand::Close, window, view_cx)
                })
            })
            .expect("update first GPUI window");

        assert_eq!(result, EffectResult::Unit);
        assert!(
            window_host.resolve(first_key).is_some(),
            "routing a command must not affect the issuing window"
        );
        assert!(
            window_host.resolve(second_key).is_none(),
            "closing a target must retire its Wabou window identity"
        );
        cx.run_until_parked();
        assert!(
            cx.update_window(second.into(), |_, _, _| ()).is_err(),
            "the registered target, rather than the issuing window, must be closed"
        );
    }

    #[test]
    fn native_gpui_close_retires_the_wabou_window_identity() {
        let platform = gpui_platform::current_platform(true);
        let mut cx = HeadlessAppContext::new(platform.text_system());
        let window_host = test_window_host();
        let window_key = window_host.reserve();
        let view_window_host = window_host.clone();
        let handle = cx
            .open_window(size(px(640.0), px(480.0)), move |window, app| {
                app.new(|view_cx| {
                    GpuiRuntimeView::new(
                        test_controller(),
                        GpuiRuntimeViewOptions {
                            window_size_persistence: None,
                            native_widget_factories: HashMap::new(),
                            test_controller: None,
                            window_key,
                            window_host: view_window_host,
                        },
                        window,
                        view_cx,
                    )
                })
            })
            .expect("open GPUI window");
        assert!(window_host.attach(window_key, handle.into()));
        let _close_subscription = cx.update(|app| window_host.observe_native_closes(app));

        cx.update_window(handle.into(), |_, window, _| window.remove_window())
            .expect("close GPUI window through the native lifecycle");
        cx.run_until_parked();

        assert!(
            window_host.resolve(window_key).is_none(),
            "native close notifications must retire the public generational key"
        );
    }

    #[test]
    fn gpui_window_create_effect_opens_an_independent_registered_runtime() {
        let platform = gpui_platform::current_platform(true);
        let mut cx = HeadlessAppContext::new(platform.text_system());
        let window_host = test_window_host();
        let root_key = window_host.reserve();
        let root_window_host = window_host.clone();
        let root = cx
            .open_window(size(px(800.0), px(600.0)), move |window, app| {
                app.new(|view_cx| {
                    GpuiRuntimeView::new(
                        test_controller(),
                        GpuiRuntimeViewOptions {
                            window_size_persistence: None,
                            native_widget_factories: HashMap::new(),
                            test_controller: None,
                            window_key: root_key,
                            window_host: root_window_host,
                        },
                        window,
                        view_cx,
                    )
                })
            })
            .expect("open root GPUI window");
        assert!(window_host.attach(root_key, root.into()));
        let root_view = root.root(&mut cx).expect("root GPUI runtime view");

        let completion = cx
            .update_window(root.into(), |_, window, app| {
                root_view.update(app, |view, view_cx| {
                    view.execute_effect(
                        EffectRequest {
                            id: EffectId(4),
                            scope: EffectScope::Runtime,
                            payload: EffectPayload::WindowCreate(WindowCreateRequest {
                                options: WindowOptions::new()
                                    .title("Created by JavaScript")
                                    .initial_inner_size(420, 280),
                            }),
                        },
                        window,
                        view_cx,
                    )
                    .expect("window creation completes synchronously")
                })
            })
            .expect("dispatch create-window effect");
        let EffectResult::Window(child_key) = completion.result else {
            panic!("expected a window resource, got {:?}", completion.result);
        };
        let child = window_host
            .resolve(child_key)
            .expect("created window must be registered");

        cx.run_until_parked();
        cx.update_window(child, |_, window, _| {
            assert_eq!(window.bounds().size, size(px(420.0), px(280.0)));
        })
        .expect("inspect created GPUI window");
        assert!(
            window_host.resolve(root_key).is_some(),
            "creating a child must preserve the source runtime"
        );
    }

    #[test]
    fn gpui_window_create_rejects_unsupported_semantics_without_reserving_a_key() {
        let platform = gpui_platform::current_platform(true);
        let mut cx = HeadlessAppContext::new(platform.text_system());
        let window_host = test_window_host();
        let root_key = window_host.reserve();
        let root_window_host = window_host.clone();
        let root = cx
            .open_window(size(px(800.0), px(600.0)), move |window, app| {
                app.new(|view_cx| {
                    GpuiRuntimeView::new(
                        test_controller(),
                        GpuiRuntimeViewOptions {
                            window_size_persistence: None,
                            native_widget_factories: HashMap::new(),
                            test_controller: None,
                            window_key: root_key,
                            window_host: root_window_host,
                        },
                        window,
                        view_cx,
                    )
                })
            })
            .expect("open root GPUI window");
        assert!(window_host.attach(root_key, root.into()));
        let root_view = root.root(&mut cx).expect("root GPUI runtime view");

        let completion = cx
            .update_window(root.into(), |_, window, app| {
                root_view.update(app, |view, view_cx| {
                    view.execute_effect(
                        EffectRequest {
                            id: EffectId(5),
                            scope: EffectScope::Runtime,
                            payload: EffectPayload::WindowCreate(WindowCreateRequest {
                                options: WindowOptions::new()
                                    .input_mode(wabou_shell::WindowInputMode::Passthrough),
                            }),
                        },
                        window,
                        view_cx,
                    )
                    .expect("window creation rejection completes synchronously")
                })
            })
            .expect("dispatch unsupported create-window effect");
        assert!(matches!(
            completion.result,
            EffectResult::Error {
                code: EffectErrorCode::Unsupported,
                ..
            }
        ));
        assert_eq!(
            window_host.reserve(),
            wabou_shell::initial_window_resource_key(1),
            "validation must happen before reserving a public window key"
        );
    }

    #[wabou_shell::gpui::test]
    fn gpui_window_snapshot_uses_logical_viewport_and_native_scale(cx: &mut TestAppContext) {
        let controller = test_controller();
        let (view, cx) = cx.add_window_view(move |window, cx| {
            GpuiRuntimeView::new(
                controller,
                GpuiRuntimeViewOptions {
                    window_size_persistence: None,
                    native_widget_factories: HashMap::new(),
                    test_controller: None,
                    window_key: wabou_shell::initial_window_resource_key(0),
                    window_host: test_window_host(),
                },
                window,
                cx,
            )
        });

        let metrics = cx.update(|window, app| {
            app.set_reduce_motion(true);
            window.resize(size(px(640.0), px(360.0)));
            window.set_scale_factor(2.0);
            window.bounds_changed(app);
            view.read(app).window_metrics(window, app)
        });
        assert_eq!(metrics.logical_width, 640);
        assert_eq!(metrics.logical_height, 360);
        assert_eq!(metrics.physical_width, 1280);
        assert_eq!(metrics.physical_height, 720);
        assert_eq!(metrics.scale_factor, 2.0);
        assert!(metrics.reduced_motion);
    }

    #[wabou_shell::gpui::test]
    fn gpui_effect_executor_uses_the_platform_clipboard(cx: &mut TestAppContext) {
        let controller = test_controller();
        let (view, cx) = cx.add_window_view(move |window, cx| {
            GpuiRuntimeView::new(
                controller,
                GpuiRuntimeViewOptions {
                    window_size_persistence: None,
                    native_widget_factories: HashMap::new(),
                    test_controller: None,
                    window_key: wabou_shell::initial_window_resource_key(0),
                    window_host: test_window_host(),
                },
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

    #[wabou_shell::gpui::test]
    fn gpui_effect_executor_uses_the_platform_path_prompt(cx: &mut TestAppContext) {
        let controller = test_controller();
        let (view, cx) = cx.add_window_view(move |window, cx| {
            GpuiRuntimeView::new(
                controller,
                GpuiRuntimeViewOptions {
                    window_size_persistence: None,
                    native_widget_factories: HashMap::new(),
                    test_controller: None,
                    window_key: wabou_shell::initial_window_resource_key(0),
                    window_host: test_window_host(),
                },
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
        let (_, results) = message_prompt_buttons(wabou_shell::MessageDialogButtons::YesNoCancel);
        assert_eq!(results, ["yes", "no", "cancel"]);
        assert!(matches!(
            message_prompt_level(wabou_shell::MessageDialogLevel::Error),
            PromptLevel::Critical
        ));
    }
}
