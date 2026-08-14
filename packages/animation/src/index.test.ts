import { describe, expect, test } from "bun:test";
import { createRoot, createSignal, flush } from "solid-js";
import {
  createLoop,
  createPulse,
  createRotation,
  createTransition,
} from "./index";

describe("Solid animation primitives", () => {
  test("owns a loop value and playback controls", () =>
    createRoot((dispose) => {
      const loop = createLoop({ from: 2, to: 4, autoplay: false });
      expect(loop.value()).toBe(2);
      expect(loop.controls.duration).toBe(1);
      dispose();
      expect(loop.controls.state).toBe("idle");
    }));

  test("creates a center-pivoted rotation matrix", () =>
    createRoot((dispose) => {
      const rotation = createRotation({ autoplay: false });
      expect(rotation.angle()).toBe(0);
      expect(rotation.transform()).toEqual([1, 0, -0, 1, 0, 0]);
      dispose();
      expect(rotation.controls.state).toBe("idle");
    }));

  test("starts a pulse at its authored lower opacity", () =>
    createRoot((dispose) => {
      const pulse = createPulse({ from: 0.35, to: 0.9, autoplay: false });
      expect(pulse.value()).toBe(0.35);
      dispose();
      expect(pulse.controls.state).toBe("idle");
    }));

  test("transitions synchronously under reduced motion and can jump", () =>
    createRoot((dispose) => {
      const [target, setTarget] = createSignal(2);
      const completed: number[] = [];
      const transition = createTransition(target, {
        reducedMotion: true,
        onComplete: (value) => completed.push(value),
      });
      expect(transition.value()).toBe(2);
      setTarget(8);
      flush();
      expect(transition.value()).toBe(8);
      transition.jump(3);
      flush();
      expect(transition.value()).toBe(3);
      expect(completed).toEqual([8, 3]);
      dispose();
    }));
});
