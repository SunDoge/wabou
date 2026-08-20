import { expect, test } from "bun:test";
import { createEffect, createRoot, flush } from "solid-js";
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

test("native metrics can publish while an unrelated Solid owner is current", () => {
  const window = useWindow();
  let observedWidth = 0;
  let widthRuns = 0;
  let compact = false;
  const dispose = createRoot((dispose) => {
    const matchesCompact = createWindowMatch({ maxWidth: 1050 }, window);
    createEffect(
      () => window.width(),
      (width) => {
        widthRuns++;
        observedWidth = width;
      },
    );
    createEffect(matchesCompact, (matches) => {
      compact = matches;
    });
    flush();
    return dispose;
  });

  dispatchHostMessage(
    "wabou:window-metrics",
    JSON.stringify({
      ...window.metrics(),
      logicalWidth: 936,
      physicalWidth: 936,
    }),
  );
  flush();

  expect(observedWidth).toBe(936);
  expect(compact).toBe(true);
  const runsAfterChange = widthRuns;
  dispatchHostMessage("wabou:window-metrics", JSON.stringify(window.metrics()));
  flush();
  expect(widthRuns).toBe(runsAfterChange);
  dispose();
});

test("invalid native metrics do not replace the last valid snapshot", () => {
  const window = useWindow();
  const previous = window.metrics();
  const originalError = console.error;
  console.error = () => undefined;
  try {
    dispatchHostMessage(
      "wabou:window-metrics",
      JSON.stringify({ ...previous, scaleFactor: "2" }),
    );
  } finally {
    console.error = originalError;
  }
  flush();
  expect(window.metrics()).toBe(previous);
});
