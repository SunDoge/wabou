import { fileURLToPath } from "node:url";
import {
  wabouStylePlugin,
  type WabouColorThemeOptions,
} from "@wabou/style-compiler";
import {
  defineConfig,
  mergeConfig,
  type ConfigEnv,
  type Plugin,
  type UserConfig,
  type UserConfigExport,
} from "vite";
import solid from "vite-plugin-solid";

export interface WabouViteOptions {
  /** Application root. Defaults to Vite's current working directory. */
  root?: string;
  /** Solid entry module. */
  entry?: string;
  /** Directory for bundle.js and its assets. */
  outDir: string;
  /** IIFE global used by Rollup. */
  globalName?: string;
  /** Additional Vite configuration merged over Wabou defaults. */
  vite?: UserConfig;
  /** Named semantic color palettes compiled into Wabou Style IR. */
  theme?: WabouColorThemeOptions;
  /** Ignore third-party metadata classes. Supports `*` globs. */
  ignoreClasses?: string[];
}

export type WabouViteOptionsExport =
  | WabouViteOptions
  | ((environment: ConfigEnv) => WabouViteOptions);

function disableSolidDependencyOptimizer(): Plugin {
  return {
    name: "wabou-disable-solid-deps-optimizer",
    enforce: "post",
    configResolved(config) {
      if (config.command === "serve") {
        config.optimizeDeps.noDiscovery = true;
        config.optimizeDeps.include = [];
      }
    },
  };
}

/** Plugins required for Solid to target Wabou instead of the browser DOM. */
export function wabouPlugins(
  root = process.cwd(),
  theme?: WabouColorThemeOptions,
  ignoreClasses?: string[],
): Plugin[] {
  return [
    wabouStylePlugin({ root, colorThemes: theme, ignoreClasses }),
    solid({
      solid: { generate: "universal", moduleName: "@wabou/solid-renderer" },
    }),
    disableSolidDependencyOptimizer(),
  ];
}

/** Define the complete conventional Vite configuration for a Wabou app. */
export function defineWabouConfig(
  options: WabouViteOptionsExport,
): UserConfigExport {
  if (typeof options === "function") {
    return defineConfig((environment) =>
      resolveWabouConfig(options(environment)),
    );
  }
  return defineConfig(resolveWabouConfig(options));
}

function resolveWabouConfig(options: WabouViteOptions): UserConfig {
  const root = options.root ?? process.cwd();
  const renderer = fileURLToPath(import.meta.resolve("@wabou/solid-renderer"));
  const defaults: UserConfig = {
    define: {
      "process.env.NODE_ENV": JSON.stringify(
        process.env.NODE_ENV ?? "production",
      ),
    },
    plugins: wabouPlugins(root, options.theme, options.ignoreClasses),
    resolve: {
      alias: {
        "@wabou/solid-renderer": renderer,
        "solid-js/web": renderer,
      },
    },
    build: {
      lib: {
        entry: options.entry ?? "ui/index.tsx",
        formats: ["iife"],
        name: options.globalName ?? "WabouApp",
        fileName: () => "bundle.js",
      },
      rollupOptions: {
        output: { inlineDynamicImports: true, assetFileNames: "bundle.[ext]" },
      },
      cssCodeSplit: false,
      outDir: options.outDir,
      emptyOutDir: true,
      minify: false,
    },
  };
  return mergeConfig(defaults, options.vite ?? {});
}
