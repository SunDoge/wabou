import type { Handle } from "@wabou/core/renderer";
import type { Shadow } from "@wabou/core/style";
import { mergeClasses } from "@wabou/core/style";
import {
  createEffect,
  createSignal,
  For as ForValue,
  type JSX,
} from "solid-js";
import { match } from "ts-pattern";
import { Popover, Text, View } from "../primitives";
import { createTypeahead } from "../primitives/interactions";
import type { Placement, PointAnchor } from "../primitives/positioner";
import { type MenuStateItem, moveMenuHighlight } from "./menu-state";
import type { PopupMotionProps } from "./popover";
import { componentsElevation, useComponentsTheme } from "./theme";

export interface DropdownMenuItem extends MenuStateItem {
  description?: string;
  destructive?: boolean;
  separatorBefore?: boolean;
  onSelect?: () => void;
}

export interface DropdownMenuTriggerProps {
  ref(node: Handle): void;
  onClick(event: { stopPropagation(): void }): void;
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
export function DropdownMenu(props: DropdownMenuProps): JSX.Element {
  const theme = useComponentsTheme();
  const [uncontrolledOpen, setUncontrolledOpen] = createSignal(
    props.defaultOpen ?? false,
  );
  const [highlighted, setHighlighted] = createSignal<string>();
  const open = () => props.open ?? uncontrolledOpen();
  const typeahead = createTypeahead<MenuStateItem>();
  let trigger: Handle | undefined;
  let content: Handle | undefined;
  let openEdge: "first" | "last" = "first";
  let wasOpen = false;

  const setOpen = (next: boolean, edge: "first" | "last" = "first") => {
    openEdge = edge;
    if (props.open === undefined) setUncontrolledOpen(next);
    props.onOpenChange?.(next);
  };
  createEffect(
    () => ({
      open: open(),
      items: props.items,
      restoreFocus: props.restoreFocus ?? true,
    }),
    ({ open: isOpen, items, restoreFocus }) => {
      if (isOpen && !wasOpen) {
        setHighlighted(moveMenuHighlight(items, undefined, openEdge));
        requestAnimationFrame(() => content?.focus());
      } else if (!isOpen && wasOpen) {
        setHighlighted(undefined);
        typeahead.reset();
        if (restoreFocus) requestAnimationFrame(() => trigger?.focus());
      }
      wasOpen = isOpen;
    },
  );
  const select = (id: string | undefined) => {
    const item = props.items.find((candidate) => candidate.id === id);
    if (!item || item.disabled) return false;
    item.onSelect?.();
    props.onAction?.(item.id);
    setOpen(false);
    return true;
  };
  const move = (direction: "first" | "last" | "next" | "previous") => {
    const next = moveMenuHighlight(props.items, highlighted(), direction);
    if (next === undefined) return false;
    setHighlighted(next);
    return true;
  };
  const handleMenuKey = (event: DropdownMenuKeyEvent) => {
    props.onContentKeyDown?.(event);
    if (event.defaultPrevented) return;
    const handled = match(event.key)
      .with("ArrowDown", () => move("next"))
      .with("ArrowUp", () => move("previous"))
      .with("Home", () => move("first"))
      .with("End", () => move("last"))
      .with("Enter", () => select(highlighted()))
      .with(" ", () => select(highlighted()))
      .with("Escape", () => {
        setOpen(false);
        return true;
      })
      .otherwise((key) => {
        const item = typeahead.search(props.items, key, highlighted());
        if (item) setHighlighted(item.id);
        return item !== undefined;
      });
    if (handled) event.preventDefault();
  };

  return (
    <Popover
      contentRole="presentation"
      popupRole="menu"
      open={open()}
      onOpenChange={(next) => setOpen(next)}
      placement={props.placement ?? "bottom-end"}
      restoreFocus={props.restoreFocus}
      outsidePointerStrategy={props.outsidePointerStrategy}
      anchorPoint={props.anchorPoint}
      contentClass={mergeClasses(
        "w-56 p-1 flex flex-col gap-0.5 rounded-xl border border-subtle bg-surface",
        props.contentClass,
      )}
      contentShadows={
        props.contentShadows === undefined
          ? componentsElevation(theme(), "floating")
          : props.contentShadows
      }
      motion={props.motion}
      trigger={(popover) =>
        props.trigger({
          ref: (node) => {
            trigger = node;
            popover.ref(node);
          },
          onClick: popover.onClick,
          onKeyDown: (event) => {
            popover.onKeyDown(event);
            const handled = match(event.key)
              .with("ArrowDown", () => {
                setOpen(true, "first");
                return true;
              })
              .with("ArrowUp", () => {
                setOpen(true, "last");
                return true;
              })
              .otherwise(() => false);
            if (handled) event.preventDefault();
          },
          "aria-haspopup": "menu",
          get "aria-expanded"() {
            return open();
          },
        })
      }
    >
      <View
        ref={(node) => (content = node)}
        role="menu"
        aria-label={props["aria-label"]}
        focusOrder={0}
        class="min-w-0 flex flex-col gap-0.5"
        onKeyDown={handleMenuKey}
      >
        <ForValue each={props.items} keyed={false}>
          {(item) => (
            <>
              {item().separatorBefore && (
                <View
                  role="presentation"
                  class="h-px flex-none my-1 bg-control"
                />
              )}
              <View
                role="menuitem"
                aria-label={item().label}
                aria-disabled={item().disabled}
                class={mergeClasses(
                  "w-full min-h-8 flex-none px-2 py-1.5 flex flex-col justify-center rounded-lg",
                  highlighted() === item().id
                    ? "bg-control-hover"
                    : "bg-transparent",
                  item().destructive ? "text-danger-primary" : "text-primary",
                )}
                style={{ opacity: item().disabled ? 0.45 : 1 }}
                onPointerMove={() =>
                  !item().disabled && setHighlighted(item().id)
                }
                onClick={() => select(item().id)}
              >
                <Text class="text-sm">{item().label}</Text>
                {item().description && (
                  <Text class="text-xs text-muted">{item().description}</Text>
                )}
              </View>
            </>
          )}
        </ForValue>
      </View>
    </Popover>
  );
}

export { moveMenuHighlight } from "./menu-state";
