import type { Dirent } from "node:fs";
import { readdir, readFile } from "node:fs/promises";
import { dirname, join, parse, sep } from "node:path";
import { createGenerator } from "@unocss/core";
import {
  presetWabou,
  resolveWabouUtility,
  validateWabouUtility,
  wabouUtilityManifest,
} from "@wabou/unocss-preset";
import type { ModuleNode, Plugin } from "vite";
import {
  STYLE_IR_VERSION,
  type StyleRule,
  type WabouStyleSheet,
} from "./ir.ts";

export interface WabouStylePluginOptions {
  root: string;
}

export function assertSupportedWabouCandidates(
  candidates: Iterable<string>,
): void {
  const unsupported = [...candidates]
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
): StyleRule[] {
  return [...candidates].sort().map((candidate, index) => {
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
  let stylesheet: WabouStyleSheet = {
    version: STYLE_IR_VERSION,
    theme: {
      spacing: wabouUtilityManifest.spacing,
      colors: wabouUtilityManifest.colors,
    },
    diagnostics: [],
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
    assertSupportedWabouCandidates(reference.matched);
    stylesheet = {
      version: STYLE_IR_VERSION,
      theme: {
        spacing: wabouUtilityManifest.spacing,
        colors: wabouUtilityManifest.colors,
      },
      diagnostics: [],
      rules: compileWabouUtilities(reference.matched),
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
    async configResolved() {
      // Candidate recognition uses the same generated theme manifest as
      // compilation and the native runtime fallback.
      referenceGenerator = await createGenerator({ presets: [presetWabou()] });
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
