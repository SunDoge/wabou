//! Renderer-owned animation primitives.
//!
//! Motions are sampled from elapsed time supplied by the host. They start no
//! threads, retain no JavaScript callbacks, and can therefore run while the
//! application runtime is otherwise idle.

use std::{collections::HashMap, hash::Hash, time::Duration};

use serde::Deserialize;
use slotmap::{SlotMap, new_key_type};

/// Opaque identifier for a runtime-owned scalar animation curve.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ValueMotionId(u32);

impl ValueMotionId {
    /// Recreate an identifier received across the host ABI.
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Numeric representation used by the private host ABI.
    pub const fn into_raw(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
enum ValueMotionDescriptor {
    Tween {
        duration: f64,
        #[serde(default)]
        easing: ValueEasing,
    },
    Spring {
        #[serde(default = "default_value_stiffness")]
        stiffness: f64,
        #[serde(default = "default_value_damping")]
        damping: f64,
        #[serde(default = "default_value_mass")]
        mass: f64,
        #[serde(default)]
        velocity: f64,
        #[serde(default = "default_epsilon")]
        rest_delta: f64,
        #[serde(default = "default_rest_speed")]
        rest_speed: f64,
    },
}

fn default_value_stiffness() -> f64 {
    170.0
}
fn default_value_damping() -> f64 {
    26.0
}
fn default_value_mass() -> f64 {
    1.0
}
fn default_rest_speed() -> f64 {
    0.01
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ValueEasing {
    Linear,
    EaseIn,
    #[default]
    EaseOut,
    EaseInOut,
    CircIn,
    CircOut,
    CircInOut,
    BackIn,
    BackOut,
    BackInOut,
    Anticipate,
    CubicBezier([f64; 4]),
}

impl ValueEasing {
    fn sample(self, value: f64) -> f64 {
        let value = value.clamp(0.0, 1.0);
        match self {
            Self::Linear => value,
            Self::EaseIn => value.powi(3),
            Self::EaseOut => 1.0 - (1.0 - value).powi(3),
            Self::EaseInOut if value < 0.5 => 4.0 * value.powi(3),
            Self::EaseInOut => 1.0 - (-2.0 * value + 2.0).powi(3) / 2.0,
            Self::CircIn => 1.0 - (1.0 - value * value).sqrt(),
            Self::CircOut => (1.0 - (value - 1.0).powi(2)).sqrt(),
            Self::CircInOut if value < 0.5 => (1.0 - (1.0 - (2.0 * value).powi(2)).sqrt()) / 2.0,
            Self::CircInOut => ((1.0 - (-2.0 * value + 2.0).powi(2)).sqrt() + 1.0) / 2.0,
            Self::BackIn => back_in(value),
            Self::BackOut => 1.0 - back_in(1.0 - value),
            Self::BackInOut if value < 0.5 => back_in(value * 2.0) / 2.0,
            Self::BackInOut => (2.0 - back_in((1.0 - value) * 2.0)) / 2.0,
            Self::Anticipate if value < 0.5 => back_in(value * 2.0) / 2.0,
            Self::Anticipate => (1.0 + (1.0 - (1.0 - value) * 2.0).powi(2)) / 2.0,
            Self::CubicBezier(points) => cubic_bezier(value, points),
        }
    }
}

fn back_in(value: f64) -> f64 {
    const OVERSHOOT: f64 = 1.70158;
    (OVERSHOOT + 1.0) * value.powi(3) - OVERSHOOT * value.powi(2)
}

fn cubic_bezier(value: f64, [x1, y1, x2, y2]: [f64; 4]) -> f64 {
    fn axis(t: f64, a: f64, b: f64) -> f64 {
        let inverse = 1.0 - t;
        3.0 * inverse * inverse * t * a + 3.0 * inverse * t * t * b + t.powi(3)
    }
    let mut low = 0.0;
    let mut high = 1.0;
    for _ in 0..18 {
        let middle = (low + high) / 2.0;
        if axis(middle, x1, x2) < value {
            low = middle;
        } else {
            high = middle;
        }
    }
    axis((low + high) / 2.0, y1, y2)
}

/// One normalized scalar sample produced by the Rust animation engine.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ValueMotionSample {
    /// Normalized value. Springs may temporarily overshoot `0..=1`.
    pub value: f64,
    /// Instantaneous normalized velocity per second.
    pub velocity: f64,
    /// Whether the finite curve has reached its exact endpoint.
    pub done: bool,
}

/// Runtime-owned registry for scalar curves used by JavaScript motion values.
#[derive(Default)]
pub struct ValueMotionStore {
    next: u32,
    motions: HashMap<ValueMotionId, ValueMotionDescriptor>,
}

impl ValueMotionStore {
    /// Parse and retain a curve descriptor.
    pub fn create_json(&mut self, json: &str) -> Result<ValueMotionId, String> {
        let descriptor: ValueMotionDescriptor =
            serde_json::from_str(json).map_err(|error| error.to_string())?;
        validate_value_motion(descriptor)?;
        self.next = self.next.wrapping_add(1).max(1);
        let id = ValueMotionId(self.next);
        self.motions.insert(id, descriptor);
        Ok(id)
    }

    /// Sample a retained curve at elapsed seconds.
    pub fn sample(&self, id: ValueMotionId, elapsed: f64) -> Option<ValueMotionSample> {
        let descriptor = *self.motions.get(&id)?;
        let elapsed = elapsed.max(0.0);
        Some(match descriptor {
            ValueMotionDescriptor::Tween { duration, easing } => {
                let progress = if duration == 0.0 {
                    1.0
                } else {
                    (elapsed / duration).clamp(0.0, 1.0)
                };
                ValueMotionSample {
                    value: easing.sample(progress),
                    velocity: 0.0,
                    done: progress >= 1.0,
                }
            }
            ValueMotionDescriptor::Spring {
                stiffness,
                damping,
                mass,
                velocity,
                rest_delta,
                rest_speed,
            } => {
                let options = SpringOptions {
                    mass,
                    stiffness,
                    damping,
                    epsilon: rest_delta,
                };
                let (displacement, velocity) = solve_spring(-1.0, velocity, elapsed, options);
                let done = displacement.abs() <= rest_delta && velocity.abs() <= rest_speed;
                ValueMotionSample {
                    value: if done { 1.0 } else { 1.0 + displacement },
                    velocity: if done { 0.0 } else { velocity },
                    done,
                }
            }
        })
    }

    /// Remove a retained curve and invalidate its handle.
    pub fn remove(&mut self, id: ValueMotionId) -> bool {
        self.motions.remove(&id).is_some()
    }
}

fn validate_value_motion(descriptor: ValueMotionDescriptor) -> Result<(), String> {
    match descriptor {
        ValueMotionDescriptor::Tween { duration, .. }
            if !duration.is_finite() || duration < 0.0 =>
        {
            Err("motion duration must be finite and non-negative".into())
        }
        ValueMotionDescriptor::Spring {
            stiffness,
            damping,
            mass,
            velocity,
            rest_delta,
            rest_speed,
        } if !stiffness.is_finite()
            || stiffness <= 0.0
            || !damping.is_finite()
            || damping < 0.0
            || !mass.is_finite()
            || mass <= 0.0
            || !velocity.is_finite()
            || !rest_delta.is_finite()
            || rest_delta <= 0.0
            || !rest_speed.is_finite()
            || rest_speed <= 0.0 =>
        {
            Err("invalid spring parameters".into())
        }
        _ => Ok(()),
    }
}

new_key_type! {
    /// Generational identity for one renderer-owned motion.
    pub struct MotionId;
}

/// Timing curve for a finite tween.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Easing {
    /// Constant velocity.
    Linear,
    /// Symmetric cubic acceleration and deceleration.
    EaseInOut,
    /// Cubic deceleration.
    #[default]
    EaseOut,
}

impl Easing {
    fn sample(self, progress: f64) -> f64 {
        let progress = progress.clamp(0.0, 1.0);
        match self {
            Self::Linear => progress,
            Self::EaseInOut if progress < 0.5 => 4.0 * progress.powi(3),
            Self::EaseInOut => 1.0 - (-2.0 * progress + 2.0).powi(3) / 2.0,
            Self::EaseOut => 1.0 - (1.0 - progress).powi(3),
        }
    }
}

/// Physical parameters for an analytic damped spring.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringOptions {
    /// Moving mass. Must be positive and finite.
    pub mass: f64,
    /// Restoring force coefficient. Must be positive and finite.
    pub stiffness: f64,
    /// Damping force coefficient. Must be non-negative and finite.
    pub damping: f64,
    /// Maximum displacement from the target before settling.
    pub epsilon: f64,
}

impl Default for SpringOptions {
    fn default() -> Self {
        Self {
            mass: 1.0,
            stiffness: 170.0,
            damping: 26.0,
            epsilon: 0.001,
        }
    }
}

impl SpringOptions {
    fn sanitized(self) -> Self {
        let defaults = Self::default();
        Self {
            mass: positive_or(self.mass, defaults.mass),
            stiffness: positive_or(self.stiffness, defaults.stiffness),
            damping: non_negative_or(self.damping, defaults.damping),
            epsilon: positive_or(self.epsilon, defaults.epsilon),
        }
    }
}

fn positive_or(value: f64, fallback: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

fn non_negative_or(value: f64, fallback: f64) -> f64 {
    if value.is_finite() && value >= 0.0 {
        value
    } else {
        fallback
    }
}

#[derive(Clone, Debug)]
struct Tween<const N: usize> {
    from: [f64; N],
    target: [f64; N],
    value: [f64; N],
    elapsed: Duration,
    duration: Duration,
    easing: Easing,
}

impl<const N: usize> Tween<N> {
    fn new(from: [f64; N], target: [f64; N], duration: Duration, easing: Easing) -> Self {
        let value = if duration.is_zero() { target } else { from };
        Self {
            from,
            target,
            value,
            elapsed: Duration::ZERO,
            duration,
            easing,
        }
    }

    fn advance(&mut self, elapsed: Duration) -> bool {
        self.elapsed = self.elapsed.saturating_add(elapsed);
        if self.duration.is_zero() || self.elapsed >= self.duration {
            self.value = self.target;
            return false;
        }
        let progress = self
            .easing
            .sample(self.elapsed.as_secs_f64() / self.duration.as_secs_f64());
        for channel in 0..N {
            self.value[channel] =
                self.from[channel] + (self.target[channel] - self.from[channel]) * progress;
        }
        true
    }

    fn retarget(&mut self, target: [f64; N], duration: Duration, easing: Easing) {
        self.from = self.value;
        self.target = target;
        self.elapsed = Duration::ZERO;
        self.duration = duration;
        self.easing = easing;
        if duration.is_zero() {
            self.value = target;
        }
    }
}

#[derive(Clone, Debug)]
struct Spring<const N: usize> {
    value: [f64; N],
    target: [f64; N],
    velocity: [f64; N],
    options: SpringOptions,
}

impl<const N: usize> Spring<N> {
    fn new(value: [f64; N], target: [f64; N], options: SpringOptions) -> Self {
        Self {
            value,
            target,
            velocity: [0.0; N],
            options: options.sanitized(),
        }
    }

    fn advance(&mut self, elapsed: Duration) -> bool {
        let seconds = elapsed.as_secs_f64();
        if seconds <= 0.0 || !seconds.is_finite() {
            return !self.is_settled();
        }
        for channel in 0..N {
            let displacement = self.value[channel] - self.target[channel];
            let (displacement, velocity) =
                solve_spring(displacement, self.velocity[channel], seconds, self.options);
            self.value[channel] = self.target[channel] + displacement;
            self.velocity[channel] = velocity;
        }
        if self.is_settled() {
            self.value = self.target;
            self.velocity = [0.0; N];
            false
        } else {
            true
        }
    }

    fn is_settled(&self) -> bool {
        let frequency = (self.options.stiffness / self.options.mass).sqrt();
        let velocity_epsilon = self.options.epsilon * frequency;
        (0..N).all(|channel| {
            (self.value[channel] - self.target[channel]).abs() <= self.options.epsilon
                && self.velocity[channel].abs() <= velocity_epsilon
        })
    }
}

/// Solve one damped oscillator channel exactly over `seconds`.
fn solve_spring(
    displacement: f64,
    velocity: f64,
    seconds: f64,
    options: SpringOptions,
) -> (f64, f64) {
    let frequency = (options.stiffness / options.mass).sqrt();
    let damping_ratio = options.damping / (2.0 * (options.stiffness * options.mass).sqrt());
    const CRITICAL_TOLERANCE: f64 = 1e-7;

    if damping_ratio < 1.0 - CRITICAL_TOLERANCE {
        let decay = damping_ratio * frequency;
        let damped = frequency * (1.0 - damping_ratio * damping_ratio).sqrt();
        let a = displacement;
        let b = (velocity + decay * displacement) / damped;
        let sin = (damped * seconds).sin();
        let cos = (damped * seconds).cos();
        let envelope = (-decay * seconds).exp();
        let position = envelope * (a * cos + b * sin);
        let velocity =
            envelope * ((-a * damped * sin + b * damped * cos) - decay * (a * cos + b * sin));
        (position, velocity)
    } else if damping_ratio <= 1.0 + CRITICAL_TOLERANCE {
        let b = velocity + frequency * displacement;
        let envelope = (-frequency * seconds).exp();
        (
            envelope * (displacement + b * seconds),
            envelope * (velocity - frequency * b * seconds),
        )
    } else {
        let root = (damping_ratio * damping_ratio - 1.0).sqrt();
        let slow = -frequency * (damping_ratio - root);
        let fast = -frequency * (damping_ratio + root);
        let slow_weight = (velocity - fast * displacement) / (slow - fast);
        let fast_weight = displacement - slow_weight;
        let slow_term = slow_weight * (slow * seconds).exp();
        let fast_term = fast_weight * (fast * seconds).exp();
        (slow_term + fast_term, slow * slow_term + fast * fast_term)
    }
}

#[derive(Clone, Debug)]
enum Motion<const N: usize> {
    Tween(Tween<N>),
    Spring(Spring<N>),
}

impl<const N: usize> Motion<N> {
    fn value(&self) -> [f64; N] {
        match self {
            Self::Tween(motion) => motion.value,
            Self::Spring(motion) => motion.value,
        }
    }

    fn advance(&mut self, elapsed: Duration) -> bool {
        match self {
            Self::Tween(motion) => motion.advance(elapsed),
            Self::Spring(motion) => motion.advance(elapsed),
        }
    }
}

/// Compact generational storage which samples only active motions.
pub struct MotionArena<const N: usize> {
    motions: SlotMap<MotionId, Motion<N>>,
    active: Vec<MotionId>,
}

impl<const N: usize> Default for MotionArena<N> {
    fn default() -> Self {
        Self {
            motions: SlotMap::with_key(),
            active: Vec::new(),
        }
    }
}

impl<const N: usize> MotionArena<N> {
    /// Insert a finite tween.
    pub fn insert_tween(
        &mut self,
        from: [f64; N],
        target: [f64; N],
        duration: Duration,
        easing: Easing,
    ) -> MotionId {
        let motion = Tween::new(from, target, duration, easing);
        let active = !duration.is_zero() && from != target;
        let id = self.motions.insert(Motion::Tween(motion));
        if active {
            self.active.push(id);
        }
        id
    }

    /// Insert an analytic spring.
    pub fn insert_spring(
        &mut self,
        value: [f64; N],
        target: [f64; N],
        options: SpringOptions,
    ) -> MotionId {
        let motion = Spring::new(value, target, options);
        let active = value != target;
        let id = self.motions.insert(Motion::Spring(motion));
        if active {
            self.active.push(id);
        }
        id
    }

    /// Current sampled value, or `None` for a stale handle.
    pub fn value(&self, id: MotionId) -> Option<[f64; N]> {
        self.motions.get(id).map(Motion::value)
    }

    /// Retarget a tween from its current sampled value.
    pub fn retarget_tween(
        &mut self,
        id: MotionId,
        target: [f64; N],
        duration: Duration,
        easing: Easing,
    ) -> bool {
        let active = {
            let Some(Motion::Tween(motion)) = self.motions.get_mut(id) else {
                return false;
            };
            motion.retarget(target, duration, easing);
            !duration.is_zero() && motion.value != target
        };
        self.set_active(id, active);
        true
    }

    /// Retarget a spring while preserving its current velocity.
    pub fn retarget_spring(
        &mut self,
        id: MotionId,
        target: [f64; N],
        options: SpringOptions,
    ) -> bool {
        let active = {
            let Some(Motion::Spring(motion)) = self.motions.get_mut(id) else {
                return false;
            };
            motion.target = target;
            motion.options = options.sanitized();
            !motion.is_settled()
        };
        self.set_active(id, active);
        true
    }

    fn set_active(&mut self, id: MotionId, active: bool) {
        let present = self.active.contains(&id);
        if active && !present {
            self.active.push(id);
        } else if !active && present {
            self.active.retain(|candidate| *candidate != id);
        }
    }

    /// Advance active motions and return every value which must be repainted.
    /// The final exact target sample is included once.
    pub fn advance(&mut self, elapsed: Duration) -> Vec<MotionId> {
        if elapsed.is_zero() {
            return Vec::new();
        }
        let dirty = self.active.clone();
        self.active.retain(|id| {
            self.motions
                .get_mut(*id)
                .is_some_and(|motion| motion.advance(elapsed))
        });
        dirty
    }

    /// Remove a motion and invalidate its handle generation.
    pub fn remove(&mut self, id: MotionId) -> bool {
        self.active.retain(|candidate| *candidate != id);
        self.motions.remove(id).is_some()
    }

    /// Whether another animation frame is required.
    pub fn is_active(&self) -> bool {
        !self.active.is_empty()
    }

    /// Whether one live motion still requires frame sampling.
    pub fn is_running(&self, id: MotionId) -> bool {
        self.active.contains(&id)
    }

    /// Number of retained motions, including settled values.
    pub fn len(&self) -> usize {
        self.motions.len()
    }

    /// Whether no motions are retained.
    pub fn is_empty(&self) -> bool {
        self.motions.is_empty()
    }
}

const IDENTITY_TRANSFORM: [f64; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransitionDescriptor {
    generation: u64,
    duration: f64,
    #[serde(default)]
    easing: TransitionEasing,
    from_transform: Option<[f64; 6]>,
    to_transform: Option<[f64; 6]>,
    from_opacity: Option<f64>,
    to_opacity: Option<f64>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
enum TransitionEasing {
    Linear,
    EaseInOut,
    #[default]
    EaseOut,
}

impl From<TransitionEasing> for Easing {
    fn from(value: TransitionEasing) -> Self {
        match value {
            TransitionEasing::Linear => Self::Linear,
            TransitionEasing::EaseInOut => Self::EaseInOut,
            TransitionEasing::EaseOut => Self::EaseOut,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpringDescriptor {
    #[serde(default = "default_response")]
    response: f64,
    #[serde(default = "default_damping_ratio")]
    damping: f64,
    #[serde(default = "default_epsilon")]
    epsilon: f64,
    #[serde(default)]
    mass: Option<f64>,
    #[serde(default)]
    stiffness: Option<f64>,
    target_transform: [f64; 6],
    #[serde(default)]
    target_opacity: Option<f64>,
    #[serde(default)]
    initial_transform: Option<[f64; 6]>,
    #[serde(default)]
    initial_opacity: Option<f64>,
    #[serde(default)]
    reduced_motion: bool,
}

fn default_response() -> f64 {
    0.5
}

fn default_damping_ratio() -> f64 {
    0.82
}

fn default_epsilon() -> f64 {
    0.001
}

impl SpringDescriptor {
    fn options(self) -> SpringOptions {
        let mass = self.mass.unwrap_or(1.0);
        let response = if self.response.is_finite() && self.response > 0.0 {
            self.response
        } else {
            default_response()
        };
        let frequency = std::f64::consts::TAU / response;
        let stiffness = self.stiffness.unwrap_or(mass * frequency * frequency);
        let damping = if self.damping.is_finite() && self.damping >= 0.0 {
            2.0 * self.damping * (stiffness * mass).sqrt()
        } else {
            2.0 * default_damping_ratio() * (stiffness * mass).sqrt()
        };
        SpringOptions {
            mass,
            stiffness,
            damping,
            epsilon: self.epsilon,
        }
    }
}

#[derive(Default)]
struct NodeMotion {
    transform: Option<MotionId>,
    opacity: Option<MotionId>,
    transition_generation: Option<u64>,
    completion_pending: bool,
    spring: bool,
}

/// Renderer values produced for one retained node.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MotionSample {
    /// Affine transform sampled after authored and runtime transforms.
    pub transform: Option<[f32; 6]>,
    /// Opacity multiplier sampled after authored opacity.
    pub opacity: Option<f32>,
}

/// A finite motion which completed during the latest advance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MotionCompletion<K> {
    /// Stable renderer node identity.
    pub node: K,
    /// Generation supplied by the transition author.
    pub generation: u64,
}

/// Built-in renderer motion algorithms accepted by [`MotionSystem`].
///
/// Adding a new algorithm extends this dispatch point while retaining the
/// same node ownership, sampling and completion lifecycle.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MotionKind {
    /// Finite duration and easing curve.
    Transition,
    /// Analytic damped spring which can preserve velocity while retargeting.
    Spring,
}

/// Host-independent retained motion state keyed by renderer node identity.
///
/// Hosts supply elapsed time and consume samples. Parsing, retargeting and
/// completion semantics live here so native and browser renderers can share
/// future motion kinds without duplicating their lifecycle code.
pub struct MotionSystem<K> {
    transforms: MotionArena<6>,
    opacities: MotionArena<1>,
    nodes: HashMap<K, NodeMotion>,
    samples: HashMap<K, MotionSample>,
}

impl<K> Default for MotionSystem<K> {
    fn default() -> Self {
        Self {
            transforms: MotionArena::default(),
            opacities: MotionArena::default(),
            nodes: HashMap::new(),
            samples: HashMap::new(),
        }
    }
}

impl<K> MotionSystem<K>
where
    K: Copy + Eq + Hash,
{
    /// Parse and install a tagged motion descriptor from the shared protocol.
    pub fn set_json(&mut self, node: K, json: &str) -> Result<(), String> {
        #[derive(Deserialize)]
        struct TaggedMotion {
            kind: MotionKind,
        }

        let tagged: TaggedMotion = serde_json::from_str(json).map_err(|error| error.to_string())?;
        self.set(node, tagged.kind, json)
    }

    /// Install or retarget one built-in motion kind.
    pub fn set(&mut self, node: K, kind: MotionKind, json: &str) -> Result<(), String> {
        match kind {
            MotionKind::Transition => self.set_transition(node, json),
            MotionKind::Spring => self.set_spring(node, json),
        }
    }

    /// Install or replace a finite transition described by the JS contract.
    pub fn set_transition(&mut self, node: K, json: &str) -> Result<(), String> {
        let descriptor: TransitionDescriptor =
            serde_json::from_str(json).map_err(|error| error.to_string())?;
        if !descriptor.duration.is_finite() || descriptor.duration < 0.0 {
            return Err("transition duration must be a finite non-negative number".into());
        }
        validate_optional_matrix(descriptor.from_transform)?;
        validate_optional_matrix(descriptor.to_transform)?;
        validate_optional_scalar(descriptor.from_opacity, "fromOpacity")?;
        validate_optional_scalar(descriptor.to_opacity, "toOpacity")?;

        let previous = self.sample(node);
        self.remove(node);
        let duration = Duration::from_secs_f64(descriptor.duration);
        let easing = descriptor.easing.into();
        let mut motion = NodeMotion {
            transition_generation: Some(descriptor.generation),
            completion_pending: true,
            ..NodeMotion::default()
        };
        if descriptor.from_transform.is_some() || descriptor.to_transform.is_some() {
            let from = descriptor
                .from_transform
                .or(previous.transform.map(|value| value.map(f64::from)))
                .unwrap_or(IDENTITY_TRANSFORM);
            let target = descriptor.to_transform.unwrap_or(IDENTITY_TRANSFORM);
            motion.transform = Some(self.transforms.insert_tween(from, target, duration, easing));
        }
        if descriptor.from_opacity.is_some() || descriptor.to_opacity.is_some() {
            let from = descriptor
                .from_opacity
                .or(previous.opacity.map(f64::from))
                .unwrap_or(1.0);
            let target = descriptor.to_opacity.unwrap_or(1.0);
            motion.opacity = Some(
                self.opacities
                    .insert_tween([from], [target], duration, easing),
            );
        }
        self.nodes.insert(node, motion);
        self.sample_node(node);
        Ok(())
    }

    /// Install or retarget an analytic spring described by the JS contract.
    pub fn set_spring(&mut self, node: K, json: &str) -> Result<(), String> {
        let descriptor: SpringDescriptor =
            serde_json::from_str(json).map_err(|error| error.to_string())?;
        validate_matrix(descriptor.target_transform)?;
        validate_optional_matrix(descriptor.initial_transform)?;
        validate_optional_scalar(descriptor.target_opacity, "targetOpacity")?;
        validate_optional_scalar(descriptor.initial_opacity, "initialOpacity")?;
        let options = descriptor.options();
        let previous = self.sample(node);

        if descriptor.reduced_motion {
            self.remove(node);
            let transform = self.transforms.insert_spring(
                descriptor.target_transform,
                descriptor.target_transform,
                options,
            );
            let opacity = descriptor
                .target_opacity
                .map(|target| self.opacities.insert_spring([target], [target], options));
            self.nodes.insert(
                node,
                NodeMotion {
                    transform: Some(transform),
                    opacity,
                    spring: true,
                    ..NodeMotion::default()
                },
            );
            self.sample_node(node);
            return Ok(());
        }

        if let Some(current) = self.nodes.get_mut(&node)
            && current.spring
        {
            if let Some(id) = current.transform {
                self.transforms
                    .retarget_spring(id, descriptor.target_transform, options);
            }
            match (current.opacity, descriptor.target_opacity) {
                (Some(id), Some(target)) => {
                    self.opacities.retarget_spring(id, [target], options);
                }
                (None, Some(target)) => {
                    let from = previous.opacity.map(f64::from).unwrap_or(1.0);
                    current.opacity = Some(self.opacities.insert_spring([from], [target], options));
                }
                _ => {}
            }
            self.sample_node(node);
            return Ok(());
        }

        self.remove(node);
        let from = descriptor
            .initial_transform
            .or(previous.transform.map(|value| value.map(f64::from)))
            .unwrap_or(IDENTITY_TRANSFORM);
        let transform = self
            .transforms
            .insert_spring(from, descriptor.target_transform, options);
        let opacity = descriptor.target_opacity.map(|target| {
            let from = descriptor
                .initial_opacity
                .or(previous.opacity.map(f64::from))
                .unwrap_or(1.0);
            self.opacities.insert_spring([from], [target], options)
        });
        self.nodes.insert(
            node,
            NodeMotion {
                transform: Some(transform),
                opacity,
                spring: true,
                ..NodeMotion::default()
            },
        );
        self.sample_node(node);
        Ok(())
    }

    /// Remove every retained motion and sample for one node.
    pub fn remove(&mut self, node: K) {
        if let Some(motion) = self.nodes.remove(&node) {
            if let Some(id) = motion.transform {
                self.transforms.remove(id);
            }
            if let Some(id) = motion.opacity {
                self.opacities.remove(id);
            }
        }
        self.samples.remove(&node);
    }

    /// Advance all active tracks and report newly completed transitions.
    pub fn advance(&mut self, elapsed: Duration) -> Vec<MotionCompletion<K>> {
        self.transforms.advance(elapsed);
        self.opacities.advance(elapsed);
        let nodes = self.nodes.keys().copied().collect::<Vec<_>>();
        let mut completed = Vec::new();
        for node in nodes {
            self.sample_node(node);
            let Some(motion) = self.nodes.get_mut(&node) else {
                continue;
            };
            let running = motion
                .transform
                .is_some_and(|id| self.transforms.is_running(id))
                || motion
                    .opacity
                    .is_some_and(|id| self.opacities.is_running(id));
            if !running
                && motion.completion_pending
                && let Some(generation) = motion.transition_generation
            {
                motion.completion_pending = false;
                completed.push(MotionCompletion { node, generation });
            }
        }
        completed
    }

    /// Current renderer values for a node, or an empty sample when unanimated.
    pub fn sample(&self, node: K) -> MotionSample {
        self.samples.get(&node).copied().unwrap_or_default()
    }

    /// Snapshot all nodes currently owned by the motion system.
    pub fn samples(&self) -> Vec<(K, MotionSample)> {
        self.nodes
            .keys()
            .map(|node| (*node, self.sample(*node)))
            .collect()
    }

    /// Whether another animation frame is required.
    pub fn is_active(&self) -> bool {
        self.transforms.is_active() || self.opacities.is_active()
    }

    fn sample_node(&mut self, node: K) {
        let Some(motion) = self.nodes.get(&node) else {
            return;
        };
        let sample = MotionSample {
            transform: motion
                .transform
                .and_then(|id| self.transforms.value(id))
                .map(|value| value.map(|value| value as f32)),
            opacity: motion
                .opacity
                .and_then(|id| self.opacities.value(id))
                .map(|value| value[0] as f32),
        };
        self.samples.insert(node, sample);
    }
}

fn validate_matrix(value: [f64; 6]) -> Result<(), String> {
    if value.into_iter().all(f64::is_finite) {
        Ok(())
    } else {
        Err("motion transform must contain only finite numbers".into())
    }
}

fn validate_optional_matrix(value: Option<[f64; 6]>) -> Result<(), String> {
    value.map_or(Ok(()), validate_matrix)
}

fn validate_optional_scalar(value: Option<f64>, name: &str) -> Result<(), String> {
    if value.is_none_or(f64::is_finite) {
        Ok(())
    } else {
        Err(format!("motion {name} must be finite"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(left: f64, right: f64, epsilon: f64) {
        assert!((left - right).abs() <= epsilon, "{left} != {right}");
    }

    #[test]
    fn tween_retargets_from_the_current_sample_and_finishes_exactly() {
        let mut arena = MotionArena::<1>::default();
        let id = arena.insert_tween([0.0], [10.0], Duration::from_millis(100), Easing::Linear);
        assert_eq!(arena.advance(Duration::from_millis(40)), [id]);
        approx(arena.value(id).unwrap()[0], 4.0, 1e-12);
        assert!(arena.retarget_tween(id, [0.0], Duration::from_millis(40), Easing::Linear));
        arena.advance(Duration::from_millis(20));
        approx(arena.value(id).unwrap()[0], 2.0, 1e-12);
        assert_eq!(arena.advance(Duration::from_secs(1)), [id]);
        assert_eq!(arena.value(id), Some([0.0]));
        assert!(!arena.is_active());
    }

    #[test]
    fn spring_is_independent_of_frame_partitioning() {
        let options = SpringOptions {
            stiffness: 210.0,
            damping: 18.0,
            ..SpringOptions::default()
        };
        let mut one = MotionArena::<1>::default();
        let mut many = MotionArena::<1>::default();
        let one_id = one.insert_spring([0.0], [100.0], options);
        let many_id = many.insert_spring([0.0], [100.0], options);
        one.advance(Duration::from_millis(160));
        for _ in 0..10 {
            many.advance(Duration::from_millis(16));
        }
        approx(
            one.value(one_id).unwrap()[0],
            many.value(many_id).unwrap()[0],
            1e-9,
        );
    }

    #[test]
    fn retargeted_spring_preserves_velocity() {
        let mut arena = MotionArena::<1>::default();
        let id = arena.insert_spring([0.0], [100.0], SpringOptions::default());
        arena.advance(Duration::from_millis(80));
        let before = arena.value(id).unwrap()[0];
        assert!(arena.retarget_spring(id, [-20.0], SpringOptions::default()));
        arena.advance(Duration::from_millis(1));
        let after = arena.value(id).unwrap()[0];
        assert!(after > before, "velocity was discarded at retarget");
    }

    #[test]
    fn all_damping_regimes_converge_and_stale_ids_are_rejected() {
        for damping in [4.0, 20.0, 60.0] {
            let mut arena = MotionArena::<2>::default();
            let id = arena.insert_spring(
                [10.0, -5.0],
                [0.0, 2.0],
                SpringOptions {
                    stiffness: 100.0,
                    damping,
                    ..SpringOptions::default()
                },
            );
            for _ in 0..1_000 {
                arena.advance(Duration::from_millis(16));
                if !arena.is_active() {
                    break;
                }
            }
            assert_eq!(arena.value(id), Some([0.0, 2.0]));
            assert!(arena.remove(id));
            assert_eq!(arena.value(id), None);
        }
    }

    #[test]
    fn invalid_spring_options_are_sanitized() {
        let mut arena = MotionArena::<1>::default();
        let id = arena.insert_spring(
            [0.0],
            [1.0],
            SpringOptions {
                mass: f64::NAN,
                stiffness: -1.0,
                damping: f64::INFINITY,
                epsilon: 0.0,
            },
        );
        arena.advance(Duration::from_millis(16));
        assert!(arena.value(id).unwrap()[0].is_finite());
    }

    #[test]
    fn motion_system_owns_transition_lifecycle_and_completion() {
        let mut system = MotionSystem::<u32>::default();
        system
            .set(
                42,
                MotionKind::Transition,
                r#"{
                    "generation": 9,
                    "duration": 0.1,
                    "easing": "linear",
                    "fromTransform": [1, 0, 0, 1, 0, 0],
                    "toTransform": [1, 0, 0, 1, 20, 10],
                    "fromOpacity": 0,
                    "toOpacity": 1
                }"#,
            )
            .unwrap();
        assert!(system.advance(Duration::from_millis(50)).is_empty());
        assert_eq!(
            system.sample(42),
            MotionSample {
                transform: Some([1.0, 0.0, 0.0, 1.0, 10.0, 5.0]),
                opacity: Some(0.5),
            }
        );
        assert_eq!(
            system.advance(Duration::from_millis(50)),
            [MotionCompletion {
                node: 42,
                generation: 9,
            }]
        );
        assert!(!system.is_active());
    }

    #[test]
    fn motion_system_retargets_springs_without_resetting_velocity() {
        let mut system = MotionSystem::<u32>::default();
        system
            .set(
                1,
                MotionKind::Spring,
                r#"{
                    "response": 0.4,
                    "damping": 0.7,
                    "initialTransform": [1, 0, 0, 1, 0, 0],
                    "targetTransform": [1, 0, 0, 1, 100, 0]
                }"#,
            )
            .unwrap();
        system.advance(Duration::from_millis(80));
        let before = system.sample(1).transform.unwrap()[4];
        system
            .set(
                1,
                MotionKind::Spring,
                r#"{
                    "response": 0.4,
                    "damping": 0.7,
                    "targetTransform": [1, 0, 0, 1, -20, 0]
                }"#,
            )
            .unwrap();
        system.advance(Duration::from_millis(1));
        assert!(system.sample(1).transform.unwrap()[4] > before);
    }

    #[test]
    fn reduced_motion_uses_the_same_sampling_path_without_frames() {
        let mut system = MotionSystem::<u32>::default();
        system
            .set(
                7,
                MotionKind::Spring,
                r#"{
                    "targetTransform": [1, 0, 0, 1, 42, 8],
                    "targetOpacity": 0.25,
                    "reducedMotion": true
                }"#,
            )
            .unwrap();
        assert!(!system.is_active());
        assert_eq!(
            system.sample(7),
            MotionSample {
                transform: Some([1.0, 0.0, 0.0, 1.0, 42.0, 8.0]),
                opacity: Some(0.25),
            }
        );
    }

    #[test]
    fn value_motion_store_samples_tweens_and_invalidates_handles() {
        let mut store = ValueMotionStore::default();
        let id = store
            .create_json(r#"{"kind":"tween","duration":1,"easing":"linear"}"#)
            .unwrap();
        assert_eq!(
            store.sample(id, 0.25),
            Some(ValueMotionSample {
                value: 0.25,
                velocity: 0.0,
                done: false,
            })
        );
        assert!(store.sample(id, 1.0).unwrap().done);
        assert!(store.remove(id));
        assert_eq!(store.sample(id, 1.0), None);
    }

    #[test]
    fn value_spring_overshoots_and_settles_at_exact_target() {
        let mut store = ValueMotionStore::default();
        let id = store
            .create_json(
                r#"{"kind":"spring","stiffness":120,"damping":8,"mass":1,"velocity":0,"restDelta":0.001,"restSpeed":0.01}"#,
            )
            .unwrap();
        assert!(store.sample(id, 0.3).unwrap().value > 1.0);
        assert_eq!(store.sample(id, 3.0).unwrap().value, 1.0);
        assert!(store.sample(id, 3.0).unwrap().done);
    }
}
