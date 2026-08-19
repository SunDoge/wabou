import { describe, expect, test } from "bun:test";
import {
  formatNodeKey,
  isNodeKey,
  NodeKeyAllocator,
  NodeKeyTable,
  nodeKey,
  nodeKeyEquals,
  nodeKeyFromSlotMapFfi,
  ROOT_NODE_KEY,
} from "./node-key";

describe("NodeKey", () => {
  test("validates the complete wire identity", () => {
    expect([ROOT_NODE_KEY.lo, ROOT_NODE_KEY.hi]).toEqual([1, 1]);
    expect(nodeKeyEquals(nodeKey(9, 3), nodeKey(9, 3))).toBe(true);
    expect(nodeKeyEquals(nodeKey(9, 3), nodeKey(9, 5))).toBe(false);
    expect(isNodeKey({ lo: 9, hi: 3 })).toBe(true);
    expect(isNodeKey({ lo: 9, hi: 2 })).toBe(false);
    expect(formatNodeKey(nodeKey(9, 3))).toBe("9v3");
    const ffiKey = nodeKeyFromSlotMapFfi(9, 3);
    expect([ffiKey.lo, ffiKey.hi]).toEqual([9, 3]);
    expect(() => nodeKeyFromSlotMapFfi(9, 2)).toThrow("non-zero odd");
    expect(() => nodeKey(0, 1)).toThrow("slot zero");
    expect(() => nodeKey(1, 2)).toThrow("non-zero odd");
  });

  test("reuses slots without reusing generations", () => {
    const allocator = new NodeKeyAllocator();
    const first = allocator.allocate();
    expect([first.lo, first.hi]).toEqual([2, 1]);
    expect(allocator.release(first)).toBe(true);
    expect(allocator.release(first)).toBe(false);

    const second = allocator.allocate();
    expect([second.lo, second.hi]).toEqual([2, 3]);
    expect(allocator.isLive(first)).toBe(false);
    expect(allocator.isLive(second)).toBe(true);
  });

  test("a slot-indexed table rejects stale generations", () => {
    const table = new NodeKeyTable<string>();
    const oldKey = nodeKey(12, 1);
    const newKey = nodeKey(12, 3);
    table.set(oldKey, "old");
    table.set(newKey, "new");

    expect(table.get(oldKey)).toBeUndefined();
    expect(table.get(newKey)).toBe("new");
    expect(table.delete(oldKey)).toBe(false);
    expect(table.delete(newKey)).toBe(true);
  });
});
