import { defineConfig, type UserConfig } from "tsdown";
import solid from "vite-plugin-solid";

const packages: Record<string, UserConfig["entry"]> = {
  animation: { index: "src/index.ts" },
  components: { index: "src/index.tsx" },
  core: {
    index: "src/index.ts",
    protocol: "src/protocol.ts",
    registry: "src/registry.ts",
    renderer: "src/renderer.ts",
    style: "src/style.ts",
    i18n: "src/i18n.ts",
    "jsx-runtime": "src/jsx.ts",
  },
  primitives: {
    index: "src/index.ts",
    interactions: "src/interactions/index.ts",
  },
  router: { index: "src/index.tsx" },
  terminal: { index: "src/index.tsx" },
  test: { index: "src/index.ts" },
  vite: {
    index: "src/index.ts",
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
    dts: true,
    outExtensions: () => ({ js: ".mjs", dts: ".d.mts" }),
    sourcemap: true,
    clean: true,
    deps: {
      neverBundle: true,
      ...(name === "core"
        ? {
            alwaysBundle: [
              /^@wabou\/protocol(?:\/|$)/,
              /^@wabou\/solid-renderer(?:\/|$)/,
              /^@wabou\/style(?:\/|$)/,
            ],
            dts: {
              neverBundle: true,
              alwaysBundle: [
                /^@wabou\/protocol(?:\/|$)/,
                /^@wabou\/solid-renderer(?:\/|$)/,
                /^@wabou\/style(?:\/|$)/,
              ],
            },
          }
        : {}),
    },
    plugins: [
      ...solid({
        solid: {
          generate: "universal",
          moduleName: name === "core" ? "./index" : "@wabou/core/renderer",
        },
      }),
    ],
  })),
);
