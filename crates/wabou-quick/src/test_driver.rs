//! Test-only bridge between QuickJS scenarios and the native event loop.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use rquickjs::{Function, prelude::Async};
use tokio::sync::oneshot;
use wabou_shell::window_lifecycle::{WindowCapabilities, WindowLifecycle, WindowPresence};
use wabou_shell::{ExtensionContext, ShellExtension, WakeCallback};
use wabou_shell::{
    FrameSource, Modifiers, Point, PointerButton, PointerEvent, PointerPhase, SemanticRole,
    SemanticSnapshot, UiEvent,
};

const CAPABILITY: &str = "test";

#[derive(Debug)]
enum TestActionKind {
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
}

#[derive(Debug)]
struct TestAction {
    kind: TestActionKind,
    completion: oneshot::Sender<bool>,
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
            let action = TestAction { kind, completion };
            if state.headless {
                if matches!(action.kind, TestActionKind::ClickByRole { .. }) {
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

            let finish = controller.clone();
            capability.set(
                "finish",
                Function::new(ctx, move |report: String| {
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
        let action = self.0.lock().ok().and_then(|mut state| {
            let index = state.actions.iter().position(|action| {
                matches!(
                    &action.kind,
                    TestActionKind::ClickByRole { window_id: target, .. } if *target == window_id
                )
            })?;
            state.actions.remove(index)
        });
        let Some(action) = action else {
            return;
        };
        let handled = match (&action.kind, snapshot.as_deref()) {
            (TestActionKind::ClickByRole { role, label, .. }, Some(snapshot)) => {
                click_semantic_target(source, snapshot, role, label)
            }
            _ => false,
        };
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
        TestActionKind::ClickByRole { .. } => false,
    };
    let _ = action.completion.send(handled);
}

fn click_semantic_target(
    source: &mut dyn FrameSource,
    snapshot: &SemanticSnapshot,
    role: &str,
    label: &str,
) -> bool {
    let role_matches = |candidate: SemanticRole| {
        matches!(
            (role, candidate),
            ("button", SemanticRole::Button)
                | ("textbox", SemanticRole::TextInput)
                | ("link", SemanticRole::Link)
                | ("dialog", SemanticRole::Dialog)
                | ("checkbox", SemanticRole::CheckBox)
                | ("radio", SemanticRole::RadioButton)
                | ("switch", SemanticRole::Switch)
                | ("combobox", SemanticRole::ComboBox)
                | ("listbox", SemanticRole::ListBox)
                | ("option", SemanticRole::Option)
                | ("label", SemanticRole::Label)
        )
    };
    let Some(node) = snapshot.nodes.iter().find(|node| {
        role_matches(node.role) && node.label.as_deref() == Some(label) && !node.disabled
    }) else {
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
