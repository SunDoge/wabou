import { expect, test } from "bun:test";
import { createRoot, createSignal, flush } from "solid-js";
import {
  calculateVirtualRange,
  createVirtualItemIdentity,
  createVirtualRow,
  validateVirtualItemKeys,
} from "./virtual-list";

test("calculates a bounded visible range with overscan", () => {
  expect(calculateVirtualRange(10_000, 20, 100, 200, 1)).toEqual({
    start: 9,
    end: 16,
  });
  expect(calculateVirtualRange(3, 20, 100, 10_000, 2)).toEqual({
    start: 0,
    end: 3,
  });
  expect(() => calculateVirtualRange(1, 0, 100, 0, 1)).toThrow(
    "itemHeight must be positive",
  );
});

test("a virtual row follows item replacement at the same index", () =>
  createRoot((dispose) => {
    const [items, setItems] = createSignal<
      readonly { id: string; label: string }[]
    >([{ id: "task", label: "waiting" }]);
    const row = createVirtualRow(items, () => 0);
    expect(row()?.label).toBe("waiting");

    setItems([{ id: "task", label: "active" }]);
    flush();
    expect(row()?.label).toBe("active");
    dispose();
  }));

test("a virtual row keeps identity across immutable item refreshes", () =>
  createRoot((dispose) => {
    const [items, setItems] = createSignal<
      readonly { id: string; value: number }[]
    >([{ id: "stable", value: 1 }]);
    const identity = createVirtualItemIdentity(
      items,
      () => 0,
      (item) => item.id,
    );
    expect(identity()?.key).toBe("string:stable");
    const stableIdentity = identity();

    setItems([{ id: "stable", value: 2 }]);
    flush();
    expect(identity()?.key).toBe("string:stable");
    expect(identity()).toBe(stableIdentity);

    setItems([{ id: "replacement", value: 2 }]);
    flush();
    expect(identity()?.key).toBe("string:replacement");
    dispose();
  }));

test("virtual list keys are unique, finite, and type-sensitive", () => {
  expect(validateVirtualItemKeys(["a", "b"], (item) => item)).toEqual([
    "a",
    "b",
  ]);
  expect(validateVirtualItemKeys([1, 2], (item) => item)).toEqual([1, 2]);
  expect(validateVirtualItemKeys(["1", 1] as const, (item) => item)).toEqual([
    "1",
    1,
  ]);
  expect(() =>
    validateVirtualItemKeys(["same", "same"], (item) => item),
  ).toThrow("duplicated at index 1");
  expect(() => validateVirtualItemKeys([Number.NaN], (item) => item)).toThrow(
    "index 0 must be finite",
  );
  expect(() =>
    validateVirtualItemKeys([undefined], (_item, index) => index),
  ).toThrow("item at index 0 is undefined");
});
