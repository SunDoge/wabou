import { afterEach, expect, test } from "bun:test";
import { colorTheme } from "./color-theme";

const originalSetColorTheme = globalThis.__wabou_set_color_theme;

afterEach(() => {
  globalThis.__wabou_set_color_theme = originalSetColorTheme;
});

test("selects a native window color theme explicitly", () => {
  const selected: string[] = [];
  globalThis.__wabou_set_color_theme = (name) => selected.push(name);

  colorTheme.set("light");

  expect(selected).toEqual(["light"]);
  expect(colorTheme.current()).toBe("light");
});

test("rejects an empty color theme name before crossing the bridge", () => {
  globalThis.__wabou_set_color_theme = () => {
    throw new Error("bridge should not be called");
  };

  expect(() => colorTheme.set("")).toThrow("cannot be empty");
});
