import { dispatchEvent, EVENT_CODE, mount, writer } from "@wabou/core/renderer";
import type { NodeKey } from "@wabou/core/protocol";
import { dispatchResizeObservation } from "@wabou/core/testing";
import { flush, type JSX } from "solid-js";
import { onTestFinished } from "vitest";

interface AuthoredNode {
  readonly id: NodeKey;
  readonly tag: string;
  parent: AuthoredNode | null;
  readonly children: AuthoredNode[];
  readonly attributes: Map<string, string>;
  text: string;
}

export interface ComponentLocator {
  readonly tag: string;
  readonly role: string;
  readonly name: string;
  readonly text: string;
  attribute(name: string): string | null;
  click(): void;
  press(key: string): void;
  /** Publish a deterministic native content-box observation. */
  resize(size: { width: number; height: number }): void;
}

export interface ComponentScreen {
  getByRole(
    role: string,
    options?: { name?: string; index?: number },
  ): ComponentLocator;
  queryByRole(
    role: string,
    options?: { name?: string; index?: number },
  ): ComponentLocator | null;
  dispose(): void;
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
export function renderComponent(render: () => JSX.Element): ComponentScreen {
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
    dropNode: writer.dropNode,
  };

  const create = (id: NodeKey, tag: string, text = "") => {
    nodes.set(key(id), {
      id,
      tag,
      parent: null,
      children: [],
      attributes: new Map(),
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
  writer.dropNode = (id) => {
    const node = nodes.get(key(id));
    if (node) detach(node);
    nodes.delete(key(id));
    originals.dropNode.call(writer, id);
  };

  let disposeMount: (() => void) | null = null;
  const restore = () => {
    Object.assign(writer, originals);
    restoreHostStubs.forEach((restoreStub) => restoreStub());
    activeHarness = false;
  };
  try {
    disposeMount = mount(render);
    flush();
    writer.flush();
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
  const all = (): AuthoredNode[] => {
    const result: AuthoredNode[] = [];
    const visit = (node: AuthoredNode) => {
      result.push(node);
      node.children.forEach(visit);
    };
    roots.forEach(visit);
    return result;
  };
  const commitEvent = (node: AuthoredNode, eventCode: number, payload = "") => {
    dispatchEvent(node.id, eventCode, payload);
    flush();
    writer.flush();
  };
  const ensureEnabled = (node: AuthoredNode, action: string) => {
    if (
      node.attributes.has("disabled") ||
      node.attributes.get("aria-disabled") === "true"
    ) {
      throw new Error(
        `cannot ${action} disabled component ${roleOf(node) ?? node.tag} "${nameOf(node)}"`,
      );
    }
  };
  const locator = (node: AuthoredNode): ComponentLocator => ({
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
    attribute: (name) => node.attributes.get(name) ?? null,
    click: () => {
      ensureEnabled(node, "click");
      commitEvent(node, EVENT_CODE.click);
    },
    press: (pressedKey) => {
      ensureEnabled(node, "press");
      if (pressedKey.length === 0) throw new Error("key must not be empty");
      const payload = JSON.stringify({ key: pressedKey, repeat: false });
      commitEvent(node, EVENT_CODE.keydown, payload);
      commitEvent(node, EVENT_CODE.keyup, payload);
    },
    resize: ({ width, height }) => {
      if (
        !Number.isFinite(width) ||
        width < 0 ||
        !Number.isFinite(height) ||
        height < 0
      ) {
        throw new RangeError("component size must be finite and non-negative");
      }
      dispatchResizeObservation(node.id, width, height);
      flush();
      writer.flush();
    },
  });
  const select = (
    matches: AuthoredNode[],
    description: string,
    index?: number,
    required = true,
  ): ComponentLocator | null => {
    if (index === undefined && matches.length > 1) {
      throw new Error(
        `found ${matches.length} matches for ${description}; pass an index`,
      );
    }
    const match = matches[index ?? 0];
    if (!match && required)
      throw new Error(`no component found for ${description}`);
    return match ? locator(match) : null;
  };

  let disposed = false;
  const screen: ComponentScreen = {
    getByRole(role, options = {}) {
      return select(
        all().filter(
          (node) =>
            roleOf(node) === role &&
            (options.name === undefined || nameOf(node) === options.name),
        ),
        `role=${role}${options.name === undefined ? "" : ` name=${JSON.stringify(options.name)}`}`,
        options.index,
      )!;
    },
    queryByRole(role, options = {}) {
      return select(
        all().filter(
          (node) =>
            roleOf(node) === role &&
            (options.name === undefined || nameOf(node) === options.name),
        ),
        `role=${role}${options.name === undefined ? "" : ` name=${JSON.stringify(options.name)}`}`,
        options.index,
        false,
      );
    },
    dispose() {
      if (disposed) return;
      disposed = true;
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
