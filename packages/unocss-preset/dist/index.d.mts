import { Preset } from "@unocss/core";
//#region src/index.d.ts
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
type Manifest = {
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
declare const wabouUtilityManifest: Manifest;
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
/** UnoCSS adapter for editor tooling over the native utility manifest. */
declare function presetWabou(): Preset;
//#endregion
export { ResolvedUtility, UtilityDiagnostic, WabouStyleDeclaration, WabouStyleValue, presetWabou, resolveWabouUtility, validateWabouUtility, wabouUtilityManifest };
//# sourceMappingURL=index.d.mts.map