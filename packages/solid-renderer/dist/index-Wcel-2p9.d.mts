import { EVENT_CODE, OP, Writer as Writer$1 } from "@wabou/protocol";
import { Affine2D, WabouStyle } from "@wabou/style";
import { Element as Element$1, JSX } from "solid-js";
//#region src/jsx.d.ts
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
//#region src/generated/native-host.d.ts
type CalendarDateInfo = {
  year: number;
  month: number;
  day: number;
};
type FrameStats = {
  build_frame_ms: number;
  js_tick_ms: number;
  scene_ms: number;
  present_ms: number;
  node_count: number;
  viewport_w: number;
  viewport_h: number;
};
type LayoutNodeMetrics = {
  id: number;
  rect: LayoutRect;
  clip: LayoutRect;
};
type LayoutRect = {
  x: number;
  y: number;
  width: number;
  height: number;
};
type LayoutSnapshot = {
  revision: number;
  viewport: LayoutRect;
  nodes: LayoutNodeMetrics[];
};
//#endregion
//#region src/host.d.ts
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
//#region src/portal.d.ts
interface PortalProps extends Omit<WabouElementProps, "children"> {
  children?: JSX.Element;
  /** Host stacking plane. `system` and `debug` are reserved for native UI. */
  plane?: "floating" | "modal";
  /** Contain native focus traversal within this overlay subtree. */
  focusScope?: "contain";
}
/** Render a native host subtree under its shared synthetic overlay root. */
declare function Portal(props: PortalProps): JSX.Element;
//#endregion
//#region src/use-fps.d.ts
/**
 * Track frames-per-second. A self-perpetuating rAF loop counts frames; a
 * 1s interval samples the count and resets it. The rAF loop keeps the host
 * redrawing (it drives `has_anim`), so this measures the active vsync rate
 * while mounted — ~60 on a 60Hz display, ~120 on 120Hz. When nothing animates,
 * the host stops redrawing and the count drops.
 *
 * ```tsx
 * const fps = createFps();
 * <div>{fps()} fps</div>
 * ```
 */
declare function createFps(): () => number;
/** @deprecated Use createFps; this primitive creates owned timers rather than consuming context. */
declare const useFps: typeof createFps;
//#endregion
//#region src/virtual-list.d.ts
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
//#region src/index.d.ts
declare const isServer = false;
declare const getRequestEvent: () => undefined;
declare const delegateEvents: () => void;
/**
 * Application and widget-package additions to the native Host API.
 *
 * Augment this interface through `declare module "@wabou/solid-renderer"`.
 * It intentionally lives at the package root: augmenting a re-exported
 * interface does not merge into the module where that interface was declared.
 */
interface HostCapabilities {}
/**
 * Low-level native JSX elements supplied by applications and widget packages.
 * Prefer a typed PascalCase component for public widgets, but augment this
 * registry when exposing the underlying custom tag is useful.
 */
interface WabouIntrinsicElements {}
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
  tabIndex?: number;
  inert?: boolean | "";
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
declare const writer: Writer$1;
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
export { spread as $, applyRef as A, insertNode as B, WabouScrollEvent as C, WabouWheelEvent as D, WabouSvgShapeProps as E, dispatchEvent as F, ref as G, memo as H, effect as I, removeNode as J, registerRoot as K, getMountRoot as L, createElement as M, createTextNode as N, Writer$1 as O, delegateEvents as P, setTransform2D as Q, getRequestEvent as R, WabouPositionedEvent as S, WabouSvgProps as T, mergeProps as U, isServer as V, mount as W, runSweep as X, render as Y, setProp as Z, WabouKeyEvent as _, Fragment as _t, HostCapabilities as a, Portal as at, WabouNodeEvent as b, jsxDEV as bt, WabouBuiltinIntrinsicElements as c, HostProvider as ct, WabouEventTarget as d, defaultHost as dt, writer as et, WabouExposedSemanticRole as f, useHost as ft, WabouIntrinsicElements as g, LayoutSnapshot as gt, WabouInputProps as h, LayoutRect as ht, Handle as i, useFps as it, createComponent$1 as j, acquireOverlayRoot as k, WabouControlProps as l, HostProviderProps as lt, WabouInputEvent as m, LayoutNodeMetrics as mt, DynamicProps as n, VirtualListProps as nt, NativeScrollbarStyle as o, PortalProps as ot, WabouImageProps as p, FrameStats as pt, releaseOverlayRoot as q, EVENT_CODE as r, createFps as rt, OP as s, Host as st, Dynamic as t, VirtualList as tt, WabouElementProps as u, LayoutTarget as ut, WabouNativeElements as v, JSX$1 as vt, WabouSemanticRole as w, WabouPointerEvent as x, jsxs as xt, WabouNativeTag as y, jsx as yt, insert as z };
//# sourceMappingURL=index-Wcel-2p9.d.mts.map