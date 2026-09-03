import "@wabou/core";
import { type Affine2D, rotate2d, translate2d } from "@wabou/core/style";
import type {
  AnimationPlaybackControlsWithThen as MotionControls,
  InterpolateOptions,
  ValueAnimationOptions,
} from "motion-dom";
import { animateValue, interpolate } from "motion-dom";
import {
  type Accessor,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  untrack,
} from "solid-js";

export {
  type MotionConfig,
  MotionConfigProvider,
  type MotionConfigProviderProps,
  useMotionConfig,
  useReducedMotion,
} from "./config";

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

/**
 * Backend-neutral repeating timeline executed by a retained native widget.
 *
 * JS owns animation intent and lifecycle; the native backend samples this
 * descriptor locally without receiving one protocol mutation per frame.
 */
export interface NativeLoopAnimation {
  readonly kind: "loop";
  /** Duration of one iteration in seconds. */
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

function positiveFinite(value: number, fallback: number): number {
  return Number.isFinite(value) && value > 0 ? value : fallback;
}

/**
 * Compile reactive Solid animation policy into a stable native timeline DTO.
 * This accessor changes only when authored policy changes, never per frame.
 */
export function createNativeLoopAnimation(
  options: NativeLoopAnimationOptions = {},
): Accessor<NativeLoopAnimation> {
  return createMemo(() => ({
    kind: "loop" as const,
    duration: positiveFinite(read(options.duration, 1), 1),
    speed: positiveFinite(read(options.speed, 1), 1),
    paused: read(options.paused, false),
    reducedMotion: read(options.reducedMotion, false),
  }));
}

export interface KeyframeAnimationOptions<V extends AnimationValue>
  extends AnimationOptions<V> {
  /** Reactive policy which pauses interpolation and publishes the final keyframe. */
  reducedMotion?: MaybeAccessor<boolean>;
  /** Value exposed while motion is reduced. Defaults to the final keyframe. */
  reducedValue?: V;
}

/**
 * Lifecycle-owned finite or repeating keyframe animation.
 *
 * This is the general primitive behind loops, pulses and component-specific
 * effects. It owns cleanup and reduced-motion behavior so components don't
 * need to coordinate raw Motion controls themselves.
 */
export function createKeyframeAnimation<V extends AnimationValue>(
  keyframes: readonly [V, V, ...V[]],
  options: KeyframeAnimationOptions<V> = {},
): ReactiveAnimation<V> {
  const {
    reducedMotion,
    reducedValue = keyframes[keyframes.length - 1],
    onUpdate,
    ...animationOptions
  } = options;
  const authoredAutoplay = animationOptions.autoplay ?? true;
  const initiallyReduced = untrack(() => read(reducedMotion, false));
  const [box, setBox] = createSignal({
    value: initiallyReduced ? reducedValue : keyframes[0],
  });
  const value = () => box().value;
  const controls = animateKeyframes(keyframes, {
    ...animationOptions,
    autoplay: authoredAutoplay && !initiallyReduced,
    onUpdate(next) {
      setBox({ value: next });
      onUpdate?.(next);
    },
  });
  let initialized = false;
  let resumeAfterReduction = authoredAutoplay;
  createEffect(
    () => read(reducedMotion, false),
    (reduced) => {
      if (reduced) {
        if (initialized) resumeAfterReduction = controls.state === "running";
        controls.pause();
        setBox({ value: reducedValue });
        onUpdate?.(reducedValue);
      } else if (initialized && resumeAfterReduction) {
        controls.play();
      }
      initialized = true;
    },
  );
  onCleanup(() => controls.stop());
  return { value, controls };
}

export interface MotionInterpolationOptions<V extends AnimationValue>
  extends Pick<InterpolateOptions<V>, "clamp" | "ease"> {}

/** Map one reactive progress value to numeric, color, or complex keyframes. */
export function createInterpolation(
  source: Accessor<number>,
  input: readonly number[],
  output: readonly number[],
  options?: MotionInterpolationOptions<number>,
): Accessor<number>;
export function createInterpolation(
  source: Accessor<number>,
  input: readonly number[],
  output: readonly string[],
  options?: MotionInterpolationOptions<string>,
): Accessor<string>;
export function createInterpolation<V extends AnimationValue>(
  source: Accessor<number>,
  input: readonly number[],
  output: readonly V[],
  options: MotionInterpolationOptions<V> = {},
): Accessor<V> {
  if (input.length === 0 || input.length !== output.length) {
    throw new RangeError(
      "animation input and output ranges must have equal non-zero lengths",
    );
  }
  if (!input.every(Number.isFinite)) {
    throw new RangeError(
      "animation input range must contain only finite numbers",
    );
  }
  const transform = interpolate([...input], [...output], options);
  return () => transform(source());
}

export interface TransitionOptions
  extends Omit<
    AnimationOptions<number>,
    "autoplay" | "onUpdate" | "onComplete"
  > {
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
export function createTransition(
  target: Accessor<number>,
  options: TransitionOptions = {},
): ReactiveTransition {
  const [value, setValue] = createSignal(
    untrack(() => read(options.initial, target())),
    { ownedWrite: true },
  );
  const [state, setState] = createSignal<AnimationState>("idle", {
    ownedWrite: true,
  });
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
        initial: _initial,
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

function createRepeatingAnimation(
  keyframes: readonly [number, number, ...number[]],
  options: RepeatingOptions & { reducedValue: number },
): ReactiveAnimation<number> {
  return createKeyframeAnimation(keyframes, options);
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
  const {
    from: _from,
    to: _to,
    reducedMotion,
    reducedValue = from,
    onUpdate,
    ...animationOptions
  } = options;
  return createRepeatingAnimation([from, to], {
    duration: 1,
    ease: "linear",
    repeat: Infinity,
    ...animationOptions,
    reducedMotion,
    reducedValue,
    onUpdate,
  });
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

export type SweepAxis = "horizontal" | "vertical";

export interface SweepGeometry {
  extent: number;
  itemRatio: number;
}

export function normalizeSweepGeometry(
  extent: number,
  itemRatio: number,
): SweepGeometry {
  return {
    extent: Number.isFinite(extent) ? Math.max(0, extent) : 0,
    itemRatio:
      Number.isFinite(itemRatio) && itemRatio > 0
        ? Math.min(1, itemRatio)
        : 0.4,
  };
}

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
export function createSweep(options: SweepOptions): SweepAnimation {
  const { extent, itemRatio, axis = "horizontal", ...loopOptions } = options;
  const loop = createLoop(loopOptions);
  const geometry = () =>
    normalizeSweepGeometry(read(extent, 0), read(itemRatio, 0.4));
  const offset = () => {
    const current = geometry();
    return (
      current.extent *
      (loop.value() * (1 + current.itemRatio) - current.itemRatio)
    );
  };
  return {
    ...loop,
    offset,
    transform: () =>
      axis === "vertical" ? translate2d(0, offset()) : translate2d(offset(), 0),
  };
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

export interface PulseOptions extends RepeatingOptions {
  from?: number;
  to?: number;
}

/** Repeating from→to→from value animation with automatic cleanup. */
export function createPulse(
  options: PulseOptions = {},
): ReactiveAnimation<number> {
  const from = options.from ?? 0.5;
  const to = options.to ?? 1;
  const {
    from: _from,
    to: _to,
    reducedMotion,
    reducedValue = to,
    onUpdate,
    ...animationOptions
  } = options;
  return createRepeatingAnimation([from, to, from], {
    duration: 1,
    ease: "easeInOut",
    repeat: Infinity,
    ...animationOptions,
    reducedMotion,
    reducedValue,
    onUpdate,
  });
}
