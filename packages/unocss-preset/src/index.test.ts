import { describe, expect, test } from "bun:test";
import { createGenerator } from "@unocss/core";
import {
  presetWabou,
  resolveWabouUtility,
  validateWabouUtility,
  wabouUtilityManifest,
} from "./index.ts";

describe("presetWabou", () => {
  test("matches Rust spacing, arbitrary values and colors", () => {
    expect(resolveWabouUtility("px-3")?.declarations).toHaveLength(2);
    expect(resolveWabouUtility("bg-slate-900")?.declarations[0].property).toBe(
      "background-color",
    );
  });

  test("rejects unsupported CSS expressions", () => {
    expect(validateWabouUtility("p-[var(--space)]")?.message).toContain(
      "invalid Wabou spacing",
    );
    expect(validateWabouUtility("w-[calc(100%-2rem)]")?.message).toContain(
      "invalid Wabou dimension",
    );
  });

  test("rejects variants in favor of explicit Solid state", () => {
    expect(validateWabouUtility("hover:bg-slate-900")?.message).toContain(
      "use Solid classList or typed style",
    );
  });

  test("generates CSS for UnoCSS tooling and editor integrations", async () => {
    const uno = await createGenerator({ presets: [presetWabou()] });
    const result = await uno.generate("flex px-3 bg-slate-900");
    expect(result.css).toContain("padding-left:12px");
    expect(result.css).toContain("background-color:#0f172aff");
    const transform = await uno.generate("translate-x-4 scale-150 rotate-45");
    expect(transform.css).toContain("transform:translateX(16px)");
    expect(transform.css).toContain("transform:scale(1.5, 1.5)");
    expect(transform.css).toContain("transform:rotate(0.785398");
  });

  test("matches Rust-generated conformance cases exactly", () => {
    for (const expected of wabouUtilityManifest.conformance) {
      const resolved = resolveWabouUtility(expected.className);
      expect({
        className: resolved?.candidate,
        declarations: resolved?.declarations,
      }).toEqual(expected);
    }
  });
});
