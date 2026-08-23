import { expect, test } from "vitest";
import {
  translatedRegions,
  updateRegionGeometry,
} from "../../apps/manga-ocr/ui/reader-state";
import type { OcrRegion } from "../../apps/manga-ocr/ui/api";

const region: OcrRegion = {
  id: "line",
  x: 10,
  y: 20,
  width: 30,
  height: 40,
  text: "原文",
  confidence: 0.9,
};

test("numeric bbox edits remain inside original image coordinates", () => {
  expect(updateRegionGeometry(region, "x", 95, { width: 100, height: 80 })).toMatchObject({
    x: 95,
    width: 5,
  });
  expect(updateRegionGeometry(region, "height", 100, { width: 100, height: 80 })).toMatchObject({
    y: 20,
    height: 60,
  });
});

test("translation overlays include only non-empty translations", () => {
  expect(
    translatedRegions([
      region,
      { ...region, id: "empty", translation: "  " },
      { ...region, id: "translated", translation: "译文" },
    ]).map((item) => item.id),
  ).toEqual(["translated"]);
});
