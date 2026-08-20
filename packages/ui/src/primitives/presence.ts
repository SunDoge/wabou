import { type Accessor, createEffect, createSignal, untrack } from "solid-js";

export type PresencePhase = "unmounted" | "entering" | "present" | "exiting";

export interface Presence {
  phase: Accessor<PresencePhase>;
  mounted: Accessor<boolean>;
  finishEnter(): void;
  finishExit(): void;
}

/** Explicit mount lifecycle for content whose exit must finish before removal. */
export function createPresence(open: Accessor<boolean>): Presence {
  const [phase, setPhase] = createSignal<PresencePhase>(
    untrack(open) ? "present" : "unmounted",
  );

  createEffect(open, (isOpen) => {
    setPhase((current) => {
      if (isOpen && (current === "unmounted" || current === "exiting")) {
        return "entering";
      }
      if (!isOpen && (current === "present" || current === "entering")) {
        return "exiting";
      }
      return current;
    });
  });

  return {
    phase,
    mounted: () => phase() !== "unmounted",
    finishEnter() {
      if (untrack(open)) {
        setPhase((current) => (current === "entering" ? "present" : current));
      }
    },
    finishExit() {
      if (!untrack(open)) {
        setPhase((current) => (current === "exiting" ? "unmounted" : current));
      }
    },
  };
}
