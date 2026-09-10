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

  test("owns editing, grapheme movement, selection and undo in CodeMirror", () => {
    const document = new CodeEditorDocument("A😀B", "json");
    document.setSelection(1, 3);
    expect(document.commitText("x")).toBe(true);
    expect(document.value).toBe("AxB");
    expect(document.selection).toEqual({ anchor: 2, head: 2 });

    expect(
      document.handleKey({ key: "z", shift: false, primary: true }),
    ).toEqual({ handled: true, changed: true });
    expect(document.value).toBe("A😀B");
    expect(document.selection).toEqual({ anchor: 1, head: 3 });
  });

  test("keeps IME preedit transient and commits it as one transaction", () => {
    const document = new CodeEditorDocument("key: ", "json");
    document.setSelection(5, 5);
    expect(document.setComposition("中文", 0, 2)).toBe(true);
    expect(document.value).toBe("key: ");
    expect(document.config("json").composition?.text).toBe("中文");
    expect(document.commitText("中文")).toBe(true);
    expect(document.value).toBe("key: 中文");
    expect(document.config("json").composition).toBeNull();
  });
});
