import { expect, test } from "bun:test";
import { createElement, setProp, writer } from "@wabou/solid-renderer";
import { createRenderEffect, createRoot, createSignal, flush } from "solid-js";
import "./animation-frame";

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
  const originalFlush = globalThis.__wabou_flush;
  globalThis.__wabou_flush = (bytes) => {
    delivered = bytes.slice();
  };
  try {
    requestAnimationFrame(() => setValue("after"));
    (globalThis as unknown as { __wabou_tick(time: number): boolean }).__wabou_tick(16);
  } finally {
    globalThis.__wabou_flush = originalFlush;
    dispose();
  }

  expect(delivered).toBeDefined();
  expect(new TextDecoder().decode(delivered)).toContain("after");
});
