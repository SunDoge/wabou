import { onCleanup } from "solid-js";

export type AnimationFrameCallback = (timestamp: number) => unknown;

/**
 * Drive explicit paint state from the native host's animation clock.
 * Return `false` to stop scheduling frames before the owner is disposed.
 */
export function createAnimationFrame(
  callback: AnimationFrameCallback,
): () => void {
  let frame = 0;
  let active = true;

  const tick = (timestamp: number) => {
    if (!active) return;
    if (callback(timestamp) === false) {
      active = false;
      return;
    }
    frame = requestAnimationFrame(tick);
  };

  frame = requestAnimationFrame(tick);

  const stop = () => {
    if (!active) return;
    active = false;
    cancelAnimationFrame(frame);
  };
  onCleanup(stop);
  return stop;
}
