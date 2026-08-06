import { describe, expect, test } from "bun:test";
import {
  classes,
  isTypedStyleValue,
  percent,
  px,
  rgba,
  shadow,
  type WabouUtility,
} from "./index.ts";

describe("typed style", () => {
  test("constructs allocation-light tagged values", () => {
    expect(px(12.5)).toMatchObject({ kind: 1, value: 12.5 });
    expect(percent(0.5)).toMatchObject({ kind: 2, value: 0.5 });
    expect(rgba(0x112233ff)).toMatchObject({ kind: 5, value: 0x112233ff });
    expect(isTypedStyleValue(px(1))).toBe(true);
    expect(isTypedStyleValue("1px")).toBe(false);
  });

  test("type-checks and joins generated utilities", () => {
    const utility = "bg-slate-900" satisfies WabouUtility;
    expect(classes("flex", "px-[13px]", utility)).toBe(
      "flex px-[13px] bg-slate-900",
    );
  });

  test("constructs Vello-native shadows in stdDev units", () => {
    expect(
      shadow({ stdDev: 6, spread: -2, offsetY: 8, color: 0x11223344 }),
    ).toEqual({
      offsetX: 0,
      offsetY: 8,
      spread: -2,
      stdDev: 6,
      color: 0x11223344,
    });
    expect(() => shadow({ stdDev: -1, color: 0 })).toThrow(
      "stdDev cannot be negative",
    );
    expect(() => shadow({ stdDev: 1, radius: -1, color: 0 })).toThrow(
      "radius cannot be negative",
    );
  });
});
