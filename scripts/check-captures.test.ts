import { afterEach, describe, expect, test } from "bun:test";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  accessibleNameDiagnostics,
  captureCommand,
  discoverCaptureCases,
  interactionContractDiagnostics,
  parseCaptureArguments,
  pngDimensions,
  rejectedStyleDiagnostics,
  selectCaptureCases,
  semanticRelationshipDiagnostics,
  semanticStateDiagnostics,
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
        checkAccessibleNames: true,
        checkSemanticStates: true,
        checkSemanticRelationships: true,
        checkInteractionContracts: true,
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
        checkAccessibleNames: true,
        checkSemanticStates: true,
        checkSemanticRelationships: true,
        checkInteractionContracts: true,
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
      checkAccessibleNames: true,
      checkSemanticStates: true,
      checkSemanticRelationships: true,
      checkInteractionContracts: true,
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
      checkAccessibleNames: true,
      checkSemanticStates: true,
      checkSemanticRelationships: true,
      checkInteractionContracts: true,
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
              attrs: [],
              listeners: [],
              widget: null,
              focusable: false,
              focusOrder: null,
              semantic: null,
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
              attrs: [],
              listeners: [],
              widget: null,
              focusable: false,
              focusOrder: null,
              semantic: null,
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
          attrs: [["role", "button"]] as Array<[string, string]>,
          listeners: [1],
          widget: null,
          focusable: true,
          focusOrder: 0,
          semantic: null,
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
          attrs: [["role", "label"]] as Array<[string, string]>,
          listeners: [],
          widget: null,
          focusable: false,
          focusOrder: null,
          semantic: null,
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

  test("requires accessible names for semantic controls", () => {
    const node = (
      lo: number,
      parentId: { lo: number; hi: number } | null,
      tag: string,
      text: string | null,
      attrs: Array<[string, string]>,
    ) => ({
      id: { lo, hi: 1 },
      parentId,
      tag,
      text,
      classes: [],
      styleDiagnostics: [],
      attrs,
      listeners: [],
      widget: null,
      focusable: false,
      focusOrder: null,
      semantic: null,
      rect: { x: 0, y: 0, width: 40, height: 20 },
      contentRect: { x: 0, y: 0, width: 40, height: 20 },
      computed: { overflowX: "Visible", overflowY: "Visible" },
    });
    const button = node(1, null, "button", null, [["role", "button"]]);
    const snapshot = {
      status: {
        viewportWidth: 100,
        viewportHeight: 100,
        deviceScale: 1,
        nodeCount: 1,
      },
      nodes: [button],
    };
    expect(accessibleNameDiagnostics(snapshot)).toHaveLength(1);
    button.attrs.push(["aria-label", "Save"]);
    expect(accessibleNameDiagnostics(snapshot)).toEqual([]);

    button.attrs = [
      ["role", "button"],
      ["aria-labelledby", "save-label"],
    ];
    snapshot.nodes.push(
      node(2, null, "text", "Save changes", [
        ["id", "save-label"],
        ["role", "label"],
      ]),
    );
    expect(accessibleNameDiagnostics(snapshot)).toEqual([]);
    button.attrs = [["role", "button"]];
    expect(accessibleNameDiagnostics(snapshot)).toHaveLength(1);
    button.attrs.push(["aria-hidden", "true"]);
    expect(accessibleNameDiagnostics(snapshot)).toEqual([]);
  });

  test("requires role-specific semantic state values", () => {
    const control = {
      id: { lo: 1, hi: 1 },
      parentId: null,
      tag: "button",
      text: null,
      classes: [],
      styleDiagnostics: [],
      attrs: [["role", "checkbox"]] as Array<[string, string]>,
      listeners: [1],
      widget: null,
      focusable: true,
      focusOrder: 0,
      semantic: {
        role: "checkbox",
        label: "Select item",
        disabled: false,
        exposed: true,
        controls: [],
        activeDescendant: null,
        states: {
          checked: null as string | null,
          pressed: null,
          selected: null,
          expanded: null,
          current: null,
          popup: null,
          modal: null,
        },
      },
      rect: { x: 0, y: 0, width: 40, height: 20 },
      contentRect: { x: 0, y: 0, width: 40, height: 20 },
      computed: { overflowX: "Visible", overflowY: "Visible" },
    };
    const snapshot = {
      status: {
        viewportWidth: 100,
        viewportHeight: 100,
        deviceScale: 1,
        nodeCount: 1,
      },
      nodes: [control],
    };
    expect(semanticStateDiagnostics(snapshot)).toHaveLength(1);
    control.attrs.push(["aria-checked", "false"]);
    control.semantic.states.checked = "false";
    expect(semanticStateDiagnostics(snapshot)).toEqual([]);
    control.semantic.states.checked = "true";
    expect(semanticStateDiagnostics(snapshot)[0]).toContain("projected");
    control.semantic.states.checked = "false";
    control.attrs[1] = ["aria-checked", "invalid"];
    expect(semanticStateDiagnostics(snapshot)).toHaveLength(1);

    control.tag = "view";
    control.attrs = [
      ["role", "slider"],
      ["aria-valuemin", "0"],
      ["aria-valuenow", "11"],
      ["aria-valuemax", "10"],
    ];
    expect(semanticStateDiagnostics(snapshot)[0]).toContain(
      "invalid slider range",
    );
    control.attrs[2] = ["aria-valuenow", "5"];
    expect(semanticStateDiagnostics(snapshot)).toEqual([]);
    control.attrs[2] = ["aria-valuenow", ""];
    expect(semanticStateDiagnostics(snapshot)[0]).toContain(
      "finite aria-valuenow",
    );
    control.attrs.push(["aria-hidden", "true"]);
    expect(semanticStateDiagnostics(snapshot)).toEqual([]);
  });

  test("requires unique and live semantic id references", () => {
    type SemanticProjection = {
      role: string;
      label: string | null;
      disabled: boolean;
      exposed: boolean;
      controls: Array<{ lo: number; hi: number }>;
      activeDescendant: { lo: number; hi: number } | null;
      states: {
        checked: string | null;
        pressed: string | null;
        selected: boolean | null;
        expanded: boolean | null;
        current: string | null;
        popup: string | null;
        modal: boolean | null;
      };
    };
    const node = (
      lo: number,
      attrs: Array<[string, string]>,
      parentId: { lo: number; hi: number } | null = null,
    ) => ({
      id: { lo, hi: 1 },
      parentId,
      tag: "view",
      text: null,
      classes: [],
      styleDiagnostics: [],
      attrs,
      listeners: [],
      widget: null,
      focusable: false,
      focusOrder: null,
      semantic: null as SemanticProjection | null,
      rect: { x: 0, y: 0, width: 40, height: 20 },
      contentRect: { x: 0, y: 0, width: 40, height: 20 },
      computed: { overflowX: "Visible", overflowY: "Visible" },
    });
    const owner = node(1, [
      ["aria-controls", "items missing items"],
      ["aria-activedescendant", "active extra"],
    ]);
    const items = node(2, [["id", "items"]]);
    const duplicate = node(3, [["id", "items"]]);
    const snapshot = {
      status: {
        viewportWidth: 100,
        viewportHeight: 100,
        deviceScale: 1,
        nodeCount: 3,
      },
      nodes: [owner, items, duplicate],
    };
    const diagnostics = semanticRelationshipDiagnostics(snapshot);
    expect(diagnostics.some((item) => item.includes("duplicates"))).toBe(true);
    expect(diagnostics.some((item) => item.includes("missing id"))).toBe(true);
    expect(diagnostics.some((item) => item.includes("repeats"))).toBe(true);
    expect(diagnostics.some((item) => item.includes("exactly one"))).toBe(true);

    owner.attrs = [
      ["aria-controls", "items"],
      ["aria-activedescendant", "active"],
    ];
    duplicate.attrs = [["id", "active"]];
    owner.semantic = {
      role: "combobox",
      label: "Example",
      disabled: false,
      exposed: true,
      controls: [items.id],
      activeDescendant: duplicate.id,
      states: {
        checked: null,
        pressed: null,
        selected: null,
        expanded: null,
        current: null,
        popup: null,
        modal: null,
      },
    };
    expect(semanticRelationshipDiagnostics(snapshot)).toEqual([]);

    owner.attrs = [
      ["id", "owner"],
      ["aria-controls", "owner"],
    ];
    expect(semanticRelationshipDiagnostics(snapshot)[0]).toContain(
      "references itself",
    );
    owner.attrs.push(["aria-hidden", "true"]);
    expect(semanticRelationshipDiagnostics(snapshot)).toEqual([]);
  });

  test("requires authored focus policy and an action for interactive controls", () => {
    const control = {
      id: { lo: 1, hi: 1 },
      parentId: null,
      tag: "button",
      text: "Save",
      classes: [],
      styleDiagnostics: [],
      attrs: [["role", "button"]] as Array<[string, string]>,
      listeners: [] as number[],
      widget: null as string | null,
      focusable: false,
      focusOrder: null as number | null,
      semantic: null,
      rect: { x: 0, y: 0, width: 40, height: 20 },
      contentRect: { x: 0, y: 0, width: 40, height: 20 },
      computed: { overflowX: "Visible", overflowY: "Visible" },
    };
    const snapshot = {
      status: {
        viewportWidth: 100,
        viewportHeight: 100,
        deviceScale: 1,
        nodeCount: 1,
      },
      nodes: [control],
    };
    expect(interactionContractDiagnostics(snapshot)).toHaveLength(2);

    control.focusOrder = 0;
    control.listeners = [1];
    expect(interactionContractDiagnostics(snapshot)).toEqual([]);

    control.focusOrder = -1;
    control.listeners = [];
    control.widget = "editor";
    expect(interactionContractDiagnostics(snapshot)).toEqual([]);

    control.attrs = [["role", "option"]];
    control.focusOrder = null;
    control.listeners = [1];
    control.widget = null;
    expect(interactionContractDiagnostics(snapshot)).toEqual([]);

    control.attrs = [["role", "button"]];
    control.focusOrder = null;
    control.focusable = true;
    expect(interactionContractDiagnostics(snapshot)).toHaveLength(2);

    control.attrs.push(["aria-disabled", "true"]);
    control.focusable = false;
    expect(interactionContractDiagnostics(snapshot)).toEqual([]);
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
      checkAccessibleNames: true,
      checkSemanticStates: true,
      checkSemanticRelationships: true,
      checkInteractionContracts: true,
    };
    const node = (lo: number, parentId: { lo: number; hi: number } | null) => ({
      id: { lo, hi: 1 },
      parentId,
      tag: "view",
      text: null,
      classes: [],
      styleDiagnostics: [],
      attrs: [],
      listeners: [],
      widget: null,
      focusable: false,
      focusOrder: null,
      semantic: null,
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
            attrs: [],
            listeners: [],
            widget: null,
            focusable: false,
            focusOrder: null,
            semantic: null,
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
    snapshot.nodes[0].tag = "button";
    snapshot.nodes[0].attrs = [["role", "button"]];
    snapshot.nodes[0].listeners = [1];
    snapshot.nodes[0].focusable = true;
    snapshot.nodes[0].focusOrder = 0;
    await writeFile(snapshotPath, JSON.stringify(snapshot));
    await expect(validateCaptureArtifacts(capture, root)).rejects.toThrow(
      "unnamed semantic controls",
    );
    capture.checkAccessibleNames = false;
    await expect(
      validateCaptureArtifacts(capture, root),
    ).resolves.toBeUndefined();
    snapshot.nodes[0].attrs = [
      ["role", "checkbox"],
      ["aria-label", "Select item"],
    ];
    await writeFile(snapshotPath, JSON.stringify(snapshot));
    await expect(validateCaptureArtifacts(capture, root)).rejects.toThrow(
      "invalid semantic states",
    );
    capture.checkSemanticStates = false;
    snapshot.nodes[0].attrs.push(["aria-controls", "missing"]);
    await writeFile(snapshotPath, JSON.stringify(snapshot));
    await expect(validateCaptureArtifacts(capture, root)).rejects.toThrow(
      "invalid semantic relationships",
    );
    capture.checkSemanticRelationships = false;
    snapshot.nodes[0].listeners = [];
    snapshot.nodes[0].focusable = false;
    snapshot.nodes[0].focusOrder = null;
    await writeFile(snapshotPath, JSON.stringify(snapshot));
    await expect(validateCaptureArtifacts(capture, root)).rejects.toThrow(
      "invalid interaction contracts",
    );
    capture.checkInteractionContracts = false;
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
