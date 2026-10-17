import { ViewProps, WabouClassList, WabouStyle as WabouStyle$1 } from "../primitives/view.mjs";
import "../primitives.mjs";
import { Shadow } from "@wabou/core/style";
import { JSX } from "solid-js";
//#region src/components/title-bar.d.ts
export interface WindowFrameProps extends ViewProps {
  class?: string;
  children?: JSX.Element;
  /** Clip the application surface to desktop-style corners when restored. */
  rounded?: boolean;
  /** Let the native window material remain visible beneath a translucent client surface. */
  material?: "solid" | "translucent";
}
export declare function windowFrameMaterialStyle(theme: "light" | "dark", material?: "solid" | "translucent"): WabouStyle$1;
export declare function windowFrameBackdropClassList(maximized: boolean, rounded?: boolean): WabouClassList;
export declare function windowFrameClientClassList(maximized: boolean, rounded?: boolean, classList?: WabouClassList): WabouClassList;
/** Two restrained client-decoration layers sized to fit the 12px backdrop. */
export declare function windowFrameShadows(theme: "light" | "dark"): Shadow[];
/**
 * Root frame for an application-owned title bar and window chrome.
 *
 * Rounded outer corners require the native window to preserve alpha and the
 * Rust host to clear with a transparent base color. Maximized windows are
 * intentionally square so their content reaches every display edge.
 */
export declare function WindowFrame(props: WindowFrameProps): JSX.Element;
export declare const titleBarClass = "border-b border-subtle";
export interface TitleBarProps extends ViewProps {
  class?: string;
  children?: JSX.Element;
}
export declare const titleBarLayoutStyle: {
  readonly display: "flex";
  readonly "flex-direction": "row";
  readonly "align-items": "center";
  readonly "flex-shrink": 0;
  readonly height: "40px";
};
export declare const titleBarDragRegionLayoutStyle: {
  readonly display: "flex";
  readonly "flex-direction": "row";
  readonly "align-items": "center";
  readonly "flex-grow": 1;
  readonly "flex-shrink": 1;
  readonly "flex-basis": "0%";
  readonly height: "100%";
};
/** Layout shell for an application-owned title bar. */
export declare function TitleBar(props: TitleBarProps): JSX.Element;
export interface TitleBarDragRegionProps extends Omit<ViewProps, "onPointerDown" | "onDblClick"> {
  class?: string;
  style?: WabouStyle$1;
  children?: JSX.Element;
}
/** Explicit non-interactive region that moves the native window. */
export declare function TitleBarDragRegion(props: TitleBarDragRegionProps): JSX.Element;
//#endregion
//# sourceMappingURL=title-bar.d.mts.map