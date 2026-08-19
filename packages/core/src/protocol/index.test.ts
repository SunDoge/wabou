import { describe, expect, test } from "bun:test";
import {
  GRAPHIC_DATA,
  GRAPHIC_SOURCE,
  INTERACTION_POLICY,
  nodeKey,
  OP,
  Writer,
} from "./index";

const k = (lo: number) => nodeKey(lo, 1);

describe("Writer limits", () => {
  test("rejects strings that cannot be represented by the wire format", () => {
    const writer = new Writer();

    expect(() => writer.createText(k(1), "x".repeat(0xffff))).toThrow(
      "maximum is 65534",
    );
  });

  test("deduplicates repeated text within one frame only", () => {
    const writer = new Writer();
    writer.createText(k(1), "🚀");
    writer.createText(k(2), "🚀");
    const frame = writer.flush()!;

    // Without a reference this is 30 bytes. The second four-byte emoji is a
    // four-byte marker/index pair instead of another length + UTF-8 payload.
    expect(frame.byteLength).toBe(36);
    expect(Array.from(frame.subarray(32, 36))).toEqual([0xff, 0xff, 0, 0]);

    writer.createText(k(3), "🚀");
    expect(writer.flush()!.byteLength).toBe(23);
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

    writer.createElement(k(1), "div");
    writer.setStyle(k(1), "width", "10px");
    writer.setStyle(k(1), "width", "20px");
    writer.setClassName(k(1), "flex items-center flex");
    writer.flush();

    expect(calls).toEqual(["div", "width", "flex", "items-center"]);
  });

  test("creates an element without a legacy attribute payload", () => {
    const writer = new Writer(() => 7);
    writer.createElement(k(42), "view");
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame.byteLength).toBe(21);
    expect(frame[8]).toBe(OP.CreateElement);
    expect(view.getUint32(9, true)).toBe(42);
    expect(view.getUint32(17, true)).toBe(7);
  });

  test("preserves the complete node generation on the wire", () => {
    const writer = new Writer(() => 7);
    writer.createElement(nodeKey(42, 3), "view");
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(view.getUint32(9, true)).toBe(42);
    expect(view.getUint32(13, true)).toBe(3);
  });

  test("encodes a runtime affine transform as little-endian floats", () => {
    const writer = new Writer();
    writer.setTransform2D(k(7), [2, 0, 0, 2, 12.5, -3.25]);
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame.byteLength).toBe(41);
    expect(frame[8]).toBe(0x12);
    expect(view.getUint32(9, true)).toBe(7);
    expect(view.getFloat32(17, true)).toBe(2);
    expect(view.getFloat32(33, true)).toBe(12.5);
    expect(view.getFloat32(37, true)).toBe(-3.25);
  });

  test("encodes typed style values without strings", () => {
    const writer = new Writer(() => 9);
    writer.setStyleValue(k(7), "width", 1, 12.5);
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame.byteLength).toBe(26);
    expect(frame[8]).toBe(OP.SetStyleValue);
    expect(view.getUint32(9, true)).toBe(7);
    expect(view.getUint32(17, true)).toBe(9);
    expect(frame[21]).toBe(1);
    expect(view.getFloat32(22, true)).toBe(12.5);
  });

  test("encodes ordered Vello shadow layers as fixed binary records", () => {
    const writer = new Writer();
    writer.setShadows(k(7), [
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

    expect(frame.byteLength).toBe(43);
    expect(frame[8]).toBe(OP.SetShadows);
    expect(view.getUint32(9, true)).toBe(7);
    expect(view.getUint16(17, true)).toBe(1);
    expect(view.getFloat32(19, true)).toBe(1);
    expect(view.getFloat32(23, true)).toBe(2);
    expect(view.getFloat32(27, true)).toBe(-3);
    expect(view.getFloat32(31, true)).toBe(4.5);
    expect(view.getUint32(35, true)).toBe(0x336699cc);
    expect(view.getFloat32(39, true)).toBe(8);
  });

  test("encodes imperative focus as a node-only operation", () => {
    const writer = new Writer();
    writer.focusNode(k(42));
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame.byteLength).toBe(17);
    expect(frame[8]).toBe(0x13);
    expect(view.getUint32(9, true)).toBe(42);
  });

  test("encodes absolute and relative native scroll operations", () => {
    const writer = new Writer();
    writer.scrollTo(k(7), Number.NaN, 120);
    writer.scrollBy(k(7), 0, -20);
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame[8]).toBe(0x14);
    expect(view.getUint32(9, true)).toBe(7);
    expect(view.getFloat32(17, true)).toBeNaN();
    expect(view.getFloat32(21, true)).toBe(120);
    expect(frame[25]).toBe(0x15);
    expect(view.getFloat32(34, true)).toBe(0);
    expect(view.getFloat32(38, true)).toBe(-20);
  });

  test("encodes an explicit host overlay plane", () => {
    const writer = new Writer();
    writer.setOverlayPlane(k(42), 2);
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame.byteLength).toBe(18);
    expect(frame[8]).toBe(OP.SetOverlayPlane);
    expect(view.getUint32(9, true)).toBe(42);
    expect(frame[17]).toBe(2);
  });

  test("encodes text behavior as one typed operation", () => {
    const writer = new Writer();
    writer.setTextBehavior(k(42), 0x03);
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame.byteLength).toBe(18);
    expect(frame[8]).toBe(OP.SetTextBehavior);
    expect(view.getUint32(9, true)).toBe(42);
    expect(frame[17]).toBe(0x03);
    expect(() => writer.setTextBehavior(k(42), 0x04)).toThrow(RangeError);
  });

  test("encodes explicit focus and subtree interaction policy", () => {
    const writer = new Writer();
    writer.setInteractionPolicy(
      k(42),
      INTERACTION_POLICY.Focusable | INTERACTION_POLICY.BlockSubtree,
      -1,
    );
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame.byteLength).toBe(22);
    expect(frame[8]).toBe(OP.SetInteractionPolicy);
    expect(view.getUint32(9, true)).toBe(42);
    expect(frame[17]).toBe(0x03);
    expect(view.getInt32(18, true)).toBe(-1);
    expect(() => writer.setInteractionPolicy(k(1), 0, 1)).toThrow(RangeError);
    expect(() => writer.setInteractionPolicy(k(1), 0x08, 0)).toThrow(
      RangeError,
    );
  });

  test("encodes graphic sources without attribute names or JSON", () => {
    const writer = new Writer();
    writer.setGraphicSource(
      k(42),
      GRAPHIC_SOURCE.NetworkRaster,
      "https://x.test/a.png",
    );
    writer.clearGraphicSource(k(42), GRAPHIC_SOURCE.NetworkRaster);
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame[8]).toBe(OP.SetGraphicSource);
    expect(view.getUint32(9, true)).toBe(42);
    expect(frame[17]).toBe(GRAPHIC_SOURCE.NetworkRaster);
    expect(frame[18]).toBe(20);
    expect(frame[40]).toBe(OP.ClearGraphicSource);
    expect(view.getUint32(41, true)).toBe(42);
    expect(frame[49]).toBe(GRAPHIC_SOURCE.NetworkRaster);
    expect(() => writer.setGraphicSource(k(1), 3, "x")).toThrow(RangeError);
  });

  test("encodes one structured widget config without a dynamic property name", () => {
    const writer = new Writer(() => 17);
    writer.setWidgetConfig(k(42), '{"caret":"#fff"}');
    writer.removeWidgetConfig(k(42));
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame[8]).toBe(OP.SetWidgetConfig);
    expect(view.getUint32(9, true)).toBe(42);
    expect(view.getUint16(17, true)).toBe(16);
    expect(frame[35]).toBe(OP.RemoveWidgetConfig);
    expect(view.getUint32(36, true)).toBe(42);
  });

  test("encodes opaque length-delimited graphic data", () => {
    const writer = new Writer();
    writer.setGraphicData(k(42), GRAPHIC_DATA.VectorPath, new Uint8Array([7, 8, 9]));
    writer.clearGraphicData(k(42), GRAPHIC_DATA.VectorPath);
    const frame = writer.flush()!;
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(frame[8]).toBe(OP.SetGraphicData);
    expect(view.getUint32(9, true)).toBe(42);
    expect(frame[17]).toBe(GRAPHIC_DATA.VectorPath);
    expect(view.getUint32(18, true)).toBe(3);
    expect(Array.from(frame.subarray(22, 25))).toEqual([7, 8, 9]);
    expect(frame[25]).toBe(OP.ClearGraphicData);
    expect(frame[34]).toBe(GRAPHIC_DATA.VectorPath);
  });
});
