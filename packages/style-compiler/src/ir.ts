export const STYLE_IR_VERSION = 3 as const;

export type Length =
  | { unit: "px"; value: number }
  | { unit: "percent"; value: number }
  | { unit: "auto" };

export type ColorValue = { kind: "literal"; rgba: number };

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
  diagnostics: string[];
  rules: StyleRule[];
}
