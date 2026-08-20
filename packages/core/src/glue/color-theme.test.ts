import { afterEach, expect, test } from "bun:test";
import { flush } from "solid-js";
import { colorTheme } from "./color-theme";

const originalSetColorTheme = globalThis.__wabou_set_color_theme;
const originalGetPalette = globalThis.__wabou_get_color_theme_palette;
const originalSetPalette = globalThis.__wabou_set_color_palette;
const originalRaf = globalThis.requestAnimationFrame;
const originalCancelRaf = globalThis.cancelAnimationFrame;

afterEach(() => {
  globalThis.__wabou_set_color_theme = originalSetColorTheme;
  globalThis.__wabou_get_color_theme_palette = originalGetPalette;
  globalThis.__wabou_set_color_palette = originalSetPalette;
  globalThis.requestAnimationFrame = originalRaf;
  globalThis.cancelAnimationFrame = originalCancelRaf;
});

function installPalettes() {
  const palettes: Record<string, number[]> = {
    dark: [0x000000ff, 0xff0000ff],
    light: [0xffffffff, 0x00ff00ff],
  };
  globalThis.__wabou_get_color_theme_palette = (name, output) => {
    const palette = palettes[name];
    if (output) output.set(palette);
    return palette.length;
  };
}

test("selects a native window color theme explicitly", () => {
  installPalettes();
  const selected: string[] = [];
  globalThis.__wabou_set_color_theme = (name) => selected.push(name);

  colorTheme.set("light");
  flush();

  expect(selected).toEqual(["light"]);
  expect(colorTheme.current()).toBe("light");
});

test("rejects an empty color theme name before crossing the bridge", () => {
  globalThis.__wabou_get_color_theme_palette = () => {
    throw new Error("bridge should not be called");
  };

  expect(() => colorTheme.set("")).toThrow("cannot be empty");
});

test("reports a missing compiled theme with configuration guidance", () => {
  globalThis.__wabou_get_color_theme_palette = () => {
    throw new Error("missing theme");
  };

  expect(() => colorTheme.set("midnight")).toThrow(
    "declare it in the `theme.themes` section of vite.config.ts",
  );
});

test("animates a complete palette in JavaScript and commits the named theme", async () => {
  installPalettes();
  globalThis.__wabou_set_color_theme = () => {};
  colorTheme.set("dark");
  flush();
  const frames: Uint32Array[] = [];
  const selected: string[] = [];
  const callbacks: FrameRequestCallback[] = [];
  globalThis.__wabou_set_color_palette = (colors) =>
    frames.push(colors.slice());
  globalThis.__wabou_set_color_theme = (name) => selected.push(name);
  globalThis.requestAnimationFrame = (callback) => {
    callbacks.push(callback);
    return callbacks.length;
  };
  globalThis.cancelAnimationFrame = () => {};

  const animation = colorTheme.animateTo("light", {
    duration: 0.1,
    easing: "linear",
    colorSpace: "srgb",
  });
  callbacks.shift()?.(0);
  callbacks.shift()?.(50);
  callbacks.shift()?.(100);
  await animation.finished;
  flush();

  expect(frames).toHaveLength(3);
  expect(frames[1][0]).toBe(0x808080ff);
  expect(frames[2]).toEqual(Uint32Array.from([0xffffffff, 0x00ff00ff]));
  expect(selected).toEqual(["light"]);
  expect(colorTheme.current()).toBe("light");
});
