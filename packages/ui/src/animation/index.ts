import "@wabou/core";
import { type Affine2D, rotate2d, translate2d } from "@wabou/core/style";
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
export {
  type MotionTarget,
  type SpringOptions,
  type TweenOptions,
  useSpring,
  useTween,
} from "./native";

export type AnimationValue = number | string;
export type AnimationType = "tween" | "spring" | false;
export type RepeatType = "loop" | "reverse" | "mirror";
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
  | readonly [number, number, number, number];

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

type CurveSample = { value: number; velocity: number; done: boolean };
type CurveDescriptor =
  | {
      kind: "tween";
      duration: number;
      easing: string | { cubicBezier: number[] };
    }
  | {
      kind: "spring";
      stiffness: number;
      damping: number;
      mass: number;
      velocity: number;
      restDelta: number;
      restSpeed: number;
    };

const hasNativeMotion = () =>
  typeof globalThis.__wabou_motion_create === "function";

function fallbackSample(
  descriptor: CurveDescriptor,
  elapsed: number,
): CurveSample {
  if (descriptor.kind === "tween") {
    const raw =
      descriptor.duration === 0
        ? 1
        : Math.min(1, elapsed / descriptor.duration);
    return { value: raw, velocity: 0, done: raw >= 1 };
  }
  // Tests without a Wabou host only need deterministic lifecycle behavior.
  const rate = Math.max(1, descriptor.damping / descriptor.mass);
  const value = 1 - Math.exp(-rate * elapsed);
  const done = 1 - value <= descriptor.restDelta;
  return { value: done ? 1 : value, velocity: 0, done };
}

class Curve {
  readonly id: number | undefined;
  readonly #output = new Float64Array(3);
  constructor(readonly descriptor: CurveDescriptor) {
    this.id = hasNativeMotion()
      ? globalThis.__wabou_motion_create(JSON.stringify(descriptor))
      : undefined;
  }
  sample(elapsed: number): CurveSample {
    if (this.id === undefined) return fallbackSample(this.descriptor, elapsed);
    if (!globalThis.__wabou_motion_sample(this.id, elapsed, this.#output))
      throw new Error("motion handle is no longer live");
    return {
      value: this.#output[0],
      velocity: this.#output[1],
      done: this.#output[2] !== 0,
    };
  }
  drop(): void {
    if (this.id !== undefined) globalThis.__wabou_motion_drop(this.id);
  }
}

function curveDescriptor<V extends AnimationValue>(
  options: AnimationOptions<V>,
): CurveDescriptor {
  if (options.type === "spring") {
    return {
      kind: "spring",
      stiffness: options.stiffness ?? 170,
      damping: options.damping ?? 26,
      mass: options.mass ?? 1,
      velocity: options.velocity ?? 0,
      restDelta: options.restDelta ?? 0.001,
      restSpeed: options.restSpeed ?? 0.01,
    };
  }
  const ease = options.ease;
  return {
    kind: "tween",
    duration: options.type === false ? 0 : Math.max(0, options.duration ?? 0.3),
    easing: Array.isArray(ease)
      ? { cubicBezier: [...ease] }
      : typeof ease === "string"
        ? ease
        : "linear",
  };
}

function parseColor(
  value: string,
): [number, number, number, number] | undefined {
  const hex = /^#([0-9a-f]{3,8})$/i.exec(value)?.[1];
  if (hex) {
    const full =
      hex.length === 3 || hex.length === 4
        ? [...hex].map((part) => part + part).join("")
        : hex;
    if (full.length === 6 || full.length === 8)
      return [
        Number.parseInt(full.slice(0, 2), 16),
        Number.parseInt(full.slice(2, 4), 16),
        Number.parseInt(full.slice(4, 6), 16),
        full.length === 8 ? Number.parseInt(full.slice(6, 8), 16) / 255 : 1,
      ];
  }
  const rgb =
    /^rgba?\(\s*([\d.]+)[, ]+([\d.]+)[, ]+([\d.]+)(?:[, /]+([\d.]+))?\s*\)$/i.exec(
      value,
    );
  return rgb
    ? [
        Number(rgb[1]),
        Number(rgb[2]),
        Number(rgb[3]),
        rgb[4] === undefined ? 1 : Number(rgb[4]),
      ]
    : undefined;
}

const linearChannel = (channel: number) => {
  const value = channel / 255;
  return value * value;
};
const srgbChannel = (channel: number) =>
  Math.max(0, Math.min(255, Math.round(255 * Math.sqrt(Math.max(0, channel)))));

function interpolateValue<V extends AnimationValue>(
  from: V,
  to: V,
  progress: number,
): V {
  if (typeof from === "number" && typeof to === "number") {
    return (from + (to - from) * progress) as V;
  }
  if (typeof from === "string" && typeof to === "string") {
    const a = parseColor(from);
    const b = parseColor(to);
    if (!a || !b)
      throw new TypeError(
        "Wabou animations support numeric and CSS color values",
      );
    const channels = [0, 1, 2].map((index) =>
      srgbChannel(
        linearChannel(a[index]) +
          (linearChannel(b[index]) - linearChannel(a[index])) * progress,
      ),
    );
    const alpha = Math.max(0, Math.min(1, a[3] + (b[3] - a[3]) * progress));
    return `rgba(${channels[0]}, ${channels[1]}, ${channels[2]}, ${Number(alpha.toFixed(4))})` as V;
  }
  throw new TypeError("animation keyframes must have one supported value type");
}

function interpolateKeyframes<V extends AnimationValue>(
  keyframes: readonly V[],
  progress: number,
  times?: readonly number[],
): V {
  if (keyframes.length === 1) return keyframes[0];
  const points =
    times?.length === keyframes.length
      ? times
      : keyframes.map((_, index) => index / (keyframes.length - 1));
  let segment = points.length - 2;
  for (let index = 0; index < points.length - 1; index++) {
    if (progress <= points[index + 1]) {
      segment = index;
      break;
    }
  }
  const start = points[segment];
  const end = points[segment + 1];
  const local = end === start ? 1 : (progress - start) / (end - start);
  return interpolateValue(keyframes[segment], keyframes[segment + 1], local);
}

class Controls<V extends AnimationValue> implements AnimationControls {
  #curve: Curve;
  #state: AnimationState;
  #speed = 1;
  #frame: number | undefined;
  #lastTime: number | undefined;
  #elapsed = 0;
  #iteration = 0;
  #resolve!: () => void;
  readonly finished: Promise<void>;

  constructor(
    private readonly keyframes: readonly V[],
    private readonly options: AnimationOptions<V>,
  ) {
    this.#curve = new Curve(curveDescriptor(options));
    this.#state = options.autoplay === false ? "paused" : "running";
    this.finished = new Promise((resolve) => {
      this.#resolve = resolve;
    });
    options.onUpdate?.(keyframes[0]);
    if (this.#state === "running") {
      options.onPlay?.();
      this.#lastTime = performance.now();
      this.#frame = requestAnimationFrame(this.#tick);
    }
  }

  get speed(): number {
    return this.#speed;
  }
  set speed(value: number) {
    if (!Number.isFinite(value) || value <= 0)
      throw new RangeError("animation speed must be positive");
    this.#speed = value;
  }
  get duration(): number {
    return this.options.duration ?? (this.options.type === "spring" ? 0 : 0.3);
  }
  get state(): AnimationState {
    return this.#state;
  }

  #tick = (time: number) => {
    if (this.#state !== "running") return;
    const delta =
      this.#lastTime === undefined
        ? 0
        : Math.max(0, (time - this.#lastTime) / 1000);
    this.#lastTime = time;
    this.#elapsed += delta * this.#speed;
    const delay =
      this.#iteration === 0
        ? (this.options.delay ?? 0)
        : (this.options.repeatDelay ?? 0);
    if (this.#elapsed < delay) {
      this.#frame = requestAnimationFrame(this.#tick);
      return;
    }
    const sample = this.#curve.sample(this.#elapsed - delay);
    const reverse =
      this.options.repeatType !== "loop" && this.#iteration % 2 === 1;
    const progress = reverse ? 1 - sample.value : sample.value;
    this.options.onUpdate?.(
      interpolateKeyframes(this.keyframes, progress, this.options.times),
    );
    if (!sample.done) {
      this.#frame = requestAnimationFrame(this.#tick);
      return;
    }
    const repeat = this.options.repeat ?? 0;
    if (repeat === Infinity || this.#iteration < repeat) {
      this.#iteration += 1;
      this.#elapsed = 0;
      this.#lastTime = time;
      this.options.onRepeat?.();
      this.#frame = requestAnimationFrame(this.#tick);
      return;
    }
    this.#finish();
  };

  #finish(): void {
    if (this.#state === "finished") return;
    const reverse =
      this.options.repeatType !== "loop" && this.#iteration % 2 === 1;
    const value = reverse
      ? this.keyframes[0]
      : this.keyframes[this.keyframes.length - 1];
    this.options.onUpdate?.(value);
    this.#state = "finished";
    this.#curve.drop();
    this.options.onComplete?.();
    this.#resolve();
  }
  play(): void {
    if (this.#state === "running" || this.#state === "finished") return;
    this.#state = "running";
    this.#lastTime = performance.now();
    this.options.onPlay?.();
    this.#frame = requestAnimationFrame(this.#tick);
  }
  pause(): void {
    if (this.#state !== "running") return;
    if (this.#frame !== undefined) cancelAnimationFrame(this.#frame);
    this.#frame = undefined;
    this.#lastTime = undefined;
    this.#state = "paused";
  }
  stop(): void {
    if (this.#state === "idle" || this.#state === "finished") return;
    if (this.#frame !== undefined) cancelAnimationFrame(this.#frame);
    this.#frame = undefined;
    this.#state = "idle";
    this.#curve.drop();
    this.options.onStop?.();
    this.#resolve();
  }
  cancel(): void {
    this.stop();
  }
  complete(): void {
    if (this.#frame !== undefined) cancelAnimationFrame(this.#frame);
    this.#frame = undefined;
    this.#finish();
  }
  then<TResult1 = void, TResult2 = never>(
    onfulfilled?: ((value: void) => TResult1 | PromiseLike<TResult1>) | null,
    onrejected?: ((reason: unknown) => TResult2 | PromiseLike<TResult2>) | null,
  ): PromiseLike<TResult1 | TResult2> {
    return this.finished.then(onfulfilled, onrejected);
  }
}

/** Animate between two supported values using a Rust-owned curve. */
export function animate<V extends AnimationValue>(
  from: V,
  to: V,
  options: AnimationOptions<V> = {},
): AnimationControls {
  return animateKeyframes([from, to], options);
}

/** Animate numeric or color keyframes using a Rust-owned curve. */
export function animateKeyframes<V extends AnimationValue>(
  keyframes: readonly [V, V, ...V[]],
  options: AnimationOptions<V> = {},
): AnimationControls {
  return new Controls(keyframes, options);
}

export interface ReactiveAnimation<T> {
  value: Accessor<T>;
  controls: AnimationControls;
}

type MaybeAccessor<T> = T | Accessor<T>;
const read = <T>(value: MaybeAccessor<T> | undefined, fallback: T): T =>
  typeof value === "function" ? (value as Accessor<T>)() : (value ?? fallback);

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
function positiveFinite(value: number, fallback: number): number {
  return Number.isFinite(value) && value > 0 ? value : fallback;
}
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
  reducedMotion?: MaybeAccessor<boolean>;
  reducedValue?: V;
}
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
      } else if (initialized && resumeAfterReduction) controls.play();
      initialized = true;
    },
  );
  onCleanup(() => controls.stop());
  return { value, controls };
}

export interface MotionInterpolationOptions<V extends AnimationValue> {
  clamp?: boolean;
  ease?: Easing;
}
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
  if (input.length === 0 || input.length !== output.length)
    throw new RangeError(
      "animation input and output ranges must have equal non-zero lengths",
    );
  if (!input.every(Number.isFinite))
    throw new RangeError(
      "animation input range must contain only finite numbers",
    );
  return () => {
    let value = source();
    if (options.clamp !== false)
      value = Math.max(input[0], Math.min(input[input.length - 1], value));
    let segment = input.length - 2;
    for (let index = 0; index < input.length - 1; index++) {
      if (value <= input[index + 1]) {
        segment = index;
        break;
      }
    }
    const progress =
      (value - input[segment]) / (input[segment + 1] - input[segment]);
    return interpolateValue(output[segment], output[segment + 1], progress);
  };
}

export interface TransitionOptions
  extends Omit<
    AnimationOptions<number>,
    "autoplay" | "duration" | "onUpdate" | "onComplete"
  > {
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
      [
        target(),
        read(options.reducedMotion, false),
        read(options.duration, 0.3),
        untrack(value),
      ] as const,
    ([next, reduced, duration, current]) => {
      if (Object.is(next, current)) return;
      if (reduced || options.type === false || duration === 0) {
        jump(next);
        return;
      }

      const run = ++generation;
      controls?.stop();
      setState("running");
      const {
        initial: _initial,
        duration: _duration,
        reducedMotion: _reducedMotion,
        onUpdate,
        onComplete,
        ...animationOptions
      } = options;
      controls = animate(current, next, {
        ...animationOptions,
        duration,
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
