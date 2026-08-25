import { expect, test } from "bun:test";
import { AnimationFrameQueue, tickAnimationFrame } from "./animation-frame";

test("requestAnimationFrame commits its transaction before writer delivery", () => {
  let pending = "before";
  let applied = "before";
  let delivered: Uint8Array | undefined;
  let serialized = "";
  const queue = new AnimationFrameQueue();
  queue.request(() => {
    pending = "after";
  });
  tickAnimationFrame(
    16,
    (bytes) => {
      delivered = bytes.slice();
    },
    () => {
      serialized = applied;
      return new TextEncoder().encode(applied);
    },
    (callback) => {
      const result = callback();
      applied = pending;
      return result;
    },
    queue,
  );

  expect(serialized).toBe("after");
  expect(delivered).toBeDefined();
  expect(new TextDecoder().decode(delivered)).toContain("after");
});
