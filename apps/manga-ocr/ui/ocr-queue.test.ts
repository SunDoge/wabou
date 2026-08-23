import { expect, test } from "vitest";
import { ocrStateLabel, prioritizeAllPageIndices } from "./ocr-queue";

test("prioritizes every page around the visible page", () => {
  expect(prioritizeAllPageIndices(8, 3)).toEqual([3, 4, 2, 5, 1, 6, 0, 7]);
  expect(prioritizeAllPageIndices(3, 0)).toEqual([0, 1, 2]);
  expect(prioritizeAllPageIndices(0, 0)).toEqual([]);
});

test("exposes stable page status labels", () => {
  expect(ocrStateLabel("recognizing")).toBe("Recognizing");
  expect(ocrStateLabel(undefined)).toBe("Not scanned");
});
