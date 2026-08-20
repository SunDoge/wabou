import type { Handle } from "@wabou/core/renderer";
import {
  createComponent,
  createContext,
  type JSX,
  onCleanup,
  useContext,
} from "solid-js";
import { match } from "ts-pattern";
import { createControllableState } from "../primitives/interactions";
import {
  DropdownMenu,
  type DropdownMenuItem,
  type DropdownMenuKeyEvent,
} from "./dropdown-menu";
import { Toolbar, ToolbarButton } from "./toolbar";
import type { PopupMotionProps } from "./popover";

interface MenubarEntry {
  value: string;
  target: Handle;
  disabled: () => boolean;
}

interface MenubarContextValue {
  openValue(): string | null;
  setOpenValue(value: string | null): boolean;
  register(entry: MenubarEntry): () => void;
  moveOpen(value: string, direction: "next" | "previous"): boolean;
}

const MenubarContext = createContext<MenubarContextValue>();

export interface MenubarProps {
  "aria-label": string;
  value?: string | null;
  defaultValue?: string | null;
  onValueChange?: (value: string | null) => void;
  loop?: boolean;
  class?: string;
  children?: JSX.Element;
}

/** Persistent application menus with one tab stop and sibling menu switching. */
export function Menubar(props: MenubarProps): JSX.Element {
  const entries: MenubarEntry[] = [];
  const state = createControllableState<string | null>({
    value: () => props.value,
    defaultValue: props.defaultValue ?? null,
    onChange: props.onValueChange,
  });
  const enabled = () => entries.filter((entry) => !entry.disabled());
  const context: MenubarContextValue = {
    openValue: state.value,
    setOpenValue: state.set,
    register(entry) {
      entries.push(entry);
      return () => {
        const index = entries.indexOf(entry);
        if (index >= 0) entries.splice(index, 1);
      };
    },
    moveOpen(value, direction) {
      const candidates = enabled();
      if (candidates.length === 0) return false;
      const index = candidates.findIndex((entry) => entry.value === value);
      const target =
        direction === "next"
          ? (candidates[index + 1] ??
            (props.loop === false ? undefined : candidates[0]))
          : (candidates[index - 1] ??
            (props.loop === false ? undefined : candidates.at(-1)));
      if (!target) return false;
      state.set(target.value);
      target.target.focus();
      return true;
    },
  };

  return createComponent(MenubarContext, {
    value: context,
    get children() {
      return (
        <Toolbar
          role="menubar"
          aria-label={props["aria-label"]}
          loop={props.loop}
          class={props.class}
        >
          {props.children}
        </Toolbar>
      );
    },
  });
}

export interface MenubarMenuProps extends PopupMotionProps {
  value: string;
  label: string;
  items: readonly DropdownMenuItem[];
  disabled?: boolean;
  onAction?: (id: string) => void;
  children?: JSX.Element;
}

export function MenubarMenu(props: MenubarMenuProps): JSX.Element {
  const menubar = useContext(MenubarContext);
  if (!menubar) throw new Error("MenubarMenu must be used inside Menubar");
  let unregister: (() => void) | undefined;
  let closeOnPointerActivation: boolean | undefined;
  let switchedByHover = false;
  onCleanup(() => unregister?.());

  const handleMenuKey = (event: DropdownMenuKeyEvent) => {
    const direction = match(event.key)
      .with("ArrowRight", () => "next" as const)
      .with("ArrowLeft", () => "previous" as const)
      .otherwise(() => undefined);
    if (!direction || !menubar.moveOpen(props.value, direction)) return;
    event.preventDefault();
  };

  return (
    <DropdownMenu
      aria-label={`${props.label} menu`}
      items={props.items}
      open={menubar.openValue() === props.value}
      onOpenChange={(open) => menubar.setOpenValue(open ? props.value : null)}
      onAction={props.onAction}
      motion={props.motion}
      placement="bottom-start"
      outsidePointerStrategy="passthrough"
      restoreFocus={
        menubar.openValue() === null || menubar.openValue() === props.value
      }
      onContentKeyDown={handleMenuKey}
      trigger={(trigger) => (
        <ToolbarButton
          {...trigger}
          role="menuitem"
          aria-label={props.label}
          disabled={props.disabled}
          ref={(node) => {
            unregister?.();
            unregister = menubar.register({
              value: props.value,
              target: node,
              disabled: () => props.disabled ?? false,
            });
            trigger.ref(node);
          }}
          onFocus={(event) => {
            const keyboard = (event.payload as { focusVisible?: boolean })
              .focusVisible;
            if (keyboard && menubar.openValue() !== null)
              menubar.setOpenValue(props.value);
          }}
          onPointerEnter={() => {
            if (
              menubar.openValue() !== null &&
              menubar.openValue() !== props.value
            ) {
              switchedByHover = true;
              menubar.setOpenValue(props.value);
            }
          }}
          onPointerDown={() => {
            closeOnPointerActivation =
              menubar.openValue() === props.value && !switchedByHover;
            switchedByHover = false;
          }}
          onPointerCancel={() => {
            closeOnPointerActivation = undefined;
          }}
          onClick={(event) => {
            event.stopPropagation();
            const close =
              closeOnPointerActivation ?? menubar.openValue() === props.value;
            closeOnPointerActivation = undefined;
            menubar.setOpenValue(close ? null : props.value);
          }}
        >
          {props.children ?? props.label}
        </ToolbarButton>
      )}
    />
  );
}
