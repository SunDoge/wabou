import type { Dirent } from "node:fs";
import { readdir, readFile } from "node:fs/promises";
import { dirname, join, parse, sep } from "node:path";
import { createGenerator } from "@unocss/core";
import type { ModuleNode, Plugin } from "vite";
import {
  presetWabou,
  resolveWabouUtility,
  validateWabouUtility,
  wabouUtilityManifest,
} from "../preset";
import {
  STYLE_IR_VERSION,
  type StyleRule,
  type WabouStyleSheet,
} from "./ir.ts";

const DEFAULT_IGNORED_CLASS_PATTERNS = ["lucide", "lucide-*"];

export interface WabouStylePluginOptions {
  root: string;
  colorThemes?: WabouColorThemeOptions;
  /** How semantic text contrast violations are reported. Defaults to `warn`. */
  themeContrast?: "warn" | "error";
  /** Metadata classes that are not Wabou utilities. Supports `*` globs. */
  ignoreClasses?: string[];
}

export function matchesClassPattern(
  candidate: string,
  pattern: string,
): boolean {
  const escaped = pattern.replace(/[.+?^${}()|[\]\\]/g, "\\$&");
  return new RegExp(`^${escaped.replaceAll("*", ".*")}$`).test(candidate);
}

export function filterIgnoredClasses(
  candidates: Iterable<string>,
  patterns: readonly string[] = [],
): string[] {
  return [...candidates].filter(
    (candidate) =>
      !patterns.some((pattern) => matchesClassPattern(candidate, pattern)),
  );
}

export interface WabouColorThemeOptions {
  default: string;
  themes: Record<
    string,
    {
      appearance: "light" | "dark";
      colors: Record<string, string>;
    }
  >;
}

type CompiledColorThemes = NonNullable<WabouStyleSheet["colorThemes"]>;

export interface ColorContrastDiagnostic {
  theme: string;
  foreground: string;
  background: string;
  ratio: number;
  minimum: number;
  /** Nearest opaque sRGB foreground, along a black/white mixing path, that passes. */
  suggestedColor?: string;
}

const TEXT_CONTRAST_PAIRS = [
  ["primary", "canvas"],
  ["primary", "surface"],
  ["secondary", "canvas"],
  ["secondary", "surface"],
  ["muted", "canvas"],
  ["muted", "surface"],
  ["on-accent", "accent"],
  ["danger-primary", "danger-surface"],
  ["success-primary", "success-surface"],
] as const;

function parseThemeColor(value: string, theme: string, token: string): number {
  const match = value.match(/^#([0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/);
  if (!match)
    throw new Error(
      `invalid color theme value for ${theme}.${token}; expected #RRGGBB or #RRGGBBAA`,
    );
  const hex = match[1];
  const parsed = Number.parseInt(hex, 16);
  return hex.length === 6 ? ((parsed << 8) | 0xff) >>> 0 : parsed >>> 0;
}

function colorChannels(rgba: number): [number, number, number, number] {
  return [
    (rgba >>> 24) / 255,
    ((rgba >>> 16) & 0xff) / 255,
    ((rgba >>> 8) & 0xff) / 255,
    (rgba & 0xff) / 255,
  ];
}

function relativeLuminance([red, green, blue]: readonly number[]): number {
  const linear = [red, green, blue].map((channel) =>
    channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4,
  );
  return 0.2126 * linear[0]! + 0.7152 * linear[1]! + 0.0722 * linear[2]!;
}

function colorContrastRatio(
  foreground: number,
  background: number,
): number | undefined {
  const [fr, fg, fb, fa] = colorChannels(foreground);
  const [br, bg, bb, ba] = colorChannels(background);
  if (ba !== 1) return;
  const composed = [
    fr * fa + br * (1 - fa),
    fg * fa + bg * (1 - fa),
    fb * fa + bb * (1 - fa),
  ];
  const foregroundLuminance = relativeLuminance(composed);
  const backgroundLuminance = relativeLuminance([br, bg, bb]);
  return (
    (Math.max(foregroundLuminance, backgroundLuminance) + 0.05) /
    (Math.min(foregroundLuminance, backgroundLuminance) + 0.05)
  );
}

function opaqueColor(red: number, green: number, blue: number): number {
  return (
    ((Math.round(red * 255) << 24) |
      (Math.round(green * 255) << 16) |
      (Math.round(blue * 255) << 8) |
      0xff) >>> 0
  );
}

function formatOpaqueColor(color: number): string {
  return `#${(color >>> 8).toString(16).padStart(6, "0")}`;
}

function passingForegroundSuggestion(
  foreground: number,
  background: number,
  minimum: number,
): string | undefined {
  const [red, green, blue] = colorChannels(foreground);
  const candidates: { color: number; distance: number }[] = [];
  for (const target of [0, 1]) {
    const endpoint = opaqueColor(target, target, target);
    if ((colorContrastRatio(endpoint, background) ?? 0) < minimum) continue;
    let low = 0;
    let high = 1;
    for (let index = 0; index < 24; index++) {
      const amount = (low + high) / 2;
      const candidate = opaqueColor(
        red + (target - red) * amount,
        green + (target - green) * amount,
        blue + (target - blue) * amount,
      );
      if ((colorContrastRatio(candidate, background) ?? 0) >= minimum)
        high = amount;
      else low = amount;
    }
    const color = opaqueColor(
      red + (target - red) * high,
      green + (target - green) * high,
      blue + (target - blue) * high,
    );
    candidates.push({ color, distance: high });
  }
  candidates.sort((left, right) => left.distance - right.distance);
  return candidates[0] ? formatOpaqueColor(candidates[0].color) : undefined;
}

/** Audit semantic text pairs that official components render as normal-sized text. */
export function auditColorThemeContrast(
  themes?: CompiledColorThemes,
  minimum = 4.5,
): ColorContrastDiagnostic[] {
  if (!themes) return [];
  const diagnostics: ColorContrastDiagnostic[] = [];
  for (const [theme, definition] of Object.entries(themes.themes)) {
    for (const [foreground, background] of TEXT_CONTRAST_PAIRS) {
      const foregroundColor = definition.colors[foreground];
      const backgroundColor = definition.colors[background];
      if (foregroundColor === undefined || backgroundColor === undefined)
        continue;
      const ratio = colorContrastRatio(foregroundColor, backgroundColor);
      if (ratio !== undefined && ratio < minimum) {
        diagnostics.push({
          theme,
          foreground,
          background,
          ratio,
          minimum,
          suggestedColor: passingForegroundSuggestion(
            foregroundColor,
            backgroundColor,
            minimum,
          ),
        });
      }
    }
  }
  return diagnostics;
}

export function compileColorThemes(
  options?: WabouColorThemeOptions,
): CompiledColorThemes | undefined {
  if (!options) return;
  const base = options.themes[options.default];
  if (!base)
    throw new Error(
      `default Wabou color theme \`${options.default}\` does not exist`,
    );
  const tokens = Object.keys(base.colors).sort();
  if (!tokens.length)
    throw new Error("Wabou color themes require at least one token");
  for (const token of tokens) {
    if (!/^[a-z][a-z0-9-]*$/.test(token))
      throw new Error(`invalid Wabou color token \`${token}\``);
    if (token in wabouUtilityManifest.colors)
      throw new Error(
        `Wabou color token \`${token}\` conflicts with a palette color`,
      );
  }
  const themes: CompiledColorThemes["themes"] = {};
  for (const [name, theme] of Object.entries(options.themes)) {
    const actual = Object.keys(theme.colors).sort();
    const missing = tokens.filter((token) => !(token in theme.colors));
    const unknown = actual.filter((token) => !tokens.includes(token));
    if (missing.length || unknown.length) {
      throw new Error(
        `Wabou color theme \`${name}\` does not match \`${options.default}\`` +
          `${missing.length ? `; missing: ${missing.join(", ")}` : ""}` +
          `${unknown.length ? `; unknown: ${unknown.join(", ")}` : ""}`,
      );
    }
    themes[name] = {
      appearance: theme.appearance,
      colors: Object.fromEntries(
        tokens.map((token) => [
          token,
          parseThemeColor(theme.colors[token], name, token),
        ]),
      ),
    };
  }
  return { default: options.default, themes };
}

function semanticColorDeclaration(
  candidate: string,
  tokens: ReadonlySet<string>,
): StyleRule["declarations"][number] | undefined {
  const match = candidate.match(/^(bg|text|border)-(.+)$/);
  if (!match || !tokens.has(match[2])) return;
  return {
    property:
      match[1] === "bg"
        ? "background-color"
        : match[1] === "text"
          ? "color"
          : "border-color",
    value: { type: "color", value: { kind: "token", name: match[2] } },
  };
}

export function assertSupportedWabouCandidates(
  candidates: Iterable<string>,
  semanticTokens: ReadonlySet<string> = new Set(),
): void {
  const unsupported = [...candidates]
    .filter((candidate) => !semanticColorDeclaration(candidate, semanticTokens))
    .map((candidate) => validateWabouUtility(candidate))
    .filter((diagnostic) => diagnostic !== undefined);
  if (unsupported.length) {
    throw new Error(
      `unsupported Wabou utilities:\n${unsupported
        .map(({ message }) => `  - ${message}`)
        .join("\n")}`,
    );
  }
}

export function compileWabouUtilities(
  candidates: Iterable<string>,
  sourceOrderStart = 0,
  semanticTokens: ReadonlySet<string> = new Set(),
): StyleRule[] {
  const ordered = [...candidates].sort();
  assertSupportedWabouCandidates(ordered, semanticTokens);
  return ordered.map((candidate, index) => {
    const semantic = semanticColorDeclaration(candidate, semanticTokens);
    if (semantic) {
      return {
        className: candidate,
        specificity: 10,
        sourceOrder: sourceOrderStart + index,
        declarations: [semantic],
      };
    }
    const utility = resolveWabouUtility(candidate);
    if (!utility) throw new Error(`unsupported Wabou utility \`${candidate}\``);
    return {
      className: candidate,
      specificity: 10,
      sourceOrder: sourceOrderStart + index,
      declarations: utility.declarations,
    };
  });
}

/**
 * Keep UnoCSS candidates scoped to JSX class props.
 *
 * Uno's default extractor scans every token, which turns values like
 * `role="tab"` or terminal command strings into accidental utilities.
 */
export function extractUtilitySource(source: string): string {
  const values: string[] = [];
  const pushValue = (value: string, expression = false) => {
    if (expression) {
      value = value.replace(
        /(?:===|!==|==|!=)\s*(?:"[^"]*"|'[^']*'|`[^`]*`)/g,
        "",
      );
    }
    // Reactive selection between complete static utilities is valid, but
    // manufacturing utility names from runtime fragments bypasses build-time
    // validation and theme/conformance guarantees.
    const interpolations = [...value.matchAll(/\$\{([^}]+)\}/g)];
    const selectsCompleteUtilities = (code: string) =>
      /^\s*[\s\S]+?\?\s*(?:"[^"]*"|'[^']*'|`[^`]*`)\s*:\s*(?:"[^"]*"|'[^']*'|`[^`]*`)\s*$/.test(
        code,
      );
    const constructsClass =
      interpolations.some((match) => !selectsCompleteUtilities(match[1])) ||
      (expression && /(?:["'`]\s*\+|\+\s*["'`])/.test(value));
    if (constructsClass) {
      throw new Error(
        "dynamic class construction is not supported; select complete static utilities with classList and put continuous values in typed style",
      );
    }
    values.push(value);
  };
  const classProp = /\bclass(?:Name)?\s*=\s*(?:"([^"]*)"|'([^']*)'|`([^`]*)`)/g;
  for (const match of source.matchAll(classProp)) {
    pushValue(match[1] ?? match[2] ?? match[3] ?? "");
  }
  const expressionStart = /\bclass(?:Name)?\s*=\s*\{/g;
  for (const match of source.matchAll(expressionStart)) {
    const start = (match.index ?? 0) + match[0].length;
    let depth = 1;
    let quote = "";
    let escaped = false;
    let end = start;
    for (; end < source.length && depth > 0; end++) {
      const character = source[end];
      if (escaped) {
        escaped = false;
        continue;
      }
      if (character === "\\") {
        escaped = true;
        continue;
      }
      if (quote) {
        if (character === quote) quote = "";
        continue;
      }
      if (character === '"' || character === "'" || character === "`") {
        quote = character;
      } else if (character === "{") {
        depth++;
      } else if (character === "}") {
        depth--;
      }
    }
    pushValue(source.slice(start, depth === 0 ? end - 1 : end), true);
  }
  const classListProp = /\bclassList\s*=\s*\{\{([\s\S]*?)\}\}/g;
  for (const match of source.matchAll(classListProp)) {
    const entries = match[1];
    const key = /(?:^|,)\s*(?:"([^"]+)"|'([^']+)'|([A-Za-z_][\w-]*))\s*:/g;
    for (const candidate of entries.matchAll(key)) {
      pushValue(candidate[1] ?? candidate[2] ?? candidate[3]);
    }
  }
  return values.join("\n");
}

/** Conventional Wabou source roots that may contain utility classes. */
export function wabouSourceDirectories(root: string): string[] {
  return ["src", "ui", "packages"].map((directory) => join(root, directory));
}

export async function findWorkspacePackages(
  root: string,
): Promise<string | undefined> {
  let directory = root;
  for (;;) {
    try {
      const manifest = JSON.parse(
        await readFile(join(directory, "package.json"), "utf8"),
      );
      if (Array.isArray(manifest.workspaces))
        return join(directory, "packages");
    } catch {
      // Keep walking: application directories need not contain package.json.
    }
    const parent = dirname(directory);
    if (parent === directory || directory === parse(directory).root) return;
    directory = parent;
  }
}

export function wabouHotUpdateModules(
  modules: ModuleNode[],
  stylesheetModule: ModuleNode,
): ModuleNode[] {
  return [...new Set([...modules, stylesheetModule])];
}

export function wabouStylePlugin(options: WabouStylePluginOptions): Plugin {
  let referenceGenerator: Awaited<ReturnType<typeof createGenerator>>;
  const sources = new Map<string, string>();
  const sourceRoots = new Set([options.root]);
  const colorThemes = compileColorThemes(options.colorThemes);
  const contrastDiagnostics = auditColorThemeContrast(colorThemes);
  const semanticTokens = new Set(
    Object.keys(colorThemes?.themes[colorThemes.default]?.colors ?? {}),
  );
  const ignoredClassPatterns = [
    ...new Set([
      ...DEFAULT_IGNORED_CLASS_PATTERNS,
      ...(options.ignoreClasses ?? []),
    ]),
  ];
  let stylesheet: WabouStyleSheet = {
    version: STYLE_IR_VERSION,
    theme: {
      spacing: wabouUtilityManifest.spacing,
      colors: wabouUtilityManifest.colors,
    },
    colorThemes,
    diagnostics: [],
    ignoredClassPatterns,
    rules: [],
  };
  const virtual = "virtual:wabou-stylesheet";
  const resolved = `\0${virtual}`;

  async function regenerate() {
    if (!referenceGenerator) return;
    const utilitySource = [...sources.values()].join("\n");
    const reference = await referenceGenerator.generate(utilitySource, {
      preflights: false,
    });
    const matched = filterIgnoredClasses(
      reference.matched,
      ignoredClassPatterns,
    );
    assertSupportedWabouCandidates(matched, semanticTokens);
    stylesheet = {
      version: STYLE_IR_VERSION,
      theme: {
        spacing: wabouUtilityManifest.spacing,
        colors: wabouUtilityManifest.colors,
      },
      colorThemes,
      diagnostics: [],
      ignoredClassPatterns,
      rules: compileWabouUtilities(matched, 0, semanticTokens),
    };
  }

  function accepts(id: string): boolean {
    const path = id.split("?", 1)[0];
    if (
      path.includes("node_modules") ||
      /\.(?:test|spec)\.(?:tsx?|jsx?)$/.test(path) ||
      ![...sourceRoots].some(
        (root) => path === root || path.startsWith(`${root}${sep}`),
      )
    )
      return false;
    if (/\.css$/.test(path)) {
      throw new Error(
        "CSS stylesheets are not supported; use static utility classes and typed style values",
      );
    }
    return /\.(tsx|ts|jsx|js)$/.test(path);
  }

  async function scan(directory: string): Promise<void> {
    let entries: Dirent[];
    try {
      entries = await readdir(directory, { withFileTypes: true });
    } catch {
      return;
    }
    await Promise.all(
      entries.map(async (entry) => {
        const path = join(directory, entry.name);
        if (entry.isDirectory()) {
          if (!["dist", "node_modules"].includes(entry.name)) await scan(path);
          return;
        }
        if (accepts(path)) {
          const contents = await readFile(path, "utf8");
          sources.set(path, extractUtilitySource(contents));
        }
      }),
    );
  }

  return {
    name: "wabou-style-compiler",
    enforce: "pre",
    async configResolved(config) {
      if (options.themeContrast === "error" && contrastDiagnostics.length > 0) {
        throw new Error(
          `Wabou theme contrast validation failed:\n${contrastDiagnostics
            .map((diagnostic) => {
              const suggestion = diagnostic.suggestedColor
                ? `; try ${diagnostic.suggestedColor}`
                : "";
              return `  - ${diagnostic.theme}.${diagnostic.foreground} has ${diagnostic.ratio.toFixed(2)}:1 contrast on ${diagnostic.background}; expected at least ${diagnostic.minimum}:1${suggestion}`;
            })
            .join("\n")}`,
        );
      }
      // Candidate recognition uses the same generated theme manifest as
      // compilation and the native runtime fallback.
      referenceGenerator = await createGenerator({
        presets: [presetWabou()],
        rules: [
          [
            /^(?:bg|text|border)-(.+)$/,
            ([, token]) =>
              semanticTokens.has(token)
                ? { "--wabou-semantic-color": token }
                : undefined,
          ],
        ],
      });
      for (const diagnostic of contrastDiagnostics) {
        const suggestion = diagnostic.suggestedColor
          ? `; try ${diagnostic.suggestedColor}`
          : "";
        config.logger.warn(
          `[wabou-style] ${diagnostic.theme}.${diagnostic.foreground} has ${diagnostic.ratio.toFixed(2)}:1 contrast on ${diagnostic.background}; expected at least ${diagnostic.minimum}:1 for normal text${suggestion}`,
        );
      }
    },
    async buildStart() {
      const workspacePackages = await findWorkspacePackages(options.root);
      if (workspacePackages) sourceRoots.add(workspacePackages);
      await Promise.all(
        [
          ...wabouSourceDirectories(options.root),
          ...(workspacePackages ? [workspacePackages] : []),
        ].map(scan),
      );
      await regenerate();
    },
    async transform(code, id) {
      if (id === resolved) return;
      if (!accepts(id)) return;
      sources.set(id, extractUtilitySource(code));
      await regenerate();
    },
    resolveId(id) {
      return id === virtual ? resolved : null;
    },
    load(id) {
      if (id !== resolved) return null;
      const json = JSON.stringify(JSON.stringify(stylesheet));
      return [
        `const __s=${json};`,
        `globalThis.__wabou_set_stylesheet?.(__s);`,
        `if(import.meta.hot)import.meta.hot.accept();`,
        `export default JSON.parse(__s);`,
      ].join("\n");
    },
    async handleHotUpdate({ file, server, modules }) {
      if (!accepts(file)) return;
      const contents = await readFile(file, "utf8");
      sources.set(file, extractUtilitySource(contents));
      await regenerate();
      const module = server.moduleGraph.getModuleById(resolved);
      if (module) {
        server.moduleGraph.invalidateModule(module);
        // Returning a module list replaces Vite's default propagation set.
        // Keep the changed TS/TSX modules so Solid Refresh can update the
        // component, and add the virtual module for the regenerated Style IR.
        return wabouHotUpdateModules(modules, module);
      }
    },
  };
}
