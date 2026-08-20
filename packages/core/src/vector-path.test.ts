import { describe, expect, test } from "vitest";
import { PathBuilder } from "./vector-path";

describe("PathBuilder", () => {
  test("writes a versioned deterministic command stream", () => {
    const path = new PathBuilder()
      .moveTo(1, 2)
      .lineTo(3, 4)
      .quadTo(5, 6, 7, 8)
      .cubicTo(9, 10, 11, 12, 13, 14)
      .close()
      .build({ stroke: 0x38bdf8ff, strokeWidth: 2 });
    const view = new DataView(path.data.buffer);
    expect(view.getUint32(0, true)).toBe(0x31504257);
    expect(view.getUint16(4, true)).toBe(1);
    expect(view.getUint16(6, true)).toBe(0);
    expect(view.getUint32(8, true)).toBe(5);
    expect(view.getUint32(12, true)).toBe(path.data.byteLength);
    expect(view.getUint32(20, true)).toBe(0x38bdf8ff);
  });

  test("snapshots do not alias the builder or returned bytes", () => {
    const builder = new PathBuilder().moveTo(1, 2);
    const first = builder.build();
    const bytes = first.data;
    bytes[0] = 0;
    builder.lineTo(3, 4);
    expect(new DataView(first.data.buffer).getUint32(0, true)).toBe(0x31504257);
    expect(new DataView(first.data.buffer).getUint32(8, true)).toBe(1);
  });

  test("marks non-drawing snapshots so the renderer can clear stale geometry", () => {
    const empty = new PathBuilder();
    expect(empty.hasCurrentPoint).toBe(false);
    expect(empty.build().drawable).toBe(false);
    const point = new PathBuilder().moveTo(1, 2);
    expect(point.hasCurrentPoint).toBe(true);
    expect(point.build().drawable).toBe(false);
    expect(new PathBuilder().moveTo(1, 2).lineTo(3, 4).build().drawable).toBe(
      true,
    );
    point.close();
    expect(point.hasCurrentPoint).toBe(false);
  });

  test("rejects non-finite geometry and invalid paint", () => {
    expect(() => new PathBuilder().lineTo(1, 2)).toThrow(/moveTo first/);
    expect(() => new PathBuilder().moveTo(0, 0).lineTo(Number.NaN, 0)).toThrow(
      /finite/,
    );
    expect(() => new PathBuilder().close()).toThrow(/moveTo first/);
    expect(() => new PathBuilder().build({ strokeWidth: 0 })).toThrow(
      /positive/,
    );
    expect(() => new PathBuilder().splineThrough([], Number.NaN)).toThrow(
      /finite/,
    );
  });

  test("builds reusable smooth splines with explicit empty-data semantics", () => {
    const empty = new PathBuilder().splineThrough([]);
    expect(empty.hasCurrentPoint).toBe(false);
    expect(empty.build().drawable).toBe(false);

    const point = new PathBuilder().splineThrough([{ x: 2, y: 3 }]);
    expect(point.hasCurrentPoint).toBe(true);
    expect(point.build().drawable).toBe(false);

    const curve = new PathBuilder().splineThrough([
      { x: 0, y: 4 },
      { x: 3, y: 1 },
      { x: 6, y: 5 },
    ]);
    const source = curve.build();
    expect(source.drawable).toBe(true);
    expect(new DataView(source.data.buffer).getUint32(8, true)).toBe(3);
  });
});
