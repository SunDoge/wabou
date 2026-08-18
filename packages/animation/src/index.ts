import "@wabou/core";
import { rotate2d, type Affine2D } from "@wabou/core/style";
import type {
  AnimationPlaybackControlsWithThen as MotionControls,
  ValueAnimationOptions,
} from "motion-dom";
import { animateValue } from "motion-dom";
import {
  createEffect,
  createSignal,
  onCleanup,
  type Accessor,
  untrack,
} from "solid-js";

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

export interface ReactiveAnimation<T> {
  value: Accessor<T>;
  controls: AnimationControls;
}

type MaybeAccessor<T> = T | Accessor<T>;

const read = <T>(value: MaybeAccessor<T> | undefined, fallback: T): T =>
  typeof value === "function" ? (value as Accessor<T>)() : (value ?? fallback);

export interface TransitionOptions
  extends Omit<
    AnimationOptions<number>,
    "autoplay" | "onUpdate" | "onComplete"
  > {
  /** Skip interpolation while the user's/application's reduced-motion policy is active. */
  reducedMotion?: MaybeAccessor<boolean>;
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
export function createTransition(
  target: Accessor<number>,
  options: TransitionOptions = {},
): ReactiveTransition {
  const [value, setValue] = createSignal(target());
  const [state, setState] = createSignal<AnimationState>("idle");
  let controls: AnimationControls | undefined;
  let generation = 0;

  const stop = () => {
    generation++;
    controls?.stop();
    controls = undefined;
    setState("idle");
  };
  const jump = (next: number) => {
    stop();
    setValue(next);
    options.onUpdate?.(next);
    options.onComplete?.(next);
  };

  createEffect(
    () =>
      [target(), read(options.reducedMotion, false), untrack(value)] as const,
    ([next, reduced, current]) => {
      if (Object.is(next, current)) return;
      if (reduced || options.type === false || options.duration === 0) {
        jump(next);
        return;
      }

      const run = ++generation;
      controls?.stop();
      setState("running");
      const {
        reducedMotion: _reducedMotion,
        onUpdate,
        onComplete,
        ...animationOptions
      } = options;
      controls = animate(current, next, {
        ...animationOptions,
        onUpdate(current) {
          if (run !== generation) return;
          setValue(current);
          onUpdate?.(current);
        },
        onComplete() {
          if (run !== generation) return;
          controls = undefined;
          setValue(next);
          setState("finished");
          onComplete?.(next);
        },
      });
    },
  );

  onCleanup(stop);
  return { value, state, jump, stop };
}

export interface LoopOptions
  extends Omit<AnimationOptions<number>, "onUpdate"> {
  from?: number;
  to?: number;
  onUpdate?: (value: number) => void;
}

/**
 * Lifecycle-owned repeating scalar animation for Solid components.
 *
 * The controls stop automatically with the current Solid owner.
 */
export function createLoop(
  options: LoopOptions = {},
): ReactiveAnimation<number> {
  const from = options.from ?? 0;
  const to = options.to ?? 1;
  const { from: _from, to: _to, onUpdate, ...animationOptions } = options;
  const [value, setValue] = createSignal(from);
  const controls = animate(from, to, {
    duration: 1,
    ease: "linear",
    repeat: Infinity,
    ...animationOptions,
    onUpdate(next) {
      setValue(next);
      onUpdate?.(next);
    },
  });
  onCleanup(() => controls.stop());
  return { value, controls };
}

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

/** Repeating center-pivoted rotation backed by Motion value animation. */
export function createRotation(
  options: RotationOptions = {},
): RotationAnimation {
  const loop = createLoop({
    ...options,
    from: options.from ?? 0,
    to: options.to ?? Math.PI * 2,
  });
  return {
    ...loop,
    angle: loop.value,
    transform: () => rotate2d(loop.value()),
  };
}

export interface PulseOptions
  extends Omit<AnimationOptions<number>, "onUpdate"> {
  from?: number;
  to?: number;
  onUpdate?: (value: number) => void;
}

/** Repeating from→to→from value animation with automatic cleanup. */
export function createPulse(
  options: PulseOptions = {},
): ReactiveAnimation<number> {
  const from = options.from ?? 0.5;
  const to = options.to ?? 1;
  const { from: _from, to: _to, onUpdate, ...animationOptions } = options;
  const [value, setValue] = createSignal(from);
  const controls = animateKeyframes([from, to, from], {
    duration: 1,
    ease: "easeInOut",
    repeat: Infinity,
    ...animationOptions,
    onUpdate(next) {
      setValue(next);
      onUpdate?.(next);
    },
  });
  onCleanup(() => controls.stop());
  return { value, controls };
}
