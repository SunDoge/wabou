import { expect, test } from "bun:test";
import { createRoot } from "solid-js";
import { createFps } from "./use-fps";

test("createFps owns and cleans up its animation frame and timer", () => {
  const scheduler = {
    requestAnimationFrame: globalThis.requestAnimationFrame,
    cancelAnimationFrame: globalThis.cancelAnimationFrame,
    setInterval: globalThis.setInterval,
    clearInterval: globalThis.clearInterval,
  };
  let nextAnimationFrame = 0;
  let animationFrameCallback: FrameRequestCallback | undefined;
  let intervalCallback: (() => void) | undefined;
  const cancelledAnimationFrames: number[] = [];
  const clearedIntervals: number[] = [];

  Object.assign(globalThis, {
    requestAnimationFrame: (callback: FrameRequestCallback) => {
      animationFrameCallback = callback;
      return ++nextAnimationFrame;
    },
    cancelAnimationFrame: (id: number) => cancelledAnimationFrames.push(id),
    setInterval: (callback: () => void) => {
      intervalCallback = callback;
      return 101;
    },
    clearInterval: (id: number) => clearedIntervals.push(id),
  });

  try {
    createRoot((dispose) => {
      const fps = createFps();
      expect(fps()).toBe(0);
      expect(nextAnimationFrame).toBe(1);
      expect(intervalCallback).toBeDefined();

      animationFrameCallback?.(0);
      expect(nextAnimationFrame).toBe(2);
      dispose();
    });

    expect(cancelledAnimationFrames).toEqual([2]);
    expect(clearedIntervals).toEqual([101]);
  } finally {
    Object.assign(globalThis, scheduler);
  }
});
