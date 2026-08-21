export interface LayoutNodeKey {
  readonly lo: number;
  readonly hi: number;
}

export interface LayoutRect {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

export interface LayoutComputedStyle {
  readonly position?: string | null;
  readonly overflowX?: string | null;
  readonly overflowY?: string | null;
  readonly overlayPlane?: string;
}

export interface LayoutSemanticProjection {
  readonly role: string;
  readonly label?: string | null;
}

export interface LayoutSnapshotNode {
  readonly id: LayoutNodeKey;
  readonly parentId?: LayoutNodeKey | null;
  readonly tag: string;
  readonly text?: string | null;
  readonly classes: readonly string[];
  readonly attrs: readonly (readonly [string, string])[];
  readonly rect: LayoutRect;
  readonly contentRect: LayoutRect;
  readonly styleDiagnostics: readonly string[];
  readonly semantic?: LayoutSemanticProjection | null;
  readonly computed: LayoutComputedStyle;
}

export interface LayoutSnapshot {
  readonly status: {
    readonly viewportWidth: number;
    readonly viewportHeight: number;
    readonly deviceScale: number;
    readonly nodeCount: number;
  };
  readonly nodes: readonly LayoutSnapshotNode[];
}

export interface LayoutQuery {
  readonly tag?: string;
  readonly role?: string;
  readonly name?: string;
  readonly text?: string;
  readonly className?: string;
}

export interface LayoutDiagnostic {
  readonly code:
    | "flow-sibling-overlap"
    | "style-diagnostic"
    | "visible-overflow";
  readonly message: string;
  readonly node: LayoutSnapshotNode;
  readonly related?: LayoutSnapshotNode;
  readonly amount?: number;
}

export interface LayoutDiagnosticOptions {
  readonly tolerance?: number;
  /** Restrict checks to descendants of this node, including itself. */
  readonly within?: LayoutSnapshotNode;
}

function record(value: unknown, path: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`invalid layout snapshot: ${path} must be an object`);
  }
  return value as Record<string, unknown>;
}

function finiteNumber(value: unknown, path: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new Error(`invalid layout snapshot: ${path} must be a finite number`);
  }
  return value;
}

function parseKey(value: unknown, path: string): LayoutNodeKey {
  const item = record(value, path);
  return {
    lo: finiteNumber(item.lo, `${path}.lo`),
    hi: finiteNumber(item.hi, `${path}.hi`),
  };
}

function parseRect(value: unknown, path: string): LayoutRect {
  const item = record(value, path);
  return {
    x: finiteNumber(item.x, `${path}.x`),
    y: finiteNumber(item.y, `${path}.y`),
    width: finiteNumber(item.width, `${path}.width`),
    height: finiteNumber(item.height, `${path}.height`),
  };
}

/** Validate the Rust snapshot boundary before layout assertions consume it. */
export function parseLayoutSnapshot(value: unknown): LayoutSnapshot {
  const root = record(value, "root");
  const status = record(root.status, "status");
  if (!Array.isArray(root.nodes)) {
    throw new Error("invalid layout snapshot: nodes must be an array");
  }
  const nodes = root.nodes.map((value, index): LayoutSnapshotNode => {
    const path = `nodes[${index}]`;
    const item = record(value, path);
    if (typeof item.tag !== "string") {
      throw new Error(`invalid layout snapshot: ${path}.tag must be a string`);
    }
    if (
      !Array.isArray(item.classes) ||
      !item.classes.every((v) => typeof v === "string")
    ) {
      throw new Error(
        `invalid layout snapshot: ${path}.classes must be a string array`,
      );
    }
    if (!Array.isArray(item.attrs)) {
      throw new Error(
        `invalid layout snapshot: ${path}.attrs must be an array`,
      );
    }
    if (
      !Array.isArray(item.styleDiagnostics) ||
      !item.styleDiagnostics.every((v) => typeof v === "string")
    ) {
      throw new Error(
        `invalid layout snapshot: ${path}.styleDiagnostics must be a string array`,
      );
    }
    return {
      ...(item as unknown as LayoutSnapshotNode),
      id: parseKey(item.id, `${path}.id`),
      parentId:
        item.parentId === null || item.parentId === undefined
          ? null
          : parseKey(item.parentId, `${path}.parentId`),
      tag: item.tag,
      classes: item.classes,
      attrs: item.attrs as LayoutSnapshotNode["attrs"],
      rect: parseRect(item.rect, `${path}.rect`),
      contentRect: parseRect(item.contentRect, `${path}.contentRect`),
      styleDiagnostics: item.styleDiagnostics,
      computed: record(item.computed, `${path}.computed`),
    };
  });
  return {
    status: {
      viewportWidth: finiteNumber(status.viewportWidth, "status.viewportWidth"),
      viewportHeight: finiteNumber(
        status.viewportHeight,
        "status.viewportHeight",
      ),
      deviceScale: finiteNumber(status.deviceScale, "status.deviceScale"),
      nodeCount: finiteNumber(status.nodeCount, "status.nodeCount"),
    },
    nodes,
  };
}

const key = (id: LayoutNodeKey): string => `${id.lo}:${id.hi}`;

function attrs(node: LayoutSnapshotNode): Map<string, string> {
  return new Map(node.attrs);
}

export function layoutRole(node: LayoutSnapshotNode): string {
  return node.semantic?.role ?? attrs(node).get("role") ?? "";
}

export function layoutName(node: LayoutSnapshotNode): string {
  return node.semantic?.label ?? attrs(node).get("aria-label") ?? "";
}

function childrenByParent(snapshot: LayoutSnapshot) {
  const children = new Map<string, LayoutSnapshotNode[]>();
  for (const node of snapshot.nodes) {
    const parent = node.parentId ? key(node.parentId) : "<root>";
    const items = children.get(parent) ?? [];
    items.push(node);
    children.set(parent, items);
  }
  return children;
}

function scopedNodes(
  snapshot: LayoutSnapshot,
  within?: LayoutSnapshotNode,
): readonly LayoutSnapshotNode[] {
  if (!within) return snapshot.nodes;
  const children = childrenByParent(snapshot);
  const result: LayoutSnapshotNode[] = [];
  const visit = (node: LayoutSnapshotNode) => {
    result.push(node);
    for (const child of children.get(key(node.id)) ?? []) visit(child);
  };
  visit(within);
  return result;
}

export function queryLayoutNodes(
  snapshot: LayoutSnapshot,
  query: LayoutQuery,
): readonly LayoutSnapshotNode[] {
  return snapshot.nodes.filter((node) => {
    if (query.tag !== undefined && node.tag !== query.tag) return false;
    if (query.role !== undefined && layoutRole(node) !== query.role)
      return false;
    if (query.name !== undefined && layoutName(node) !== query.name)
      return false;
    if (query.text !== undefined && node.text !== query.text) return false;
    if (
      query.className !== undefined &&
      !node.classes.includes(query.className)
    )
      return false;
    return true;
  });
}

export function getLayoutNode(
  snapshot: LayoutSnapshot,
  query: LayoutQuery,
): LayoutSnapshotNode {
  const matches = queryLayoutNodes(snapshot, query);
  if (matches.length !== 1) {
    throw new Error(
      `expected one layout node for ${JSON.stringify(query)}, found ${matches.length}`,
    );
  }
  return matches[0];
}

function depthOf(
  node: LayoutSnapshotNode,
  nodes: ReadonlyMap<string, LayoutSnapshotNode>,
): number {
  let depth = 0;
  let parent = node.parentId ? nodes.get(key(node.parentId)) : undefined;
  while (parent && depth < nodes.size) {
    depth += 1;
    parent = parent.parentId ? nodes.get(key(parent.parentId)) : undefined;
  }
  return depth;
}

const rectText = (rect: LayoutRect) =>
  `${rect.x.toFixed(1)},${rect.y.toFixed(1)} ${rect.width.toFixed(1)}x${rect.height.toFixed(1)}`;

/** Stable text projection of the exact Style IR + Taffy result. */
export function formatLayoutTree(snapshot: LayoutSnapshot): string {
  const nodes = new Map(snapshot.nodes.map((node) => [key(node.id), node]));
  const lines = [
    `viewport ${snapshot.status.viewportWidth}x${snapshot.status.viewportHeight} scale=${snapshot.status.deviceScale} nodes=${snapshot.nodes.length}`,
  ];
  for (const node of snapshot.nodes) {
    const parts = [
      `${"  ".repeat(depthOf(node, nodes))}${node.tag}#${key(node.id)}`,
    ];
    const role = layoutRole(node);
    const name = layoutName(node);
    if (role) parts.push(`role=${role}`);
    if (name) parts.push(`name=${JSON.stringify(name)}`);
    if (node.text) parts.push(`text=${JSON.stringify(node.text.slice(0, 80))}`);
    parts.push(`rect=(${rectText(node.rect)})`);
    parts.push(`content=(${rectText(node.contentRect)})`);
    const overflowX = node.computed.overflowX ?? "Visible";
    const overflowY = node.computed.overflowY ?? "Visible";
    if (overflowX !== "Visible" || overflowY !== "Visible")
      parts.push(`overflow=${overflowX}/${overflowY}`);
    if (node.classes.length > 0)
      parts.push(`class=${JSON.stringify(node.classes.join(" "))}`);
    lines.push(parts.join(" "));
  }
  return `${lines.join("\n")}\n`;
}

function overflowAmount(outer: LayoutRect, inner: LayoutRect): number {
  return Math.max(
    0,
    outer.x - inner.x,
    outer.y - inner.y,
    inner.x + inner.width - (outer.x + outer.width),
    inner.y + inner.height - (outer.y + outer.height),
  );
}

export function visibleOverflowDiagnostics(
  snapshot: LayoutSnapshot,
  options: LayoutDiagnosticOptions = {},
): readonly LayoutDiagnostic[] {
  const tolerance = options.tolerance ?? 1;
  const nodes = new Map(snapshot.nodes.map((node) => [key(node.id), node]));
  const diagnostics: LayoutDiagnostic[] = [];
  for (const node of scopedNodes(snapshot, options.within)) {
    let parent = node.parentId ? nodes.get(key(node.parentId)) : undefined;
    while (parent) {
      if (
        (parent.computed.overflowX ?? "Visible") !== "Visible" ||
        (parent.computed.overflowY ?? "Visible") !== "Visible"
      )
        break;
      const amount = overflowAmount(parent.rect, node.rect);
      if (amount > tolerance) {
        diagnostics.push({
          code: "visible-overflow",
          node,
          related: parent,
          amount,
          message: `${node.tag} ${key(node.id)} extends ${amount.toFixed(1)}px outside ${parent.tag} ${key(parent.id)}`,
        });
        break;
      }
      parent = parent.parentId ? nodes.get(key(parent.parentId)) : undefined;
    }
  }
  return diagnostics;
}

function overlaps(first: LayoutRect, second: LayoutRect, tolerance: number) {
  return (
    first.width > 0 &&
    first.height > 0 &&
    second.width > 0 &&
    second.height > 0 &&
    first.x + first.width > second.x + tolerance &&
    second.x + second.width > first.x + tolerance &&
    first.y + first.height > second.y + tolerance &&
    second.y + second.height > first.y + tolerance
  );
}

/** Opt-in collision check for normal-flow siblings. */
export function siblingCollisionDiagnostics(
  snapshot: LayoutSnapshot,
  options: LayoutDiagnosticOptions = {},
): readonly LayoutDiagnostic[] {
  const tolerance = options.tolerance ?? 0.5;
  const scope = new Set(
    scopedNodes(snapshot, options.within).map((node) => key(node.id)),
  );
  const children = childrenByParent(snapshot);
  const diagnostics: LayoutDiagnostic[] = [];
  for (const siblings of children.values()) {
    const flow = siblings.filter(
      (node) =>
        scope.has(key(node.id)) && node.computed.position !== "Absolute",
    );
    for (let index = 0; index < flow.length; index += 1) {
      for (const second of flow.slice(index + 1)) {
        const first = flow[index];
        if (
          first.computed.overlayPlane !== second.computed.overlayPlane ||
          !overlaps(first.rect, second.rect, tolerance)
        )
          continue;
        diagnostics.push({
          code: "flow-sibling-overlap",
          node: second,
          related: first,
          message: `${first.tag} ${key(first.id)} overlaps sibling ${second.tag} ${key(second.id)}`,
        });
      }
    }
  }
  return diagnostics;
}

export function styleDiagnostics(
  snapshot: LayoutSnapshot,
  options: LayoutDiagnosticOptions = {},
): readonly LayoutDiagnostic[] {
  return scopedNodes(snapshot, options.within).flatMap((node) =>
    node.styleDiagnostics.map((message) => ({
      code: "style-diagnostic" as const,
      node,
      message,
    })),
  );
}

export function assertNoLayoutDiagnostics(
  diagnostics: readonly LayoutDiagnostic[],
): void {
  if (diagnostics.length === 0) return;
  throw new Error(
    `layout diagnostics:\n${diagnostics.map((item) => `  - [${item.code}] ${item.message}`).join("\n")}`,
  );
}
