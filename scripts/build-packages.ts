import { randomUUID } from "node:crypto";
import { mkdir, readdir, rename, rm, stat } from "node:fs/promises";
import { dirname, join, relative, resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
const packageNames = ["core", "terminal", "test", "ui", "vite"] as const;
const stageName = `.wabou-dist-${randomUUID()}`;

async function filesBelow(directory: string): Promise<string[]> {
  const result: string[] = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      result.push(...(await filesBelow(path)));
    } else if (entry.isFile()) {
      result.push(path);
    }
  }
  return result;
}

async function promotePackage(name: (typeof packageNames)[number]) {
  const staged = resolve(root, "packages", name, stageName);
  const destination = resolve(root, "packages", name, "dist");
  const stagedFiles = await filesBelow(staged);
  const next = new Set(stagedFiles.map((path) => relative(staged, path)));

  await mkdir(destination, { recursive: true });
  for (const source of stagedFiles) {
    const output = resolve(destination, relative(staged, source));
    await mkdir(dirname(output), { recursive: true });
    // A same-filesystem file rename replaces each artifact atomically. Readers
    // therefore keep seeing the previous complete file until its replacement
    // is ready instead of observing tsdown's cleaned, half-built dist tree.
    await rename(source, output);
  }

  for (const output of await filesBelow(destination)) {
    if (!next.has(relative(destination, output))) await rm(output);
  }
}

try {
  const child = Bun.spawn(["bun", "--bun", "tsdown", "--concurrency", "1"], {
    cwd: root,
    env: { ...process.env, WABOU_PACKAGE_STAGE_NAME: stageName },
    stdout: "inherit",
    stderr: "inherit",
  });
  if ((await child.exited) !== 0) throw new Error("package build failed");

  for (const name of packageNames) {
    const info = await stat(resolve(root, "packages", name, stageName)).catch(
      () => undefined,
    );
    if (!info?.isDirectory())
      throw new Error(`missing staged package: ${name}`);
    await promotePackage(name);
  }
} finally {
  await Promise.all(
    packageNames.map((name) =>
      rm(resolve(root, "packages", name, stageName), {
        recursive: true,
        force: true,
      }),
    ),
  );
}
