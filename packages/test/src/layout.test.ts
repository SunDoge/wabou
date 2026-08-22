import { describe, expect, test } from "bun:test";
import {
  assertLayoutRectContains,
  layoutRectBottom,
  layoutRectRight,
} from "./layout";
import { reactiveRuntimeDiagnostic } from "./layout-node";

describe("layout rect assertions", () => {
  test("reports stable right and bottom edges", () => {
    const rect = { x: 12, y: 20, width: 80, height: 40 };
    expect(layoutRectRight(rect)).toBe(92);
    expect(layoutRectBottom(rect)).toBe(60);
  });

  test("accepts contained geometry and the default rounding tolerance", () => {
    const outer = { x: 10, y: 10, width: 100, height: 80 };
    expect(() =>
      assertLayoutRectContains(
        outer,
        { x: 9, y: 11, width: 102, height: 78 },
        { label: "fixture" },
      ),
    ).not.toThrow();
  });

  test("identifies geometry outside its component boundary", () => {
    expect(() =>
      assertLayoutRectContains(
        { x: 0, y: 0, width: 100, height: 100 },
        { x: 20, y: 20, width: 90, height: 40 },
        { label: "component body" },
      ),
    ).toThrow("component body");
  });

  test("recognizes reactive runtime diagnostics in command output", () => {
    expect(
      reactiveRuntimeDiagnostic(
        "info before\nWARN js: [STRICT_READ_UNTRACKED] direct read\ninfo after",
      ),
    ).toBe("WARN js: [STRICT_READ_UNTRACKED] direct read");
    expect(
      reactiveRuntimeDiagnostic("ERROR js: [REACTIVITY_HALTED] update ignored"),
    ).toContain("[REACTIVITY_HALTED]");
    expect(reactiveRuntimeDiagnostic("ordinary renderer warning")).toBeUndefined();
  });
});
