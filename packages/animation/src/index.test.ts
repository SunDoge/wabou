import { describe, expect, test } from "bun:test";
import { createRoot } from "solid-js";
import { createLoop, createPulse, createRotation } from "./index";

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
});
