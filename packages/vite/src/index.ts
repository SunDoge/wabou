import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  type WabouColorThemeOptions,
  wabouStylePlugin,
} from "./style-compiler";
import MagicString from "magic-string";
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
  /** ECMA-402 locale and time-zone data included in the application bundle. */
  intl?: WabouIntlOptions;
}

export interface WabouIntlOptions {
  /** FormatJS locale-data modules. Defaults to English and Chinese. */
  locales?: string[];
  /** Time-zone data set. `golden` is the compact recommended default. */
  timeZones?: "golden" | "all";
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
  intl?: WabouIntlOptions,
  entry = "ui/index.tsx",
): Plugin[] {
  return [
    wabouIntlPlugin(root, entry, intl ?? manifestIntl(root)),
    wabouStylePlugin({ root, colorThemes: theme, ignoreClasses }),
    ...solid({
      solid: { generate: "universal", moduleName: "@wabou/core/renderer" },
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
  const debug =
    process.env.WABOU_ENV_DEBUG === "true" || environment.command === "serve";
  const sourcemap =
    sourceMap === "true"
      ? true
      : sourceMap === "false"
        ? false
        : (manifestSourceMap(root) ?? debug);
  const renderer = fileURLToPath(import.meta.resolve("@wabou/core/renderer"));
  const defaults: UserConfig = {
    define: {
      "process.env.NODE_ENV": JSON.stringify(
        process.env.NODE_ENV ??
          (environment.command === "serve" ? "development" : "production"),
      ),
    },
    plugins: wabouPlugins(
      root,
      options.theme,
      options.ignoreClasses,
      options.intl,
      options.entry ?? "ui/index.tsx",
    ),
    resolve: {
      alias: {
        "@wabou/core/renderer": renderer,
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
      // Keep development output readable for mapped QuickJS diagnostics, but
      // do not ship the same multi-megabyte unminified dependency sources in
      // release artifacts. Vite's esbuild minifier preserves the single IIFE
      // consumed by the native runtime.
      minify: debug ? false : "esbuild",
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

const INTL_DATA_ID = "virtual:wabou-intl-data";
const RESOLVED_INTL_DATA_ID = `\0${INTL_DATA_ID}`;

function wabouIntlPlugin(
  root: string,
  entry: string,
  options: WabouIntlOptions,
): Plugin {
  const entryPath = resolve(root, entry);
  return {
    name: "wabou-intl-data",
    enforce: "pre",
    resolveId(id) {
      return id === INTL_DATA_ID ? RESOLVED_INTL_DATA_ID : undefined;
    },
    load(id) {
      if (id !== RESOLVED_INTL_DATA_ID) return undefined;
      return intlDataModule(options);
    },
    transform(code, id) {
      if (id.split("?", 1)[0] !== entryPath) return undefined;
      const transformed = new MagicString(code);
      transformed.prepend(`import ${JSON.stringify(INTL_DATA_ID)};\n`);
      return {
        code: transformed.toString(),
        map: transformed.generateMap({ hires: true }),
      };
    },
  };
}

function intlDataModule(options: WabouIntlOptions): string {
  const locales = [...new Set(options.locales ?? ["en", "zh"])];
  if (locales.length === 0)
    throw new Error("Wabou intl.locales cannot be empty");
  for (const locale of locales) {
    if (!/^[A-Za-z0-9-]+$/.test(locale)) {
      throw new Error(
        `invalid Wabou Intl locale module ${JSON.stringify(locale)}`,
      );
    }
  }
  const imports = [
    "@formatjs/intl-getcanonicallocales/polyfill.js",
    "@formatjs/intl-locale/polyfill.js",
    "@formatjs/intl-pluralrules/polyfill.js",
    ...locales.map(
      (locale) => `@formatjs/intl-pluralrules/locale-data/${locale}.js`,
    ),
    "@formatjs/intl-numberformat/polyfill.js",
    ...locales.map(
      (locale) => `@formatjs/intl-numberformat/locale-data/${locale}.js`,
    ),
    "@formatjs/intl-datetimeformat/polyfill.js",
    ...locales.map(
      (locale) => `@formatjs/intl-datetimeformat/locale-data/${locale}.js`,
    ),
    options.timeZones === "all"
      ? "@formatjs/intl-datetimeformat/add-all-tz.js"
      : "@formatjs/intl-datetimeformat/add-golden-tz.js",
  ];
  // A virtual module has no package location of its own, so Vite otherwise
  // resolves these imports from the application root. Resolve them here from
  // @wabou/vite, which owns the dependencies, so isolated workspace installs
  // do not require applications to repeat our implementation dependencies.
  const resolvedImports = imports.map((id) =>
    fileURLToPath(import.meta.resolve(id)),
  );
  return `${resolvedImports.map((id) => `import ${JSON.stringify(id)};`).join("\n")}
Intl.DateTimeFormat.__setDefaultTimeZone?.(__wabou_system_time_zone());`;
}

function manifestIntl(root: string): WabouIntlOptions {
  if (!existsSync(resolve(root, "wabou.toml"))) return {};
  const { manifest, path } = readManifest(root);
  const intl = (manifest as { intl?: unknown }).intl;
  if (intl === undefined) return {};
  if (typeof intl !== "object" || intl === null || Array.isArray(intl)) {
    throw new Error(`${path} intl must be a table`);
  }
  const value = intl as { locales?: unknown; "time-zones"?: unknown };
  if (
    value.locales !== undefined &&
    (!Array.isArray(value.locales) ||
      value.locales.some((locale) => typeof locale !== "string"))
  ) {
    throw new Error(`${path} intl.locales must be an array of strings`);
  }
  if (
    value["time-zones"] !== undefined &&
    value["time-zones"] !== "golden" &&
    value["time-zones"] !== "all"
  ) {
    throw new Error(`${path} intl.time-zones must be "golden" or "all"`);
  }
  return {
    locales: value.locales as string[] | undefined,
    timeZones: value["time-zones"] as "golden" | "all" | undefined,
  };
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
