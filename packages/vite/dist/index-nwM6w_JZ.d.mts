import { n as WabouColorThemeOptions, r as WabouThemeColor } from "./theme-contract-CGexNhcT.mjs";
import { ConfigEnv, Plugin, UserConfig, UserConfigExport } from "vite";
//#region src/browser.d.ts
/** Browser-specific host configuration; UI entries keep the normal mount API. */
interface WabouBrowserOptions {
  /** Existing page canvas receiving Wabou paint. */
  canvas: string;
  /** wasm-bindgen's generated web JS module, relative to the application root. */
  runtimeModule: string;
  /** TTF/OTF URLs available to the shared Rust font context. */
  fonts?: readonly string[];
}
//#endregion
//#region src/style-compiler/vite.d.ts
/** Validate a generated or shared theme color at its declaration site. */
declare function color(value: string): WabouThemeColor;
/**
 * Define and eagerly validate a color theme while preserving its concrete
 * theme names and semantic token keys for editor completion.
 */
declare function defineWabouTheme<const T extends WabouColorThemeOptions>(theme: T): T;
//#endregion
//#region src/index.d.ts
interface WabouViteOptions {
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
}
interface WabouWindowOptions {
  title?: string;
  width?: number;
  height?: number;
  decorations?: boolean;
}
interface WabouPackageOptions {
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
interface WabouBindingsOptions {
  /** Generated ambient declaration path, relative to the application root. */
  output?: string;
  /** Rust source files or directories scanned for Wabou JS markers. */
  sources?: string[];
}
/** Complete application configuration shared by Vite and the Wabou CLI. */
interface WabouProjectConfig extends WabouViteOptions {
  window?: WabouWindowOptions;
  ui?: {
    entry?: string;
  };
  package?: WabouPackageOptions;
  bindings?: WabouBindingsOptions;
}
type WabouViteOptionsExport = WabouViteOptions | ((environment: ConfigEnv) => WabouViteOptions);
type WabouProjectConfigExport = WabouProjectConfig | ((environment: ConfigEnv) => WabouProjectConfig | Promise<WabouProjectConfig>);
/** Define the single typed configuration consumed by both Vite and the CLI. */
declare function defineConfig$1(options: WabouProjectConfigExport): UserConfigExport;
/** Detect a Wabou source workspace while allowing applications to live below it. */
declare function hasWabouWorkspaceSources(start: string): boolean;
/** Plugins required for Solid to target Wabou instead of the browser DOM. */
declare function wabouPlugins(root?: string, theme?: WabouColorThemeOptions, ignoreClasses?: string[], themeContrast?: "warn" | "error"): Plugin[];
/** @deprecated Use defineConfig and wabou.config.ts. */
declare function defineWabouConfig(options: WabouViteOptionsExport): UserConfigExport;
//#endregion
export { WabouViteOptions as a, defineConfig$1 as c, wabouPlugins as d, color as f, WabouProjectConfigExport as i, defineWabouConfig as l, WabouBrowserOptions as m, WabouPackageOptions as n, WabouViteOptionsExport as o, defineWabouTheme as p, WabouProjectConfig as r, WabouWindowOptions as s, WabouBindingsOptions as t, hasWabouWorkspaceSources as u };
//# sourceMappingURL=index-nwM6w_JZ.d.mts.map