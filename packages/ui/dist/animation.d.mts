import { MotionConfig, MotionConfigProvider, MotionConfigProviderProps, useMotionConfig, useReducedMotion } from "./animation/config.mjs";
import { MotionTarget, SpringOptions, TweenOptions, useSpring, useTween } from "./animation/native.mjs";
import "@wabou/core";
import { Affine2D } from "@wabou/core/style";
import { Accessor } from "solid-js";
//#region src/animation/index.d.ts
export type AnimationValue = number | string;
export type AnimationType = "tween" | "spring" | false;
export type RepeatType = "loop" | "reverse" | "mirror";
export type Easing = "linear" | "easeIn" | "easeOut" | "easeInOut" | "circIn" | "circOut" | "circInOut" | "backIn" | "backOut" | "backInOut" | "anticipate" | readonly [number, number, number, number];
/** Wabou animation options. Time values are seconds. */
export interface AnimationOptions<V extends AnimationValue = number> {
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
export type AnimationState = "idle" | "running" | "paused" | "finished";
/** Playback handle for a Rust-sampled Wabou value animation. */
export interface AnimationControls extends PromiseLike<void> {
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
export declare function animate<V extends AnimationValue>(from: V, to: V, options?: AnimationOptions<V>): AnimationControls;
/** Animate numeric or color keyframes using a Rust-owned curve. */
export declare function animateKeyframes<V extends AnimationValue>(keyframes: readonly [V, V, ...V[]], options?: AnimationOptions<V>): AnimationControls;
export interface ReactiveAnimation<T> {
  value: Accessor<T>;
  controls: AnimationControls;
}
type MaybeAccessor<T> = T | Accessor<T>;
export interface NativeLoopAnimation {
  readonly kind: "loop";
  readonly duration: number;
  readonly speed: number;
  readonly paused: boolean;
  readonly reducedMotion: boolean;
}
export interface NativeLoopAnimationOptions {
  duration?: MaybeAccessor<number>;
  speed?: MaybeAccessor<number>;
  paused?: MaybeAccessor<boolean>;
  reducedMotion?: MaybeAccessor<boolean>;
}
export declare function createNativeLoopAnimation(options?: NativeLoopAnimationOptions): Accessor<NativeLoopAnimation>;
export interface KeyframeAnimationOptions<V extends AnimationValue> extends AnimationOptions<V> {
  reducedMotion?: MaybeAccessor<boolean>;
  reducedValue?: V;
}
export declare function createKeyframeAnimation<V extends AnimationValue>(keyframes: readonly [V, V, ...V[]], options?: KeyframeAnimationOptions<V>): ReactiveAnimation<V>;
export interface MotionInterpolationOptions {
  clamp?: boolean;
  ease?: Easing;
}
export declare function createInterpolation(source: Accessor<number>, input: readonly number[], output: readonly number[], options?: MotionInterpolationOptions): Accessor<number>;
export declare function createInterpolation(source: Accessor<number>, input: readonly number[], output: readonly string[], options?: MotionInterpolationOptions): Accessor<string>;
export interface TransitionOptions extends Omit<AnimationOptions<number>, "autoplay" | "duration" | "onUpdate" | "onComplete"> {
  /** Duration in seconds. May react to the current transition direction. */
  duration?: MaybeAccessor<number>;
  /** Skip interpolation while the user's/application's reduced-motion policy is active. */
  reducedMotion?: MaybeAccessor<boolean>;
  /** One-time starting value. Defaults to the current target. */
  initial?: MaybeAccessor<number>;
  onUpdate?: (value: number) => void;
  onComplete?: (value: number) => void;
}
export interface ReactiveTransition {
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
export declare function createTransition(target: Accessor<number>, options?: TransitionOptions): ReactiveTransition;
interface RepeatingOptions extends Omit<AnimationOptions<number>, "onUpdate"> {
  /** Reactive policy which pauses the loop and publishes `reducedValue`. */
  reducedMotion?: MaybeAccessor<boolean>;
  /** Stable value exposed while motion is reduced. */
  reducedValue?: number;
  onUpdate?: (value: number) => void;
}
export interface LoopOptions extends RepeatingOptions {
  from?: number;
  to?: number;
}
/**
 * Lifecycle-owned repeating scalar animation for Solid components.
 *
 * The controls stop automatically with the current Solid owner.
 */
export declare function createLoop(options?: LoopOptions): ReactiveAnimation<number>;
export interface RotationOptions extends Omit<LoopOptions, "from" | "to"> {
  /** Initial angle in radians. Defaults to zero. */
  from?: number;
  /** Final angle in radians. Defaults to one full turn. */
  to?: number;
}
export interface RotationAnimation extends ReactiveAnimation<number> {
  angle: Accessor<number>;
  transform: Accessor<Affine2D>;
}
export type SweepAxis = "horizontal" | "vertical";
export interface SweepGeometry {
  extent: number;
  itemRatio: number;
}
export declare function normalizeSweepGeometry(extent: number, itemRatio: number): SweepGeometry;
export interface SweepOptions extends LoopOptions {
  /** Current container width or height in logical pixels. */
  extent: MaybeAccessor<number>;
  /** Moving item's size as a fraction of the container. Defaults to 0.4. */
  itemRatio?: MaybeAccessor<number>;
  axis?: SweepAxis;
}
export interface SweepAnimation extends ReactiveAnimation<number> {
  offset: Accessor<number>;
  transform: Accessor<Affine2D>;
}
/**
 * Move an item completely across one measured axis using only a runtime
 * transform. Both repeat boundaries remain outside the container, avoiding a
 * visible reset and avoiding per-frame layout invalidation.
 */
export declare function createSweep(options: SweepOptions): SweepAnimation;
/** Repeating center-pivoted rotation backed by Motion value animation. */
export declare function createRotation(options?: RotationOptions): RotationAnimation;
export interface PulseOptions extends RepeatingOptions {
  from?: number;
  to?: number;
}
/** Repeating from→to→from value animation with automatic cleanup. */
export declare function createPulse(options?: PulseOptions): ReactiveAnimation<number>;
//#endregion
export { type MotionConfig, MotionConfigProvider, type MotionConfigProviderProps, type MotionTarget, type SpringOptions, type TweenOptions, useMotionConfig, useReducedMotion, useSpring, useTween };
//# sourceMappingURL=animation.d.mts.map