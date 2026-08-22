import { expect, test } from "bun:test";
import { GRAPHIC_DATA } from "@wabou/core/protocol";
import { mount, writer } from "@wabou/core/renderer";
import { createComponent, flush } from "solid-js";
import { encodeQrCode, QRCode, qrCodePath } from "../../dist/index.mjs";

test("uqr produces a square matrix with standard finder patterns", () => {
  const matrix = encodeQrCode("https://wabou.dev");
  expect(matrix.size).toBeGreaterThanOrEqual(21);
  expect(matrix.data).toHaveLength(matrix.size);
  expect(matrix.data.every((row) => row.length === matrix.size)).toBe(true);
  expect(matrix.data[0].slice(0, 7)).toEqual([
    true,
    true,
    true,
    true,
    true,
    true,
    true,
  ]);
  expect(matrix.data[6].slice(0, 7)).toEqual([
    true,
    true,
    true,
    true,
    true,
    true,
    true,
  ]);
});

test("matrix modules are batched into one retained vector path", () => {
  const source = qrCodePath(encodeQrCode("Wabou"), 180, 4);
  expect(source.drawable).toBe(true);
  expect(source.data.byteLength).toBeLessThan(64 * 1024);
});

test("QRCode writes one native vector graphic instead of module nodes", () => {
  const written: number[] = [];
  const setGraphicData = writer.setGraphicData.bind(writer);
  writer.setGraphicData = (_id, kind) => written.push(kind);

  let disposeMount: (() => void) | undefined;
  try {
    disposeMount = mount(() =>
      createComponent(QRCode, {
        value: "https://github.com/SunDoge/wabou",
        "aria-label": "Project QR code",
      }),
    );
    flush();
  } finally {
    disposeMount?.();
    writer.setGraphicData = setGraphicData;
  }

  expect(written).toEqual([GRAPHIC_DATA.VectorPath]);
});
