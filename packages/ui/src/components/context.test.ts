import { expect, test } from "bun:test";
import { createComponent, createRoot } from "solid-js";
import {
  ComponentsProvider,
  type ComponentsTheme,
  componentsElevation,
  useComponentsTheme,
} from "./theme";

const resolve = (value: unknown): unknown =>
  typeof value === "function" ? resolve(value()) : value;

test("useComponentsTheme reads the nearest provider and has a stable default", () => {
  expect(useComponentsTheme()()).toBe("dark");
  let received: ComponentsTheme | undefined;

  createRoot((dispose) => {
    resolve(
      createComponent(ComponentsProvider, {
        theme: "light",
        get children() {
          received = useComponentsTheme()();
          return null;
        },
      }),
    );
    dispose();
  });

  expect(received).toBe("light");
});

test("component elevations use GPUI-sized native shadows and a themed popup ring", () => {
  const light = componentsElevation("light", "floating");
  const dark = componentsElevation("dark", "floating");

  expect(light).toHaveLength(3);
  expect(light[0]).toMatchObject({ spread: 1, stdDev: 0, color: 0x0000001a });
  expect(dark[0]).toMatchObject({ spread: 1, stdDev: 0, color: 0xffffff1a });
  expect(dark.slice(1)).toEqual(light.slice(1));
  expect(componentsElevation("dark", "modal")).toMatchObject([
    { offsetY: 20, stdDev: 25, spread: -5, color: 0x0000001a },
    { offsetY: 8, stdDev: 10, spread: -6, color: 0x0000001a },
  ]);
});
