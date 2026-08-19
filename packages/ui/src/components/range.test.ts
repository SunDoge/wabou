import { describe, expect, test } from "bun:test";
import {
  decimalPlaces,
  finiteOr,
  normalizePercentage,
  normalizeRange,
} from "./range";

describe("range normalization", () => {
  test("rejects non-finite values before they reach layout or semantics", () => {
    expect(finiteOr(Number.NaN, 4)).toBe(4);
    expect(finiteOr(Number.POSITIVE_INFINITY, 4)).toBe(4);
    expect(normalizePercentage(Number.NaN)).toBe(0);
    expect(normalizePercentage(Number.NEGATIVE_INFINITY)).toBe(0);
    expect(normalizePercentage(140)).toBe(100);
    expect(normalizePercentage(-20)).toBe(0);
  });

  test("produces an ordered range with a positive finite step", () => {
    expect(normalizeRange(Number.NaN, Number.POSITIVE_INFINITY, 0)).toEqual({
      min: 0,
      max: 100,
      step: 1,
    });
    expect(normalizeRange(20, 10, -2)).toEqual({
      min: 20,
      max: 20,
      step: 1,
    });
  });

  test("counts decimal places for ordinary and exponent notation", () => {
    expect(decimalPlaces(1)).toBe(0);
    expect(decimalPlaces(0.125)).toBe(3);
    expect(decimalPlaces(1e-7)).toBe(7);
    expect(decimalPlaces(1.25e-3)).toBe(5);
    expect(decimalPlaces(1.25e3)).toBe(0);
  });
});
