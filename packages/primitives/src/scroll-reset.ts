import { type Accessor, createEffect, on } from "solid-js";

export interface ScrollResetTarget {
  scrollTo(options: { left?: number; top?: number }): void;
}

export interface ScrollResetOptions<K> {
  /** The explicitly owned viewport; no global/window fallback is used. */
  target: Accessor<ScrollResetTarget | undefined>;
  /** Reset whenever this navigation or content identity changes. */
  key: Accessor<K>;
  left?: number;
  top?: number;
}

/** Reset one explicitly selected native viewport after its key changes. */
export function createScrollReset<K>(
  options: ScrollResetOptions<K>,
): () => void {
  const reset = () => {
    options.target()?.scrollTo({
      left: options.left ?? 0,
      top: options.top ?? 0,
    });
  };
  createEffect(on(options.key, reset, { defer: true }));
  return reset;
}
