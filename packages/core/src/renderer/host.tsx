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
  DebugOverlayPaintStats,
  FrameStats,
  LayoutRect,
  LayoutSnapshot,
  NativeHostApi,
  NodeKey as NativeNodeKey,
} from "./generated/native-host";
import type { HostModules } from "./index";

export type {
  DebugOverlayPaintStats,
  FrameStats,
  LayoutNodeMetrics,
  LayoutRect,
  LayoutScrollMetrics,
  LayoutSnapshot,
} from "./generated/native-host";

declare function __wabou_open_url(url: string): boolean;
declare function __wabou_load_font(path: string): boolean;
declare function __wabou_frame_stats(): string;
declare function __wabou_set_debug_overlay(
  layout: boolean,
  clips: boolean,
  hitTarget: boolean,
): boolean;
declare function __wabou_debug_overlay_paint_stats(): string;
declare function __wabou_layout_snapshot(
  ids: Uint32Array,
  output: Float64Array | undefined,
): number;
declare function __wabou_system_locale(): string;
declare function __wabou_system_time_zone(): string;
declare function __wabou_system_calendar_date(): string;
declare function __wabou_first_weekday(locale: string): number;
declare function __wabou_format_number(
  locale: string,
  value: number,
  options: string,
): string;
declare function __wabou_parse_number(
  locale: string,
  input: string,
  options: string,
): number;
declare function __wabou_plural_category(
  locale: string,
  value: number,
  options: string,
): string;
declare function __wabou_format_date(
  locale: string,
  epochMillis: number,
  options: string,
): string;
declare function __wabou_format_relative_time(
  locale: string,
  value: number,
  unit: string,
  options: string,
): string;
declare const __wabou_host_modules: Record<string, object>;

export type LayoutTarget = NodeKey | { readonly id: NodeKey };

export interface DebugOverlayOptions {
  /** Draw every retained node's border box. */
  layout?: boolean;
  /** Draw unique effective overflow and scroll clips. */
  clips?: boolean;
  /** Draw the current native pointer hit target. */
  hitTarget?: boolean;
}

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
    firstWeekday(locale: string): number;
    formatNumber(locale: string, value: number, options: string): string;
    parseNumber(locale: string, input: string, options: string): number;
    pluralCategory(locale: string, value: number, options: string): string;
    formatDate(locale: string, epochMillis: number, options: string): string;
    formatRelativeTime(
      locale: string,
      value: number,
      unit: string,
      options: string,
    ): string;
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

/** Augment `HostModules` in generated/user declarations. */
export type Host = BuiltinHost & HostModules;

/** Checked adapter around the private Rust/QuickJS ABI. */
const LAYOUT_SNAPSHOT_VERSION = 1;
const LAYOUT_SNAPSHOT_HEADER_LENGTH = 8;
const LAYOUT_SNAPSHOT_NODE_LENGTH = 14;
let layoutSnapshotBuffer = new Float64Array(0);

function readRect(values: Float64Array, offset: number): LayoutRect {
  return {
    x: values[offset],
    y: values[offset + 1],
    width: values[offset + 2],
    height: values[offset + 3],
  };
}

function readLayoutSnapshot(ids: readonly NativeNodeKey[]): LayoutSnapshot {
  const packedIds = new Uint32Array(ids.length * 2);
  for (let index = 0; index < ids.length; index++) {
    packedIds[index * 2] = ids[index].lo;
    packedIds[index * 2 + 1] = ids[index].hi;
  }
  let required = __wabou_layout_snapshot(packedIds, layoutSnapshotBuffer);
  if (layoutSnapshotBuffer.length < required) {
    layoutSnapshotBuffer = new Float64Array(required);
    required = __wabou_layout_snapshot(packedIds, layoutSnapshotBuffer);
  }
  if (
    required < LAYOUT_SNAPSHOT_HEADER_LENGTH ||
    layoutSnapshotBuffer[0] !== LAYOUT_SNAPSHOT_VERSION
  )
    throw new Error("unsupported native layout snapshot format");
  const nodeCount = layoutSnapshotBuffer[7];
  if (
    !Number.isInteger(nodeCount) ||
    required !==
      LAYOUT_SNAPSHOT_HEADER_LENGTH + nodeCount * LAYOUT_SNAPSHOT_NODE_LENGTH
  )
    throw new Error("invalid native layout snapshot length");

  const nodes = [] as LayoutSnapshot["nodes"];
  for (let index = 0; index < nodeCount; index++) {
    const offset =
      LAYOUT_SNAPSHOT_HEADER_LENGTH + index * LAYOUT_SNAPSHOT_NODE_LENGTH;
    nodes.push({
      id: {
        lo: layoutSnapshotBuffer[offset],
        hi: layoutSnapshotBuffer[offset + 1],
      },
      rect: readRect(layoutSnapshotBuffer, offset + 2),
      clip: readRect(layoutSnapshotBuffer, offset + 6),
      scroll: {
        offsetX: layoutSnapshotBuffer[offset + 10],
        offsetY: layoutSnapshotBuffer[offset + 11],
        rangeX: layoutSnapshotBuffer[offset + 12],
        rangeY: layoutSnapshotBuffer[offset + 13],
      },
    });
  }
  return {
    revision: layoutSnapshotBuffer[1] + layoutSnapshotBuffer[2] * 0x1_0000_0000,
    viewport: readRect(layoutSnapshotBuffer, 3),
    nodes,
  };
}

const nativeHost: NativeHostApi = {
  openUrl: (url) => __wabou_open_url(url),
  loadFont: (path) => __wabou_load_font(path),
  frameStats: () => JSON.parse(__wabou_frame_stats()) as FrameStats | null,
  setDebugOverlay: (layout, clips, hitTarget) =>
    __wabou_set_debug_overlay(layout, clips, hitTarget),
  debugOverlayPaintStats: () =>
    JSON.parse(
      __wabou_debug_overlay_paint_stats(),
    ) as DebugOverlayPaintStats | null,
  layoutSnapshot: readLayoutSnapshot,
  systemLocale: () => __wabou_system_locale(),
  systemTimeZone: () => __wabou_system_time_zone(),
  systemCalendarDate: () =>
    JSON.parse(__wabou_system_calendar_date()) as CalendarDateInfo,
  firstWeekday: (locale) =>
    typeof __wabou_first_weekday === "function"
      ? __wabou_first_weekday(locale)
      : new Intl.Locale(locale).getWeekInfo().firstDay % 7,
  formatNumber: (locale, value, options) =>
    typeof __wabou_format_number === "function"
      ? __wabou_format_number(locale, value, options)
      : new Intl.NumberFormat(locale, JSON.parse(options)).format(value),
  parseNumber: (locale, input, options) => {
    if (typeof __wabou_parse_number === "function")
      return __wabou_parse_number(locale, input, options);
    const formatter = new Intl.NumberFormat(locale, JSON.parse(options));
    const parts = formatter.formatToParts(12345.6);
    const group = parts.find((part) => part.type === "group")?.value ?? ",";
    const decimal = parts.find((part) => part.type === "decimal")?.value ?? ".";
    const normalized = input
      .replaceAll(group, "")
      .replace(decimal, ".")
      .replace(/[^0-9+.-]/g, "");
    return Number(normalized);
  },
  pluralCategory: (locale, value, options) =>
    typeof __wabou_plural_category === "function"
      ? __wabou_plural_category(locale, value, options)
      : new Intl.PluralRules(locale, JSON.parse(options)).select(value),
  formatDate: (locale, epochMillis, options) =>
    typeof __wabou_format_date === "function"
      ? __wabou_format_date(locale, epochMillis, options)
      : new Intl.DateTimeFormat(locale, JSON.parse(options)).format(
          epochMillis,
        ),
  formatRelativeTime: (locale, value, unit, options) =>
    typeof __wabou_format_relative_time === "function"
      ? __wabou_format_relative_time(locale, value, unit, options)
      : new Intl.RelativeTimeFormat(locale, JSON.parse(options)).format(
          value,
          unit as Intl.RelativeTimeFormatUnit,
        ),
};

const builtinHost: BuiltinHost = {
  system: { openUrl: nativeHost.openUrl },
  fonts: { load: nativeHost.loadFont },
  diagnostics: {
    frameStats: nativeHost.frameStats,
    overlayPaintStats: nativeHost.debugOverlayPaintStats,
    setOverlay: (options) =>
      nativeHost.setDebugOverlay(
        options.layout ?? false,
        options.clips ?? false,
        options.hitTarget ?? false,
      ),
  },
  intl: {
    locale: nativeHost.systemLocale,
    timeZone: nativeHost.systemTimeZone,
    today: nativeHost.systemCalendarDate,
    firstWeekday: nativeHost.firstWeekday,
    formatNumber: nativeHost.formatNumber,
    parseNumber: nativeHost.parseNumber,
    pluralCategory: nativeHost.pluralCategory,
    formatDate: nativeHost.formatDate,
    formatRelativeTime: nativeHost.formatRelativeTime,
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
  typeof __wabou_host_modules === "undefined" ? {} : __wabou_host_modules,
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
