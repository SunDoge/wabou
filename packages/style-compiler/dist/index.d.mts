import { ModuleNode, Plugin } from "vite";
//#region src/ir.d.ts
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
//#region src/support-matrix.d.ts
/**
 * CSS support matrix — compiler ↔ host contract.
 *
 * - **supported**: compile to Style IR and apply in Rust `apply_ir`.
 * - **unsupported**: compile-time error (never emit IR).
 *
 * Source of truth: `../css-support-matrix.json` (also `include_str!`'d by
 * wabou-runtime tests so Rust cannot drift).
 */
type SupportKind = "supported" | "unsupported";
type SampleSpec = string | {
  sample: string;
  rustOnly?: boolean;
};
type CssSupportMatrix = {
  version: number;
  description: string;
  supported: Record<string, SampleSpec>;
  unsupported: Record<string, string>;
  unsupportedPrefixes: Record<string, string>;
};
declare const CSS_SUPPORT_MATRIX: CssSupportMatrix;
/** Every property name Rust `apply_ir` must accept. */
declare function allHostProperties(): string[];
/** Properties the compiler is allowed to emit (excludes rust-only aliases). */
declare function allCompilerProperties(): string[];
declare function propertySample(property: string): string | undefined;
/**
 * Classify a property for the compiler. Returns a human-readable reject
 * message when the property must not enter Style IR.
 */
declare function rejectUnsupportedProperty(property: string): string | undefined;
declare function supportKind(property: string): SupportKind | "unknown";
//#endregion
//#region src/vite.d.ts
interface WabouStylePluginOptions {
  root: string;
  colorThemes?: WabouColorThemeOptions;
  /** Metadata classes that are not Wabou utilities. Supports `*` globs. */
  ignoreClasses?: string[];
}
declare function matchesClassPattern(candidate: string, pattern: string): boolean;
declare function filterIgnoredClasses(candidates: Iterable<string>, patterns?: readonly string[]): string[];
interface WabouColorThemeOptions {
  default: string;
  themes: Record<string, {
    appearance: "light" | "dark";
    colors: Record<string, string>;
  }>;
}
type CompiledColorThemes = NonNullable<WabouStyleSheet["colorThemes"]>;
declare function compileColorThemes(options?: WabouColorThemeOptions): CompiledColorThemes | undefined;
declare function assertSupportedWabouCandidates(candidates: Iterable<string>, semanticTokens?: ReadonlySet<string>): void;
declare function compileWabouUtilities(candidates: Iterable<string>, sourceOrderStart?: number, semanticTokens?: ReadonlySet<string>): StyleRule[];
/**
 * Keep UnoCSS candidates scoped to JSX class props.
 *
 * Uno's default extractor scans every token, which turns values like
 * `role="tab"` or terminal command strings into accidental utilities.
 */
declare function extractUtilitySource(source: string): string;
/** Conventional Wabou source roots that may contain utility classes. */
declare function wabouSourceDirectories(root: string): string[];
declare function findWorkspacePackages(root: string): Promise<string | undefined>;
declare function wabouHotUpdateModules(modules: ModuleNode[], stylesheetModule: ModuleNode): ModuleNode[];
declare function wabouStylePlugin(options: WabouStylePluginOptions): Plugin;
//#endregion
export { CSS_SUPPORT_MATRIX, ColorValue, CssSupportMatrix, Length, STYLE_IR_VERSION, StyleDeclaration, StyleRule, StyleValue, SupportKind, WabouColorThemeOptions, WabouStylePluginOptions, WabouStyleSheet, allCompilerProperties, allHostProperties, assertSupportedWabouCandidates, compileColorThemes, compileWabouUtilities, extractUtilitySource, filterIgnoredClasses, findWorkspacePackages, matchesClassPattern, propertySample, rejectUnsupportedProperty, supportKind, wabouHotUpdateModules, wabouSourceDirectories, wabouStylePlugin };
//# sourceMappingURL=index.d.mts.map