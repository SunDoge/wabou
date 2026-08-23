import { expect, test } from "vitest";
import { ocrStateLabel, prioritizePageIndices } from "./ocr-queue";

test("prioritizes a bounded neighborhood around the visible page", () => {
  expect(prioritizePageIndices(8, 3)).toEqual([3, 4, 2, 5, 1]);
  expect(prioritizePageIndices(3, 0)).toEqual([0, 1, 2]);
  expect(prioritizePageIndices(0, 0)).toEqual([]);
});

test("exposes stable page status labels", () => {
  expect(ocrStateLabel("recognizing")).toBe("Recognizing");
  expect(ocrStateLabel(undefined)).toBe("Not scanned");
});
