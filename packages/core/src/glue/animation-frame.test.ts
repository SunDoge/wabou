import { expect, test } from "bun:test";
import { createRenderEffect, createRoot, createSignal, flush } from "solid-js";
import {
  requestAnimationFrameImpl,
  tickAnimationFrame,
} from "./animation-frame";

test("requestAnimationFrame commits Solid effects before writer delivery", () => {
  let setValue!: (value: string) => string;
  let applied = "";
  const dispose = createRoot((dispose) => {
    const [value, write] = createSignal("before");
    setValue = write;
    createRenderEffect(value, (next) => {
      applied = next;
    });
    return dispose;
  });
  flush();
  expect(applied).toBe("before");

  let delivered: Uint8Array | undefined;
  let serialized = "";
  try {
    requestAnimationFrameImpl(() => setValue("after"));
    tickAnimationFrame(
      16,
      (bytes) => {
        delivered = bytes.slice();
      },
      () => {
        serialized = applied;
        return new TextEncoder().encode(applied);
      },
    );
  } finally {
    dispose();
  }

  expect(serialized).toBe("after");
  expect(delivered).toBeDefined();
  expect(new TextDecoder().decode(delivered)).toContain("after");
});
