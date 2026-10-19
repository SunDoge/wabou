import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import solid from "@solidjs/vite-plugin";
import { mergeConfig, type UserConfig } from "vite";

export interface WabouTestConfigOptions {
  /** Additional Vitest/Vite configuration merged over Wabou's defaults. */
  vite?: UserConfig;
}

type ComponentTestUserConfig = UserConfig & {
  test?: {
    environment?: string;
    include?: string[];
    setupFiles?: string[];
    server?: { deps?: { inline?: (string | RegExp)[] } };
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
        // Vendored preview releases keep the Wabou repository inside the
        // application tree. Restrict discovery to application component tests
        // so Vitest does not collect framework tests from `vendor/wabou`.
        include: ["ui/**/*.test.{ts,tsx}"],
        server: {
          deps: {
            // Apply the client alias to transitive Solid imports too. Native
            // Node imports select the server runtime and split reactive roots.
            // Wabou's component harness and renderer also keep process-global
            // host glue. Transform them inside each Vitest isolate so a fresh
            // component tree cannot reuse state from another test file.
            inline: ["solid-js", /@solidjs\//, /@wabou\//],
          },
        },
      },
    },
    options.vite ?? {},
  ) as ComponentTestUserConfig;
}
