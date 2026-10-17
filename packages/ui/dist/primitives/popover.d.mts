import { Easing } from "../animation.mjs";
import { ViewProps, WabouStyle as WabouStyle$1 } from "./view.mjs";
import { OverlayDismissReason, OverlayPlane } from "./overlay-layer.mjs";
import { Placement, PointAnchor } from "./positioner.mjs";
import { Shadow } from "@wabou/core/style";
import { JSX } from "solid-js";
import { Handle } from "@wabou/core/renderer";
//#region src/primitives/popover.d.ts
export interface PopoverTriggerProps {
  ref: (node: Handle) => void;
  onPointerDown: (event: {
    button?: number;
    stopPropagation(): void;
  }) => void;
  onPointerCancel: () => void;
  onClick: (event: {
    stopPropagation(): void;
  }) => void;
  onKeyDown: (event: {
    key: string;
    preventDefault(): void;
    stopPropagation(): void;
  }) => void;
  "aria-haspopup"?: "dialog" | "listbox" | "menu" | "tree" | "grid";
  "aria-expanded": boolean;
}
interface PopoverBaseProps {
  trigger: (props: PopoverTriggerProps) => JSX.Element;
  children?: JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean, reason?: OverlayDismissReason | "trigger") => void;
  placement?: Placement;
  offset?: number;
  /** Positions from a viewport point instead of the trigger's layout box. */
  anchorPoint?: () => PointAnchor | undefined;
  contentClass?: string;
  contentStyle?: WabouStyle$1;
  contentShadows?: readonly Shadow[] | null;
  /** Removes the positioned content subtree from native hit testing. */
  contentInteractionBlocked?: boolean;
  /** Keeps composed hover/focus surfaces open while the pointer is inside. */
  onContentPointerEnter?: ViewProps["onPointerEnter"];
  onContentPointerLeave?: ViewProps["onPointerLeave"];
  onContentFocusIn?: ViewProps["onFocusIn"];
  onContentFocusOut?: ViewProps["onFocusOut"];
  closeOnEscape?: boolean;
  restoreFocus?: boolean;
  /**
   * `passthrough` dismisses from global pointer capture while allowing the
   * underlying target to receive the same gesture. Modal-like surfaces keep
   * the default full-viewport backdrop.
   */
  outsidePointerStrategy?: "backdrop" | "passthrough";
  /** Defaults to the nearest overlay plane, or `floating` at app content. */
  plane?: OverlayPlane;
  /** Set to false to keep presence semantics while disabling visual motion. */
  motion?: false | PopoverMotionOptions;
  /** Open on primary pointer-down; useful for native-feeling selects and menus. */
  openOnPointerDown?: boolean;
}
export interface PopoverMotionOptions {
  duration?: number;
  ease?: Easing;
  /** Initial scale around the panel center. Defaults to 0.98. */
  fromScale?: number;
}
export type PopoverProps = PopoverBaseProps & ({
  /** Required until the native semantic tree resolves aria-labelledby. */
  "aria-label": string;
  contentRole?: "dialog";
  popupRole?: never;
} | {
  /** Flatten the positioned shell when its child owns popup semantics. */
  contentRole: "presentation";
  popupRole: "listbox" | "menu" | "tree" | "grid" | "tooltip";
  "aria-label"?: never;
});
/** A root-layer floating panel positioned from native layout snapshots. */
export declare function Popover(props: PopoverProps): JSX.Element;
//#endregion
export {};
//# sourceMappingURL=popover.d.mts.map