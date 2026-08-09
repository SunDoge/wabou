import {
  dispatchFireAndForget,
  dispatchResourceEffect,
  effectOps,
} from "./effects";

export interface CreateWindowOptions {
  title?: string;
  width?: number;
  height?: number;
  minWidth?: number;
  minHeight?: number;
  resizable?: boolean;
  /** Preserve rendered alpha when the platform compositor supports it. */
  transparent?: boolean;
}

export interface WindowHandle {
  readonly id: number;
  close(): void;
  setMaximized(value: boolean): void;
  setTitle(title: string): void;
}

function handle(id: number): WindowHandle {
  return Object.freeze({
    id,
    close: () => dispatchFireAndForget(effectOps.windowClose, { windowId: id }),
    setMaximized: (value: boolean) =>
      dispatchFireAndForget(effectOps.windowSetMaximized, {
        windowId: id,
        value,
      }),
    setTitle: (title: string) =>
      dispatchFireAndForget(effectOps.windowSetTitle, { windowId: id, title }),
  });
}

/** Create an independent native window running this application's bundle. */
export function createWindow(options: CreateWindowOptions = {}): WindowHandle {
  return handle(dispatchResourceEffect(effectOps.windowCreate, options));
}

/** An imperative handle for the native window that owns this JS runtime. */
export function currentWindow(): WindowHandle {
  return handle(
    (globalThis as typeof globalThis & { __wabou_window_id: number })
      .__wabou_window_id,
  );
}
