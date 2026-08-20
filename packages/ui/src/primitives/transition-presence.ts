import { type Accessor, createEffect, untrack } from "solid-js";
import {
  createTransition,
  type Easing,
  type ReactiveTransition,
} from "../animation";
import { createPresence, type Presence, type PresencePhase } from "./presence";

export interface TransitionPresenceOptions {
  /**
   * Reactive visual readiness. Mounting still follows `open`, but entry waits
   * for this value. This is useful for positioned overlays that must complete
   * native measurement before becoming visible.
   */
  ready?: Accessor<boolean>;
  /** Start visual progress independently from logical presence. */
  initialProgress?: number;
  duration?: number;
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
export function createTransitionPresence(
  open: Accessor<boolean>,
  options: TransitionPresenceOptions = {},
): TransitionPresence {
  const presence: Presence = createPresence(open);
  const visuallyPresent = () => open() && (options.ready?.() ?? true);
  const transition = createTransition(() => (visuallyPresent() ? 1 : 0), {
    initial: options.initialProgress,
    duration: options.duration ?? 0.16,
    ease: options.ease ?? "easeOut",
    reducedMotion: options.reducedMotion,
    onComplete(value) {
      if (value === 1 && untrack(open)) presence.finishEnter();
      else if (value === 0 && !untrack(open)) presence.finishExit();
    },
  });
  createEffect(
    () =>
      [
        open(),
        visuallyPresent(),
        transition.value(),
        presence.phase(),
      ] as const,
    ([isOpen, isVisible, progress, phase]) => {
      // createTransition intentionally skips completion when target and value
      // already match. Presence still has to settle in that case, such as
      // when an overlay closes before native positioning has completed.
      if (isOpen && isVisible && progress === 1 && phase === "entering") {
        presence.finishEnter();
      } else if (!isOpen && progress === 0 && phase === "exiting") {
        presence.finishExit();
      }
    },
  );

  return {
    phase: presence.phase,
    mounted: presence.mounted,
    progress: transition.value,
    transition,
  };
}
