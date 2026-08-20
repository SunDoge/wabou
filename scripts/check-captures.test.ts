import { afterEach, describe, expect, test } from "bun:test";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  captureCommand,
  discoverCaptureCases,
  validateCaptureSnapshot,
} from "./check-captures";

const roots: string[] = [];

afterEach(async () => {
  await Promise.all(
    roots.splice(0).map((root) => rm(root, { recursive: true })),
  );
});

async function fixture(): Promise<string> {
  const root = await mkdtemp(join(tmpdir(), "wabou-capture-discovery-"));
  roots.push(root);
  await mkdir(join(root, "apps", "demo", "captures", "nested"), {
    recursive: true,
  });
  await writeFile(join(root, "apps", "demo", "captures", "wide.ts"), "");
  await writeFile(
    join(root, "apps", "demo", "captures", "nested", "compact.ts"),
    "",
  );
  return root;
}

describe("authored capture discovery", () => {
  test("applies app defaults and per-scenario viewport overrides", async () => {
    const root = await fixture();
    await writeFile(
      join(root, "apps", "demo", "captures", "config.json"),
      JSON.stringify({
        defaults: { width: 1200, height: 800, waitMs: 100 },
        overrides: {
          "nested/compact.ts": { width: 700, height: 500, scaleFactor: 2 },
        },
      }),
    );

    expect(await discoverCaptureCases(root)).toEqual([
      {
        application: "apps/demo",
        scenario: "apps/demo/captures/nested/compact.ts",
        output: "target/wabou-captures/demo/nested/compact.png",
        snapshot: "target/wabou-captures/demo/nested/compact.json",
        width: 700,
        height: 500,
        scaleFactor: 2,
        waitMs: 100,
      },
      {
        application: "apps/demo",
        scenario: "apps/demo/captures/wide.ts",
        output: "target/wabou-captures/demo/wide.png",
        snapshot: "target/wabou-captures/demo/wide.json",
        width: 1200,
        height: 800,
        scaleFactor: 1,
        waitMs: 100,
      },
    ]);
  });

  test("rejects overrides that no longer identify a capture", async () => {
    const root = await fixture();
    await writeFile(
      join(root, "apps", "demo", "captures", "config.json"),
      JSON.stringify({ overrides: { "removed.ts": { width: 700 } } }),
    );

    await expect(discoverCaptureCases(root)).rejects.toThrow("removed.ts");
  });

  test("only later captures reuse the already built application bundle", () => {
    const capture = {
      application: "apps/demo",
      scenario: "apps/demo/captures/main.ts",
      output: "target/wabou-captures/demo/main.png",
      snapshot: "target/wabou-captures/demo/main.json",
      width: 800,
      height: 600,
      scaleFactor: 1,
      waitMs: 250,
    };

    expect(captureCommand(capture, false)).not.toContain("--skip-build");
    expect(captureCommand(capture, true)).toContain("--skip-build");
    expect(captureCommand(capture, false)).toContain(capture.snapshot);
  });

  test("validates that a snapshot describes the requested final frame", () => {
    const capture = {
      application: "apps/demo",
      scenario: "apps/demo/captures/main.ts",
      output: "target/wabou-captures/demo/main.png",
      snapshot: "target/wabou-captures/demo/main.json",
      width: 800,
      height: 600,
      scaleFactor: 2,
      waitMs: 250,
    };
    expect(
      validateCaptureSnapshot(
        {
          status: {
            viewportWidth: 800,
            viewportHeight: 600,
            deviceScale: 2,
            nodeCount: 1,
          },
          nodes: [
            {
              rect: { x: 0, y: 0, width: 800, height: 600 },
              contentRect: { x: 0, y: 0, width: 800, height: 600 },
            },
          ],
        },
        capture,
      ).nodes,
    ).toHaveLength(1);

    expect(() =>
      validateCaptureSnapshot(
        {
          status: {
            viewportWidth: 801,
            viewportHeight: 600,
            deviceScale: 2,
            nodeCount: 1,
          },
          nodes: [
            {
              rect: { x: 0, y: 0, width: Number.NaN, height: 600 },
              contentRect: { x: 0, y: 0, width: 800, height: 600 },
            },
          ],
        },
        capture,
      ),
    ).toThrow("finite number");
  });
});
