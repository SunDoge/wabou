import { n as WabouColorThemeOptions } from "./theme-contract-CGexNhcT.mjs";
import { Preset } from "@unocss/core";
//#region src/preset/index.d.ts
type Length = {
  unit: "px" | "percent";
  value: number;
} | {
  unit: "auto";
};
export type WabouStyleValue = {
  type: "keyword";
  value: string;
} | {
  type: "boolean";
  value: boolean;
} | {
  type: "number";
  value: number;
} | {
  type: "length";
  value: Length;
} | {
  type: "color";
  value: {
    kind: "literal";
    rgba: number;
  };
} | {
  type: "list";
  values: WabouStyleValue[];
} | {
  type: "record";
  fields: Record<string, WabouStyleValue>;
};
export type WabouStyleDeclaration = {
  property: string;
  value: WabouStyleValue;
};
type RustParsedUtility = {
  className: string;
  declarations: WabouStyleDeclaration[];
};
export type WabouUtilityManifest = {
  version: number;
  spacing: Record<string, number>;
  colors: Record<string, number>;
  staticUtilities: Record<string, WabouStyleDeclaration[]>;
  dynamicRules: {
    resolver: "spacing" | "dimension" | "color" | "opacity" | "number" | "ratio" | "length" | "translate" | "scale" | "rotate";
    prefixes: {
      name: string;
      properties: string[];
    }[];
  }[];
  conformance: RustParsedUtility[];
};
export declare const wabouUtilityManifest: WabouUtilityManifest;
export type UtilityDiagnostic = {
  candidate: string;
  message: string;
};
export type ResolvedUtility = {
  candidate: string;
  matcher: string;
  declarations: WabouStyleDeclaration[];
};
export declare function resolveWabouUtility(candidate: string): ResolvedUtility | undefined;
export declare function validateWabouUtility(candidate: string): UtilityDiagnostic | undefined;
export interface WabouPresetOptions {
  /** Application theme whose semantic colors should appear in editor tooling. */
  theme?: WabouColorThemeOptions;
  /** Additional semantic colors supplied outside the application theme. */
  semanticColors?: readonly string[];
}
/** UnoCSS adapter for editor tooling over the native utility manifest. */
export declare function presetWabou(options?: WabouPresetOptions): Preset;
//#endregion
export {};
//# sourceMappingURL=preset.d.mts.map