// @wabou/solid-renderer
//
// Binds real `solid-js` to the binary bridge: implements
// `solid-js/universal`'s `RendererOptions<NodeType>` over an in-memory handle
// tree, where every mutation emits a protocol op into a shared `Writer`.
// Solid owns the reactive core + reconciler (createSignal/<For>/JSX); we only
// supply the node hooks.
//
// Solid's universal JSX compiles to
//   `import { createElement, insertNode, insert, setProp, createComponent }
//    from "<moduleName>"`
// where moduleName is this package. So we eagerly build ONE renderer at module
// load and re-export its methods as named exports below.

import { EVENT_CODE, type EventType, OP, Writer } from "@wabou/protocol";
import { type Affine2D, isTypedStyleValue, type Shadow } from "@wabou/style";
import { createMemo, splitProps, untrack } from "solid-js";
export const isServer = false;
export const getRequestEvent = () => undefined;
export const delegateEvents = () => {};

import type { JSX } from "solid-js";
import { createRenderer as solidCreateRenderer } from "solid-js/universal";

// Host-provided global (injected by Rust) for logging from the renderer.
declare function __wabou_log(level: "error", message: string): void;

/** A pure-JS handle standing in for a DOM node. id == protocol node id. */
export interface Handle {
  id: number;
  tag: string;
  parent: Handle | null;
  firstChild: Handle | null;
  lastChild: Handle | null;
  prev: Handle | null;
  next: Handle | null;
  href?: string;
  /** Request native keyboard focus for this node on the next bridge flush. */
  focus(): void;
  /** Set this overflow container's native scroll offset. */
  scrollTo(options: { left?: number; top?: number }): void;
  scrollTo(left: number, top: number): void;
  /** Adjust this overflow container's native scroll offset. */
  scrollBy(options: { left?: number; top?: number }): void;
  scrollBy(left: number, top: number): void;
}

const FREE_LIST: number[] = [];
const GENERATIONS: number[] = [];
let nextSlot = 2; // 1 is reserved for the host-supplied root mount

const listenersBySlot: (Map<number, (e: unknown) => void> | undefined)[] = [];
/** solid id -> WeakRef<Handle>, so event dispatch can walk the parent chain for bubbling without leaking memory. */
const nodesBySlot: (WeakRef<Handle> | undefined)[] = [];
const classesByNode = new WeakMap<
  Handle,
  { base: string; toggles: Record<string, boolean> }
>();

function emitClasses(writer: Writer, node: Handle): void {
  const state = classesByNode.get(node);
  if (!state) return;
  const tokens = new Set(state.base.split(/\s+/).filter(Boolean));
  for (const [names, enabled] of Object.entries(state.toggles)) {
    for (const token of names.split(/\s+/).filter(Boolean)) {
      if (enabled) tokens.add(token);
      else tokens.delete(token);
    }
  }
  writer.setClassName(node.id, [...tokens].join(" "));
}

const finalizationRegistry =
  typeof FinalizationRegistry !== "undefined"
    ? new FinalizationRegistry<number>((id) => {
        const slot = id & 0xfffff;
        const expectedGen = id >>> 20;
        if (GENERATIONS[slot] !== expectedGen) return;
        nodesBySlot[slot] = undefined;
        listenersBySlot[slot] = undefined;
        writer.dropNode(id);
        freeId(id);
      })
    : null;

const sweepSet = new Set<Handle>();

export function runSweep(): void {
  if (sweepSet.size === 0) return;
  for (const node of sweepSet) {
    if (node.parent !== null) continue; // re-attached

    // recursively destroy
    const destroy = (n: Handle) => {
      const slot = n.id & 0xfffff;
      if (nodesBySlot[slot] === undefined) return;

      finalizationRegistry?.unregister(n);
      nodesBySlot[slot] = undefined;
      listenersBySlot[slot] = undefined;
      writer.dropNode(n.id);
      freeId(n.id);
      let c = n.firstChild;
      while (c) {
        destroy(c);
        c = c.next;
      }
    };
    destroy(node);
  }
  sweepSet.clear();
}

function newId(): number {
  let slot: number;
  if (FREE_LIST.length > 0) {
    slot = FREE_LIST.pop()!;
  } else {
    slot = nextSlot++;
    GENERATIONS[slot] = 0;
  }
  const gen = GENERATIONS[slot];
  return ((gen << 20) | slot) >>> 0;
}

function freeId(id: number) {
  const slot = id & 0xfffff;
  GENERATIONS[slot] = (GENERATIONS[slot] + 1) & 0xfff;
  FREE_LIST.push(slot);
}

function imperativeMethods(
  id: number,
): Pick<Handle, "focus" | "scrollTo" | "scrollBy"> {
  const coordinates = (
    first: number | { left?: number; top?: number },
    second?: number,
  ): [number, number] =>
    typeof first === "number"
      ? [first, second ?? Number.NaN]
      : [first.left ?? Number.NaN, first.top ?? Number.NaN];
  const scrollTo = ((
    first: number | { left?: number; top?: number },
    second?: number,
  ) => {
    const [x, y] = coordinates(first, second);
    writer.scrollTo(id, x, y);
  }) as Handle["scrollTo"];
  const scrollBy = ((
    first: number | { left?: number; top?: number },
    second?: number,
  ) => {
    const [x, y] = coordinates(first, second);
    writer.scrollBy(id, x, y);
  }) as Handle["scrollBy"];
  return {
    focus: () => writer.focusNode(id),
    scrollTo,
    scrollBy,
  };
}

function makeHandle(tag: string): Handle {
  const id = newId();
  const h: Handle = {
    id,
    tag,
    parent: null,
    firstChild: null,
    lastChild: null,
    prev: null,
    next: null,
    ...imperativeMethods(id),
  };
  if (typeof WeakRef !== "undefined") {
    nodesBySlot[id & 0xfffff] = new WeakRef(h);
  }
  if (finalizationRegistry) {
    finalizationRegistry.register(h, h.id, h);
  }
  return h;
}

function linkChild(parent: Handle, child: Handle, ref: Handle | null): void {
  child.parent = parent;
  if (ref == null) {
    child.prev = parent.lastChild;
    child.next = null;
    if (parent.lastChild) parent.lastChild.next = child;
    else parent.firstChild = child;
    parent.lastChild = child;
  } else {
    child.prev = ref.prev;
    child.next = ref;
    if (ref.prev) ref.prev.next = child;
    else parent.firstChild = child;
    ref.prev = child;
  }
}

function unlinkChild(parent: Handle, child: Handle): void {
  if (child.prev) child.prev.next = child.next;
  else parent.firstChild = child.next;
  if (child.next) child.next.prev = child.prev;
  else parent.lastChild = child.prev;
  child.parent = child.prev = child.next = null;
}

/** Translate a setProperty call into protocol ops. Shared by both hooks. */
function applyProperty(
  writer: Writer,
  node: Handle,
  name: string,
  value: unknown,
  prev: unknown,
): void {
  if (value === prev) return;
  if (name === "transform") {
    const matrix =
      value == null || value === false ? [1, 0, 0, 1, 0, 0] : value;
    if (
      Array.isArray(matrix) &&
      matrix.length === 6 &&
      matrix.every((part) => typeof part === "number" && Number.isFinite(part))
    ) {
      writer.setTransform2D(node.id, matrix as unknown as Affine2D);
    }
    return;
  }
  if (name === "class" || name === "className") {
    const state = classesByNode.get(node) ?? { base: "", toggles: {} };
    state.base = value == null || value === false ? "" : String(value);
    classesByNode.set(node, state);
    emitClasses(writer, node);
    return;
  }
  if (name === "classList") {
    const state = classesByNode.get(node) ?? { base: "", toggles: {} };
    state.toggles = {};
    if (value && typeof value === "object") {
      for (const [token, enabled] of Object.entries(value)) {
        state.toggles[token] = Boolean(enabled);
      }
    }
    classesByNode.set(node, state);
    emitClasses(writer, node);
    return;
  }
  if (name === "shadows") {
    if (value == null || value === false) {
      writer.removeStyle(node.id, "box-shadow");
    } else if (Array.isArray(value)) {
      writer.setShadows(node.id, value as readonly Shadow[]);
    }
    return;
  }
  if (value == null || value === false) {
    // Event handlers are stored under listenersByNode (keyed by event code),
    // not as DOM attributes — so an on* prop going null/false must remove the
    // listener, not call removeAttribute (which would be a no-op and leak the
    // old handler). Matches the binding side in the on* branch below.
    if (name.startsWith("on") && name.length > 2) {
      const t = EVENT_CODE[name.slice(2).toLowerCase() as EventType] ?? null;
      if (t != null) {
        const slot = node.id & 0xfffff;
        writer.removeEventListener(node.id, t);
        listenersBySlot[slot]?.delete(t);
      }
      return;
    }
    if (name === "href") node.href = undefined;
    writer.removeAttribute(node.id, name);
    return;
  }
  if (name === "style" && typeof value === "object" && value !== null) {
    const rec = value as Record<string, unknown>;
    const prec = (prev && typeof prev === "object" ? prev : {}) as Record<
      string,
      unknown
    >;
    for (const k in rec) {
      const next = rec[k];
      if (k in prec && next === prec[k]) continue;
      if (next == null || next === false) {
        writer.removeStyle(node.id, k);
        continue;
      }
      if (isTypedStyleValue(next)) {
        writer.setStyleValue(node.id, k, next.kind, next.value);
        continue;
      }
      writer.setStyle(node.id, k, String(next));
    }
    for (const k in prec) if (!(k in rec)) writer.removeStyle(node.id, k);
    return;
  }
  if (name === "textContent") {
    writer.setText(node.id, String(value));
    return;
  }
  if (name === "href") node.href = String(value);
  if (name.startsWith("on") && typeof value === "function") {
    const t = EVENT_CODE[name.slice(2).toLowerCase() as EventType];
    if (t == null) return;
    writer.addEventListener(node.id, t);
    const slot = node.id & 0xfffff;
    let m = listenersBySlot[slot];
    if (!m) {
      m = new Map();
      listenersBySlot[slot] = m;
    }
    m.set(t, value as (e: unknown) => void);
    return;
  }
  writer.setAttribute(node.id, name, String(value));
}

// ---------------------------------------------------------------------------
// The single renderer + shared writer. Module-level so the named exports
// consumed by compiled JSX are available at load time.
// ---------------------------------------------------------------------------

const writer = new Writer();

const renderer = solidCreateRenderer<Handle>({
  createElement(tag) {
    const h = makeHandle(tag);
    writer.createElement(h.id, tag);
    return h;
  },
  createTextNode(value) {
    const h = makeHandle("#text");
    writer.createText(h.id, String(value));
    return h;
  },
  replaceText(textNode, value) {
    writer.setText(textNode.id, String(value));
  },
  isTextNode(node) {
    return node.tag === "#text";
  },
  setProperty(node, name, value, prev) {
    applyProperty(writer, node, name, value, prev);
  },
  insertNode(parent, node, anchor) {
    if (node.parent) {
      unlinkChild(node.parent, node);
    }
    if (anchor) {
      linkChild(parent, node, anchor);
      writer.insertBefore(parent.id, node.id, anchor.id);
    } else {
      linkChild(parent, node, null);
      writer.appendChild(parent.id, node.id);
    }
  },
  removeNode(parent, node) {
    unlinkChild(parent, node);
    writer.removeChild(parent.id, node.id);
    sweepSet.add(node);
  },
  getParentNode(node) {
    return node.parent ?? undefined;
  },
  getFirstChild(node) {
    return node.firstChild ?? undefined;
  },
  getNextSibling(node) {
    return node.next ?? undefined;
  },
});

// Re-exported for the app's host glue (render entry + writer flush).
export { writer };
/** Imperative paint-only transform state for high-frequency animation. */
export function setTransform2D(node: Handle, matrix: Affine2D): void {
  if (!matrix.every(Number.isFinite)) return;
  writer.setTransform2D(node.id, matrix);
}
export const render = renderer.render as any as (
  code: () => JSX.Element,
  node: Handle,
) => () => void;

// Re-exported because compiled universal JSX imports these from this module.
export const createElement = renderer.createElement;
export const createTextNode = renderer.createTextNode;
export const insertNode = renderer.insertNode;
export function removeNode(parent: Handle, node: Handle): void {
  unlinkChild(parent, node);
  writer.removeChild(parent.id, node.id);
  sweepSet.add(node);
}
export const insert = renderer.insert;
export const setProp = renderer.setProp;
export const createComponent = renderer.createComponent;
export const effect = renderer.effect;
export const memo = renderer.memo;
export const spread = renderer.spread;
export const mergeProps = renderer.mergeProps;
export const use = renderer.use;

export function Dynamic(props: any) {
  const [local, others] = splitProps(props, ["component"]);
  const cached = createMemo(() => local.component);

  return createMemo(() => {
    const component = cached();
    switch (typeof component) {
      case "function":
        return untrack(() => component(others));
      case "string": {
        const el = createElement(component);
        spread(el, others, false);
        return el;
      }
    }
    return null;
  });
}

/** Register the root mount handle so bubbling reaches window-level listeners. */
export function registerRoot(root: Handle): void {
  if (typeof WeakRef !== "undefined") {
    nodesBySlot[root.id & 0xfffff] = new WeakRef(root);
  }
}

/** Dispose callback for the last `mount()` — used by in-process HMR full reload. */
let activeMountDispose: (() => void) | null = null;
let mountedRoot: Handle | null = null;

/** Current native window root, used by renderer-level facilities like Portal. */
export function getMountRoot(): Handle {
  if (!mountedRoot) throw new Error("Portal must be rendered inside mount()");
  return mountedRoot;
}

/** Mount a Solid application into the host-provided root node. */
export function mount(code: () => JSX.Element): () => void {
  // Full-reload re-imports the entry and calls mount again; tear down the
  // previous tree first so DropNode ops free host boxes and slot ids.
  if (activeMountDispose) {
    try {
      activeMountDispose();
    } catch (error) {
      __wabou_log("error", `mount dispose before remount failed: ${error}`);
    }
    activeMountDispose = null;
  }
  const root: Handle = {
    id: 1,
    tag: "#root",
    parent: null,
    firstChild: null,
    lastChild: null,
    prev: null,
    next: null,
    ...imperativeMethods(1),
  };
  mountedRoot = root;
  registerRoot(root);
  const dispose = render(code, root);
  activeMountDispose = () => {
    dispose();
    if (mountedRoot === root) mountedRoot = null;
    runSweep();
    writer.flush();
  };
  return () => {
    if (activeMountDispose) {
      activeMountDispose();
      activeMountDispose = null;
    }
  };
}

/**
 * Solid compatibility adapter for a native Wabou event. It walks the Handle
 * tree for bubbling and presents JSX handlers with a small familiar object;
 * this is deliberately not a complete DOM Event implementation.
 */
export function dispatchEvent(
  solidId: number,
  eventCode: number,
  payloadStr: string,
  numericData?: ArrayLike<number>,
): boolean {
  let data: Record<string, unknown> = {};
  if (payloadStr) {
    try {
      data = JSON.parse(payloadStr);
    } catch {
      /* ignore malformed */
    }
  } else {
    const ed = numericData;
    if (ed) {
      if (
        eventCode === EVENT_CODE.pointerup ||
        eventCode === EVENT_CODE.pointerdown ||
        eventCode === EVENT_CODE.pointermove ||
        eventCode === EVENT_CODE.click
      ) {
        data.clientX = ed[0];
        data.clientY = ed[1];
        data.button = ed[2];
        data.buttons = ed[3];
        data.mods = ed[4];
      } else if (eventCode === EVENT_CODE.wheel) {
        data.clientX = ed[0];
        data.clientY = ed[1];
        data.deltaX = ed[5];
        data.deltaY = ed[6];
      }
    }
  }

  let stopped = false;
  let defaultPrevented = false;
  const ev = {
    target: { id: solidId, ...data },
    currentTarget: { id: solidId, ...data },
    type: eventName(eventCode),
    ...data,
    stopPropagation() {
      stopped = true;
    },
    preventDefault() {
      defaultPrevented = true;
    },
    get defaultPrevented() {
      return defaultPrevented;
    },
    get propagationStopped() {
      return stopped;
    },
  };

  bubble(solidId, eventCode, ev);
  return defaultPrevented;
}

function derefHandle(id: number): Handle | undefined {
  const stored = nodesBySlot[id & 0xfffff] as any;
  return stored instanceof WeakRef ? stored.deref() : stored;
}

/** Walk parent chain from `nodeId`, firing `code` listeners until stopped. */
function bubble(nodeId: number, code: number, ev: any): void {
  let cur: number | null = nodeId;
  while (cur != null) {
    const slot: number = cur & 0xfffff;
    ev.currentTarget = cur === nodeId ? ev.target : { id: cur };
    const m = listenersBySlot[slot];
    const fn = m?.get(code);
    if (fn) {
      try {
        fn(ev);
      } catch (e) {
        __wabou_log("error", String(e));
      }
    }
    if (ev.propagationStopped) return;
    cur = derefHandle(cur)?.parent?.id ?? null;
  }
}

/** event code -> DOM event name (for ev.type). */
function eventName(code: number): string {
  for (const [name, c] of Object.entries(EVENT_CODE)) {
    if (c === code) return name;
  }
  return "unknown";
}

export {
  defaultHost,
  type FrameStats,
  type Host,
  HostProvider,
  type HostProviderProps,
  type LayoutNodeMetrics,
  type LayoutRect,
  type LayoutSnapshot,
  type LayoutTarget,
  useHost,
} from "./host";
export { Portal, type PortalProps } from "./portal";
export { useFps } from "./use-fps";
export { VirtualList, type VirtualListProps } from "./virtual-list";
export type { JSX, Writer };
export { EVENT_CODE, OP };
