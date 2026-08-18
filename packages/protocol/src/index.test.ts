import { describe, expect, test } from "bun:test";
import { INTERACTION_POLICY, OP, Writer } from "./index";

describe("Writer limits", () => {
  test("rejects strings that cannot be represented by the wire format", () => {
    const writer = new Writer();

    expect(() => writer.createText(1, "x".repeat(0xffff))).toThrow(
      "maximum is 65534",
    );
  });

  test("deduplicates repeated text within one frame only", () => {
    const writer = new Writer();
    writer.createText(1, "🚀");
    writer.createText(2, "🚀");
    const frame = writer.flush()!;

    // Without a reference this is 30 bytes. The second four-byte emoji is a
    // four-byte marker/index pair instead of another length + UTF-8 payload.
    expect(frame.byteLength).toBe(28);
    expect(Array.from(frame.subarray(24, 28))).toEqual([0xff, 0xff, 0, 0]);

    writer.createText(3, "🚀");
    expect(writer.flush()!.byteLength).toBe(19);
  });

  test("interns structural strings once and writes stable atom IDs", () => {
    const calls: string[] = [];
    const ids = new Map<string, number>();
    const writer = new Writer((value) => {
      calls.push(value);
      let id = ids.get(value);
      if (id === undefined) {
        id = ids.size + 1;
        ids.set(value, id);
      }
      return id;
    });

    writer.createElement(1, "div");
    writer.setStyle(1, "width", "10px");
    writer.setStyle(1, "width", "20px");
    writer.setClassName(1, "flex items-center flex");
    writer.flush();

    expect(calls).toEqual(["div", "width", "flex", "items-center"]);
  });

  test("encodes a runtime affine transform as little-endian floats", () => {
    const writer = new Writer();
    writer.setTransform2D(7, [2, 0, 0, 2, 12.5, -3.25]);
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame.byteLength).toBe(37);
    expect(frame[8]).toBe(0x12);
    expect(view.getUint32(9, true)).toBe(7);
    expect(view.getFloat32(13, true)).toBe(2);
    expect(view.getFloat32(29, true)).toBe(12.5);
    expect(view.getFloat32(33, true)).toBe(-3.25);
  });

  test("encodes typed style values without strings", () => {
    const writer = new Writer(() => 9);
    writer.setStyleValue(7, "width", 1, 12.5);
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame.byteLength).toBe(22);
    expect(frame[8]).toBe(OP.SetStyleValue);
    expect(view.getUint32(9, true)).toBe(7);
    expect(view.getUint32(13, true)).toBe(9);
    expect(frame[17]).toBe(1);
    expect(view.getFloat32(18, true)).toBe(12.5);
  });

  test("encodes ordered Vello shadow layers as fixed binary records", () => {
    const writer = new Writer();
    writer.setShadows(7, [
      {
        offsetX: 1,
        offsetY: 2,
        spread: -3,
        stdDev: 4.5,
        color: 0x336699cc,
        radius: 8,
      },
    ]);
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame.byteLength).toBe(39);
    expect(frame[8]).toBe(OP.SetShadows);
    expect(view.getUint32(9, true)).toBe(7);
    expect(view.getUint16(13, true)).toBe(1);
    expect(view.getFloat32(15, true)).toBe(1);
    expect(view.getFloat32(19, true)).toBe(2);
    expect(view.getFloat32(23, true)).toBe(-3);
    expect(view.getFloat32(27, true)).toBe(4.5);
    expect(view.getUint32(31, true)).toBe(0x336699cc);
    expect(view.getFloat32(35, true)).toBe(8);
  });

  test("encodes imperative focus as a node-only operation", () => {
    const writer = new Writer();
    writer.focusNode(42);
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame.byteLength).toBe(13);
    expect(frame[8]).toBe(0x13);
    expect(view.getUint32(9, true)).toBe(42);
  });

  test("encodes absolute and relative native scroll operations", () => {
    const writer = new Writer();
    writer.scrollTo(7, Number.NaN, 120);
    writer.scrollBy(7, 0, -20);
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame[8]).toBe(0x14);
    expect(view.getUint32(9, true)).toBe(7);
    expect(view.getFloat32(13, true)).toBeNaN();
    expect(view.getFloat32(17, true)).toBe(120);
    expect(frame[21]).toBe(0x15);
    expect(view.getFloat32(26, true)).toBe(0);
    expect(view.getFloat32(30, true)).toBe(-20);
  });

  test("encodes an explicit host overlay plane", () => {
    const writer = new Writer();
    writer.setOverlayPlane(42, 2);
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame.byteLength).toBe(14);
    expect(frame[8]).toBe(OP.SetOverlayPlane);
    expect(view.getUint32(9, true)).toBe(42);
    expect(frame[13]).toBe(2);
  });

  test("encodes text behavior as one typed operation", () => {
    const writer = new Writer();
    writer.setTextBehavior(42, 0x03);
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame.byteLength).toBe(14);
    expect(frame[8]).toBe(OP.SetTextBehavior);
    expect(view.getUint32(9, true)).toBe(42);
    expect(frame[13]).toBe(0x03);
    expect(() => writer.setTextBehavior(42, 0x04)).toThrow(RangeError);
  });

  test("encodes explicit focus and subtree interaction policy", () => {
    const writer = new Writer();
    writer.setInteractionPolicy(
      42,
      INTERACTION_POLICY.Focusable | INTERACTION_POLICY.BlockSubtree,
      -1,
    );
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame.byteLength).toBe(18);
    expect(frame[8]).toBe(OP.SetInteractionPolicy);
    expect(view.getUint32(9, true)).toBe(42);
    expect(frame[13]).toBe(0x03);
    expect(view.getInt32(14, true)).toBe(-1);
    expect(() => writer.setInteractionPolicy(1, 0, 1)).toThrow(RangeError);
    expect(() => writer.setInteractionPolicy(1, 0x04, 0)).toThrow(RangeError);
  });

  test("encodes one structured widget config without a dynamic property name", () => {
    const writer = new Writer(() => 17);
    writer.setWidgetConfig(42, '{"caret":"#fff"}');
    writer.removeWidgetConfig(42);
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame[8]).toBe(OP.SetWidgetConfig);
    expect(view.getUint32(9, true)).toBe(42);
    expect(view.getUint16(13, true)).toBe(16);
    expect(frame[31]).toBe(OP.RemoveWidgetConfig);
    expect(view.getUint32(32, true)).toBe(42);
  });
});
