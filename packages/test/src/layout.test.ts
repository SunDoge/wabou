import { describe, expect, test } from "bun:test";
import {
  assertLayoutRectContains,
  type LayoutSnapshot,
  layoutColorContrast,
  layoutRectBottom,
  layoutRectRight,
  visibleOverflowDiagnostics,
  visualQualityDiagnostics,
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
    expect(
      reactiveRuntimeDiagnostic("ordinary renderer warning"),
    ).toBeUndefined();
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
    expect(diagnostic?.message).toContain(
      'text="A filename that does not fit"',
    );
    expect(diagnostic?.message).toContain('name="Attachment row"');
    expect(diagnostic?.message).toContain("rect=(5.0,5.0 90.0x20.0)");
    expect(diagnostic?.message).toContain(
      'path: view#1:1["Attachment row"] > text#2:1',
    );
  });

  test("measures opaque theme color contrast", () => {
    expect(layoutColorContrast("#000", "#fff")).toBeCloseTo(21, 4);
    expect(layoutColorContrast("#777777", "#ffffff")).toBeCloseTo(4.478, 2);
    expect(layoutColorContrast("#00000080", "#ffffff")).toBeUndefined();
  });

  test("reports weak text contrast and undersized interactive targets", () => {
    const snapshot: LayoutSnapshot = {
      status: {
        viewportWidth: 200,
        viewportHeight: 100,
        deviceScale: 1,
        nodeCount: 3,
      },
      nodes: [
        {
          id: { lo: 1, hi: 1 },
          tag: "view",
          classes: ["bg-surface"],
          attrs: [],
          rect: { x: 0, y: 0, width: 200, height: 100 },
          contentRect: { x: 0, y: 0, width: 200, height: 100 },
          styleDiagnostics: [],
          computed: { background: "#ffffff" },
        },
        {
          id: { lo: 2, hi: 1 },
          parentId: { lo: 1, hi: 1 },
          tag: "text",
          text: "Barely visible",
          classes: ["text-muted"],
          attrs: [],
          rect: { x: 8, y: 8, width: 90, height: 20 },
          contentRect: { x: 8, y: 8, width: 90, height: 20 },
          styleDiagnostics: [],
          computed: { textColor: "#aaaaaa", fontSize: 14 },
        },
        {
          id: { lo: 3, hi: 1 },
          parentId: { lo: 1, hi: 1 },
          tag: "button",
          classes: ["w-5", "h-5"],
          attrs: [["role", "button"]],
          rect: { x: 120, y: 8, width: 20, height: 20 },
          contentRect: { x: 120, y: 8, width: 20, height: 20 },
          styleDiagnostics: [],
          computed: { background: "#ffffff" },
        },
        {
          id: { lo: 4, hi: 1 },
          parentId: { lo: 1, hi: 1 },
          tag: "button",
          classes: ["w-11", "h-6"],
          attrs: [["role", "switch"]],
          rect: { x: 145, y: 8, width: 44, height: 24 },
          contentRect: { x: 145, y: 8, width: 44, height: 24 },
          styleDiagnostics: [],
          computed: { background: "#dddddd" },
        },
      ],
    };

    const diagnostics = visualQualityDiagnostics(snapshot);
    expect(diagnostics.map((item) => item.code)).toEqual([
      "low-text-contrast",
      "interactive-target-too-small",
    ]);
    expect(diagnostics[0]?.message).toContain("2.32:1 text contrast");
    expect(diagnostics[1]?.message).toContain("20.0x20.0");
  });
});
