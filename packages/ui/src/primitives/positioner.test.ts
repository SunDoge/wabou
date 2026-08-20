import { describe, expect, test } from "bun:test";
import { nodeKey, type NodeKey } from "@wabou/core/protocol";
import {
  computeFloatingPosition,
  computeHostFloatingPosition,
  flip,
  type LayoutRect,
  offset,
  shift,
} from "./positioner";

type Target = "reference" | "floating";

const viewport: LayoutRect = { x: 0, y: 0, width: 300, height: 200 };
const k = (lo: number) => nodeKey(lo, 1);

function platform(rects: Record<Target, LayoutRect>) {
  return {
    getRect: (target: Target) => rects[target],
    getClippingRect: () => viewport,
  };
}

describe("Floating UI platform adapter", () => {
  test("positions from plain Wabou layout rectangles without a DOM", async () => {
    const result = await computeFloatingPosition("reference", "floating", {
      platform: platform({
        reference: { x: 100, y: 50, width: 40, height: 20 },
        floating: { x: 0, y: 0, width: 80, height: 30 },
      }),
      placement: "bottom-start",
      middleware: [offset(8)],
    });

    expect(result).toMatchObject({ x: 100, y: 78, placement: "bottom-start" });
  });

  test("flips an overlay that would leave the viewport", async () => {
    const result = await computeFloatingPosition("reference", "floating", {
      platform: platform({
        reference: { x: 100, y: 175, width: 40, height: 20 },
        floating: { x: 0, y: 0, width: 80, height: 40 },
      }),
      placement: "bottom",
      middleware: [flip()],
    });

    expect(result).toMatchObject({ x: 80, y: 135, placement: "top" });
  });

  test("shifts an overlay back inside the viewport", async () => {
    const result = await computeFloatingPosition("reference", "floating", {
      platform: platform({
        reference: { x: 280, y: 50, width: 20, height: 20 },
        floating: { x: 0, y: 0, width: 80, height: 30 },
      }),
      placement: "bottom",
      middleware: [shift({ crossAxis: true })],
    });

    expect(result).toMatchObject({ x: 220, y: 70, placement: "bottom" });
  });

  test("batches native handles into one coherent Host snapshot", async () => {
    let calls = 0;
    const host = {
      layout: {
        snapshot: (targets: ReadonlyArray<NodeKey | { id: NodeKey }>) => {
          calls++;
          expect(targets).toHaveLength(2);
          return {
            revision: 7,
            viewport,
            nodes: [
              {
                id: k(1),
                rect: { x: 20, y: 30, width: 40, height: 10 },
                clip: viewport,
                scroll: { offsetX: 0, offsetY: 0, rangeX: 0, rangeY: 0 },
              },
              {
                id: k(2),
                rect: { x: 0, y: 0, width: 60, height: 20 },
                clip: viewport,
                scroll: { offsetX: 0, offsetY: 0, rangeX: 0, rangeY: 0 },
              },
            ],
          };
        },
      },
    };

    const result = await computeHostFloatingPosition(k(1), { id: k(2) }, host, {
      placement: "bottom-start",
    });

    expect(calls).toBe(1);
    expect(result).toMatchObject({ x: 20, y: 40 });
  });

  test("rejects an incomplete completed-layout snapshot", async () => {
    const host = {
      layout: {
        snapshot: () => ({
          revision: 8,
          viewport,
          nodes: [
            {
              id: k(1),
              rect: { x: 20, y: 30, width: 40, height: 10 },
              clip: viewport,
              scroll: { offsetX: 0, offsetY: 0, rangeX: 0, rangeY: 0 },
            },
          ],
        }),
      },
    };

    await expect(
      computeHostFloatingPosition(k(1), k(2), host, {
        placement: "bottom-start",
      }),
    ).rejects.toThrow(
      "Layout target 2v1 is not present in completed revision 8",
    );
  });
});
