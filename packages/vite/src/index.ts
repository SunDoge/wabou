import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  type WabouColorThemeOptions,
  wabouStylePlugin,
} from "@wabou/style-compiler";
import { parse } from "smol-toml";
import {
  type ConfigEnv,
  defineConfig,
  mergeConfig,
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
  const outDir =
    process.env.WABOU_OUT_DIR ?? options.outDir ?? manifestOutDir(root);
  const sourceMap = process.env.WABOU_SOURCE_MAP;
  const sourcemap =
    sourceMap === "true"
      ? true
      : sourceMap === "false"
        ? false
        : (manifestSourceMap(root) ?? process.env.WABOU_ENV_DEBUG === "true");
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
      sourcemap,
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
  const config = mergeConfig(defaults, options.vite ?? {});
  // The CLI owns the complete application artifact. App-level Vite overrides
  // remain useful for direct builds, but must not redirect or change the
  // profile selected by `wabou build`/`wabou run`.
  if (process.env.WABOU_OUT_DIR !== undefined) {
    config.build = { ...config.build, outDir };
  }
  if (process.env.WABOU_SOURCE_MAP !== undefined) {
    config.build = { ...config.build, sourcemap };
  }
  return config;
}

function manifestOutDir(root: string): string {
  const { manifest, path } = readManifest(root);
  const outDir = (manifest as { build?: { "out-dir"?: unknown } }).build?.[
    "out-dir"
  ];
  if (typeof outDir !== "string" || outDir.trim() === "") {
    throw new Error(`${path} must declare a non-empty build.out-dir`);
  }
  return outDir;
}

function manifestSourceMap(root: string): boolean | undefined {
  if (!existsSync(resolve(root, "wabou.toml"))) return undefined;
  const { manifest, path } = readManifest(root);
  const sourceMap = (manifest as { build?: { "source-map"?: unknown } })
    .build?.["source-map"];
  if (sourceMap === undefined) return undefined;
  if (typeof sourceMap !== "boolean") {
    throw new Error(`${path} build.source-map must be true or false`);
  }
  return sourceMap;
}

function readManifest(root: string): { manifest: unknown; path: string } {
  const path = resolve(root, "wabou.toml");
  try {
    return { manifest: parse(readFileSync(path, "utf8")), path };
  } catch (error) {
    throw new Error(`cannot read Wabou build output from ${path}: ${error}`);
  }
}
