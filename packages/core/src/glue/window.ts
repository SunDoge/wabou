import {
  dispatchEffect,
  dispatchFireAndForget,
  effectOps,
} from "./effects";
import {
  createResourceKeyFamily,
  type ResourceKey,
} from "../protocol/resource-key";

const windowKeys = createResourceKeyFamily("window");
export type WindowKey = ResourceKey<"window">;

export function windowKeyFromJSON(value: unknown): WindowKey {
  return windowKeys.fromJSON(value);
}

export interface CreateWindowOptions {
  title?: string;
  width?: number;
  height?: number;
  minWidth?: number;
  minHeight?: number;
  resizable?: boolean;
  /** Show native window-manager borders and title bar. */
  decorations?: boolean;
  /** Preserve rendered alpha when the platform compositor supports it. */
  transparent?: boolean;
}

export interface WindowHandle {
  readonly id: WindowKey;
  close(): void;
  minimize(): void;
  setMaximized(value: boolean): void;
  setTitle(title: string): void;
  /** Begin a compositor-managed move operation for a custom title bar. */
  startDragging(): void;
  /** Restore this window if it was hidden or released to the tray. */
  show(): void;
}

function handle(id: WindowKey): WindowHandle {
  return Object.freeze({
    id,
    close: () => dispatchFireAndForget(effectOps.windowClose, { windowId: id }),
    minimize: () =>
      dispatchFireAndForget(effectOps.windowMinimize, { windowId: id }),
    setMaximized: (value: boolean) =>
      dispatchFireAndForget(effectOps.windowSetMaximized, {
        windowId: id,
        value,
      }),
    setTitle: (title: string) =>
      dispatchFireAndForget(effectOps.windowSetTitle, { windowId: id, title }),
    startDragging: () =>
      dispatchFireAndForget(effectOps.windowStartDragging, { windowId: id }),
    show: () => dispatchFireAndForget(effectOps.windowShow, { windowId: id }),
  });
}

/** Create an independent native window running this application's bundle. */
export function createWindow(
  options: CreateWindowOptions = {},
): Promise<WindowHandle> {
  return dispatchEffect<unknown>(effectOps.windowCreate, options).then((key) =>
    handle(windowKeyFromJSON(key)),
  );
}

/** An imperative handle for the native window that owns this JS runtime. */
export function currentWindow(): WindowHandle {
  return handle(windowKeys.fromParts(__wabou_window_id_lo, __wabou_window_id_hi));
}
