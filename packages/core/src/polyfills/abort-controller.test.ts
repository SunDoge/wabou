import { expect, test } from "bun:test";
import { installAbortControllerPolyfill } from "./abort-controller";

test("installs interoperable cancellation globals", () => {
  const runtime = globalThis as typeof globalThis;
  const previousController = runtime.AbortController;
  const previousSignal = runtime.AbortSignal;

  try {
    Reflect.deleteProperty(runtime, "AbortController");
    Reflect.deleteProperty(runtime, "AbortSignal");
    installAbortControllerPolyfill();

    const controller = new AbortController();
    let events = 0;
    controller.signal.addEventListener("abort", () => {
      events += 1;
    });
    controller.abort();
    controller.abort();

    expect(controller.signal).toBeInstanceOf(AbortSignal);
    expect(controller.signal.aborted).toBe(true);
    expect(events).toBe(1);
  } finally {
    runtime.AbortController = previousController;
    runtime.AbortSignal = previousSignal;
  }
});
