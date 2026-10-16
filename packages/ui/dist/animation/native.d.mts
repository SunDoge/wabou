import { WabouNativeSpring, WabouNativeTransition } from "@wabou/core";
import { Affine2D } from "@wabou/core/style";
import { Accessor } from "solid-js";
//#region src/animation/native.d.ts
type MaybeAccessor<T> = T | Accessor<T>;
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
export declare function useSpring(target: MaybeAccessor<MotionTarget>, options?: SpringOptions): Accessor<WabouNativeSpring>;
export interface TweenOptions {
  duration?: number;
  easing?: "linear" | "easeInOut" | "easeOut";
  /** Animate from this value on mount. `false` and omission both snap initially. */
  initial?: MotionTarget | false;
  reducedMotion?: MaybeAccessor<boolean>;
}
/** Publish an interruptible finite tween to the native renderer. */
export declare function useTween(target: MaybeAccessor<MotionTarget>, options?: TweenOptions): Accessor<WabouNativeTransition>;
//#endregion
export {};
//# sourceMappingURL=native.d.mts.map