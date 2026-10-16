import { Placement } from "../primitives/positioner.mjs";
import "../primitives.mjs";
import { PopupMotionProps } from "./popover.mjs";
import { createDelayedOpenController } from "./delayed-open.mjs";
import { JSX } from "solid-js";
import { Handle } from "@wabou/core/renderer";
//#region src/components/tooltip.d.ts
export interface TooltipTriggerProps {
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
}
export interface TooltipProps extends PopupMotionProps {
  trigger(props: TooltipTriggerProps): JSX.Element;
  children?: JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?(open: boolean): void;
  openDelay?: number;
  closeDelay?: number;
  placement?: Placement;
  offset?: number;
  contentClass?: string;
  disabled?: boolean;
  /** Optional keyboard shortcut presented as a native keycap. */
  shortcut?: string;
}
export interface TooltipContentProps {
  children?: JSX.Element;
  shortcut?: string;
  id?: string;
  class?: string;
}
/** Visual content shared by managed and explicitly composed tooltips. */
export declare function TooltipContent(props: TooltipContentProps): JSX.Element;
/** A delayed, non-interactive label for pointer and keyboard focus targets. */
export declare function Tooltip(props: TooltipProps): JSX.Element;
//#endregion
export { createDelayedOpenController as createTooltipDelayController };
//# sourceMappingURL=tooltip.d.mts.map