import { describe, expect, test } from "vitest";
import { PathBuilder } from "@wabou/ui";
import { appendSmoothPath } from "./smooth-path";

describe("appendSmoothPath", () => {
  test("emits cubic segments instead of a polyline", () => {
    const source = appendSmoothPath(new PathBuilder(), [
      { x: 0, y: 10 },
      { x: 10, y: 0 },
      { x: 20, y: 10 },
    ]).build();
    const data = source.data;
    expect(new DataView(data.buffer).getUint32(8, true)).toBe(3);
    expect(data[36]).toBe(1);
    expect(data[48]).toBe(4);
    expect(data[76]).toBe(4);
  });
});
