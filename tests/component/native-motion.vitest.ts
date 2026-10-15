import { createRoot, createSignal, flush } from "solid-js";
import { describe, expect, test } from "vitest";
import { useSpring, useTween } from "../../packages/ui/src/animation/native";

describe("native motion bindings", () => {
  test("publishes spring targets without producing per-frame values", () =>
    createRoot((dispose) => {
      const [x, setX] = createSignal(12, { ownedWrite: true });
      const spring = useSpring(() => ({
        transform: [1, 0, 0, 1, x(), 0] as const,
        opacity: 0.8,
      }));

      expect(spring()).toMatchObject({
        kind: "spring",
        targetTransform: [1, 0, 0, 1, 12, 0],
        initialTransform: [1, 0, 0, 1, 12, 0],
        targetOpacity: 0.8,
        initialOpacity: 0.8,
      });
      setX(40);
      flush();
      expect(spring()).toMatchObject({
        targetTransform: [1, 0, 0, 1, 40, 0],
      });
      expect(spring()).not.toHaveProperty("initialTransform");
      dispose();
    }));

  test("increments tween generations only when the target changes", () =>
    createRoot((dispose) => {
      const [opacity, setOpacity] = createSignal(0, { ownedWrite: true });
      const tween = useTween(() => ({ opacity: opacity() }), {
        duration: 0.3,
      });

      expect(tween()).toMatchObject({
        kind: "transition",
        generation: 1,
        duration: 0,
        fromOpacity: 0,
        toOpacity: 0,
      });
      expect(tween().generation).toBe(1);
      setOpacity(1);
      flush();
      expect(tween()).toMatchObject({
        generation: 2,
        duration: 0.3,
        toOpacity: 1,
      });
      expect(tween()).not.toHaveProperty("fromOpacity");
      dispose();
    }));
});
