import { describe, expect, test } from "bun:test";
import { findAmbiguousForUsage } from "./check-for-identity";

describe("explicit Solid list identity", () => {
  test("rejects the ambiguous For spelling", () => {
    expect(
      findAmbiguousForUsage(
        `const view = <For each={items()}>{render}</For>;`,
        "view.tsx",
      ),
    ).toEqual([{ file: "view.tsx", line: 1, column: 15 }]);
  });

  test("accepts explicit value and entity list primitives", () => {
    expect(
      findAmbiguousForUsage(`
        <ForValue each={labels()}>{render}</ForValue>;
        <ForEntity each={items()} by={(item) => item.id}>{render}</ForEntity>;
      `),
    ).toEqual([]);
  });
});
