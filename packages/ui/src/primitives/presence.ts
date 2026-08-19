import { createEffect, createSignal, type Accessor } from "solid-js";

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
    open() ? "present" : "unmounted",
  );

  createEffect(open, (isOpen) => {
    if (isOpen) {
      if (phase() === "unmounted" || phase() === "exiting") {
        setPhase("entering");
      }
    } else if (phase() === "present" || phase() === "entering") {
      setPhase("exiting");
    }
  });

  return {
    phase,
    mounted: () => phase() !== "unmounted",
    finishEnter() {
      if (open() && phase() === "entering") setPhase("present");
    },
    finishExit() {
      if (!open() && phase() === "exiting") setPhase("unmounted");
    },
  };
}
