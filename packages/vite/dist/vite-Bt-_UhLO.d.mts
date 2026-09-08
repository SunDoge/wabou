import { Plugin } from "vite";
//#region src/style-compiler/vite.d.ts
/** A build-time sRGB color accepted by the Wabou theme compiler. */
type WabouThemeColor = `#${string}`;
interface WabouColorThemeOptions {
  default: string;
  themes: Record<string, {
    appearance: "light" | "dark";
    colors: Record<string, WabouThemeColor>;
  }>;
}
/** Validate a generated or shared theme color at its declaration site. */
declare function color(value: string): WabouThemeColor;
/**
 * Define and eagerly validate a color theme while preserving its concrete
 * theme names and semantic token keys for editor completion.
 */
declare function defineWabouTheme<const T extends WabouColorThemeOptions>(theme: T): T;
//#endregion
export { defineWabouTheme as i, WabouThemeColor as n, color as r, WabouColorThemeOptions as t };
//# sourceMappingURL=vite-Bt-_UhLO.d.mts.map