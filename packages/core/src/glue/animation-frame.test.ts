import { expect, test } from "bun:test";
import { createElement, setProp, writer } from "../renderer";
import { createRenderEffect, createRoot, createSignal, flush } from "solid-js";
import {
  requestAnimationFrameImpl,
  tickAnimationFrame,
} from "./animation-frame";

test("requestAnimationFrame commits Solid effects before writer delivery", () => {
  const node = createElement("text");
  let setValue!: (value: string) => string;
  const dispose = createRoot((dispose) => {
    const [value, write] = createSignal("before");
    setValue = write;
    createRenderEffect(value, (next) => {
      setProp(node, "textContent", next, undefined);
    });
    return dispose;
  });
  flush();
  writer.flush();

  let delivered: Uint8Array | undefined;
  try {
    requestAnimationFrameImpl(() => setValue("after"));
    tickAnimationFrame(16, (bytes) => {
      delivered = bytes.slice();
    });
  } finally {
    dispose();
  }

  expect(delivered).toBeDefined();
  expect(new TextDecoder().decode(delivered)).toContain("after");
});
