import { expect, test } from "bun:test";
import { flush } from "solid-js";
import { dispatchHostMessage } from "./host-messages";
import { useWindow } from "./window-metrics";

test("window metrics expose one reactive logical coordinate space", () => {
  const window = useWindow();
  dispatchHostMessage(
    "wabou:window-metrics",
    JSON.stringify({
      windowId: 7,
      logicalWidth: 800,
      logicalHeight: 600,
      physicalWidth: 1600,
      physicalHeight: 1200,
      scaleFactor: 2,
      maximized: true,
      focused: true,
    }),
  );
  flush();

  expect(window.width()).toBe(800);
  expect(window.id).toBe(7);
  expect(window.height()).toBe(600);
  expect(window.scaleFactor()).toBe(2);
  expect(window.maximized()).toBe(true);
  expect(window.metrics().physicalWidth).toBe(1600);
});
