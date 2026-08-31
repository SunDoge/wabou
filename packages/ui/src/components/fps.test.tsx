import { expect, test } from "bun:test";
import { mount } from "@wabou/core/renderer";
import { createComponent } from "solid-js";
// Exercise the transformed public artifact, including its conditional owner cleanup.
import { Fps } from "../../dist/index.mjs";

test("controlled FPS badges do not keep the native frame clock active", () => {
  const originalRequest = globalThis.requestAnimationFrame;
  const originalCancel = globalThis.cancelAnimationFrame;
  const originalSetInterval = globalThis.setInterval;
  const originalClearInterval = globalThis.clearInterval;
  let requested = 0;
  Object.assign(globalThis, {
    requestAnimationFrame: () => ++requested,
    cancelAnimationFrame: () => {},
    setInterval: () => 1,
    clearInterval: () => {},
  });

  try {
    const dispose = mount(() => createComponent(Fps, { value: 60 }));
    expect(requested).toBe(0);
    dispose();

    const disposeLive = mount(() => createComponent(Fps, { live: true }));
    expect(requested).toBe(1);
    disposeLive();
  } finally {
    Object.assign(globalThis, {
      requestAnimationFrame: originalRequest,
      cancelAnimationFrame: originalCancel,
      setInterval: originalSetInterval,
      clearInterval: originalClearInterval,
    });
  }
});
