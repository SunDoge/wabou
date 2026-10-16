import { Easing, ReactiveTransition } from "../animation.mjs";
import { PresencePhase } from "./presence.mjs";
import { Accessor } from "solid-js";
//#region src/primitives/transition-presence.d.ts
export interface TransitionPresenceOptions {
  /**
   * Reactive visual readiness. Mounting still follows `open`, but entry waits
   * for this value. This is useful for positioned overlays that must complete
   * native measurement before becoming visible.
   */
  ready?: Accessor<boolean>;
  /** Start visual progress independently from logical presence. */
  initialProgress?: number;
  duration?: number | Accessor<number>;
  ease?: Easing;
  reducedMotion?: boolean | Accessor<boolean>;
}
export interface TransitionPresence {
  phase: Accessor<PresencePhase>;
  mounted: Accessor<boolean>;
  /** Normalized visual progress: 0 when hidden and 1 when fully present. */
  progress: Accessor<number>;
  transition: ReactiveTransition;
}
/**
 * Couples logical presence to an interruptible visual transition.
 *
 * Closing disables the logical surface immediately while keeping its visual
 * subtree mounted until progress reaches zero. Reopening during exit simply
 * retargets the current transition instead of remounting the subtree.
 */
export declare function createTransitionPresence(open: Accessor<boolean>, options?: TransitionPresenceOptions): TransitionPresence;
//#endregion
//# sourceMappingURL=transition-presence.d.mts.map