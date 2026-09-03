import { afterEach, describe, expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import type { ConfigEnv, Plugin, UserConfig, UserConfigExport } from "vite";
import {
  color,
  defaultWabouColorThemes,
  defineWabouConfig,
  defineWabouTheme,
  hasWabouWorkspaceSources,
  wabouPlugins,
} from "./index";
import { auditColorThemeContrast, compileColorThemes } from "./style-compiler";

describe("@wabou/vite", () => {
  const buildEnvironment = [
    "WABOU_OUT_DIR",
    "WABOU_SOURCE_MAP",
    "WABOU_ENV_DEBUG",
  ] as const;
  const originalEnvironment = Object.fromEntries(
    buildEnvironment.map((name) => [name, process.env[name]]),
  );

  afterEach(() => {
    for (const name of buildEnvironment) {
      const value = originalEnvironment[name];
      if (value === undefined) delete process.env[name];
      else process.env[name] = value;
    }
  });

  async function resolveConfig(
    exported: UserConfigExport,
    command: ConfigEnv["command"] = "build",
  ): Promise<UserConfig> {
    if (typeof exported !== "function") return await exported;
    return await exported({
      command,
      mode: command === "serve" ? "development" : "production",
      isSsrBuild: false,
      isPreview: false,
    });
  }

  test("composes the required Wabou plugins in stable order", () => {
    expect(wabouPlugins("/app").map((plugin: Plugin) => plugin.name)).toEqual([
      "wabou-intl-data",
      "wabou-style-compiler",
      "solid:boundary-modules",
      "solid",
      "wabou-configure-deps-optimizer",
    ]);
  });

  test("ships the semantic color contract required by official UI components", () => {
    const compiled = compileColorThemes(defaultWabouColorThemes);
    expect(compiled?.default).toBe("light");
    for (const theme of Object.values(compiled?.themes ?? {})) {
      expect(Object.keys(theme.colors).sort()).toEqual(
        [
          "accent",
          "accent-hover",
          "accent-pressed",
          "canvas",
          "control",
          "control-hover",
          "control-pressed",
          "danger",
          "danger-hover",
          "danger-pressed",
          "danger-primary",
          "danger-surface",
          "focus",
          "input",
          "muted",
          "on-accent",
          "primary",
          "secondary",
          "selected",
          "strong",
          "subtle",
          "success-primary",
          "success-surface",
          "surface",
          "surface-muted",
        ].sort(),
      );
    }
    expect(auditColorThemeContrast(compiled!)).toEqual([]);
    expect(defaultWabouColorThemes.themes.light.colors.canvas).toBe("#ffffff");
  });

  test("defines and validates typed application themes eagerly", () => {
    const accent = color("#2563eb");
    const theme = defineWabouTheme({
      default: "light",
      themes: {
        light: {
          appearance: "light",
          colors: { canvas: "#ffffff", accent },
        },
        dark: {
          appearance: "dark",
          colors: { canvas: "#121418", accent: "#4c8dff" },
        },
      },
    });

    expect(theme.themes.light.colors.accent).toBe("#2563eb");
    expect(() => color("rgb(1 2 3)")).toThrow(
      "expected #RRGGBB or #RRGGBBAA",
    );
    expect(() =>
      defineWabouTheme({
        default: "missing",
        themes: {
          light: {
            appearance: "light",
            colors: { canvas: "#ffffff" },
          },
        },
      }),
    ).toThrow("does not exist");
  });

  test("defines conventional entry and bundle output", async () => {
    const config = await resolveConfig(
      defineWabouConfig({
        outDir: "/dist/demo/resources",
      }),
    );
    expect(config.build?.outDir).toBe("/dist/demo/resources");
    expect(config.build?.cssCodeSplit).toBe(false);
    expect(config.build?.sourcemap).toBe(false);
    expect(config.build?.minify).toBe("esbuild");
    expect(config.build?.lib).toMatchObject({
      entry: "ui/index.tsx",
      formats: ["iife"],
      name: "WabouApp",
    });
    const aliases = config.resolve?.alias as Record<string, string>;
    const renderer = aliases["@wabou/core/renderer"];
    expect(renderer).toBeString();
    expect(aliases["solid-js/web"]).toBe(renderer);
    expect(renderer).toMatch(/core\/(?:src|dist)\/renderer(?:\.ts|\.mjs)$/);
    expect(renderer).toContain("/packages/core/src/renderer.ts");
    expect(existsSync(renderer)).toBe(true);
    expect(config.resolve?.dedupe).toEqual(["solid-js"]);
    expect(config.resolve?.conditions).toContain("wabou-source");
    expect(config.optimizeDeps).toMatchObject({
      noDiscovery: true,
      include: ["@tanstack/router-core"],
    });
    expect(config.define?.["process.env.NODE_ENV"]).toBe(
      JSON.stringify(process.env.NODE_ENV ?? "production"),
    );
  });

  test("only enables workspace source resolution when source packages exist", async () => {
    expect(hasWabouWorkspaceSources(resolve(import.meta.dir, "../../.."))).toBe(
      true,
    );
    expect(hasWabouWorkspaceSources("/definitely-not-a-wabou-workspace")).toBe(
      false,
    );

    const config = await resolveConfig(
      defineWabouConfig({
        root: "/definitely-not-a-wabou-workspace",
        outDir: "/tmp/wabou-test-output",
      }),
    );
    expect(config.resolve?.conditions).toBeUndefined();
  });

  test("uses the build context supplied by the Wabou CLI", async () => {
    process.env.WABOU_OUT_DIR = "/dist/demo/debug/resources";
    process.env.WABOU_ENV_DEBUG = "true";
    process.env.WABOU_SOURCE_MAP = "true";
    const config = await resolveConfig(
      defineWabouConfig({ outDir: "/ignored" }),
    );
    expect(config.build?.outDir).toBe("/dist/demo/debug/resources");
    expect(config.build?.sourcemap).toBe(true);
    expect(config.build?.minify).toBe(false);
  });

  test("merges app-specific Vite overrides", async () => {
    const extra: Plugin = { name: "app-plugin" };
    const config = await resolveConfig(
      defineWabouConfig({
        outDir: "dist",
        globalName: "Inspector",
        vite: { plugins: [extra], build: { sourcemap: true } },
      }),
    );
    expect(config.build?.sourcemap).toBe(true);
    expect(config.build?.lib).toMatchObject({ name: "Inspector" });
    expect(config.plugins).toContain(extra);
  });

  test("uses Solid development diagnostics while serving", async () => {
    const config = await resolveConfig(
      defineWabouConfig({ outDir: "dist" }),
      "serve",
    );
    expect(config.define?.["process.env.NODE_ENV"]).toBe(
      JSON.stringify(process.env.NODE_ENV ?? "development"),
    );
  });

  test("selects an entry from the Vite mode", async () => {
    const exported = defineWabouConfig(({ mode }) => ({
      entry: mode === "ui-test" ? "ui/fixture.tsx" : undefined,
      outDir: "dist",
    }));
    expect(typeof exported).toBe("function");
    if (typeof exported !== "function")
      throw new Error("expected config factory");
    const config = (await exported({
      command: "build",
      mode: "ui-test",
      isSsrBuild: false,
      isPreview: false,
    })) as UserConfig;
    expect(config.build?.lib).toMatchObject({ entry: "ui/fixture.tsx" });
  });
});
