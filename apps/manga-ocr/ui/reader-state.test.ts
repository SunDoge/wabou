import { describe, expect, test } from "bun:test";
import { selectRegionAndReveal } from "./reader-state";

describe("manga reader region selection", () => {
  test("selects an annotation and reveals its matching result item", () => {
    const selected: Array<string | null> = [];
    let focused = 0;
    const targets = new Map([
      ["region-2", { focus: () => focused++ }],
    ]);

    selectRegionAndReveal(
      "region-2",
      (id) => selected.push(id),
      targets,
    );

    expect(selected).toEqual(["region-2"]);
    expect(focused).toBe(1);
  });

  test("clears selection without trying to reveal an item", () => {
    const selected: Array<string | null> = [];
    selectRegionAndReveal(null, (id) => selected.push(id), new Map());
    expect(selected).toEqual([null]);
  });
});
