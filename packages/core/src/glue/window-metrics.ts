import { createSignal, type Accessor } from "solid-js";
import { subscribe } from "./host-messages";
import { currentWindow, type WindowHandle } from "./window";
import { usePlatformServices } from "./platform-context";

export interface WindowMetrics {
  windowId: number;
  logicalWidth: number;
  logicalHeight: number;
  physicalWidth: number;
  physicalHeight: number;
  scaleFactor: number;
  maximized: boolean;
  focused: boolean;
}

export interface WindowState extends WindowHandle {
  metrics: Accessor<WindowMetrics>;
  width: Accessor<number>;
  height: Accessor<number>;
  scaleFactor: Accessor<number>;
  maximized: Accessor<boolean>;
  focused: Accessor<boolean>;
}

const initial: WindowMetrics = {
  windowId:
    (globalThis as typeof globalThis & { __wabou_window_id?: number })
      .__wabou_window_id ?? 0,
  logicalWidth: 0,
  logicalHeight: 0,
  physicalWidth: 0,
  physicalHeight: 0,
  scaleFactor: 1,
  maximized: false,
  focused: false,
};

const [metrics, setMetrics] = createSignal(initial, { equals: false });

subscribe("wabou:window-metrics", (payload) => {
  if (typeof payload !== "string") return;
  const next = JSON.parse(payload) as WindowMetrics;
  setMetrics(next);
});

const state: WindowState = {
  get id() {
    return metrics().windowId;
  },
  close: () => currentWindow().close(),
  setMaximized: (value) => currentWindow().setMaximized(value),
  setTitle: (title) => currentWindow().setTitle(title),
  metrics,
  width: () => metrics().logicalWidth,
  height: () => metrics().logicalHeight,
  scaleFactor: () => metrics().scaleFactor,
  maximized: () => metrics().maximized,
  focused: () => metrics().focused,
};

/** Reactive state and controls for the native window owning this JS runtime. */
export function useWindow(): WindowState {
  return usePlatformServices().window ?? state;
}
