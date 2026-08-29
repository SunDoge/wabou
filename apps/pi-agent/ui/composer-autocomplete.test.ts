import { describe, expect, test } from "vitest";
import {
  composerAutocompleteRows,
  detectComposerTrigger,
  replaceComposerTrigger,
} from "./composer-autocomplete";

describe("Pi Agent composer autocomplete", () => {
  test("recognizes commands only at the start of the active line", () => {
    expect(detectComposerTrigger("/rev", 4)).toEqual({
      kind: "command",
      query: "rev",
      start: 0,
      end: 4,
    });
    expect(detectComposerTrigger("explain /rev", 12)).toBeNull();
    expect(detectComposerTrigger("context\n/rev", 12)).toEqual({
      kind: "command",
      query: "rev",
      start: 8,
      end: 12,
    });
  });

  test("recognizes file tokens and preserves JavaScript UTF-16 offsets", () => {
    expect(detectComposerTrigger("😀 inspect @src/app", 19)).toEqual({
      kind: "file",
      query: "src/app",
      start: 11,
      end: 19,
    });
  });

  test("filters rows and replaces only the active token", () => {
    const trigger = detectComposerTrigger("Keep @render nearby", 12);
    expect(trigger).not.toBeNull();
    if (!trigger) return;
    const [row] = composerAutocompleteRows(
      trigger,
      [],
      ["src/app.tsx", "crates/renderer.rs"],
    );
    expect(row?.label).toBe("crates/renderer.rs");
    expect(
      row && replaceComposerTrigger("Keep @render nearby", trigger, row),
    ).toEqual({
      text: "Keep @crates/renderer.rs  nearby",
      cursor: 25,
    });
  });
});
