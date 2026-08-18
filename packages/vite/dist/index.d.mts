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
  /** Named semantic color palettes compiled into Wabou Style IR. */
  theme?: WabouColorThemeOptions;
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
type WabouViteOptionsExport = WabouViteOptions | ((environment: ConfigEnv) => WabouViteOptions);
/** Plugins required for Solid to target Wabou instead of the browser DOM. */
declare function wabouPlugins(root?: string, theme?: WabouColorThemeOptions, ignoreClasses?: string[], intl?: WabouIntlOptions, entry?: string): Plugin[];
/** Define the complete conventional Vite configuration for a Wabou app. */
declare function defineWabouConfig(options: WabouViteOptionsExport): UserConfigExport;
//#endregion
export { WabouIntlOptions, WabouViteOptions, WabouViteOptionsExport, defineWabouConfig, wabouPlugins };
//# sourceMappingURL=index.d.mts.map