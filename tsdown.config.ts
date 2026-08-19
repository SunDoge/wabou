import { fileURLToPath } from "node:url";
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
  ui: {
    index: "src/index.ts",
    i18n: "src/i18n.ts",
    primitives: "src/primitives.ts",
    "jsx-runtime": "src/jsx.ts",
  },
  vite: {
    index: "src/index.ts",
    preset: "src/preset/index.ts",
    runtime: "src/runtime/client.ts",
    "utility-manifest": "src/utility-manifest.ts",
  },
};

const uiImplementationPackages = [
  /^@wabou\/animation(?:\/|$)/,
  /^@wabou\/components(?:\/|$)/,
  /^@wabou\/primitives(?:\/|$)/,
  /^@wabou\/router(?:\/|$)/,
];

const uiSourceAliases = {
  "@wabou/animation": fileURLToPath(
    new URL("./packages/animation/src/index.ts", import.meta.url),
  ),
  "@wabou/components": fileURLToPath(
    new URL("./packages/components/src/index.tsx", import.meta.url),
  ),
  "@wabou/primitives/interactions": fileURLToPath(
    new URL("./packages/primitives/src/interactions/index.ts", import.meta.url),
  ),
  "@wabou/primitives": fileURLToPath(
    new URL("./packages/primitives/src/index.ts", import.meta.url),
  ),
  "@wabou/router": fileURLToPath(
    new URL("./packages/router/src/index.tsx", import.meta.url),
  ),
};

// The UI declaration build consumes these aliased workspace sources. Keep
// package builds sequential (see packages:build) so another config cannot
// replace a declaration graph while the facade is bundling it.

export default defineConfig(
  Object.entries(packages).map(([name, entry]) => ({
    name: `@wabou/${name}`,
    cwd: `packages/${name}`,
    entry,
    format: "esm",
    platform: "neutral",
    target: "es2022",
    alias: name === "ui" ? uiSourceAliases : undefined,
    dts: name === "ui" ? { eager: true, newContext: true } : true,
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
        : name === "ui"
          ? {
              alwaysBundle: uiImplementationPackages,
              dts: {
                neverBundle: true,
                alwaysBundle: uiImplementationPackages,
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
