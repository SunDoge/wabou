import type { Handle } from "@wabou/core/renderer";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import {
  createComponent,
  createContext,
  type JSX,
  omit,
  onCleanup,
  useContext,
} from "solid-js";
import { Icon, rotate2d, View, type ViewProps } from "../primitives";
import { createControllableState } from "../primitives/interactions";
import { Button, type ButtonProps } from "./button";
import { mergeClasses } from "@wabou/core/style";
import { Popover } from "./popover";

interface NavigationEntry {
  value: string;
  target: Handle;
  disabled(): boolean;
}

interface NavigationMenuContextValue {
  openValue(): string | null;
  setOpenValue(value: string | null): boolean;
  registerTrigger(entry: NavigationEntry): () => void;
  registerContent(value: string, render: () => JSX.Element): () => void;
  content(): JSX.Element;
  move(value: string, direction: "next" | "previous"): boolean;
}

interface NavigationItemContextValue {
  value: string;
  disabled(): boolean;
}

const NavigationMenuContext = createContext<NavigationMenuContextValue>();
const NavigationItemContext = createContext<NavigationItemContextValue>();

function requireNavigationMenu(): NavigationMenuContextValue {
  const context = useContext(NavigationMenuContext);
  if (!context) throw new Error("NavigationMenu child requires a root");
  return context;
}

function requireNavigationItem(): NavigationItemContextValue {
  const context = useContext(NavigationItemContext);
  if (!context) throw new Error("NavigationMenu child requires an item");
  return context;
}

export interface NavigationMenuProps {
  "aria-label": string;
  value?: string | null;
  defaultValue?: string | null;
  onValueChange?: (value: string | null) => void;
  loop?: boolean;
  class?: string;
  viewportClass?: string;
  children?: JSX.Element;
}

export function NavigationMenu(props: NavigationMenuProps): JSX.Element {
  const entries: NavigationEntry[] = [];
  const contents = new Map<string, () => JSX.Element>();
  const state = createControllableState<string | null>({
    value: () => props.value,
    defaultValue: props.defaultValue ?? null,
    onChange: props.onValueChange,
  });
  const enabled = () => entries.filter((entry) => !entry.disabled());
  const context: NavigationMenuContextValue = {
    openValue: state.value,
    setOpenValue: state.set,
    registerTrigger(entry) {
      entries.push(entry);
      return () => {
        const index = entries.indexOf(entry);
        if (index >= 0) entries.splice(index, 1);
      };
    },
    registerContent(value, render) {
      contents.set(value, render);
      return () => {
        if (contents.get(value) !== render) return;
        contents.delete(value);
      };
    },
    content() {
      const value = state.value();
      return value ? contents.get(value)?.() : undefined;
    },
    move(value, direction) {
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
      target.target.focus();
      if (state.value() !== null) state.set(target.value);
      return true;
    },
  };

  return createComponent(NavigationMenuContext, {
    value: context,
    get children() {
      return (
        <Popover
          aria-label={props["aria-label"]}
          open={state.value() !== null}
          onOpenChange={(open) => {
            if (!open) state.set(null);
          }}
          placement="bottom-start"
          outsidePointerStrategy="passthrough"
          contentClass={mergeClasses(
            "w-[520px] max-w-full min-w-0 overflow-hidden rounded-lg border border-subtle bg-surface p-2 shadow-md",
            props.viewportClass,
          )}
          trigger={(trigger) => (
            <View
              ref={trigger.ref}
              role="group"
              aria-label={props["aria-label"]}
              class={mergeClasses(
                "relative inline-flex flex-none items-center justify-center",
                props.class,
              )}
            >
              {props.children}
            </View>
          )}
        >
          {context.content()}
        </Popover>
      );
    },
  });
}

export function NavigationMenuList(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element {
  return (
    <View
      role="menubar"
      class={mergeClasses(
        "flex flex-none items-center justify-center gap-1",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export interface NavigationMenuItemProps {
  value: string;
  disabled?: boolean;
  class?: string;
  children?: JSX.Element;
}

export function NavigationMenuItem(
  props: NavigationMenuItemProps,
): JSX.Element {
  const context: NavigationItemContextValue = {
    value: props.value,
    disabled: () => props.disabled ?? false,
  };
  return (
    <NavigationItemContext value={context}>
      <View class={mergeClasses("relative flex-none", props.class)}>
        {props.children}
      </View>
    </NavigationItemContext>
  );
}

export function navigationMenuTriggerClass(
  open: boolean,
  className?: string,
): string {
  return mergeClasses(
    "h-9 px-3 inline-flex flex-none items-center justify-center gap-1 rounded-md border-transparent text-sm font-medium",
    open ? "bg-selected text-primary" : "bg-transparent text-secondary",
    className,
  );
}

export interface NavigationMenuTriggerProps
  extends Omit<ButtonProps, "variant"> {}

export function NavigationMenuTrigger(
  props: NavigationMenuTriggerProps,
): JSX.Element {
  const menu = requireNavigationMenu();
  const item = requireNavigationItem();
  let unregister: (() => void) | undefined;
  onCleanup(() => unregister?.());
  const open = () => menu.openValue() === item.value;
  const forwarded = omit(props, "class", "children", "ref", "onClick");
  return (
    <Button
      {...forwarded}
      role="menuitem"
      aria-haspopup="dialog"
      aria-expanded={open()}
      disabled={props.disabled ?? item.disabled()}
      ref={(node) => {
        unregister?.();
        unregister = menu.registerTrigger({
          value: item.value,
          target: node,
          disabled: () => props.disabled ?? item.disabled(),
        });
        props.ref?.(node);
      }}
      variant="ghost"
      class={navigationMenuTriggerClass(open(), props.class)}
      onPointerEnter={() => {
        if (menu.openValue() !== null) menu.setOpenValue(item.value);
      }}
      onKeyDown={(event) => {
        if (event.key === "ArrowDown") {
          menu.setOpenValue(item.value);
          event.preventDefault();
        } else if (event.key === "ArrowRight" || event.key === "ArrowLeft") {
          menu.move(
            item.value,
            event.key === "ArrowRight" ? "next" : "previous",
          );
          event.preventDefault();
        }
        props.onKeyDown?.(event);
      }}
      onClick={(event) => {
        menu.setOpenValue(open() ? null : item.value);
        props.onClick?.(event);
      }}
    >
      {props.children}
      <Icon
        aria-hidden="true"
        source={chevronDown}
        size={14}
        class="text-muted"
        transform={open() ? rotate2d(Math.PI) : undefined}
      />
    </Button>
  );
}

export function NavigationMenuContent(props: ViewProps): JSX.Element {
  const menu = requireNavigationMenu();
  const item = requireNavigationItem();
  const forwarded = omit(props, "class", "children");
  const render = () => (
    <View
      {...forwarded}
      role="group"
      class={mergeClasses("w-full min-w-0 flex flex-col gap-1", props.class)}
    >
      {props.children}
    </View>
  );
  const unregister = menu.registerContent(item.value, render);
  onCleanup(() => unregister?.());
  return undefined as unknown as JSX.Element;
}

export interface NavigationMenuLinkProps extends ButtonProps {
  active?: boolean;
  closeOnSelect?: boolean;
}

export function NavigationMenuLink(
  props: NavigationMenuLinkProps,
): JSX.Element {
  const menu = requireNavigationMenu();
  const forwarded = omit(props, "active", "closeOnSelect", "class", "onClick");
  return (
    <Button
      {...forwarded}
      role="link"
      variant="ghost"
      class={mergeClasses(
        "w-full h-auto min-w-0 flex flex-col items-start gap-1 rounded-md p-2 text-left",
        props.active && "bg-selected",
        props.class,
      )}
      onClick={(event) => {
        if (props.closeOnSelect ?? true) menu.setOpenValue(null);
        props.onClick?.(event);
      }}
    >
      <View class="w-full min-w-0 flex flex-col items-start gap-1">
        {props.children}
      </View>
    </Button>
  );
}

export function NavigationMenuIndicator(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      aria-hidden="true"
      class={mergeClasses(
        "absolute left-1/2 bottom-0 w-2 h-0.5 rounded-full bg-accent",
        props.class,
      )}
    />
  );
}

/** The shared Popover content already is the native viewport. */
export function NavigationMenuViewport(): JSX.Element {
  return undefined as unknown as JSX.Element;
}
