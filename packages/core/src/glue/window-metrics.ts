import { type Accessor, createSignal } from "solid-js";
import { subscribe } from "./host-messages";
import { usePlatformServices } from "./platform-context";
import {
  currentWindow,
  type WindowHandle,
  type WindowKey,
  windowKeyFromJSON,
} from "./window";

export interface WindowMetrics {
  windowId: WindowKey;
  logicalWidth: number;
  logicalHeight: number;
  physicalWidth: number;
  physicalHeight: number;
  scaleFactor: number;
  maximized: boolean;
  focused: boolean;
  colorScheme: "light" | "dark" | null;
}

export interface WindowState extends WindowHandle {
  metrics: Accessor<WindowMetrics>;
  width: Accessor<number>;
  height: Accessor<number>;
  scaleFactor: Accessor<number>;
  maximized: Accessor<boolean>;
  focused: Accessor<boolean>;
  colorScheme: Accessor<"light" | "dark">;
}

/** Inclusive logical-pixel constraints evaluated against the native client area. */
export interface WindowSizeQuery {
  minWidth?: number;
  maxWidth?: number;
  minHeight?: number;
  maxHeight?: number;
}

/**
 * Create a reactive native-window size query without CSS media-query semantics.
 * A zero-sized pre-boot viewport never matches, avoiding a compact-layout flash.
 */
export function createWindowMatch(
  query: WindowSizeQuery,
  window: WindowState = useWindow(),
): Accessor<boolean> {
  const entries = [
    ["minWidth", query.minWidth],
    ["maxWidth", query.maxWidth],
    ["minHeight", query.minHeight],
    ["maxHeight", query.maxHeight],
  ] as const;
  for (const [name, value] of entries) {
    if (value !== undefined && (!Number.isFinite(value) || value < 0)) {
      throw new RangeError(`${name} must be a finite non-negative number`);
    }
  }
  if (
    query.minWidth !== undefined &&
    query.maxWidth !== undefined &&
    query.minWidth > query.maxWidth
  ) {
    throw new RangeError("minWidth cannot exceed maxWidth");
  }
  if (
    query.minHeight !== undefined &&
    query.maxHeight !== undefined &&
    query.minHeight > query.maxHeight
  ) {
    throw new RangeError("minHeight cannot exceed maxHeight");
  }

  return () => {
    const width = window.width();
    const height = window.height();
    if (width <= 0 || height <= 0) return false;
    return (
      (query.minWidth === undefined || width >= query.minWidth) &&
      (query.maxWidth === undefined || width <= query.maxWidth) &&
      (query.minHeight === undefined || height >= query.minHeight) &&
      (query.maxHeight === undefined || height <= query.maxHeight)
    );
  };
}

const initial: WindowMetrics = {
  windowId: windowKeyFromJSON({
    lo:
      (globalThis as typeof globalThis & { __wabou_window_id_lo?: number })
        .__wabou_window_id_lo ?? 1,
    hi:
      (globalThis as typeof globalThis & { __wabou_window_id_hi?: number })
        .__wabou_window_id_hi ?? 1,
  }),
  logicalWidth: 0,
  logicalHeight: 0,
  physicalWidth: 0,
  physicalHeight: 0,
  scaleFactor: 1,
  maximized: false,
  focused: false,
  colorScheme: "light",
};

const [metrics, setMetrics] = createSignal(initial, { equals: false });

subscribe("wabou:window-metrics", (payload) => {
  if (typeof payload !== "string") return;
  const next = JSON.parse(payload) as Omit<WindowMetrics, "windowId"> & {
    windowId: unknown;
  };
  setMetrics({ ...next, windowId: windowKeyFromJSON(next.windowId) });
});

const state: WindowState = {
  get id() {
    return metrics().windowId;
  },
  close: () => currentWindow().close(),
  minimize: () => currentWindow().minimize(),
  setMaximized: (value) => currentWindow().setMaximized(value),
  setTitle: (title) => currentWindow().setTitle(title),
  startDragging: () => currentWindow().startDragging(),
  metrics,
  width: () => metrics().logicalWidth,
  height: () => metrics().logicalHeight,
  scaleFactor: () => metrics().scaleFactor,
  maximized: () => metrics().maximized,
  focused: () => metrics().focused,
  colorScheme: () => metrics().colorScheme ?? "light",
};

/** Reactive state and controls for the native window owning this JS runtime. */
export function useWindow(): WindowState {
  return usePlatformServices().window ?? state;
}
