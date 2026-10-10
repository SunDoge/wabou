//! Owned-display verification of an idle native request deadline.
use blockui_runtime::{
    ActorId, Message, Runtime,
    native::NativeSource,
    native_model::NativeFamily,
    request::{Reply, RequestError},
};
use std::{
    future::Future,
    path::Path,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};
use wabou_shell_vello::{self as shell, FrameSource, TextContext};

struct Probe {
    source: NativeSource<NativeFamily>,
    reply: Option<Reply>,
    started: Option<Instant>,
    deadline: Option<Instant>,
    pending_seen: bool,
    exit: bool,
    polls: usize,
    frames: usize,
    wake: Option<Waker>,
}
impl FrameSource for Probe {
    fn build_frame(
        &mut self,
        tcx: &mut TextContext,
        w: u32,
        h: u32,
    ) -> Vec<shell::layout::PlacedNode> {
        self.frames += 1;
        self.source.build_frame(tcx, w, h)
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
        struct ProxyWake(Arc<dyn Fn() + Send + Sync>);
        impl std::task::Wake for ProxyWake {
            fn wake(self: Arc<Self>) {
                (self.0)();
            }
        }
        self.wake = Some(Waker::from(Arc::new(ProxyWake(wake.clone()))));
        self.source.set_wake_callback(wake);
    }
    fn animation_deadline(&self) -> Option<Instant> {
        self.source.animation_deadline()
    }
    fn push_frame_stats(&mut self, _: &shell::FrameStats) {
        if self.started.is_none() {
            // Start only after the first real surface frame has completed.
            self.started = Some(Instant::now());
            // Increment handles the request but intentionally never replies.
            self.reply = Some(
                self.source
                    .actor
                    .ask(
                        ActorId(1),
                        Message {
                            action: blockui_abi::key("increment"),
                            value: 0,
                        },
                        Duration::from_millis(300),
                    )
                    .unwrap(),
            );
            self.deadline = self.animation_deadline();
            assert!(self.deadline.is_some());
        }
    }
    fn poll_async(&mut self) -> bool {
        self.polls += 1;
        let changed = self.source.poll_async();
        if let Some(reply) = self.reply.as_mut() {
            match Pin::new(reply).poll(&mut Context::from_waker(self.wake.as_ref().unwrap())) {
                Poll::Pending => self.pending_seen = true,
                Poll::Ready(result) => {
                    assert_eq!(result, Err(RequestError::Timeout));
                    assert!(
                        self.pending_seen,
                        "request must enter pending state before expiry"
                    );
                    assert!(Instant::now() >= self.deadline.unwrap());
                    assert_eq!(self.animation_deadline(), None);
                    assert!(!self.source.has_anim());
                    assert!(
                        self.polls < 100,
                        "idle deadline must not require busy polling"
                    );
                    println!(
                        "NATIVE_REQUEST_REPORT {}",
                        serde_json::json!({"timeout":true,"pending_before_timeout":true,"idle_after_timeout":true,"elapsed_ms":self.started.unwrap().elapsed().as_secs_f64()*1000.0,"polls":self.polls,"frames":self.frames})
                    );
                    self.reply = None;
                    self.exit = true;
                }
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
        .ok_or("native_request <counter.so>")?;
    let mut runtime = Runtime::new(std::env::args().nth(2).unwrap_or_else(|| ".runtime".into()));
    runtime.spawn(ActorId(1), Path::new(&library))?;
    let source = NativeSource::without_watch(NativeFamily::new(runtime, ActorId(1))?)?;
    let probe = Probe {
        source,
        reply: None,
        started: None,
        deadline: None,
        pending_seen: false,
        exit: false,
        polls: 0,
        frames: 0,
        wake: None,
    };
    shell::run_window_with_options(
        Box::new(probe),
        shell::WindowOptions::new().title("BlockUI — Request Deadline"),
    )
    .map_err(|e| e.to_string())
}
