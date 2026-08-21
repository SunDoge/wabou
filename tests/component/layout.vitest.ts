import {
  assertNoLayoutDiagnostics,
  formatLayoutTree,
  getLayoutNode,
  parseLayoutSnapshot,
  type LayoutSnapshot,
  type LayoutSnapshotNode,
  siblingCollisionDiagnostics,
  styleDiagnostics,
  visibleOverflowDiagnostics,
} from "@wabou/test/layout";
import { layoutCommandArgs } from "@wabou/test/layout/node";
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
    view#1:1 role=toolbar name=\"Toolbar\" rect=(0.0,0.0 100.0x40.0) content=(0.0,0.0 100.0x40.0) class=\"flex overflow-visible\"
      text#2:1 role=label name=\"Long label\" text=\"Long label\" rect=(0.0,0.0 110.0x20.0) content=(0.0,0.0 110.0x20.0)
      button#3:1 role=button name=\"Copy\" rect=(90.0,0.0 20.0x20.0) content=(90.0,0.0 20.0x20.0)
    "
  `);
});

test("lets component tests opt into overflow, collision and style contracts", () => {
  const snapshot = fixture();

  expect(visibleOverflowDiagnostics(snapshot)).toHaveLength(2);
  expect(siblingCollisionDiagnostics(snapshot)).toHaveLength(1);
  expect(styleDiagnostics(snapshot)).toHaveLength(1);
  expect(() =>
    assertNoLayoutDiagnostics(visibleOverflowDiagnostics(snapshot)),
  ).toThrow("[visible-overflow]");
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
