import { describe, expect, test } from "bun:test";
import { emptyClass } from "./empty";
import {
  responsiveGridColumnCount,
  responsiveGridRemainderCount,
} from "./layout";

describe("ResponsiveGrid", () => {
  test("uses a safe initial column count before native measurement", () => {
    expect(
      responsiveGridColumnCount({
        width: 0,
        minColumnWidth: 200,
        maxColumns: 4,
      }),
    ).toBe(1);
    expect(
      responsiveGridColumnCount({
        width: 0,
        minColumnWidth: 200,
        maxColumns: 3,
        initialColumns: 4,
      }),
    ).toBe(3);
  });

  test("accounts for gaps and clamps to the supported maximum", () => {
    const columns = (width: number) =>
      responsiveGridColumnCount({
        width,
        minColumnWidth: 200,
        gap: 16,
        maxColumns: 4,
      });

    expect(columns(199)).toBe(1);
    expect(columns(416)).toBe(2);
    expect(columns(632)).toBe(3);
    expect(columns(848)).toBe(4);
    expect(columns(2_000)).toBe(4);
  });

  test("fills only the unused cells in the final row", () => {
    expect(responsiveGridRemainderCount(8, 3)).toBe(1);
    expect(responsiveGridRemainderCount(8, 2)).toBe(0);
    expect(responsiveGridRemainderCount(1, 4)).toBe(3);
    expect(responsiveGridRemainderCount(0, 4)).toBe(0);
  });

  test("can avoid a single orphaned cell in the final row", () => {
    const columns = (width: number, itemCount: number) =>
      responsiveGridColumnCount({
        width,
        minColumnWidth: 176,
        gap: 8,
        maxColumns: 3,
        itemCount,
        balanceLastRow: true,
      });

    expect(columns(432, 3)).toBe(1);
    expect(columns(560, 4)).toBe(2);
    expect(columns(720, 3)).toBe(3);
    expect(columns(432, 4)).toBe(2);
  });
});

describe("Empty", () => {
  test("plain variant embeds without adding another surface", () => {
    expect(emptyClass("plain", "flex-1")).toBe(
      "w-full min-w-0 p-8 items-center justify-center gap-6 text-center min-h-0 bg-transparent flex-1",
    );
    expect(emptyClass()).toContain("border-subtle");
    expect(emptyClass()).toContain("shadow-xs");
  });
});
