import { describe, expect, test } from "bun:test";
import { createRoot, createSignal, flush } from "solid-js";
import {
  createLoop,
  createPulse,
  createRotation,
  createSweep,
  createTransition,
  normalizeSweepGeometry,
} from "./index";

describe("Solid animation primitives", () => {
  test("can start a transition independently from its current target", () =>
    createRoot((dispose) => {
      const transition = createTransition(() => 1, {
        initial: 0,
        duration: 1,
      });
      expect(transition.value()).toBe(0);
      dispose();
    }));

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

  test("sweeps fully outside a measured axis without changing layout", () =>
    createRoot((dispose) => {
      const [extent, setExtent] = createSignal(100);
      const sweep = createSweep({
        extent,
        itemRatio: 0.25,
        axis: "vertical",
        autoplay: false,
      });
      expect(sweep.offset()).toBe(-25);
      expect(sweep.transform()).toEqual([1, 0, 0, 1, 0, -25]);
      setExtent(200);
      flush();
      expect(sweep.transform()).toEqual([1, 0, 0, 1, 0, -50]);
      dispose();
    }));

  test("publishes a stable sweep value while motion is reduced", () =>
    createRoot((dispose) => {
      const [reduced, setReduced] = createSignal(true);
      const sweep = createSweep({
        extent: 100,
        reducedMotion: reduced,
        reducedValue: 0.5,
      });
      flush();
      expect(sweep.value()).toBe(0.5);
      expect(sweep.offset()).toBeCloseTo(30);
      expect(sweep.transform().slice(0, 4)).toEqual([1, 0, 0, 1]);
      expect(sweep.controls.state).toBe("paused");

      setReduced(false);
      flush();
      expect(sweep.controls.state).toBe("running");
      dispose();
    }));

  test("does not resume a manually paused loop after reduced motion", () =>
    createRoot((dispose) => {
      const [reduced, setReduced] = createSignal(false);
      const loop = createLoop({ reducedMotion: reduced });
      flush();
      loop.controls.pause();
      setReduced(true);
      flush();
      setReduced(false);
      flush();
      expect(loop.controls.state).toBe("paused");
      dispose();
    }));

  test("normalizes unsafe sweep geometry", () => {
    expect(normalizeSweepGeometry(Number.NaN, 0)).toEqual({
      extent: 0,
      itemRatio: 0.4,
    });
    expect(normalizeSweepGeometry(-20, 3)).toEqual({
      extent: 0,
      itemRatio: 1,
    });
  });

  test("starts a pulse at its authored lower opacity", () =>
    createRoot((dispose) => {
      const pulse = createPulse({ from: 0.35, to: 0.9, autoplay: false });
      expect(pulse.value()).toBe(0.35);
      dispose();
      expect(pulse.controls.state).toBe("idle");
    }));

  test("never starts a pulse while motion is initially reduced", () =>
    createRoot((dispose) => {
      const [reduced, setReduced] = createSignal(true);
      const pulse = createPulse({
        from: 0.35,
        to: 0.9,
        reducedMotion: reduced,
      });
      flush();
      expect(pulse.value()).toBe(0.9);
      expect(pulse.controls.state).toBe("paused");
      setReduced(false);
      flush();
      expect(pulse.controls.state).toBe("running");
      dispose();
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
