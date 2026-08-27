import { describe, expect, test } from "bun:test";

import { CodeEditorDocument } from "./code-editor-state";

describe("headless CodeEditor document", () => {
  test("updates JSON through CodeMirror transactions and Lezer ranges", () => {
    const document = new CodeEditorDocument('{"enabled":true}', "json");
    const initial = document.update('{"enabled":true}', "json");
    const value = '{"enabled":false,"port":9090}';
    const updated = document.update(value, "json");

    expect(
      initial.syntax?.ranges.some((range) => range.kind === "boolean"),
    ).toBe(true);
    expect(updated.syntax).toMatchObject({
      language: "json",
      offsetEncoding: "utf16",
      documentLength: value.length,
    });
    expect(updated.syntax?.ranges.map((range) => range.kind)).toContain(
      "number",
    );
    expect(document.value).toBe(value);
  });

  test("uses explicit UTF-16 offsets across astral Unicode", () => {
    const value = '{"😀":true}';
    const config = new CodeEditorDocument(value, "json").update(value, "json");

    expect(config.syntax?.documentLength).toBe(value.length);
    expect(
      config.syntax?.ranges.every((range) => range.to <= value.length),
    ).toBe(true);
  });
});
