import "@wabou/core";
import type {
  AnimationPlaybackControlsWithThen as MotionControls,
  ValueAnimationOptions,
} from "motion-dom";
import { animateValue } from "motion-dom";

export type AnimationValue = number | string;
export type AnimationType = "tween" | "spring" | false;
export type RepeatType = "loop" | "reverse" | "mirror";
export type EasingFunction = (progress: number) => number;
export type Easing =
  | "linear"
  | "easeIn"
  | "easeOut"
  | "easeInOut"
  | "circIn"
  | "circOut"
  | "circInOut"
  | "backIn"
  | "backOut"
  | "backInOut"
  | "anticipate"
  | EasingFunction
  | readonly [number, number, number, number];

/** Wabou animation options. All time values are expressed in seconds. */
export interface AnimationOptions<V extends AnimationValue = number> {
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

export type AnimationState = "idle" | "running" | "paused" | "finished";

/** Backend-independent playback handle returned by Wabou animations. */
export interface AnimationControls extends PromiseLike<void> {
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
export type AnimationPlaybackControls = AnimationControls;

class Controls implements AnimationControls {
  constructor(private readonly backend: MotionControls) {}

  get time(): number {
    return this.backend.time;
  }
  set time(value: number) {
    this.backend.time = value;
  }
  get speed(): number {
    return this.backend.speed;
  }
  set speed(value: number) {
    this.backend.speed = value;
  }
  get duration(): number {
    return this.backend.duration;
  }
  get state(): AnimationState {
    const state = this.backend.state;
    if (state === "running" || state === "paused" || state === "finished") {
      return state;
    }
    return "idle";
  }
  get finished(): Promise<void> {
    return this.backend.finished.then(() => undefined);
  }
  play(): void {
    this.backend.play();
  }
  pause(): void {
    this.backend.pause();
  }
  stop(): void {
    this.backend.stop();
  }
  cancel(): void {
    this.backend.cancel();
  }
  complete(): void {
    this.backend.complete();
  }
  // biome-ignore lint/suspicious/noThenProperty: Playback controls intentionally implement PromiseLike.
  then<TResult1 = void, TResult2 = never>(
    // biome-ignore lint/suspicious/noConfusingVoidType: PromiseLike<void> requires this callback shape.
    onfulfilled?: ((value: void) => TResult1 | PromiseLike<TResult1>) | null,
    onrejected?: ((reason: unknown) => TResult2 | PromiseLike<TResult2>) | null,
  ): PromiseLike<TResult1 | TResult2> {
    return this.finished.then(onfulfilled, onrejected);
  }
}

function normalizeOptions<V extends AnimationValue>(
  options: AnimationOptions<V>,
): ValueAnimationOptions<V> {
  const normalized = { ...options } as ValueAnimationOptions<V>;
  for (const key of ["delay", "duration", "repeatDelay"] as const) {
    const seconds = options[key];
    if (typeof seconds === "number") normalized[key] = seconds * 1000;
  }
  if (typeof options.visualDuration === "number") {
    normalized.visualDuration = options.visualDuration * 1000;
  }
  return normalized;
}

/** Animate between two scalar values. */
export function animate<V extends AnimationValue>(
  from: V,
  to: V,
  options: AnimationOptions<V> = {},
): AnimationControls {
  return animateKeyframes([from, to], options);
}

/** Animate through two or more scalar keyframes. */
export function animateKeyframes<V extends AnimationValue>(
  keyframes: readonly [V, V, ...V[]],
  options: AnimationOptions<V> = {},
): AnimationControls {
  const backend = animateValue({
    ...normalizeOptions(options),
    keyframes: [...keyframes],
  });
  return new Controls(backend);
}
