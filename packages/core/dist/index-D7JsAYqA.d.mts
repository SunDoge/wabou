import { m as Writer } from "./index-CpjySDmz.mjs";
import { E as WabouStyle, t as Affine2D } from "./index-Bq47FJfe.mjs";
import { n as WabouIntrinsicElements, t as HostCapabilities } from "./registry-DXOPfC3L.mjs";
import { Element as Element$1, JSX } from "solid-js";
//#region ../solid-renderer/src/jsx.d.ts
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
//#region ../solid-renderer/src/generated/native-host.d.ts
type CalendarDateInfo = {
  /**  Proleptic Gregorian year. */
  year: number;
  /**  One-based month. */
  month: number;
  /**  One-based day of month. */
  day: number;
};
type FrameStats = {
  /**  Total Rust frame construction time in milliseconds. */
  build_frame_ms: number;
  /**  QuickJS animation-frame callback time in milliseconds. */
  js_tick_ms: number;
  /**  Vello scene construction time in milliseconds. */
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
  id: number;
  /**  Border box in logical window coordinates. */
  rect: LayoutRect;
  /**  Effective ancestor clip in logical window coordinates. */
  clip: LayoutRect;
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
type LayoutSnapshot = {
  /**  Monotonic layout revision used to detect stale snapshots. */
  revision: number;
  /**  Current logical viewport. */
  viewport: LayoutRect;
  /**  Metrics for the requested node identifiers that still exist. */
  nodes: LayoutNodeMetrics[];
};
//#endregion
//#region ../solid-renderer/src/host.d.ts
type LayoutTarget = number | {
  readonly id: number;
};
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
//#region ../solid-renderer/src/portal.d.ts
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
//#region ../solid-renderer/src/virtual-list.d.ts
interface VirtualListProps<T> {
  /** Accessor for the full backing array. Only the visible slice renders. */
  items: () => readonly T[];
  /** Fixed height of every row, in logical pixels. */
  itemHeight: number;
  /** Visible viewport height in logical pixels. */
  viewportHeight: number;
  /** Extra rows rendered above/below the viewport. Defaults to 4. */
  overscan?: number;
  /** Explicit semantic role for the viewport, such as `listbox`. */
  role?: WabouSemanticRole;
  /** Accessible name for the native scroll viewport. */
  accessibilityLabel?: string;
  /** Render a single row given its item and absolute index. */
  children: (item: T, index: number) => JSX.Element;
}
/**
 * Windowed Solid list backed by TanStack Virtual's framework-neutral core.
 * Rust remains authoritative for scrolling, clipping, hit testing and the
 * native scrollbar; this adapter supplies viewport/offset observations instead
 * of relying on HTMLElement, ResizeObserver or getBoundingClientRect().
 */
declare function VirtualList<T>(props: VirtualListProps<T>): JSX.Element;
//#endregion
//#region ../solid-renderer/src/index.d.ts
declare const isServer = false;
declare const getRequestEvent: () => undefined;
declare const delegateEvents: () => void;
/**
 * Deliberately small set of structural host tags understood by Wabou apps.
 * This is not the HTML element registry: unsupported Web tags must be wrapped
 * by an explicit component or registered as a custom native element.
 */
interface WabouBuiltinIntrinsicElements {
  button: WabouControlProps;
  img: WabouImageProps;
  input: WabouInputProps;
  view: WabouElementProps;
  svg: WabouSvgProps;
  path: WabouSvgShapeProps;
  circle: WabouSvgShapeProps;
}
type WabouNativeElements = WabouBuiltinIntrinsicElements & WabouIntrinsicElements;
type WabouNativeTag = keyof WabouNativeElements & string;
type WabouSemanticRole = "alert" | "button" | "cell" | "checkbox" | "columnheader" | "combobox" | "dialog" | "grid" | "gridcell" | "group" | "heading" | "img" | "label" | "link" | "listbox" | "menu" | "menuitem" | "none" | "option" | "presentation" | "progressbar" | "radio" | "radiogroup" | "row" | "rowheader" | "slider" | "status" | "switch" | "tab" | "tablist" | "tabpanel" | "table" | "textbox" | "tree" | "treeitem";
/** Roles that remain addressable after semantic-tree projection. */
type WabouExposedSemanticRole = Exclude<WabouSemanticRole, "none" | "presentation">;
type EventHandler<E> = {
  bivarianceHack(event: E): void;
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
  "aria-label"?: string;
  "aria-hidden"?: boolean | "true" | "false";
  "aria-modal"?: boolean | "true" | "false";
  "aria-haspopup"?: "dialog" | "grid" | "listbox" | "menu" | "tree";
  "aria-expanded"?: boolean;
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
}
interface WabouControlProps extends WabouElementProps {
  disabled?: boolean;
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
  readonly id: number;
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
  id: number;
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
declare function dispatchEvent(solidId: number, eventCode: number, payloadStr: string, numericData?: ArrayLike<number>): boolean;
//#endregion
export { PortalProps as $, dispatchEvent as A, ref as B, WabouWheelEvent as C, createElement as D, createComponent$1 as E, insertNode as F, runSweep as G, releaseOverlayRoot as H, isServer as I, spread as J, setProp as K, memo as L, getMountRoot as M, getRequestEvent as N, createTextNode as O, insert as P, Portal as Q, mergeProps as R, WabouSvgShapeProps as S, applyRef as T, removeNode as U, registerRoot as V, render as W, VirtualList as X, writer as Y, VirtualListProps as Z, WabouPointerEvent as _, WabouBuiltinIntrinsicElements as a, useHost as at, WabouSemanticRole as b, WabouEventTarget as c, LayoutRect as ct, WabouInputEvent as d, JSX$1 as dt, Host as et, WabouInputProps as f, jsx as ft, WabouNodeEvent as g, WabouNativeTag as h, NativeScrollbarStyle as i, defaultHost as it, effect as j, delegateEvents as k, WabouExposedSemanticRole as l, LayoutSnapshot as lt, WabouNativeElements as m, jsxs as mt, DynamicProps as n, HostProviderProps as nt, WabouControlProps as o, FrameStats as ot, WabouKeyEvent as p, jsxDEV as pt, setTransform2D as q, Handle as r, LayoutTarget as rt, WabouElementProps as s, LayoutNodeMetrics as st, Dynamic as t, HostProvider as tt, WabouImageProps as u, Fragment as ut, WabouPositionedEvent as v, acquireOverlayRoot as w, WabouSvgProps as x, WabouScrollEvent as y, mount as z };
//# sourceMappingURL=index-D7JsAYqA.d.mts.map