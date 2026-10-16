import { PopupMotionProps } from "./popover.mjs";
import { DropdownMenuItem } from "./dropdown-menu.mjs";
import { JSX } from "solid-js";
//#region src/components/menubar.d.ts
export interface MenubarProps {
  "aria-label": string;
  value?: string | null;
  defaultValue?: string | null;
  onValueChange?: (value: string | null) => void;
  loop?: boolean;
  class?: string;
  children?: JSX.Element;
}
/** Persistent application menus with one tab stop and sibling menu switching. */
export declare function Menubar(props: MenubarProps): JSX.Element;
export interface MenubarMenuProps extends PopupMotionProps {
  value: string;
  label: string;
  items: readonly DropdownMenuItem[];
  disabled?: boolean;
  onAction?: (id: string) => void;
  children?: JSX.Element;
}
export declare function MenubarMenu(props: MenubarMenuProps): JSX.Element;
//#endregion
//# sourceMappingURL=menubar.d.mts.map