// Wabou Solid renderer implementation.
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

import {
  EVENT_CODE,
  type EventType,
  formatNodeKey,
  GRAPHIC_DATA,
  GRAPHIC_SOURCE,
  INTERACTION_POLICY,
  type NodeKey,
  NodeKeyAllocator,
  NodeKeyTable,
  nodeKeyEquals,
  OP,
  ROOT_NODE_KEY,
  TEXT_BEHAVIOR,
  Writer,
} from "../protocol";

export {
  GRAPHIC_SOURCE,
  INTERACTION_POLICY,
  TEXT_BEHAVIOR,
} from "../protocol";

import { createMemo, omit, untrack } from "solid-js";
import type { HostCapabilities, WabouIntrinsicElements } from "../registry";
import {
  type Affine2D,
  assertInlineStyleValue,
  isTypedStyleValue,
  type Shadow,
  type WabouStyle,
} from "../style";
import { isVectorPath, type VectorPath } from "../vector-path";

export type {
  HostCapabilities,
  WabouIntrinsicElements,
} from "../registry";
export const isServer = false;
export const getRequestEvent = () => undefined;
export const delegateEvents = () => {};

import { createRenderer as solidCreateRenderer } from "@solidjs/universal";
import type { JSX } from "./jsx";

/**
 * Deliberately small set of structural host tags understood by Wabou apps.
 * This is not the HTML element registry: unsupported Web tags must be wrapped
 * by an explicit component or registered as a custom native element.
 */
export interface WabouBuiltinIntrinsicElements {
  view: WabouElementProps;
  "vector-path": WabouVectorPathProps;
}

export type WabouNativeElements = WabouBuiltinIntrinsicElements &
  WabouIntrinsicElements;
export type WabouNativeTag = keyof WabouNativeElements & string;

export type WabouSemanticRole =
  | "alert"
  | "alertdialog"
  | "button"
  | "cell"
  | "checkbox"
  | "columnheader"
  | "combobox"
  | "dialog"
  | "grid"
  | "gridcell"
  | "group"
  | "heading"
  | "img"
  | "label"
  | "link"
  | "listbox"
  | "menu"
  | "menubar"
  | "menuitem"
  | "none"
  | "option"
  | "presentation"
  | "progressbar"
  | "radio"
  | "radiogroup"
  | "region"
  | "row"
  | "rowheader"
  | "separator"
  | "slider"
  | "spinbutton"
  | "status"
  | "switch"
  | "tab"
  | "tablist"
  | "tabpanel"
  | "table"
  | "textbox"
  | "tree"
  | "treeitem"
  | "toolbar"
  | "tooltip";

/** Roles that remain addressable after semantic-tree projection. */
export type WabouExposedSemanticRole = Exclude<
  WabouSemanticRole,
  "none" | "presentation"
>;

type EventHandler<E> = {
  // Event return values have no synchronous meaning. Runtime dispatch still
  // observes thenables so async failures retain their native event context.
  bivarianceHack(event: E): unknown;
}["bivarianceHack"];

/** Props shared by low-level native JSX elements. */
export interface WabouElementProps {
  id?: string;
  class?: string;
  classList?: Record<string, boolean | undefined>;
  style?: string | WabouStyle;
  children?: JSX.Element;
  ref?: Handle | ((node: Handle) => void);
  role?: WabouSemanticRole;
  /** Enables native focus; negative values skip sequential navigation. */
  focusOrder?: number;
  /** Removes this subtree from input, focus, and accessibility routing. */
  interactionBlocked?: boolean;
  /** Contains sequential native focus within this logical subtree. */
  focusContained?: boolean;
  /** Places this subtree in a native overlay plane above ordinary content. */
  overlayPlane?: "content" | "floating" | "modal";
  "aria-label"?: string;
  "aria-hidden"?: boolean | "true" | "false";
  "aria-modal"?: boolean | "true" | "false";
  "aria-haspopup"?: "dialog" | "grid" | "listbox" | "menu" | "tree";
  "aria-expanded"?: boolean;
  "aria-orientation"?: "horizontal" | "vertical";
  "aria-controls"?: string;
  "aria-activedescendant"?: string;
  "aria-checked"?: boolean | "mixed";
  "aria-current"?: boolean | "date" | "location" | "page" | "step" | "time";
  "aria-selected"?: boolean;
  "aria-pressed"?: boolean | "mixed";
  "aria-valuemin"?: number;
  "aria-valuemax"?: number;
  "aria-valuenow"?: number;
  "aria-valuetext"?: string;
  onClick?: EventHandler<WabouPointerEvent>;
  onContextMenu?: EventHandler<WabouPointerEvent>;
  onPointerEnter?: EventHandler<WabouPointerEvent>;
  onPointerLeave?: EventHandler<WabouPointerEvent>;
  onPointerOver?: EventHandler<WabouPointerEvent>;
  onPointerOut?: EventHandler<WabouPointerEvent>;
  onPointerDown?: EventHandler<WabouPointerEvent>;
  onPointerMove?: EventHandler<WabouPointerEvent>;
  onPointerUp?: EventHandler<WabouPointerEvent>;
  onPointerCancel?: EventHandler<WabouPointerEvent>;
  onDblClick?: EventHandler<WabouPointerEvent>;
  onKeyDown?: EventHandler<WabouKeyEvent>;
  onKeyUp?: EventHandler<WabouKeyEvent>;
  onFocus?: EventHandler<WabouNodeEvent>;
  onBlur?: EventHandler<WabouNodeEvent>;
  onFocusIn?: EventHandler<WabouNodeEvent>;
  onFocusOut?: EventHandler<WabouNodeEvent>;
  onWheel?: EventHandler<WabouWheelEvent>;
  onScroll?: EventHandler<WabouScrollEvent>;
  onImeEnabled?: EventHandler<WabouNodeEvent>;
  onImePreedit?: EventHandler<WabouImePreeditEvent>;
  onImeCommit?: EventHandler<WabouTextCommitEvent>;
  onImeDeleteSurrounding?: EventHandler<WabouImeDeleteSurroundingEvent>;
  onImeDisabled?: EventHandler<WabouNodeEvent>;
  /** Preventing this event keeps the native window open. */
  onWindowCloseRequested?: EventHandler<WabouNodeEvent>;
}

export interface WabouControlProps extends WabouElementProps {
  disabled?: boolean;
}

export interface WabouVectorPathProps extends WabouElementProps {
  source: VectorPath;
}

export interface WabouInputProps extends WabouControlProps {
  type?: "text";
  value?: string;
  placeholder?: string;
  readOnly?: boolean;
  onInput?: EventHandler<WabouInputEvent>;
}

export interface WabouImageProps extends WabouElementProps {
  src?: string;
}

export interface WabouSvgProps extends WabouElementProps {
  viewBox?: string;
  fill?: string;
}

export interface WabouSvgShapeProps extends WabouElementProps {
  d?: string;
  cx?: string | number;
  cy?: string | number;
  r?: string | number;
  fill?: string;
  stroke?: string;
  "stroke-width"?: string | number;
  "stroke-linecap"?: "butt" | "round" | "square";
  opacity?: string | number;
}

/** Event shape emitted by a native Wabou node or custom widget. */
export interface WabouEventTarget {
  readonly id: NodeKey;
}

export interface WabouNodeEvent<T extends object = object> {
  readonly type: string;
  readonly target: WabouEventTarget & Partial<T>;
  readonly currentTarget: WabouEventTarget & Partial<T>;
  readonly defaultPrevented: boolean;
  readonly propagationStopped: boolean;
  preventDefault(): void;
  stopPropagation(): void;
  stopImmediatePropagation(): void;
  readonly payload: T;
}

/** Whether a bubbled handler is running on the node originally hit. */
export function isDirectEvent(
  event: Pick<WabouNodeEvent, "target" | "currentTarget">,
): boolean {
  return nodeKeyEquals(event.target.id, event.currentTarget.id);
}

export interface WabouPositionedEvent extends WabouNodeEvent {
  readonly clientX: number;
  readonly clientY: number;
  readonly offsetX: number;
  readonly offsetY: number;
}

export interface WabouPointerEvent extends WabouPositionedEvent {
  readonly button: number;
  readonly buttons: number;
  readonly mods: number;
}

export type WabouGlobalPointerEventType =
  | "pointerdown"
  | "pointerup"
  | "pointermove"
  | "click"
  | "contextmenu";

export type WabouGlobalPointerListener = (
  target: Handle | undefined,
  event: WabouPointerEvent,
) => void;

const globalPointerListeners = new Map<
  WabouGlobalPointerEventType,
  Set<WabouGlobalPointerListener>
>();

/** Observe native pointer dispatch before ordinary bubbling. */
export function observeGlobalPointerEvent(
  type: WabouGlobalPointerEventType,
  listener: WabouGlobalPointerListener,
): () => void {
  const listeners = globalPointerListeners.get(type) ?? new Set();
  listeners.add(listener);
  globalPointerListeners.set(type, listeners);
  return () => {
    listeners.delete(listener);
    if (listeners.size === 0) globalPointerListeners.delete(type);
  };
}

export interface WabouKeyEvent extends WabouNodeEvent {
  readonly key: string;
  readonly code: string;
  /** Physical Shift, Control, Alt, and Meta modifier bits. */
  readonly mods: number;
  /** Whether the physical modifiers form the platform Primary chord. */
  readonly primary: boolean;
  readonly repeat: boolean;
}

export interface WabouWheelEvent extends WabouPositionedEvent {
  readonly deltaX: number;
  readonly deltaY: number;
}

export interface WabouScrollEvent extends WabouNodeEvent {
  readonly scrollX?: number;
  readonly scrollY?: number;
}

export interface WabouInputEvent extends WabouNodeEvent {
  readonly currentTarget: WabouEventTarget & { value: string };
}

export interface WabouTextCommitEvent extends WabouNodeEvent {
  readonly data: string;
  readonly source: "keyboard" | "ime" | "paste";
}

export interface WabouImePreeditEvent extends WabouNodeEvent {
  readonly data: string;
  readonly cursorStart: number | null;
  readonly cursorEnd: number | null;
}

export interface WabouImeDeleteSurroundingEvent extends WabouNodeEvent {
  readonly beforeBytes: number;
  readonly afterBytes: number;
}

export interface NativeScrollbarStyle {
  visibility?: "auto" | "always" | "hidden";
  /** Idle time before an auto scrollbar fades, in milliseconds. */
  hideDelay?: number;
  /** Auto-hide fade duration in milliseconds. Zero hides immediately. */
  fadeDuration?: number;
  thickness?: number;
  margin?: number;
  minThumbLength?: number;
  /** Negative or omitted uses a pill radius. */
  radius?: number;
  /** Packed colors in `0xRRGGBBAA` order. */
  trackColor?: number;
  thumbColor?: number;
  hoverColor?: number;
  activeColor?: number;
}

// Host-provided global (injected by Rust) for logging from the renderer.
declare function __wabou_log(level: "error", message: string): void;

/** A pure-JS handle standing in for a DOM node. id == protocol node id. */
export interface Handle {
  readonly id: NodeKey;
  tag: string;
  parent: Handle | null;
  firstChild: Handle | null;
  lastChild: Handle | null;
  prev: Handle | null;
  next: Handle | null;
  /** Request native keyboard focus for this node on the next bridge flush. */
  focus(): void;
  /** Set this overflow container's native scroll offset. */
  scrollTo(options: { left?: number; top?: number }): void;
  scrollTo(left: number, top: number): void;
  /** Adjust this overflow container's native scroll offset. */
  scrollBy(options: { left?: number; top?: number }): void;
  scrollBy(left: number, top: number): void;
}

const nodeKeys = new NodeKeyAllocator();
const listenersByNode = new NodeKeyTable<Map<number, (e: unknown) => void>>();
/** NodeKey -> WeakRef<Handle>, so bubbling does not retain detached nodes. */
const nodesByKey = new NodeKeyTable<WeakRef<Handle>>();
const classesByNode = new WeakMap<
  Handle,
  { base: string; toggles: Record<string, boolean> }
>();
const interactionByNode = new WeakMap<
  Handle,
  { focusOrder: number | null; blocked: boolean; contained: boolean }
>();

function emitInteractionPolicy(writer: Writer, node: Handle): void {
  const state = interactionByNode.get(node) ?? {
    focusOrder: null,
    blocked: false,
    contained: false,
  };
  let flags = 0;
  if (state.focusOrder !== null) flags |= INTERACTION_POLICY.Focusable;
  if (state.blocked) flags |= INTERACTION_POLICY.BlockSubtree;
  if (state.contained) flags |= INTERACTION_POLICY.ContainFocus;
  writer.setInteractionPolicy(node.id, flags, state.focusOrder ?? 0);
}

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
    ? new FinalizationRegistry<NodeKey>((id) => {
        if (!nodeKeys.isLive(id)) return;
        nodesByKey.delete(id);
        listenersByNode.delete(id);
        writer.dropNode(id);
        nodeKeys.release(id);
      })
    : null;

const sweepSet = new Set<Handle>();

export function runSweep(): void {
  if (sweepSet.size === 0) return;
  for (const node of sweepSet) {
    if (node.parent !== null) continue; // re-attached

    // recursively destroy
    const destroy = (n: Handle) => {
      if (!nodesByKey.has(n.id)) return;

      finalizationRegistry?.unregister(n);
      nodesByKey.delete(n.id);
      listenersByNode.delete(n.id);
      writer.dropNode(n.id);
      nodeKeys.release(n.id);
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

function imperativeMethods(
  id: NodeKey,
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
  const id = nodeKeys.allocate();
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
  interactionByNode.set(h, {
    focusOrder: null,
    blocked: false,
    contained: false,
  });
  if (typeof WeakRef !== "undefined") {
    nodesByKey.set(id, new WeakRef(h));
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
  if (name === "overlayPlane") {
    const plane = value === "modal" ? 2 : value === "floating" ? 1 : 0;
    writer.setOverlayPlane(node.id, plane);
    return;
  }
  if (name === "textBehavior") {
    const behavior =
      value && typeof value === "object"
        ? (value as { flags?: unknown; maxLines?: unknown })
        : { flags: value, maxLines: 0 };
    const flags =
      behavior.flags == null || behavior.flags === false
        ? 0
        : Number(behavior.flags);
    const maxLines =
      behavior.maxLines == null || behavior.maxLines === false
        ? 0
        : Number(behavior.maxLines);
    writer.setTextBehavior(node.id, flags);
    writer.setTextMaxLines(node.id, maxLines);
    return;
  }
  if (
    name === "focusOrder" ||
    name === "interactionBlocked" ||
    name === "focusContained"
  ) {
    const state = interactionByNode.get(node)!;
    if (name === "focusOrder") {
      state.focusOrder =
        value == null || value === false ? null : Number(value);
    } else if (name === "interactionBlocked") {
      state.blocked = value === true;
    } else {
      state.contained = value === true;
    }
    emitInteractionPolicy(writer, node);
    return;
  }
  if (name === "scrollbar") {
    const style = (
      value && typeof value === "object" ? value : {}
    ) as NativeScrollbarStyle;
    writer.setScrollbarStyle(node.id, {
      visibility:
        style.visibility === "always"
          ? 1
          : style.visibility === "hidden"
            ? 2
            : 0,
      hideDelay: style.hideDelay ?? 500,
      fadeDuration: style.fadeDuration ?? 200,
      thickness: style.thickness ?? 10,
      margin: style.margin ?? 2,
      minThumbLength: style.minThumbLength ?? 32,
      radius: style.radius ?? -1,
      trackColor: style.trackColor ?? 0x00000000,
      thumbColor: style.thumbColor ?? 0x64748fbe,
      hoverColor: style.hoverColor ?? 0x64748fe1,
      activeColor: style.activeColor ?? 0x475569ff,
    });
    return;
  }
  if (name === "source") {
    if (node.tag === "vector-path") {
      if (value == null || value === false)
        writer.clearGraphicData(node.id, GRAPHIC_DATA.VectorPath);
      else if (isVectorPath(value)) {
        if (value.drawable)
          writer.setGraphicData(node.id, GRAPHIC_DATA.VectorPath, value.data);
        else writer.clearGraphicData(node.id, GRAPHIC_DATA.VectorPath);
      } else throw new TypeError("invalid native vector path source");
      return;
    }
    if (node.tag === "svg") {
      if (value == null || value === false) {
        writer.clearGraphicSource(node.id, GRAPHIC_SOURCE.Svg);
      } else if (typeof value === "string") {
        writer.setGraphicSource(node.id, GRAPHIC_SOURCE.Svg, value);
      } else {
        throw new TypeError("invalid native SVG source");
      }
      return;
    }
  }
  if (name === "resource") {
    if (node.tag !== "img") throw new TypeError("resource is only supported by Image");
    if (value == null || value === false) {
      writer.clearGraphicSource(node.id, GRAPHIC_SOURCE.ResourceRaster);
      return;
    }
    if (typeof value !== "object") throw new TypeError("Image requires an image resource handle");
    const handle = value as { lo?: unknown; hi?: unknown };
    if (!Number.isInteger(handle.lo) || !Number.isInteger(handle.hi))
      throw new TypeError("Image requires an image resource handle");
    writer.setGraphicSource(node.id, GRAPHIC_SOURCE.ResourceRaster, `${handle.lo}:${handle.hi}`);
    return;
  }
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
  if (name === "widgetConfig") {
    if (value == null || value === false) {
      writer.removeWidgetConfig(node.id);
      return;
    }
    if (!isStructuredConfigValue(value)) {
      throw new TypeError("widgetConfig must be a plain object or array");
    }
    writer.setWidgetConfig(node.id, stringifyWidgetConfig(value));
    return;
  }
  // ARIA boolean values are enumerated strings, not HTML boolean
  // attributes. `false` must cross the bridge instead of removing the
  // attribute, otherwise accessibility cannot distinguish false from unknown.
  if (name.startsWith("aria-") && typeof value === "boolean") {
    writer.setAttribute(node.id, name, String(value));
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
        writer.removeEventListener(node.id, t);
        listenersByNode.get(node.id)?.delete(t);
      }
      return;
    }
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
      assertInlineStyleValue(k, next);
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
  if (name.startsWith("on") && typeof value === "function") {
    const t = EVENT_CODE[name.slice(2).toLowerCase() as EventType];
    if (t == null) return;
    writer.addEventListener(node.id, t);
    let m = listenersByNode.get(node.id);
    if (!m) {
      m = new Map();
      listenersByNode.set(node.id, m);
    }
    m.set(t, value as (e: unknown) => void);
    return;
  }
  if (isStructuredConfigValue(value)) {
    throw new TypeError(
      `object prop \`${name}\` is unsupported; use \`widgetConfig\` for native widget configuration`,
    );
  }
  writer.setAttribute(node.id, name, String(value));
}

const MAX_WIDGET_CONFIG_DEPTH = 32;

function isStructuredConfigValue(value: unknown): value is object {
  if (Array.isArray(value)) return true;
  if (value === null || typeof value !== "object") return false;
  const proto = Object.getPrototypeOf(value);
  return proto === Object.prototype || proto === null;
}

function stringifyWidgetConfig(value: object): string {
  const ancestors = new Set<object>();
  const visit = (current: unknown, depth: number): void => {
    if (depth > MAX_WIDGET_CONFIG_DEPTH) {
      throw new TypeError(
        `widgetConfig exceeds ${MAX_WIDGET_CONFIG_DEPTH} levels`,
      );
    }
    if (
      current === null ||
      typeof current === "string" ||
      typeof current === "boolean"
    )
      return;
    if (typeof current === "number") {
      if (!Number.isFinite(current))
        throw new TypeError("widgetConfig contains a non-finite number");
      return;
    }
    if (typeof current !== "object" || !isStructuredConfigValue(current)) {
      throw new TypeError("widgetConfig contains a non-JSON value");
    }
    if (ancestors.has(current))
      throw new TypeError("widgetConfig contains a cycle");
    ancestors.add(current);
    if (Array.isArray(current)) {
      for (const item of current) visit(item, depth + 1);
    } else {
      for (const item of Object.values(current)) visit(item, depth + 1);
    }
    ancestors.delete(current);
  };
  visit(value, 0);
  return JSON.stringify(value);
}

// ---------------------------------------------------------------------------
// The single renderer + shared writer. Module-level so the named exports
// consumed by compiled JSX are available at load time.
// ---------------------------------------------------------------------------

const writer = new Writer();

const renderer = solidCreateRenderer<Handle>({
  createElement(tag, staticProps) {
    const h = makeHandle(tag);
    writer.createElement(h.id, tag);
    if (staticProps) {
      for (const [name, value] of Object.entries(staticProps)) {
        applyProperty(writer, h, name, value, undefined);
      }
    }
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
export const applyRef = renderer.applyRef;
export const ref = renderer.ref;

type DynamicComponent = (props: never) => JSX.Element;
type DynamicTarget = WabouNativeTag | DynamicComponent;
export type DynamicProps<T extends DynamicTarget> = {
  component: T;
} & (T extends WabouNativeTag
  ? WabouNativeElements[T]
  : T extends (props: infer Props) => unknown
    ? Props
    : never);

export function Dynamic<T extends DynamicTarget>(
  props: DynamicProps<T>,
): JSX.Element {
  const local = props as { component: DynamicTarget };
  const others = omit(props as unknown as Record<string, unknown>, "component");
  const cached = createMemo(() => local.component);

  return createMemo(() => {
    const component = cached();
    switch (typeof component) {
      case "function":
        return untrack(() => component(others as never));
      case "string": {
        const el = createElement(component);
        spread(el, others, false);
        return el;
      }
    }
    return null;
  }) as unknown as JSX.Element;
}

/** Register the root mount handle so bubbling reaches window-level listeners. */
export function registerRoot(root: Handle): void {
  if (typeof WeakRef !== "undefined") {
    nodesByKey.set(root.id, new WeakRef(root));
  }
}

/** Dispose callback for the last `mount()` — used by in-process HMR full reload. */
let activeMountDispose: (() => void) | null = null;
let mountedRoot: Handle | null = null;
type PublicOverlayPlane = "floating" | "modal";
const overlayRoots = new Map<
  PublicOverlayPlane,
  { node: Handle; users: number }
>();

/** Current native window root, used by renderer-level facilities like Portal. */
export function getMountRoot(): Handle {
  if (!mountedRoot) throw new Error("Portal must be rendered inside mount()");
  return mountedRoot;
}

/** Acquire the shared synthetic host root for one public overlay plane. */
export function acquireOverlayRoot(plane: PublicOverlayPlane): Handle {
  const existing = overlayRoots.get(plane);
  if (existing) {
    existing.users++;
    return existing.node;
  }
  const node = createElement("view") as Handle;
  spread(
    node,
    {
      overlayPlane: plane,
      style: {
        position: "absolute",
        left: 0,
        top: 0,
        width: "100%",
        height: "100%",
        "pointer-events": "none",
      },
    },
    false,
  );
  insertNode(getMountRoot(), node, undefined);
  overlayRoots.set(plane, { node, users: 1 });
  return node;
}

export function releaseOverlayRoot(plane: PublicOverlayPlane): void {
  const entry = overlayRoots.get(plane);
  if (!entry || --entry.users > 0) return;
  overlayRoots.delete(plane);
  if (entry.node.parent) removeNode(entry.node.parent, entry.node);
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
    id: ROOT_NODE_KEY,
    tag: "#root",
    parent: null,
    firstChild: null,
    lastChild: null,
    prev: null,
    next: null,
    ...imperativeMethods(ROOT_NODE_KEY),
  };
  mountedRoot = root;
  overlayRoots.clear();
  registerRoot(root);
  const dispose = render(code, root);
  activeMountDispose = () => {
    dispose();
    overlayRoots.clear();
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
  solidId: NodeKey,
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
        eventCode === EVENT_CODE.pointerdown ||
        eventCode === EVENT_CODE.pointermove ||
        eventCode === EVENT_CODE.pointerup ||
        eventCode === EVENT_CODE.pointerenter ||
        eventCode === EVENT_CODE.pointerleave ||
        eventCode === EVENT_CODE.pointercancel ||
        eventCode === EVENT_CODE.pointerover ||
        eventCode === EVENT_CODE.pointerout ||
        eventCode === EVENT_CODE.click ||
        eventCode === EVENT_CODE.contextmenu ||
        eventCode === EVENT_CODE.dblclick
      ) {
        data.clientX = ed[0];
        data.clientY = ed[1];
        data.offsetX = ed[2];
        data.offsetY = ed[3];
        data.button = ed[4];
        data.buttons = ed[5];
        data.mods = ed[6];
      } else if (eventCode === EVENT_CODE.wheel) {
        data.clientX = ed[0];
        data.clientY = ed[1];
        data.offsetX = ed[2];
        data.offsetY = ed[3];
        data.deltaX = ed[7];
        data.deltaY = ed[8];
      } else if (eventCode === EVENT_CODE.scroll) {
        data.scrollX = ed[9];
        data.scrollY = ed[10];
      }
    }
  }

  let stopped = false;
  let defaultPrevented = false;
  const ev = {
    target: { id: solidId, ...data },
    currentTarget: { id: solidId, ...data },
    type: eventName(eventCode),
    payload: data,
    ...data,
    stopPropagation() {
      stopped = true;
    },
    stopImmediatePropagation() {
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

  const globalType = ev.type as WabouGlobalPointerEventType;
  const globalListeners = globalPointerListeners.get(globalType);
  if (globalListeners) {
    const target = derefHandle(solidId);
    for (const listener of [...globalListeners]) {
      try {
        listener(target, ev as WabouPointerEvent);
      } catch (error) {
        logEventHandlerFailure(eventCode, solidId, solidId, error);
      }
    }
  }

  bubble(solidId, eventCode, ev);
  return defaultPrevented;
}

function derefHandle(id: NodeKey): Handle | undefined {
  return nodesByKey.get(id)?.deref();
}

/** Walk parent chain from `nodeId`, firing `code` listeners until stopped. */
function bubble(nodeId: NodeKey, code: number, ev: any): void {
  let cur: NodeKey | null = nodeId;
  while (cur != null) {
    ev.currentTarget = nodeKeyEquals(cur, nodeId) ? ev.target : { id: cur };
    const m = listenersByNode.get(cur);
    const fn = m?.get(code);
    if (fn) {
      try {
        const result = fn(ev);
        if (isPromiseLike(result)) {
          const current = cur;
          void Promise.resolve(result).then(undefined, (error) =>
            logEventHandlerFailure(code, current, nodeId, error),
          );
        }
      } catch (e) {
        logEventHandlerFailure(code, cur, nodeId, e);
      }
    }
    if (ev.propagationStopped) return;
    cur = derefHandle(cur)?.parent?.id ?? null;
  }
}

function isPromiseLike(value: unknown): value is PromiseLike<unknown> {
  return (
    value !== null &&
    (typeof value === "object" || typeof value === "function") &&
    typeof (value as { then?: unknown }).then === "function"
  );
}

function logEventHandlerFailure(
  code: number,
  current: NodeKey,
  target: NodeKey,
  error: unknown,
): void {
  const detail =
    error && typeof error === "object" && "stack" in error
      ? String((error as { stack?: unknown }).stack ?? error)
      : String(error);
  __wabou_log(
    "error",
    `[wabou-event] ${eventName(code)} handler failed at node ${formatNodeKey(current)} (target ${formatNodeKey(target)})\n${detail}`,
  );
}

/** event code -> DOM event name (for ev.type). */
function eventName(code: number): string {
  for (const [name, c] of Object.entries(EVENT_CODE)) {
    if (c === code) return name;
  }
  return "unknown";
}

export {
  type BuiltinHost,
  type DebugOverlayPaintStats,
  type DebugOverlayOptions,
  defaultHost,
  type FrameStats,
  type Host,
  HostProvider,
  type HostProviderProps,
  type LayoutNodeMetrics,
  type LayoutRect,
  type LayoutScrollMetrics,
  type LayoutSnapshot,
  type LayoutTarget,
  useHost,
} from "./host";
export { Portal, type PortalProps } from "./portal";
export { createFps } from "./use-fps";
export { VirtualList, type VirtualListProps } from "./virtual-list";
export type { JSX, Writer };
export { EVENT_CODE, OP };
