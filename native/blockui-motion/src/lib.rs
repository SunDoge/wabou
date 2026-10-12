//! Host-owned, renderer-independent scalar animation. Time is supplied by the host
//! or a test; this crate starts no threads and retains no application callbacks.
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

pub const SCROLL_DURATION: Duration = Duration::from_millis(120);
pub const HOVER_DURATION: Duration = Duration::from_millis(120);
pub const DISCLOSURE_DURATION: Duration = Duration::from_millis(120);
pub const SWITCH_DURATION: Duration = Duration::from_millis(160);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Property {
    Hover,
    Switch,
    Disclosure,
    ScrollX,
    ScrollY,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    pub node: u64,
    pub property: Property,
}
impl Key {
    pub const fn new(node: u64, property: Property) -> Self {
        Self { node, property }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Easing {
    Linear,
    #[default]
    EaseOutCubic,
}
impl Easing {
    fn sample(self, value: f64) -> f64 {
        match self {
            Self::Linear => value,
            Self::EaseOutCubic => 1.0 - (1.0 - value).powi(3),
        }
    }
}
struct Track {
    from: f64,
    target: f64,
    value: f64,
    start: Duration,
    duration: Duration,
    easing: Easing,
}
impl Track {
    fn sample(&mut self, now: Duration) -> bool {
        let elapsed = now.saturating_sub(self.start);
        if elapsed >= self.duration {
            self.value = self.target;
            return false;
        }
        let progress = self
            .easing
            .sample(elapsed.as_secs_f64() / self.duration.as_secs_f64());
        self.value = self.from * (1.0 - progress) + self.target * progress;
        true
    }
}
/// One timeline per stable renderer/mount. Only active tracks are sampled each frame.
#[derive(Default)]
pub struct Timeline {
    now: Duration,
    reduced_motion: bool,
    tracks: BTreeMap<Key, Track>,
    active: BTreeSet<Key>,
}
impl Timeline {
    pub fn now(&self) -> Duration {
        self.now
    }
    pub fn is_active(&self) -> bool {
        !self.active.is_empty()
    }
    pub fn len(&self) -> usize {
        self.tracks.len()
    }
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }
    pub fn value(&self, key: Key) -> Option<f64> {
        self.tracks.get(&key).map(|t| t.value)
    }
    /// Initial mount snaps. Retargeting starts at the current interpolated value;
    /// submitting the same target does not restart an animation.
    pub fn retarget(
        &mut self,
        key: Key,
        target: f64,
        duration: Duration,
        easing: Easing,
    ) -> Result<bool, &'static str> {
        if !target.is_finite() {
            return Err("animation target must be finite");
        }
        let Some(track) = self.tracks.get_mut(&key) else {
            self.tracks.insert(
                key,
                Track {
                    from: target,
                    target,
                    value: target,
                    start: self.now,
                    duration: Duration::ZERO,
                    easing,
                },
            );
            return Ok(true);
        };
        if track.target == target && duration != Duration::ZERO && !self.reduced_motion {
            return Ok(false);
        }
        if track.target == target && track.value == target {
            return Ok(false);
        }
        track.sample(self.now);
        track.from = track.value;
        track.target = target;
        track.start = self.now;
        track.duration = if self.reduced_motion {
            Duration::ZERO
        } else {
            duration
        };
        track.easing = easing;
        if track.duration.is_zero() || track.value == target {
            track.value = target;
            self.active.remove(&key);
        } else {
            self.active.insert(key);
        }
        Ok(true)
    }
    /// Reject backwards time without changing any track. Includes the final sample
    /// in the dirty set so renderers paint exact end values before going idle.
    pub fn advance_to(&mut self, now: Duration) -> Result<Vec<Key>, &'static str> {
        if now < self.now {
            return Err("animation clock must be monotonic");
        }
        if now == self.now {
            return Ok(Vec::new());
        }
        self.now = now;
        let dirty: Vec<_> = self.active.iter().copied().collect();
        self.active
            .retain(|key| self.tracks.get_mut(key).unwrap().sample(now));
        Ok(dirty)
    }
    pub fn advance_by(&mut self, elapsed: Duration) -> Result<Vec<Key>, &'static str> {
        self.advance_to(
            self.now
                .checked_add(elapsed)
                .ok_or("animation clock overflow")?,
        )
    }
    pub fn set_reduced_motion(&mut self, reduced: bool) -> Vec<Key> {
        self.reduced_motion = reduced;
        if !reduced {
            return Vec::new();
        }
        let dirty = std::mem::take(&mut self.active);
        for key in &dirty {
            let t = self.tracks.get_mut(key).unwrap();
            t.value = t.target;
            t.duration = Duration::ZERO;
        }
        dirty.into_iter().collect()
    }
    /// Call after reconciliation: removed widgets cannot retain dormant tracks.
    pub fn retain(&mut self, mut live: impl FnMut(Key) -> bool) {
        self.tracks.retain(|key, _| live(*key));
        self.active.retain(|key| self.tracks.contains_key(key));
    }
    pub fn remove(&mut self, key: Key) {
        self.tracks.remove(&key);
        self.active.remove(&key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const KEY: Key = Key::new(1, Property::Switch);
    #[test]
    fn retargeting_is_continuous_and_equal_targets_do_not_restart() {
        let mut t = Timeline::default();
        t.retarget(KEY, 0.0, SWITCH_DURATION, Easing::Linear)
            .unwrap();
        assert!(!t.is_active());
        t.retarget(KEY, 1.0, SWITCH_DURATION, Easing::Linear)
            .unwrap();
        t.advance_by(Duration::from_millis(80)).unwrap();
        assert_eq!(t.value(KEY), Some(0.5));
        assert!(
            !t.retarget(KEY, 1.0, SWITCH_DURATION, Easing::Linear)
                .unwrap()
        );
        t.retarget(KEY, 0.0, SWITCH_DURATION, Easing::Linear)
            .unwrap();
        assert_eq!(t.value(KEY), Some(0.5));
        t.advance_by(Duration::from_millis(80)).unwrap();
        assert_eq!(t.value(KEY), Some(0.25));
        assert_eq!(t.advance_by(Duration::from_millis(80)).unwrap(), [KEY]);
        assert_eq!(t.value(KEY), Some(0.0));
        assert!(!t.is_active());
        assert!(t.advance_by(Duration::from_secs(1)).unwrap().is_empty());
    }
    #[test]
    fn reduction_cancellation_and_unmount_bound_retained_tracks() {
        let mut t = Timeline::default();
        for node in 0..100 {
            let key = Key::new(node, Property::Hover);
            t.retarget(key, 0.0, HOVER_DURATION, Easing::default())
                .unwrap();
            t.retarget(key, 1.0, HOVER_DURATION, Easing::default())
                .unwrap();
        }
        t.advance_by(Duration::from_millis(60)).unwrap();
        assert_eq!(t.value(Key::new(0, Property::Hover)), Some(0.875));
        assert_eq!(t.set_reduced_motion(true).len(), 100);
        assert!(!t.is_active());
        t.retain(|key| key.node == 0);
        assert_eq!(t.len(), 1);
        let key = Key::new(0, Property::Hover);
        t.retarget(key, 0.0, HOVER_DURATION, Easing::default())
            .unwrap();
        assert_eq!(t.value(key), Some(0.0));
        t.remove(key);
        assert!(t.is_empty());
    }
    #[test]
    fn invalid_input_is_transactional_and_delayed_frames_finish_exactly() {
        let mut t = Timeline::default();
        t.retarget(KEY, 0.0, SWITCH_DURATION, Easing::default())
            .unwrap();
        assert!(
            t.retarget(KEY, f64::NAN, SWITCH_DURATION, Easing::default())
                .is_err()
        );
        assert_eq!(t.value(KEY), Some(0.0));
        t.retarget(KEY, 1.0, SWITCH_DURATION, Easing::default())
            .unwrap();
        t.advance_by(Duration::from_secs(10)).unwrap();
        assert_eq!(t.value(KEY), Some(1.0));
        assert!(t.advance_to(Duration::ZERO).is_err());
        assert_eq!(t.now(), Duration::from_secs(10));
        t.retarget(KEY, 0.0, Duration::ZERO, Easing::default())
            .unwrap();
        assert_eq!(t.value(KEY), Some(0.0));
        assert!(!t.is_active());
    }
}
