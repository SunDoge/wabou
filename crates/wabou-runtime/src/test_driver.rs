//! Test-only bridge between QuickJS scenarios and the native event loop.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
    },
    InputByRole {
        window_id: u64,
        role: String,
        label: String,
        input: TestInput,
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
    completion: oneshot::Sender<bool>,
    deadline: Instant,
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
    query_result: Option<String>,
    headless: bool,
}

#[derive(Clone, Default)]
pub(crate) struct TestController(Arc<Mutex<TestState>>);

impl TestController {
    fn request(&self, kind: TestActionKind) -> oneshot::Receiver<bool> {
        let (completion, receiver) = oneshot::channel();
        let wake = {
            let Ok(mut state) = self.0.lock() else {
                return receiver;
            };
            let action = TestAction {
                kind,
                completion,
                deadline: Instant::now() + Duration::from_secs(2),
            };
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
                        async move { receiver.await.unwrap_or(false) }
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
                        async move { receiver.await.unwrap_or(false) }
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
                        async move { receiver.await.unwrap_or(false) }
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
                    Async(move |window_id: u64, role: String, label: String| {
                        let receiver = click.request(TestActionKind::ClickByRole {
                            window_id,
                            role,
                            label,
                        });
                        async move { receiver.await.unwrap_or(false) }
                    }),
                )?,
            )?;

            let input = controller.clone();
            capability.set(
                "inputByRole",
                Function::new(
                    ctx.clone(),
                    Async(
                        move |window_id: u64, role: String, label: String, raw: String| {
                            let receiver =
                                serde_json::from_str::<TestInput>(&raw).ok().map(|action| {
                                    input.request(TestActionKind::InputByRole {
                                        window_id,
                                        role,
                                        label,
                                        input: action,
                                    })
                                });
                            async move {
                                match receiver {
                                    Some(receiver) => receiver.await.unwrap_or(false),
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
                Function::new(ctx.clone(), move |report: String| {
                    if let Ok(mut state) = finish.0.lock() {
                        state.report = Some(report);
                        if let Some(wake) = &state.wake {
                            wake();
                        }
                        true
                    } else {
                        false
                    }
                })?,
            )?;

            let query_result = controller.clone();
            capability.set(
                "takeQueryResult",
                Function::new(ctx.clone(), move || {
                    query_result
                        .0
                        .lock()
                        .ok()
                        .and_then(|mut state| state.query_result.take())
                        .unwrap_or_else(|| "null".into())
                })?,
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
        if let Some(snapshot) = snapshot.as_deref()
            && let Some(action) = self.0.lock().ok().and_then(|state| {
                state.actions.iter().find_map(|action| match &action.kind {
                    TestActionKind::ClickByRole {
                        window_id: target_window,
                        role,
                        label,
                    } if *target_window == window_id => semantic_target(snapshot, role, label),
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
            let ready = match (&action.kind, snapshot.as_deref()) {
                (TestActionKind::WaitForIdle(_), _) => true,
                (kind, Some(snapshot)) => action_semantic_target(kind, snapshot).is_some(),
                _ => false,
            };
            if !ready
                && Instant::now() >= action.deadline
                && let Some(snapshot) = snapshot.as_deref()
            {
                let requested_role = match &action.kind {
                    TestActionKind::ClickByRole { role, .. }
                    | TestActionKind::InputByRole { role, .. } => Some(role.as_str()),
                    _ => None,
                };
                let candidate_count = snapshot
                    .nodes
                    .iter()
                    .filter(|node| {
                        requested_role.is_none_or(|role| semantic_role_matches(role, node.role))
                    })
                    .count();
                let candidates = snapshot
                    .nodes
                    .iter()
                    .filter(|node| {
                        requested_role.is_none_or(|role| semantic_role_matches(role, node.role))
                    })
                    .take(24)
                    .filter_map(|node| {
                        node.label.as_deref().map(|label| {
                            format!("{:?} {label:?} disabled={}", node.role, node.disabled)
                        })
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                tracing::warn!(
                    target: "wabou::test",
                    action = ?action.kind,
                    revision = snapshot.revision,
                    candidate_count,
                    candidates,
                    "semantic locator timed out"
                );
            }
            (ready || Instant::now() >= action.deadline)
                .then(|| state.actions.remove(index))
                .flatten()
        });
        let Some(action) = action else {
            return;
        };
        let handled = match (&action.kind, snapshot.as_deref()) {
            (TestActionKind::WaitForIdle(_), _) => true,
            (TestActionKind::ClickByRole { role, label, .. }, Some(snapshot)) => {
                click_semantic_target(source, snapshot, role, label)
            }
            (
                TestActionKind::InputByRole {
                    role, label, input, ..
                },
                Some(snapshot),
            ) => input_semantic_target(source, snapshot, role, label, input),
            _ => false,
        };
        if let (
            TestActionKind::InputByRole {
                role,
                label,
                input: TestInput::Probe,
                ..
            },
            Some(snapshot),
        ) = (&action.kind, snapshot.as_deref())
            && let Some(node) = semantic_query_target(snapshot, role, label)
        {
            self.set_query_result(node, snapshot.focus == Some(node.id));
        }
        let _ = action.completion.send(handled);
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

    fn set_query_result(&self, node: &wabou_shell::SemanticNode, focused: bool) {
        let toggle_value = |state: Option<wabou_shell::SemanticToggleState>| match state {
            Some(wabou_shell::SemanticToggleState::Off) => serde_json::Value::Bool(false),
            Some(wabou_shell::SemanticToggleState::On) => serde_json::Value::Bool(true),
            Some(wabou_shell::SemanticToggleState::Mixed) => {
                serde_json::Value::String("mixed".into())
            }
            None => serde_json::Value::Null,
        };
        let result = serde_json::json!({
            "name": node.label,
            "value": node.value,
            "disabled": node.disabled,
            "checked": toggle_value(node.states.checked),
            "pressed": toggle_value(node.states.pressed),
            "selected": node.states.selected,
            "expanded": node.states.expanded,
            "focused": focused,
        })
        .to_string();
        if let Ok(mut state) = self.0.lock() {
            state.query_result = Some(result);
        }
    }
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
    };
    let _ = action.completion.send(handled);
}

fn action_requires_semantics(kind: &TestActionKind) -> bool {
    matches!(
        kind,
        TestActionKind::ClickByRole { .. } | TestActionKind::InputByRole { .. }
    )
}

fn action_requires_source_poll(kind: &TestActionKind) -> bool {
    matches!(kind, TestActionKind::WaitForIdle(_)) || action_requires_semantics(kind)
}

fn action_window_id(kind: &TestActionKind) -> Option<u64> {
    match kind {
        TestActionKind::WaitForIdle(window_id)
        | TestActionKind::ClickByRole { window_id, .. }
        | TestActionKind::InputByRole { window_id, .. } => Some(*window_id),
        _ => None,
    }
}

fn action_semantic_target<'a>(
    kind: &TestActionKind,
    snapshot: &'a SemanticSnapshot,
) -> Option<&'a wabou_shell::SemanticNode> {
    match kind {
        TestActionKind::InputByRole {
            role,
            label,
            input: TestInput::Probe,
            ..
        } => semantic_query_target(snapshot, role, label),
        TestActionKind::ClickByRole { role, label, .. }
        | TestActionKind::InputByRole { role, label, .. } => semantic_target(snapshot, role, label),
        _ => None,
    }
}

fn click_semantic_target(
    source: &mut dyn FrameSource,
    snapshot: &SemanticSnapshot,
    role: &str,
    label: &str,
) -> bool {
    let Some(node) = semantic_target(snapshot, role, label) else {
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
) -> bool {
    let node = if matches!(input, TestInput::Probe) {
        semantic_query_target(snapshot, role, label)
    } else {
        semantic_target(snapshot, role, label)
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
) -> Option<&'a wabou_shell::SemanticNode> {
    snapshot.nodes.iter().find(|node| {
        semantic_role_matches(role, node.role)
            && node.label.as_deref() == Some(label)
            && !node.disabled
    })
}

fn semantic_query_target<'a>(
    snapshot: &'a SemanticSnapshot,
    role: &str,
    label: &str,
) -> Option<&'a wabou_shell::SemanticNode> {
    snapshot
        .nodes
        .iter()
        .find(|node| semantic_role_matches(role, node.role) && node.label.as_deref() == Some(label))
}

fn semantic_role_matches(role: &str, candidate: SemanticRole) -> bool {
    matches!(
        (role, candidate),
        ("button", SemanticRole::Button)
            | ("textbox", SemanticRole::TextInput)
            | ("link", SemanticRole::Link)
            | ("dialog", SemanticRole::Dialog)
            | ("alert", SemanticRole::Alert)
            | ("status", SemanticRole::Status)
            | ("checkbox", SemanticRole::CheckBox)
            | ("radio", SemanticRole::RadioButton)
            | ("switch", SemanticRole::Switch)
            | ("combobox", SemanticRole::ComboBox)
            | ("listbox", SemanticRole::ListBox)
            | ("option", SemanticRole::Option)
            | ("table", SemanticRole::Table)
            | ("row", SemanticRole::Row)
            | ("cell", SemanticRole::Cell)
            | ("columnheader", SemanticRole::ColumnHeader)
            | ("rowheader", SemanticRole::RowHeader)
            | ("slider", SemanticRole::Slider)
            | ("label", SemanticRole::Label)
    )
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
            let handled = match action.kind {
                TestActionKind::WaitForIdle(_) => true,
                TestActionKind::NativeClose {
                    window_id,
                    mutable_visibility,
                } => {
                    let handled = context.hide_window_with_capabilities(
                        window_id,
                        Some(WindowCapabilities { mutable_visibility }),
                    );
                    self.snapshot(window_id, context);
                    handled
                }
                TestActionKind::ShowWindow(window_id) => {
                    let handled = context.show_window(window_id);
                    self.snapshot(window_id, context);
                    handled
                }
                TestActionKind::ClickByRole {
                    window_id,
                    role,
                    label,
                } => context.click_by_role(window_id, &role, &label),
                TestActionKind::InputByRole {
                    window_id,
                    role,
                    label,
                    input,
                } => {
                    let Some(node) = context.semantic_node_by_role(window_id, &role, &label) else {
                        let _ = action.completion.send(false);
                        continue;
                    };
                    if node.disabled && !matches!(input, TestInput::Probe) {
                        let _ = action.completion.send(false);
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
                    if matches!(input, TestInput::Probe) {
                        let focused = context.semantic_node_focused(window_id, node.id);
                        self.controller.set_query_result(&node, focused);
                    }
                    events
                        .into_iter()
                        .all(|event| context.dispatch_event(window_id, event))
                }
            };
            let _ = action.completion.send(handled);
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

    fn node() -> wabou_shell::SemanticNode {
        wabou_shell::SemanticNode {
            id: 7,
            role: SemanticRole::TextInput,
            label: Some("Editor".into()),
            value: None,
            bounds: [10.0, 20.0, 110.0, 60.0],
            children: Vec::new(),
            disabled: false,
            states: wabou_shell::SemanticStates::default(),
        }
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
            ..SemanticSnapshot::default()
        };

        assert!(semantic_target(&snapshot, "alert", "Failed").is_some());
        assert!(semantic_target(&snapshot, "status", "Saved").is_some());
    }
}
