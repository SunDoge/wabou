import { describe, expect, test } from "bun:test";
import type { Plugin, UserConfig } from "vite";
import { defineWabouConfig, wabouPlugins } from "./index";

describe("@wabou/vite", () => {
  test("composes the required Wabou plugins in stable order", () => {
    expect(wabouPlugins("/app").map((plugin: Plugin) => plugin.name)).toEqual([
      "wabou-style-compiler",
      "solid",
      "wabou-disable-solid-deps-optimizer",
    ]);
  });

  test("defines conventional entry and bundle output", () => {
    const config = defineWabouConfig({
      outDir: "/dist/demo/resources",
    }) as UserConfig;
    expect(config.build?.outDir).toBe("/dist/demo/resources");
    expect(config.build?.cssCodeSplit).toBe(false);
    expect(config.build?.lib).toMatchObject({
      entry: "ui/index.tsx",
      formats: ["iife"],
      name: "WabouApp",
    });
    expect(config.resolve?.alias).toMatchObject({
      "solid-js/web": expect.stringContaining("solid-renderer/src/index.ts"),
    });
    expect(config.define?.["process.env.NODE_ENV"]).toBe(
      JSON.stringify(process.env.NODE_ENV ?? "production"),
    );
  });

  test("merges app-specific Vite overrides", () => {
    const extra: Plugin = { name: "app-plugin" };
    const config = defineWabouConfig({
      outDir: "dist",
      globalName: "Inspector",
      vite: { plugins: [extra], build: { sourcemap: true } },
    }) as UserConfig;
    expect(config.build?.sourcemap).toBe(true);
    expect(config.build?.lib).toMatchObject({ name: "Inspector" });
    expect(config.plugins).toContain(extra);
  });
});
