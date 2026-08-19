import { describe, expect, test } from "bun:test";
import { nextAccordionValue } from "./disclosure";

describe("nextAccordionValue", () => {
  test("selects one item and optionally collapses it", () => {
    expect(nextAccordionValue("first", "single", "second")).toBe("second");
    expect(nextAccordionValue("first", "single", "first", true)).toBe("");
    expect(nextAccordionValue("first", "single", "first", false)).toBe("first");
  });

  test("adds and removes items in multiple mode without mutation", () => {
    const current = ["first"];
    expect(nextAccordionValue(current, "multiple", "second")).toEqual([
      "first",
      "second",
    ]);
    expect(current).toEqual(["first"]);
    expect(nextAccordionValue(current, "multiple", "first")).toEqual([]);
  });
});
