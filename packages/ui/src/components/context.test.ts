import { expect, test } from "bun:test";
import {
  componentsControlSize,
  componentsElevation,
  componentsThemeContract,
} from "./theme";

test("default desktop geometry is shared by high-frequency controls", () => {
  expect(componentsThemeContract).toMatchObject({
    controlHeight: { sm: 28, default: 32, lg: 40, icon: 32 },
    controlPaddingX: { sm: 8, default: 10, lg: 12, icon: 0 },
    typography: {
      xs: { size: 12, lineHeight: 16 },
      sm: { size: 14, lineHeight: 20 },
      md: { size: 16, lineHeight: 24 },
      lg: { size: 18, lineHeight: 28 },
      xl: { size: 20, lineHeight: 28 },
    },
    controlRadius: 6,
    containerRadius: 8,
    containerPadding: 20,
    sectionGap: 16,
  });
  expect(componentsControlSize("default")).toBe(
    "h-8 px-2.5 gap-2 text-sm rounded-md",
  );
  expect(componentsControlSize("icon")).toContain("w-8 h-8");
});

test("component elevations use restrained native shadows and a themed popup ring", () => {
  const light = componentsElevation("light", "floating");
  const dark = componentsElevation("dark", "floating");

  expect(light).toHaveLength(3);
  expect(light[0]).toMatchObject({ spread: 1, stdDev: 0, color: 0x00000014 });
  expect(dark[0]).toMatchObject({ spread: 1, stdDev: 0, color: 0xffffff1f });
  expect(dark.slice(1)).toEqual(light.slice(1));
  expect(componentsElevation("dark", "modal")).toMatchObject([
    { offsetY: 20, stdDev: 25, spread: -5, color: 0x0000001a },
    { offsetY: 8, stdDev: 10, spread: -6, color: 0x0000001a },
  ]);
});
