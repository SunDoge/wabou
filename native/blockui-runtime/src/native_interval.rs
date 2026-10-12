//! Isolated live-window fixture: periodic progress must come from host wakes.
use super::*;

struct Identity {
    state: usize,
    count: usize,
    notes: usize,
    editor: usize,
    node: NodeId,
    selection: Option<shell::WidgetTextSelection>,
    generation: u64,
    worker: Option<thread::ThreadId>,
}
struct WindowProof {
    source: NativeSource<Actor>,
    replacement: PathBuf,
    rejected: PathBuf,
    phase: u8,
    count: i64,
    version: u64,
    identity: Option<Identity>,
}
impl WindowProof {
    fn retained(&self) {
        let old = self.identity.as_ref().unwrap();
        let input = key("counter.notes");
        let task = &self.source.actor.tasks[&key("counter.interval")];
        assert_eq!(self.source.actor.state_address(), old.state);
        assert_eq!(self.source.actor.slot_address("count"), old.count);
        assert_eq!(self.source.actor.slot_address("notes"), old.notes);
        assert_eq!(self.source.actor.text("notes"), "a😀bc periodic draft");
        assert_eq!(
            (&**self.source.editors.get(&input).unwrap() as *const TextInput) as usize,
            old.editor
        );
        assert_eq!(self.source.ids[&input], old.node);
        assert_eq!(self.source.focus, Some(input));
        assert!(self.source.window_focused);
        assert!(self.source.ime_state().is_some());
        assert_eq!(self.source.editors[&input].text_selection(), old.selection);
        assert_eq!(
            self.source.actor.view.renderer[&key("counter.scroll")].scroll,
            137.0
        );
        assert_eq!(task.generation(), old.generation);
        assert_eq!(task.worker_id(), old.worker);
        assert_eq!(self.source.actor.artifact_count(), 1);
    }
}
impl FrameSource for WindowProof {
    fn set_device_scale(&mut self, scale: f64) {
        self.source.set_device_scale(scale);
    }
    fn prepare_for_event(&mut self, tcx: &mut TextContext) {
        self.source.prepare_for_event(tcx);
    }
    fn set_wake_callback(&mut self, wake: shell::WakeCallback) {
        self.source.set_wake_callback(wake);
    }
    fn poll_async(&mut self) -> bool {
        let before = self.source.actor.integer("count");
        let changed = self.source.poll_async();
        let delta = self.source.actor.integer("count") - before;
        if delta != 0 {
            assert_eq!(
                delta,
                if self.phase < 3 { 1 } else { 10 },
                "periodic delivery used stale code"
            );
        }
        changed
    }
    fn handle_event(&mut self, event: UiEvent) -> EventResponse {
        self.source.handle_event(event)
    }
    fn ime_state(&self) -> Option<shell::ImeState> {
        self.source.ime_state()
    }
    fn base_color(&self) -> Color {
        self.source.base_color()
    }
    // No caret deadline or continuous redraw while waiting. Winit's host wake
    // must drain timer completions, rather than accidentally advancing on animation.
    fn has_anim(&self) -> bool {
        self.source.capture
    }
    fn build_frame(&mut self, tcx: &mut TextContext, width: u32, height: u32) -> Vec<PlacedNode> {
        if self.phase == 0 {
            // Establish native widget focus through the host event route after
            // window creation. This is not an OS input-method/focus automation test.
            self.source.handle_event(UiEvent::Focus(true));
        }
        let nodes = self.source.build_frame(tcx, width, height);
        assert!(
            self.source.metrics.is_some(),
            "fixture must run in a real Winit window"
        );
        let input = key("counter.notes");
        match self.phase {
            0 => {
                self.source
                    .actor
                    .dispatch(Message {
                        action: key("start_interval"),
                        value: 0,
                    })
                    .unwrap();
                let task = &self.source.actor.tasks[&key("counter.interval")];
                let selection = self.source.editors[&input].text_selection();
                assert_eq!(selection.as_ref().map(|s| (s.anchor, s.head)), Some((1, 3)));
                self.identity = Some(Identity {
                    state: self.source.actor.state_address(),
                    count: self.source.actor.slot_address("count"),
                    notes: self.source.actor.slot_address("notes"),
                    editor: (&**self.source.editors.get(&input).unwrap() as *const TextInput)
                        as usize,
                    node: self.source.ids[&input],
                    selection,
                    generation: task.generation(),
                    worker: task.worker_id(),
                });
                assert!(task.worker_id().is_some());
                self.phase = 1;
            }
            1 if self.source.actor.integer("count") > 0 => {
                self.retained();
                assert!(self.source.replace(&self.rejected).is_err());
                self.retained();
                assert_eq!(self.source.actor.implementation_version(), self.version);
                self.count = self.source.actor.integer("count");
                self.phase = 2;
            }
            2 if self.source.actor.integer("count") > self.count => {
                self.retained();
                assert_eq!(self.source.actor.implementation_version(), self.version);
                eprintln!(
                    "PASS live periodic window keeps old callbacks running after rejected replacement without animation deadlines"
                );
                self.count = self.source.actor.integer("count");
                self.source.replace(&self.replacement).unwrap();
                self.retained();
                assert_eq!(self.source.actor.integer("count"), self.count);
                assert_ne!(self.source.actor.implementation_version(), self.version);
                self.phase = 3;
            }
            3 if self.source.actor.integer("count") > self.count => {
                self.retained();
                assert_eq!((self.source.actor.integer("count") - self.count) % 10, 0);
                assert!(
                    self.source.actor.view.bindings[&key("counter.count")]
                        .text
                        .contains("new code")
                );
                self.source
                    .actor
                    .dispatch(Message {
                        action: key("stop_interval"),
                        value: 0,
                    })
                    .unwrap();
                assert!(self.source.actor.tasks.is_empty());
                eprintln!(
                    "PASS live periodic window wakes into new code and retains state/slot/task generation/worker/editor/UTF16 selection/focus/scroll across real DLL replacement"
                );
                self.source.capture = true;
                self.phase = 4;
            }
            _ => {}
        }
        nodes
    }
    fn take_screenshot_request(&mut self) -> Option<shell::ScreenshotRequest> {
        self.source.take_screenshot_request()
    }
    fn complete_screenshot(&mut self, path: &Path, result: Result<PathBuf, String>) {
        self.source.complete_screenshot(path, result);
    }
    fn take_effect(&mut self) -> Option<shell::EffectRequest> {
        self.source.take_effect()
    }
}

pub fn proof(actor: Actor, replacement: &Path, rejected: &Path) -> Result<(), String> {
    let mut source = NativeSource::without_watch(actor)?;
    source
        .actor
        .set_text(key("notes"), "a😀bc periodic draft")?;
    source.sync()?;
    let input = key("counter.notes");
    source.set_focus(Some(input));
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":3}}"#)
        .unwrap();
    source
        .actor
        .view
        .renderer
        .get_mut(&key("counter.scroll"))
        .unwrap()
        .scroll = 137.0;
    let version = source.actor.implementation_version();
    let proof = WindowProof {
        source,
        replacement: replacement.into(),
        rejected: rejected.into(),
        phase: 0,
        count: 0,
        version,
        identity: None,
    };
    shell::run_window_with_options(
        Box::new(proof),
        shell::WindowOptions::new()
            .title("Periodic actor reload")
            .initial_inner_size(720, 620),
    )
    .map_err(|error| error.to_string())
}
