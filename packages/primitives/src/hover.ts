import { createSignal } from "solid-js";

export interface HoverResult {
  hovered: () => boolean;
  bindings: {
    onPointerEnter: () => void;
    onPointerLeave: () => void;
  };
}

/** Reactive hover state and event bindings for a single target. */
export function createHover(): HoverResult {
  const [hovered, setHovered] = createSignal(false);
  return {
    hovered,
    bindings: {
      onPointerEnter: () => setHovered(true),
      onPointerLeave: () => setHovered(false),
    },
  };
}
