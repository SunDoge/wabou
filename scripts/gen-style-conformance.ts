import { writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { compileWabouUtilities } from "../packages/style-compiler/src/vite.ts";
import { STYLE_IR_VERSION } from "../packages/style-compiler/src/ir.ts";
import manifest from "../packages/unocss-preset/generated/manifest.json";

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
  rules: compileWabouUtilities(candidates),
};
await writeFile(
  resolve(
    import.meta.dir,
    "../crates/wabou-quick/src/gen/style-conformance.json",
  ),
  `${JSON.stringify(stylesheet, null, 2)}\n`,
);
console.log(`generated ${stylesheet.rules.length} style conformance rules`);
