import { afterEach, describe, expect, test } from "bun:test";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  captureCommand,
  discoverCaptureCases,
  parseCaptureArguments,
  pngDimensions,
  rejectedStyleDiagnostics,
  selectCaptureCases,
  textContainmentDiagnostics,
  validateCaptureArtifacts,
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

function pngHeader(width: number, height: number): Uint8Array {
  const bytes = new Uint8Array(24);
  bytes.set([137, 80, 78, 71, 13, 10, 26, 10]);
  bytes.set([0, 0, 0, 13, 73, 72, 68, 82], 8);
  const view = new DataView(bytes.buffer);
  view.setUint32(16, width, false);
  view.setUint32(20, height, false);
  return bytes;
}

describe("authored capture discovery", () => {
  test("parses exact repeatable scenario selection", () => {
    expect(
      parseCaptureArguments([
        "--check-existing",
        "--scenario",
        "apps/demo/captures/first.ts",
        "--scenario",
        "apps\\demo\\captures\\second.ts",
      ]),
    ).toEqual({
      list: false,
      checkExisting: true,
      scenarios: [
        "apps/demo/captures/first.ts",
        "apps/demo/captures/second.ts",
      ],
    });
    expect(() => parseCaptureArguments(["--scenario"])).toThrow("requires");
    expect(() => parseCaptureArguments(["--unknown"])).toThrow("unsupported");
    expect(() => parseCaptureArguments(["--list", "--check-existing"])).toThrow(
      "cannot be combined",
    );
  });

  test("selects all captures when no scenario filter is supplied", async () => {
    const root = await fixture();
    const captures = await discoverCaptureCases(root);
    expect(selectCaptureCases(captures, [])).toBe(captures);
    expect(
      selectCaptureCases(captures, ["apps/demo/captures/wide.ts"]),
    ).toEqual([captures[1]]);
    expect(() =>
      selectCaptureCases(captures, ["apps/demo/captures/missing.ts"]),
    ).toThrow("missing.ts");
  });

  test("applies app defaults and per-scenario viewport overrides", async () => {
    const root = await fixture();
    await writeFile(
      join(root, "apps", "demo", "captures", "config.json"),
      JSON.stringify({
        defaults: { width: 1200, height: 800, waitMs: 100 },
        overrides: {
          "nested/compact.ts": {
            width: 700,
            height: 500,
            scaleFactor: 2,
            checkTextContainment: false,
          },
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
        checkTextContainment: false,
        checkStyleDiagnostics: true,
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
        checkTextContainment: true,
        checkStyleDiagnostics: true,
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
      checkTextContainment: true,
      checkStyleDiagnostics: true,
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
      checkTextContainment: true,
      checkStyleDiagnostics: true,
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
              id: { lo: 1, hi: 1 },
              parentId: null,
              tag: "view",
              text: null,
              classes: [],
              styleDiagnostics: [],
              rect: { x: 0, y: 0, width: 800, height: 600 },
              contentRect: { x: 0, y: 0, width: 800, height: 600 },
              computed: { overflowX: "Visible", overflowY: "Visible" },
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
              id: { lo: 1, hi: 1 },
              parentId: null,
              tag: "view",
              text: null,
              classes: [],
              styleDiagnostics: [],
              rect: { x: 0, y: 0, width: Number.NaN, height: 600 },
              contentRect: { x: 0, y: 0, width: 800, height: 600 },
              computed: { overflowX: "Visible", overflowY: "Visible" },
            },
          ],
        },
        capture,
      ),
    ).toThrow("finite number");
  });

  test("reports text escaping visible ancestors but stops at clip boundaries", () => {
    const base = {
      status: {
        viewportWidth: 100,
        viewportHeight: 100,
        deviceScale: 1,
        nodeCount: 2,
      },
      nodes: [
        {
          id: { lo: 1, hi: 1 },
          parentId: null,
          tag: "button",
          text: null,
          classes: ["w-10"],
          styleDiagnostics: [] as string[],
          rect: { x: 0, y: 0, width: 40, height: 20 },
          contentRect: { x: 0, y: 0, width: 40, height: 20 },
          computed: { overflowX: "Visible", overflowY: "Visible" },
        },
        {
          id: { lo: 2, hi: 1 },
          parentId: { lo: 1, hi: 1 },
          tag: "text",
          text: "too wide",
          classes: [],
          styleDiagnostics: [] as string[],
          rect: { x: 0, y: 0, width: 60, height: 20 },
          contentRect: { x: 0, y: 0, width: 60, height: 20 },
          computed: { overflowX: "Visible", overflowY: "Visible" },
        },
      ],
    };
    expect(textContainmentDiagnostics(base)).toHaveLength(1);
    base.nodes[0].computed.overflowX = "Hidden";
    expect(textContainmentDiagnostics(base)).toEqual([]);
    base.nodes[1].styleDiagnostics = ["unsupported utility `bad-class`"];
    expect(rejectedStyleDiagnostics(base)).toEqual([
      "text 2:1 (no classes): unsupported utility `bad-class`",
    ]);
  });

  test("rejects duplicate, dangling, and cyclic retained-node identities", () => {
    const capture = {
      application: "apps/demo",
      scenario: "apps/demo/captures/main.ts",
      output: "target/wabou-captures/demo/main.png",
      snapshot: "target/wabou-captures/demo/main.json",
      width: 100,
      height: 100,
      scaleFactor: 1,
      waitMs: 0,
      checkTextContainment: true,
      checkStyleDiagnostics: true,
    };
    const node = (lo: number, parentId: { lo: number; hi: number } | null) => ({
      id: { lo, hi: 1 },
      parentId,
      tag: "view",
      text: null,
      classes: [],
      styleDiagnostics: [],
      rect: { x: 0, y: 0, width: 100, height: 100 },
      contentRect: { x: 0, y: 0, width: 100, height: 100 },
      computed: { overflowX: "Visible", overflowY: "Visible" },
    });
    const snapshot = (nodes: ReturnType<typeof node>[]) => ({
      status: {
        viewportWidth: 100,
        viewportHeight: 100,
        deviceScale: 1,
        nodeCount: nodes.length,
      },
      nodes,
    });
    expect(() =>
      validateCaptureSnapshot(
        snapshot([node(1, null), node(1, null)]),
        capture,
      ),
    ).toThrow("duplicate node id");
    expect(() =>
      validateCaptureSnapshot(
        snapshot([node(1, null), node(2, { lo: 9, hi: 1 })]),
        capture,
      ),
    ).toThrow("missing parent");
    expect(() =>
      validateCaptureSnapshot(
        snapshot([
          node(1, null),
          node(2, { lo: 3, hi: 1 }),
          node(3, { lo: 2, hi: 1 }),
        ]),
        capture,
      ),
    ).toThrow("parent cycle");
  });

  test("revalidates existing artifacts without invoking a renderer", async () => {
    const root = await fixture();
    const [capture] = await discoverCaptureCases(root);
    if (!capture) throw new Error("expected a discovered capture");
    await expect(validateCaptureArtifacts(capture, root)).rejects.toThrow(
      capture.output,
    );

    await mkdir(join(root, "target", "wabou-captures", "demo", "nested"), {
      recursive: true,
    });
    await writeFile(
      join(root, capture.output),
      pngHeader(
        Math.round(capture.width * capture.scaleFactor),
        Math.round(capture.height * capture.scaleFactor),
      ),
    );
    await writeFile(
      join(root, capture.snapshot),
      JSON.stringify({
        status: {
          viewportWidth: capture.width,
          viewportHeight: capture.height,
          deviceScale: capture.scaleFactor,
          nodeCount: 1,
        },
        nodes: [
          {
            id: { lo: 1, hi: 1 },
            parentId: null,
            tag: "view",
            text: null,
            classes: [],
            styleDiagnostics: [],
            rect: { x: 0, y: 0, width: capture.width, height: capture.height },
            contentRect: {
              x: 0,
              y: 0,
              width: capture.width,
              height: capture.height,
            },
            computed: { overflowX: "Visible", overflowY: "Visible" },
          },
        ],
      }),
    );
    await expect(
      validateCaptureArtifacts(capture, root),
    ).resolves.toBeUndefined();

    const snapshotPath = join(root, capture.snapshot);
    const snapshot = JSON.parse(await Bun.file(snapshotPath).text());
    snapshot.nodes[0].styleDiagnostics = ["unsupported utility `bad-class`"];
    await writeFile(snapshotPath, JSON.stringify(snapshot));
    await expect(validateCaptureArtifacts(capture, root)).rejects.toThrow(
      "rejected styles",
    );
    capture.checkStyleDiagnostics = false;
    await expect(
      validateCaptureArtifacts(capture, root),
    ).resolves.toBeUndefined();
  });

  test("reads physical PNG dimensions from the mandatory IHDR chunk", () => {
    expect(pngDimensions(pngHeader(1800, 1200))).toEqual({
      width: 1800,
      height: 1200,
    });
    expect(() => pngDimensions(new Uint8Array(24))).toThrow("IHDR");
    expect(() => pngDimensions(pngHeader(0, 1200))).toThrow("zero");
  });
});
