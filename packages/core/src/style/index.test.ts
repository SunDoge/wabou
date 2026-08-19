import { describe, expect, test } from "bun:test";
import {
  assertInlineStyleValue,
  classes,
  isTypedStyleValue,
  number,
  percent,
  px,
  rgba,
  rotate2d,
  shadow,
  type WabouStyle,
  type WabouUtility,
} from "./index.ts";

describe("typed style", () => {
  test("exposes a property-aware style type", () => {
    const valid = {
      width: px(10),
      color: rgba(0xffffffff),
    } satisfies WabouStyle;
    expect(valid.width.value).toBe(10);
    // @ts-expect-error colors cannot be used as dimensions.
    const invalidKind = { width: rgba(0xffffffff) } satisfies WabouStyle;
    // @ts-expect-error camelCase properties are not part of WabouStyle.
    const invalidName = { alignItems: "center" } satisfies WabouStyle;
    expect([invalidKind, invalidName]).toHaveLength(2);
  });

  test("constructs allocation-light tagged values", () => {
    expect(px(12.5)).toMatchObject({ kind: 1, value: 12.5 });
    expect(percent(0.5)).toMatchObject({ kind: 2, value: 0.5 });
    expect(rgba(0x112233ff)).toMatchObject({ kind: 5, value: 0x112233ff });
    expect(isTypedStyleValue(px(1))).toBe(true);
    expect(isTypedStyleValue("1px")).toBe(false);
    expect(() => px(Number.NaN)).toThrow("px must be finite");
    expect(() => number(Number.POSITIVE_INFINITY)).toThrow(
      "number must be finite",
    );
  });

  test("rejects unsupported properties and mismatched typed values", () => {
    expect(() => assertInlineStyleValue("alignItems", "center")).toThrow(
      "use align-items",
    );
    expect(() => assertInlineStyleValue("width", rgba(0xff0000ff))).toThrow(
      "invalid for width",
    );
    expect(() => assertInlineStyleValue("box-shadow", "none")).toThrow(
      "unsupported inline style property",
    );
    expect(() => assertInlineStyleValue("width", px(10))).not.toThrow();
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

  test("constructs center-pivoted runtime rotation without a duplicate offset", () => {
    const quarterTurn = rotate2d(Math.PI / 2);
    expect(quarterTurn[0]).toBeCloseTo(0);
    expect(quarterTurn[1]).toBeCloseTo(1);
    expect(quarterTurn[2]).toBeCloseTo(-1);
    expect(quarterTurn[3]).toBeCloseTo(0);
    expect(quarterTurn.slice(4)).toEqual([0, 0]);
  });
});
