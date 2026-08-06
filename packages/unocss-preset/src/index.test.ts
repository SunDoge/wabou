import { describe, expect, test } from "bun:test";
import { createGenerator } from "@unocss/core";
import {
  presetWabou,
  resolveWabouUtility,
  validateWabouUtility,
  wabouUtilityManifest,
} from "./index.ts";

describe("presetWabou", () => {
  const normalizeF32 = (value: unknown): unknown => {
    if (typeof value === "number") return Math.fround(value);
    if (Array.isArray(value)) return value.map(normalizeF32);
    if (value && typeof value === "object")
      return Object.fromEntries(
        Object.entries(value).map(([key, item]) => [key, normalizeF32(item)]),
      );
    return value;
  };
  test("matches Rust spacing, arbitrary values and colors", () => {
    expect(resolveWabouUtility("px-3")?.declarations).toHaveLength(2);
    expect(resolveWabouUtility("bg-slate-900")?.declarations[0].property).toBe(
      "background-color",
    );
    expect(resolveWabouUtility("gap-x-4")?.declarations).toHaveLength(1);
    expect(resolveWabouUtility("-mt-4")?.declarations[0].value).toEqual({
      type: "length",
      value: { unit: "px", value: -16 },
    });
    expect(
      resolveWabouUtility("bg-[#336699cc]")?.declarations[0].value,
    ).toEqual({
      type: "color",
      value: { kind: "literal", rgba: 0x336699cc },
    });
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
    for (const candidate of [
      "hover:bg-slate-900",
      "focus:w-4",
      "active:scale-150",
      "disabled:opacity-50",
      "sm:flex",
      "dark:bg-black",
    ]) {
      expect(validateWabouUtility(candidate)?.message).toContain(
        "use Solid classList or typed style",
      );
    }
    expect(validateWabouUtility("transition")?.message).toContain(
      "unsupported",
    );
    expect(validateWabouUtility("animate-spin")?.message).toContain(
      "unsupported",
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
      expect(
        normalizeF32({
          className: resolved?.candidate,
          declarations: resolved?.declarations,
        }),
      ).toEqual(normalizeF32(expected));
    }
  });

  test("uses manifest prefix/property expansion instead of a TS edge table", () => {
    const gapX = wabouUtilityManifest.dynamicRules
      .find(({ resolver }) => resolver === "spacing")
      ?.prefixes.find(({ name }) => name === "gap-x");
    expect(gapX?.properties).toEqual(["column-gap"]);
    expect(resolveWabouUtility("gap-x-4")?.declarations[0].property).toBe(
      "column-gap",
    );
  });

  test("resolves every manifest dynamic prefix to its declared properties", () => {
    const token: Record<string, string> = {
      spacing: "4",
      dimension: "4",
      color: "slate-900",
      opacity: "50",
      number: "[2]",
      ratio: "[16/9]",
      length: "[2px]",
      translate: "4",
      scale: "125",
      rotate: "30",
    };
    for (const rule of wabouUtilityManifest.dynamicRules) {
      for (const prefix of rule.prefixes) {
        const candidate = `${prefix.name}-${token[rule.resolver]}`;
        expect(
          resolveWabouUtility(candidate)?.declarations.map(
            ({ property }) => property,
          ),
        ).toEqual(prefix.properties);
      }
    }
  });
});
