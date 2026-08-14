import { expect, test } from "bun:test";

import {
  titleBarDragRegionLayoutStyle,
  titleBarClass,
  titleBarLayoutStyle,
} from "./title-bar";

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
