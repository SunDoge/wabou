import { ButtonProps } from "./button.mjs";
import { JSX } from "solid-js";
//#region src/components/toolbar.d.ts
export type ToolbarOrientation = "horizontal" | "vertical";
export interface ToolbarProps {
  "aria-label": string;
  /** Semantic role used by composite controls built on the toolbar primitive. */
  role?: "toolbar" | "menubar";
  orientation?: ToolbarOrientation;
  loop?: boolean;
  class?: string;
  children?: JSX.Element;
}
/** A compact command surface with one native tab stop and arrow navigation. */
export declare function Toolbar(props: ToolbarProps): JSX.Element;
export interface ToolbarButtonProps extends Omit<ButtonProps, "focusOrder"> {}
export declare function ToolbarButton(props: ToolbarButtonProps): JSX.Element;
export interface ToolbarToggleProps extends Omit<ToolbarButtonProps, "aria-pressed" | "onClick" | "variant"> {
  pressed?: boolean;
  defaultPressed?: boolean;
  onPressedChange?(pressed: boolean): void;
}
export declare function ToolbarToggle(props: ToolbarToggleProps): JSX.Element;
export declare function ToolbarGroup(props: {
  "aria-label"?: string;
  class?: string;
  children?: JSX.Element;
}): JSX.Element;
export declare function ToolbarSeparator(props: {
  class?: string;
}): JSX.Element;
//#endregion
//# sourceMappingURL=toolbar.d.mts.map