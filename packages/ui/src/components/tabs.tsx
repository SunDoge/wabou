import type { Handle } from "@wabou/core/renderer";
import { mergeClasses } from "@wabou/core/style";
import x from "lucide-static/icons/x.svg?raw";
import {
  createComponent,
  createContext,
  type JSX,
  onCleanup,
  Show,
  useContext,
} from "solid-js";
import { match } from "ts-pattern";
import {
  type ButtonState,
  Button as HeadlessButton,
  Icon,
  Text,
  View,
} from "../primitives";
import {
  createControllableState,
  createRovingFocus,
} from "../primitives/interactions";
import { Button } from "./button";

const orientationClass = (
  orientation: "horizontal" | "vertical",
  horizontal: string,
  vertical: string,
) =>
  match(orientation)
    .with("horizontal", () => horizontal)
    .with("vertical", () => vertical)
    .exhaustive();

interface TabsContextValue {
  value: () => string | undefined;
  orientation: () => "horizontal" | "vertical";
  select(value: string): void;
  register(value: string, node: Handle, disabled: () => boolean): () => void;
  move(value: string, key: string): boolean;
}

const TabsContext = createContext<TabsContextValue>();

export interface TabsProps {
  value?: string;
  defaultValue?: string;
  orientation?: "horizontal" | "vertical";
  class?: string;
  children?: JSX.Element;
  onValueChange?: (value: string) => void;
}

export function Tabs(props: TabsProps): JSX.Element {
  const state = createControllableState<string | undefined>({
    value: () => props.value,
    defaultValue: props.defaultValue,
    onChange: (value) => value !== undefined && props.onValueChange?.(value),
  });
  const value = state.value;
  const select = (next: string) => {
    state.set(next);
  };
  const roving = createRovingFocus({
    orientation: () => props.orientation ?? "horizontal",
    onMove: select,
  });
  const context: TabsContextValue = {
    value,
    orientation: () => props.orientation ?? "horizontal",
    select,
    register: (next, node, disabled) => {
      const unregister = roving.register({ id: next, target: node, disabled });
      if (value() === undefined) select(next);
      return unregister;
    },
    move: roving.move,
  };
  return createComponent(TabsContext, {
    value: context,
    get children() {
      return (
        <View
          class={mergeClasses(
            "flex gap-3",
            orientationClass(context.orientation(), "flex-col", "flex-row"),
            props.class,
          )}
        >
          {props.children}
        </View>
      );
    },
  });
}

export function TabsList(props: {
  variant?: "default" | "line";
  /** Keep tab semantics and roving focus while leaving layout and paint to the caller. */
  unstyled?: boolean;
  "aria-label"?: string;
  class?: string;
  children?: JSX.Element;
}): JSX.Element {
  const context = useContext(TabsContext);
  if (!context) throw new Error("TabsList must be used inside Tabs");
  return (
    <View
      role="tablist"
      aria-label={props["aria-label"]}
      aria-orientation={context.orientation()}
      class={
        props.unstyled
          ? props.class
          : mergeClasses(
              "flex-none flex items-center gap-1",
              orientationClass(context.orientation(), "flex-row", "flex-col"),
              match(props.variant ?? "default")
                .with("default", () => "p-0.5 rounded-lg bg-control")
                .with("line", () => "bg-transparent")
                .exhaustive(),
              props.class,
            )
      }
    >
      {props.children}
    </View>
  );
}

export interface TabsTriggerProps {
  value: string;
  disabled?: boolean;
  /** Keep tab behavior and semantics without applying the component skin. */
  unstyled?: boolean;
  "aria-label"?: string;
  class?: string | ((state: ButtonState) => string);
  children?: JSX.Element;
}

export interface TabsItemState {
  selected: boolean;
}

export interface TabsItemProps {
  value: string;
  disabled?: boolean;
  closeLabel?: string;
  onClose?: () => void;
  class?: string | ((state: TabsItemState) => string);
  triggerClass?: string | ((state: ButtonState) => string);
  children?: JSX.Element;
}

/**
 * Bounded, optionally closeable tab chrome. The tab trigger and close action
 * remain sibling hit targets so closing a tab never selects it first.
 */
export function TabsItem(props: TabsItemProps): JSX.Element {
  const context = useContext(TabsContext);
  if (!context) throw new Error("TabsItem must be used inside Tabs");
  const selected = () => context.value() === props.value;
  const itemClass = () =>
    typeof props.class === "function"
      ? props.class({ selected: selected() })
      : props.class;
  return (
    <View
      class={mergeClasses(
        "h-8 min-w-24 max-w-56 flex flex-row items-center overflow-hidden rounded-md border border-transparent",
        selected()
          ? "bg-surface text-primary shadow-xs"
          : "bg-transparent text-muted",
        itemClass(),
      )}
    >
      <TabsTrigger
        unstyled
        value={props.value}
        disabled={props.disabled}
        class={(state) =>
          mergeClasses(
            "h-full min-w-0 flex-1 px-2 flex flex-row items-center gap-2",
            !selected() && state.hovered && "bg-control-hover text-primary",
            state.focusVisible && "border border-focus",
            typeof props.triggerClass === "function"
              ? props.triggerClass(state)
              : props.triggerClass,
          )
        }
      >
        {props.children}
      </TabsTrigger>
      {props.onClose ? (
        <Button
          variant="ghost"
          size="icon"
          aria-label={props.closeLabel ?? `Close ${props.value}`}
          class="w-6 h-6 flex-none text-muted"
          style={{ padding: 0 }}
          onClick={(event) => {
            event.stopPropagation();
            props.onClose?.();
          }}
        >
          <Icon source={x} size={12} />
        </Button>
      ) : null}
    </View>
  );
}

export function TabsTrigger(props: TabsTriggerProps): JSX.Element {
  const context = useContext(TabsContext);
  if (!context) throw new Error("TabsTrigger must be used inside Tabs");
  const selected = () => context.value() === props.value;
  let unregister: (() => void) | undefined;
  onCleanup(() => unregister?.());
  return (
    <HeadlessButton
      unstyled
      role="tab"
      aria-label={props["aria-label"]}
      disabled={props.disabled}
      selected={selected()}
      aria-selected={selected()}
      ref={(node) => {
        unregister?.();
        unregister = context.register(
          props.value,
          node,
          () => props.disabled ?? false,
        );
      }}
      class={(state) =>
        props.unstyled
          ? typeof props.class === "function"
            ? props.class(state)
            : (props.class ?? "")
          : mergeClasses(
              "h-7 px-3 items-center justify-center rounded-md border border-transparent text-sm font-medium",
              match({ selected: selected(), hovered: state.hovered })
                .with(
                  { selected: true },
                  () => "bg-surface text-primary shadow-xs",
                )
                .with({ hovered: true }, () => "bg-control-hover text-primary")
                .otherwise(() => "bg-transparent text-muted"),
              state.focusVisible && "border-focus",
              typeof props.class === "function"
                ? props.class(state)
                : props.class,
            )
      }
      style={(state) => ({ opacity: state.disabled ? 0.45 : 1 })}
      onClick={() => context.select(props.value)}
      onKeyDown={(event) => {
        if (context.move(props.value, event.key)) event.preventDefault();
      }}
    >
      {props.unstyled ? (
        props.children
      ) : (
        <Text class="text-sm font-medium">{props.children}</Text>
      )}
    </HeadlessButton>
  );
}

export function TabsContent(props: {
  value: string;
  /** Keep stateful/native content mounted while hiding inactive panels. */
  keepMounted?: boolean;
  class?: string;
  children?: JSX.Element;
}): JSX.Element {
  const context = useContext(TabsContext);
  if (!context) throw new Error("TabsContent must be used inside Tabs");
  const selected = () => context.value() === props.value;
  return (
    <Show when={props.keepMounted || selected()}>
      <View
        role="tabpanel"
        aria-hidden={!selected()}
        class={mergeClasses(
          "w-full min-w-0 flex-none flex flex-col",
          props.class,
        )}
        style={{ display: selected() ? "flex" : "none" }}
      >
        {props.children}
      </View>
    </Show>
  );
}
