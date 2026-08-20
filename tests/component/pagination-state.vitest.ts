import { clampPage, createPaginationRange } from "@wabou/ui";
import { expect, test } from "vitest";

test("pagination ranges expose boundaries, siblings, and directional gaps", () => {
  expect(createPaginationRange({ count: 20, page: 10 })).toEqual([
    1,
    "ellipsis-start",
    9,
    10,
    11,
    "ellipsis-end",
    20,
  ]);
});

test("pagination ranges replace one-page gaps with actionable pages", () => {
  expect(createPaginationRange({ count: 10, page: 4 })).toEqual([
    1,
    2,
    3,
    4,
    5,
    "ellipsis-end",
    10,
  ]);
  expect(createPaginationRange({ count: 10, page: 7 })).toEqual([
    1,
    "ellipsis-start",
    6,
    7,
    8,
    9,
    10,
  ]);
});

test("pagination normalization clamps invalid inputs deterministically", () => {
  expect(clampPage(-20, 4)).toBe(1);
  expect(clampPage(20, 4)).toBe(4);
  expect(
    createPaginationRange({ count: Number.NaN, page: Number.NaN }),
  ).toEqual([1]);
});
