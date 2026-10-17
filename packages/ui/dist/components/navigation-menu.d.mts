import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { ButtonProps } from "./button.mjs";
import { JSX } from "solid-js";
//#region src/components/navigation-menu.d.ts
export interface NavigationMenuProps {
  "aria-label": string;
  value?: string | null;
  defaultValue?: string | null;
  onValueChange?: (value: string | null) => void;
  loop?: boolean;
  class?: string;
  viewportClass?: string;
  children?: JSX.Element;
}
export declare function NavigationMenu(props: NavigationMenuProps): JSX.Element;
export declare function NavigationMenuList(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
export interface NavigationMenuItemProps {
  value: string;
  disabled?: boolean;
  class?: string;
  children?: JSX.Element;
}
export declare function NavigationMenuItem(props: NavigationMenuItemProps): JSX.Element;
export declare function navigationMenuTriggerClass(open: boolean, className?: string): string;
export interface NavigationMenuTriggerProps extends Omit<ButtonProps, "variant"> {}
export declare function NavigationMenuTrigger(props: NavigationMenuTriggerProps): JSX.Element;
export declare function NavigationMenuContent(props: ViewProps): JSX.Element;
export interface NavigationMenuLinkProps extends ButtonProps {
  active?: boolean;
  closeOnSelect?: boolean;
}
export declare function NavigationMenuLink(props: NavigationMenuLinkProps): JSX.Element;
export declare function NavigationMenuIndicator(props: ViewProps): JSX.Element;
/** The shared Popover content already is the native viewport. */
export declare function NavigationMenuViewport(): JSX.Element;
//#endregion
//# sourceMappingURL=navigation-menu.d.mts.map