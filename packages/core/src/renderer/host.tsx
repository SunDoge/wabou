import {
  createComponent,
  createContext,
  getOwner,
  type JSX,
  useContext,
} from "solid-js";
import { isNodeKey, type NodeKey } from "../protocol";
import type {
  CalendarDateInfo,
  FrameStats,
  LayoutRect,
  LayoutSnapshot,
  NativeHostApi,
} from "./generated/native-host";
import type { HostCapabilities } from "./index";

export type {
  FrameStats,
  LayoutNodeMetrics,
  LayoutRect,
  LayoutSnapshot,
} from "./generated/native-host";

declare function __wabou_open_url(url: string): boolean;
declare function __wabou_load_font(path: string): boolean;
declare function __wabou_frame_stats(): string;
declare function __wabou_layout_snapshot(ids: Uint32Array): string;
declare function __wabou_system_locale(): string;
declare function __wabou_system_time_zone(): string;
declare function __wabou_system_calendar_date(): string;
declare const __wabou_capabilities: Record<string, object>;

export type LayoutTarget = NodeKey | { readonly id: NodeKey };

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
export type Host = BuiltinHost & HostCapabilities;

/** Checked adapter around the private Rust/QuickJS ABI. */
const nativeHost: NativeHostApi = {
  openUrl: (url) => __wabou_open_url(url),
  loadFont: (path) => __wabou_load_font(path),
  frameStats: () => JSON.parse(__wabou_frame_stats()) as FrameStats | null,
  layoutSnapshot: (ids) =>
    JSON.parse(
      __wabou_layout_snapshot(
        Uint32Array.from(ids.flatMap((id) => [id.lo, id.hi])),
      ),
    ) as LayoutSnapshot,
  systemLocale: () => __wabou_system_locale(),
  systemTimeZone: () => __wabou_system_time_zone(),
  systemCalendarDate: () =>
    JSON.parse(__wabou_system_calendar_date()) as CalendarDateInfo,
};

const builtinHost: BuiltinHost = {
  system: { openUrl: nativeHost.openUrl },
  fonts: { load: nativeHost.loadFont },
  diagnostics: { frameStats: nativeHost.frameStats },
  intl: {
    locale: nativeHost.systemLocale,
    timeZone: nativeHost.systemTimeZone,
    today: nativeHost.systemCalendarDate,
  },
  layout: {
    snapshot: (targets) =>
      nativeHost.layoutSnapshot(
        targets.map((target) => (isNodeKey(target) ? target : target.id)),
      ),
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
) as unknown as Host;

const HostContext = createContext<Host>(defaultHost);

export interface HostProviderProps {
  value: Host;
  children?: JSX.Element;
}

/** Bind host capabilities to a Solid subtree (normally one window). */
export function HostProvider(props: HostProviderProps): JSX.Element {
  return createComponent(HostContext, {
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
  return (getOwner() ? useContext(HostContext) : defaultHost) as T;
}
