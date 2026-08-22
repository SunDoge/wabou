import {
  assertNoLayoutDiagnostics,
  formatLayoutTree,
  getLayoutNode,
  type LayoutSnapshot,
  type LayoutSnapshotNode,
  parseLayoutSnapshot,
  siblingCollisionDiagnostics,
  styleDiagnostics,
  textCollisionDiagnostics,
  visibleOverflowDiagnostics,
} from "@wabou/test/layout";
import {
  layoutCommandArgs,
  parseLayoutFixtureReport,
  validateLayoutFixtureReport,
} from "@wabou/test/layout/node";
import { expect, test } from "vitest";

function node(
  lo: number,
  tag: string,
  rect: [number, number, number, number],
  overrides: Partial<LayoutSnapshotNode> = {},
): LayoutSnapshotNode {
  return {
    id: { lo, hi: 1 },
    parentId: null,
    tag,
    text: null,
    classes: [],
    attrs: [],
    rect: { x: rect[0], y: rect[1], width: rect[2], height: rect[3] },
    contentRect: {
      x: rect[0],
      y: rect[1],
      width: rect[2],
      height: rect[3],
    },
    styleDiagnostics: [],
    computed: {
      position: "Relative",
      overflowX: "Visible",
      overflowY: "Visible",
      overlayPlane: "Content",
    },
    ...overrides,
  };
}

function fixture(): LayoutSnapshot {
  return {
    status: {
      viewportWidth: 800,
      viewportHeight: 600,
      deviceScale: 1,
      nodeCount: 3,
    },
    nodes: [
      node(1, "view", [0, 0, 100, 40], {
        attrs: [["aria-label", "Toolbar"]],
        semantic: { role: "toolbar", label: "Toolbar" },
        classes: ["flex", "overflow-visible"],
      }),
      node(2, "text", [0, 0, 110, 20], {
        parentId: { lo: 1, hi: 1 },
        text: "Long label",
        semantic: { role: "label", label: "Long label" },
      }),
      node(3, "button", [90, 0, 20, 20], {
        parentId: { lo: 1, hi: 1 },
        attrs: [["aria-label", "Copy"]],
        semantic: { role: "button", label: "Copy" },
        styleDiagnostics: ["unsupported utility `bad-class`"],
      }),
    ],
  };
}

test("formats and queries the exact retained Taffy projection in TypeScript", () => {
  const snapshot = fixture();

  expect(getLayoutNode(snapshot, { role: "button", name: "Copy" }).id.lo).toBe(
    3,
  );
  expect(formatLayoutTree(snapshot)).toMatchInlineSnapshot(`
    "viewport 800x600 scale=1 nodes=3
    view#1:1 role=toolbar name="Toolbar" rect=(0.0,0.0 100.0x40.0) content=(0.0,0.0 100.0x40.0) class="flex overflow-visible"
      text#2:1 role=label name="Long label" text="Long label" rect=(0.0,0.0 110.0x20.0) content=(0.0,0.0 110.0x20.0)
      button#3:1 role=button name="Copy" rect=(90.0,0.0 20.0x20.0) content=(90.0,0.0 20.0x20.0)
    "
  `);
});

test("lets component tests opt into overflow, collision and style contracts", () => {
  const snapshot = fixture();

  expect(visibleOverflowDiagnostics(snapshot)).toHaveLength(2);
  expect(siblingCollisionDiagnostics(snapshot)).toHaveLength(1);
  expect(textCollisionDiagnostics(snapshot)).toHaveLength(0);
  expect(styleDiagnostics(snapshot)).toHaveLength(1);
  expect(() =>
    assertNoLayoutDiagnostics(visibleOverflowDiagnostics(snapshot)),
  ).toThrow("[visible-overflow]");
});

test("finds text collisions across different component subtrees", () => {
  const snapshot: LayoutSnapshot = {
    ...fixture(),
    nodes: [
      node(1, "view", [0, 0, 100, 40]),
      node(2, "view", [0, 0, 100, 10], { parentId: { lo: 1, hi: 1 } }),
      node(3, "text", [0, 0, 60, 20], {
        parentId: { lo: 2, hi: 1 },
        text: "Reaction",
      }),
      node(4, "view", [0, 18, 100, 10], { parentId: { lo: 1, hi: 1 } }),
      node(5, "text", [0, 18, 60, 20], {
        parentId: { lo: 4, hi: 1 },
        text: "Delivered",
      }),
    ],
  };

  expect(siblingCollisionDiagnostics(snapshot)).toEqual([]);
  expect(textCollisionDiagnostics(snapshot)).toHaveLength(1);
  expect(() =>
    assertNoLayoutDiagnostics(textCollisionDiagnostics(snapshot)),
  ).toThrow("[text-overlap]");
});

test("ignores one-pixel flex rounding but supports strict collision checks", () => {
  const snapshot = fixture();
  snapshot.nodes[1].rect = { x: 0, y: 0, width: 51, height: 20 };
  snapshot.nodes[2].rect = { x: 50, y: 0, width: 50, height: 20 };

  expect(siblingCollisionDiagnostics(snapshot)).toEqual([]);
  expect(siblingCollisionDiagnostics(snapshot, { tolerance: 0 })).toHaveLength(
    1,
  );
});

test("builds the no-GPU CLI invocation for Node or Bun tests", () => {
  expect(
    layoutCommandArgs({
      app: "apps/gallery",
      out: "/tmp/layout.json",
      width: 800,
      height: 600,
      scaleFactor: 2,
      skipBuild: true,
    }),
  ).toEqual([
    "layout",
    "apps/gallery",
    "--out",
    "/tmp/layout.json",
    "--width",
    "800",
    "--height",
    "600",
    "--scale-factor",
    "2",
    "--skip-build",
  ]);
});

test("builds a single-process fixture batch invocation", () => {
  expect(
    layoutCommandArgs({
      app: "apps/gallery",
      out: "/tmp/report.json",
      batch: "/tmp/manifest.json",
      mode: "layout-test",
      skipBuild: true,
    }),
  ).toEqual([
    "layout",
    "apps/gallery",
    "--out",
    "/tmp/report.json",
    "--batch",
    "/tmp/manifest.json",
    "--mode",
    "layout-test",
    "--skip-build",
  ]);
});

test("rejects drifted or malformed Rust layout snapshots at the boundary", () => {
  expect(parseLayoutSnapshot(fixture())).toEqual(fixture());
  expect(() =>
    parseLayoutSnapshot({
      ...fixture(),
      nodes: [
        {
          ...fixture().nodes[0],
          rect: { x: 0, y: 0, width: "wide", height: 40 },
        },
      ],
    }),
  ).toThrow("nodes[0].rect.width must be a finite number");
});

test("parses every native snapshot in a fixture batch", () => {
  const report = parseLayoutFixtureReport({
    version: 1,
    totalDurationMs: 2,
    cases: [
      { id: "first", durationMs: 1, snapshot: fixture() },
      { id: "second", durationMs: 1, snapshot: fixture() },
    ],
  });
  expect(report.cases.map((entry) => entry.id)).toEqual(["first", "second"]);
  expect(() =>
    parseLayoutFixtureReport({
      version: 1,
      totalDurationMs: 1,
      cases: [{ id: "broken", durationMs: 1 }],
    }),
  ).toThrow("result at index 0");
});

test("runs fixture-owned assertions and rejects style parser diagnostics by default", async () => {
  const clean = {
    version: 1 as const,
    totalDurationMs: 1,
    cases: [{ id: "clean", durationMs: 1, snapshot: fixture() }],
  };
  let asserted = false;
  await validateLayoutFixtureReport(clean, [
    {
      id: "clean",
      allowStyleDiagnostics: true,
      assert(snapshot) {
        asserted = snapshot.status.nodeCount === 3;
      },
    },
  ]);
  expect(asserted).toBe(true);

  const styled = structuredClone(fixture());
  styled.nodes[0].styleDiagnostics = ["unsupported utility `broken`"];
  await expect(
    validateLayoutFixtureReport(
      {
        version: 1,
        totalDurationMs: 1,
        cases: [{ id: "styled", durationMs: 1, snapshot: styled }],
      },
      [{ id: "styled" }],
    ),
  ).rejects.toThrow("[style-diagnostic]");

  await expect(
    validateLayoutFixtureReport(clean, [
      {
        id: "clean",
        allowStyleDiagnostics: true,
        checks: ["visible-overflow"],
      },
    ]),
  ).rejects.toThrow("[visible-overflow]");
});
