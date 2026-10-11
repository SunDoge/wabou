//! Independent native windows share an event loop, never plugin-owned callbacks.
use blockui_runtime::{
    App, Message,
    app::WindowOptions,
    native::{NativeSource, ReloadConfig},
    native_model::{NativeFamily, NativeModel},
};
use std::{cell::RefCell, path::PathBuf, rc::Rc};
use wabou_shell_vello::{self as shell, FrameSource};

#[derive(Default)]
struct Status {
    closed: [bool; 2],
    keys: [Option<shell::WindowResourceKey>; 2],
    wakes: [Option<shell::WakeCallback>; 2],
    verified: [bool; 2],
    failure_seen: bool,
    second_verified: bool,
}
struct Probe {
    source: NativeSource<NativeFamily>,
    index: usize,
    replacement: PathBuf,
    status: Rc<RefCell<Status>>,
    phase: u8,
    close: bool,
    dirty: bool,
    first_closed: bool,
    state: usize,
    slot: usize,
    version: u64,
    watch: bool,
    second_handled: bool,
    resized: bool,
}
impl Drop for Probe {
    fn drop(&mut self) {
        let mut status = self.status.borrow_mut();
        status.closed[self.index] = true;
        for wake in status.wakes.iter().flatten() {
            wake();
        }
    }
}
impl FrameSource for Probe {
    fn set_device_scale(&mut self, scale: f64) {
        self.source.set_device_scale(scale);
    }
    fn set_wake_callback(&mut self, wake: shell::WakeCallback) {
        self.status.borrow_mut().wakes[self.index] = Some(wake.clone());
        self.source.set_wake_callback(wake);
    }
    fn handle_event(&mut self, event: shell::UiEvent) -> shell::EventResponse {
        self.source.handle_event(event)
    }
    fn prepare_for_event(&mut self, cx: &mut shell::TextContext) {
        self.source.prepare_for_event(cx);
    }
    fn poll_async(&mut self) -> bool {
        let mut changed = self.source.poll_async();
        if self.watch
            && self.index == 0
            && self.phase == 10
            && self
                .source
                .diagnostic
                .contains("intentional multi-window failure")
        {
            assert_eq!(
                self.source.actor.root_actor().implementation_version(),
                self.version
            );
            assert_eq!(self.source.actor.root_actor().integer("count"), 1);
            self.source
                .actor
                .root_actor_mut()
                .dispatch(Message {
                    action: blockui_abi::key("increment"),
                    value: 0,
                })
                .unwrap();
            self.source.actor.tick().unwrap();
            assert_eq!(self.source.actor.root_actor().integer("count"), 2);
            self.phase = 11;
            self.status.borrow_mut().failure_seen = true;
            for wake in self.status.borrow().wakes.iter().flatten() {
                wake();
            }
            println!("MULTI_WATCH_FAILED_OLD_CODE_RUNNING");
            changed = true;
        }
        if self.watch
            && self.index == 1
            && !self.second_handled
            && self.status.borrow().failure_seen
        {
            assert_eq!(
                self.source.actor.root_actor().implementation_version(),
                self.version
            );
            assert_eq!(self.source.actor.root_actor().integer("count"), 0);
            self.source
                .actor
                .root_actor_mut()
                .dispatch(Message {
                    action: blockui_abi::key("increment"),
                    value: 0,
                })
                .unwrap();
            self.source.actor.tick().unwrap();
            self.second_handled = true;
            changed = true;
        }
        if self.watch
            && self.index == 0
            && self.phase == 11
            && self.source.actor.root_actor().implementation_version() != self.version
        {
            assert_eq!(self.source.actor.root_actor().integer("count"), 2);
            self.source
                .actor
                .root_actor_mut()
                .dispatch(Message {
                    action: blockui_abi::key("increment"),
                    value: 0,
                })
                .unwrap();
            self.source.actor.tick().unwrap();
            assert_eq!(self.source.actor.root_actor().integer("count"), 4);
            self.phase = 1;
            changed = true;
        }
        let closed = self.index == 1 && self.status.borrow().closed[0];
        changed |= closed && !self.first_closed;
        self.first_closed |= closed;
        changed | std::mem::take(&mut self.dirty)
    }
    fn base_color(&self) -> vello_common::peniko::Color {
        self.source.base_color()
    }
    fn build_frame(
        &mut self,
        cx: &mut shell::TextContext,
        w: u32,
        h: u32,
    ) -> Vec<shell::layout::PlacedNode> {
        self.source.build_frame(cx, w, h)
    }
    fn push_frame_stats(&mut self, _: &shell::FrameStats) {
        let metrics = self.source.metrics.expect("real window metrics");
        {
            let mut status = self.status.borrow_mut();
            if let Some(key) = status.keys[self.index] {
                assert_eq!(
                    metrics.window_key, key,
                    "reload recreated the native window"
                );
            } else {
                status.keys[self.index] = Some(metrics.window_key);
            }
        }
        if self.watch
            && self.index == 0
            && self.phase == 10
            && metrics.logical_width == 480
            && metrics.logical_height == 360
            && !self.resized
        {
            self.resized = true;
            println!("MULTI_WATCH_RESIZED");
        }
        let actor = self.source.actor.root_actor();
        assert_eq!(actor.state_address(), self.state);
        assert_eq!(actor.slot_address("notes"), self.slot);
        assert_eq!(
            actor.text("notes"),
            if self.index == 0 {
                "first window 😀"
            } else {
                "second window 😀"
            }
        );
        assert!(self.source.actor.view().bindings.values().any(|binding| {
            binding.node.kind == blockui_abi::ENTRY && binding.text == actor.text("notes")
        }));
        assert_eq!(
            metrics.logical_width,
            if self.index == 0 {
                if self.resized { 480 } else { 400 }
            } else {
                560
            }
        );
        assert_eq!(
            metrics.logical_height,
            if self.index == 0 {
                if self.resized { 360 } else { 300 }
            } else {
                400
            }
        );
        assert_eq!(
            metrics.physical_width,
            (f64::from(metrics.logical_width) * metrics.scale_factor).round() as u32
        );
        assert_eq!(
            metrics.physical_height,
            (f64::from(metrics.logical_height) * metrics.scale_factor).round() as u32
        );
        if self.watch
            && self.index == 1
            && self.second_handled
            && !self.status.borrow().second_verified
        {
            assert!(
                self.source
                    .actor
                    .view()
                    .bindings
                    .values()
                    .any(|binding| binding.text.starts_with("Count: 1"))
            );
            self.status.borrow_mut().second_verified = true;
            println!("MULTI_WATCH_OTHER_WINDOW_RUNNING");
        }
        if self.index == 0 && self.phase == 0 {
            self.source
                .actor
                .root_actor_mut()
                .dispatch(Message {
                    action: blockui_abi::key("increment"),
                    value: 0,
                })
                .unwrap();
            if self.watch {
                self.source.actor.tick().unwrap();
                self.phase = 10;
                println!("MULTI_WATCH_READY");
                return;
            }
            self.source.replace(&self.replacement).unwrap();
            assert_ne!(
                self.source.actor.root_actor().implementation_version(),
                self.version
            );
            self.source
                .actor
                .root_actor_mut()
                .dispatch(Message {
                    action: blockui_abi::key("increment"),
                    value: 0,
                })
                .unwrap();
            assert_eq!(self.source.actor.root_actor().integer("count"), 11);
            self.phase = 1;
            self.dirty = true;
            self.status.borrow().wakes[self.index].as_ref().unwrap()();
        } else if self.index == 1 && self.phase == 0 && self.status.borrow().closed[0] {
            assert_eq!(actor.implementation_version(), self.version);
            assert_eq!(actor.integer("count"), if self.watch { 1 } else { 0 });
            self.source
                .actor
                .root_actor_mut()
                .dispatch(Message {
                    action: blockui_abi::key("increment"),
                    value: 0,
                })
                .unwrap();
            self.phase = 1;
            self.dirty = true;
            self.status.borrow().wakes[self.index].as_ref().unwrap()();
        } else if self.phase == 1 {
            let expected = match (self.index, self.watch) {
                (0, true) => 4,
                (0, false) => 11,
                (_, true) => 2,
                (_, false) => 1,
            };
            assert!(
                self.source
                    .actor
                    .view()
                    .bindings
                    .values()
                    .any(|binding| binding.text.starts_with(&format!("Count: {expected}")))
            );
            self.status.borrow_mut().verified[self.index] = true;
            self.phase = 2;
            self.close = true;
            self.status.borrow().wakes[self.index].as_ref().unwrap()();
        }
    }
    fn take_effect(&mut self) -> Option<shell::EffectRequest> {
        if !std::mem::take(&mut self.close) {
            return None;
        }
        let window_id = self.source.metrics.unwrap().window_key;
        Some(shell::EffectRequest {
            id: shell::EffectId(1),
            scope: shell::effect::EffectScope::Window(window_id),
            payload: shell::EffectPayload::WindowControl {
                window_id,
                command: shell::WindowCommand::Close,
            },
        })
    }
}
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let base = args
        .get(1)
        .ok_or("multi_window <base.so> [--watch <repo> | --verify <replacement.so> <artifacts>]")?;
    let watch = args.get(2).map(String::as_str) == Some("--watch-verify");
    if !watch && args.get(2).map(String::as_str) != Some("--verify") {
        let mut first =
            App::new(base, ".runtime/first").window(WindowOptions::new().title("First counter"));
        if args.get(2).map(String::as_str) == Some("--watch") {
            let repo = args.get(3).ok_or("--watch needs the framework directory")?;
            first = first.watch(ReloadConfig::from_environment(std::path::Path::new(repo)));
        } else if args.len() > 2 {
            return Err("expected --watch or --verify".into());
        }
        return App::run_windows([
            first,
            App::new(base, ".runtime/second").window(WindowOptions::new().title("Second counter")),
        ]);
    }
    let replacement = PathBuf::from(args.get(3).ok_or("missing replacement")?);
    let artifacts = PathBuf::from(args.get(4).ok_or("missing artifacts")?);
    let rejected = App::run_windows([
        App::new(base, artifacts.join("rejected-first")),
        App::new(
            artifacts.join("missing.so"),
            artifacts.join("rejected-second"),
        ),
    ]);
    assert!(rejected.is_err());
    for entry in std::fs::read_dir(&artifacts).map_err(|e| e.to_string())? {
        assert_eq!(
            std::fs::read_dir(entry.unwrap().path()).unwrap().count(),
            0,
            "failed admission leaked an actor"
        );
    }
    let mut first = App::new(base, artifacts.join("first"));
    if watch {
        let mut config = ReloadConfig::from_environment(&replacement);
        config.snapshots = artifacts.join("watch-builds");
        first = first.watch(config);
    }
    let windows = App::into_windows([
        first.window(
            WindowOptions::new()
                .title("First counter")
                .initial_inner_size(400, 300),
        ),
        App::new(base, artifacts.join("second")).window(
            WindowOptions::new()
                .title("Second counter")
                .initial_inner_size(560, 400),
        ),
    ])?;
    // A colliding application must not take ownership of the live actor's
    // directory, unload its library or remove its generations on rejection.
    assert!(
        App::new(base, artifacts.join("first"))
            .into_window()
            .is_err()
    );
    let status = Rc::new(RefCell::new(Status::default()));
    let mut state_addresses = Vec::new();
    let windows = windows
        .into_iter()
        .enumerate()
        .map(|(index, (mut source, options))| {
            let input = source
                .actor
                .root_widget(blockui_abi::key("counter.notes"))
                .unwrap();
            source
                .actor
                .set_widget_text(
                    input,
                    if index == 0 {
                        "first window 😀"
                    } else {
                        "second window 😀"
                    },
                )
                .unwrap();
            let actor = source.actor.root_actor();
            let state = actor.state_address();
            state_addresses.push(state);
            let probe = Probe {
                state,
                slot: actor.slot_address("notes"),
                version: actor.implementation_version(),
                source,
                index,
                replacement: replacement.clone(),
                status: status.clone(),
                watch,
                second_handled: false,
                resized: false,
                phase: 0,
                close: false,
                dirty: false,
                first_closed: false,
            };
            (Box::new(probe) as Box<dyn FrameSource>, options)
        })
        .collect();
    assert_ne!(state_addresses[0], state_addresses[1]);
    shell::run_windows(windows).map_err(|e| e.to_string())?;
    let status = status.borrow();
    assert_eq!(status.verified, [true, true]);
    assert_eq!(status.closed, [true, true]);
    if watch {
        assert!(status.failure_seen && status.second_verified);
        println!("PASS multi-window watcher failure/recovery isolates old and new code");
    }
    assert_ne!(status.keys[0].unwrap(), status.keys[1].unwrap());
    println!(
        "PASS two real native windows: isolated state, one DLL replacement, first window disposal, second window continues through old code, both submitted updated frames; failed startup cleans admitted actors"
    );
    Ok(())
}
