//! Owned-display proof of shared-executor completion in an idle native window.
use blockui_runtime::{
    ActorId, Message, Runtime, native::NativeSource, native_model::NativeFamily,
    native_model::NativeModel,
};
use std::{
    future::Future,
    path::Path,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
    time::Instant,
};
use wabou_shell_vello::{self as shell, FrameSource, TextContext};

struct Gate {
    ready: bool,
    pending: bool,
    waker: Option<Waker>,
}
struct GateFuture {
    gate: Arc<std::sync::Mutex<Gate>>,
    notify: Arc<dyn Fn() + Send + Sync>,
}
impl Future for GateFuture {
    type Output = Message;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Message> {
        let mut gate = self.gate.lock().unwrap();
        if gate.ready {
            return Poll::Ready(Message {
                action: blockui_abi::key("increment"),
                value: 0,
            });
        }
        gate.waker = Some(cx.waker().clone());
        let first = !gate.pending;
        gate.pending = true;
        drop(gate);
        if first {
            (self.notify)();
        }
        Poll::Pending
    }
}
struct Probe {
    source: NativeSource<NativeFamily>,
    replacement: std::path::PathBuf,
    prequeued: bool,
    gate: Arc<std::sync::Mutex<Gate>>,
    started: Option<Instant>,
    released: bool,
    rendered: bool,
    reported: bool,
    exit: bool,
    polls: usize,
    frames: usize,
    wake: Option<Arc<dyn Fn() + Send + Sync>>,
}
impl FrameSource for Probe {
    fn build_frame(
        &mut self,
        tcx: &mut TextContext,
        w: u32,
        h: u32,
    ) -> Vec<shell::layout::PlacedNode> {
        self.frames += 1;
        let nodes = self.source.build_frame(tcx, w, h);
        self.rendered = self.source.actor.view().bindings.values().any(|binding| {
            binding.text.starts_with("Count: 10") && binding.text.contains("new code")
        });
        nodes
    }
    fn base_color(&self) -> vello_common::peniko::Color {
        self.source.base_color()
    }
    fn set_device_scale(&mut self, scale: f64) {
        self.source.set_device_scale(scale);
    }
    fn handle_event(&mut self, event: shell::UiEvent) -> shell::EventResponse {
        self.source.handle_event(event)
    }
    fn set_wake_callback(&mut self, wake: Arc<dyn Fn() + Send + Sync>) {
        self.wake = Some(wake.clone());
        self.source.set_wake_callback(wake);
    }
    fn animation_deadline(&self) -> Option<Instant> {
        self.source.animation_deadline()
    }
    fn push_frame_stats(&mut self, _: &shell::FrameStats) {
        if self.started.is_none() {
            self.started = Some(Instant::now());
            assert_eq!(self.animation_deadline(), None);
            assert!(!self.source.has_anim());
            let future = GateFuture {
                gate: self.gate.clone(),
                notify: self.wake.clone().unwrap(),
            };
            self.source
                .actor
                .spawn_async(
                    ActorId(1),
                    1,
                    future,
                    Message {
                        action: blockui_abi::key("task_done"),
                        value: -1,
                    },
                    blockui_runtime::AsyncReloadPolicy::Preserve,
                )
                .unwrap();
        } else if let Some(started) = self.started
            && self.rendered
            && !self.reported
        {
            assert!(self.released);
            assert_eq!(self.source.actor.root_actor().integer("count"), 10);
            assert!(self.polls < 100);
            let scoped = self
                .source
                .actor
                .root_actor()
                .scope_ids()
                .contains(&blockui_abi::key("counter.async.panel"));
            if scoped {
                let actor = self.source.actor.root_actor();
                assert!(actor.view.root_dependencies.is_empty());
                assert_eq!(
                    actor.view.bindings[&blockui_abi::key("async.root.build")].text,
                    "Async root build 1"
                );
                assert!(
                    actor.view.scope_dependencies[&blockui_abi::key("counter.async.panel")]
                        .dependencies
                        .contains(&blockui_runtime::Dependency::State(blockui_abi::key(
                            "count"
                        )))
                );
            }
            println!(
                "NATIVE_FUTURE_REPORT {}",
                serde_json::json!({"scoped_completion":scoped,"pending_seen":!self.prequeued,"queued_before_window":self.prequeued,"reloaded_while_pending":!self.prequeued,"reloaded_after_queue":self.prequeued,"state_and_worker_retained":true,"new_code_handled_completion":true,"completion_presented":true,"idle":self.animation_deadline().is_none() && !self.source.has_anim(),"elapsed_ms":started.elapsed().as_secs_f64()*1000.0,"polls":self.polls,"frames":self.frames})
            );
            self.reported = true;
            self.exit = true;
            (self.wake.as_ref().unwrap())();
        }
    }
    fn poll_async(&mut self) -> bool {
        self.polls += 1;
        let changed = self.source.poll_async();
        if !self.released {
            let mut gate = self.gate.lock().unwrap();
            if gate.pending {
                assert_eq!(self.animation_deadline(), None);
                let actor = self.source.actor.root_actor();
                let slot = actor.slot_address("count");
                let version = actor.implementation_version();
                let worker = actor.tasks[&1].worker_id();
                self.source.replace(&self.replacement).unwrap();
                let actor = self.source.actor.root_actor();
                assert_eq!(actor.slot_address("count"), slot);
                assert_eq!(actor.tasks[&1].worker_id(), worker);
                assert!(!actor.tasks[&1].is_finished());
                assert_ne!(actor.implementation_version(), version);
                assert_eq!(actor.integer("count"), 0);
                gate.ready = true;
                self.released = true;
                let waker = gate.waker.take().unwrap();
                drop(gate);
                waker.wake();
            }
        }
        changed || self.exit
    }
    fn take_effect(&mut self) -> Option<shell::EffectRequest> {
        std::mem::take(&mut self.exit).then_some(shell::EffectRequest {
            id: shell::EffectId(1),
            scope: shell::effect::EffectScope::Runtime,
            payload: shell::EffectPayload::ApplicationExit,
        })
    }
}
fn main() -> Result<(), String> {
    let library = std::env::args()
        .nth(1)
        .ok_or("native_future <base.so> <replacement.so> <artifact-directory>")?;
    let replacement: std::path::PathBuf = std::env::args()
        .nth(2)
        .ok_or("missing replacement DLL")?
        .into();
    let mut runtime = Runtime::new(std::env::args().nth(3).unwrap_or_else(|| ".runtime".into()));
    runtime.spawn(ActorId(1), Path::new(&library))?;
    let prequeued = std::env::args().nth(4).as_deref() == Some("queued");
    let queued_started = Instant::now();
    if prequeued {
        let message = Message {
            action: blockui_abi::key("increment"),
            value: 0,
        };
        runtime.spawn_async(
            ActorId(1),
            1,
            std::future::ready(message),
            message,
            blockui_runtime::AsyncReloadPolicy::Preserve,
        )?;
        let deadline = Instant::now() + std::time::Duration::from_secs(2);
        while !runtime.actor(ActorId(1)).unwrap().tasks[&1].is_finished() {
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        let actor = runtime.actor(ActorId(1)).unwrap();
        let slot = actor.slot_address("count");
        let worker = actor.tasks[&1].worker_id();
        runtime.reload(ActorId(1), &replacement)?;
        let actor = runtime.actor(ActorId(1)).unwrap();
        assert_eq!(actor.slot_address("count"), slot);
        assert_eq!(actor.tasks[&1].worker_id(), worker);
        assert_eq!(actor.integer("count"), 0);
    }
    let source = NativeSource::without_watch(NativeFamily::new(runtime, ActorId(1))?)?;
    let probe = Probe {
        source,
        replacement,
        prequeued,
        gate: Arc::new(std::sync::Mutex::new(Gate {
            ready: false,
            pending: false,
            waker: None,
        })),
        started: prequeued.then_some(queued_started),
        released: prequeued,
        rendered: false,
        reported: false,
        exit: false,
        polls: 0,
        frames: 0,
        wake: None,
    };
    shell::run_window_with_options(
        Box::new(probe),
        shell::WindowOptions::new().title("BlockUI — Future Completion"),
    )
    .map_err(|e| e.to_string())
}
