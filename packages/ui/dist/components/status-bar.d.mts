import { TextProps, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { ComponentTone } from "./tone.mjs";
import { JSX } from "solid-js";
//#region src/components/status-bar.d.ts
export interface StatusBarProps extends Omit<ViewProps, "class"> {
  class?: string;
}
/** Persistent low-emphasis application state at the bottom of a desktop window. */
export declare function StatusBar(props: StatusBarProps): JSX.Element;
export interface StatusBarItemProps extends Omit<TextProps, "class"> {
  class?: string;
  grow?: boolean;
}
export declare function StatusBarItem(props: StatusBarItemProps): JSX.Element;
export declare function StatusBarSeparator(props: {
  class?: string;
}): JSX.Element;
export interface StatusBarGroupProps extends Omit<ViewProps, "class"> {
  class?: string;
  grow?: boolean;
  shrink?: boolean;
}
/** A shrink-safe status bar group for icons, indicators and related text. */
export declare function StatusBarGroup(props: StatusBarGroupProps): JSX.Element;
export type StatusBarIndicatorTone = Exclude<ComponentTone, "neutral"> | "muted";
export interface StatusBarIndicatorProps {
  tone?: StatusBarIndicatorTone;
  class?: string;
}
/** Compact, decorative state indicator with a theme-aware semantic tone. */
export declare function StatusBarIndicator(props: StatusBarIndicatorProps): JSX.Element;
//#endregion
//# sourceMappingURL=status-bar.d.mts.map