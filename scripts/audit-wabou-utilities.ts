import type { Dirent } from "node:fs";
import { readdir, readFile } from "node:fs/promises";
import { join } from "node:path";
import { createGenerator, presetUno } from "unocss";
import { extractUtilitySource } from "../packages/vite/src/style-compiler/vite.ts";
import { validateWabouUtility } from "../packages/unocss-preset/src/index.ts";

const root = process.cwd();
const source: string[] = [];

async function scan(directory: string): Promise<void> {
  let entries: Dirent[];
  try {
    entries = await readdir(directory, { withFileTypes: true });
  } catch {
    return;
  }
  for (const entry of entries) {
    if (["dist", "node_modules", "target"].includes(entry.name)) continue;
    const path = join(directory, entry.name);
    if (entry.isDirectory()) await scan(path);
    else if (
      /\.[jt]sx?$/.test(entry.name) &&
      !/\.(?:test|spec)\.[jt]sx?$/.test(entry.name)
    ) {
      source.push(extractUtilitySource(await readFile(path, "utf8")));
    }
  }
}

await Promise.all([scan(join(root, "apps")), scan(join(root, "packages"))]);
const uno = await createGenerator({ presets: [presetUno()] });
const generated = await uno.generate(source.join("\n"));
const unsupported = [...generated.matched]
  .filter((candidate) => validateWabouUtility(candidate))
  .sort();

if (unsupported.length) {
  console.error(
    `${unsupported.length} utilities are not implemented by wabou-style:`,
  );
  console.error(unsupported.join("\n"));
  process.exitCode = 1;
} else {
  console.log(`${generated.matched.size} utilities are covered by wabou-style`);
}
