import { ConfigEnv, Plugin, UserConfig, UserConfigExport } from "vite";
//#region src/style-compiler/vite.d.ts
interface WabouColorThemeOptions {
  default: string;
  themes: Record<string, {
    appearance: "light" | "dark";
    colors: Record<string, string>;
  }>;
}
//#endregion
//#region src/index.d.ts
interface WabouViteOptions {
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
  /** Resolve Wabou workspace packages from source. Auto-detected in this repository. */
  workspaceSource?: boolean;
  /** Named semantic color palettes compiled into Wabou Style IR. */
  theme?: WabouColorThemeOptions;
  /** Treat insufficient semantic text contrast as a warning or build error. */
  themeContrast?: "warn" | "error";
  /** Ignore third-party metadata classes. Supports `*` globs. */
  ignoreClasses?: string[];
  /** ECMA-402 locale and time-zone data included in the application bundle. */
  intl?: WabouIntlOptions;
}
interface WabouIntlOptions {
  /** FormatJS locale-data modules. Defaults to English and Chinese. */
  locales?: string[];
  /** Time-zone data set. `golden` is the compact recommended default. */
  timeZones?: "golden" | "all";
}
/**
 * Semantic colors used by `@wabou/ui` when an application does not provide a
 * theme. Keeping this at the Vite boundary means every official component is
 * usable in a minimal project while applications can still replace the whole
 * token contract explicitly.
 */
declare const defaultWabouColorThemes: WabouColorThemeOptions;
type WabouViteOptionsExport = WabouViteOptions | ((environment: ConfigEnv) => WabouViteOptions);
/** Detect a Wabou source workspace while allowing applications to live below it. */
declare function hasWabouWorkspaceSources(start: string): boolean;
/** Plugins required for Solid to target Wabou instead of the browser DOM. */
declare function wabouPlugins(root?: string, theme?: WabouColorThemeOptions, ignoreClasses?: string[], intl?: WabouIntlOptions, entry?: string, themeContrast?: "warn" | "error"): Plugin[];
/** Define the complete conventional Vite configuration for a Wabou app. */
declare function defineWabouConfig(options: WabouViteOptionsExport): UserConfigExport;
//#endregion
export { WabouIntlOptions, WabouViteOptions, WabouViteOptionsExport, defaultWabouColorThemes, defineWabouConfig, hasWabouWorkspaceSources, wabouPlugins };
//# sourceMappingURL=index.d.mts.map