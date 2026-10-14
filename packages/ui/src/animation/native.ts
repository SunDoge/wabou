import type { WabouNativeSpring, WabouNativeTransition } from "@wabou/core";
import type { Affine2D } from "@wabou/core/style";
import { type Accessor, createMemo } from "solid-js";

const identity: Affine2D = [1, 0, 0, 1, 0, 0];

type MaybeAccessor<T> = T | Accessor<T>;

function read<T>(value: MaybeAccessor<T>): T {
  return typeof value === "function" ? (value as Accessor<T>)() : value;
}

/** Renderer properties which can currently run without per-frame JS work. */
export interface MotionTarget {
  transform?: Affine2D;
  opacity?: number;
}

export interface SpringOptions {
  /** Approximate response period in seconds. */
  response?: number;
  /** Damping ratio: below 1 bounces, 1 is critical, above 1 is overdamped. */
  damping?: number;
  mass?: number;
  stiffness?: number;
  restDelta?: number;
  /** Animate from this value on mount. `false` and omission both snap initially. */
  initial?: MotionTarget | false;
  reducedMotion?: MaybeAccessor<boolean>;
}

/**
 * Publish spring targets to the native renderer.
 *
 * The accessor changes only when its authored target changes. Rust retains the
 * sampled value and velocity between publications and across ordinary JS HMR.
 */
export function useSpring(
  target: MaybeAccessor<MotionTarget>,
  options: SpringOptions = {},
): Accessor<WabouNativeSpring> {
  let first = true;
  return createMemo(() => {
    const next = read(target);
    const reducedMotion = options.reducedMotion
      ? read(options.reducedMotion)
      : false;
    const initial = first && options.initial ? options.initial : undefined;
    const snapInitially = first && !initial;
    first = false;
    const initialTransform = snapInitially
      ? (next.transform ?? identity)
      : initial?.transform;
    const initialOpacity = snapInitially ? next.opacity : initial?.opacity;
    return {
      kind: "spring",
      response: options.response ?? 0.5,
      damping: options.damping ?? 0.82,
      epsilon: options.restDelta ?? 0.001,
      targetTransform: next.transform ?? identity,
      reducedMotion,
      ...(options.mass === undefined ? {} : { mass: options.mass }),
      ...(options.stiffness === undefined
        ? {}
        : { stiffness: options.stiffness }),
      ...(next.opacity === undefined ? {} : { targetOpacity: next.opacity }),
      ...(initialTransform === undefined ? {} : { initialTransform }),
      ...(initialOpacity === undefined ? {} : { initialOpacity }),
    };
  });
}

export interface TweenOptions {
  duration?: number;
  easing?: "linear" | "easeInOut" | "easeOut";
  /** Animate from this value on mount. `false` and omission both snap initially. */
  initial?: MotionTarget | false;
  reducedMotion?: MaybeAccessor<boolean>;
}

/** Publish an interruptible finite tween to the native renderer. */
export function useTween(
  target: MaybeAccessor<MotionTarget>,
  options: TweenOptions = {},
): Accessor<WabouNativeTransition> {
  let generation = 0;
  let first = true;
  return createMemo(() => {
    const next = read(target);
    const initial = first && options.initial ? options.initial : undefined;
    const snapInitially = first && !initial;
    first = false;
    generation += 1;
    const reducedMotion = options.reducedMotion
      ? read(options.reducedMotion)
      : false;
    const fromTransform = snapInitially
      ? (next.transform ?? identity)
      : initial?.transform;
    const fromOpacity = snapInitially ? next.opacity : initial?.opacity;
    return {
      kind: "transition",
      generation,
      duration: reducedMotion || snapInitially ? 0 : (options.duration ?? 0.2),
      easing: options.easing ?? "easeOut",
      ...(fromTransform === undefined ? {} : { fromTransform }),
      ...(next.transform === undefined ? {} : { toTransform: next.transform }),
      ...(fromOpacity === undefined ? {} : { fromOpacity }),
      ...(next.opacity === undefined ? {} : { toOpacity: next.opacity }),
    };
  });
}
