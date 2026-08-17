import { afterEach, describe, expect, test } from "bun:test";
import type { ConfigEnv, Plugin, UserConfig, UserConfigExport } from "vite";
import { defineWabouConfig, wabouPlugins } from "./index";

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
      "wabou-disable-solid-deps-optimizer",
    ]);
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
    expect(config.resolve?.alias).toMatchObject({
      "solid-js/web": expect.stringContaining("solid-renderer"),
    });
    expect(config.define?.["process.env.NODE_ENV"]).toBe(
      JSON.stringify(process.env.NODE_ENV ?? "production"),
    );
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
