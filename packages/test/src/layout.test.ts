import { describe, expect, test } from "bun:test";
import {
  assertLayoutRectContains,
  layoutRectBottom,
  layoutRectRight,
  type LayoutSnapshot,
  visibleOverflowDiagnostics,
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

  test("reports actionable context for visible overflow", () => {
    const snapshot: LayoutSnapshot = {
      status: {
        viewportWidth: 100,
        viewportHeight: 100,
        deviceScale: 1,
        nodeCount: 2,
      },
      nodes: [
        {
          id: { lo: 1, hi: 1 },
          tag: "view",
          classes: ["w-20", "overflow-visible"],
          attrs: [["aria-label", "Attachment row"]],
          rect: { x: 0, y: 0, width: 80, height: 40 },
          contentRect: { x: 0, y: 0, width: 80, height: 40 },
          styleDiagnostics: [],
          computed: { overflowX: "Visible", overflowY: "Visible" },
        },
        {
          id: { lo: 2, hi: 1 },
          parentId: { lo: 1, hi: 1 },
          tag: "text",
          text: "A filename that does not fit",
          classes: ["font-medium"],
          attrs: [],
          rect: { x: 5, y: 5, width: 90, height: 20 },
          contentRect: { x: 5, y: 5, width: 90, height: 20 },
          styleDiagnostics: [],
          computed: {},
        },
      ],
    };

    const [diagnostic] = visibleOverflowDiagnostics(snapshot);
    expect(diagnostic?.message).toContain("extends 15.0px outside");
    expect(diagnostic?.message).toContain('text="A filename that does not fit"');
    expect(diagnostic?.message).toContain('name="Attachment row"');
    expect(diagnostic?.message).toContain("rect=(5.0,5.0 90.0x20.0)");
    expect(diagnostic?.message).toContain(
      'path: view#1:1["Attachment row"] > text#2:1',
    );
  });
});
