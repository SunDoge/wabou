import { readFile } from "node:fs/promises";
import { defineConfig, type UserConfig } from "tsdown";
import solid from "vite-plugin-solid";

const stageName = process.env.WABOU_PACKAGE_STAGE_NAME;

const rawTextPlugin = {
  name: "wabou-raw-text",
  async load(id: string) {
    if (!id.endsWith("?raw")) return undefined;
    const source = await readFile(id.slice(0, -"?raw".length), "utf8");
    return `export default ${JSON.stringify(source)};`;
  },
};

const packages: Record<string, UserConfig["entry"]> = {
  core: {
    index: "src/index.ts",
    protocol: "src/protocol.ts",
    effects: "src/effects.ts",
    registry: "src/registry.ts",
    renderer: "src/renderer.ts",
    testing: "src/testing.ts",
    style: "src/style.ts",
    i18n: "src/i18n.ts",
    "jsx-runtime": "src/jsx.ts",
  },
  terminal: { index: "src/index.tsx" },
  test: {
    index: "src/index.ts",
    component: "src/component.ts",
    layout: "src/layout.ts",
    "layout-node": "src/layout-node.ts",
    "layout-fixtures": "src/layout-fixtures.ts",
  },
  ui: {
    index: "src/index.ts",
    i18n: "src/i18n.ts",
    primitives: "src/primitives/index.ts",
    "jsx-runtime": "src/jsx.ts",
  },
  vite: {
    index: "src/index.ts",
    test: "src/test.ts",
    preset: "src/preset/index.ts",
    runtime: "src/runtime/client.ts",
    "utility-manifest": "src/utility-manifest.ts",
  },
};

export default defineConfig(
  Object.entries(packages).map(([name, entry]) => ({
    name: `@wabou/${name}`,
    cwd: `packages/${name}`,
    entry,
    format: "esm",
    platform: "neutral",
    target: "es2022",
    dts: name === "ui" ? { eager: true, newContext: true } : true,
    outDir: stageName ?? "dist",
    outExtensions: () => ({ js: ".mjs", dts: ".d.mts" }),
    sourcemap: true,
    clean: true,
    deps: {
      neverBundle: true,
      // `?raw` is a build-time contract, not a module specifier that package
      // consumers can execute. Inline UI icon sources while keeping ordinary
      // package dependencies external and tree-shakeable.
      alwaysBundle: name === "ui" ? [/^lucide-static\/icons\//] : undefined,
      onlyBundle: name === "ui" ? ["lucide-static"] : undefined,
    },
    plugins: [
      rawTextPlugin,
      ...solid({
        solid: {
          generate: "universal",
          moduleName: name === "core" ? "./index" : "@wabou/core/renderer",
        },
      }),
    ],
  })),
);
