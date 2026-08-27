import { g as NodeKey$1, h as Writer } from "./protocol-pMJDDBQV.mjs";
import { n as WabouIntrinsicElements, t as HostCapabilities } from "./registry-DXOPfC3L.mjs";
import { A as WabouStyle, t as Affine2D } from "./style-BWA6KX47.mjs";
import { Accessor, Element as Element$1, JSX } from "solid-js";
//#region src/vector-path.d.ts
type PathFillRule = "nonzero" | "evenodd";
type PathLineCap = "butt" | "round" | "square";
type PathLineJoin = "miter" | "round" | "bevel";
interface VectorPathPaint {
  /** Packed RGBA (`0xRRGGBBAA`). Omit to disable filling. */
  fill?: number;
  /** Packed RGBA (`0xRRGGBBAA`). Omit to disable stroking. */
  stroke?: number;
  strokeWidth?: number;
  fillRule?: PathFillRule;
  lineCap?: PathLineCap;
  lineJoin?: PathLineJoin;
  miterLimit?: number;
}
interface PathPoint {
  readonly x: number;
  readonly y: number;
}
/** Immutable path snapshot suitable for signals, memos, and component props. */
interface VectorPath {
  readonly kind: "wabou-vector-path";
  /** Whether the command stream contains at least one drawable segment. */
  readonly drawable: boolean;
  readonly data: Uint8Array;
}
declare class PathBuilder {
  #private;
  /** Whether line/curve/close commands currently have an active subpath. */
  get hasCurrentPoint(): boolean;
  moveTo(x: number, y: number): this;
  lineTo(x: number, y: number): this;
  quadTo(cx: number, cy: number, x: number, y: number): this;
  cubicTo(c1x: number, c1y: number, c2x: number, c2y: number, x: number, y: number): this;
  close(): this;
  /** Append a Catmull–Rom spline converted to native cubic Bézier segments. */
  splineThrough(points: readonly PathPoint[], tension?: number): this;
  /** Create an immutable snapshot. Later builder mutations cannot alter it. */
  build(paint?: VectorPathPaint): VectorPath;
}
declare function isVectorPath(value: unknown): value is VectorPath;
//#endregion
//#region src/renderer/jsx.d.ts
/** Renderer-owned JSX namespace for Solid 2's automatic JSX type lookup. */
declare namespace JSX$1 {
  type Element = Element$1 | Handle | readonly Element[];
  type CSSProperties = WabouStyle;
  type ElementClass = {};
  type ElementAttributesProperty = {};
  interface ElementChildrenAttribute {
    children: {};
  }
  interface IntrinsicElements extends WabouBuiltinIntrinsicElements, WabouIntrinsicElements {}
  type IntrinsicAttributes = {};
}
declare module "solid-js" {
  namespace JSX {
    type Element = JSX$1.Element;
    type CSSProperties = JSX$1.CSSProperties;
    interface ElementClass {}
    interface ElementAttributesProperty {}
    interface ElementChildrenAttribute {
      children: {};
    }
    interface IntrinsicElements extends WabouBuiltinIntrinsicElements, WabouIntrinsicElements {}
    interface IntrinsicAttributes {}
  }
}
declare function jsx(): never;
declare const jsxs: typeof jsx;
declare const jsxDEV: typeof jsx;
declare const Fragment: (props: WabouElementProps) => JSX$1.Element;
//#endregion
//#region src/renderer/generated/native-host.d.ts
type CalendarDateInfo = {
  /**  Proleptic Gregorian year. */
  year: number;
  /**  One-based month. */
  month: number;
  /**  One-based day of month. */
  day: number;
};
type DebugOverlayPaintStats = {
  /**  Monotonic paint-pass sequence. Zero means no pass has completed. */
  sequence: number;
  /**  Whether an overlay was enabled for that pass. */
  enabled: boolean;
  /**  Number of layout border boxes stroked into the debug scene. */
  layout_bounds: number;
  /**  Number of unique clip rectangles stroked into the debug scene. */
  clip_bounds: number;
  /**  Number of hit-target or selected-node highlights painted. */
  highlights: number;
};
type FrameStats = {
  /**  Total Rust frame construction time in milliseconds. */
  build_frame_ms: number;
  /**  QuickJS animation-frame callback time in milliseconds. */
  js_tick_ms: number;
  /**  Backend-neutral AnyRender scene construction time in milliseconds. */
  scene_ms: number;
  /**  Surface rendering and presentation time in milliseconds. */
  present_ms: number;
  /**  Number of retained nodes in the frame. */
  node_count: number;
  /**  Logical viewport width. */
  viewport_w: number;
  /**  Logical viewport height. */
  viewport_h: number;
};
type LayoutNodeMetrics = {
  /**  Solid-side node identifier. */
  id: NodeKey;
  /**  Border box in logical window coordinates. */
  rect: LayoutRect;
  /**  Effective ancestor clip in logical window coordinates. */
  clip: LayoutRect;
  /**  Current offset and scrollable range in logical pixels. */
  scroll: LayoutScrollMetrics;
};
type LayoutRect = {
  /**  Left edge. */
  x: number;
  /**  Top edge. */
  y: number;
  /**  Non-negative width. */
  width: number;
  /**  Non-negative height. */
  height: number;
};
type LayoutScrollMetrics = {
  /**  Current horizontal scroll offset. */
  offsetX: number;
  /**  Current vertical scroll offset. */
  offsetY: number;
  /**  Maximum horizontal scroll offset. */
  rangeX: number;
  /**  Maximum vertical scroll offset. */
  rangeY: number;
};
type LayoutSnapshot = {
  /**  Monotonic layout revision used to detect stale snapshots. */
  revision: number;
  /**  Current logical viewport. */
  viewport: LayoutRect;
  /**  Metrics for the requested node identifiers that still exist. */
  nodes: LayoutNodeMetrics[];
};
type NodeKey = {
  /**  Slot index. Zero is reserved by the wire protocol. */
  lo: number;
  /**  Non-zero odd generation, matching SlotMap's FFI representation. */
  hi: number;
};
//#endregion
//#region src/renderer/host.d.ts
type LayoutTarget = NodeKey$1 | {
  readonly id: NodeKey$1;
};
interface DebugOverlayOptions {
  /** Draw every retained node's border box. */
  layout?: boolean;
  /** Draw unique effective overflow and scroll clips. */
  clips?: boolean;
  /** Draw the current native pointer hit target. */
  hitTarget?: boolean;
}
interface BuiltinHost {
  readonly system: {
    /** Open an http(s) URL using the operating system's default handler. */
    openUrl(url: string): boolean;
  };
  readonly fonts: {
    /** Load a TTF/OTF file into this window's text context. */
    load(path: string): boolean;
  };
  readonly diagnostics: {
    /** Latest render timings, or null before the first completed frame. */
    frameStats(): FrameStats | null;
    /** Configure native diagnostic layers. Returns false without DevTools support. */
    setOverlay(options: DebugOverlayOptions): boolean;
    /** Evidence from the most recent native overlay paint pass. */
    overlayPaintStats(): DebugOverlayPaintStats | null;
  };
  readonly intl: {
    /** Locale reported by the operating system, falling back to en-US. */
    locale(): string;
    /** IANA time-zone identifier reported by the operating system. */
    timeZone(): string;
    /** Current Gregorian date in the operating system's local time zone. */
    today(): CalendarDateInfo;
  };
  readonly layout: {
    /** Measure several nodes from one coherent, completed native layout. */
    snapshot(targets: readonly LayoutTarget[]): LayoutSnapshot;
    /** Latest completed border-box measurement, or null for a stale handle. */
    measure(target: LayoutTarget): LayoutRect | null;
    /** Effective ancestor overflow clip, falling back to the viewport. */
    clippingRect(target: LayoutTarget): LayoutRect | null;
    /** Logical-pixel viewport from the latest completed layout. */
    viewport(): LayoutRect;
  };
}
/** Augment `HostCapabilities` in generated/user declarations. */
type Host = BuiltinHost & HostCapabilities;
declare const defaultHost: Host;
interface HostProviderProps {
  value: Host;
  children?: JSX.Element;
}
/** Bind host capabilities to a Solid subtree (normally one window). */
declare function HostProvider(props: HostProviderProps): JSX.Element;
/** Return the host associated with the current Solid owner/window. */
declare function useHost<T extends Host = Host>(): T;
//#endregion
//#region src/renderer/portal.d.ts
interface PortalProps extends Omit<WabouElementProps, "children"> {
  children?: JSX.Element;
  /** Host stacking plane. `system` and `debug` are reserved for native UI. */
  plane?: "floating" | "modal";
  /** Contain native focus traversal within this overlay subtree. */
  focusContained?: boolean;
}
/** Render a native host subtree under its shared synthetic overlay root. */
declare function Portal(props: PortalProps): JSX.Element;
//#endregion
//#region src/renderer/virtual-list.d.ts
interface VirtualListProps<T> {
  /** Accessor for the full backing array. Only the visible slice renders. */
  items: () => readonly T[];
  /** Fixed height of every row, in logical pixels. */
  itemHeight: number;
  /**
   * Visible viewport height in logical pixels. When omitted, the list fills
   * its bounded parent and observes its completed native layout size.
   */
  viewportHeight?: number;
  /** Classes applied to the native scroll viewport. */
  class?: string;
  /** Extra rows rendered above/below the viewport. Defaults to 4. */
  overscan?: number;
  /** Stable application identity. Required so refreshed objects do not remount rows. */
  getItemKey: (item: T, index: number) => string | number;
  /** Explicit semantic role for the viewport, such as `listbox`. */
  role?: WabouSemanticRole;
  /** Accessible name for the native scroll viewport. */
  accessibilityLabel?: string;
  /** Render a single row given its item and absolute index. */
  children: (item: Accessor<T>, index: Accessor<number>) => JSX.Element;
}
/**
 * Windowed Solid list backed by TanStack Virtual's framework-neutral core.
 * Rust remains authoritative for scrolling, clipping, hit testing and the
 * native scrollbar; this adapter supplies viewport/offset observations instead
 * of relying on HTMLElement, ResizeObserver or getBoundingClientRect().
 */
declare function VirtualList<T>(props: VirtualListProps<T>): JSX.Element;
//#endregion
//#region src/renderer/index.d.ts
declare const isServer = false;
declare const getRequestEvent: () => undefined;
declare const delegateEvents: () => void;
/**
 * Deliberately small set of structural host tags understood by Wabou apps.
 * This is not the HTML element registry: unsupported Web tags must be wrapped
 * by an explicit component or registered as a custom native element.
 */
interface WabouBuiltinIntrinsicElements {
  view: WabouElementProps;
  "vector-path": WabouVectorPathProps;
}
type WabouNativeElements = WabouBuiltinIntrinsicElements & WabouIntrinsicElements;
type WabouNativeTag = keyof WabouNativeElements & string;
type WabouSemanticRole = "alert" | "alertdialog" | "button" | "cell" | "checkbox" | "columnheader" | "combobox" | "dialog" | "grid" | "gridcell" | "group" | "heading" | "img" | "label" | "link" | "listbox" | "menu" | "menubar" | "menuitem" | "none" | "option" | "presentation" | "progressbar" | "radio" | "radiogroup" | "region" | "row" | "rowheader" | "separator" | "slider" | "spinbutton" | "status" | "switch" | "tab" | "tablist" | "tabpanel" | "table" | "textbox" | "tree" | "treeitem" | "toolbar" | "tooltip";
/** Roles that remain addressable after semantic-tree projection. */
type WabouExposedSemanticRole = Exclude<WabouSemanticRole, "none" | "presentation">;
type EventHandler<E> = {
  bivarianceHack(event: E): unknown;
}["bivarianceHack"];
/** Props shared by low-level native JSX elements. */
interface WabouElementProps {
  id?: string;
  class?: string;
  classList?: Record<string, boolean | undefined>;
  style?: string | WabouStyle;
  children?: JSX$1.Element;
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
interface WabouControlProps extends WabouElementProps {
  disabled?: boolean;
}
interface WabouVectorPathProps extends WabouElementProps {
  source: VectorPath;
}
interface WabouInputProps extends WabouControlProps {
  type?: "text";
  value?: string;
  placeholder?: string;
  readOnly?: boolean;
  onInput?: EventHandler<WabouInputEvent>;
}
interface WabouImageProps extends WabouElementProps {
  src?: string;
}
interface WabouSvgProps extends WabouElementProps {
  viewBox?: string;
  fill?: string;
}
interface WabouSvgShapeProps extends WabouElementProps {
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
interface WabouEventTarget {
  readonly id: NodeKey$1;
}
interface WabouNodeEvent<T extends object = object> {
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
declare function isDirectEvent(event: Pick<WabouNodeEvent, "target" | "currentTarget">): boolean;
interface WabouPositionedEvent extends WabouNodeEvent {
  readonly clientX: number;
  readonly clientY: number;
  readonly offsetX: number;
  readonly offsetY: number;
}
interface WabouPointerEvent extends WabouPositionedEvent {
  readonly button: number;
  readonly buttons: number;
  readonly mods: number;
}
type WabouGlobalPointerEventType = "pointerdown" | "pointerup" | "pointermove" | "click" | "contextmenu";
type WabouGlobalPointerListener = (target: Handle | undefined, event: WabouPointerEvent) => void;
/** Observe native pointer dispatch before ordinary bubbling. */
declare function observeGlobalPointerEvent(type: WabouGlobalPointerEventType, listener: WabouGlobalPointerListener): () => void;
interface WabouKeyEvent extends WabouNodeEvent {
  readonly key: string;
  readonly code: string;
  /** Physical Shift, Control, Alt, and Meta modifier bits. */
  readonly mods: number;
  /** Whether the physical modifiers form the platform Primary chord. */
  readonly primary: boolean;
  readonly repeat: boolean;
}
interface WabouWheelEvent extends WabouPositionedEvent {
  readonly deltaX: number;
  readonly deltaY: number;
}
interface WabouScrollEvent extends WabouNodeEvent {
  readonly scrollX?: number;
  readonly scrollY?: number;
}
interface WabouInputEvent extends WabouNodeEvent {
  readonly currentTarget: WabouEventTarget & {
    value: string;
  };
}
interface WabouTextCommitEvent extends WabouNodeEvent {
  readonly data: string;
  readonly source: "keyboard" | "ime" | "paste";
}
interface WabouImePreeditEvent extends WabouNodeEvent {
  readonly data: string;
  readonly cursorStart: number | null;
  readonly cursorEnd: number | null;
}
interface WabouImeDeleteSurroundingEvent extends WabouNodeEvent {
  readonly beforeBytes: number;
  readonly afterBytes: number;
}
interface NativeScrollbarStyle {
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
/** A pure-JS handle standing in for a DOM node. id == protocol node id. */
interface Handle {
  readonly id: NodeKey$1;
  tag: string;
  parent: Handle | null;
  firstChild: Handle | null;
  lastChild: Handle | null;
  prev: Handle | null;
  next: Handle | null;
  /** Request native keyboard focus for this node on the next bridge flush. */
  focus(): void;
  /** Set this overflow container's native scroll offset. */
  scrollTo(options: {
    left?: number;
    top?: number;
  }): void;
  scrollTo(left: number, top: number): void;
  /** Adjust this overflow container's native scroll offset. */
  scrollBy(options: {
    left?: number;
    top?: number;
  }): void;
  scrollBy(left: number, top: number): void;
}
declare function runSweep(): void;
declare const writer: Writer;
/** Imperative paint-only transform state for high-frequency animation. */
declare function setTransform2D(node: Handle, matrix: Affine2D): void;
declare const render: (code: () => JSX$1.Element, node: Handle) => () => void;
declare const createElement: (tag: string, staticProps?: Record<string, unknown>) => Handle;
declare const createTextNode: (value: string) => Handle;
declare const insertNode: (parent: Handle, node: Handle, anchor?: Handle | undefined) => void;
declare function removeNode(parent: Handle, node: Handle): void;
declare const insert: <T>(parent: any, accessor: T | (() => T), marker?: any | null, initial?: any) => Handle;
declare const setProp: <T>(node: Handle, name: string, value: T, prev?: T | undefined) => T;
declare const createComponent$1: <T>(Comp: (props: T) => Handle, props: T) => Handle;
declare const effect: <T>(fn: (prev?: T) => T, effect: (value: T, prev?: T) => void) => void;
declare const memo: <T>(fn: () => T, equal: boolean) => () => T;
declare const spread: <T extends object>(node: any, props: T, skipChildren?: boolean) => void;
declare const mergeProps: (...sources: unknown[]) => unknown;
declare const applyRef: (r: ((element: Handle) => void) | ((element: Handle) => void)[], element: Handle) => void;
declare const ref: (fn: () => ((element: Handle) => void) | ((element: Handle) => void)[], element: Handle) => void;
type DynamicComponent = (props: never) => JSX$1.Element;
type DynamicTarget = WabouNativeTag | DynamicComponent;
type DynamicProps<T extends DynamicTarget> = {
  component: T;
} & (T extends WabouNativeTag ? WabouNativeElements[T] : T extends ((props: infer Props) => unknown) ? Props : never);
declare function Dynamic<T extends DynamicTarget>(props: DynamicProps<T>): JSX$1.Element;
/** Register the root mount handle so bubbling reaches window-level listeners. */
declare function registerRoot(root: Handle): void;
type PublicOverlayPlane = "floating" | "modal";
/** Current native window root, used by renderer-level facilities like Portal. */
declare function getMountRoot(): Handle;
/** Acquire the shared synthetic host root for one public overlay plane. */
declare function acquireOverlayRoot(plane: PublicOverlayPlane): Handle;
declare function releaseOverlayRoot(plane: PublicOverlayPlane): void;
/** Mount a Solid application into the host-provided root node. */
declare function mount(code: () => JSX$1.Element): () => void;
/**
 * Solid compatibility adapter for a native Wabou event. It walks the Handle
 * tree for bubbling and presents JSX handlers with a small familiar object;
 * this is deliberately not a complete DOM Event implementation.
 */
declare function dispatchEvent(solidId: NodeKey$1, eventCode: number, payloadStr: string, numericData?: ArrayLike<number>): boolean;
//#endregion
export { runSweep as $, acquireOverlayRoot as A, PathLineJoin as At, insert as B, WabouScrollEvent as C, JSX$1 as Ct, WabouTextCommitEvent as D, PathBuilder as Dt, WabouSvgShapeProps as E, jsxs as Et, delegateEvents as F, mergeProps as G, isDirectEvent as H, dispatchEvent as I, ref as J, mount as K, effect as L, createComponent$1 as M, VectorPath as Mt, createElement as N, VectorPathPaint as Nt, WabouVectorPathProps as O, PathFillRule as Ot, createTextNode as P, isVectorPath as Pt, render as Q, getMountRoot as R, WabouPositionedEvent as S, Fragment as St, WabouSvgProps as T, jsxDEV as Tt, isServer as U, insertNode as V, memo as W, releaseOverlayRoot as X, registerRoot as Y, removeNode as Z, WabouKeyEvent as _, FrameStats as _t, WabouBuiltinIntrinsicElements as a, VirtualListProps as at, WabouNodeEvent as b, LayoutScrollMetrics as bt, WabouEventTarget as c, BuiltinHost as ct, WabouGlobalPointerListener as d, HostProvider as dt, setProp as et, WabouImageProps as f, HostProviderProps as ft, WabouInputProps as g, DebugOverlayPaintStats as gt, WabouInputEvent as h, useHost as ht, NativeScrollbarStyle as i, VirtualList as it, applyRef as j, PathPoint as jt, WabouWheelEvent as k, PathLineCap as kt, WabouExposedSemanticRole as l, DebugOverlayOptions as lt, WabouImePreeditEvent as m, defaultHost as mt, DynamicProps as n, spread as nt, WabouControlProps as o, Portal as ot, WabouImeDeleteSurroundingEvent as p, LayoutTarget as pt, observeGlobalPointerEvent as q, Handle as r, writer as rt, WabouElementProps as s, PortalProps as st, Dynamic as t, setTransform2D as tt, WabouGlobalPointerEventType as u, Host as ut, WabouNativeElements as v, LayoutNodeMetrics as vt, WabouSemanticRole as w, jsx as wt, WabouPointerEvent as x, LayoutSnapshot as xt, WabouNativeTag as y, LayoutRect as yt, getRequestEvent as z };
//# sourceMappingURL=index-Bpe-FHMi.d.mts.map