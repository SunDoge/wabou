import { mkdir, readFile } from "node:fs/promises";
import { basename, dirname, relative, resolve } from "node:path";

const root = resolve(import.meta.dir, "..");

interface CaptureViewport {
  width: number;
  height: number;
  scaleFactor: number;
  waitMs: number;
  checkTextContainment: boolean;
  checkStyleDiagnostics: boolean;
  checkAccessibleNames: boolean;
}

interface CaptureConfig {
  defaults: CaptureViewport;
  overrides: Record<string, Partial<CaptureViewport>>;
}

export interface CaptureCase extends CaptureViewport {
  application: string;
  scenario: string;
  output: string;
  snapshot: string;
}

interface SnapshotRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface PngDimensions {
  width: number;
  height: number;
}

interface SnapshotNodeKey {
  lo: number;
  hi: number;
}

interface CaptureSnapshotNode {
  id: SnapshotNodeKey;
  parentId: SnapshotNodeKey | null;
  tag: string;
  text: string | null;
  classes: string[];
  styleDiagnostics: string[];
  attrs: Array<[string, string]>;
  rect: SnapshotRect;
  contentRect: SnapshotRect;
  computed: { overflowX: string | null; overflowY: string | null };
}

interface CaptureSnapshot {
  status: {
    viewportWidth: number;
    viewportHeight: number;
    deviceScale: number;
    nodeCount: number;
  };
  nodes: CaptureSnapshotNode[];
}

export function captureCommand(
  capture: CaptureCase,
  skipBuild: boolean,
): string[] {
  const args = [
    "cargo",
    "run",
    "-p",
    "wabou-cli",
    "--",
    "render",
    capture.application,
    "--with-host",
    "--scenario",
    capture.scenario,
    "--out",
    capture.output,
    "--snapshot",
    capture.snapshot,
    "--width",
    String(capture.width),
    "--height",
    String(capture.height),
    "--scale-factor",
    String(capture.scaleFactor),
    "--wait-ms",
    String(capture.waitMs),
  ];
  if (skipBuild) args.push("--skip-build");
  return args;
}

const fallbackViewport: CaptureViewport = {
  width: 1440,
  height: 900,
  scaleFactor: 1,
  waitMs: 250,
  checkTextContainment: true,
  checkStyleDiagnostics: true,
  checkAccessibleNames: true,
};
const viewportKeys = new Set([
  "width",
  "height",
  "scaleFactor",
  "waitMs",
  "checkTextContainment",
  "checkStyleDiagnostics",
  "checkAccessibleNames",
]);

function finiteNumber(
  value: unknown,
  name: string,
  minimum: number,
  maximum: number,
  integer = false,
): number {
  if (
    typeof value !== "number" ||
    !Number.isFinite(value) ||
    value < minimum ||
    value > maximum ||
    (integer && !Number.isInteger(value))
  ) {
    throw new Error(`${name} must be between ${minimum} and ${maximum}`);
  }
  return value;
}

function parseViewport(
  value: unknown,
  name: string,
  partial: boolean,
): Partial<CaptureViewport> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${name} must be an object`);
  }
  const record = value as Record<string, unknown>;
  for (const key of Object.keys(record)) {
    if (!viewportKeys.has(key))
      throw new Error(`${name}.${key} is unsupported`);
  }
  const viewport: Partial<CaptureViewport> = {};
  if (record.width !== undefined)
    viewport.width = finiteNumber(
      record.width,
      `${name}.width`,
      1,
      16_384,
      true,
    );
  if (record.height !== undefined)
    viewport.height = finiteNumber(
      record.height,
      `${name}.height`,
      1,
      16_384,
      true,
    );
  if (record.scaleFactor !== undefined)
    viewport.scaleFactor = finiteNumber(
      record.scaleFactor,
      `${name}.scaleFactor`,
      0.25,
      8,
    );
  if (record.waitMs !== undefined)
    viewport.waitMs = finiteNumber(
      record.waitMs,
      `${name}.waitMs`,
      0,
      60_000,
      true,
    );
  if (record.checkTextContainment !== undefined) {
    if (typeof record.checkTextContainment !== "boolean") {
      throw new Error(`${name}.checkTextContainment must be a boolean`);
    }
    viewport.checkTextContainment = record.checkTextContainment;
  }
  if (record.checkStyleDiagnostics !== undefined) {
    if (typeof record.checkStyleDiagnostics !== "boolean") {
      throw new Error(`${name}.checkStyleDiagnostics must be a boolean`);
    }
    viewport.checkStyleDiagnostics = record.checkStyleDiagnostics;
  }
  if (record.checkAccessibleNames !== undefined) {
    if (typeof record.checkAccessibleNames !== "boolean") {
      throw new Error(`${name}.checkAccessibleNames must be a boolean`);
    }
    viewport.checkAccessibleNames = record.checkAccessibleNames;
  }
  if (!partial) {
    return { ...fallbackViewport, ...viewport };
  }
  return viewport;
}

async function loadConfig(captureDirectory: string): Promise<CaptureConfig> {
  const path = resolve(captureDirectory, "config.json");
  if (!(await Bun.file(path).exists())) {
    return { defaults: fallbackViewport, overrides: {} };
  }
  const value: unknown = JSON.parse(await readFile(path, "utf8"));
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${path} must contain an object`);
  }
  const record = value as Record<string, unknown>;
  for (const key of Object.keys(record)) {
    if (key !== "defaults" && key !== "overrides") {
      throw new Error(`${path} contains unsupported key ${key}`);
    }
  }
  const defaults = {
    ...fallbackViewport,
    ...parseViewport(record.defaults ?? {}, `${path}.defaults`, true),
  };
  const rawOverrides = record.overrides ?? {};
  if (
    rawOverrides === null ||
    typeof rawOverrides !== "object" ||
    Array.isArray(rawOverrides)
  ) {
    throw new Error(`${path}.overrides must be an object`);
  }
  const overrides = Object.fromEntries(
    Object.entries(rawOverrides as Record<string, unknown>).map(
      ([scenario, viewport]) => [
        scenario,
        parseViewport(viewport, `${path}.overrides.${scenario}`, true),
      ],
    ),
  );
  return { defaults, overrides };
}

export async function discoverCaptureCases(
  workspaceRoot = root,
): Promise<CaptureCase[]> {
  const glob = new Bun.Glob("apps/*/captures/**/*.ts");
  const sources: string[] = [];
  for await (const source of glob.scan({
    cwd: workspaceRoot,
    onlyFiles: true,
  })) {
    sources.push(source.replaceAll("\\", "/"));
  }
  sources.sort();

  const configs = new Map<string, Promise<CaptureConfig>>();
  const cases: CaptureCase[] = [];
  for (const scenario of sources) {
    const parts = scenario.split("/");
    const application = parts.slice(0, 2).join("/");
    const captureDirectory = resolve(workspaceRoot, application, "captures");
    const relativeScenario = parts.slice(3).join("/");
    let config = configs.get(application);
    if (!config) {
      config = loadConfig(captureDirectory);
      configs.set(application, config);
    }
    const resolved = await config;
    const override = resolved.overrides[relativeScenario] ?? {};
    cases.push({
      application,
      scenario,
      output: `target/wabou-captures/${basename(application)}/${relativeScenario.replace(/\.ts$/u, ".png")}`,
      snapshot: `target/wabou-captures/${basename(application)}/${relativeScenario.replace(/\.ts$/u, ".json")}`,
      ...resolved.defaults,
      ...override,
    });
  }

  for (const [application, configPromise] of configs) {
    const config = await configPromise;
    const known = new Set(
      cases
        .filter((capture) => capture.application === application)
        .map((capture) => relativeScenarioPath(capture.scenario)),
    );
    for (const override of Object.keys(config.overrides)) {
      if (!known.has(override)) {
        throw new Error(
          `${application}/captures/config.json overrides missing scenario ${override}`,
        );
      }
    }
  }
  return cases;
}

function snapshotNumber(value: unknown, path: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new Error(`${path} must be a finite number`);
  }
  return value;
}

function snapshotRect(value: unknown, path: string): SnapshotRect {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${path} must be an object`);
  }
  const rect = value as Record<string, unknown>;
  return {
    x: snapshotNumber(rect.x, `${path}.x`),
    y: snapshotNumber(rect.y, `${path}.y`),
    width: snapshotNumber(rect.width, `${path}.width`),
    height: snapshotNumber(rect.height, `${path}.height`),
  };
}

function snapshotNodeKey(value: unknown, path: string): SnapshotNodeKey {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${path} must be an object`);
  }
  const key = value as Record<string, unknown>;
  return {
    lo: snapshotNumber(key.lo, `${path}.lo`),
    hi: snapshotNumber(key.hi, `${path}.hi`),
  };
}

function optionalString(value: unknown, path: string): string | null {
  if (value === null) return null;
  if (typeof value !== "string")
    throw new Error(`${path} must be a string or null`);
  return value;
}

export function validateCaptureSnapshot(
  value: unknown,
  capture: CaptureCase,
): CaptureSnapshot {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${capture.snapshot} must contain an object`);
  }
  const snapshot = value as Record<string, unknown>;
  if (
    snapshot.status === null ||
    typeof snapshot.status !== "object" ||
    Array.isArray(snapshot.status)
  ) {
    throw new Error(`${capture.snapshot}.status must be an object`);
  }
  const status = snapshot.status as Record<string, unknown>;
  if (!Array.isArray(snapshot.nodes)) {
    throw new Error(`${capture.snapshot}.nodes must be an array`);
  }
  const parsed: CaptureSnapshot = {
    status: {
      viewportWidth: snapshotNumber(
        status.viewportWidth,
        `${capture.snapshot}.status.viewportWidth`,
      ),
      viewportHeight: snapshotNumber(
        status.viewportHeight,
        `${capture.snapshot}.status.viewportHeight`,
      ),
      deviceScale: snapshotNumber(
        status.deviceScale,
        `${capture.snapshot}.status.deviceScale`,
      ),
      nodeCount: snapshotNumber(
        status.nodeCount,
        `${capture.snapshot}.status.nodeCount`,
      ),
    },
    nodes: snapshot.nodes.map((value, index) => {
      if (value === null || typeof value !== "object" || Array.isArray(value)) {
        throw new Error(
          `${capture.snapshot}.nodes[${index}] must be an object`,
        );
      }
      const node = value as Record<string, unknown>;
      if (
        !Array.isArray(node.classes) ||
        !node.classes.every((item) => typeof item === "string")
      ) {
        throw new Error(
          `${capture.snapshot}.nodes[${index}].classes must be a string array`,
        );
      }
      if (
        !Array.isArray(node.styleDiagnostics) ||
        !node.styleDiagnostics.every((item) => typeof item === "string")
      ) {
        throw new Error(
          `${capture.snapshot}.nodes[${index}].styleDiagnostics must be a string array`,
        );
      }
      if (
        !Array.isArray(node.attrs) ||
        !node.attrs.every(
          (item) =>
            Array.isArray(item) &&
            item.length === 2 &&
            item.every((part) => typeof part === "string"),
        )
      ) {
        throw new Error(
          `${capture.snapshot}.nodes[${index}].attrs must be string pairs`,
        );
      }
      if (
        node.computed === null ||
        typeof node.computed !== "object" ||
        Array.isArray(node.computed)
      ) {
        throw new Error(
          `${capture.snapshot}.nodes[${index}].computed must be an object`,
        );
      }
      const computed = node.computed as Record<string, unknown>;
      return {
        id: snapshotNodeKey(node.id, `${capture.snapshot}.nodes[${index}].id`),
        parentId:
          node.parentId === null
            ? null
            : snapshotNodeKey(
                node.parentId,
                `${capture.snapshot}.nodes[${index}].parentId`,
              ),
        tag:
          optionalString(node.tag, `${capture.snapshot}.nodes[${index}].tag`) ??
          "",
        text: optionalString(
          node.text,
          `${capture.snapshot}.nodes[${index}].text`,
        ),
        classes: node.classes,
        styleDiagnostics: node.styleDiagnostics,
        attrs: node.attrs,
        rect: snapshotRect(
          node.rect,
          `${capture.snapshot}.nodes[${index}].rect`,
        ),
        contentRect: snapshotRect(
          node.contentRect,
          `${capture.snapshot}.nodes[${index}].contentRect`,
        ),
        computed: {
          overflowX: optionalString(
            computed.overflowX,
            `${capture.snapshot}.nodes[${index}].computed.overflowX`,
          ),
          overflowY: optionalString(
            computed.overflowY,
            `${capture.snapshot}.nodes[${index}].computed.overflowY`,
          ),
        },
      };
    }),
  };
  if (
    parsed.status.viewportWidth !== capture.width ||
    parsed.status.viewportHeight !== capture.height
  ) {
    throw new Error(
      `${capture.snapshot} viewport ${parsed.status.viewportWidth}x${parsed.status.viewportHeight} does not match requested ${capture.width}x${capture.height}`,
    );
  }
  if (parsed.status.deviceScale !== capture.scaleFactor) {
    throw new Error(
      `${capture.snapshot} scale ${parsed.status.deviceScale} does not match requested ${capture.scaleFactor}`,
    );
  }
  if (
    parsed.status.nodeCount !== parsed.nodes.length ||
    parsed.nodes.length === 0
  ) {
    throw new Error(
      `${capture.snapshot} node count ${parsed.status.nodeCount} does not match ${parsed.nodes.length} retained nodes`,
    );
  }
  validateSnapshotGraph(parsed, capture.snapshot);
  return parsed;
}

function nodeKey(key: SnapshotNodeKey): string {
  return `${key.lo}:${key.hi}`;
}

function validateSnapshotGraph(snapshot: CaptureSnapshot, name: string): void {
  const nodes = new Map<string, CaptureSnapshotNode>();
  for (const node of snapshot.nodes) {
    const key = nodeKey(node.id);
    if (nodes.has(key))
      throw new Error(`${name} contains duplicate node id ${key}`);
    nodes.set(key, node);
  }
  if (!snapshot.nodes.some((node) => node.parentId === null)) {
    throw new Error(`${name} has no retained-tree root`);
  }
  for (const node of snapshot.nodes) {
    const visited = new Set<string>();
    let current: CaptureSnapshotNode | undefined = node;
    while (current?.parentId) {
      const currentKey = nodeKey(current.id);
      if (visited.has(currentKey)) {
        throw new Error(
          `${name} contains a parent cycle at node ${currentKey}`,
        );
      }
      visited.add(currentKey);
      const parentKey = nodeKey(current.parentId);
      current = nodes.get(parentKey);
      if (!current) {
        throw new Error(
          `${name} node ${nodeKey(node.id)} references missing parent ${parentKey}`,
        );
      }
    }
  }
}

function overflowAmount(inner: SnapshotRect, outer: SnapshotRect): number {
  return Math.max(
    outer.x - inner.x,
    outer.y - inner.y,
    inner.x + inner.width - (outer.x + outer.width),
    inner.y + inner.height - (outer.y + outer.height),
  );
}

export function textContainmentDiagnostics(
  snapshot: CaptureSnapshot,
  tolerance = 1,
): string[] {
  const nodes = new Map(snapshot.nodes.map((node) => [nodeKey(node.id), node]));
  const diagnostics: string[] = [];
  for (const node of snapshot.nodes) {
    if (node.text === null) continue;
    let ancestor = node.parentId
      ? nodes.get(nodeKey(node.parentId))
      : undefined;
    while (ancestor) {
      if (
        ancestor.computed.overflowX !== "Visible" ||
        ancestor.computed.overflowY !== "Visible"
      ) {
        break;
      }
      const overflow = overflowAmount(node.rect, ancestor.rect);
      if (overflow > tolerance) {
        diagnostics.push(
          `text ${JSON.stringify(node.text)} (${node.tag} ${nodeKey(node.id)}) exceeds ancestor ${ancestor.tag} ${nodeKey(ancestor.id)} by ${overflow.toFixed(1)}px; ancestor classes: ${ancestor.classes.join(" ") || "<none>"}`,
        );
        break;
      }
      ancestor = ancestor.parentId
        ? nodes.get(nodeKey(ancestor.parentId))
        : undefined;
    }
  }
  return diagnostics;
}

export function rejectedStyleDiagnostics(snapshot: CaptureSnapshot): string[] {
  return snapshot.nodes.flatMap((node) =>
    node.styleDiagnostics.map(
      (diagnostic) =>
        `${node.tag} ${nodeKey(node.id)} (${node.classes.join(" ") || "no classes"}): ${diagnostic}`,
    ),
  );
}

const namedRoles = new Set([
  "button",
  "checkbox",
  "combobox",
  "dialog",
  "img",
  "link",
  "listbox",
  "menuitem",
  "menuitemcheckbox",
  "menuitemradio",
  "option",
  "radio",
  "searchbox",
  "slider",
  "spinbutton",
  "switch",
  "tab",
  "textbox",
  "treeitem",
]);
const namedTags = new Set(["button", "input", "select", "textarea"]);

function nodeAttrs(node: CaptureSnapshotNode): Map<string, string> {
  return new Map(node.attrs);
}

export function accessibleNameDiagnostics(snapshot: CaptureSnapshot): string[] {
  const children = new Map<string, CaptureSnapshotNode[]>();
  const ids = new Map<string, CaptureSnapshotNode>();
  for (const node of snapshot.nodes) {
    if (node.parentId) {
      const key = nodeKey(node.parentId);
      const siblings = children.get(key) ?? [];
      siblings.push(node);
      children.set(key, siblings);
    }
    const id = nodeAttrs(node).get("id")?.trim();
    if (id) ids.set(id, node);
  }
  const textContent = (node: CaptureSnapshotNode): string => {
    if (nodeAttrs(node).get("aria-hidden") === "true") return "";
    return [
      node.text ?? "",
      ...(children.get(nodeKey(node.id)) ?? []).map(textContent),
    ]
      .join(" ")
      .trim();
  };
  const diagnostics: string[] = [];
  for (const node of snapshot.nodes) {
    const attrs = nodeAttrs(node);
    if (attrs.get("aria-hidden") === "true") continue;
    const role = attrs.get("role") ?? node.tag;
    if (role === "none" || role === "presentation") continue;
    if (!namedRoles.has(role) && !namedTags.has(node.tag)) continue;
    const direct = attrs.get("aria-label")?.trim() ?? "";
    const references = (attrs.get("aria-labelledby") ?? "")
      .split(/\s+/u)
      .filter(Boolean);
    const referenced = references
      .map((id) => ids.get(id))
      .filter((label): label is CaptureSnapshotNode => label !== undefined)
      .map(textContent)
      .join(" ")
      .trim();
    if (!direct && !referenced && !textContent(node)) {
      diagnostics.push(
        `${node.tag} ${nodeKey(node.id)} with role ${role} has no aria-label, resolved aria-labelledby, or descendant text`,
      );
    }
  }
  return diagnostics;
}

export async function validateCaptureArtifacts(
  capture: CaptureCase,
  workspaceRoot = root,
): Promise<void> {
  const output = resolve(workspaceRoot, capture.output);
  if (!(await Bun.file(output).exists()) || Bun.file(output).size === 0) {
    throw new Error(
      `capture did not produce ${relative(workspaceRoot, output)}`,
    );
  }
  const dimensions = pngDimensions(
    new Uint8Array(await readFile(output)),
    capture.output,
  );
  const expectedWidth = Math.max(
    1,
    Math.round(capture.width * capture.scaleFactor),
  );
  const expectedHeight = Math.max(
    1,
    Math.round(capture.height * capture.scaleFactor),
  );
  if (
    dimensions.width !== expectedWidth ||
    dimensions.height !== expectedHeight
  ) {
    throw new Error(
      `${capture.output} physical size ${dimensions.width}x${dimensions.height} does not match logical ${capture.width}x${capture.height} at ${capture.scaleFactor}x (${expectedWidth}x${expectedHeight})`,
    );
  }
  const snapshot = resolve(workspaceRoot, capture.snapshot);
  if (!(await Bun.file(snapshot).exists()) || Bun.file(snapshot).size === 0) {
    throw new Error(
      `capture did not produce ${relative(workspaceRoot, snapshot)}`,
    );
  }
  const parsed = validateCaptureSnapshot(
    JSON.parse(await readFile(snapshot, "utf8")),
    capture,
  );
  if (capture.checkTextContainment) {
    const diagnostics = textContainmentDiagnostics(parsed);
    if (diagnostics.length > 0) {
      throw new Error(
        `${relative(workspaceRoot, snapshot)} has visible text overflow:\n${diagnostics.map((item) => `  - ${item}`).join("\n")}`,
      );
    }
  }
  if (capture.checkStyleDiagnostics) {
    const diagnostics = rejectedStyleDiagnostics(parsed);
    if (diagnostics.length > 0) {
      throw new Error(
        `${relative(workspaceRoot, snapshot)} has rejected styles:\n${diagnostics.map((item) => `  - ${item}`).join("\n")}`,
      );
    }
  }
  if (capture.checkAccessibleNames) {
    const diagnostics = accessibleNameDiagnostics(parsed);
    if (diagnostics.length > 0) {
      throw new Error(
        `${relative(workspaceRoot, snapshot)} has unnamed semantic controls:\n${diagnostics.map((item) => `  - ${item}`).join("\n")}`,
      );
    }
  }
}

export function pngDimensions(bytes: Uint8Array, name = "PNG"): PngDimensions {
  const signature = [137, 80, 78, 71, 13, 10, 26, 10];
  if (
    bytes.length < 24 ||
    signature.some((byte, index) => bytes[index] !== byte) ||
    bytes[12] !== 73 ||
    bytes[13] !== 72 ||
    bytes[14] !== 68 ||
    bytes[15] !== 82
  ) {
    throw new Error(`${name} is not a PNG with an IHDR header`);
  }
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const width = view.getUint32(16, false);
  const height = view.getUint32(20, false);
  if (width === 0 || height === 0) {
    throw new Error(`${name} has invalid zero dimensions`);
  }
  return { width, height };
}

function relativeScenarioPath(scenario: string): string {
  return scenario.split("/captures/")[1] ?? scenario;
}

export interface CaptureArguments {
  list: boolean;
  checkExisting: boolean;
  scenarios: string[];
}

export function parseCaptureArguments(arguments_: string[]): CaptureArguments {
  const parsed: CaptureArguments = {
    list: false,
    checkExisting: false,
    scenarios: [],
  };
  for (let index = 0; index < arguments_.length; index++) {
    const argument = arguments_[index];
    if (argument === "--list") parsed.list = true;
    else if (argument === "--check-existing") parsed.checkExisting = true;
    else if (argument === "--scenario") {
      const scenario = arguments_[++index];
      if (!scenario || scenario.startsWith("--")) {
        throw new Error("--scenario requires an apps/*/captures/**/*.ts path");
      }
      parsed.scenarios.push(scenario.replaceAll("\\", "/"));
    } else {
      throw new Error(`unsupported argument ${argument}`);
    }
  }
  if (parsed.list && (parsed.checkExisting || parsed.scenarios.length > 0)) {
    throw new Error(
      "--list cannot be combined with capture selection or checking",
    );
  }
  return parsed;
}

export function selectCaptureCases(
  discovered: CaptureCase[],
  scenarios: string[],
): CaptureCase[] {
  if (scenarios.length === 0) return discovered;
  const selected = new Set(scenarios);
  const captures = discovered.filter((capture) =>
    selected.has(capture.scenario),
  );
  if (captures.length !== selected.size) {
    const missing = [...selected].filter(
      (scenario) => !captures.some((capture) => capture.scenario === scenario),
    );
    throw new Error(
      `unknown capture scenario${missing.length === 1 ? "" : "s"}: ${missing.join(", ")}`,
    );
  }
  return captures;
}

async function main(): Promise<void> {
  const arguments_ = parseCaptureArguments(process.argv.slice(2));
  const discovered = await discoverCaptureCases();
  if (discovered.length === 0) {
    throw new Error("no apps/*/captures/**/*.ts scenarios were discovered");
  }
  if (arguments_.list) {
    console.log(JSON.stringify(discovered, null, 2));
    return;
  }
  const captures = selectCaptureCases(discovered, arguments_.scenarios);

  const checkExisting = arguments_.checkExisting;
  const builtApplications = new Set<string>();
  for (const capture of captures) {
    const output = resolve(root, capture.output);
    if (!checkExisting) {
      await mkdir(dirname(output), { recursive: true });
      console.log(`[capture] rendering ${capture.scenario}`);
      const args = captureCommand(
        capture,
        builtApplications.has(capture.application),
      );
      const child = Bun.spawn(args, {
        cwd: root,
        stdin: "inherit",
        stdout: "inherit",
        stderr: "inherit",
      });
      const exitCode = await child.exited;
      if (exitCode !== 0) process.exit(exitCode);
      builtApplications.add(capture.application);
    }
    await validateCaptureArtifacts(capture);
  }
  console.log(
    `${checkExisting ? "checked existing artifacts for" : "verified"} ${captures.length} authored captures`,
  );
}

if (import.meta.main) await main();
