import { createComponent, createContext, type JSX, useContext } from "solid-js";

declare function __wabou_open_url(url: string): boolean;
declare function __wabou_load_font(path: string): boolean;
declare function __wabou_frame_stats(): string;
declare function __wabou_layout_snapshot(ids: Uint32Array): string;
declare const __wabou_capabilities: Record<string, object>;

/** Per-frame timings reported by the native Wabou host. */
export interface FrameStats {
  build_frame_ms: number;
  js_tick_ms: number;
  scene_ms: number;
  present_ms: number;
  node_count: number;
  viewport_w: number;
  viewport_h: number;
}

export interface LayoutRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface LayoutNodeMetrics {
  id: number;
  rect: LayoutRect;
  clip: LayoutRect;
}

export interface LayoutSnapshot {
  revision: number;
  viewport: LayoutRect;
  nodes: LayoutNodeMetrics[];
}

export type LayoutTarget = number | { readonly id: number };

/** Platform capabilities currently implemented by the native host. */
export interface HostCapabilities {}

export interface BuiltinHost {
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
export type Host = BuiltinHost & HostCapabilities;

/** Default adapter around the private Rust/QuickJS ABI. */
const builtinHost: BuiltinHost = {
  system: {
    openUrl: (url) => __wabou_open_url(url),
  },
  fonts: {
    load: (path) => __wabou_load_font(path),
  },
  diagnostics: {
    frameStats: () => {
      const value = JSON.parse(__wabou_frame_stats()) as FrameStats | null;
      return value;
    },
  },
  layout: {
    snapshot: (targets) => {
      const ids = Uint32Array.from(targets, (target) =>
        typeof target === "number" ? target : target.id,
      );
      return JSON.parse(__wabou_layout_snapshot(ids)) as LayoutSnapshot;
    },
    measure: (target) => {
      const snapshot = builtinHost.layout.snapshot([target]);
      return snapshot.nodes[0]?.rect ?? null;
    },
    clippingRect: (target) => {
      const snapshot = builtinHost.layout.snapshot([target]);
      return snapshot.nodes[0]?.clip ?? null;
    },
    viewport: () => builtinHost.layout.snapshot([]).viewport,
  },
};

export const defaultHost: Host = Object.assign(
  builtinHost,
  typeof __wabou_capabilities === "undefined" ? {} : __wabou_capabilities,
) as Host;

const HostContext = createContext<Host>(defaultHost);

export interface HostProviderProps {
  value: Host;
  children?: JSX.Element;
}

/** Bind host capabilities to a Solid subtree (normally one window). */
export function HostProvider(props: HostProviderProps): JSX.Element {
  return createComponent(HostContext.Provider, {
    get value() {
      return props.value;
    },
    get children() {
      return props.children;
    },
  });
}

/** Return the host associated with the current Solid owner/window. */
export function useHost<T extends Host = Host>(): T {
  return useContext(HostContext) as T;
}
