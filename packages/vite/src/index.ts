import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
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
import { parse } from "smol-toml";

export interface WabouViteOptions {
  /** Application root. Defaults to Vite's current working directory. */
  root?: string;
  /** Solid entry module. */
  entry?: string;
  /** Directory for bundle.js and its assets. Defaults to build.out-dir in wabou.toml. */
  outDir?: string;
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
    ...solid({
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
      resolveWabouConfig(options(environment), environment),
    );
  }
  return defineConfig((environment) =>
    resolveWabouConfig(options, environment),
  );
}

function resolveWabouConfig(
  options: WabouViteOptions,
  environment: ConfigEnv,
): UserConfig {
  const root = options.root ?? process.cwd();
  const outDir = options.outDir ?? manifestOutDir(root);
  const renderer = fileURLToPath(import.meta.resolve("@wabou/solid-renderer"));
  const defaults: UserConfig = {
    define: {
      "process.env.NODE_ENV": JSON.stringify(
        process.env.NODE_ENV ??
          (environment.command === "serve" ? "development" : "production"),
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
      // The native QuickJS host consumes this map to report TS/TSX locations
      // instead of opaque generated bundle offsets.
      sourcemap: true,
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
      outDir,
      emptyOutDir: true,
      minify: false,
    },
  };
  return mergeConfig(defaults, options.vite ?? {});
}

function manifestOutDir(root: string): string {
  const path = resolve(root, "wabou.toml");
  let manifest: unknown;
  try {
    manifest = parse(readFileSync(path, "utf8"));
  } catch (error) {
    throw new Error(`cannot read Wabou build output from ${path}: ${error}`);
  }
  const outDir = (manifest as { build?: { "out-dir"?: unknown } }).build?.[
    "out-dir"
  ];
  if (typeof outDir !== "string" || outDir.trim() === "") {
    throw new Error(`${path} must declare a non-empty build.out-dir`);
  }
  return outDir;
}
