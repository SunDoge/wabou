//#region src/style-compiler/ir.d.ts
declare const STYLE_IR_VERSION: 6;
type Length = {
  unit: "px";
  value: number;
} | {
  unit: "percent";
  value: number;
} | {
  unit: "auto";
};
type ColorValue = {
  kind: "literal";
  rgba: number;
} | {
  kind: "token";
  name: string;
};
type StyleValue = {
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
  value: ColorValue;
} | {
  type: "list";
  values: StyleValue[];
} | {
  type: "record";
  fields: Record<string, StyleValue>;
};
interface StyleDeclaration {
  property: string;
  value: StyleValue;
  important?: true;
}
interface StyleRule {
  className: string;
  declarations: StyleDeclaration[];
  specificity: number;
  sourceOrder: number;
}
interface WabouStyleSheet {
  version: typeof STYLE_IR_VERSION;
  theme: {
    spacing: Record<string, number>;
    colors: Record<string, number>;
  };
  colorThemes?: {
    default: string;
    themes: Record<string, {
      appearance: "light" | "dark";
      colors: Record<string, number>;
    }>;
  };
  diagnostics: string[];
  ignoredClassPatterns: string[];
  rules: StyleRule[];
}
//#endregion
//#region src/style-compiler/browser.d.ts
export declare function assertSupportedWabouCandidates(candidates: Iterable<string>, semanticTokens?: ReadonlySet<string>): void;
export declare function compileWabouUtilities(candidates: Iterable<string>, sourceOrderStart?: number, semanticTokens?: ReadonlySet<string>): StyleRule[];
//#endregion
export type { StyleRule, WabouStyleSheet };
//# sourceMappingURL=browser-style.d.mts.map