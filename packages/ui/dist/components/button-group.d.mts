import { TextProps, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { ButtonGroupButtonSize, ButtonGroupButtonVariant, ButtonGroupOrientation } from "./button-group-context.mjs";
import { JSX } from "solid-js";
//#region src/components/button-group.d.ts
export interface ButtonGroupProps extends Omit<ViewProps, "class"> {
  orientation?: ButtonGroupOrientation;
  size?: ButtonGroupButtonSize;
  variant?: ButtonGroupButtonVariant;
  disabled?: boolean;
  class?: string;
}
/** A single bordered control surface composed from ordinary Wabou buttons. */
export declare function ButtonGroup(props: ButtonGroupProps): JSX.Element;
export declare function ButtonGroupText(props: TextProps): JSX.Element;
export interface ButtonGroupSeparatorProps {
  orientation?: "horizontal" | "vertical";
  class?: string;
}
export declare function ButtonGroupSeparator(props: ButtonGroupSeparatorProps): JSX.Element;
//#endregion
//# sourceMappingURL=button-group.d.mts.map