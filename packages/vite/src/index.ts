import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import solid from "@solidjs/vite-plugin";
import MagicString from "magic-string";
import {
  type ConfigEnv,
  defineConfig as defineViteConfig,
  mergeConfig,
  type Plugin,
  type UserConfig,
  type UserConfigExport,
} from "vite";
import { browserHostPlugin, type WabouBrowserOptions } from "./browser";
import { wabouStylePlugin } from "./style-compiler";
import {
  defaultWabouColorThemes,
  type WabouColorThemeOptions,
} from "./theme-contract.ts";

export type { WabouBrowserOptions } from "./browser";
export {
  color,
  defineWabouTheme,
} from "./style-compiler";
export {
  type DefaultWabouSemanticColorToken,
  defaultWabouColorThemes,
  defaultWabouSemanticColorTokens,
  type WabouColorThemeOptions,
  type WabouThemeColor,
} from "./theme-contract.ts";

export interface WabouViteOptions {
  /** Native QuickJS bundle or browser canvas application. Defaults to native. */
  target?: "native" | "browser";
  /** Browser host options, required for target: browser. */
  browser?: WabouBrowserOptions;
  /** Application root. Defaults to Vite's current working directory. */
  root?: string;
  /** Solid entry module. */
  entry?: string;
  /** Directory for bundle.js and its assets. The CLI supplies its profile directory. */
  outDir?: string;
  /** Generate source maps. Defaults to true in development and false in release builds. */
  sourceMap?: boolean;
  /** IIFE global used by Rollup. */
  globalName?: string;
  /** Additional Vite configuration merged over Wabou defaults. */
  vite?: UserConfig;
  /** Resolve Wabou workspace packages from source. Auto-detected in this repository. */
  workspaceSource?: boolean;
  /** Named semantic color palettes compiled into Wabou Style IR. */
  theme?: WabouColorThemeOptions;
  /** Treat insufficient semantic text contrast as a warning or build error. */
  themeContrast?: "warn" | "error";
  /** Ignore third-party metadata classes. Supports `*` globs. */
  ignoreClasses?: string[];
  /** ECMA-402 locale and time-zone data included in the application bundle. */
  intl?: WabouIntlOptions | false;
}

export interface WabouWindowOptions {
  title?: string;
  width?: number;
  height?: number;
  decorations?: boolean;
}

export interface WabouPackageOptions {
  productName: string;
  identifier: string;
  description?: string;
  authors?: string[];
  copyright?: string;
  licenseFile?: string;
  icons?: string[];
  resources?: string[];
  formats?: ("app" | "dmg" | "nsis" | "wix" | "deb" | "appimage" | "pacman")[];
}

/** Complete application configuration shared by Vite and the Wabou CLI. */
export interface WabouProjectConfig extends WabouViteOptions {
  window?: WabouWindowOptions;
  ui?: { entry?: string };
  package?: WabouPackageOptions;
}

export interface WabouIntlOptions {
  /** APIs to include. Dependencies are included automatically; defaults to all. */
  apis?: ("number" | "date" | "plural")[];
  /** FormatJS locale-data modules. Defaults to English and Chinese. */
  locales?: string[];
  /** Time-zone data set. `golden` is the compact recommended default. */
  timeZones?: "golden" | "all";
}

export type WabouViteOptionsExport =
  | WabouViteOptions
  | ((environment: ConfigEnv) => WabouViteOptions);

export type WabouProjectConfigExport =
  | WabouProjectConfig
  | ((
      environment: ConfigEnv,
    ) => WabouProjectConfig | Promise<WabouProjectConfig>);

const PROJECT_CONFIG = "__wabouConfig";

/** Define the single typed configuration consumed by both Vite and the CLI. */
export function defineConfig(
  options: WabouProjectConfigExport,
): UserConfigExport {
  const viteConfig = defineViteConfig(async (environment) => {
    const resolved =
      typeof options === "function" ? await options(environment) : options;
    return resolveWabouConfig(resolved, environment);
  });
  Object.defineProperty(viteConfig, PROJECT_CONFIG, { value: options });
  return viteConfig;
}

function wabouWorkspaceRoot(start: string): string | undefined {
  let directory = resolve(start);
  for (;;) {
    if (
      existsSync(resolve(directory, "packages/ui/src/index.ts")) &&
      existsSync(resolve(directory, "packages/core/src/index.ts"))
    ) {
      return directory;
    }
    const parent = resolve(directory, "..");
    if (parent === directory) return;
    directory = parent;
  }
}

/** Detect a Wabou source workspace while allowing applications to live below it. */
export function hasWabouWorkspaceSources(start: string): boolean {
  return wabouWorkspaceRoot(start) !== undefined;
}

function configureDependencyOptimizer(): Plugin {
  return {
    name: "wabou-configure-deps-optimizer",
    enforce: "post",
    configResolved(config) {
      if (config.command === "serve") {
        config.optimizeDeps.noDiscovery = true;
        // Automatic discovery can prebundle Solid against the DOM renderer,
        // but explicitly configured dependencies are still valuable. In
        // particular, packages with dense/cyclic ESM graphs (TanStack Router)
        // can overflow QuickJS's recursive module linker when served as many
        // individual modules.
        config.optimizeDeps.include ??= [];
      }
    },
  };
}

/** Plugins required for Solid to target Wabou instead of the browser DOM. */
export function wabouPlugins(
  root = process.cwd(),
  theme?: WabouColorThemeOptions,
  ignoreClasses?: string[],
  intl?: WabouIntlOptions | false,
  entry = "ui/index.tsx",
  themeContrast: "warn" | "error" = "warn",
): Plugin[] {
  return [
    wabouIntlPlugin(root, entry, intl ?? {}),
    wabouStylePlugin({
      root,
      colorThemes: theme ?? defaultWabouColorThemes,
      ignoreClasses,
      themeContrast,
    }),
    ...solid({
      solid: { generate: "universal", moduleName: "@wabou/core/renderer" },
    }),
    configureDependencyOptimizer(),
  ];
}

/** @deprecated Use defineConfig and wabou.config.ts. */
export function defineWabouConfig(
  options: WabouViteOptionsExport,
): UserConfigExport {
  if (typeof options === "function") {
    return defineViteConfig((environment) =>
      resolveWabouConfig(options(environment), environment),
    );
  }
  return defineViteConfig((environment) =>
    resolveWabouConfig(options, environment),
  );
}

function resolveWabouConfig(
  options: WabouProjectConfig,
  environment: ConfigEnv,
): UserConfig {
  const root = options.root ?? process.cwd();
  const entry =
    process.env.WABOU_VITE_ENTRY ??
    options.entry ??
    options.ui?.entry ??
    "ui/index.tsx";
  if (options.target === "browser" && !options.browser)
    throw new Error("target: browser requires browser host options");
  const outDir =
    process.env.WABOU_OUT_DIR ??
    options.outDir ??
    (options.target === "browser" ? "dist" : "dist/resources");
  const sourceMap = process.env.WABOU_SOURCE_MAP;
  const debug =
    process.env.WABOU_ENV_DEBUG === "true" || environment.command === "serve";
  const sourcemap =
    sourceMap === "true"
      ? true
      : sourceMap === "false"
        ? false
        : (options.sourceMap ?? debug);
  const workspaceRoot = wabouWorkspaceRoot(root);
  const workspaceSource =
    options.workspaceSource ?? workspaceRoot !== undefined;
  const renderer =
    workspaceSource && workspaceRoot
      ? resolve(workspaceRoot, "packages/core/src/renderer.ts")
      : fileURLToPath(import.meta.resolve("@wabou/core/renderer"));
  if (options.target === "browser") {
    if (!options.browser)
      throw new Error("target: browser requires browser host options");
    return mergeConfig(
      {
        root,
        plugins: [
          browserHostPlugin(root, entry, options.browser),
          ...wabouPlugins(
            root,
            options.theme,
            options.ignoreClasses,
            false,
            entry,
            options.themeContrast,
          ),
        ],
        resolve: {
          dedupe: ["solid-js"],
          conditions: workspaceSource
            ? ["wabou-source", "browser"]
            : ["browser"],
          alias: { "@wabou/core/renderer": renderer, "solid-js/web": renderer },
        },
        server: { fs: { allow: workspaceRoot ? [workspaceRoot] : [root] } },
        optimizeDeps: { noDiscovery: true },
        build: {
          outDir: options.outDir ?? "dist",
          emptyOutDir: true,
          target: "esnext",
          sourcemap,
        },
      },
      options.vite ?? {},
    );
  }
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
      entry,
      options.themeContrast,
    ),
    resolve: {
      // Published Wabou packages and application source must share one Solid
      // reactive graph. Package-manager store paths otherwise allow Vite to
      // bundle duplicate runtimes that cannot observe each other's signals.
      dedupe: ["solid-js"],
      conditions: workspaceSource ? ["wabou-source"] : undefined,
      alias: {
        "@wabou/core/renderer": renderer,
        "solid-js/web": renderer,
      },
    },
    optimizeDeps: {
      noDiscovery: true,
      include: ["@tanstack/router-core"],
    },
    build: {
      // The native QuickJS host consumes this map to report TS/TSX locations
      // instead of opaque generated bundle offsets.
      sourcemap,
      lib: {
        entry,
        formats: ["iife"],
        name: options.globalName ?? "WabouApp",
        fileName: () => "bundle.js",
      },
      rolldownOptions: {
        output: { assetFileNames: "bundle.[ext]" },
      },
      cssCodeSplit: false,
      outDir,
      emptyOutDir: true,
      // Keep development output readable for mapped QuickJS diagnostics, but
      // do not ship the same multi-megabyte unminified dependency sources in
      // release artifacts. Vite's Oxc minifier preserves the single IIFE
      // consumed by the native runtime.
      minify: debug ? false : "oxc",
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
  options: WabouIntlOptions | false,
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

function intlDataModule(options: WabouIntlOptions | false): string {
  if (options === false) return "export {};";
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
  const apis = new Set(options.apis ?? ["number", "date", "plural"]);
  for (const api of apis) {
    if (!["number", "date", "plural"].includes(api))
      throw new Error(`invalid Wabou Intl API ${api}`);
  }
  if (apis.size === 0) return "export {};";
  const imports = [
    "@formatjs/intl-getcanonicallocales/polyfill.js",
    "@formatjs/intl-locale/polyfill.js",
  ];
  const include = (name: string) => {
    imports.push(
      `@formatjs/intl-${name}/polyfill.js`,
      ...locales.map(
        (locale) => `@formatjs/intl-${name}/locale-data/${locale}.js`,
      ),
    );
  };
  if (apis.has("plural") || apis.has("number") || apis.has("date"))
    include("pluralrules");
  if (apis.has("number") || apis.has("date")) include("numberformat");
  if (apis.has("date")) {
    include("datetimeformat");
    imports.push(
      options.timeZones === "all"
        ? "@formatjs/intl-datetimeformat/add-all-tz.js"
        : "@formatjs/intl-datetimeformat/add-golden-tz.js",
    );
  }
  // A virtual module has no package location of its own, so Vite otherwise
  // resolves these imports from the application root. Resolve them here from
  // @wabou/vite, which owns the dependencies, so isolated workspace installs
  // do not require applications to repeat our implementation dependencies.
  const resolvedImports = imports.map((id) =>
    fileURLToPath(import.meta.resolve(id)),
  );
  return `${resolvedImports.map((id) => `import ${JSON.stringify(id)};`).join("\n")}
${apis.has("date") ? "Intl.DateTimeFormat.__setDefaultTimeZone?.(__wabou_system_time_zone());" : ""}`;
}
