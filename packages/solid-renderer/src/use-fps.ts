import { createSignal, onCleanup } from "solid-js";

/**
 * Track frames-per-second. A self-perpetuating rAF loop counts frames; a
 * 1s interval samples the count and resets it. The rAF loop keeps the host
 * redrawing (it drives `has_anim`), so this measures the active vsync rate
 * while mounted — ~60 on a 60Hz display, ~120 on 120Hz. When nothing animates,
 * the host stops redrawing and the count drops.
 *
 * ```tsx
 * const fps = createFps();
 * <div>{fps()} fps</div>
 * ```
 */
export function createFps(): () => number {
  const [fps, setFps] = createSignal(0);
  let frames = 0;
  let last = performance.now();
  let rafId = 0;
  const loop = () => {
    frames++;
    rafId = requestAnimationFrame(loop);
  };
  rafId = requestAnimationFrame(loop);
  const intervalId = setInterval(() => {
    const now = performance.now();
    const dt = now - last;
    last = now;
    if (dt > 0) setFps(Math.round((frames * 1000) / dt));
    frames = 0;
  }, 1000);
  onCleanup(() => {
    cancelAnimationFrame(rafId);
    clearInterval(intervalId);
  });
  return fps;
}

/** @deprecated Use createFps; this primitive creates owned timers rather than consuming context. */
export const useFps = createFps;
