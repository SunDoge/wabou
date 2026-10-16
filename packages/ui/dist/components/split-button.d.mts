import { ButtonProps } from "./button.mjs";
import { DropdownMenuItem } from "./dropdown-menu.mjs";
import { JSX } from "solid-js";
//#region src/components/split-button.d.ts
export interface SplitButtonProps extends Omit<ButtonProps, "children" | "aria-label"> {
  label: string;
  menuLabel?: string;
  items: readonly DropdownMenuItem[];
  onAction?: (id: string) => void;
  onClick?: ButtonProps["onClick"];
  class?: string;
}
/**
 * Desktop split action: the label always performs the primary command while
 * the adjacent arrow owns alternative commands and their keyboard behavior.
 */
export declare function SplitButton(props: SplitButtonProps): JSX.Element;
//#endregion
//# sourceMappingURL=split-button.d.mts.map