import { i as defineWabouTheme, n as WabouThemeColor, r as color, t as WabouColorThemeOptions } from "./vite-Bt-_UhLO.mjs";
import { ConfigEnv, Plugin, UserConfig, UserConfigExport } from "vite";
//#region src/theme-contract.d.ts
/**
 * Semantic colors guaranteed by Wabou's built-in component theme.
 *
 * Keep this independent from the Vite plugin so editor tooling can load the
 * contract without initializing Vite or the style compiler.
 */
declare const defaultWabouSemanticColorTokens: readonly ["canvas", "surface", "surface-muted", "input", "control", "control-hover", "control-pressed", "selected", "primary", "secondary", "muted", "subtle", "strong", "accent", "accent-hover", "accent-pressed", "on-accent", "danger", "danger-hover", "danger-pressed", "danger-surface", "danger-primary", "success-surface", "success-primary", "focus"];
type DefaultWabouSemanticColorToken = (typeof defaultWabouSemanticColorTokens)[number];
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
export { type DefaultWabouSemanticColorToken, type WabouColorThemeOptions, WabouIntlOptions, type WabouThemeColor, WabouViteOptions, WabouViteOptionsExport, color, defaultWabouColorThemes, defaultWabouSemanticColorTokens, defineWabouConfig, defineWabouTheme, hasWabouWorkspaceSources, wabouPlugins };
//# sourceMappingURL=index.d.mts.map