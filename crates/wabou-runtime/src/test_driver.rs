//! Test-only bridge between QuickJS scenarios and the native event loop.

use std::collections::{HashMap, VecDeque};
use std::path::{Component, Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

use rquickjs::{Ctx, Function, Object, prelude::Async};
use serde::Deserialize;
use tokio::sync::oneshot;
use wabou_shell_api as wabou_shell;
use wabou_shell_api::{
    FileDropEvent, ImeEvent, KeyEvent, KeyLocation, KeyPhase, Modifiers, Point, PointerButton,
    PointerEvent, PointerPhase, SemanticRole, SemanticSnapshot, UiEvent, WakeCallback, WheelEvent,
    WindowCapabilities, WindowIntent, WindowLifecycle, WindowPresence,
};

const CAPABILITY: &str = "test";
const MAX_FIXTURE_BYTES: usize = 16 * 1024 * 1024;
static NEXT_FIXTURE_DIRECTORY: AtomicU64 = AtomicU64::new(1);
type WindowKey = wabou_shell::WindowResourceKey;

trait SemanticTestSource {
    #[cfg(test)]
    fn semantic_snapshot(&self) -> Option<Arc<SemanticSnapshot>>;
    fn handle_event(&mut self, event: UiEvent);
    fn handle_semantic_action(&mut self, action: wabou_shell::SemanticAction) -> bool;
}

#[derive(Clone, Copy, Debug)]
enum FileDropPhase {
    Entered,
    Moved,
    Left,
    Dropped,
}

impl From<FileDropPhase> for wabou_shell::FileDropPhase {
    fn from(value: FileDropPhase) -> Self {
        match value {
            FileDropPhase::Entered => Self::Entered,
            FileDropPhase::Moved => Self::Moved,
            FileDropPhase::Left => Self::Left,
            FileDropPhase::Dropped => Self::Dropped,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestLocatorSelector {
    role: String,
    name: String,
    index: Option<usize>,
}

fn window_key(lo: u32, hi: u32) -> Option<WindowKey> {
    WindowKey::from_parts(lo, hi)
}

#[derive(Debug)]
enum TestActionKind {
    WaitForIdle(WindowKey),
    NativeClose {
        window_key: WindowKey,
        mutable_visibility: bool,
    },
    ShowWindow(WindowKey),
    ResizeWindow {
        window_key: WindowKey,
        width: u32,
        height: u32,
    },
    FileDrop {
        window_key: WindowKey,
        phase: FileDropPhase,
        paths: Vec<PathBuf>,
    },
    ClickByRole {
        window_key: WindowKey,
        role: String,
        label: String,
        index: Option<usize>,
        scope: Vec<TestLocatorSelector>,
    },
    InputByRole {
        window_key: WindowKey,
        role: String,
        label: String,
        input: TestInput,
        index: Option<usize>,
        scope: Vec<TestLocatorSelector>,
    },
    QueryByRole {
        window_key: WindowKey,
        role: String,
        label: String,
        index: Option<usize>,
        scope: Vec<TestLocatorSelector>,
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
    viewport: Option<(u32, u32)>,
}

#[derive(Debug)]
struct TestFixtureDirectory {
    path: PathBuf,
}

impl TestFixtureDirectory {
    fn new() -> Self {
        let sequence = NEXT_FIXTURE_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        Self {
            path: std::env::temp_dir().join(format!(
                "wabou-test-fixtures-{}-{sequence}",
                std::process::id()
            )),
        }
    }

    fn write_text(&self, relative: &str, contents: &str) -> Result<PathBuf, String> {
        if relative.is_empty()
            || Path::new(relative)
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err("fixture path must contain only relative normal components".into());
        }
        if contents.len() > MAX_FIXTURE_BYTES {
            return Err("fixture contents exceed the 16 MB safety limit".into());
        }
        let path = self.path.join(relative);
        let parent = path
            .parent()
            .ok_or_else(|| "fixture path has no parent".to_owned())?;
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create fixture directory: {error}"))?;
        std::fs::write(&path, contents)
            .map_err(|error| format!("cannot write fixture file: {error}"))?;
        Ok(path)
    }
}

impl Drop for TestFixtureDirectory {
    fn drop(&mut self) {
        match std::fs::remove_dir_all(&self.path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => tracing::warn!(
                path = %self.path.display(),
                %error,
                "failed to clean behavior-test fixture directory"
            ),
        }
    }
}

#[derive(Default)]
struct TestState {
    actions: VecDeque<TestAction>,
    windows: HashMap<WindowKey, WindowSnapshot>,
    wake: Option<WakeCallback>,
    report: Option<String>,
    semantic_snapshots: HashMap<WindowKey, Arc<SemanticSnapshot>>,
    #[cfg(test)]
    headless_viewports: HashMap<WindowKey, (u32, u32)>,
    #[cfg(test)]
    headless: bool,
}

#[derive(Clone)]
#[doc(hidden)]
/// Shared controller for renderer-independent native behavior tests.
pub struct TestController {
    state: Arc<Mutex<TestState>>,
    effects: crate::effect_trace::EffectTrace,
    fixtures: Arc<TestFixtureDirectory>,
}

/// Backend adapter used by native behavior tests.
///
/// This is intentionally a narrow semantic/input contract rather than a
/// renderer API. Alternate window hosts can therefore execute the same
/// `@wabou/test` scenarios without projecting a GPUI tree.
#[doc(hidden)]
pub trait NativeTestHost {
    fn semantic_snapshot(&mut self, window_key: WindowKey) -> Option<Arc<SemanticSnapshot>>;
    fn dispatch_event(&mut self, window_key: WindowKey, event: UiEvent) -> bool;
    fn dispatch_semantic_action(
        &mut self,
        window_key: WindowKey,
        action: wabou_shell::SemanticAction,
    ) -> bool;
    fn hide_window(&mut self, window_key: WindowKey, mutable_visibility: bool) -> bool;
    fn show_window(&mut self, window_key: WindowKey) -> bool;
    fn resize_window(&mut self, window_key: WindowKey, width: u32, height: u32) -> bool;
    fn window_viewport(&self, window_key: WindowKey) -> Option<(u32, u32)>;
}

struct NativeSemanticSource<'a> {
    host: &'a mut dyn NativeTestHost,
    window_key: WindowKey,
}

impl SemanticTestSource for NativeSemanticSource<'_> {
    #[cfg(test)]
    fn semantic_snapshot(&self) -> Option<Arc<SemanticSnapshot>> {
        None
    }

    fn handle_event(&mut self, event: UiEvent) {
        let _ = self.host.dispatch_event(self.window_key, event);
    }

    fn handle_semantic_action(&mut self, action: wabou_shell::SemanticAction) -> bool {
        self.host.dispatch_semantic_action(self.window_key, action)
    }
}

impl Default for TestController {
    fn default() -> Self {
        Self::new(crate::effect_trace::EffectTrace::fixtures())
    }
}

impl TestController {
    /// Create a controller backed by the same effect trace installed in the
    /// application runtime.
    ///
    /// Alternate native hosts must share this trace with their QuickJS effect
    /// bridge so deterministic fixtures are visible on both sides.
    #[doc(hidden)]
    pub fn new(effects: crate::effect_trace::EffectTrace) -> Self {
        Self {
            state: Arc::new(Mutex::new(TestState::default())),
            effects,
            fixtures: Arc::new(TestFixtureDirectory::new()),
        }
    }

    /// Attach native windows and their event-loop wake callback to the shared
    /// behavior-test queue.
    #[doc(hidden)]
    pub fn connect_native_windows(
        &self,
        window_keys: impl IntoIterator<Item = WindowKey>,
        wake: WakeCallback,
    ) {
        if let Ok(mut state) = self.state.lock() {
            state.wake = Some(wake);
            for window_key in window_keys {
                state.windows.insert(
                    window_key,
                    WindowSnapshot {
                        lifecycle: WindowLifecycle::visible(),
                        viewport: None,
                    },
                );
            }
        }
    }

    /// Drain one native host checkpoint through the same semantic locator and
    /// input machinery used by GPUI behavior tests.
    #[doc(hidden)]
    pub fn poll_native_host(&self, window_key: WindowKey, host: &mut dyn NativeTestHost) -> bool {
        if let Some(viewport) = host.window_viewport(window_key)
            && let Ok(mut state) = self.state.lock()
            && let Some(window) = state.windows.get_mut(&window_key)
        {
            window.viewport = Some(viewport);
        }
        let mut handled = self.poll_native_window_action(window_key, host);
        let snapshot = host.semantic_snapshot(window_key);
        if let Some(snapshot) = snapshot.as_ref()
            && let Ok(mut state) = self.state.lock()
        {
            state
                .semantic_snapshots
                .insert(window_key, snapshot.clone());
        }
        let mut source = NativeSemanticSource { host, window_key };
        handled |= self.poll_semantic_source(window_key, snapshot, &mut source);
        handled
    }

    fn poll_native_window_action(
        &self,
        window_key: WindowKey,
        host: &mut dyn NativeTestHost,
    ) -> bool {
        let action = self.state.lock().ok().and_then(|mut state| {
            let index = state.actions.iter().position(|action| {
                matches!(
                    &action.kind,
                    TestActionKind::NativeClose { window_key: target, .. }
                        | TestActionKind::ShowWindow(target)
                        | TestActionKind::ResizeWindow { window_key: target, .. }
                        if *target == window_key
                )
            })?;
            state.actions.remove(index)
        });
        let Some(action) = action else {
            return false;
        };
        let handled = match &action.kind {
            TestActionKind::NativeClose {
                mutable_visibility, ..
            } => host.hide_window(window_key, *mutable_visibility),
            TestActionKind::ShowWindow(_) => host.show_window(window_key),
            TestActionKind::ResizeWindow { width, height, .. } => {
                host.resize_window(window_key, *width, *height)
            }
            _ => unreachable!("native window action filter and mapping stay aligned"),
        };
        if handled && let Ok(mut state) = self.state.lock() {
            let snapshot = state.windows.entry(window_key).or_insert(WindowSnapshot {
                lifecycle: WindowLifecycle::visible(),
                viewport: None,
            });
            match &action.kind {
                TestActionKind::NativeClose {
                    mutable_visibility, ..
                } => {
                    let _ = snapshot.lifecycle.transition(
                        WindowIntent::Hide,
                        WindowCapabilities {
                            mutable_visibility: *mutable_visibility,
                        },
                    );
                }
                TestActionKind::ShowWindow(_) => {
                    let _ = snapshot
                        .lifecycle
                        .transition(WindowIntent::Show, WindowCapabilities::default());
                }
                TestActionKind::ResizeWindow { width, height, .. } => {
                    snapshot.viewport = Some((*width, *height));
                }
                _ => unreachable!("native window action filter and state update stay aligned"),
            }
        }
        let _ = action.completion.send(TestActionResult::Handled(handled));
        true
    }

    fn request(&self, kind: TestActionKind) -> oneshot::Receiver<TestActionResult> {
        let (completion, receiver) = oneshot::channel();
        let wake = {
            let Ok(mut state) = self.state.lock() else {
                return receiver;
            };
            if state.report.is_some() {
                let _ = completion.send(cancelled_result(&kind));
                return receiver;
            }
            let action = TestAction { kind, completion };
            #[cfg(test)]
            if state.headless {
                if action_requires_source_poll(&action.kind) {
                    state.actions.push_back(action);
                } else {
                    apply_headless_action(&mut state, action);
                }
            } else {
                state.actions.push_back(action);
            }
            #[cfg(not(test))]
            state.actions.push_back(action);
            state.wake.clone()
        };
        if let Some(wake) = wake {
            wake();
        }
        receiver
    }

    fn finish(&self, report: String) -> bool {
        let wake = {
            let Ok(mut state) = self.state.lock() else {
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

    /// Mount the private behavior-test capability into one JavaScript runtime.
    #[doc(hidden)]
    pub fn mount(&self, js: &crate::JsRuntime) -> rquickjs::Result<()> {
        let controller = self.clone();
        js.mount_capability(CAPABILITY, move |ctx, capability| {
            controller.mount_object(ctx, capability)
        })
    }

    /// Populate a test capability object owned by another QuickJS host.
    #[doc(hidden)]
    pub fn mount_object<'js>(
        &self,
        ctx: Ctx<'js>,
        capability: Object<'js>,
    ) -> rquickjs::Result<()> {
        let controller = self.clone();
        let fixtures = controller.fixtures.clone();
        capability.set(
            "writeTextFile",
            Function::new(
                ctx.clone(),
                move |relative: String, contents: String| match fixtures
                    .write_text(&relative, &contents)
                {
                    Ok(path) => serde_json::json!({
                        "path": path.to_string_lossy(),
                    })
                    .to_string(),
                    Err(error) => serde_json::json!({ "error": error }).to_string(),
                },
            )?,
        )?;

        let native_close = controller.clone();
        capability.set(
            "nativeClose",
            Function::new(
                ctx.clone(),
                Async(move |lo: u32, hi: u32, mutable_visibility: bool| {
                    let receiver = window_key(lo, hi).map(|window_key| {
                        native_close.request(TestActionKind::NativeClose {
                            window_key,
                            mutable_visibility,
                        })
                    });
                    async move {
                        match receiver {
                            Some(receiver) => {
                                matches!(receiver.await, Ok(TestActionResult::Handled(true)))
                            }
                            None => false,
                        }
                    }
                }),
            )?,
        )?;

        let show = controller.clone();
        capability.set(
            "showWindow",
            Function::new(
                ctx.clone(),
                Async(move |lo: u32, hi: u32| {
                    let receiver = window_key(lo, hi)
                        .map(|window_key| show.request(TestActionKind::ShowWindow(window_key)));
                    async move {
                        match receiver {
                            Some(receiver) => {
                                matches!(receiver.await, Ok(TestActionResult::Handled(true)))
                            }
                            None => false,
                        }
                    }
                }),
            )?,
        )?;

        let idle = controller.clone();
        capability.set(
            "waitForIdle",
            Function::new(
                ctx.clone(),
                Async(move |lo: u32, hi: u32| {
                    let receiver = window_key(lo, hi)
                        .map(|window_key| idle.request(TestActionKind::WaitForIdle(window_key)));
                    async move {
                        match receiver {
                            Some(receiver) => {
                                matches!(receiver.await, Ok(TestActionResult::Handled(true)))
                            }
                            None => false,
                        }
                    }
                }),
            )?,
        )?;

        let query = controller.clone();
        capability.set(
            "resizeWindow",
            Function::new(
                ctx.clone(),
                Async(move |lo: u32, hi: u32, width: u32, height: u32| {
                    let receiver = window_key(lo, hi).map(|window_key| {
                        query.request(TestActionKind::ResizeWindow {
                            window_key,
                            width,
                            height,
                        })
                    });
                    async move {
                        match receiver {
                            Some(receiver) => {
                                matches!(receiver.await, Ok(TestActionResult::Handled(true)))
                            }
                            None => false,
                        }
                    }
                }),
            )?,
        )?;

        let query = controller.clone();
        capability.set(
            "windowState",
            Function::new(ctx.clone(), move |lo: u32, hi: u32| {
                window_key(lo, hi)
                    .map(|window_key| query.window_state_json(window_key))
                    .unwrap_or_else(|| "null".into())
            })?,
        )?;

        let file_drop = controller.clone();
        capability.set(
            "fileDrop",
            Function::new(
                ctx.clone(),
                Async(move |lo: u32, hi: u32, phase: String, paths_json: String| {
                    let phase = match phase.as_str() {
                        "entered" => Some(FileDropPhase::Entered),
                        "moved" => Some(FileDropPhase::Moved),
                        "left" => Some(FileDropPhase::Left),
                        "dropped" => Some(FileDropPhase::Dropped),
                        _ => None,
                    };
                    let paths = serde_json::from_str::<Vec<PathBuf>>(&paths_json).ok();
                    let receiver = window_key(lo, hi).zip(phase).zip(paths).map(
                        |((window_key, phase), paths)| {
                            file_drop.request(TestActionKind::FileDrop {
                                window_key,
                                phase,
                                paths,
                            })
                        },
                    );
                    async move {
                        match receiver {
                            Some(receiver) => {
                                matches!(receiver.await, Ok(TestActionResult::Handled(true)))
                            }
                            None => false,
                        }
                    }
                }),
            )?,
        )?;

        let query = controller.clone();
        capability.set(
            "windowViewport",
            Function::new(ctx.clone(), move |lo: u32, hi: u32| {
                window_key(lo, hi)
                    .map(|window_key| query.window_viewport_json(window_key))
                    .unwrap_or_else(|| "null".into())
            })?,
        )?;

        let click = controller.clone();
        capability.set(
            "clickByRole",
            Function::new(
                ctx.clone(),
                Async(
                    move |lo: u32,
                          hi: u32,
                          role: String,
                          label: String,
                          index: Option<usize>,
                          scope_json: String| {
                        let receiver = window_key(lo, hi).and_then(|window_key| {
                            let scope = serde_json::from_str(&scope_json).ok()?;
                            Some(click.request(TestActionKind::ClickByRole {
                                window_key,
                                role,
                                label,
                                index,
                                scope,
                            }))
                        });
                        async move {
                            match receiver {
                                Some(receiver) => {
                                    matches!(receiver.await, Ok(TestActionResult::Handled(true)))
                                }
                                None => false,
                            }
                        }
                    },
                ),
            )?,
        )?;

        let input = controller.clone();
        capability.set(
            "inputByRole",
            Function::new(
                ctx.clone(),
                Async(
                    move |lo: u32,
                          hi: u32,
                          role: String,
                          label: String,
                          raw: String,
                          index: Option<usize>,
                          scope_json: String| {
                        let receiver = window_key(lo, hi).and_then(|window_key| {
                            let scope = serde_json::from_str(&scope_json).ok()?;
                            serde_json::from_str::<TestInput>(&raw).ok().map(|action| {
                                input.request(TestActionKind::InputByRole {
                                    window_key,
                                    role,
                                    label,
                                    input: action,
                                    index,
                                    scope,
                                })
                            })
                        });
                        async move {
                            match receiver {
                                Some(receiver) => {
                                    matches!(receiver.await, Ok(TestActionResult::Handled(true)))
                                }
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

        let effects = controller.clone();
        capability.set(
            "queueEffect",
            Function::new(
                ctx.clone(),
                move |capability: u32, method: u16, result_json: String| {
                    let result = serde_json::from_str::<wabou_shell::EffectResult>(&result_json)
                        .map_err(|error| format!("invalid effect fixture result: {error}"));
                    result
                        .and_then(|result| {
                            effects.effects.enqueue_fixture(
                                wabou_shell::EffectOp::new(capability, method),
                                result,
                            )
                        })
                        .err()
                },
            )?,
        )?;

        let effects = controller.clone();
        capability.set(
            "takePendingEffectFixtures",
            Function::new(ctx.clone(), move || {
                effects
                    .effects
                    .take_pending_fixtures()
                    .into_iter()
                    .map(|op| format!("{}:{}", op.capability.0, op.method.0))
                    .collect::<Vec<_>>()
                    .join(",")
            })?,
        )?;

        let query = controller.clone();
        capability.set(
            "queryByRole",
            Function::new(
                ctx.clone(),
                Async(
                    move |lo: u32,
                          hi: u32,
                          role: String,
                          label: String,
                          index: Option<usize>,
                          scope_json: String| {
                        let receiver = window_key(lo, hi).and_then(|window_key| {
                            let scope = serde_json::from_str(&scope_json).ok()?;
                            Some(query.request(TestActionKind::QueryByRole {
                                window_key,
                                role,
                                label,
                                index,
                                scope,
                            }))
                        });
                        async move {
                            match receiver {
                                Some(receiver) => match receiver.await {
                                    Ok(TestActionResult::Query(result)) => result,
                                    _ => None,
                                },
                                _ => None,
                            }
                        }
                    },
                ),
            )?,
        )?;
        Ok(())
    }

    fn window_state_json(&self, window_key: WindowKey) -> String {
        let snapshot = self
            .state
            .lock()
            .ok()
            .and_then(|state| state.windows.get(&window_key).copied());
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

    fn window_viewport_json(&self, window_key: WindowKey) -> String {
        let viewport = self.state.lock().ok().and_then(|state| {
            #[cfg(test)]
            if let Some(viewport) = state.headless_viewports.get(&window_key) {
                return Some(*viewport);
            }
            state.windows.get(&window_key)?.viewport
        });
        viewport.map_or_else(
            || "null".into(),
            |(width, height)| format!(r#"{{"x":0,"y":0,"width":{width},"height":{height}}}"#),
        )
    }

    /// Take the scenario's final structured report once.
    #[doc(hidden)]
    pub fn take_report(&self) -> Option<String> {
        self.state.lock().ok()?.report.take()
    }

    /// Return whether the scenario published its final report.
    #[doc(hidden)]
    pub fn has_report(&self) -> bool {
        self.state.lock().is_ok_and(|state| state.report.is_some())
    }

    /// Return the pass flag without consuming the final report.
    #[doc(hidden)]
    pub fn report_passed(&self) -> Option<bool> {
        let state = self.state.lock().ok()?;
        let report = state.report.as_deref()?;
        serde_json::from_str::<serde_json::Value>(report)
            .ok()?
            .get("passed")?
            .as_bool()
    }

    #[cfg(test)]
    pub(crate) fn initialize_headless(
        &self,
        window_keys: impl IntoIterator<Item = WindowKey>,
        width: u32,
        height: u32,
    ) {
        if let Ok(mut state) = self.state.lock() {
            state.headless = true;
            for window_key in window_keys {
                state.windows.insert(
                    window_key,
                    WindowSnapshot {
                        lifecycle: WindowLifecycle::visible(),
                        viewport: Some((width, height)),
                    },
                );
                state.headless_viewports.insert(window_key, (width, height));
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn headless_viewport(&self, window_key: WindowKey) -> Option<(u32, u32)> {
        self.state
            .lock()
            .ok()?
            .headless_viewports
            .get(&window_key)
            .copied()
    }

    #[cfg(test)]
    fn poll_headless_source(&self, window_key: WindowKey, source: &mut dyn SemanticTestSource) {
        let snapshot = source.semantic_snapshot();
        let _ = self.poll_semantic_source(window_key, snapshot, source);
    }

    fn poll_semantic_source(
        &self,
        window_key: WindowKey,
        snapshot: Option<Arc<SemanticSnapshot>>,
        source: &mut dyn SemanticTestSource,
    ) -> bool {
        if let Some(snapshot) = snapshot.as_ref() {
            self.record_semantic_snapshot(window_key, snapshot.clone());
        }
        if let Some(snapshot) = snapshot.as_deref()
            && let Some(action) = self.state.lock().ok().and_then(|state| {
                state.actions.iter().find_map(|action| match &action.kind {
                    TestActionKind::ClickByRole {
                        window_key: target_window,
                        role,
                        label,
                        index,
                        scope,
                    } if *target_window == window_key => {
                        semantic_target(snapshot, role, label, *index, scope)
                    }
                    _ => None,
                })
            })
            && source.handle_semantic_action(wabou_shell::SemanticAction::ScrollIntoView {
                target: action.id,
            })
        {
            return true;
        }
        let action = self.state.lock().ok().and_then(|mut state| {
            let index = state.actions.iter().position(|action| {
                matches!(
                    action_window_key(&action.kind),
                    Some(target) if target == window_key && action_requires_source_poll(&action.kind)
                )
            })?;
            let action = state.actions.get(index)?;
            let ready = action_ready(&action.kind, snapshot.as_deref());
            ready.then(|| state.actions.remove(index)).flatten()
        });
        let Some(action) = action else {
            return false;
        };
        let handled = match (&action.kind, snapshot.as_deref()) {
            (TestActionKind::WaitForIdle(_), _) => true,
            (TestActionKind::FileDrop { phase, paths, .. }, _) => {
                source.handle_event(UiEvent::FileDrop(FileDropEvent {
                    phase: (*phase).into(),
                    paths: paths.clone(),
                    position: None,
                }));
                true
            }
            (
                TestActionKind::ClickByRole {
                    role,
                    label,
                    index,
                    scope,
                    ..
                },
                Some(snapshot),
            ) => click_semantic_target(source, snapshot, role, label, *index, scope),
            (
                TestActionKind::InputByRole {
                    role,
                    label,
                    input,
                    index,
                    scope,
                    ..
                },
                Some(snapshot),
            ) => input_semantic_target(source, snapshot, role, label, input, *index, scope),
            _ => false,
        };
        let result = match (&action.kind, snapshot.as_deref()) {
            (
                TestActionKind::QueryByRole {
                    role,
                    label,
                    index,
                    scope,
                    ..
                },
                Some(snapshot),
            ) => TestActionResult::Query(locator_query_json(snapshot, role, label, *index, scope)),
            _ => TestActionResult::Handled(handled),
        };
        let _ = action.completion.send(result);
        true
    }

    fn record_semantic_snapshot(&self, window_key: WindowKey, snapshot: Arc<SemanticSnapshot>) {
        if let Ok(mut state) = self.state.lock() {
            state.semantic_snapshots.insert(window_key, snapshot);
        }
    }

    /// Build a redacted semantic artifact for a failed native scenario.
    #[doc(hidden)]
    pub fn semantic_artifact(&self) -> serde_json::Value {
        let snapshots = self
            .state
            .lock()
            .map(|state| state.semantic_snapshots.clone())
            .unwrap_or_default();
        let mut windows = snapshots.into_iter().collect::<Vec<_>>();
        windows.sort_unstable_by_key(|(window_key, _)| window_key.as_ffi());
        serde_json::json!({
            "version": 1,
            "windows": windows
                .into_iter()
                .map(|(window_key, snapshot)| semantic_snapshot_json(window_key, &snapshot))
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

fn semantic_toggle_json(state: Option<wabou_shell::SemanticToggleState>) -> serde_json::Value {
    match state.map(wabou_shell::SemanticToggleState::as_str) {
        Some("false") => serde_json::Value::Bool(false),
        Some("true") => serde_json::Value::Bool(true),
        Some("mixed") => serde_json::Value::String("mixed".into()),
        Some(value) => unreachable!("unknown semantic toggle state {value}"),
        None => serde_json::Value::Null,
    }
}

fn locator_snapshot_json(node: &wabou_shell::SemanticNode, focused: bool) -> String {
    let current = node
        .states
        .current
        .map(wabou_shell::SemanticCurrent::as_str);
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
            "checked": semantic_toggle_json(node.states.checked),
            "pressed": semantic_toggle_json(node.states.pressed),
            "selected": node.states.selected,
            "current": current,
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
    scope: &[TestLocatorSelector],
) -> Option<String> {
    let role = SemanticRole::from_name(role)?;
    let matches = scoped_candidates(snapshot, scope)?
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

fn scoped_candidates<'a>(
    snapshot: &'a SemanticSnapshot,
    scope: &[TestLocatorSelector],
) -> Option<Vec<&'a wabou_shell::SemanticNode>> {
    let mut candidates = snapshot.exposed_nodes();
    for selector in scope {
        let role = SemanticRole::from_name(&selector.role)?;
        let matches = candidates
            .into_iter()
            .filter(|node| node.role == role && node.label.as_deref() == Some(&selector.name))
            .collect::<Vec<_>>();
        let owner = match selector.index {
            Some(index) => matches.get(index).copied(),
            None if matches.len() == 1 => matches.first().copied(),
            None => None,
        }?;
        candidates = semantic_descendants(snapshot, owner.id);
    }
    Some(candidates)
}

fn semantic_descendants(
    snapshot: &SemanticSnapshot,
    owner: u64,
) -> Vec<&wabou_shell::SemanticNode> {
    let by_id = snapshot
        .nodes
        .iter()
        .map(|node| (node.id, node))
        .collect::<HashMap<_, _>>();
    let mut stack = by_id
        .get(&owner)
        .into_iter()
        .flat_map(|node| node.children.iter().rev().copied())
        .collect::<Vec<_>>();
    let mut descendants = std::collections::HashSet::new();
    while let Some(id) = stack.pop() {
        if !descendants.insert(id) {
            continue;
        }
        if let Some(node) = by_id.get(&id) {
            stack.extend(node.children.iter().rev().copied());
        }
    }
    snapshot
        .nodes
        .iter()
        .filter(|node| descendants.contains(&node.id))
        .collect()
}

fn semantic_snapshot_json(window_key: WindowKey, snapshot: &SemanticSnapshot) -> serde_json::Value {
    serde_json::json!({
        "windowId": window_key,
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
            "checked": semantic_toggle_json(node.states.checked),
            "pressed": semantic_toggle_json(node.states.pressed),
            "selected": node.states.selected,
            "current": node.states.current.map(wabou_shell::SemanticCurrent::as_str),
            "expanded": node.states.expanded,
            "focused": snapshot.focus == Some(node.id),
        })).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
fn apply_headless_action(state: &mut TestState, action: TestAction) {
    let handled = match action.kind {
        TestActionKind::NativeClose {
            window_key,
            mutable_visibility,
        } => state.windows.get_mut(&window_key).is_some_and(|snapshot| {
            snapshot.lifecycle.transition(
                WindowIntent::Hide,
                WindowCapabilities { mutable_visibility },
            );
            true
        }),
        TestActionKind::ShowWindow(window_key) => {
            state.windows.get_mut(&window_key).is_some_and(|snapshot| {
                snapshot
                    .lifecycle
                    .transition(WindowIntent::Show, WindowCapabilities::default());
                true
            })
        }
        TestActionKind::ResizeWindow {
            window_key,
            width,
            height,
        } => state
            .headless_viewports
            .get_mut(&window_key)
            .is_some_and(|viewport| {
                *viewport = (width, height);
                true
            }),
        TestActionKind::FileDrop { .. } => false,
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
    matches!(
        kind,
        TestActionKind::WaitForIdle(_) | TestActionKind::FileDrop { .. }
    ) || action_requires_semantics(kind)
}

fn action_window_key(kind: &TestActionKind) -> Option<WindowKey> {
    match kind {
        TestActionKind::WaitForIdle(window_key)
        | TestActionKind::ResizeWindow { window_key, .. }
        | TestActionKind::FileDrop { window_key, .. }
        | TestActionKind::ClickByRole { window_key, .. }
        | TestActionKind::InputByRole { window_key, .. }
        | TestActionKind::QueryByRole { window_key, .. } => Some(*window_key),
        _ => None,
    }
}

fn action_ready(kind: &TestActionKind, snapshot: Option<&SemanticSnapshot>) -> bool {
    match (kind, snapshot) {
        (TestActionKind::WaitForIdle(_), _) => true,
        (TestActionKind::FileDrop { .. }, _) => true,
        (kind, Some(_)) if action_requires_semantics(kind) => true,
        _ => false,
    }
}

fn click_semantic_target(
    source: &mut dyn SemanticTestSource,
    snapshot: &SemanticSnapshot,
    role: &str,
    label: &str,
    index: Option<usize>,
    scope: &[TestLocatorSelector],
) -> bool {
    let Some(node) = semantic_target(snapshot, role, label, index, scope) else {
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
        properties: wabou_shell::PointerProperties::default(),
    }));
    source.handle_event(UiEvent::Pointer(PointerEvent {
        phase: PointerPhase::Up,
        position: point,
        button: Some(PointerButton::Primary),
        buttons: 0,
        modifiers: Modifiers::default(),
        properties: wabou_shell::PointerProperties::default(),
    }));
    true
}

fn input_semantic_target(
    source: &mut dyn SemanticTestSource,
    snapshot: &SemanticSnapshot,
    role: &str,
    label: &str,
    input: &TestInput,
    index: Option<usize>,
    scope: &[TestLocatorSelector],
) -> bool {
    let node = if input_allows_disabled_target(input) {
        semantic_query_target(snapshot, role, label, index, scope)
    } else {
        semantic_target(snapshot, role, label, index, scope)
    };
    let Some(node) = node else {
        return false;
    };
    dispatch_test_input(source, node, input)
}

fn input_allows_disabled_target(input: &TestInput) -> bool {
    matches!(input, TestInput::Probe | TestInput::Wheel { .. })
}

fn dispatch_test_input(
    source: &mut dyn SemanticTestSource,
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
    scope: &[TestLocatorSelector],
) -> Option<&'a wabou_shell::SemanticNode> {
    let node = semantic_query_target(snapshot, role, label, index, scope)?;
    (!node.disabled).then_some(node)
}

fn semantic_query_target<'a>(
    snapshot: &'a SemanticSnapshot,
    role: &str,
    label: &str,
    index: Option<usize>,
    scope: &[TestLocatorSelector],
) -> Option<&'a wabou_shell::SemanticNode> {
    let role = SemanticRole::from_name(role)?;
    let mut matches = scoped_candidates(snapshot, scope)?
        .into_iter()
        .filter(|node| node.role == role && node.label.as_deref() == Some(label));
    match index {
        Some(index) => matches.nth(index),
        None => {
            let node = matches.next()?;
            matches.next().is_none().then_some(node)
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
                    properties: wabou_shell::PointerProperties::default(),
                }),
                UiEvent::Pointer(PointerEvent {
                    phase: PointerPhase::Move,
                    position: end,
                    button: Some(PointerButton::Primary),
                    buttons: 1,
                    modifiers: Modifiers::default(),
                    properties: wabou_shell::PointerProperties::default(),
                }),
                UiEvent::Pointer(PointerEvent {
                    phase: PointerPhase::Up,
                    position: end,
                    button: Some(PointerButton::Primary),
                    buttons: 0,
                    modifiers: Modifiers::default(),
                    properties: wabou_shell::PointerProperties::default(),
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
                    synthetic: false,
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
            delta_mode: wabou_shell::WheelDeltaMode::Pixel,
            phase: wabou_shell::GesturePhase::Changed,
            modifiers: Modifiers::default(),
        })],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(lo: u32) -> WindowKey {
        WindowKey::from_parts(lo, 1).unwrap()
    }

    struct SemanticSource(Arc<SemanticSnapshot>);

    impl SemanticTestSource for SemanticSource {
        fn semantic_snapshot(&self) -> Option<Arc<SemanticSnapshot>> {
            Some(self.0.clone())
        }

        fn handle_event(&mut self, _event: UiEvent) {}

        fn handle_semantic_action(&mut self, _action: wabou_shell::SemanticAction) -> bool {
            false
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
            key(2),
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
        assert_eq!(
            artifact["windows"][0]["windowId"],
            serde_json::json!({ "lo": 2, "hi": 1 })
        );
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
        let mut node = node();
        node.states.current = Some(wabou_shell::SemanticCurrent::Page);
        let snapshot =
            serde_json::from_str::<serde_json::Value>(&locator_snapshot_json(&node, false))
                .unwrap();
        assert_eq!(
            snapshot["bounds"],
            serde_json::json!({ "x": 10.0, "y": 20.0, "width": 100.0, "height": 40.0 })
        );
        assert_eq!(snapshot["current"], "page");
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
    fn wheel_can_target_disabled_content_but_actions_cannot() {
        assert!(input_allows_disabled_target(&TestInput::Wheel {
            delta_x: 0.0,
            delta_y: 40.0,
        }));
        assert!(input_allows_disabled_target(&TestInput::Probe));
        assert!(!input_allows_disabled_target(&TestInput::Drag {
            delta_x: 1.0,
            delta_y: 1.0,
        }));
        assert!(!input_allows_disabled_target(&TestInput::Key {
            key: "Enter".into(),
            modifiers: 0,
        }));
    }

    #[test]
    fn text_fixtures_are_exact_isolated_and_cleaned_up() {
        let fixtures = TestFixtureDirectory::new();
        let root = fixtures.path.clone();
        let path = fixtures
            .write_text("nested/sample.torrent", "d3:fooi1ee")
            .unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"d3:fooi1ee");
        assert!(fixtures.write_text("../escape", "no").is_err());
        assert!(fixtures.write_text("/absolute", "no").is_err());
        drop(fixtures);
        assert!(!root.exists());
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

        assert!(semantic_target(&snapshot, "alert", "Failed", None, &[]).is_some());
        assert!(semantic_target(&snapshot, "status", "Saved", None, &[]).is_some());
    }

    #[test]
    fn headless_locators_support_repository_component_roles() {
        let roles = [
            ("group", SemanticRole::Group),
            ("img", SemanticRole::Image),
            ("radiogroup", SemanticRole::RadioGroup),
            ("region", SemanticRole::Region),
            ("menu", SemanticRole::Menu),
            ("menubar", SemanticRole::MenuBar),
            ("menuitem", SemanticRole::MenuItem),
            ("tree", SemanticRole::Tree),
            ("treeitem", SemanticRole::TreeItem),
            ("toolbar", SemanticRole::Toolbar),
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
                semantic_target(&snapshot, role, "Target", None, &[]).is_some(),
                "missing locator role {role}"
            );
        }
    }

    #[test]
    fn headless_semantic_actions_read_each_completed_snapshot_without_native_timeout() {
        let snapshot = SemanticSnapshot::default();
        let probe = TestActionKind::InputByRole {
            window_key: key(1),
            role: "button".into(),
            label: "Appears later".into(),
            input: TestInput::Probe,
            index: None,
            scope: vec![],
        };
        let click = TestActionKind::ClickByRole {
            window_key: key(1),
            role: "button".into(),
            label: "Appears later".into(),
            index: None,
            scope: vec![],
        };

        assert!(action_ready(&probe, Some(&snapshot)));
        assert!(action_ready(&click, Some(&snapshot)));
    }

    #[test]
    fn concurrent_queries_keep_results_attached_to_their_requests() {
        let controller = TestController::default();
        controller.initialize_headless([key(1), key(2)], 800, 600);
        let first = controller.request(TestActionKind::QueryByRole {
            window_key: key(1),
            role: "textbox".into(),
            label: "First".into(),
            index: None,
            scope: vec![],
        });
        let second = controller.request(TestActionKind::QueryByRole {
            window_key: key(2),
            role: "textbox".into(),
            label: "Second".into(),
            index: None,
            scope: vec![],
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
        controller.poll_headless_source(key(1), &mut first_source);
        controller.poll_headless_source(key(2), &mut second_source);

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
            &locator_query_json(&snapshot, "textbox", "Editor", None, &[]).unwrap(),
        )
        .unwrap();
        assert_eq!(query["matchCount"], 2);
        assert!(semantic_query_target(&snapshot, "textbox", "Editor", None, &[]).is_none());
        assert!(semantic_target(&snapshot, "textbox", "Editor", None, &[]).is_none());
        assert_eq!(
            semantic_query_target(&snapshot, "textbox", "Editor", Some(1), &[]).map(|node| node.id),
            Some(8)
        );
    }

    #[test]
    fn scoped_locators_resolve_duplicate_names_within_one_semantic_subtree() {
        let mut first_group = node();
        first_group.id = 10;
        first_group.role = SemanticRole::RadioGroup;
        first_group.label = Some("Primary rating".into());
        first_group.children = vec![11];
        let mut first_radio = node();
        first_radio.id = 11;
        first_radio.role = SemanticRole::RadioButton;
        first_radio.label = Some("4 stars".into());

        let mut second_group = first_group.clone();
        second_group.id = 20;
        second_group.label = Some("Secondary rating".into());
        second_group.children = vec![21];
        let mut second_radio = first_radio.clone();
        second_radio.id = 21;

        let snapshot = SemanticSnapshot {
            nodes: vec![first_group, first_radio, second_group, second_radio],
            root_children: vec![10, 20],
            ..SemanticSnapshot::default()
        };
        assert!(semantic_query_target(&snapshot, "radio", "4 stars", None, &[]).is_none());

        let scope = [TestLocatorSelector {
            role: "radiogroup".into(),
            name: "Secondary rating".into(),
            index: None,
        }];
        assert_eq!(
            semantic_query_target(&snapshot, "radio", "4 stars", None, &scope).map(|node| node.id),
            Some(21)
        );
        let query = locator_query_json(&snapshot, "radio", "4 stars", None, &scope).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&query).unwrap()["matchCount"],
            1
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

        assert!(locator_query_json(&snapshot, "button", "Background", None, &[]).is_none());
        assert!(semantic_target(&snapshot, "button", "Background", None, &[]).is_none());
        assert!(semantic_target(&snapshot, "dialog", "Settings", None, &[]).is_some());
    }

    #[test]
    fn deterministic_windows_accept_runtime_logical_resizes() {
        let controller = TestController::default();
        controller.initialize_headless([key(1)], 1100, 720);

        let result = controller.request(TestActionKind::ResizeWindow {
            window_key: key(1),
            width: 900,
            height: 600,
        });
        assert!(matches!(
            result.blocking_recv(),
            Ok(TestActionResult::Handled(true))
        ));
        assert_eq!(controller.headless_viewport(key(1)), Some((900, 600)));
        assert_eq!(
            controller.window_viewport_json(key(1)),
            r#"{"x":0,"y":0,"width":900,"height":600}"#
        );
    }

    #[test]
    fn finishing_cancels_pending_and_future_actions_without_replacing_the_report() {
        let controller = TestController::default();
        let pending_input = controller.request(TestActionKind::ClickByRole {
            window_key: key(1),
            role: "button".into(),
            label: "Late action".into(),
            index: None,
            scope: vec![],
        });
        let pending_query = controller.request(TestActionKind::QueryByRole {
            window_key: key(1),
            role: "textbox".into(),
            label: "Late query".into(),
            index: None,
            scope: vec![],
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

        let future = controller.request(TestActionKind::WaitForIdle(key(1)));
        assert!(matches!(
            future.blocking_recv(),
            Ok(TestActionResult::Handled(false))
        ));
        let state = controller.state.lock().unwrap();
        assert!(state.actions.is_empty());
        assert_eq!(state.report.as_deref(), Some("first report"));
    }
}
