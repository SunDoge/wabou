import { WabouColorThemeOptions } from "@wabou/style-compiler";
import { ConfigEnv, Plugin, UserConfig, UserConfigExport } from "vite";
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
}
type WabouViteOptionsExport = WabouViteOptions | ((environment: ConfigEnv) => WabouViteOptions);
/** Plugins required for Solid to target Wabou instead of the browser DOM. */
declare function wabouPlugins(root?: string, theme?: WabouColorThemeOptions, ignoreClasses?: string[]): Plugin[];
/** Define the complete conventional Vite configuration for a Wabou app. */
declare function defineWabouConfig(options: WabouViteOptionsExport): UserConfigExport;
//#endregion
export { WabouViteOptions, WabouViteOptionsExport, defineWabouConfig, wabouPlugins };
//# sourceMappingURL=index.d.mts.map