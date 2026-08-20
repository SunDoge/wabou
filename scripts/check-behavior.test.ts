import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, test } from "bun:test";
import { discoverBehaviorApps } from "./check-behavior";

const roots: string[] = [];

afterEach(async () => {
  await Promise.all(
    roots.splice(0).map((root) => rm(root, { recursive: true })),
  );
});

describe("native behavior suite discovery", () => {
  test("discovers nested behavior suites once per app", async () => {
    const root = await mkdtemp(join(tmpdir(), "wabou-behavior-discovery-"));
    roots.push(root);
    await mkdir(join(root, "apps", "zeta", "tests", "nested"), {
      recursive: true,
    });
    await mkdir(join(root, "apps", "alpha", "tests"), { recursive: true });
    await writeFile(join(root, "apps", "zeta", "tests", "one.behavior.ts"), "");
    await writeFile(
      join(root, "apps", "zeta", "tests", "nested", "two.behavior.ts"),
      "",
    );
    await writeFile(
      join(root, "apps", "alpha", "tests", "app.behavior.ts"),
      "",
    );

    expect(await discoverBehaviorApps(root)).toEqual([
      "apps/alpha",
      "apps/zeta",
    ]);
  });

  test("does not pretend standalone environment-specific scenarios are suites", async () => {
    const root = await mkdtemp(join(tmpdir(), "wabou-behavior-discovery-"));
    roots.push(root);
    await mkdir(join(root, "apps", "special", "tests"), { recursive: true });
    await writeFile(
      join(root, "apps", "special", "tests", "failure.scenario.ts"),
      "",
    );

    expect(await discoverBehaviorApps(root)).toEqual([]);
  });
});
