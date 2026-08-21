import { describe, expect, test } from "bun:test";
import { aspectRatioStyle } from "./aspect-ratio";

describe("AspectRatio", () => {
  test("defaults to a square and preserves caller styles", () => {
    expect(aspectRatioStyle(undefined, { width: 320 })).toEqual({
      width: 320,
      "aspect-ratio": 1,
    });
  });

  test("uses a native numeric ratio", () => {
    expect(aspectRatioStyle(16 / 9)["aspect-ratio"]).toBe(16 / 9);
  });

  test("rejects invalid layout constraints before reaching Rust", () => {
    expect(() => aspectRatioStyle(0)).toThrow(RangeError);
    expect(() => aspectRatioStyle(Number.NaN)).toThrow(RangeError);
  });
});
