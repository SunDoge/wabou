import { TextProps, ViewProps } from "../primitives/view.mjs";
import { ButtonProps } from "../primitives/button.mjs";
import { ScrollAreaProps } from "../primitives/scroll-area.mjs";
import "../primitives.mjs";
import { SearchFieldProps } from "./search-field.mjs";
import { ComponentsElevation } from "./theme.mjs";
import { JSX } from "solid-js";
//#region src/components/sidebar.d.ts
export interface SidebarSearchGroup<Item> {
  label: string;
  items: readonly Item[];
}
/**
 * Filter grouped sidebar data without taking ownership of routing or identity.
 * Group labels participate in matching so a query can reveal a whole section.
 */
export declare function filterSidebarGroups<Item>(groups: readonly SidebarSearchGroup<Item>[], query: string, searchableText: (item: Item) => string): SidebarSearchGroup<Item>[];
export interface SidebarProps extends Omit<ViewProps, "class"> {
  class?: string;
  /** Native shadow recipe for sidebars that float inside a window frame. */
  elevation?: ComponentsElevation;
}
/** Structural application sidebar. State, routing and width remain explicit. */
export declare function Sidebar(props: SidebarProps): JSX.Element;
export declare function SidebarHeader(props: ViewProps): JSX.Element;
export declare function SidebarSearch(props: SearchFieldProps): JSX.Element;
export interface SidebarContentProps extends ScrollAreaProps {
  contentClass?: string;
}
/** The only scrolling region in a standard sidebar; header/footer stay fixed. */
export declare function SidebarContent(props: SidebarContentProps): JSX.Element;
export declare function SidebarGroup(props: ViewProps): JSX.Element;
export declare function SidebarGroupLabel(props: TextProps): JSX.Element;
export interface SidebarMenuProps extends Omit<ViewProps, "class"> {
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  class?: string;
}
/**
 * Single-selection navigation scope for sidebar destinations.
 * Buttons without a value remain actions and never become selected items.
 */
export declare function SidebarMenu(props: SidebarMenuProps): JSX.Element;
export interface SidebarMenuButtonProps extends Omit<ButtonProps, "class" | "unstyled"> {
  /** Value controlled by the nearest SidebarMenu. Omit for action buttons. */
  value?: string;
  class?: string;
}
/** Consistent navigation row; applications still own activation and routing. */
export declare function SidebarMenuButton(props: SidebarMenuButtonProps): JSX.Element;
/** Fixed icon slot that keeps navigation labels on one shared baseline. */
export declare function SidebarMenuIcon(props: ViewProps): JSX.Element;
/** Truncating label slot for rows that also contain icons or suffix actions. */
export declare function SidebarMenuLabel(props: TextProps): JSX.Element;
/** End-aligned metadata or action slot that never compresses the row label. */
export declare function SidebarMenuSuffix(props: ViewProps): JSX.Element;
export declare function SidebarEmpty(props: {
  title?: string;
  description?: string;
  class?: string;
}): JSX.Element;
export declare function SidebarFooter(props: ViewProps): JSX.Element;
//#endregion
//# sourceMappingURL=sidebar.d.mts.map