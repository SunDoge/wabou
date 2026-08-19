import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
const patterns = [
  "crates/wabou-runtime/src/gen/**/*",
  "crates/wabou-shell/src/generated/**/*",
  "docs/host-abi.md",
  "packages/*/dist/**/*",
  "packages/*/generated/**/*",
  "packages/core/src/generated/**/*",
  "packages/vite/src/preset/manifest.json",
  "packages/vite/src/style-compiler/css-support-matrix.json",
];

async function snapshot(): Promise<Map<string, string>> {
  const paths = new Set<string>();
  for (const pattern of patterns) {
    const glob = new Bun.Glob(pattern);
    for await (const path of glob.scan({ cwd: root, onlyFiles: true }))
      paths.add(path);
  }
  const entries = await Promise.all(
    [...paths]
      .sort()
      .map(
        async (path) =>
          [path, await readFile(resolve(root, path), "utf8")] as const,
      ),
  );
  return new Map(entries);
}

const before = await snapshot();
const child = Bun.spawn(["bun", "run", "gen"], {
  cwd: root,
  stdout: "inherit",
  stderr: "inherit",
});
if ((await child.exited) !== 0) throw new Error("generation failed");
const after = await snapshot();
const changed = [...new Set([...before.keys(), ...after.keys()])].filter(
  (path) => before.get(path) !== after.get(path),
);
if (changed.length > 0) {
  console.error(
    `generated artifacts were stale:\n${changed.map((path) => `  - ${path}`).join("\n")}`,
  );
  process.exitCode = 1;
} else {
  console.log(`verified ${after.size} generated artifacts`);
}
