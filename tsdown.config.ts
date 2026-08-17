import { defineConfig, type UserConfig } from "tsdown";
import solid from "vite-plugin-solid";

const packages: Record<string, UserConfig["entry"]> = {
  animation: { index: "src/index.ts" },
  components: { index: "src/index.tsx" },
  core: { index: "src/index.ts" },
  interactions: { index: "src/index.ts" },
  primitives: { index: "src/index.ts" },
  protocol: { index: "src/index.ts" },
  router: { index: "src/index.tsx" },
  "solid-renderer": {
    index: "src/index.ts",
    "jsx-runtime": "src/jsx.ts",
  },
  "style-compiler": { index: "src/index.ts" },
  style: { index: "src/index.ts" },
  terminal: { index: "src/index.tsx" },
  test: { index: "src/index.ts" },
  "unocss-preset": { index: "src/index.ts" },
  vite: {
    index: "src/index.ts",
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
    dts: true,
    outExtensions: () => ({ js: ".mjs", dts: ".d.mts" }),
    sourcemap: true,
    clean: true,
    deps: {
      neverBundle: true,
    },
    plugins: [
      ...solid({
        solid: {
          generate: "universal",
          moduleName: "@wabou/solid-renderer",
        },
      }),
    ],
  })),
);
