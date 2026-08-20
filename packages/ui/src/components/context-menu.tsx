import type { Handle, WabouPointerEvent } from "@wabou/core/renderer";
import type { Shadow } from "@wabou/core/style";
import { createSignal, type JSX } from "solid-js";
import { DropdownMenu, type DropdownMenuItem } from "./dropdown-menu";

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

export interface ContextMenuProps {
  trigger(props: ContextMenuTriggerProps): JSX.Element;
  items: readonly DropdownMenuItem[];
  "aria-label": string;
  onAction?: (id: string) => void;
  contentClass?: string;
  contentShadows?: readonly Shadow[] | null;
}

/** An action menu anchored to the native secondary-click coordinate. */
export function ContextMenu(props: ContextMenuProps): JSX.Element {
  const [open, setOpen] = createSignal(false);
  const [point, setPoint] = createSignal<{ x: number; y: number }>();

  return (
    <DropdownMenu
      aria-label={props["aria-label"]}
      items={props.items}
      open={open()}
      onOpenChange={setOpen}
      onAction={props.onAction}
      anchorPoint={point}
      contentClass={props.contentClass}
      contentShadows={props.contentShadows}
      trigger={(menu) =>
        props.trigger({
          ref: menu.ref,
          onContextMenu: (event) => {
            event.preventDefault();
            event.stopPropagation();
            setPoint({ x: event.clientX, y: event.clientY });
            setOpen(true);
          },
          onKeyDown: (event) => {
            if (event.key === "ContextMenu" || event.key === "F10") {
              event.preventDefault();
              event.stopPropagation();
              setPoint(undefined);
              setOpen(true);
              return;
            }
            menu.onKeyDown(event);
          },
          "aria-haspopup": "menu",
          get "aria-expanded"() {
            return open();
          },
        })
      }
    />
  );
}
