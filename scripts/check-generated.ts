import { mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
const generatedPatterns = [
  "crates/wabou-runtime/src/gen/**/*",
  "crates/wabou-shell-api/src/generated/**/*",
  "docs/host-abi.md",
  "packages/*/dist/**/*",
  "packages/*/generated/**/*",
  "packages/core/src/generated/**/*",
  "packages/core/src/style/generated/**/*",
  "packages/vite/src/preset/manifest.json",
  "packages/vite/src/style-compiler/css-support-matrix.json",
];

export type FileSnapshot = Map<string, Uint8Array>;

export async function snapshotFiles(
  workspaceRoot: string,
  patterns: readonly string[],
): Promise<FileSnapshot> {
  const paths = new Set<string>();
  for (const pattern of patterns) {
    const glob = new Bun.Glob(pattern);
    for await (const path of glob.scan({
      cwd: workspaceRoot,
      onlyFiles: true,
    })) {
      paths.add(path);
    }
  }
  const entries = await Promise.all(
    [...paths]
      .sort()
      .map(
        async (path) =>
          [path, await readFile(resolve(workspaceRoot, path))] as const,
      ),
  );
  return new Map(entries);
}

export function changedSnapshotPaths(
  before: FileSnapshot,
  after: FileSnapshot,
): string[] {
  return [...new Set([...before.keys(), ...after.keys()])]
    .filter((path) => {
      const oldContents = before.get(path);
      const newContents = after.get(path);
      return (
        oldContents === undefined ||
        newContents === undefined ||
        !Buffer.from(oldContents).equals(newContents)
      );
    })
    .sort();
}

export async function restoreSnapshot(
  workspaceRoot: string,
  before: FileSnapshot,
  after: FileSnapshot,
): Promise<void> {
  await Promise.all(
    [...after.keys()]
      .filter((path) => !before.has(path))
      .map((path) => rm(resolve(workspaceRoot, path), { force: true })),
  );
  await Promise.all(
    [...before].map(async ([path, contents]) => {
      const absolute = resolve(workspaceRoot, path);
      await mkdir(dirname(absolute), { recursive: true });
      await writeFile(absolute, contents);
    }),
  );
}

async function main(): Promise<void> {
  const before = await snapshotFiles(root, generatedPatterns);
  const child = Bun.spawn(["bun", "run", "gen"], {
    cwd: root,
    stdout: "inherit",
    stderr: "inherit",
  });
  const exitCode = await child.exited;
  const after = await snapshotFiles(root, generatedPatterns);
  const changed = changedSnapshotPaths(before, after);

  if (changed.length > 0) await restoreSnapshot(root, before, after);
  if (exitCode !== 0)
    throw new Error("generation failed; restored generated artifacts");
  if (changed.length > 0) {
    console.error(
      `generated artifacts were stale (worktree restored):\n${changed
        .map((path) => `  - ${path}`)
        .join("\n")}`,
    );
    process.exitCode = 1;
    return;
  }

  console.log(`verified ${after.size} generated artifacts`);
}

if (import.meta.main) await main();
