import { Placement } from "../primitives/positioner.mjs";
import "../primitives.mjs";
import { PopupMotionProps } from "./popover.mjs";
import { DelayedOpenController, DelayedOpenOptions, createDelayedOpenController } from "./delayed-open.mjs";
import { Shadow } from "@wabou/core/style";
import { JSX } from "solid-js";
import { Handle } from "@wabou/core/renderer";
//#region src/components/hover-card.d.ts
export interface HoverCardTriggerProps {
  ref(node: Handle): void;
  onPointerEnter(): void;
  onPointerLeave(): void;
  onFocus(): void;
  onBlur(): void;
  onKeyDown(event: {
    key: string;
    preventDefault(): void;
    stopPropagation(): void;
  }): void;
  "aria-haspopup": "dialog";
  "aria-expanded": boolean;
}
export interface HoverCardProps extends PopupMotionProps {
  "aria-label": string;
  trigger(props: HoverCardTriggerProps): JSX.Element;
  children?: JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?(open: boolean): void;
  openDelay?: number;
  closeDelay?: number;
  placement?: Placement;
  offset?: number;
  contentClass?: string;
  contentShadows?: readonly Shadow[] | null;
  disabled?: boolean;
}
/** A preview surface that tolerates pointer travel between trigger and card. */
export declare function HoverCard(props: HoverCardProps): JSX.Element;
//#endregion
export { type DelayedOpenController, type DelayedOpenOptions, createDelayedOpenController };
//# sourceMappingURL=hover-card.d.mts.map