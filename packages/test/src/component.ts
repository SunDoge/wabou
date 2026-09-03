import { PlatformProvider, type PlatformServices } from "@wabou/core";
import {
  type EventType,
  INTERACTION_POLICY,
  type NodeKey,
} from "@wabou/core/protocol";
import {
  type BuiltinHost,
  dispatchEvent,
  EVENT_CODE,
  type Host,
  HostProvider,
  mount,
  writer,
} from "@wabou/core/renderer";
import { dispatchResizeObservation } from "@wabou/core/testing";
import { createComponent, flush as flushSolid, type JSX } from "solid-js";
import { onTestFinished, vi } from "vitest";

interface AuthoredNode {
  readonly id: NodeKey;
  readonly tag: string;
  parent: AuthoredNode | null;
  readonly children: AuthoredNode[];
  readonly attributes: Map<string, string>;
  focusOrder: number | null;
  interactionBlocked: boolean;
  focusContained: boolean;
  projectionBoundary: boolean;
  overlayPlane: ComponentOverlayPlane;
  className: string;
  readonly styles: Map<string, ComponentStyleValue>;
  widgetConfig: unknown;
  transform: readonly [number, number, number, number, number, number] | null;
  text: string;
}

export type ComponentOverlayPlane = "content" | "floating" | "modal";

export interface ComponentTypedStyleValue {
  readonly kind: number;
  readonly value: number;
}

export type ComponentStyleValue = string | ComponentTypedStyleValue;

export interface ComponentRoleListOptions {
  name?: string;
  disabled?: boolean;
  readOnly?: boolean;
  checked?: boolean | "mixed";
  selected?: boolean;
  expanded?: boolean;
  pressed?: boolean | "mixed";
  busy?: boolean;
  current?: boolean | string;
  orientation?: "horizontal" | "vertical";
  focused?: boolean;
}

export interface ComponentRoleQueryOptions extends ComponentRoleListOptions {
  /** Select one occurrence in depth-first authored order. */
  index?: number;
}

export interface ComponentQueries {
  getByRole(
    role: string,
    options?: ComponentRoleQueryOptions,
  ): ComponentLocator;
  queryByRole(
    role: string,
    options?: ComponentRoleQueryOptions,
  ): ComponentLocator | null;
  getAllByRole(
    role: string,
    options?: ComponentRoleListOptions,
  ): readonly ComponentLocator[];
  queryAllByRole(
    role: string,
    options?: ComponentRoleListOptions,
  ): readonly ComponentLocator[];
}

export interface ComponentSnapshotNode {
  readonly tag: string;
  readonly role?: string;
  readonly name?: string;
  readonly text?: string;
  readonly className?: string;
  readonly attributes?: Readonly<Record<string, string>>;
  readonly styles?: Readonly<Record<string, ComponentStyleValue>>;
  readonly focusOrder?: number;
  readonly interactionBlocked?: true;
  readonly focusContained?: true;
  readonly projectionBoundary?: true;
  readonly overlayPlane?: Exclude<ComponentOverlayPlane, "content">;
  readonly transform?: readonly [
    number,
    number,
    number,
    number,
    number,
    number,
  ];
  readonly children?: readonly ComponentSnapshotNode[];
}

export interface ComponentLocator extends ComponentQueries {
  /** Exact retained-node identity. Changes prove that the component remounted. */
  readonly identity: Readonly<{ lo: number; hi: number }>;
  /** Direct authored parent, or null at the component render root. */
  readonly parent: ComponentLocator | null;
  readonly tag: string;
  readonly role: string;
  readonly name: string;
  readonly text: string;
  readonly className: string;
  /** Last authored string or typed value emitted for an inline style property. */
  style(name: string): ComponentStyleValue | null;
  /** Last structured configuration authored for a native widget. */
  readonly widgetConfig: unknown;
  /** Direct authored children for visual protocol assertions. Prefer role queries for behavior. */
  readonly children: readonly ComponentLocator[];
  /** Stable authored protocol tree without transient NodeKeys. */
  snapshot(): ComponentSnapshotNode;
  /** Find the nearest attached self-or-ancestor matching an authored role. */
  closestByRole(
    role: string,
    options?: ComponentRoleListOptions,
  ): ComponentLocator | null;
  /** Disabled state as authored through `disabled` or `aria-disabled`. */
  readonly disabled: boolean;
  /** Read-only state as authored through `readOnly` or `aria-readonly`. */
  readonly readOnly: boolean;
  /** Toggle state authored through `aria-checked`. */
  readonly checked: boolean | "mixed" | null;
  /** Selection state authored through `aria-selected`. */
  readonly selected: boolean | null;
  /** Disclosure state authored through `aria-expanded`. */
  readonly expanded: boolean | null;
  /** Toggle-button state authored through `aria-pressed`. */
  readonly pressed: boolean | "mixed" | null;
  /** Pending state authored through `aria-busy`. */
  readonly busy: boolean;
  /** Current item state authored through `aria-current`. */
  readonly current: boolean | string | null;
  /** Component axis authored through `aria-orientation`. */
  readonly orientation: "horizontal" | "vertical" | null;
  /** Authored textual value, including an input's controlled display value. */
  readonly value: string | null;
  /** Numeric range state authored through `aria-valuenow`. */
  readonly numericValue: number | null;
  /** Lower numeric range bound authored through `aria-valuemin`. */
  readonly minNumericValue: number | null;
  /** Upper numeric range bound authored through `aria-valuemax`. */
  readonly maxNumericValue: number | null;
  /** Human-readable numeric value authored through `aria-valuetext`. */
  readonly valueText: string | null;
  /** Last runtime affine transform emitted through the native protocol. */
  readonly transform:
    | readonly [number, number, number, number, number, number]
    | null;
  /** Whether this locator owns the harness's native focus simulation. */
  readonly focused: boolean;
  /** Native tab order emitted through Wabou's interaction policy protocol. */
  readonly focusOrder: number | null;
  /** Whether native pointer and keyboard routing is blocked for this subtree. */
  readonly interactionBlocked: boolean;
  /** Whether native focus traversal is contained by this subtree. */
  readonly focusContained: boolean;
  /** Native stacking plane authored for this node. */
  readonly overlayPlane: ComponentOverlayPlane;
  /** Whether this node owns an explicit retained GPUI projection boundary. */
  readonly projectionBoundary: boolean;
  attribute(name: string): string | null;
  pointerDown(position?: ComponentPointerPosition): void;
  /** Dispatch an uncaptured native pointer move with no pressed buttons. */
  movePointer(position?: ComponentPointerPosition): void;
  /** Dispatch a captured native pointer move while preserving button state. */
  pointerMove(position?: ComponentPointerPosition): void;
  pointerUp(position?: ComponentPointerPosition): void;
  click(): void;
  /** Dispatch a secondary-click context-menu event at a deterministic point. */
  contextMenu(position?: ComponentPointerPosition): void;
  press(key: string, options?: ComponentKeyOptions): void;
  input(value: string): void;
  /** Dispatch a typed host event for custom-widget/component contracts. */
  emit(type: EventType, payload?: unknown): void;
  /** Complete the native transition currently authored by this node. */
  finishNativeTransition(): void;
  /** Dispatch native focus/focusin, blurring the previously focused locator. */
  focus(): void;
  /** Dispatch native blur/focusout when this locator owns focus. */
  blur(): void;
  hover(): void;
  unhover(): void;
  /** Publish a deterministic native content-box observation. */
  resize(size: { width: number; height: number }): void;
}

export interface ComponentPointerPosition {
  clientX?: number;
  clientY?: number;
  offsetX?: number;
  offsetY?: number;
}

export interface ComponentKeyOptions {
  code?: string;
  /** Physical Shift, Control, Alt, and Meta modifier bits. */
  mods?: number;
  primary?: boolean;
  repeat?: boolean;
}

export interface ComponentScreen extends ComponentQueries {
  /** Current top-level authored nodes, including synthetic overlay roots. */
  readonly roots: readonly ComponentLocator[];
  /** Stable authored protocol forest suitable for Vitest snapshots. */
  snapshot(): readonly ComponentSnapshotNode[];
  /** Commit reactive work scheduled outside a locator action, such as a timer. */
  flush(): void;
  /** Advance a harness-owned fake clock and commit resulting reactive work. */
  advanceTime(milliseconds: number): Promise<void>;
  /** Retry an assertion while committing Promise-driven component updates. */
  waitFor<T>(
    assertion: () => T | Promise<T>,
    options?: ComponentWaitForOptions,
  ): Promise<T>;
  dispose(): void;
}

export interface ComponentWaitForOptions {
  /** Total retry budget in milliseconds. Defaults to 1000. */
  timeout?: number;
  /** Retry interval in milliseconds. Defaults to 10. */
  interval?: number;
}

export interface RenderComponentOptions {
  /** Host fixture injected into the component subtree. */
  host?: Host;
  /** Native platform services overridden for this component subtree. */
  platform?: Partial<PlatformServices>;
  /** Use a fake clock owned and restored by this component screen. */
  clock?: "real" | "fake";
}

export interface TestHostCall {
  readonly path: string;
  readonly args: readonly unknown[];
}

export interface TestHostFixture<H extends Host> {
  readonly host: H;
  readonly calls: readonly TestHostCall[];
  callsTo(path: string): readonly TestHostCall[];
  clearCalls(): void;
}

export interface TestBuiltinHost {
  system?: Partial<BuiltinHost["system"]>;
  fonts?: Partial<BuiltinHost["fonts"]>;
  diagnostics?: Partial<BuiltinHost["diagnostics"]>;
  intl?: Partial<BuiltinHost["intl"]>;
  layout?: Partial<BuiltinHost["layout"]>;
}

function componentDescendants(
  root: ComponentLocator,
): readonly ComponentLocator[] {
  const nodes: ComponentLocator[] = [];
  const visit = (node: ComponentLocator) => {
    nodes.push(node);
    node.children.forEach(visit);
  };
  visit(root);
  return nodes;
}

function locatorDescription(locator: ComponentLocator): string {
  const role = locator.role ? ` role=${JSON.stringify(locator.role)}` : "";
  const name = locator.name ? ` name=${JSON.stringify(locator.name)}` : "";
  return `<${locator.tag}${role}${name}>`;
}

function ownsComponentResponsibility(
  locator: ComponentLocator,
  responsibility: string,
): boolean {
  return (locator.attribute("data-wabou-owns") ?? "")
    .split(/\s+/u)
    .includes(responsibility);
}

/**
 * Assert that a compound control has one background owner.
 *
 * Transparent descendants are content layers, not surface owners. This catches
 * accidental combinations such as an InputGroup and its native input both
 * painting `bg-input`.
 */
export function assertSingleSurfaceOwner(
  root: ComponentLocator,
): ComponentLocator {
  const descendants = componentDescendants(root);
  const owners = descendants.filter((locator) =>
    ownsComponentResponsibility(locator, "surface"),
  );
  if (owners.length !== 1) {
    throw new Error(
      `${locatorDescription(root)} must have exactly one visible surface owner; found ${owners.length}: ${owners.map(locatorDescription).join(", ") || "none"}`,
    );
  }
  const forbiddenClass = /^(?:bg|border|rounded|shadow)(?:-|$)/u;
  const forbiddenStyles = [
    "background",
    "background-color",
    "border",
    "border-width",
    "border-radius",
    "box-shadow",
  ] as const;
  for (const content of descendants) {
    if (
      content === owners[0] ||
      !ownsComponentResponsibility(content, "native-editor")
    ) {
      continue;
    }
    const classes = content.className
      .split(/\s+/u)
      .filter((candidate) => forbiddenClass.test(candidate));
    const styles = forbiddenStyles.filter(
      (property) => content.style(property) !== null,
    );
    if (classes.length > 0 || styles.length > 0) {
      throw new Error(
        `${locatorDescription(content)} is native content inside ${locatorDescription(owners[0])} and must not author visual chrome; found ${[...classes, ...styles].join(", ")}`,
      );
    }
  }
  return owners[0];
}

/** Assert an explicit number of native focus owners inside one composition. */
export function assertFocusOwnerCount(
  root: ComponentLocator,
  expected: number,
): readonly ComponentLocator[] {
  if (!Number.isInteger(expected) || expected < 0) {
    throw new RangeError(
      "expected focus owner count must be a non-negative integer",
    );
  }
  const owners = componentDescendants(root).filter(
    (locator) => locator.focusOrder !== null,
  );
  if (owners.length !== expected) {
    throw new Error(
      `${locatorDescription(root)} must have ${expected} native focus owner(s); found ${owners.length}: ${owners.map(locatorDescription).join(", ") || "none"}`,
    );
  }
  return owners;
}

/** Assert that a rendered overlay is attached to the requested native plane. */
export function assertInOverlayPlane(
  locator: ComponentLocator,
  expected: Exclude<ComponentOverlayPlane, "content">,
): void {
  let current: ComponentLocator | null = locator;
  while (current) {
    if (current.overlayPlane === expected) return;
    current = current.parent;
  }
  throw new Error(
    `${locatorDescription(locator)} is not mounted in the ${expected} overlay plane`,
  );
}

function missingHostMethod(path: string): never {
  throw new Error(`test host method ${path} is not configured`);
}

/** Create a typed, deterministic Host with automatic call recording. */
export function createTestHost<C extends object = Record<string, never>>(
  capabilities?: C,
  builtins: TestBuiltinHost = {},
): TestHostFixture<Host & C> {
  const calls: TestHostCall[] = [];
  const viewport = { x: 0, y: 0, width: 1_024, height: 768 };
  const unmeasured = { x: 0, y: 0, width: 0, height: 0 };
  const defaultLayout: BuiltinHost["layout"] = {
    snapshot: (targets) => ({
      revision: 0,
      viewport,
      nodes: targets.map((target) => ({
        id: "id" in target ? target.id : target,
        rect: unmeasured,
        clip: viewport,
        scroll: { offsetX: 0, offsetY: 0, rangeX: 0, rangeY: 0 },
      })),
    }),
    measure: () => unmeasured,
    clippingRect: () => viewport,
    viewport: () => viewport,
  };
  const base: BuiltinHost & C = Object.assign(
    {
      system: {
        openUrl: (url: string) => missingHostMethod(`system.openUrl(${url})`),
        ...builtins.system,
      },
      fonts: {
        load: (path: string) => missingHostMethod(`fonts.load(${path})`),
        ...builtins.fonts,
      },
      diagnostics: {
        frameStats: () => null,
        setOverlay: () => false,
        overlayPaintStats: () => null,
        ...builtins.diagnostics,
      },
      intl: {
        locale: () => "en-US",
        timeZone: () => "UTC",
        today: () => ({ year: 1970, month: 1, day: 1 }),
        ...builtins.intl,
      },
      layout: {
        ...defaultLayout,
        ...builtins.layout,
      },
    } satisfies BuiltinHost,
    capabilities ?? ({} as C),
  );
  const cache = new WeakMap<object, object>();
  const wrap = (value: object, path: string): object => {
    const cached = cache.get(value);
    if (cached) return cached;
    const methods = new Map<string, (...args: unknown[]) => unknown>();
    const proxy = new Proxy(value, {
      get(target, property, receiver) {
        const child = Reflect.get(target, property, receiver) as unknown;
        if (typeof property !== "string") return child;
        const childPath = path ? `${path}.${property}` : property;
        if (typeof child === "function") {
          const existing = methods.get(property);
          if (existing) return existing;
          const method = (...args: unknown[]) => {
            calls.push({ path: childPath, args });
            return Reflect.apply(child, target, args);
          };
          methods.set(property, method);
          return method;
        }
        if (child && typeof child === "object") return wrap(child, childPath);
        return child;
      },
    });
    cache.set(value, proxy);
    return proxy;
  };
  return {
    host: wrap(base, "") as Host & C,
    calls,
    callsTo: (path) => calls.filter((call) => call.path === path),
    clearCalls: () => calls.splice(0),
  };
}

let activeHarness = false;
let activeScreen: ComponentScreen | null = null;

/** Dispose the active component tree. Vitest users get this automatically. */
export function cleanupComponents(): void {
  activeScreen?.dispose();
}

const key = (id: NodeKey): string => `${id.lo}:${id.hi}`;

const implicitRole = (tag: string): string | null => {
  if (tag === "button") return "button";
  if (tag === "input") return "textbox";
  return null;
};

function installHostStub(name: string, stub: unknown = () => {}): () => void {
  const target = globalThis as Record<string, unknown>;
  const hadOwn = Object.hasOwn(target, name);
  const previous = target[name];
  if (typeof previous !== "function") target[name] = stub;
  return () => {
    if (hadOwn) target[name] = previous;
    else delete target[name];
  };
}

/**
 * Mount a component into Wabou's real Solid renderer while recording its
 * authored host tree. This is intentionally a fast component-contract test:
 * native layout, hit testing, and final semantic projection remain the job of
 * `wabou test` behavior scenarios.
 */
export function renderComponent(
  render: () => JSX.Element,
  options: RenderComponentOptions = {},
): ComponentScreen {
  if (
    options.clock !== undefined &&
    options.clock !== "real" &&
    options.clock !== "fake"
  ) {
    throw new RangeError(
      `unsupported component clock ${JSON.stringify(options.clock)}`,
    );
  }
  if (activeHarness) {
    throw new Error(
      "renderComponent supports one active component screen at a time",
    );
  }
  activeHarness = true;

  // Measurement is a normal component concern, not a reason to boot a native
  // host. The no-op subscription keeps the initial unmeasured state explicit;
  // native geometry remains covered by behavior tests.
  const restoreHostStubs = [
    installHostStub("__wabou_resize_observe"),
    installHostStub("__wabou_resize_unobserve"),
    installHostStub("__wabou_flush"),
    installHostStub("__wabou_log"),
    installHostStub(
      "__wabou_layout_snapshot",
      (ids: Uint32Array, output?: Float64Array) => {
        const values = [1, 0, 0, 0, 0, 1_024, 768, ids.length / 2];
        for (let index = 0; index < ids.length / 2; index++) {
          values.push(
            ids[index * 2],
            ids[index * 2 + 1],
            0,
            0,
            0,
            0,
            0,
            0,
            1_024,
            768,
            0,
            0,
            0,
            0,
          );
        }
        if (output && output.length >= values.length) output.set(values);
        return values.length;
      },
    ),
  ];

  const nodes = new Map<string, AuthoredNode>();
  const roots: AuthoredNode[] = [];
  const originals = {
    createElement: writer.createElement,
    createText: writer.createText,
    appendChild: writer.appendChild,
    insertBefore: writer.insertBefore,
    removeChild: writer.removeChild,
    setText: writer.setText,
    setAttribute: writer.setAttribute,
    removeAttribute: writer.removeAttribute,
    setClassName: writer.setClassName,
    setStyle: writer.setStyle,
    setStyleValue: writer.setStyleValue,
    removeStyle: writer.removeStyle,
    setTransform2D: writer.setTransform2D,
    setWidgetConfig: writer.setWidgetConfig,
    removeWidgetConfig: writer.removeWidgetConfig,
    setInteractionPolicy: writer.setInteractionPolicy,
    setOverlayPlane: writer.setOverlayPlane,
    setProjectionBoundary: writer.setProjectionBoundary,
    dropNode: writer.dropNode,
    focusNode: writer.focusNode,
  };

  const create = (id: NodeKey, tag: string, text = "") => {
    nodes.set(key(id), {
      id,
      tag,
      parent: null,
      children: [],
      attributes: new Map(),
      focusOrder: null,
      interactionBlocked: false,
      focusContained: false,
      projectionBoundary: false,
      overlayPlane: "content",
      className: "",
      styles: new Map(),
      widgetConfig: null,
      transform: null,
      text,
    });
  };
  const detach = (node: AuthoredNode) => {
    const siblings = node.parent?.children ?? roots;
    const index = siblings.indexOf(node);
    if (index >= 0) siblings.splice(index, 1);
    node.parent = null;
  };
  const attach = (parentId: NodeKey, childId: NodeKey, refId?: NodeKey) => {
    const child = nodes.get(key(childId));
    if (!child) return;
    detach(child);
    const parent = nodes.get(key(parentId)) ?? null;
    if (parent === child) {
      throw new Error(
        `component protocol attempted to attach node ${key(childId)} to itself`,
      );
    }
    for (let ancestor = parent; ancestor; ancestor = ancestor.parent) {
      if (ancestor === child) {
        throw new Error(
          `component protocol attempted to attach ancestor node ${key(childId)} below its descendant`,
        );
      }
    }
    const siblings = parent?.children ?? roots;
    const refIndex = refId
      ? siblings.findIndex((candidate) => key(candidate.id) === key(refId))
      : -1;
    siblings.splice(refIndex < 0 ? siblings.length : refIndex, 0, child);
    child.parent = parent;
  };

  writer.createElement = (id, tag) => {
    create(id, tag);
    originals.createElement.call(writer, id, tag);
  };
  writer.createText = (id, text) => {
    create(id, "#text", text);
    originals.createText.call(writer, id, text);
  };
  writer.appendChild = (parent, child) => {
    attach(parent, child);
    originals.appendChild.call(writer, parent, child);
  };
  writer.insertBefore = (parent, child, ref) => {
    attach(parent, child, ref);
    originals.insertBefore.call(writer, parent, child, ref);
  };
  writer.removeChild = (parent, child) => {
    const node = nodes.get(key(child));
    if (node) detach(node);
    originals.removeChild.call(writer, parent, child);
  };
  writer.setText = (id, text) => {
    const node = nodes.get(key(id));
    if (node) node.text = text;
    originals.setText.call(writer, id, text);
  };
  writer.setAttribute = (id, name, value) => {
    nodes.get(key(id))?.attributes.set(name, value);
    originals.setAttribute.call(writer, id, name, value);
  };
  writer.removeAttribute = (id, name) => {
    nodes.get(key(id))?.attributes.delete(name);
    originals.removeAttribute.call(writer, id, name);
  };
  writer.setClassName = (id, value) => {
    const node = nodes.get(key(id));
    if (node) node.className = value;
    originals.setClassName.call(writer, id, value);
  };
  writer.setStyle = (id, name, value) => {
    nodes.get(key(id))?.styles.set(name, value);
    originals.setStyle.call(writer, id, name, value);
  };
  writer.setStyleValue = (id, name, kind, value) => {
    nodes.get(key(id))?.styles.set(name, { kind, value });
    originals.setStyleValue.call(writer, id, name, kind, value);
  };
  writer.removeStyle = (id, name) => {
    nodes.get(key(id))?.styles.delete(name);
    originals.removeStyle.call(writer, id, name);
  };
  writer.setTransform2D = (id, value) => {
    const node = nodes.get(key(id));
    if (node) node.transform = [...value];
    originals.setTransform2D.call(writer, id, value);
  };
  writer.setWidgetConfig = (id, json) => {
    const node = nodes.get(key(id));
    if (node) node.widgetConfig = JSON.parse(json);
    originals.setWidgetConfig.call(writer, id, json);
  };
  writer.removeWidgetConfig = (id) => {
    const node = nodes.get(key(id));
    if (node) node.widgetConfig = null;
    originals.removeWidgetConfig.call(writer, id);
  };
  writer.setInteractionPolicy = (id, flags, focusOrder) => {
    const node = nodes.get(key(id));
    if (node) {
      node.focusOrder =
        (flags & INTERACTION_POLICY.Focusable) !== 0 ? focusOrder : null;
      node.interactionBlocked = (flags & INTERACTION_POLICY.BlockSubtree) !== 0;
      node.focusContained = (flags & INTERACTION_POLICY.ContainFocus) !== 0;
    }
    originals.setInteractionPolicy.call(writer, id, flags, focusOrder);
  };
  writer.setOverlayPlane = (id, plane) => {
    const node = nodes.get(key(id));
    if (node) {
      node.overlayPlane =
        plane === 2 ? "modal" : plane === 1 ? "floating" : "content";
    }
    originals.setOverlayPlane.call(writer, id, plane);
  };
  writer.setProjectionBoundary = (id, enabled) => {
    const node = nodes.get(key(id));
    if (node) node.projectionBoundary = enabled;
    originals.setProjectionBoundary.call(writer, id, enabled);
  };
  writer.dropNode = (id) => {
    const node = nodes.get(key(id));
    if (node) detach(node);
    nodes.delete(key(id));
    originals.dropNode.call(writer, id);
  };

  let disposeMount: (() => void) | null = null;
  let flushDepth = 0;
  let fakeFrameTime = 0;
  let restorePerformanceNow: (() => void) | undefined;
  const flushUpdates = () => {
    // Imperative host operations such as focus can be produced by an effect
    // while this drain is already running. Solid processes those writes in the
    // same continuation; recursively calling flush() only emits a warning and
    // cannot make the result more synchronous.
    if (flushDepth > 0) return;
    flushDepth += 1;
    try {
      flushSolid();
    } finally {
      flushDepth -= 1;
    }
    writer.flush();
  };
  const restore = () => {
    Object.assign(writer, originals);
    restoreHostStubs.forEach((restoreStub) => {
      restoreStub();
    });
    restorePerformanceNow?.();
    if (options.clock === "fake") vi.useRealTimers();
    activeHarness = false;
  };
  try {
    if (options.clock === "fake") {
      vi.useFakeTimers();
      fakeFrameTime = performance.now();
      const performanceNow = vi
        .spyOn(performance, "now")
        .mockImplementation(() => fakeFrameTime);
      restorePerformanceNow = () => performanceNow.mockRestore();
    }
    let content = render;
    const host = options.host;
    if (host) {
      const child = content;
      content = () =>
        createComponent(HostProvider, {
          value: host,
          get children() {
            return child();
          },
        });
    }
    const platform = options.platform;
    if (platform) {
      const child = content;
      content = () =>
        createComponent(PlatformProvider, {
          value: platform,
          get children() {
            return child();
          },
        });
    }
    disposeMount = mount(content);
    flushUpdates();
  } catch (error) {
    restore();
    throw error;
  }

  const textOf = (node: AuthoredNode): string => {
    const text: string[] = [];
    const pending = [node];
    const visited = new Set<AuthoredNode>();
    while (pending.length > 0) {
      const current = pending.pop();
      if (!current) continue;
      if (visited.has(current)) {
        throw new Error(
          `component protocol tree contains a cycle at node ${key(current.id)}`,
        );
      }
      visited.add(current);
      if (current.tag === "#text") {
        text.push(current.text);
        continue;
      }
      for (let index = current.children.length - 1; index >= 0; index -= 1) {
        const child = current.children[index];
        if (child) pending.push(child);
      }
    }
    return text.join("");
  };
  const roleOf = (node: AuthoredNode): string | null =>
    node.attributes.get("role") ?? implicitRole(node.tag);
  const nameOf = (node: AuthoredNode): string =>
    node.attributes.get("aria-label") ?? textOf(node).trim();
  const booleanState = (node: AuthoredNode, name: string): boolean | null => {
    const value = node.attributes.get(name);
    if (value === undefined) return null;
    if (value === "true") return true;
    if (value === "false") return false;
    throw new Error(
      `${name} must be true or false, received ${JSON.stringify(value)}`,
    );
  };
  const toggleState = (
    node: AuthoredNode,
    name: string,
  ): boolean | "mixed" | null => {
    const value = node.attributes.get(name);
    if (value === "mixed") return "mixed";
    return booleanState(node, name);
  };
  const disabledState = (node: AuthoredNode): boolean =>
    node.attributes.has("disabled") ||
    node.attributes.get("aria-disabled") === "true";
  const readOnlyState = (node: AuthoredNode): boolean =>
    node.attributes.has("readOnly") ||
    node.attributes.get("aria-readonly") === "true";
  const numericState = (node: AuthoredNode, name: string): number | null => {
    const value = node.attributes.get(name);
    if (value === undefined) return null;
    const number = Number(value);
    if (!Number.isFinite(number)) {
      throw new Error(
        `${name} must be a finite number, received ${JSON.stringify(value)}`,
      );
    }
    return number;
  };
  const currentState = (node: AuthoredNode): boolean | string | null => {
    const value = node.attributes.get("aria-current");
    if (value === undefined) return null;
    if (value === "true") return true;
    if (value === "false") return false;
    return value;
  };
  const orientationState = (
    node: AuthoredNode,
  ): "horizontal" | "vertical" | null => {
    const value = node.attributes.get("aria-orientation");
    if (value === undefined) return null;
    if (value === "horizontal" || value === "vertical") return value;
    throw new Error(
      `aria-orientation must be horizontal or vertical, received ${JSON.stringify(value)}`,
    );
  };
  const all = (): AuthoredNode[] => {
    const result: AuthoredNode[] = [];
    const pending = [...roots].reverse();
    const visited = new Set<AuthoredNode>();
    while (pending.length > 0) {
      const node = pending.pop();
      if (!node) continue;
      if (visited.has(node)) {
        throw new Error(
          `component protocol tree contains a cycle at node ${key(node.id)}`,
        );
      }
      visited.add(node);
      result.push(node);
      for (let index = node.children.length - 1; index >= 0; index -= 1) {
        const child = node.children[index];
        if (child) pending.push(child);
      }
    }
    return result;
  };
  const descendantsOf = (root: AuthoredNode): AuthoredNode[] => {
    const result: AuthoredNode[] = [];
    const pending = [...root.children].reverse();
    const visited = new Set<AuthoredNode>([root]);
    while (pending.length > 0) {
      const node = pending.pop();
      if (!node) continue;
      if (visited.has(node)) {
        throw new Error(
          `component protocol tree contains a cycle at node ${key(node.id)}`,
        );
      }
      visited.add(node);
      result.push(node);
      for (let index = node.children.length - 1; index >= 0; index -= 1) {
        const child = node.children[index];
        if (child) pending.push(child);
      }
    }
    return result;
  };
  const scopeNodes = (root: AuthoredNode | null): AuthoredNode[] => {
    if (root === null) return all();
    if (!all().includes(root)) {
      throw new Error(
        `cannot query within detached component ${roleOf(root) ?? root.tag} "${nameOf(root)}"`,
      );
    }
    return descendantsOf(root);
  };
  const describeRole = (
    role: string,
    options: ComponentRoleQueryOptions | ComponentRoleListOptions,
  ) => {
    const filters = Object.entries(options)
      .filter(([name, value]) => name !== "index" && value !== undefined)
      .map(([name, value]) => ` ${name}=${JSON.stringify(value)}`)
      .join("");
    return `role=${role}${filters}`;
  };
  const matchesState = (
    node: AuthoredNode,
    options: ComponentRoleQueryOptions | ComponentRoleListOptions,
  ) =>
    (options.disabled === undefined ||
      disabledState(node) === options.disabled) &&
    (options.readOnly === undefined ||
      readOnlyState(node) === options.readOnly) &&
    (options.checked === undefined ||
      toggleState(node, "aria-checked") === options.checked) &&
    (options.selected === undefined ||
      booleanState(node, "aria-selected") === options.selected) &&
    (options.expanded === undefined ||
      booleanState(node, "aria-expanded") === options.expanded) &&
    (options.pressed === undefined ||
      toggleState(node, "aria-pressed") === options.pressed) &&
    (options.busy === undefined ||
      booleanState(node, "aria-busy") === options.busy) &&
    (options.current === undefined || currentState(node) === options.current) &&
    (options.orientation === undefined ||
      orientationState(node) === options.orientation) &&
    (options.focused === undefined ||
      (focusedNode === node) === options.focused);
  const matchingRole = (
    root: AuthoredNode | null,
    role: string,
    options: ComponentRoleQueryOptions | ComponentRoleListOptions,
  ) => scopeNodes(root).filter((node) => matchesRole(node, role, options));
  const matchesRole = (
    node: AuthoredNode,
    role: string,
    options: ComponentRoleListOptions,
  ) =>
    roleOf(node) === role &&
    (options.name === undefined || nameOf(node) === options.name) &&
    matchesState(node, options);
  const scopeSuffix = (root: AuthoredNode | null) =>
    root === null
      ? ""
      : ` within ${roleOf(root) ?? root.tag} "${nameOf(root)}"`;
  const resolveOne = (
    root: AuthoredNode | null,
    role: string,
    options: ComponentRoleQueryOptions,
    required: boolean,
  ): ComponentLocator | null => {
    if (
      options.index !== undefined &&
      (!Number.isSafeInteger(options.index) || options.index < 0)
    ) {
      throw new RangeError("component locator index must be non-negative");
    }
    const matches = matchingRole(root, role, options);
    const description = `${describeRole(role, options)}${scopeSuffix(root)}`;
    if (options.index === undefined && matches.length > 1) {
      throw new Error(
        `found ${matches.length} matches for ${description}; pass an index or use getAllByRole`,
      );
    }
    const match = matches[options.index ?? 0];
    if (!match) {
      if (required) throw new Error(`no component found for ${description}`);
      return null;
    }
    return locator(match);
  };
  const resolveAll = (
    root: AuthoredNode | null,
    role: string,
    options: ComponentRoleListOptions,
    required: boolean,
  ): readonly ComponentLocator[] => {
    const matches = matchingRole(root, role, options);
    if (required && matches.length === 0) {
      throw new Error(
        `no components found for ${describeRole(role, options)}${scopeSuffix(root)}`,
      );
    }
    return matches.map(locator);
  };
  const queries = (root: AuthoredNode | null): ComponentQueries => ({
    getByRole: (role, options = {}) => {
      const result = resolveOne(root, role, options, true);
      if (!result)
        throw new Error("required component query returned no result");
      return result;
    },
    queryByRole: (role, options = {}) => resolveOne(root, role, options, false),
    getAllByRole: (role, options = {}) => resolveAll(root, role, options, true),
    queryAllByRole: (role, options = {}) =>
      resolveAll(root, role, options, false),
  });
  const commitEvent = (node: AuthoredNode, eventCode: number, payload = "") => {
    dispatchEvent(node.id, eventCode, payload);
    flushUpdates();
  };
  let focusedNode: AuthoredNode | null = null;
  const blurFocusedNode = (flush = true) => {
    if (!focusedNode) return;
    const previous = focusedNode;
    focusedNode = null;
    dispatchEvent(previous.id, EVENT_CODE.blur, "");
    dispatchEvent(previous.id, EVENT_CODE.focusout, "");
    if (flush) flushUpdates();
  };
  const focusAuthoredNode = (node: AuthoredNode, flush = true) => {
    if (focusedNode === node) return;
    blurFocusedNode(false);
    focusedNode = node;
    dispatchEvent(node.id, EVENT_CODE.focus, "");
    dispatchEvent(node.id, EVENT_CODE.focusin, "");
    if (flush) flushUpdates();
  };
  // Roving-focus components call Handle.focus(), which writes the same native
  // focus operation used by the real host. Reflect it back into component
  // events so unit tests exercise that imperative path instead of faking it.
  writer.focusNode = (id) => {
    originals.focusNode.call(writer, id);
    const node = nodes.get(key(id));
    // Handle.focus() can be called from an overlay/effect while Solid is
    // already draining. Those writes belong to that continuation; recursively
    // flushing here only warns and cannot make them more synchronous.
    if (node) focusAuthoredNode(node, false);
  };
  const ensureAttached = (node: AuthoredNode, action: string) => {
    if (all().includes(node)) return;
    throw new Error(
      `cannot ${action} detached component ${roleOf(node) ?? node.tag} "${nameOf(node)}"`,
    );
  };
  const ensureEnabled = (node: AuthoredNode, action: string) => {
    ensureAttached(node, action);
    if (disabledState(node)) {
      throw new Error(
        `cannot ${action} disabled component ${roleOf(node) ?? node.tag} "${nameOf(node)}"`,
      );
    }
  };
  const ensureEditable = (node: AuthoredNode) => {
    ensureEnabled(node, "input into");
    if (readOnlyState(node)) {
      throw new Error(
        `cannot input into read-only component ${roleOf(node) ?? node.tag} "${nameOf(node)}"`,
      );
    }
  };
  const pointerPayload = (
    position: ComponentPointerPosition,
    buttons: number,
    button = 0,
  ): string => {
    const clientX = position.clientX ?? position.offsetX ?? 0;
    const clientY = position.clientY ?? position.offsetY ?? 0;
    const offsetX = position.offsetX ?? clientX;
    const offsetY = position.offsetY ?? clientY;
    if (
      !Number.isFinite(clientX) ||
      !Number.isFinite(clientY) ||
      !Number.isFinite(offsetX) ||
      !Number.isFinite(offsetY)
    ) {
      throw new RangeError("component pointer coordinates must be finite");
    }
    return JSON.stringify({
      clientX,
      clientY,
      offsetX,
      offsetY,
      button,
      buttons,
      mods: 0,
    });
  };
  function locator(node: AuthoredNode): ComponentLocator {
    const result: Omit<ComponentLocator, "parent" | "children"> = {
      ...queries(node),
      get identity() {
        return { lo: node.id.lo, hi: node.id.hi };
      },
      get tag() {
        return node.tag;
      },
      get role() {
        return roleOf(node) ?? "";
      },
      get name() {
        return nameOf(node);
      },
      get text() {
        return textOf(node);
      },
      get className() {
        return node.className;
      },
      style: (name) => node.styles.get(name) ?? null,
      get widgetConfig() {
        return node.widgetConfig;
      },
      snapshot: () => snapshotNode(node),
      closestByRole: (role, options = {}) => {
        ensureAttached(node, "query ancestors of");
        let current: AuthoredNode | null = node;
        while (current) {
          if (matchesRole(current, role, options)) return locator(current);
          current = current.parent;
        }
        return null;
      },
      get disabled() {
        return disabledState(node);
      },
      get readOnly() {
        return readOnlyState(node);
      },
      get checked() {
        return toggleState(node, "aria-checked");
      },
      get selected() {
        return booleanState(node, "aria-selected");
      },
      get expanded() {
        return booleanState(node, "aria-expanded");
      },
      get pressed() {
        return toggleState(node, "aria-pressed");
      },
      get busy() {
        return booleanState(node, "aria-busy") === true;
      },
      get current() {
        return currentState(node);
      },
      get orientation() {
        return orientationState(node);
      },
      get value() {
        return node.attributes.get("value") ?? null;
      },
      get numericValue() {
        return numericState(node, "aria-valuenow");
      },
      get minNumericValue() {
        return numericState(node, "aria-valuemin");
      },
      get maxNumericValue() {
        return numericState(node, "aria-valuemax");
      },
      get valueText() {
        return node.attributes.get("aria-valuetext") ?? null;
      },
      get transform() {
        return node.transform;
      },
      get focused() {
        return focusedNode === node;
      },
      get focusOrder() {
        return node.focusOrder;
      },
      get interactionBlocked() {
        return node.interactionBlocked;
      },
      get focusContained() {
        return node.focusContained;
      },
      get overlayPlane() {
        return node.overlayPlane;
      },
      get projectionBoundary() {
        return node.projectionBoundary;
      },
      attribute: (name) => node.attributes.get(name) ?? null,
      pointerDown: (position = {}) => {
        ensureEnabled(node, "press");
        commitEvent(node, EVENT_CODE.pointerdown, pointerPayload(position, 1));
      },
      movePointer: (position = {}) => {
        ensureEnabled(node, "move pointer over");
        commitEvent(node, EVENT_CODE.pointermove, pointerPayload(position, 0));
      },
      pointerMove: (position = {}) => {
        ensureEnabled(node, "drag");
        commitEvent(node, EVENT_CODE.pointermove, pointerPayload(position, 1));
      },
      pointerUp: (position = {}) => {
        ensureEnabled(node, "release");
        commitEvent(node, EVENT_CODE.pointerup, pointerPayload(position, 0));
      },
      click: () => {
        ensureEnabled(node, "click");
        commitEvent(node, EVENT_CODE.pointerdown, pointerPayload({}, 1));
        commitEvent(node, EVENT_CODE.pointerup, pointerPayload({}, 0));
        commitEvent(node, EVENT_CODE.click);
      },
      contextMenu: (position = {}) => {
        ensureEnabled(node, "open context menu for");
        commitEvent(
          node,
          EVENT_CODE.contextmenu,
          pointerPayload(position, 0, 2),
        );
      },
      press: (pressedKey, options = {}) => {
        ensureEnabled(node, "press");
        if (pressedKey.length === 0) throw new Error("key must not be empty");
        const payload = JSON.stringify({
          key: pressedKey,
          code: options.code ?? "",
          mods: options.mods ?? 0,
          primary: options.primary ?? false,
          repeat: options.repeat ?? false,
        });
        commitEvent(node, EVENT_CODE.keydown, payload);
        commitEvent(node, EVENT_CODE.keyup, payload);
      },
      input: (value) => {
        ensureEditable(node);
        focusAuthoredNode(node);
        commitEvent(node, EVENT_CODE.input, JSON.stringify({ value }));
      },
      emit: (type, payload = "") => {
        ensureAttached(node, `dispatch ${type} to`);
        const encoded =
          typeof payload === "string" ? payload : JSON.stringify(payload);
        commitEvent(node, EVENT_CODE[type], encoded);
      },
      finishNativeTransition: () => {
        ensureAttached(node, "finish the native transition of");
        const encoded = node.attributes.get("__wabou_native_transition");
        if (!encoded) {
          throw new Error(
            `component ${roleOf(node) ?? node.tag} "${nameOf(node)}" has no native transition`,
          );
        }
        const transition = JSON.parse(encoded) as { generation?: unknown };
        if (!Number.isSafeInteger(transition.generation)) {
          throw new Error(
            `component native transition has an invalid generation: ${encoded}`,
          );
        }
        commitEvent(
          node,
          EVENT_CODE.transitionend,
          JSON.stringify({ generation: transition.generation }),
        );
      },
      focus: () => {
        ensureEnabled(node, "focus");
        focusAuthoredNode(node);
      },
      blur: () => {
        ensureAttached(node, "blur");
        if (focusedNode === node) blurFocusedNode();
      },
      hover: () => {
        ensureEnabled(node, "hover");
        commitEvent(node, EVENT_CODE.pointerenter);
      },
      unhover: () => {
        ensureAttached(node, "unhover");
        commitEvent(node, EVENT_CODE.pointerleave);
      },
      resize: ({ width, height }) => {
        ensureAttached(node, "resize");
        if (
          !Number.isFinite(width) ||
          width < 0 ||
          !Number.isFinite(height) ||
          height < 0
        ) {
          throw new RangeError(
            "component size must be finite and non-negative",
          );
        }
        dispatchResizeObservation(node.id, width, height);
        flushUpdates();
      },
    };
    // Parent and children intentionally stay non-enumerable. Assertion
    // formatters enumerate received objects and would otherwise recurse over
    // this bidirectional graph instead of showing the failed expectation.
    Object.defineProperties(result, {
      parent: {
        enumerable: false,
        get: () => (node.parent ? locator(node.parent) : null),
      },
      children: {
        enumerable: false,
        get: () => node.children.map(locator),
      },
    });
    return result as ComponentLocator;
  }

  let disposed = false;
  const snapshotNode = (node: AuthoredNode): ComponentSnapshotNode => {
    const attributes = Object.fromEntries(
      [...node.attributes.entries()].sort(([left], [right]) =>
        left.localeCompare(right),
      ),
    );
    const styles = Object.fromEntries(
      [...node.styles.entries()].sort(([left], [right]) =>
        left.localeCompare(right),
      ),
    );
    const role = roleOf(node);
    const name = nameOf(node);
    const text = textOf(node);
    return {
      tag: node.tag,
      ...(role ? { role } : {}),
      ...(name ? { name } : {}),
      ...(text ? { text } : {}),
      ...(node.className ? { className: node.className } : {}),
      ...(Object.keys(attributes).length > 0 ? { attributes } : {}),
      ...(Object.keys(styles).length > 0 ? { styles } : {}),
      ...(node.focusOrder !== null ? { focusOrder: node.focusOrder } : {}),
      ...(node.interactionBlocked ? { interactionBlocked: true as const } : {}),
      ...(node.focusContained ? { focusContained: true as const } : {}),
      ...(node.projectionBoundary ? { projectionBoundary: true as const } : {}),
      ...(node.overlayPlane !== "content"
        ? { overlayPlane: node.overlayPlane }
        : {}),
      ...(node.transform ? { transform: node.transform } : {}),
      ...(node.children.length > 0
        ? { children: node.children.map(snapshotNode) }
        : {}),
    };
  };
  const screen: ComponentScreen = {
    ...queries(null),
    get roots() {
      return roots.map(locator);
    },
    snapshot() {
      return roots.map(snapshotNode);
    },
    flush() {
      flushUpdates();
    },
    async advanceTime(milliseconds) {
      if (options.clock !== "fake") {
        throw new Error(
          'advanceTime requires renderComponent(..., { clock: "fake" })',
        );
      }
      if (!Number.isFinite(milliseconds) || milliseconds < 0) {
        throw new RangeError(
          "component clock duration must be finite and non-negative",
        );
      }
      const tick = (globalThis as Record<string, unknown>).__wabou_tick;
      // Integer milliseconds preserve exact setTimeout boundaries in fake
      // timers while approximating the native 60 Hz frame cadence.
      const frameInterval = 16;
      let remaining = milliseconds;
      if (remaining === 0) {
        vi.advanceTimersByTime(0);
        if (typeof tick === "function") tick(fakeFrameTime);
        flushUpdates();
        await Promise.resolve();
        return;
      }
      while (remaining > 0) {
        const elapsed = Math.min(frameInterval, remaining);
        vi.advanceTimersByTime(elapsed);
        fakeFrameTime += elapsed;
        if (typeof tick === "function") tick(fakeFrameTime);
        flushUpdates();
        // motion-dom clears its synchronous frame-time cache in a microtask.
        // Yielding here makes a rapid retarget start from this frame rather
        // than an earlier animation's timestamp.
        await Promise.resolve();
        remaining -= elapsed;
      }
    },
    async waitFor(assertion, waitOptions = {}) {
      const timeout = waitOptions.timeout ?? 1_000;
      const interval = waitOptions.interval ?? 10;
      if (!Number.isFinite(timeout) || timeout < 0) {
        throw new RangeError(
          "component wait timeout must be finite and non-negative",
        );
      }
      if (!Number.isFinite(interval) || interval <= 0) {
        throw new RangeError(
          "component wait interval must be finite and positive",
        );
      }
      let lastError: unknown;
      for (let elapsed = 0; elapsed <= timeout; elapsed += interval) {
        // Async event handlers and mocked capabilities normally resume in the
        // next microtask. Commit their Solid and protocol work before probing.
        await Promise.resolve();
        flushUpdates();
        try {
          return await assertion();
        } catch (error) {
          lastError = error;
        }
        if (elapsed + interval <= timeout) {
          if (options.clock === "fake") {
            // Keep timer advancement explicit through advanceTime(). A
            // microtask yield still lets Promise-backed host fixtures settle.
            await Promise.resolve();
          } else {
            await new Promise<void>((resolve) => setTimeout(resolve, interval));
          }
        }
      }
      const detail = lastError instanceof Error ? `: ${lastError.message}` : "";
      throw new Error(`component wait timed out after ${timeout}ms${detail}`, {
        cause: lastError,
      });
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      focusedNode = null;
      try {
        disposeMount?.();
      } finally {
        restore();
        if (activeScreen === screen) activeScreen = null;
      }
    },
  };
  activeScreen = screen;
  onTestFinished(() => screen.dispose());
  return screen;
}
