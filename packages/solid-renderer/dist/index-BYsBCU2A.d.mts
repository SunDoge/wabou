import { EVENT_CODE, OP, Writer as Writer$1 } from "@wabou/protocol";
import { Affine2D } from "@wabou/style";
import { Element as Element$1, JSX } from "solid-js";
import { JSX as JSX$1 } from "@solidjs/web";
//#region src/jsx.d.ts
declare module "@solidjs/web" {
  namespace JSX {
    interface CustomAttributes<T> {
      classList?: Record<string, boolean | undefined>;
      tabIndex?: number;
    }
  }
}
/** Renderer-owned JSX namespace for Solid 2's automatic JSX type lookup. */
declare namespace JSX$2 {
  type Element = Element$1 | Handle | readonly Element[];
  type CSSProperties = JSX$1.CSSProperties;
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
    type Element = JSX$2.Element;
    type CSSProperties = JSX$2.CSSProperties;
    interface ElementClass {}
    interface ElementAttributesProperty {}
    interface ElementChildrenAttribute {
      children: {};
    }
    interface IntrinsicElements extends WabouBuiltinIntrinsicElements, WabouIntrinsicElements {}
    interface IntrinsicAttributes {}
    type ButtonHTMLAttributes<T> = JSX$1.ButtonHTMLAttributes<T>;
    type InputHTMLAttributes<T> = JSX$1.InputHTMLAttributes<T>;
    type SvgSVGAttributes<T> = JSX$1.SvgSVGAttributes<T>;
  }
}
declare function jsx(): never;
declare const jsxs: typeof jsx;
declare const jsxDEV: typeof jsx;
declare const Fragment: (props: WabouElementProps) => JSX$2.Element;
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
interface PortalProps {
  children?: JSX.Element;
  /** Host stacking plane. `system` and `debug` are reserved for native UI. */
  plane?: "floating" | "modal";
  [name: string]: unknown;
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
  role?: JSX$1.HTMLAttributes<HTMLDivElement>["role"];
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
  article: JSX$1.HTMLAttributes<HTMLElement>;
  aside: JSX$1.HTMLAttributes<HTMLElement>;
  button: JSX$1.ButtonHTMLAttributes<HTMLButtonElement>;
  div: JSX$1.HTMLAttributes<HTMLDivElement>;
  footer: JSX$1.HTMLAttributes<HTMLElement>;
  h1: JSX$1.HTMLAttributes<HTMLHeadingElement>;
  header: JSX$1.HTMLAttributes<HTMLElement>;
  i: JSX$1.HTMLAttributes<HTMLElement>;
  img: JSX$1.ImgHTMLAttributes<HTMLImageElement>;
  input: Omit<JSX$1.InputHTMLAttributes<HTMLInputElement>, "type"> & {
    type?: "text";
  };
  label: JSX$1.LabelHTMLAttributes<HTMLLabelElement>;
  main: JSX$1.HTMLAttributes<HTMLElement>;
  nav: JSX$1.HTMLAttributes<HTMLElement>;
  ol: JSX$1.OlHTMLAttributes<HTMLOListElement>;
  p: JSX$1.HTMLAttributes<HTMLParagraphElement>;
  section: JSX$1.HTMLAttributes<HTMLElement>;
  span: JSX$1.HTMLAttributes<HTMLSpanElement>;
  strong: JSX$1.HTMLAttributes<HTMLElement>;
  svg: JSX$1.SvgSVGAttributes<SVGSVGElement>;
  path: JSX$1.PathSVGAttributes<SVGPathElement>;
  circle: JSX$1.CircleSVGAttributes<SVGCircleElement>;
}
/** Props shared by low-level native JSX elements. */
interface WabouElementProps {
  class?: string;
  classList?: Record<string, boolean | undefined>;
  style?: string | JSX$2.CSSProperties;
  children?: JSX$2.Element;
  ref?: Handle | ((node: Handle) => void);
}
/** Event shape emitted by a native Wabou node or custom widget. */
interface WabouNodeEvent<T extends object = Record<string, unknown>> {
  readonly type: string;
  readonly target: Handle;
  readonly currentTarget: Handle;
  readonly defaultPrevented: boolean;
  readonly propagationStopped: boolean;
  preventDefault(): void;
  stopPropagation(): void;
  stopImmediatePropagation(): void;
  readonly payload: T;
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
declare const render: (code: () => JSX$2.Element, node: Handle) => () => void;
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
declare function Dynamic(props: any): import("solid-js").SourceAccessor<any>;
/** Register the root mount handle so bubbling reaches window-level listeners. */
declare function registerRoot(root: Handle): void;
type PublicOverlayPlane = "floating" | "modal";
/** Current native window root, used by renderer-level facilities like Portal. */
declare function getMountRoot(): Handle;
/** Acquire the shared synthetic host root for one public overlay plane. */
declare function acquireOverlayRoot(plane: PublicOverlayPlane): Handle;
declare function releaseOverlayRoot(plane: PublicOverlayPlane): void;
/** Mount a Solid application into the host-provided root node. */
declare function mount(code: () => JSX$2.Element): () => void;
/**
 * Solid compatibility adapter for a native Wabou event. It walks the Handle
 * tree for bubbling and presents JSX handlers with a small familiar object;
 * this is deliberately not a complete DOM Event implementation.
 */
declare function dispatchEvent(solidId: number, eventCode: number, payloadStr: string, numericData?: ArrayLike<number>): boolean;
//#endregion
export { LayoutSnapshot as $, releaseOverlayRoot as A, createFps as B, insertNode as C, mount as D, mergeProps as E, setTransform2D as F, HostProvider as G, Portal as H, spread as I, defaultHost as J, HostProviderProps as K, writer as L, render as M, runSweep as N, ref as O, setProp as P, LayoutRect as Q, VirtualList as R, insert as S, memo as T, PortalProps as U, useFps as V, Host as W, FrameStats as X, useHost as Y, LayoutNodeMetrics as Z, delegateEvents as _, NativeScrollbarStyle as a, getMountRoot as b, WabouElementProps as c, Writer$1 as d, Fragment as et, acquireOverlayRoot as f, createTextNode as g, createElement as h, HostCapabilities as i, jsxs as it, removeNode as j, registerRoot as k, WabouIntrinsicElements as l, createComponent$1 as m, EVENT_CODE as n, jsx as nt, OP as o, applyRef as p, LayoutTarget as q, Handle as r, jsxDEV as rt, WabouBuiltinIntrinsicElements as s, Dynamic as t, JSX$2 as tt, WabouNodeEvent as u, dispatchEvent as v, isServer as w, getRequestEvent as x, effect as y, VirtualListProps as z };
//# sourceMappingURL=index-BYsBCU2A.d.mts.map