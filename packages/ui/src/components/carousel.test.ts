import { describe, expect, test } from "bun:test";
import { normalizeCarouselIndex } from "./carousel";

describe("Carousel", () => {
  test("clamps finite carousels", () => {
    expect(normalizeCarouselIndex(-1, 3, false)).toBe(0);
    expect(normalizeCarouselIndex(8, 3, false)).toBe(2);
  });

  test("wraps looped carousels in both directions", () => {
    expect(normalizeCarouselIndex(-1, 3, true)).toBe(2);
    expect(normalizeCarouselIndex(3, 3, true)).toBe(0);
  });

  test("keeps empty carousels at zero", () => {
    expect(normalizeCarouselIndex(4, 0, true)).toBe(0);
  });
});
