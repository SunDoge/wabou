import { writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { STYLE_IR_VERSION } from "../../../packages/vite/src/style-compiler/ir.ts";
import { compileWabouUtilities } from "../../../packages/vite/src/style-compiler/vite.ts";
import { validateWabouUtility } from "../../../packages/vite/src/preset/index.ts";
import manifest from "../../../packages/vite/src/preset/manifest.json";

const root = resolve(import.meta.dir, "../../..");

const token: Record<string, string> = {
  spacing: "4",
  dimension: "4",
  color: "slate-900",
  opacity: "50",
  number: "[2]",
  ratio: "[16/9]",
  length: "[2px]",
  translate: "4",
  scale: "125",
  rotate: "30",
};
const candidates = new Set(Object.keys(manifest.staticUtilities));
for (const rule of manifest.dynamicRules) {
  const value = token[rule.resolver];
  if (!value) throw new Error(`missing conformance token for ${rule.resolver}`);
  for (const prefix of rule.prefixes) candidates.add(`${prefix.name}-${value}`);
}
for (const fixture of manifest.conformance) candidates.add(fixture.className);

const stylesheet = {
  version: STYLE_IR_VERSION,
  theme: {
    spacing: manifest.spacing,
    colors: manifest.colors,
  },
  diagnostics: [],
  // The parser manifest describes every recognized utility, including
  // legacy-only candidates. Formal conformance exercises only candidates that
  // the GPUI support contract allows the compiler to emit.
  rules: compileWabouUtilities(
    [...candidates].filter((candidate) => !validateWabouUtility(candidate)),
  ),
};
await writeFile(
  resolve(root, "crates/wabou-runtime/src/gen/style-conformance.json"),
  `${JSON.stringify(stylesheet, null, 2)}\n`,
);
console.log(`generated ${stylesheet.rules.length} style conformance rules`);
