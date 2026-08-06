import { describe, expect, test } from "bun:test";
import { resolve } from "node:path";
import {
  assertSupportedWabouCandidates,
  compileWabouUtilities,
  extractUtilitySource,
  findWorkspacePackages,
} from "./vite";

describe("utility source extraction", () => {
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
    const workspace = resolve(import.meta.dir, "../../..");
    expect(
      await findWorkspacePackages(resolve(workspace, "apps/gallery")),
    ).toBe(resolve(workspace, "packages"));
  });
});
