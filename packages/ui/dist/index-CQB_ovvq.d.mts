import { WabouNativeSpring, WabouNativeTransition } from "@wabou/core";
import { Affine2D } from "@wabou/core/style";
import { Accessor, JSX } from "solid-js";
//#region src/animation/config.d.ts
interface MotionConfig {
  reducedMotion: Accessor<boolean>;
}
interface MotionConfigProviderProps {
  /** Disable non-essential interpolation while preserving final UI state. */
  reducedMotion?: boolean;
  children?: JSX.Element;
}
/** Application-level motion policy inherited by all styled Wabou components. */
declare function MotionConfigProvider(props: MotionConfigProviderProps): JSX.Element;
declare function useMotionConfig(): MotionConfig;
declare function useReducedMotion(): Accessor<boolean>;
//#endregion
//#region src/animation/native.d.ts
type MaybeAccessor$1<T> = T | Accessor<T>;
/** Renderer properties which can currently run without per-frame JS work. */
interface MotionTarget {
  transform?: Affine2D;
  opacity?: number;
}
interface SpringOptions {
  /** Approximate response period in seconds. */
  response?: number;
  /** Damping ratio: below 1 bounces, 1 is critical, above 1 is overdamped. */
  damping?: number;
  mass?: number;
  stiffness?: number;
  restDelta?: number;
  /** Animate from this value on mount. `false` and omission both snap initially. */
  initial?: MotionTarget | false;
  reducedMotion?: MaybeAccessor$1<boolean>;
}
/**
 * Publish spring targets to the native renderer.
 *
 * The accessor changes only when its authored target changes. Rust retains the
 * sampled value and velocity between publications and across ordinary JS HMR.
 */
declare function useSpring(target: MaybeAccessor$1<MotionTarget>, options?: SpringOptions): Accessor<WabouNativeSpring>;
interface TweenOptions {
  duration?: number;
  easing?: "linear" | "easeInOut" | "easeOut";
  /** Animate from this value on mount. `false` and omission both snap initially. */
  initial?: MotionTarget | false;
  reducedMotion?: MaybeAccessor$1<boolean>;
}
/** Publish an interruptible finite tween to the native renderer. */
declare function useTween(target: MaybeAccessor$1<MotionTarget>, options?: TweenOptions): Accessor<WabouNativeTransition>;
//#endregion
//#region src/animation/index.d.ts
type AnimationValue = number | string;
type AnimationType = "tween" | "spring" | false;
type RepeatType = "loop" | "reverse" | "mirror";
type Easing = "linear" | "easeIn" | "easeOut" | "easeInOut" | "circIn" | "circOut" | "circInOut" | "backIn" | "backOut" | "backInOut" | "anticipate" | readonly [number, number, number, number];
/** Wabou animation options. Time values are seconds. */
interface AnimationOptions<V extends AnimationValue = number> {
  type?: AnimationType;
  duration?: number;
  delay?: number;
  ease?: Easing;
  times?: number[];
  repeat?: number;
  repeatType?: RepeatType;
  repeatDelay?: number;
  autoplay?: boolean;
  stiffness?: number;
  damping?: number;
  mass?: number;
  velocity?: number;
  restSpeed?: number;
  restDelta?: number;
  onUpdate?: (value: V) => void;
  onPlay?: () => void;
  onComplete?: () => void;
  onRepeat?: () => void;
  onStop?: () => void;
}
type AnimationState = "idle" | "running" | "paused" | "finished";
/** Playback handle for a Rust-sampled Wabou value animation. */
interface AnimationControls extends PromiseLike<void> {
  speed: number;
  readonly duration: number;
  readonly state: AnimationState;
  readonly finished: Promise<void>;
  play(): void;
  pause(): void;
  stop(): void;
  cancel(): void;
  complete(): void;
}
/** Animate between two supported values using a Rust-owned curve. */
declare function animate<V extends AnimationValue>(from: V, to: V, options?: AnimationOptions<V>): AnimationControls;
/** Animate numeric or color keyframes using a Rust-owned curve. */
declare function animateKeyframes<V extends AnimationValue>(keyframes: readonly [V, V, ...V[]], options?: AnimationOptions<V>): AnimationControls;
interface ReactiveAnimation<T> {
  value: Accessor<T>;
  controls: AnimationControls;
}
type MaybeAccessor<T> = T | Accessor<T>;
interface NativeLoopAnimation {
  readonly kind: "loop";
  readonly duration: number;
  readonly speed: number;
  readonly paused: boolean;
  readonly reducedMotion: boolean;
}
interface NativeLoopAnimationOptions {
  duration?: MaybeAccessor<number>;
  speed?: MaybeAccessor<number>;
  paused?: MaybeAccessor<boolean>;
  reducedMotion?: MaybeAccessor<boolean>;
}
declare function createNativeLoopAnimation(options?: NativeLoopAnimationOptions): Accessor<NativeLoopAnimation>;
interface KeyframeAnimationOptions<V extends AnimationValue> extends AnimationOptions<V> {
  reducedMotion?: MaybeAccessor<boolean>;
  reducedValue?: V;
}
declare function createKeyframeAnimation<V extends AnimationValue>(keyframes: readonly [V, V, ...V[]], options?: KeyframeAnimationOptions<V>): ReactiveAnimation<V>;
interface MotionInterpolationOptions<V extends AnimationValue> {
  clamp?: boolean;
  ease?: Easing;
}
declare function createInterpolation(source: Accessor<number>, input: readonly number[], output: readonly number[], options?: MotionInterpolationOptions<number>): Accessor<number>;
declare function createInterpolation(source: Accessor<number>, input: readonly number[], output: readonly string[], options?: MotionInterpolationOptions<string>): Accessor<string>;
interface TransitionOptions extends Omit<AnimationOptions<number>, "autoplay" | "duration" | "onUpdate" | "onComplete"> {
  /** Duration in seconds. May react to the current transition direction. */
  duration?: MaybeAccessor<number>;
  /** Skip interpolation while the user's/application's reduced-motion policy is active. */
  reducedMotion?: MaybeAccessor<boolean>;
  /** One-time starting value. Defaults to the current target. */
  initial?: MaybeAccessor<number>;
  onUpdate?: (value: number) => void;
  onComplete?: (value: number) => void;
}
interface ReactiveTransition {
  value: Accessor<number>;
  state: Accessor<AnimationState>;
  /** Cancel the current run and synchronously move to a value. */
  jump(value: number): void;
  stop(): void;
}
/**
 * Lifecycle-owned scalar transition that retargets from its current value.
 *
 * Unlike a one-shot animation, changing `target` while a run is active does
 * not restart from the previous keyframe. This makes it suitable for rapidly
 * toggled disclosure, hover and selection state.
 */
declare function createTransition(target: Accessor<number>, options?: TransitionOptions): ReactiveTransition;
interface RepeatingOptions extends Omit<AnimationOptions<number>, "onUpdate"> {
  /** Reactive policy which pauses the loop and publishes `reducedValue`. */
  reducedMotion?: MaybeAccessor<boolean>;
  /** Stable value exposed while motion is reduced. */
  reducedValue?: number;
  onUpdate?: (value: number) => void;
}
interface LoopOptions extends RepeatingOptions {
  from?: number;
  to?: number;
}
/**
 * Lifecycle-owned repeating scalar animation for Solid components.
 *
 * The controls stop automatically with the current Solid owner.
 */
declare function createLoop(options?: LoopOptions): ReactiveAnimation<number>;
interface RotationOptions extends Omit<LoopOptions, "from" | "to"> {
  /** Initial angle in radians. Defaults to zero. */
  from?: number;
  /** Final angle in radians. Defaults to one full turn. */
  to?: number;
}
interface RotationAnimation extends ReactiveAnimation<number> {
  angle: Accessor<number>;
  transform: Accessor<Affine2D>;
}
type SweepAxis = "horizontal" | "vertical";
interface SweepGeometry {
  extent: number;
  itemRatio: number;
}
declare function normalizeSweepGeometry(extent: number, itemRatio: number): SweepGeometry;
interface SweepOptions extends LoopOptions {
  /** Current container width or height in logical pixels. */
  extent: MaybeAccessor<number>;
  /** Moving item's size as a fraction of the container. Defaults to 0.4. */
  itemRatio?: MaybeAccessor<number>;
  axis?: SweepAxis;
}
interface SweepAnimation extends ReactiveAnimation<number> {
  offset: Accessor<number>;
  transform: Accessor<Affine2D>;
}
/**
 * Move an item completely across one measured axis using only a runtime
 * transform. Both repeat boundaries remain outside the container, avoiding a
 * visible reset and avoiding per-frame layout invalidation.
 */
declare function createSweep(options: SweepOptions): SweepAnimation;
/** Repeating center-pivoted rotation backed by Motion value animation. */
declare function createRotation(options?: RotationOptions): RotationAnimation;
interface PulseOptions extends RepeatingOptions {
  from?: number;
  to?: number;
}
/** Repeating from→to→from value animation with automatic cleanup. */
declare function createPulse(options?: PulseOptions): ReactiveAnimation<number>;
//#endregion
export { createRotation as A, MotionConfigProvider as B, animate as C, createLoop as D, createKeyframeAnimation as E, SpringOptions as F, useMotionConfig as H, TweenOptions as I, useSpring as L, createTransition as M, normalizeSweepGeometry as N, createNativeLoopAnimation as O, MotionTarget as P, useTween as R, TransitionOptions as S, createInterpolation as T, useReducedMotion as U, MotionConfigProviderProps as V, RotationOptions as _, AnimationValue as a, SweepGeometry as b, LoopOptions as c, NativeLoopAnimationOptions as d, PulseOptions as f, RotationAnimation as g, RepeatType as h, AnimationType as i, createSweep as j, createPulse as k, MotionInterpolationOptions as l, ReactiveTransition as m, AnimationOptions as n, Easing as o, ReactiveAnimation as p, AnimationState as r, KeyframeAnimationOptions as s, AnimationControls as t, NativeLoopAnimation as u, SweepAnimation as v, animateKeyframes as w, SweepOptions as x, SweepAxis as y, MotionConfig as z };
//# sourceMappingURL=index-CQB_ovvq.d.mts.map