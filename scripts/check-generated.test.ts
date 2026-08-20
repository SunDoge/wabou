import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, test } from "bun:test";
import {
  changedSnapshotPaths,
  restoreSnapshot,
  snapshotFiles,
} from "./check-generated";

const roots: string[] = [];

afterEach(async () => {
  await Promise.all(
    roots.splice(0).map((root) => rm(root, { recursive: true })),
  );
});

describe("generated artifact transaction", () => {
  test("detects and restores modified, removed, and newly generated files", async () => {
    const root = await mkdtemp(join(tmpdir(), "wabou-generated-check-"));
    roots.push(root);
    const generated = join(root, "generated");
    await mkdir(generated);
    await writeFile(join(generated, "changed.txt"), "original");
    await writeFile(join(generated, "removed.txt"), "keep me");
    const before = await snapshotFiles(root, ["generated/**/*"]);

    await writeFile(join(generated, "changed.txt"), "rewritten");
    await rm(join(generated, "removed.txt"));
    await writeFile(join(generated, "created.txt"), "new output");
    const after = await snapshotFiles(root, ["generated/**/*"]);

    expect(changedSnapshotPaths(before, after)).toEqual([
      "generated/changed.txt",
      "generated/created.txt",
      "generated/removed.txt",
    ]);
    await restoreSnapshot(root, before, after);
    expect(await readFile(join(generated, "changed.txt"), "utf8")).toBe(
      "original",
    );
    expect(await readFile(join(generated, "removed.txt"), "utf8")).toBe(
      "keep me",
    );
    expect(Bun.file(join(generated, "created.txt")).size).toBe(0);
  });

  test("compares bytes instead of assuming generated files are UTF-8", async () => {
    const before = new Map([["artifact.bin", Uint8Array.of(0, 255, 1)]]);
    const equal = new Map([["artifact.bin", Uint8Array.of(0, 255, 1)]]);
    const changed = new Map([["artifact.bin", Uint8Array.of(0, 254, 1)]]);

    expect(changedSnapshotPaths(before, equal)).toEqual([]);
    expect(changedSnapshotPaths(before, changed)).toEqual(["artifact.bin"]);
  });
});
