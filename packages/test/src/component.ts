import { INTERACTION_POLICY, type NodeKey } from "@wabou/core/protocol";
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
import { onTestFinished } from "vitest";

interface AuthoredNode {
  readonly id: NodeKey;
  readonly tag: string;
  parent: AuthoredNode | null;
  readonly children: AuthoredNode[];
  readonly attributes: Map<string, string>;
  focusOrder: number | null;
  interactionBlocked: boolean;
  focusContained: boolean;
  className: string;
  transform: readonly [number, number, number, number, number, number] | null;
  text: string;
}

export interface ComponentRoleQueryOptions {
  name?: string;
  /** Select one occurrence in depth-first authored order. */
  index?: number;
}

export interface ComponentRoleListOptions {
  name?: string;
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

export interface ComponentLocator extends ComponentQueries {
  readonly tag: string;
  readonly role: string;
  readonly name: string;
  readonly text: string;
  readonly className: string;
  /** Disabled state as authored through `disabled` or `aria-disabled`. */
  readonly disabled: boolean;
  /** Toggle state authored through `aria-checked`. */
  readonly checked: boolean | "mixed" | null;
  /** Selection state authored through `aria-selected`. */
  readonly selected: boolean | null;
  /** Disclosure state authored through `aria-expanded`. */
  readonly expanded: boolean | null;
  /** Toggle-button state authored through `aria-pressed`. */
  readonly pressed: boolean | "mixed" | null;
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
  attribute(name: string): string | null;
  pointerDown(position?: ComponentPointerPosition): void;
  /** Dispatch a captured native pointer move while preserving button state. */
  pointerMove(position?: ComponentPointerPosition): void;
  pointerUp(position?: ComponentPointerPosition): void;
  click(): void;
  /** Dispatch a secondary-click context-menu event at a deterministic point. */
  contextMenu(position?: ComponentPointerPosition): void;
  press(key: string): void;
  input(value: string): void;
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

export interface ComponentScreen extends ComponentQueries {
  /** Commit reactive work scheduled outside a locator action, such as a timer. */
  flush(): void;
  dispose(): void;
}

export interface RenderComponentOptions {
  /** Host fixture injected into the component subtree. */
  host?: Host;
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

function missingHostMethod(path: string): never {
  throw new Error(`test host method ${path} is not configured`);
}

/** Create a typed, deterministic Host with automatic call recording. */
export function createTestHost<C extends object = Record<string, never>>(
  capabilities?: C,
  builtins: TestBuiltinHost = {},
): TestHostFixture<Host & C> {
  const calls: TestHostCall[] = [];
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
        snapshot: () => missingHostMethod("layout.snapshot"),
        measure: () => missingHostMethod("layout.measure"),
        clippingRect: () => missingHostMethod("layout.clippingRect"),
        viewport: () => missingHostMethod("layout.viewport"),
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

function installHostStub(name: string): () => void {
  const target = globalThis as Record<string, unknown>;
  const hadOwn = Object.hasOwn(target, name);
  const previous = target[name];
  if (typeof previous !== "function") target[name] = () => {};
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
    setTransform2D: writer.setTransform2D,
    setInteractionPolicy: writer.setInteractionPolicy,
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
      className: "",
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
  writer.setTransform2D = (id, value) => {
    const node = nodes.get(key(id));
    if (node) node.transform = [...value];
    originals.setTransform2D.call(writer, id, value);
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
  writer.dropNode = (id) => {
    const node = nodes.get(key(id));
    if (node) detach(node);
    nodes.delete(key(id));
    originals.dropNode.call(writer, id);
  };

  let disposeMount: (() => void) | null = null;
  let flushDepth = 0;
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
    activeHarness = false;
  };
  try {
    disposeMount = mount(() =>
      options.host
        ? createComponent(HostProvider, {
            value: options.host,
            get children() {
              return render();
            },
          })
        : render(),
    );
    flushUpdates();
  } catch (error) {
    restore();
    throw error;
  }

  const textOf = (node: AuthoredNode): string =>
    node.tag === "#text" ? node.text : node.children.map(textOf).join("");
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
  const all = (): AuthoredNode[] => {
    const result: AuthoredNode[] = [];
    const visit = (node: AuthoredNode) => {
      result.push(node);
      node.children.forEach(visit);
    };
    roots.forEach(visit);
    return result;
  };
  const descendantsOf = (root: AuthoredNode): AuthoredNode[] => {
    const result: AuthoredNode[] = [];
    const visit = (node: AuthoredNode) => {
      node.children.forEach((child) => {
        result.push(child);
        visit(child);
      });
    };
    visit(root);
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
  ) =>
    `role=${role}${options.name === undefined ? "" : ` name=${JSON.stringify(options.name)}`}`;
  const matchingRole = (
    root: AuthoredNode | null,
    role: string,
    options: ComponentRoleQueryOptions | ComponentRoleListOptions,
  ) =>
    scopeNodes(root).filter(
      (node) =>
        roleOf(node) === role &&
        (options.name === undefined || nameOf(node) === options.name),
    );
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
  const blurFocusedNode = () => {
    if (!focusedNode) return;
    const previous = focusedNode;
    focusedNode = null;
    commitEvent(previous, EVENT_CODE.blur);
    commitEvent(previous, EVENT_CODE.focusout);
  };
  const focusAuthoredNode = (node: AuthoredNode) => {
    if (focusedNode === node) return;
    blurFocusedNode();
    focusedNode = node;
    commitEvent(node, EVENT_CODE.focus);
    commitEvent(node, EVENT_CODE.focusin);
  };
  // Roving-focus components call Handle.focus(), which writes the same native
  // focus operation used by the real host. Reflect it back into component
  // events so unit tests exercise that imperative path instead of faking it.
  writer.focusNode = (id) => {
    originals.focusNode.call(writer, id);
    const node = nodes.get(key(id));
    if (node) focusAuthoredNode(node);
  };
  const ensureAttached = (node: AuthoredNode, action: string) => {
    if (all().includes(node)) return;
    throw new Error(
      `cannot ${action} detached component ${roleOf(node) ?? node.tag} "${nameOf(node)}"`,
    );
  };
  const ensureEnabled = (node: AuthoredNode, action: string) => {
    ensureAttached(node, action);
    if (
      node.attributes.has("disabled") ||
      node.attributes.get("aria-disabled") === "true"
    ) {
      throw new Error(
        `cannot ${action} disabled component ${roleOf(node) ?? node.tag} "${nameOf(node)}"`,
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
    return {
      ...queries(node),
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
      get disabled() {
        return (
          node.attributes.has("disabled") ||
          node.attributes.get("aria-disabled") === "true"
        );
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
      attribute: (name) => node.attributes.get(name) ?? null,
      pointerDown: (position = {}) => {
        ensureEnabled(node, "press");
        commitEvent(node, EVENT_CODE.pointerdown, pointerPayload(position, 1));
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
      press: (pressedKey) => {
        ensureEnabled(node, "press");
        if (pressedKey.length === 0) throw new Error("key must not be empty");
        const payload = JSON.stringify({ key: pressedKey, repeat: false });
        commitEvent(node, EVENT_CODE.keydown, payload);
        commitEvent(node, EVENT_CODE.keyup, payload);
      },
      input: (value) => {
        ensureEnabled(node, "input");
        commitEvent(node, EVENT_CODE.input, JSON.stringify({ value }));
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
  }

  let disposed = false;
  const screen: ComponentScreen = {
    ...queries(null),
    flush() {
      flushUpdates();
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
