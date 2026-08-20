import { expect, test } from "bun:test";

import {
  titleBarClass,
  titleBarDragRegionLayoutStyle,
  titleBarLayoutStyle,
  windowFrameBackdropClassList,
  windowFrameClientClassList,
  windowFrameShadows,
} from "./title-bar";

test("window frame reserves a transparent decoration inset only while restored", () => {
  expect(windowFrameBackdropClassList(false)).toEqual({ "p-3": true });
  expect(windowFrameBackdropClassList(true)).toEqual({ "p-3": false });
  expect(windowFrameBackdropClassList(false, false)).toEqual({ "p-3": false });

  expect(windowFrameClientClassList(false)).toMatchObject({
    "rounded-xl": true,
    border: true,
    "border-subtle": true,
    "overflow-hidden": true,
  });
  expect(windowFrameClientClassList(true)).toMatchObject({
    "rounded-xl": false,
    border: false,
    "border-subtle": false,
    "overflow-hidden": false,
  });
  expect(
    windowFrameClientClassList(false, false, { "bg-canvas": true }),
  ).toEqual({
    "bg-canvas": true,
    "rounded-xl": false,
    border: false,
    "border-subtle": false,
    "overflow-hidden": false,
  });
});

test("window frame uses bounded ambient and contact shadows", () => {
  const light = windowFrameShadows("light");
  const dark = windowFrameShadows("dark");
  expect(light).toHaveLength(2);
  expect(light.map((layer) => layer.stdDev)).toEqual([3, 1.5]);
  expect(light.every((layer) => layer.radius === 12)).toBe(true);
  expect(dark.map((layer) => layer.color)).not.toEqual(
    light.map((layer) => layer.color),
  );
});

test("custom title bar owns structural row layout without generated utilities", () => {
  expect(titleBarClass).toBe("border-b border-subtle");
  expect(titleBarLayoutStyle).toMatchObject({
    display: "flex",
    "flex-direction": "row",
    "align-items": "center",
    "flex-shrink": 0,
  });
  expect(titleBarDragRegionLayoutStyle).toMatchObject({
    "flex-grow": 1,
    "flex-shrink": 1,
    "flex-basis": "0%",
  });
});
