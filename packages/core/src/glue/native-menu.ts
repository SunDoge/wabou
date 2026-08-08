import { dispatchEffect, effectOps } from "./effects";
import { currentWindow } from "./window";

export type NativeMenuItem =
  | {
      kind: "item";
      id: string;
      label: string;
      enabled?: boolean;
      checked?: boolean;
    }
  | { kind: "separator" }
  | { kind: "submenu"; label: string; items: NativeMenuItem[] };

export interface NativeMenuPosition {
  readonly x: number;
  readonly y: number;
}

export interface NativeMenuOptions {
  readonly windowId?: number;
  readonly position?: NativeMenuPosition;
  readonly items: readonly NativeMenuItem[];
}

/** Show a platform context menu and resolve with the selected item id. */
export function showNativeMenu(options: NativeMenuOptions): Promise<string> {
  return dispatchEffect<string>(effectOps.contextMenuShow, {
    windowId: options.windowId ?? currentWindow().id,
    position: options.position,
    items: options.items,
  });
}
