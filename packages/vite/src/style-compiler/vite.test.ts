import { describe, expect, test } from "bun:test";
import { resolve } from "node:path";
import {
  auditColorThemeContrast,
  assertSupportedWabouCandidates,
  compileWabouUtilities,
  compileColorThemes,
  extractUtilitySource,
  findWorkspacePackages,
  filterIgnoredClasses,
  matchesClassPattern,
} from "./vite";

describe("utility source extraction", () => {
  test("filters exact and globbed third-party metadata classes", () => {
    expect(matchesClassPattern("lucide-sun", "lucide-*")).toBe(true);
    expect(matchesClassPattern("text-primary", "lucide-*")).toBe(false);
    expect(
      filterIgnoredClasses(
        ["lucide", "lucide-icon", "lucide-sun", "w-4"],
        ["lucide", "lucide-*"],
      ),
    ).toEqual(["w-4"]);
  });

  test("turns unsupported utility candidates into build errors", () => {
    expect(() =>
      assertSupportedWabouCandidates(["flex", "p-[var(--space)]"]),
    ).toThrow("invalid Wabou spacing");
    expect(() =>
      assertSupportedWabouCandidates(["hover:bg-slate-900"]),
    ).toThrow("use Solid classList or typed style");
  });

  test("compiles utilities directly to typed Style IR without CSS", () => {
    expect(compileWabouUtilities(["px-[13px]", "bg-slate-900"])).toEqual([
      {
        className: "bg-slate-900",
        specificity: 10,
        sourceOrder: 0,
        declarations: [
          {
            property: "background-color",
            value: {
              type: "color",
              value: { kind: "literal", rgba: 253176575 },
            },
          },
        ],
      },
      {
        className: "px-[13px]",
        specificity: 10,
        sourceOrder: 1,
        declarations: [
          {
            property: "padding-left",
            value: { type: "length", value: { unit: "px", value: 13 } },
          },
          {
            property: "padding-right",
            value: { type: "length", value: { unit: "px", value: 13 } },
          },
        ],
      },
    ]);
  });

  test("compiles complete named palettes and semantic color utilities", () => {
    const themes = compileColorThemes({
      default: "dark",
      themes: {
        dark: {
          appearance: "dark",
          colors: { canvas: "#020617", primary: "#f1f5f9" },
        },
        light: {
          appearance: "light",
          colors: { canvas: "#f8fafc", primary: "#0f172a" },
        },
      },
    });
    const tokens = new Set(Object.keys(themes!.themes.dark.colors));
    expect(
      compileWabouUtilities(["bg-canvas", "text-primary"], 0, tokens),
    ).toEqual([
      {
        className: "bg-canvas",
        specificity: 10,
        sourceOrder: 0,
        declarations: [
          {
            property: "background-color",
            value: { type: "color", value: { kind: "token", name: "canvas" } },
          },
        ],
      },
      {
        className: "text-primary",
        specificity: 10,
        sourceOrder: 1,
        declarations: [
          {
            property: "color",
            value: { type: "color", value: { kind: "token", name: "primary" } },
          },
        ],
      },
    ]);
  });

  test("rejects incomplete color palettes", () => {
    expect(() =>
      compileColorThemes({
        default: "dark",
        themes: {
          dark: {
            appearance: "dark",
            colors: { canvas: "#020617", primary: "#f1f5f9" },
          },
          light: { appearance: "light", colors: { canvas: "#f8fafc" } },
        },
      }),
    ).toThrow("missing: primary");
  });

  test("reports semantic text tokens that wash out against app surfaces", () => {
    const themes = compileColorThemes({
      default: "light",
      themes: {
        light: {
          appearance: "light",
          colors: {
            canvas: "#f7f7f8",
            surface: "#ffffff",
            primary: "#242424",
            secondary: "#666666",
            muted: "#929292",
          },
        },
      },
    });

    expect(auditColorThemeContrast(themes)).toEqual([
      expect.objectContaining({
        theme: "light",
        foreground: "muted",
        background: "canvas",
        minimum: 4.5,
      }),
      expect.objectContaining({
        theme: "light",
        foreground: "muted",
        background: "surface",
        minimum: 4.5,
      }),
    ]);
    expect(auditColorThemeContrast(themes)[0]?.ratio).toBeCloseTo(2.91, 2);
  });

  test("only exposes explicit JSX class props to UnoCSS", () => {
    const source = `
      <View role="tab" title="flex hidden" class="flex items-center" />
      <View className={'h-8 px-3'} command="grid" />
    `;

    expect(extractUtilitySource(source)).toBe("flex items-center\n'h-8 px-3'");
    expect(extractUtilitySource(source)).not.toContain("role");
    expect(extractUtilitySource(source)).not.toContain("hidden");
    expect(extractUtilitySource(source)).not.toContain("grid");
  });

  test("retains candidates from reactive class expressions", () => {
    const source = `<View class={active() ? "bg-slate-700" : "bg-transparent"} />`;

    expect(extractUtilitySource(source)).toContain("bg-slate-700");
    expect(extractUtilitySource(source)).toContain("bg-transparent");
  });

  test("ignores comparison constants inside class expressions", () => {
    const source = `<View class={variant() === "outline" ? "border" : "border-0"} />`;
    const extracted = extractUtilitySource(source);
    expect(extracted).not.toContain("outline");
    expect(extracted).toContain("border");
    expect(extracted).toContain("border-0");
  });

  test("extracts explicit Solid classList keys", () => {
    const source = `<View class="flex bg-slate-900" classList={{
      "bg-slate-700": hovered(),
      'opacity-50': disabled(),
      hidden: collapsed(),
    }} />`;
    const extracted = extractUtilitySource(source);
    expect(extracted).toContain("bg-slate-700");
    expect(extracted).toContain("opacity-50");
    expect(extracted).toContain("hidden");
  });

  test("rejects dynamically constructed utility strings", () => {
    expect(() =>
      extractUtilitySource(
        String.raw`<View class={\`flex p-[\${gap()}px]\`} />`,
      ),
    ).toThrow("put continuous values in typed style");
    expect(() =>
      extractUtilitySource('<View class={"flex p-[" + gap() + "px]"} />'),
    ).toThrow("put continuous values in typed style");
    expect(() =>
      extractUtilitySource(String.raw`<View class={\`bg-\${color()}\`} />`),
    ).toThrow("select complete static utilities with classList");
    expect(() =>
      extractUtilitySource('<View class={"bg-" + color()} />'),
    ).toThrow("select complete static utilities with classList");
  });

  test("allows reactive selection between complete static utilities", () => {
    expect(() =>
      extractUtilitySource(
        String.raw`<View class={\`flex \${active() ? "bg-slate-700" : "bg-transparent"}\`} />`,
      ),
    ).not.toThrow();
  });

  test("discovers shared workspace packages from a nested app", async () => {
    const workspace = resolve(import.meta.dir, "../../../..");
    expect(
      await findWorkspacePackages(resolve(workspace, "apps/gallery")),
    ).toBe(resolve(workspace, "packages"));
  });
});
