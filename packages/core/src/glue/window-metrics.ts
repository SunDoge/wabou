import { type Accessor, createMemo, createSignal } from "solid-js";
import { subscribeJson } from "./host-messages";
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
  outerX: number | null;
  outerY: number | null;
  occluded: boolean;
  colorScheme: "light" | "dark" | null;
}

export interface WindowState extends WindowHandle {
  metrics: Accessor<WindowMetrics>;
  width: Accessor<number>;
  height: Accessor<number>;
  scaleFactor: Accessor<number>;
  maximized: Accessor<boolean>;
  focused: Accessor<boolean>;
  outerX: Accessor<number | null>;
  outerY: Accessor<number | null>;
  occluded: Accessor<boolean>;
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

  return createMemo(
    () => {
      const width = window.width();
      const height = window.height();
      if (width <= 0 || height <= 0) return false;
      return (
        (query.minWidth === undefined || width >= query.minWidth) &&
        (query.maxWidth === undefined || width <= query.maxWidth) &&
        (query.minHeight === undefined || height >= query.minHeight) &&
        (query.maxHeight === undefined || height <= query.maxHeight)
      );
    },
    { sync: true },
  );
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
  outerX: null,
  outerY: null,
  occluded: false,
  colorScheme: "light",
};

// Native window events can arrive while an unrelated component, behavior-test
// action, or event handler is the current Solid owner. The metrics store is a
// process-level host signal, so those writes are intentional cross-owner writes.
function sameMetrics(previous: WindowMetrics, next: WindowMetrics): boolean {
  return (
    previous.windowId.lo === next.windowId.lo &&
    previous.windowId.hi === next.windowId.hi &&
    previous.logicalWidth === next.logicalWidth &&
    previous.logicalHeight === next.logicalHeight &&
    previous.physicalWidth === next.physicalWidth &&
    previous.physicalHeight === next.physicalHeight &&
    previous.scaleFactor === next.scaleFactor &&
    previous.maximized === next.maximized &&
    previous.focused === next.focused &&
    previous.outerX === next.outerX &&
    previous.outerY === next.outerY &&
    previous.occluded === next.occluded &&
    previous.colorScheme === next.colorScheme
  );
}

const [metrics, setMetrics] = createSignal(initial, {
  // Headless hosts and some platforms can publish an unchanged metrics
  // snapshot on consecutive frames. Do not repeatedly invalidate the entire
  // responsive tree for an object that is only referentially new.
  equals: sameMetrics,
  ownedWrite: true,
});

function decodeWindowMetrics(value: unknown): WindowMetrics {
  if (typeof value !== "object" || value === null)
    throw new TypeError("window metrics must be an object");
  const next = value as Record<string, unknown>;
  const finiteNumber = (field: string): number => {
    const number = next[field];
    if (typeof number !== "number" || !Number.isFinite(number))
      throw new TypeError(`window metrics ${field} must be a finite number`);
    return number;
  };
  if (
    typeof next.maximized !== "boolean" ||
    typeof next.focused !== "boolean" ||
    typeof next.occluded !== "boolean"
  )
    throw new TypeError("window metrics flags must be booleans");
  for (const field of ["outerX", "outerY"] as const) {
    if (
      next[field] !== null &&
      (typeof next[field] !== "number" || !Number.isFinite(next[field]))
    )
      throw new TypeError(`window metrics ${field} must be null or a finite number`);
  }
  if (
    next.colorScheme !== null &&
    next.colorScheme !== "light" &&
    next.colorScheme !== "dark"
  )
    throw new TypeError("window metrics colorScheme is invalid");
  return {
    windowId: windowKeyFromJSON(next.windowId),
    logicalWidth: finiteNumber("logicalWidth"),
    logicalHeight: finiteNumber("logicalHeight"),
    physicalWidth: finiteNumber("physicalWidth"),
    physicalHeight: finiteNumber("physicalHeight"),
    scaleFactor: finiteNumber("scaleFactor"),
    maximized: next.maximized,
    focused: next.focused,
    outerX: next.outerX as number | null,
    outerY: next.outerY as number | null,
    occluded: next.occluded,
    colorScheme: next.colorScheme,
  };
}

subscribeJson("wabou:window-metrics", setMetrics, {
  decode: decodeWindowMetrics,
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
  show: () => currentWindow().show(),
  metrics,
  width: () => metrics().logicalWidth,
  height: () => metrics().logicalHeight,
  scaleFactor: () => metrics().scaleFactor,
  maximized: () => metrics().maximized,
  focused: () => metrics().focused,
  outerX: () => metrics().outerX,
  outerY: () => metrics().outerY,
  occluded: () => metrics().occluded,
  colorScheme: () => metrics().colorScheme ?? "light",
};

/** Reactive state and controls for the native window owning this JS runtime. */
export function useWindow(): WindowState {
  return usePlatformServices().window ?? state;
}
