//! Test-only bridge between QuickJS scenarios and the native event loop.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use rquickjs::{Function, prelude::Async};
use serde::Deserialize;
use tokio::sync::oneshot;
use wabou_shell::window_lifecycle::{WindowCapabilities, WindowLifecycle, WindowPresence};
use wabou_shell::{
    ExtensionContext, ImeEvent, KeyEvent, KeyLocation, KeyPhase, ShellExtension, WakeCallback,
    WheelEvent,
};
use wabou_shell::{
    FrameSource, Modifiers, Point, PointerButton, PointerEvent, PointerPhase, SemanticRole,
    SemanticSnapshot, UiEvent,
};

const CAPABILITY: &str = "test";

#[derive(Debug)]
enum TestActionKind {
    WaitForIdle(u64),
    NativeClose {
        window_id: u64,
        mutable_visibility: bool,
    },
    ShowWindow(u64),
    ClickByRole {
        window_id: u64,
        role: String,
        label: String,
        index: Option<usize>,
    },
    InputByRole {
        window_id: u64,
        role: String,
        label: String,
        input: TestInput,
        index: Option<usize>,
    },
    QueryByRole {
        window_id: u64,
        role: String,
        label: String,
        index: Option<usize>,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum TestInput {
    Probe,
    Drag { delta_x: f64, delta_y: f64 },
    Key { key: String, modifiers: u8 },
    Text { text: String },
    Paste { text: String },
    Ime { text: String },
    Wheel { delta_x: f64, delta_y: f64 },
}

#[derive(Debug)]
struct TestAction {
    kind: TestActionKind,
    completion: oneshot::Sender<TestActionResult>,
}

#[derive(Debug)]
enum TestActionResult {
    Handled(bool),
    Query(Option<String>),
}

#[derive(Clone, Copy, Debug)]
struct WindowSnapshot {
    lifecycle: WindowLifecycle,
}

#[derive(Default)]
struct TestState {
    actions: VecDeque<TestAction>,
    windows: HashMap<u64, WindowSnapshot>,
    wake: Option<WakeCallback>,
    report: Option<String>,
    semantic_snapshots: HashMap<u64, Arc<SemanticSnapshot>>,
    headless: bool,
}

#[derive(Clone, Default)]
pub(crate) struct TestController(Arc<Mutex<TestState>>);

impl TestController {
    fn request(&self, kind: TestActionKind) -> oneshot::Receiver<TestActionResult> {
        let (completion, receiver) = oneshot::channel();
        let wake = {
            let Ok(mut state) = self.0.lock() else {
                return receiver;
            };
            if state.report.is_some() {
                let _ = completion.send(cancelled_result(&kind));
                return receiver;
            }
            let action = TestAction { kind, completion };
            if state.headless {
                if action_requires_source_poll(&action.kind) {
                    state.actions.push_back(action);
                } else {
                    apply_headless_action(&mut state, action);
                }
            } else {
                state.actions.push_back(action);
            }
            state.wake.clone()
        };
        if let Some(wake) = wake {
            wake();
        }
        receiver
    }

    fn finish(&self, report: String) -> bool {
        let wake = {
            let Ok(mut state) = self.0.lock() else {
                return false;
            };
            if state.report.is_some() {
                return false;
            }
            state.report = Some(report);
            for action in state.actions.drain(..) {
                let _ = action.completion.send(cancelled_result(&action.kind));
            }
            state.wake.clone()
        };
        if let Some(wake) = wake {
            wake();
        }
        true
    }

    pub(crate) fn mount(&self, js: &crate::JsRuntime) -> rquickjs::Result<()> {
        let controller = self.clone();
        js.mount_capability(CAPABILITY, move |ctx, capability| {
            let native_close = controller.clone();
            capability.set(
                "nativeClose",
                Function::new(
                    ctx.clone(),
                    Async(move |window_id: u64, mutable_visibility: bool| {
                        let receiver = native_close.request(TestActionKind::NativeClose {
                            window_id,
                            mutable_visibility,
                        });
                        async move { matches!(receiver.await, Ok(TestActionResult::Handled(true))) }
                    }),
                )?,
            )?;

            let show = controller.clone();
            capability.set(
                "showWindow",
                Function::new(
                    ctx.clone(),
                    Async(move |window_id: u64| {
                        let receiver = show.request(TestActionKind::ShowWindow(window_id));
                        async move { matches!(receiver.await, Ok(TestActionResult::Handled(true))) }
                    }),
                )?,
            )?;

            let idle = controller.clone();
            capability.set(
                "waitForIdle",
                Function::new(
                    ctx.clone(),
                    Async(move |window_id: u64| {
                        let receiver = idle.request(TestActionKind::WaitForIdle(window_id));
                        async move { matches!(receiver.await, Ok(TestActionResult::Handled(true))) }
                    }),
                )?,
            )?;

            let query = controller.clone();
            capability.set(
                "windowState",
                Function::new(ctx.clone(), move |window_id: u64| {
                    query.window_state_json(window_id)
                })?,
            )?;

            let click = controller.clone();
            capability.set(
                "clickByRole",
                Function::new(
                    ctx.clone(),
                    Async(move |window_id: u64, role: String, label: String, index: Option<usize>| {
                        let receiver = click.request(TestActionKind::ClickByRole {
                            window_id,
                            role,
                            label,
                            index,
                        });
                        async move { matches!(receiver.await, Ok(TestActionResult::Handled(true))) }
                    }),
                )?,
            )?;

            let input = controller.clone();
            capability.set(
                "inputByRole",
                Function::new(
                    ctx.clone(),
                    Async(
                        move |window_id: u64,
                              role: String,
                              label: String,
                              raw: String,
                              index: Option<usize>| {
                            let receiver =
                                serde_json::from_str::<TestInput>(&raw).ok().map(|action| {
                                    input.request(TestActionKind::InputByRole {
                                        window_id,
                                        role,
                                        label,
                                        input: action,
                                        index,
                                    })
                                });
                            async move {
                                match receiver {
                                    Some(receiver) => matches!(
                                        receiver.await,
                                        Ok(TestActionResult::Handled(true))
                                    ),
                                    None => false,
                                }
                            }
                        },
                    ),
                )?,
            )?;

            let finish = controller.clone();
            capability.set(
                "finish",
                Function::new(ctx.clone(), move |report: String| finish.finish(report))?,
            )?;

            let query = controller.clone();
            capability.set(
                "queryByRole",
                Function::new(
                    ctx.clone(),
                    Async(
                        move |window_id: u64, role: String, label: String, index: Option<usize>| {
                            let receiver = query.request(TestActionKind::QueryByRole {
                                window_id,
                                role,
                                label,
                                index,
                            });
                            async move {
                                match receiver.await {
                                    Ok(TestActionResult::Query(result)) => result,
                                    _ => None,
                                }
                            }
                        },
                    ),
                )?,
            )?;
            Ok(())
        })
    }

    fn window_state_json(&self, window_id: u64) -> String {
        let snapshot = self
            .0
            .lock()
            .ok()
            .and_then(|state| state.windows.get(&window_id).copied());
        let Some(snapshot) = snapshot else {
            return "null".into();
        };
        let presence = match snapshot.lifecycle.presence() {
            WindowPresence::Visible => "visible",
            WindowPresence::Hidden => "hidden",
            WindowPresence::SurfaceReleased => "surface-released",
            WindowPresence::Closed => "closed",
        };
        format!(
            r#"{{"presence":"{presence}","surfaceGeneration":{}}}"#,
            snapshot.lifecycle.surface_generation()
        )
    }

    pub(crate) fn take_report(&self) -> Option<String> {
        self.0.lock().ok()?.report.take()
    }

    pub(crate) fn initialize_headless(&self, window_ids: impl IntoIterator<Item = u64>) {
        if let Ok(mut state) = self.0.lock() {
            state.headless = true;
            for window_id in window_ids {
                state.windows.insert(
                    window_id,
                    WindowSnapshot {
                        lifecycle: WindowLifecycle::visible(),
                    },
                );
            }
        }
    }

    pub(crate) fn poll_headless_source(&self, window_id: u64, source: &mut dyn FrameSource) {
        let snapshot = source.semantic_snapshot();
        if let Some(snapshot) = snapshot.as_ref() {
            self.record_semantic_snapshot(window_id, snapshot.clone());
        }
        if let Some(snapshot) = snapshot.as_deref()
            && let Some(action) = self.0.lock().ok().and_then(|state| {
                state.actions.iter().find_map(|action| match &action.kind {
                    TestActionKind::ClickByRole {
                        window_id: target_window,
                        role,
                        label,
                        index,
                    } if *target_window == window_id => {
                        semantic_target(snapshot, role, label, *index)
                    }
                    _ => None,
                })
            })
            && source.handle_semantic_action(wabou_shell::SemanticAction::ScrollIntoView {
                target: action.id,
            })
        {
            return;
        }
        let action = self.0.lock().ok().and_then(|mut state| {
            let index = state.actions.iter().position(|action| {
                matches!(
                    action_window_id(&action.kind),
                    Some(target) if target == window_id && action_requires_source_poll(&action.kind)
                )
            })?;
            let action = state.actions.get(index)?;
            let ready = action_ready(&action.kind, snapshot.as_deref());
            ready.then(|| state.actions.remove(index)).flatten()
        });
        let Some(action) = action else {
            return;
        };
        let handled = match (&action.kind, snapshot.as_deref()) {
            (TestActionKind::WaitForIdle(_), _) => true,
            (
                TestActionKind::ClickByRole {
                    role, label, index, ..
                },
                Some(snapshot),
            ) => click_semantic_target(source, snapshot, role, label, *index),
            (
                TestActionKind::InputByRole {
                    role,
                    label,
                    input,
                    index,
                    ..
                },
                Some(snapshot),
            ) => input_semantic_target(source, snapshot, role, label, input, *index),
            _ => false,
        };
        let result = match (&action.kind, snapshot.as_deref()) {
            (
                TestActionKind::QueryByRole {
                    role, label, index, ..
                },
                Some(snapshot),
            ) => TestActionResult::Query(locator_query_json(snapshot, role, label, *index)),
            _ => TestActionResult::Handled(handled),
        };
        let _ = action.completion.send(result);
    }

    pub(crate) fn has_report(&self) -> bool {
        self.0.lock().is_ok_and(|state| state.report.is_some())
    }

    pub(crate) fn report_passed(&self) -> Option<bool> {
        let state = self.0.lock().ok()?;
        let report = state.report.as_deref()?;
        serde_json::from_str::<serde_json::Value>(report)
            .ok()?
            .get("passed")?
            .as_bool()
    }

    fn record_semantic_snapshot(&self, window_id: u64, snapshot: Arc<SemanticSnapshot>) {
        if let Ok(mut state) = self.0.lock() {
            state.semantic_snapshots.insert(window_id, snapshot);
        }
    }

    pub(crate) fn semantic_artifact(&self) -> serde_json::Value {
        let snapshots = self
            .0
            .lock()
            .map(|state| state.semantic_snapshots.clone())
            .unwrap_or_default();
        let mut windows = snapshots.into_iter().collect::<Vec<_>>();
        windows.sort_unstable_by_key(|(window_id, _)| *window_id);
        serde_json::json!({
            "version": 1,
            "windows": windows
                .into_iter()
                .map(|(window_id, snapshot)| semantic_snapshot_json(window_id, &snapshot))
                .collect::<Vec<_>>(),
        })
    }
}

fn cancelled_result(kind: &TestActionKind) -> TestActionResult {
    if matches!(kind, TestActionKind::QueryByRole { .. }) {
        TestActionResult::Query(None)
    } else {
        TestActionResult::Handled(false)
    }
}

fn locator_snapshot_json(node: &wabou_shell::SemanticNode, focused: bool) -> String {
    let toggle_value = |state: Option<wabou_shell::SemanticToggleState>| match state {
        Some(wabou_shell::SemanticToggleState::Off) => serde_json::Value::Bool(false),
        Some(wabou_shell::SemanticToggleState::On) => serde_json::Value::Bool(true),
        Some(wabou_shell::SemanticToggleState::Mixed) => serde_json::Value::String("mixed".into()),
        None => serde_json::Value::Null,
    };
    serde_json::json!({
            "name": node.label,
            "value": node.value,
            "numericValue": node.numeric_value,
            "minNumericValue": node.min_numeric_value,
            "maxNumericValue": node.max_numeric_value,
            "bounds": {
                "x": node.bounds[0],
                "y": node.bounds[1],
                "width": node.bounds[2] - node.bounds[0],
                "height": node.bounds[3] - node.bounds[1],
            },
            "disabled": node.disabled,
            "checked": toggle_value(node.states.checked),
            "pressed": toggle_value(node.states.pressed),
            "selected": node.states.selected,
            "expanded": node.states.expanded,
            "focused": focused,
    })
    .to_string()
}

fn locator_query_json(
    snapshot: &SemanticSnapshot,
    role: &str,
    label: &str,
    index: Option<usize>,
) -> Option<String> {
    let role = SemanticRole::from_name(role)?;
    let matches = snapshot
        .exposed_nodes()
        .into_iter()
        .filter(|node| node.role == role && node.label.as_deref() == Some(label))
        .collect::<Vec<_>>();
    if matches.is_empty() {
        return None;
    }
    let selected = match index {
        Some(index) => matches.get(index).copied(),
        None if matches.len() == 1 => matches.first().copied(),
        None => matches.first().copied(),
    };
    let locator = selected.map(|node| {
        serde_json::from_str::<serde_json::Value>(&locator_snapshot_json(
            node,
            snapshot.focus == Some(node.id),
        ))
        .expect("locator snapshots are generated from serializable values")
    });
    Some(
        serde_json::json!({
            "matchCount": matches.len(),
            "snapshot": locator,
        })
        .to_string(),
    )
}

fn semantic_snapshot_json(window_id: u64, snapshot: &SemanticSnapshot) -> serde_json::Value {
    let toggle = |state: Option<wabou_shell::SemanticToggleState>| match state {
        Some(wabou_shell::SemanticToggleState::Off) => serde_json::Value::Bool(false),
        Some(wabou_shell::SemanticToggleState::On) => serde_json::Value::Bool(true),
        Some(wabou_shell::SemanticToggleState::Mixed) => serde_json::Value::String("mixed".into()),
        None => serde_json::Value::Null,
    };
    serde_json::json!({
        "windowId": window_id,
        "revision": snapshot.revision,
        "rootChildren": snapshot.root_children,
        "focus": snapshot.focus,
        "modalRoot": snapshot.modal_root,
        "nodes": snapshot.nodes.iter().map(|node| serde_json::json!({
            "id": node.id,
            "role": node.role.as_str(),
            // Generic containers inherit concatenated descendant text for
            // accessibility fallback. That can be enormous and is not a role
            // addressable by @wabou/test, so it adds noise rather than useful
            // locator evidence here.
            "name": if node.role == SemanticRole::Generic {
                None
            } else {
                node.label.as_deref()
            },
            // Values can contain credentials or application data. The shape is
            // useful for diagnostics, but the contents do not belong in an
            // automatically-created failure artifact.
            "hasValue": node.value.is_some(),
            "bounds": node.bounds,
            "children": node.children,
            "controls": node.controls,
            "activeDescendant": node.active_descendant,
            "disabled": node.disabled,
            "checked": toggle(node.states.checked),
            "pressed": toggle(node.states.pressed),
            "selected": node.states.selected,
            "expanded": node.states.expanded,
            "focused": snapshot.focus == Some(node.id),
        })).collect::<Vec<_>>(),
    })
}

fn apply_headless_action(state: &mut TestState, action: TestAction) {
    let handled = match action.kind {
        TestActionKind::NativeClose {
            window_id,
            mutable_visibility,
        } => state.windows.get_mut(&window_id).is_some_and(|snapshot| {
            snapshot.lifecycle.transition(
                wabou_shell::window_lifecycle::WindowIntent::Hide,
                WindowCapabilities { mutable_visibility },
            );
            true
        }),
        TestActionKind::ShowWindow(window_id) => {
            state.windows.get_mut(&window_id).is_some_and(|snapshot| {
                snapshot.lifecycle.transition(
                    wabou_shell::window_lifecycle::WindowIntent::Show,
                    WindowCapabilities::default(),
                );
                true
            })
        }
        TestActionKind::WaitForIdle(_) => false,
        TestActionKind::ClickByRole { .. } => false,
        TestActionKind::InputByRole { .. } => false,
        TestActionKind::QueryByRole { .. } => false,
    };
    let _ = action.completion.send(TestActionResult::Handled(handled));
}

fn action_requires_semantics(kind: &TestActionKind) -> bool {
    matches!(
        kind,
        TestActionKind::ClickByRole { .. }
            | TestActionKind::InputByRole { .. }
            | TestActionKind::QueryByRole { .. }
    )
}

fn action_requires_source_poll(kind: &TestActionKind) -> bool {
    matches!(kind, TestActionKind::WaitForIdle(_)) || action_requires_semantics(kind)
}

fn action_window_id(kind: &TestActionKind) -> Option<u64> {
    match kind {
        TestActionKind::WaitForIdle(window_id)
        | TestActionKind::ClickByRole { window_id, .. }
        | TestActionKind::InputByRole { window_id, .. }
        | TestActionKind::QueryByRole { window_id, .. } => Some(*window_id),
        _ => None,
    }
}

fn action_ready(kind: &TestActionKind, snapshot: Option<&SemanticSnapshot>) -> bool {
    match (kind, snapshot) {
        (TestActionKind::WaitForIdle(_), _) => true,
        (kind, Some(_)) if action_requires_semantics(kind) => true,
        _ => false,
    }
}

fn click_semantic_target(
    source: &mut dyn FrameSource,
    snapshot: &SemanticSnapshot,
    role: &str,
    label: &str,
    index: Option<usize>,
) -> bool {
    let Some(node) = semantic_target(snapshot, role, label, index) else {
        return false;
    };
    let point = Point {
        x: f64::from((node.bounds[0] + node.bounds[2]) * 0.5),
        y: f64::from((node.bounds[1] + node.bounds[3]) * 0.5),
    };
    source.handle_event(UiEvent::Pointer(PointerEvent {
        phase: PointerPhase::Down,
        position: point,
        button: Some(PointerButton::Primary),
        buttons: 1,
        modifiers: Modifiers::default(),
    }));
    source.handle_event(UiEvent::Pointer(PointerEvent {
        phase: PointerPhase::Up,
        position: point,
        button: Some(PointerButton::Primary),
        buttons: 0,
        modifiers: Modifiers::default(),
    }));
    true
}

fn input_semantic_target(
    source: &mut dyn FrameSource,
    snapshot: &SemanticSnapshot,
    role: &str,
    label: &str,
    input: &TestInput,
    index: Option<usize>,
) -> bool {
    let node = if matches!(input, TestInput::Probe) {
        semantic_query_target(snapshot, role, label, index)
    } else {
        semantic_target(snapshot, role, label, index)
    };
    let Some(node) = node else {
        return false;
    };
    dispatch_test_input(source, node, input)
}

fn dispatch_test_input(
    source: &mut dyn FrameSource,
    node: &wabou_shell::SemanticNode,
    input: &TestInput,
) -> bool {
    if matches!(
        input,
        TestInput::Key { .. }
            | TestInput::Text { .. }
            | TestInput::Paste { .. }
            | TestInput::Ime { .. }
    ) {
        // Focusing an already-focused node is a valid no-op and may report
        // `false`; the semantic lookup already proved the target is usable.
        source.handle_semantic_action(wabou_shell::SemanticAction::Focus { target: node.id });
    }
    for event in test_input_events(node, input) {
        source.handle_event(event);
    }
    true
}

fn semantic_target<'a>(
    snapshot: &'a SemanticSnapshot,
    role: &str,
    label: &str,
    index: Option<usize>,
) -> Option<&'a wabou_shell::SemanticNode> {
    let node = semantic_query_target(snapshot, role, label, index)?;
    (!node.disabled).then_some(node)
}

fn semantic_query_target<'a>(
    snapshot: &'a SemanticSnapshot,
    role: &str,
    label: &str,
    index: Option<usize>,
) -> Option<&'a wabou_shell::SemanticNode> {
    snapshot.node_by_role(SemanticRole::from_name(role)?, label, index)
}

pub(crate) struct TestDriver {
    controller: TestController,
}

impl TestDriver {
    pub(crate) fn new(controller: TestController) -> Self {
        Self { controller }
    }

    fn snapshot(&self, window_id: u64, context: &ExtensionContext<'_>) {
        let Some(lifecycle) = context.window_lifecycle(window_id) else {
            return;
        };
        if let Ok(mut state) = self.controller.0.lock() {
            state
                .windows
                .insert(window_id, WindowSnapshot { lifecycle });
        }
    }
}

impl ShellExtension for TestDriver {
    fn initialize(&mut self, wake: WakeCallback) -> Result<(), String> {
        self.controller
            .0
            .lock()
            .map_err(|_| "test controller mutex poisoned".to_string())?
            .wake = Some(wake);
        Ok(())
    }

    fn poll(&mut self, context: &mut ExtensionContext<'_>) {
        loop {
            let action = self
                .controller
                .0
                .lock()
                .ok()
                .and_then(|mut state| state.actions.pop_front());
            let Some(action) = action else {
                break;
            };
            if let Some(window_id) = action_window_id(&action.kind)
                && let Some(snapshot) = context.semantic_snapshot(window_id)
            {
                self.controller
                    .record_semantic_snapshot(window_id, snapshot);
            }
            let result = match action.kind {
                TestActionKind::WaitForIdle(_) => TestActionResult::Handled(true),
                TestActionKind::NativeClose {
                    window_id,
                    mutable_visibility,
                } => {
                    let handled = context.hide_window_with_capabilities(
                        window_id,
                        Some(WindowCapabilities { mutable_visibility }),
                    );
                    self.snapshot(window_id, context);
                    TestActionResult::Handled(handled)
                }
                TestActionKind::ShowWindow(window_id) => {
                    let handled = context.show_window(window_id);
                    self.snapshot(window_id, context);
                    TestActionResult::Handled(handled)
                }
                TestActionKind::ClickByRole {
                    window_id,
                    role,
                    label,
                    index,
                } => TestActionResult::Handled(match index {
                    Some(index) => context.click_by_role_at(window_id, &role, &label, index),
                    None => context.click_by_role(window_id, &role, &label),
                }),
                TestActionKind::InputByRole {
                    window_id,
                    role,
                    label,
                    input,
                    index,
                } => {
                    let node = match index {
                        Some(index) => {
                            context.semantic_node_by_role_at(window_id, &role, &label, index)
                        }
                        None => context.semantic_node_by_role(window_id, &role, &label),
                    };
                    let Some(node) = node else {
                        let _ = action.completion.send(TestActionResult::Handled(false));
                        continue;
                    };
                    if node.disabled && !matches!(input, TestInput::Probe) {
                        let _ = action.completion.send(TestActionResult::Handled(false));
                        continue;
                    }
                    match &input {
                        TestInput::Key { .. }
                        | TestInput::Text { .. }
                        | TestInput::Paste { .. }
                        | TestInput::Ime { .. } => {
                            context.focus_semantic_node(window_id, node.id);
                        }
                        _ => {}
                    }
                    let events = test_input_events(&node, &input);
                    TestActionResult::Handled(
                        events
                            .into_iter()
                            .all(|event| context.dispatch_event(window_id, event)),
                    )
                }
                TestActionKind::QueryByRole {
                    window_id,
                    role,
                    label,
                    index,
                } => TestActionResult::Query(
                    context
                        .semantic_snapshot(window_id)
                        .and_then(|snapshot| locator_query_json(&snapshot, &role, &label, index)),
                ),
            };
            let _ = action.completion.send(result);
        }
        if self
            .controller
            .0
            .lock()
            .is_ok_and(|state| state.report.is_some())
        {
            context.exit();
        }
    }
}

fn test_input_events(node: &wabou_shell::SemanticNode, input: &TestInput) -> Vec<UiEvent> {
    let center = Point {
        x: f64::from((node.bounds[0] + node.bounds[2]) * 0.5),
        y: f64::from((node.bounds[1] + node.bounds[3]) * 0.5),
    };
    match input {
        TestInput::Probe => Vec::new(),
        TestInput::Drag { delta_x, delta_y } => {
            let end = Point {
                x: center.x + delta_x,
                y: center.y + delta_y,
            };
            vec![
                UiEvent::Pointer(PointerEvent {
                    phase: PointerPhase::Down,
                    position: center,
                    button: Some(PointerButton::Primary),
                    buttons: 1,
                    modifiers: Modifiers::default(),
                }),
                UiEvent::Pointer(PointerEvent {
                    phase: PointerPhase::Move,
                    position: end,
                    button: Some(PointerButton::Primary),
                    buttons: 1,
                    modifiers: Modifiers::default(),
                }),
                UiEvent::Pointer(PointerEvent {
                    phase: PointerPhase::Up,
                    position: end,
                    button: Some(PointerButton::Primary),
                    buttons: 0,
                    modifiers: Modifiers::default(),
                }),
            ]
        }
        TestInput::Key { key, modifiers } => [KeyPhase::Down, KeyPhase::Up]
            .into_iter()
            .map(|phase| {
                UiEvent::Key(KeyEvent {
                    phase,
                    key: key.clone(),
                    key_without_modifiers: key.clone(),
                    code: key.clone(),
                    text: None,
                    text_with_all_modifiers: None,
                    location: KeyLocation::Standard,
                    modifiers: Modifiers::from_bits_truncate(*modifiers),
                    repeat: false,
                })
            })
            .collect(),
        TestInput::Text { text } => vec![UiEvent::TextInput(text.clone())],
        TestInput::Paste { text } => vec![UiEvent::Paste(text.clone())],
        TestInput::Ime { text } => vec![
            UiEvent::Ime(ImeEvent::Enabled),
            UiEvent::Ime(ImeEvent::Preedit {
                text: text.clone(),
                cursor: Some((text.len(), text.len())),
            }),
            UiEvent::Ime(ImeEvent::Commit(text.clone())),
            UiEvent::Ime(ImeEvent::Disabled),
        ],
        TestInput::Wheel { delta_x, delta_y } => vec![UiEvent::Wheel(WheelEvent {
            position: center,
            delta_x: *delta_x,
            delta_y: *delta_y,
            modifiers: Modifiers::default(),
        })],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SemanticSource(Arc<SemanticSnapshot>);

    impl FrameSource for SemanticSource {
        fn build_frame(
            &mut self,
            _tcx: &mut wabou_shell::TextContext,
            _width: u32,
            _height: u32,
        ) -> Vec<wabou_shell::layout::PlacedNode> {
            Vec::new()
        }

        fn semantic_snapshot(&self) -> Option<Arc<SemanticSnapshot>> {
            Some(self.0.clone())
        }

        fn base_color(&self) -> vello::peniko::Color {
            vello::peniko::Color::BLACK
        }
    }

    fn node() -> wabou_shell::SemanticNode {
        wabou_shell::SemanticNode {
            id: 7,
            role: SemanticRole::TextInput,
            label: Some("Editor".into()),
            value: None,
            numeric_value: None,
            min_numeric_value: None,
            max_numeric_value: None,
            bounds: [10.0, 20.0, 110.0, 60.0],
            children: Vec::new(),
            controls: Vec::new(),
            active_descendant: None,
            disabled: false,
            states: wabou_shell::SemanticStates::default(),
        }
    }

    #[test]
    fn semantic_failure_artifact_exposes_structure_without_control_values() {
        let controller = TestController::default();
        let mut sensitive = node();
        sensitive.value = Some("must-not-be-persisted".into());
        sensitive.controls = vec![8];
        sensitive.active_descendant = Some(8);
        sensitive.states.checked = Some(wabou_shell::SemanticToggleState::Mixed);
        let mut generic = node();
        generic.id = 8;
        generic.role = SemanticRole::Generic;
        generic.label = Some("large concatenated descendant label".into());
        controller.record_semantic_snapshot(
            2,
            Arc::new(SemanticSnapshot {
                revision: 9,
                nodes: vec![sensitive, generic],
                root_children: vec![7, 8],
                focus: Some(7),
                modal_root: None,
            }),
        );

        let artifact = controller.semantic_artifact();
        assert_eq!(artifact["version"], 1);
        assert_eq!(artifact["windows"][0]["windowId"], 2);
        assert_eq!(artifact["windows"][0]["nodes"][0]["role"], "textbox");
        assert_eq!(artifact["windows"][0]["nodes"][0]["hasValue"], true);
        assert_eq!(artifact["windows"][0]["nodes"][0]["checked"], "mixed");
        assert_eq!(artifact["windows"][0]["nodes"][0]["focused"], true);
        assert_eq!(
            artifact["windows"][0]["nodes"][0]["controls"],
            serde_json::json!([8])
        );
        assert_eq!(artifact["windows"][0]["nodes"][0]["activeDescendant"], 8);
        assert_eq!(
            artifact["windows"][0]["nodes"][1]["name"],
            serde_json::Value::Null
        );
        assert!(!artifact.to_string().contains("must-not-be-persisted"));
        assert!(
            !artifact
                .to_string()
                .contains("large concatenated descendant label")
        );
    }

    #[test]
    fn locator_snapshot_exposes_logical_origin_and_size() {
        let snapshot =
            serde_json::from_str::<serde_json::Value>(&locator_snapshot_json(&node(), false))
                .unwrap();
        assert_eq!(
            snapshot["bounds"],
            serde_json::json!({ "x": 10.0, "y": 20.0, "width": 100.0, "height": 40.0 })
        );
    }

    #[test]
    fn drag_is_a_captured_pointer_sequence_from_semantic_center() {
        let events = test_input_events(
            &node(),
            &TestInput::Drag {
                delta_x: 25.0,
                delta_y: -5.0,
            },
        );
        assert_eq!(events.len(), 3);
        let UiEvent::Pointer(down) = &events[0] else {
            panic!()
        };
        let UiEvent::Pointer(moved) = &events[1] else {
            panic!()
        };
        let UiEvent::Pointer(up) = &events[2] else {
            panic!()
        };
        assert_eq!(
            (down.phase, down.position.x, down.position.y, down.buttons),
            (PointerPhase::Down, 60.0, 40.0, 1)
        );
        assert_eq!(
            (
                moved.phase,
                moved.position.x,
                moved.position.y,
                moved.buttons
            ),
            (PointerPhase::Move, 85.0, 35.0, 1)
        );
        assert_eq!((up.phase, up.buttons), (PointerPhase::Up, 0));
    }

    #[test]
    fn ime_action_preserves_the_full_native_lifecycle() {
        let events = test_input_events(&node(), &TestInput::Ime { text: "你".into() });
        assert!(matches!(events.as_slice(), [
            UiEvent::Ime(ImeEvent::Enabled),
            UiEvent::Ime(ImeEvent::Preedit { cursor: Some((3, 3)), .. }),
            UiEvent::Ime(ImeEvent::Commit(text)),
            UiEvent::Ime(ImeEvent::Disabled),
        ] if text == "你"));
    }

    #[test]
    fn headless_locators_support_live_region_roles() {
        let snapshot = SemanticSnapshot {
            nodes: vec![
                wabou_shell::SemanticNode {
                    role: SemanticRole::Alert,
                    label: Some("Failed".into()),
                    ..node()
                },
                wabou_shell::SemanticNode {
                    id: 8,
                    role: SemanticRole::Status,
                    label: Some("Saved".into()),
                    ..node()
                },
            ],
            root_children: vec![7, 8],
            ..SemanticSnapshot::default()
        };

        assert!(semantic_target(&snapshot, "alert", "Failed", None).is_some());
        assert!(semantic_target(&snapshot, "status", "Saved", None).is_some());
    }

    #[test]
    fn headless_locators_support_repository_component_roles() {
        let roles = [
            ("group", SemanticRole::Group),
            ("img", SemanticRole::Image),
            ("radiogroup", SemanticRole::RadioGroup),
            ("menu", SemanticRole::Menu),
            ("menuitem", SemanticRole::MenuItem),
            ("tree", SemanticRole::Tree),
            ("treeitem", SemanticRole::TreeItem),
            ("tablist", SemanticRole::TabList),
            ("tab", SemanticRole::Tab),
            ("tabpanel", SemanticRole::TabPanel),
            ("grid", SemanticRole::Grid),
            ("gridcell", SemanticRole::GridCell),
        ];
        let nodes = roles
            .iter()
            .enumerate()
            .map(|(index, (_, role))| wabou_shell::SemanticNode {
                id: index as u64 + 10,
                role: *role,
                label: Some("Target".into()),
                ..node()
            })
            .collect::<Vec<_>>();
        let snapshot = SemanticSnapshot {
            root_children: nodes.iter().map(|node| node.id).collect(),
            nodes,
            ..SemanticSnapshot::default()
        };

        for (role, _) in roles {
            assert!(
                semantic_target(&snapshot, role, "Target", None).is_some(),
                "missing locator role {role}"
            );
        }
    }

    #[test]
    fn headless_semantic_actions_read_each_completed_snapshot_without_native_timeout() {
        let snapshot = SemanticSnapshot::default();
        let probe = TestActionKind::InputByRole {
            window_id: 1,
            role: "button".into(),
            label: "Appears later".into(),
            input: TestInput::Probe,
            index: None,
        };
        let click = TestActionKind::ClickByRole {
            window_id: 1,
            role: "button".into(),
            label: "Appears later".into(),
            index: None,
        };

        assert!(action_ready(&probe, Some(&snapshot)));
        assert!(action_ready(&click, Some(&snapshot)));
    }

    #[test]
    fn concurrent_queries_keep_results_attached_to_their_requests() {
        let controller = TestController::default();
        controller.initialize_headless([1, 2]);
        let first = controller.request(TestActionKind::QueryByRole {
            window_id: 1,
            role: "textbox".into(),
            label: "First".into(),
            index: None,
        });
        let second = controller.request(TestActionKind::QueryByRole {
            window_id: 2,
            role: "textbox".into(),
            label: "Second".into(),
            index: None,
        });
        let snapshot = |label: &str, value: &str| {
            let mut target = node();
            target.label = Some(label.into());
            target.value = Some(value.into());
            Arc::new(SemanticSnapshot {
                root_children: vec![target.id],
                nodes: vec![target],
                ..SemanticSnapshot::default()
            })
        };
        let mut first_source = SemanticSource(snapshot("First", "one"));
        let mut second_source = SemanticSource(snapshot("Second", "two"));

        // Complete both requests before either receiver is observed. A shared
        // query-result slot would overwrite or consume one of these values.
        controller.poll_headless_source(1, &mut first_source);
        controller.poll_headless_source(2, &mut second_source);

        let TestActionResult::Query(Some(first)) = first.blocking_recv().unwrap() else {
            panic!("first query did not return its snapshot")
        };
        let TestActionResult::Query(Some(second)) = second.blocking_recv().unwrap() else {
            panic!("second query did not return its snapshot")
        };
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&first).unwrap()["snapshot"]["value"],
            "one"
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&second).unwrap()["snapshot"]["value"],
            "two"
        );
    }

    #[test]
    fn locator_queries_report_ambiguous_matches_without_selecting_one() {
        let mut duplicate = node();
        duplicate.id = 8;
        let snapshot = SemanticSnapshot {
            nodes: vec![node(), duplicate],
            root_children: vec![7, 8],
            ..SemanticSnapshot::default()
        };

        let query = serde_json::from_str::<serde_json::Value>(
            &locator_query_json(&snapshot, "textbox", "Editor", None).unwrap(),
        )
        .unwrap();
        assert_eq!(query["matchCount"], 2);
        assert!(semantic_query_target(&snapshot, "textbox", "Editor", None).is_none());
        assert!(semantic_target(&snapshot, "textbox", "Editor", None).is_none());
        assert_eq!(
            semantic_query_target(&snapshot, "textbox", "Editor", Some(1)).map(|node| node.id),
            Some(8)
        );
    }

    #[test]
    fn locators_cannot_query_nodes_behind_an_active_modal() {
        let mut background = node();
        background.id = 7;
        background.role = SemanticRole::Button;
        background.label = Some("Background".into());
        let mut modal = node();
        modal.id = 8;
        modal.role = SemanticRole::Dialog;
        modal.label = Some("Settings".into());
        let snapshot = SemanticSnapshot {
            nodes: vec![background, modal],
            root_children: vec![7, 8],
            modal_root: Some(8),
            ..SemanticSnapshot::default()
        };

        assert!(locator_query_json(&snapshot, "button", "Background", None).is_none());
        assert!(semantic_target(&snapshot, "button", "Background", None).is_none());
        assert!(semantic_target(&snapshot, "dialog", "Settings", None).is_some());
    }

    #[test]
    fn finishing_cancels_pending_and_future_actions_without_replacing_the_report() {
        let controller = TestController::default();
        let pending_input = controller.request(TestActionKind::ClickByRole {
            window_id: 1,
            role: "button".into(),
            label: "Late action".into(),
            index: None,
        });
        let pending_query = controller.request(TestActionKind::QueryByRole {
            window_id: 1,
            role: "textbox".into(),
            label: "Late query".into(),
            index: None,
        });

        assert!(controller.finish("first report".into()));
        assert!(!controller.finish("replacement report".into()));
        assert!(matches!(
            pending_input.blocking_recv(),
            Ok(TestActionResult::Handled(false))
        ));
        assert!(matches!(
            pending_query.blocking_recv(),
            Ok(TestActionResult::Query(None))
        ));

        let future = controller.request(TestActionKind::WaitForIdle(1));
        assert!(matches!(
            future.blocking_recv(),
            Ok(TestActionResult::Handled(false))
        ));
        let state = controller.0.lock().unwrap();
        assert!(state.actions.is_empty());
        assert_eq!(state.report.as_deref(), Some("first report"));
    }
}
