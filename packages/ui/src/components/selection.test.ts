import { describe, expect, test } from "bun:test";
import { nextToggleGroupValue } from "./selection";

describe("ToggleGroup", () => {
  test("toggles a single value on and off", () => {
    expect(nextToggleGroupValue("", "bold", "single")).toBe("bold");
    expect(nextToggleGroupValue("bold", "bold", "single")).toBe("");
  });

  test("adds and removes multiple values without mutation", () => {
    const current = ["bold"] as const;
    expect(nextToggleGroupValue(current, "italic", "multiple")).toEqual([
      "bold",
      "italic",
    ]);
    expect(current).toEqual(["bold"]);
    expect(
      nextToggleGroupValue(["bold", "italic"], "bold", "multiple"),
    ).toEqual(["italic"]);
  });
});
