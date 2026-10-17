import { TextProps, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/item.d.ts
export type ItemVariant = "default" | "outline" | "muted";
export type ItemSize = "default" | "sm";
export type ItemMediaVariant = "default" | "icon" | "image";
export declare function itemClass(variant?: ItemVariant, size?: ItemSize, className?: string): string;
export interface ItemProps extends Omit<ViewProps, "class"> {
  variant?: ItemVariant;
  size?: ItemSize;
  /** Persistent list selection, independent from transient pointer hover. */
  selected?: boolean;
  /** Disable the complete row subtree, including trailing actions. */
  disabled?: boolean;
  class?: string;
}
/** A composable list row based on shadcn's Item anatomy. */
export declare function Item(props: ItemProps): JSX.Element;
export declare function ItemGroup(props: ViewProps): JSX.Element;
export declare function ItemSeparator(props: {
  class?: string;
}): JSX.Element;
export declare function itemMediaClass(variant?: ItemMediaVariant, className?: string): string;
export declare function ItemMedia(props: {
  children?: JSX.Element;
  variant?: ItemMediaVariant;
  class?: string;
}): JSX.Element;
export declare function ItemContent(props: ViewProps): JSX.Element;
export declare function ItemTitle(props: TextProps): JSX.Element;
export declare function ItemDescription(props: TextProps): JSX.Element;
export declare function ItemActions(props: ViewProps): JSX.Element;
export declare function ItemHeader(props: ViewProps): JSX.Element;
export declare const ItemFooter: typeof ItemHeader;
//#endregion
//# sourceMappingURL=item.d.mts.map