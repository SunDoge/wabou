import { expect, test } from "bun:test";
import { useWindow } from "@wabou/core";
import { createRoot, flush } from "solid-js";
import { dispatchHostMessage } from "../../../core/src/glue/host-messages";
import { useReducedMotion } from "./config";

test("motion inherits the native GPUI reduced-motion preference", () => {
  const window = useWindow();
  const current = window.metrics();
  const dispose = createRoot((dispose) => {
    const reducedMotion = useReducedMotion();
    expect(reducedMotion()).toBe(current.reducedMotion);

    dispatchHostMessage(
      "wabou:window-metrics",
      JSON.stringify({ ...current, reducedMotion: !current.reducedMotion }),
    );
    flush();
    expect(reducedMotion()).toBe(!current.reducedMotion);
    return dispose;
  });

  dispatchHostMessage("wabou:window-metrics", JSON.stringify(current));
  flush();
  dispose();
});
