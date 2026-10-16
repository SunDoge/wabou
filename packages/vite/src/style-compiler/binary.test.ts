import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";
import { encodeStyleSheet, styleSheetModuleSource } from "./binary";
import type { WabouStyleSheet } from "./ir";

const fixture = fileURLToPath(
  new URL("../../../../fixtures/style-ir/hmr-color.json", import.meta.url),
);

describe("binary Style IR", () => {
  test("matches the shared Rust decoder fixture", async () => {
    const sheet = JSON.parse(
      await readFile(fixture, "utf8"),
    ) as WabouStyleSheet;
    const expected = await readFile(fixture.replace(/\.json$/, ".bin"));
    const encoded = encodeStyleSheet(sheet);
    expect(encoded).toEqual(new Uint8Array(expected));
    expect(encoded.byteLength).toBeLessThan(
      new TextEncoder().encode(JSON.stringify(sheet)).byteLength,
    );
  });

  test("embeds bytes without retaining a parsed stylesheet object", async () => {
    const sheet = JSON.parse(
      await readFile(fixture, "utf8"),
    ) as WabouStyleSheet;
    const source = styleSheetModuleSource(sheet);
    expect(source).toContain("new Uint8Array");
    expect(source).toContain("__wabou_set_stylesheet?.(b)");
    expect(source).not.toContain("JSON.parse");
    expect(source).not.toContain("hmr-color");
  });

  test("generated module reconstructs the exact binary payload", async () => {
    const sheet = JSON.parse(
      await readFile(fixture, "utf8"),
    ) as WabouStyleSheet;
    const expected = encodeStyleSheet(sheet);
    let received: Uint8Array | undefined;
    const previous = globalThis.__wabou_set_stylesheet;
    globalThis.__wabou_set_stylesheet = (bytes) => {
      received = bytes;
    };
    try {
      Function(styleSheetModuleSource(sheet).replace("export {};", ""))();
    } finally {
      globalThis.__wabou_set_stylesheet = previous;
    }
    expect(received).toEqual(expected);
  });
});
