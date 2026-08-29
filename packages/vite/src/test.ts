import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mergeConfig, type UserConfig } from "vite";
import solid from "@solidjs/vite-plugin";

export interface WabouTestConfigOptions {
  /** Additional Vitest/Vite configuration merged over Wabou's defaults. */
  vite?: UserConfig;
}

type ComponentTestUserConfig = UserConfig & {
  test?: {
    environment?: string;
    setupFiles?: string[];
  };
};

/** Configure Vitest to compile Wabou TSX through Solid's universal renderer. */
export function defineWabouTestConfig(
  options: WabouTestConfigOptions = {},
): ComponentTestUserConfig {
  const core = fileURLToPath(import.meta.resolve("@wabou/core"));
  const renderer = fileURLToPath(import.meta.resolve("@wabou/core/renderer"));
  const testing = fileURLToPath(import.meta.resolve("@wabou/core/testing"));
  const solidEntry = `${dirname(fileURLToPath(import.meta.resolve("solid-js/package.json")))}/dist/solid.js`;
  return mergeConfig(
    {
      plugins: solid({
        solid: {
          generate: "universal",
          moduleName: "@wabou/core/renderer",
        },
      }),
      resolve: {
        conditions: ["browser", "wabou-source"],
        dedupe: ["solid-js"],
        alias: [
          { find: /^solid-js$/, replacement: solidEntry },
          // The root entry and renderer must share one HostContext. Resolving
          // only the renderer to its built entry while `wabou-source` resolves
          // the root entry to source creates two module instances.
          { find: /^@wabou\/core$/, replacement: core },
          { find: /^@wabou\/core\/renderer$/, replacement: renderer },
          // Test adapters dispatch into the same host-message registry used by
          // components. A source-resolved subpath would create an isolated
          // registry and make native observations disappear in tests.
          { find: /^@wabou\/core\/testing$/, replacement: testing },
          { find: /^solid-js\/web$/, replacement: renderer },
        ],
      },
      test: {
        environment: "node",
      },
    },
    options.vite ?? {},
  ) as ComponentTestUserConfig;
}
