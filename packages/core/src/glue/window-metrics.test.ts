import { expect, test } from "bun:test";
import { flush } from "solid-js";
import { dispatchHostMessage } from "./host-messages";
import { createWindowMatch, useWindow } from "./window-metrics";

test("window metrics expose one reactive logical coordinate space", () => {
  const window = useWindow();
  dispatchHostMessage(
    "wabou:window-metrics",
    JSON.stringify({
      windowId: { lo: 7, hi: 1 },
      logicalWidth: 800,
      logicalHeight: 600,
      physicalWidth: 1600,
      physicalHeight: 1200,
      scaleFactor: 2,
      maximized: true,
      focused: true,
      colorScheme: "dark",
    }),
  );
  flush();

  expect(window.width()).toBe(800);
  expect(window.id).toMatchObject({ lo: 7, hi: 1 });
  expect(window.height()).toBe(600);
  expect(window.scaleFactor()).toBe(2);
  expect(window.maximized()).toBe(true);
  expect(window.colorScheme()).toBe("dark");
  expect(window.metrics().physicalWidth).toBe(1600);
});

test("native window size queries are reactive and reject invalid ranges", () => {
  let width = 1000;
  const window = {
    ...useWindow(),
    width: () => width,
    height: () => 700,
  };
  const compact = createWindowMatch({ maxWidth: 1059 }, window);
  expect(compact()).toBe(true);
  width = 1200;
  expect(compact()).toBe(false);
  expect(() =>
    createWindowMatch({ minWidth: 900, maxWidth: 800 }, window),
  ).toThrow("minWidth cannot exceed maxWidth");
});
