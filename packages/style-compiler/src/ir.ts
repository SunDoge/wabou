export const STYLE_IR_VERSION = 5 as const;

export type Length =
  | { unit: "px"; value: number }
  | { unit: "percent"; value: number }
  | { unit: "auto" };

export type ColorValue =
  | { kind: "literal"; rgba: number }
  | { kind: "token"; name: string };

export type StyleValue =
  | { type: "keyword"; value: string }
  | { type: "boolean"; value: boolean }
  | { type: "number"; value: number }
  | { type: "length"; value: Length }
  | { type: "color"; value: ColorValue }
  | { type: "list"; values: StyleValue[] }
  | { type: "record"; fields: Record<string, StyleValue> };

export interface StyleDeclaration {
  property: string;
  value: StyleValue;
  important?: true;
}

export interface StyleRule {
  className: string;
  declarations: StyleDeclaration[];
  specificity: number;
  sourceOrder: number;
}

export interface WabouStyleSheet {
  version: typeof STYLE_IR_VERSION;
  theme: {
    spacing: Record<string, number>;
    colors: Record<string, number>;
  };
  colorThemes?: {
    default: string;
    themes: Record<
      string,
      {
        appearance: "light" | "dark";
        colors: Record<string, number>;
      }
    >;
  };
  diagnostics: string[];
  rules: StyleRule[];
}
