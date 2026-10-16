import { Placement, PointAnchor } from "../primitives/positioner.mjs";
import { MenuStateItem, moveMenuHighlight } from "./menu-state.mjs";
import { PopupMotionProps } from "./popover.mjs";
import { Shadow } from "@wabou/core/style";
import { JSX } from "solid-js";
import { Handle } from "@wabou/core/renderer";
//#region src/components/dropdown-menu.d.ts
export interface DropdownMenuItem extends MenuStateItem {
  /** Optional decorative Lucide/static SVG shown in the shared leading slot. */
  icon?: string;
  /** Checked state. Defining this reserves the shared leading status slot. */
  checked?: boolean;
  description?: string;
  /** Human-readable platform shortcut, such as `Ctrl K` or `⌘ K`. */
  shortcut?: string;
  destructive?: boolean;
  separatorBefore?: boolean;
  onSelect?: () => void;
}
export interface DropdownMenuTriggerProps {
  ref(node: Handle): void;
  onClick(event: {
    stopPropagation(): void;
  }): void;
  onKeyDown(event: {
    key: string;
    preventDefault(): void;
    stopPropagation(): void;
  }): void;
  "aria-haspopup": "menu";
  "aria-expanded": boolean;
}
export interface DropdownMenuProps extends PopupMotionProps {
  trigger(props: DropdownMenuTriggerProps): JSX.Element;
  items: readonly DropdownMenuItem[];
  "aria-label": string;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  onAction?: (id: string) => void;
  contentClass?: string;
  contentShadows?: readonly Shadow[] | null;
  placement?: Placement;
  /** Skip returning focus when ownership moves directly to a sibling menu. */
  restoreFocus?: boolean;
  outsidePointerStrategy?: "backdrop" | "passthrough";
  /** Observe or override keys before the menu's vertical navigation runs. */
  onContentKeyDown?: (event: DropdownMenuKeyEvent) => void;
  /** Optional viewport point used by context-menu style triggers. */
  anchorPoint?: () => PointAnchor | undefined;
}
export interface DropdownMenuKeyEvent {
  key: string;
  readonly defaultPrevented?: boolean;
  preventDefault(): void;
}
/** A compact action menu with native focus, typeahead, and overlay routing. */
export declare function DropdownMenu(props: DropdownMenuProps): JSX.Element;
//#endregion
export { moveMenuHighlight };
//# sourceMappingURL=dropdown-menu.d.mts.map