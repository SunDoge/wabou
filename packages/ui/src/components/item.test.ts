import { describe, expect, test } from "bun:test";
import { itemClass, itemMediaClass } from "./item";

describe("Item", () => {
  test("keeps rows shrinkable and wraps narrow content", () => {
    const classes = itemClass();
    expect(classes).toContain("min-w-0");
    expect(classes).toContain("flex-wrap");
    expect(classes).toContain("border-transparent");
  });

  test("maps variants without relying on CSS selectors", () => {
    expect(itemClass("outline", "sm")).toContain("border-subtle");
    expect(itemClass("outline", "sm")).toContain("py-3");
    expect(itemClass("muted")).toContain("bg-control");
  });

  test("gives icon and image media explicit native geometry", () => {
    expect(itemMediaClass("icon")).toContain("w-8 h-8");
    expect(itemMediaClass("image")).toContain("w-10 h-10");
    expect(itemMediaClass("image")).toContain("overflow-hidden");
  });
});
