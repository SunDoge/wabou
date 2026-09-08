import { n as WabouColorThemeOptions } from "./theme-contract-BXj8mg9N.mjs";
import { Preset } from "@unocss/core";
//#region src/preset/index.d.ts
type Length = {
  unit: "px" | "percent";
  value: number;
} | {
  unit: "auto";
};
type WabouStyleValue = {
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
type WabouStyleDeclaration = {
  property: string;
  value: WabouStyleValue;
};
type RustParsedUtility = {
  className: string;
  declarations: WabouStyleDeclaration[];
};
type WabouUtilityManifest = {
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
declare const wabouUtilityManifest: WabouUtilityManifest;
type UtilityDiagnostic = {
  candidate: string;
  message: string;
};
type ResolvedUtility = {
  candidate: string;
  matcher: string;
  declarations: WabouStyleDeclaration[];
};
declare function resolveWabouUtility(candidate: string): ResolvedUtility | undefined;
declare function validateWabouUtility(candidate: string): UtilityDiagnostic | undefined;
interface WabouPresetOptions {
  /** Application theme whose semantic colors should appear in editor tooling. */
  theme?: WabouColorThemeOptions;
  /** Additional semantic colors supplied outside the application theme. */
  semanticColors?: readonly string[];
}
/** UnoCSS adapter for editor tooling over the native utility manifest. */
declare function presetWabou(options?: WabouPresetOptions): Preset;
//#endregion
export { ResolvedUtility, UtilityDiagnostic, WabouPresetOptions, WabouStyleDeclaration, WabouStyleValue, WabouUtilityManifest, presetWabou, resolveWabouUtility, validateWabouUtility, wabouUtilityManifest };
//# sourceMappingURL=preset.d.mts.map