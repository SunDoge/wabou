import { PopupMotionProps } from "./popover.mjs";
import { DropdownMenuItem } from "./dropdown-menu.mjs";
import { Shadow } from "@wabou/core/style";
import { JSX } from "solid-js";
import { Handle, WabouPointerEvent } from "@wabou/core/renderer";
//#region src/components/context-menu.d.ts
export interface ContextMenuTriggerProps {
  ref(node: Handle): void;
  onContextMenu(event: WabouPointerEvent): void;
  onKeyDown(event: {
    key: string;
    preventDefault(): void;
    stopPropagation(): void;
  }): void;
  "aria-haspopup": "menu";
  "aria-expanded": boolean;
}
export interface ContextMenuProps extends PopupMotionProps {
  trigger(props: ContextMenuTriggerProps): JSX.Element;
  items: readonly DropdownMenuItem[];
  "aria-label": string;
  onAction?: (id: string) => void;
  contentClass?: string;
  contentShadows?: readonly Shadow[] | null;
}
/** An action menu anchored to the native secondary-click coordinate. */
export declare function ContextMenu(props: ContextMenuProps): JSX.Element;
//#endregion
//# sourceMappingURL=context-menu.d.mts.map