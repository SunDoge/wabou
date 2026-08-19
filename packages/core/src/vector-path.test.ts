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

  test("rejects non-finite geometry and invalid paint", () => {
    expect(() => new PathBuilder().lineTo(Number.NaN, 0)).toThrow(/finite/);
    expect(() => new PathBuilder().build({ strokeWidth: 0 })).toThrow(/positive/);
  });
});
