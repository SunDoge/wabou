import { describe, expect, test } from "bun:test";
import { nodeKey } from "@wabou/core/protocol";
import type { Handle } from "@wabou/core/renderer";
import { floatingFromNode, floatingFromPoint } from "./positioner";

describe("native floating-position contract", () => {
  test("retains the full generational identity of a node anchor", () => {
    const anchor = { id: nodeKey(17, 5) } as Handle;

    const position = floatingFromNode(anchor, {
      placement: "right-end",
      offset: 9,
      margin: 12,
    });

    expect(position.anchor.kind).toBe("node");
    if (position.anchor.kind !== "node") throw new Error("expected node anchor");
    expect(position.anchor.id).toBe(anchor.id);
    expect(position.placement).toBe("right-end");
    expect(position.offset).toBe(9);
    expect(position.margin).toBe(12);
  });

  test("accepts finite viewport point anchors", () => {
    expect(floatingFromPoint({ x: 42, y: 64 })).toEqual({
      anchor: { kind: "point", x: 42, y: 64 },
      placement: undefined,
      offset: undefined,
      margin: undefined,
    });
    expect(() => floatingFromPoint({ x: Number.NaN, y: 0 })).toThrow(
      "floating point anchor must be finite",
    );
  });
});
