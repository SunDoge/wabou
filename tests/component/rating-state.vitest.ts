import { clampRatingValue, normalizeRatingMax, ratingLabel } from "@wabou/ui";
import { expect, test } from "vitest";

test("rating state normalizes host-untrusted numeric inputs", () => {
  expect(normalizeRatingMax(undefined)).toBe(5);
  expect(normalizeRatingMax(Number.NaN)).toBe(5);
  expect(normalizeRatingMax(100)).toBe(20);
  expect(clampRatingValue(-2, 5)).toBe(0);
  expect(clampRatingValue(2.6, 5)).toBe(3);
  expect(clampRatingValue(12, 5)).toBe(5);
});

test("rating labels keep singular and plural semantics explicit", () => {
  expect(ratingLabel(1)).toBe("1 star");
  expect(ratingLabel(2)).toBe("2 stars");
});
