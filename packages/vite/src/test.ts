import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mergeConfig, type UserConfig } from "vite";
import solid from "vite-plugin-solid";

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
  const renderer = fileURLToPath(import.meta.resolve("@wabou/core/renderer"));
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
          { find: "@wabou/core/renderer", replacement: renderer },
          { find: "solid-js/web", replacement: renderer },
        ],
      },
      test: {
        environment: "node",
      },
    },
    options.vite ?? {},
  ) as ComponentTestUserConfig;
}
