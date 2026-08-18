import "@wabou/core";
import { Affine2D } from "@wabou/core/style";
import { Accessor } from "solid-js";
//#region src/index.d.ts
type AnimationValue = number | string;
type AnimationType = "tween" | "spring" | false;
type RepeatType = "loop" | "reverse" | "mirror";
type EasingFunction = (progress: number) => number;
type Easing = "linear" | "easeIn" | "easeOut" | "easeInOut" | "circIn" | "circOut" | "circInOut" | "backIn" | "backOut" | "backInOut" | "anticipate" | EasingFunction | readonly [number, number, number, number];
/** Wabou animation options. All time values are expressed in seconds. */
interface AnimationOptions<V extends AnimationValue = number> {
  type?: AnimationType;
  duration?: number;
  visualDuration?: number;
  delay?: number;
  ease?: Easing | Easing[];
  times?: number[];
  repeat?: number;
  repeatType?: RepeatType;
  repeatDelay?: number;
  autoplay?: boolean;
  stiffness?: number;
  damping?: number;
  mass?: number;
  bounce?: number;
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
/** Backend-independent playback handle returned by Wabou animations. */
interface AnimationControls extends PromiseLike<void> {
  time: number;
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
/** @deprecated Use AnimationControls. */
type AnimationPlaybackControls = AnimationControls;
/** Animate between two scalar values. */
declare function animate<V extends AnimationValue>(from: V, to: V, options?: AnimationOptions<V>): AnimationControls;
/** Animate through two or more scalar keyframes. */
declare function animateKeyframes<V extends AnimationValue>(keyframes: readonly [V, V, ...V[]], options?: AnimationOptions<V>): AnimationControls;
interface ReactiveAnimation<T> {
  value: Accessor<T>;
  controls: AnimationControls;
}
type MaybeAccessor<T> = T | Accessor<T>;
interface TransitionOptions extends Omit<AnimationOptions<number>, "autoplay" | "onUpdate" | "onComplete"> {
  /** Skip interpolation while the user's/application's reduced-motion policy is active. */
  reducedMotion?: MaybeAccessor<boolean>;
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
interface LoopOptions extends Omit<AnimationOptions<number>, "onUpdate"> {
  from?: number;
  to?: number;
  onUpdate?: (value: number) => void;
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
/** Repeating center-pivoted rotation backed by Motion value animation. */
declare function createRotation(options?: RotationOptions): RotationAnimation;
interface PulseOptions extends Omit<AnimationOptions<number>, "onUpdate"> {
  from?: number;
  to?: number;
  onUpdate?: (value: number) => void;
}
/** Repeating from→to→from value animation with automatic cleanup. */
declare function createPulse(options?: PulseOptions): ReactiveAnimation<number>;
//#endregion
export { AnimationControls, AnimationOptions, AnimationPlaybackControls, AnimationState, AnimationType, AnimationValue, Easing, EasingFunction, LoopOptions, PulseOptions, ReactiveAnimation, ReactiveTransition, RepeatType, RotationAnimation, RotationOptions, TransitionOptions, animate, animateKeyframes, createLoop, createPulse, createRotation, createTransition };
//# sourceMappingURL=index.d.mts.map