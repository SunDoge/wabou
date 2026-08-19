import type { Handle } from "@wabou/core/renderer";
import { Button as HeadlessButton, Text, View } from "../primitives";
import {
  createControllableState,
  createRovingFocus,
} from "../primitives/interactions";
import {
  createComponent,
  createContext,
  type JSX,
  onCleanup,
  useContext,
} from "solid-js";
import { match } from "ts-pattern";
import { join } from "./class-names";

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
          class={join(
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
      class={join(
        "flex-none flex items-center gap-1",
        orientationClass(context.orientation(), "flex-row", "flex-col"),
        match(props.variant ?? "default")
          .with("default", () => "p-0.5 rounded-md bg-control")
          .with("line", () => "bg-transparent")
          .exhaustive(),
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export interface TabsTriggerProps {
  value: string;
  disabled?: boolean;
  class?: string;
  children?: JSX.Element;
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
        join(
          "h-7 px-3 items-center justify-center rounded-sm border border-transparent text-sm font-medium",
          match({ selected: selected(), hovered: state.hovered })
            .with({ selected: true }, () => "bg-surface text-primary shadow-xs")
            .with({ hovered: true }, () => "bg-control-hover text-primary")
            .otherwise(() => "bg-transparent text-muted"),
          state.focusVisible && "border-focus",
          props.class,
        )
      }
      style={(state) => ({ opacity: state.disabled ? 0.45 : 1 })}
      onClick={() => context.select(props.value)}
      onKeyDown={(event) => {
        if (context.move(props.value, event.key)) event.preventDefault();
      }}
    >
      <Text class="text-sm font-medium">{props.children}</Text>
    </HeadlessButton>
  );
}

export function TabsContent(props: {
  value: string;
  class?: string;
  children?: JSX.Element;
}): JSX.Element {
  const context = useContext(TabsContext);
  if (!context) throw new Error("TabsContent must be used inside Tabs");
  return context.value() === props.value ? (
    <View role="tabpanel" class={join("flex-1", props.class)}>
      {props.children}
    </View>
  ) : null;
}
