import { expect, test } from "bun:test";
import { useWindow } from "@wabou/core";
import { dispatchHostMessageForTest } from "@wabou/core/testing";
import { createRoot, flush } from "solid-js";
import { useReducedMotion } from "./config";

test("motion inherits the native reduced-motion preference", () => {
  const window = useWindow();
  const current = window.metrics();
  const dispose = createRoot((dispose) => {
    const reducedMotion = useReducedMotion();
    expect(reducedMotion()).toBe(current.reducedMotion);

    dispatchHostMessageForTest(
      "wabou:window-metrics",
      JSON.stringify({ ...current, reducedMotion: !current.reducedMotion }),
    );
    flush();
    expect(reducedMotion()).toBe(!current.reducedMotion);
    return dispose;
  });

  dispatchHostMessageForTest("wabou:window-metrics", JSON.stringify(current));
  flush();
  dispose();
});
