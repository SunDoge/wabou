import { TextProps, ViewProps } from "../primitives/view.mjs";
import { ModalOpenChangeReason, ModalProps } from "../primitives/modal.mjs";
import "../primitives.mjs";
import { ButtonProps } from "./button.mjs";
import { JSX } from "solid-js";
//#region src/components/drawer.d.ts
export type DrawerDirection = "top" | "right" | "bottom" | "left";
export type DrawerOpenChangeReason = ModalOpenChangeReason | "drag";
export declare function drawerDragOffset(direction: DrawerDirection, rawDelta: number): number;
export declare function drawerShouldDismiss(offset: number, size: number, threshold: number): boolean;
export interface DrawerProps extends Omit<ModalProps, "defaultOpen" | "onOpenChange" | "contentClass" | "contentTransform" | "motion"> {
  defaultOpen?: boolean;
  onOpenChange?(open: boolean, reason: DrawerOpenChangeReason): void;
  direction?: DrawerDirection;
  dismissible?: boolean;
  dismissThreshold?: number;
  contentClass?: string;
}
/** A focus-isolated edge drawer with a captured native drag-to-dismiss gesture. */
export declare function Drawer(props: DrawerProps): JSX.Element;
export declare function DrawerHandle(props: ViewProps): JSX.Element;
export declare function DrawerHeader(props: ViewProps): JSX.Element;
export declare function DrawerFooter(props: ViewProps): JSX.Element;
export declare function DrawerTitle(props: TextProps): JSX.Element;
export declare function DrawerDescription(props: TextProps): JSX.Element;
export declare function DrawerClose(props: ButtonProps): JSX.Element;
//#endregion
//# sourceMappingURL=drawer.d.mts.map