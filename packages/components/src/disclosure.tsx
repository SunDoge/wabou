import { Button, Text, View } from "@wabou/primitives";
import {
  createContext,
  type JSX,
  Show,
  splitProps,
  useContext,
} from "solid-js";
import { match } from "ts-pattern";
import { createControllableState } from "./state";

const join = (...values: Array<string | undefined | false>) =>
  values.filter(Boolean).join(" ");

interface CollapsibleContextValue {
  open: () => boolean;
  toggle: () => void;
  disabled: () => boolean;
}
const CollapsibleContext = createContext<CollapsibleContextValue>();
const useCollapsible = () => {
  const value = useContext(CollapsibleContext);
  if (!value) throw new Error("Collapsible parts must be inside Collapsible");
  return value;
};

export interface CollapsibleProps {
  children?: JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  disabled?: boolean;
  onOpenChange?: (open: boolean) => void;
  class?: string;
}
export function Collapsible(props: CollapsibleProps) {
  const state = createControllableState({
    value: () => props.open,
    defaultValue: props.defaultOpen ?? false,
    disabled: () => props.disabled ?? false,
    onChange: props.onOpenChange,
  });
  const context = {
    open: state.value,
    toggle: () => {
      state.set(!state.value());
    },
    disabled: () => props.disabled ?? false,
  };
  return (
    <CollapsibleContext.Provider value={context}>
      <View class={join("flex flex-col", props.class)}>{props.children}</View>
    </CollapsibleContext.Provider>
  );
}
export function CollapsibleTrigger(props: {
  children?: JSX.Element;
  class?: string;
}) {
  const context = useCollapsible();
  return (
    <Button
      unstyled
      disabled={context.disabled()}
      aria-expanded={context.open()}
      class="w-full"
      onClick={context.toggle}
    >
      <View
        class={join(
          "w-full flex items-center justify-between gap-3",
          props.class,
        )}
      >
        {props.children}
        <Text class="flex-none text-muted">{context.open() ? "−" : "+"}</Text>
      </View>
    </Button>
  );
}
export function CollapsibleContent(props: {
  children?: JSX.Element;
  class?: string;
}) {
  const context = useCollapsible();
  return (
    <Show when={context.open()}>
      <View class={props.class}>{props.children}</View>
    </Show>
  );
}

export type AccordionType = "single" | "multiple";
type AccordionValue = string | string[];
export function nextAccordionValue(
  current: AccordionValue,
  type: AccordionType,
  item: string,
  collapsible = false,
): AccordionValue {
  return match(type)
    .with("single", () => (current === item && collapsible ? "" : item))
    .with("multiple", () => {
      const values = Array.isArray(current) ? current : [];
      return values.includes(item)
        ? values.filter((value) => value !== item)
        : [...values, item];
    })
    .exhaustive();
}
interface AccordionContextValue {
  active: (value: string) => boolean;
  toggle: (value: string) => void;
  disabled: () => boolean;
}
const AccordionContext = createContext<AccordionContextValue>();
const AccordionItemContext = createContext<{
  value: string;
  disabled: () => boolean;
}>();
const useAccordion = () => {
  const value = useContext(AccordionContext);
  if (!value) throw new Error("Accordion parts must be inside Accordion");
  return value;
};
const useAccordionItem = () => {
  const value = useContext(AccordionItemContext);
  if (!value) throw new Error("Accordion parts must be inside AccordionItem");
  return value;
};
export interface AccordionProps {
  children?: JSX.Element;
  type?: AccordionType;
  value?: AccordionValue;
  defaultValue?: AccordionValue;
  collapsible?: boolean;
  disabled?: boolean;
  onValueChange?: (value: AccordionValue) => void;
  class?: string;
}
export function Accordion(props: AccordionProps) {
  const type = () => props.type ?? "single";
  const state = createControllableState<AccordionValue>({
    value: () => props.value,
    defaultValue: props.defaultValue ?? (type() === "multiple" ? [] : ""),
    disabled: () => props.disabled ?? false,
    onChange: props.onValueChange,
  });
  const context: AccordionContextValue = {
    active: (item) =>
      Array.isArray(state.value())
        ? state.value().includes(item)
        : state.value() === item,
    toggle: (item) => {
      state.set(
        nextAccordionValue(state.value(), type(), item, props.collapsible),
      );
    },
    disabled: () => props.disabled ?? false,
  };
  return (
    <AccordionContext.Provider value={context}>
      <View class={join("flex flex-col", props.class)}>{props.children}</View>
    </AccordionContext.Provider>
  );
}
export function AccordionItem(props: {
  value: string;
  disabled?: boolean;
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <AccordionItemContext.Provider
      value={{ value: props.value, disabled: () => props.disabled ?? false }}
    >
      <View class={join("flex flex-col border-b border-subtle", props.class)}>
        {props.children}
      </View>
    </AccordionItemContext.Provider>
  );
}
export function AccordionTrigger(props: {
  children?: JSX.Element;
  class?: string;
}) {
  const root = useAccordion();
  const item = useAccordionItem();
  const open = () => root.active(item.value);
  return (
    <Button
      unstyled
      disabled={root.disabled() || item.disabled()}
      aria-expanded={open()}
      class="w-full"
      onClick={() => root.toggle(item.value)}
    >
      <View
        class={join(
          "w-full py-4 flex items-center justify-between gap-4",
          props.class,
        )}
      >
        <Text class="min-w-0 whitespace-normal text-sm font-medium text-primary">
          {props.children}
        </Text>
        <Text class="flex-none text-muted">{open() ? "⌃" : "⌄"}</Text>
      </View>
    </Button>
  );
}
export function AccordionContent(props: {
  children?: JSX.Element;
  class?: string;
}) {
  const root = useAccordion();
  const item = useAccordionItem();
  return (
    <Show when={root.active(item.value)}>
      <View class={join("pb-4", props.class)}>{props.children}</View>
    </Show>
  );
}
