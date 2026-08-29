import { describe, expect, test } from "bun:test";
import {
  assertInlineStyleValue,
  classes,
  isTypedStyleValue,
  mergeClasses,
  number,
  percent,
  px,
  rgba,
  rotate2d,
  shadow,
  utilityConflictProperties,
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

  test("merges utility conflicts by their Style IR properties", () => {
    expect(utilityConflictProperties("text-[18px]")).toEqual([
      "font-size",
    ]);
    expect(utilityConflictProperties("border-[3px]")).toEqual([
      "border-top-width",
      "border-right-width",
      "border-bottom-width",
      "border-left-width",
    ]);
    expect(utilityConflictProperties("aspect-[16/9]")).toEqual([
      "aspect-ratio",
    ]);
    expect(mergeClasses("w-full min-w-0", "w-48")).toBe("min-w-0 w-48");
    expect(mergeClasses("p-2", "px-4")).toBe("p-2 px-4");
    expect(mergeClasses("px-4", "p-2")).toBe("p-2");
    expect(mergeClasses("overflow-hidden", "overflow-x-scroll")).toBe(
      "overflow-hidden overflow-x-scroll",
    );
    expect(mergeClasses("overflow-x-scroll", "overflow-hidden")).toBe(
      "overflow-hidden",
    );
    expect(mergeClasses("translate-x-2", "translate-y-4")).toBe(
      "translate-x-2 translate-y-4",
    );
    expect(mergeClasses("translate-x-2", "translate-x-4")).toBe(
      "translate-x-4",
    );
    expect(mergeClasses("bg-surface", "bg-canvas")).toBe("bg-canvas");
    expect(mergeClasses("text-secondary", "text-brand/80")).toBe(
      "text-brand/80",
    );
  });

  test("preserves conditional and unknown third-party classes", () => {
    expect(
      mergeClasses("flex lucide-image", false, undefined, "lucide-image block"),
    ).toBe("lucide-image lucide-image block");
    expect(mergeClasses("-w-4", "w-full")).toBe("-w-4 w-full");
    expect(mergeClasses("p-auto", "p-2")).toBe("p-auto p-2");
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
