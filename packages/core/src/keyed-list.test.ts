import { expect, test } from "bun:test";
import { reconcileKeyedList } from "./keyed-list";

interface Item {
  id: string;
  value: number;
}

const keyOf = (item: Item) => item.id;

test("keyed list patches remove, upsert, and authoritatively reorder", () => {
  expect(
    reconcileKeyedList(
      [
        { id: "a", value: 1 },
        { id: "b", value: 2 },
      ],
      {
        removed: ["a"],
        upserted: [
          { id: "b", value: 20 },
          { id: "c", value: 3 },
        ],
        order: ["c", "b"],
      },
      keyOf,
    ),
  ).toEqual([
    { id: "c", value: 3 },
    { id: "b", value: 20 },
  ]);
});

test("keyed list patches reject incomplete, duplicate, and unknown orders", () => {
  const current = [
    { id: "a", value: 1 },
    { id: "b", value: 2 },
  ];
  expect(
    reconcileKeyedList(
      current,
      { removed: [], upserted: [], order: ["a"] },
      keyOf,
    ),
  ).toBeUndefined();
  expect(
    reconcileKeyedList(
      current,
      { removed: [], upserted: [], order: ["a", "a"] },
      keyOf,
    ),
  ).toBeUndefined();
  expect(
    reconcileKeyedList(
      current,
      { removed: [], upserted: [], order: ["a", "missing"] },
      keyOf,
    ),
  ).toBeUndefined();
});
