use super::*;
use vello_shell::motion::{MotionSample, MotionSystem};

#[derive(Default)]
pub(super) struct NativeMotionState {
    timeline: MotionSystem<NodeId>,
    last_tick: Option<Instant>,
}

impl NativeMotionState {
    pub(super) fn set_json(&mut self, node: NodeId, json: &str) -> Result<(), String> {
        self.timeline.set_json(node, json)?;
        self.restart_clock();
        Ok(())
    }

    pub(super) fn remove_node(&mut self, node: NodeId) {
        self.timeline.remove(node);
    }

    pub(super) fn advance(&mut self, now: Instant) -> Vec<(NodeId, u64)> {
        let elapsed = self
            .last_tick
            .replace(now)
            .map_or(Duration::ZERO, |previous| {
                now.saturating_duration_since(previous)
            });
        self.timeline
            .advance(elapsed)
            .into_iter()
            .map(|completion| (completion.node, completion.generation))
            .collect()
    }

    pub(super) fn sample(&self, node: NodeId) -> MotionSample {
        self.timeline.sample(node)
    }

    pub(super) fn samples(&self) -> Vec<(NodeId, MotionSample)> {
        self.timeline.samples()
    }

    pub(super) fn is_active(&self) -> bool {
        self.timeline.is_active()
    }

    fn restart_clock(&mut self) {
        self.last_tick = Some(Instant::now());
    }
}

impl Applier {
    pub(super) fn tick_native_motions(&mut self) {
        let completed = self.document.motion.advance(Instant::now());
        let samples = self.document.motion.samples();
        let transforms_active = samples.iter().any(|(_, sample)| sample.transform.is_some());
        for (node, sample) in samples {
            if let Some(paint) = self.document.node_store.tree.get_node_context_mut(node) {
                paint.motion_transform = sample.transform;
                paint.motion_opacity = sample.opacity;
            }
        }
        if transforms_active && self.document.motion.is_active() {
            self.document
                .invalidation
                .insert(InvalidationFlags::GEOMETRY);
        }
        for (node, generation) in completed {
            let Some(target) = self.document.node_store.solid_id_for_node(node) else {
                continue;
            };
            let payload = serde_json::json!({ "generation": generation }).to_string();
            self.dispatch_json(target, event::TRANSITIONEND, &payload);
        }
    }

    pub(super) fn clear_native_motion(&mut self, node: NodeId) {
        self.document.motion.remove_node(node);
        if let Some(paint) = self.document.node_store.tree.get_node_context_mut(node) {
            paint.motion_transform = None;
            paint.motion_opacity = None;
        }
        self.document
            .invalidation
            .insert(InvalidationFlags::GEOMETRY);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node() -> NodeId {
        TaffyTree::<()>::new()
            .new_leaf(taffy::Style::default())
            .unwrap()
    }

    #[test]
    fn native_clock_drives_shared_transition_and_completion() {
        let mut state = NativeMotionState::default();
        let node = node();
        state
            .set_json(
                node,
                r#"{
                    "kind": "transition",
                    "generation": 7,
                    "duration": 0.1,
                    "easing": "linear",
                    "fromTransform": [1, 0, 0, 1, 0, 0],
                    "toTransform": [1, 0, 0, 1, 10, 20],
                    "fromOpacity": 0,
                    "toOpacity": 1
                }"#,
            )
            .unwrap();
        let start = Instant::now();
        state.last_tick = Some(start);
        assert!(state.advance(start + Duration::from_millis(50)).is_empty());
        assert_eq!(
            state.sample(node).transform,
            Some([1.0, 0.0, 0.0, 1.0, 5.0, 10.0])
        );
        assert!((state.sample(node).opacity.unwrap() - 0.5).abs() < 1e-6);
        assert_eq!(
            state.advance(start + Duration::from_millis(100)),
            [(node, 7)]
        );
        assert_eq!(state.advance(start + Duration::from_millis(200)), []);
    }
}
