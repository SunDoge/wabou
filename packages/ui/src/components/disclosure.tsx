import type { Handle } from "@wabou/core/renderer";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import { createContext, type JSX, omit, onCleanup, useContext } from "solid-js";
import { createTransition, useReducedMotion } from "../animation";
import {
  Button,
  type ButtonProps,
  CollapsiblePresence,
  Icon,
  rotate2d,
  Text,
  View,
  type ViewProps,
} from "../primitives";
import {
  createControllableState,
  createDisclosure,
  createRovingFocus,
  isSelected,
  toggleSelection,
} from "../primitives/interactions";
import { join } from "./class-names";

function DisclosureIndicator(props: {
  open: () => boolean;
  reducedMotion: () => boolean;
}) {
  const rotation = createTransition(() => (props.open() ? Math.PI : 0), {
    duration: 0.2,
    ease: "easeOut",
    reducedMotion: props.reducedMotion,
  });
  return (
    <View
      class="w-4 h-4 flex-none"
      transform={rotate2d(rotation.value())}
      aria-hidden="true"
    >
      <Icon source={chevronDown} class="text-muted" size={16} />
    </View>
  );
}

interface CollapsibleContextValue {
  open: () => boolean;
  toggle: () => void;
  disabled: () => boolean;
  reducedMotion: () => boolean;
}
const CollapsibleContext = createContext<CollapsibleContextValue>();
const useCollapsible = () => {
  const value = useContext(CollapsibleContext);
  if (!value) throw new Error("Collapsible parts must be inside Collapsible");
  return value;
};

export interface CollapsibleProps
  extends Omit<ViewProps, "children" | "class"> {
  children?: JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  disabled?: boolean;
  reducedMotion?: boolean;
  onOpenChange?: (open: boolean) => void;
  class?: string;
}
export function Collapsible(props: CollapsibleProps) {
  const inheritedReducedMotion = useReducedMotion();
  const state = createDisclosure({
    open: () => props.open,
    defaultOpen: props.defaultOpen,
    disabled: () => props.disabled ?? false,
    onOpenChange: props.onOpenChange,
  });
  const context = {
    open: state.open,
    toggle: state.toggle,
    disabled: state.disabled,
    reducedMotion: () => props.reducedMotion ?? inheritedReducedMotion(),
  };
  const rest = omit(
    props,
    "open",
    "defaultOpen",
    "disabled",
    "reducedMotion",
    "onOpenChange",
    "class",
    "children",
  );
  return (
    <CollapsibleContext value={context}>
      <View {...rest} class={join("flex flex-col", props.class)}>
        {props.children}
      </View>
    </CollapsibleContext>
  );
}
export interface CollapsibleTriggerProps
  extends Omit<ButtonProps, "children" | "class" | "unstyled"> {
  children?: JSX.Element;
  class?: string;
}
export function CollapsibleTrigger(props: CollapsibleTriggerProps) {
  const context = useCollapsible();
  const rest = omit(props, "children", "class", "onClick");
  return (
    <Button
      {...rest}
      unstyled
      disabled={context.disabled() || props.disabled}
      aria-expanded={context.open()}
      class="w-full"
      onClick={(event) => {
        props.onClick?.(event);
        if (!event.defaultPrevented) context.toggle();
      }}
    >
      <View
        class={join(
          "w-full flex items-center justify-between gap-3",
          props.class,
        )}
      >
        {props.children}
        <DisclosureIndicator
          open={context.open}
          reducedMotion={context.reducedMotion}
        />
      </View>
    </Button>
  );
}
export type CollapsibleContentProps = ViewProps;
export function CollapsibleContent(props: CollapsibleContentProps) {
  const context = useCollapsible();
  const contentProps = omit(props, "children", "class", "style");
  return (
    <CollapsiblePresence
      open={context.open()}
      reducedMotion={context.reducedMotion()}
      contentClass={props.class}
      contentProps={contentProps}
      contentStyle={props.style}
    >
      {props.children}
    </CollapsiblePresence>
  );
}

export type AccordionType = "single" | "multiple";
type AccordionValue = string | readonly string[];
export function nextAccordionValue(
  current: AccordionValue,
  type: AccordionType,
  item: string,
  collapsible = false,
): AccordionValue {
  const next = toggleSelection(current, item, type, collapsible);
  return next ?? "";
}
interface AccordionContextValue {
  active: (value: string) => boolean;
  toggle: (value: string) => void;
  disabled: () => boolean;
  reducedMotion: () => boolean;
  register(value: string, node: Handle, disabled: () => boolean): () => void;
  move(value: string, key: string): boolean;
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
export interface AccordionProps extends Omit<ViewProps, "children" | "class"> {
  children?: JSX.Element;
  type?: AccordionType;
  value?: AccordionValue;
  defaultValue?: AccordionValue;
  collapsible?: boolean;
  disabled?: boolean;
  reducedMotion?: boolean;
  onValueChange?: (value: AccordionValue) => void;
  class?: string;
}
export function Accordion(props: AccordionProps) {
  const inheritedReducedMotion = useReducedMotion();
  const type = () => props.type ?? "single";
  const state = createControllableState<AccordionValue>({
    value: () => props.value,
    defaultValue: props.defaultValue ?? (type() === "multiple" ? [] : ""),
    disabled: () => props.disabled ?? false,
    onChange: props.onValueChange,
  });
  const roving = createRovingFocus({ orientation: () => "vertical" });
  const context: AccordionContextValue = {
    active: (item) => isSelected(state.value(), item),
    toggle: (item) => {
      state.set(
        nextAccordionValue(state.value(), type(), item, props.collapsible),
      );
    },
    disabled: () => props.disabled ?? false,
    reducedMotion: () => props.reducedMotion ?? inheritedReducedMotion(),
    register: (value, node, disabled) =>
      roving.register({ id: value, target: node, disabled }),
    move: roving.move,
  };
  const rest = omit(
    props,
    "type",
    "value",
    "defaultValue",
    "collapsible",
    "disabled",
    "reducedMotion",
    "onValueChange",
    "class",
    "children",
  );
  return (
    <AccordionContext value={context}>
      <View {...rest} class={join("flex flex-col", props.class)}>
        {props.children}
      </View>
    </AccordionContext>
  );
}
export interface AccordionItemProps
  extends Omit<ViewProps, "children" | "class"> {
  value: string;
  disabled?: boolean;
  children?: JSX.Element;
  class?: string;
}
export function AccordionItem(props: AccordionItemProps) {
  const rest = omit(props, "value", "disabled", "children", "class");
  return (
    <AccordionItemContext
      value={{ value: props.value, disabled: () => props.disabled ?? false }}
    >
      <View
        {...rest}
        class={join("flex flex-col border-b border-subtle", props.class)}
      >
        {props.children}
      </View>
    </AccordionItemContext>
  );
}
export interface AccordionTriggerProps
  extends Omit<ButtonProps, "children" | "class" | "unstyled"> {
  children?: JSX.Element;
  class?: string;
}
export function AccordionTrigger(props: AccordionTriggerProps) {
  const root = useAccordion();
  const item = useAccordionItem();
  const open = () => root.active(item.value);
  const rest = omit(props, "children", "class", "ref", "onClick", "onKeyDown");
  let unregister: (() => void) | undefined;
  onCleanup(() => unregister?.());
  return (
    <Button
      {...rest}
      unstyled
      disabled={root.disabled() || item.disabled() || props.disabled}
      aria-expanded={open()}
      class="w-full"
      ref={(node) => {
        unregister?.();
        unregister = root.register(
          item.value,
          node,
          () => root.disabled() || item.disabled() || (props.disabled ?? false),
        );
        props.ref?.(node);
      }}
      onClick={(event) => {
        props.onClick?.(event);
        if (!event.defaultPrevented) root.toggle(item.value);
      }}
      onKeyDown={(event) => {
        props.onKeyDown?.(event);
        if (!event.defaultPrevented && root.move(item.value, event.key)) {
          event.preventDefault();
        }
      }}
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
        <DisclosureIndicator open={open} reducedMotion={root.reducedMotion} />
      </View>
    </Button>
  );
}
export type AccordionContentProps = ViewProps;
export function AccordionContent(props: AccordionContentProps) {
  const root = useAccordion();
  const item = useAccordionItem();
  const contentProps = omit(props, "children", "class", "style");
  return (
    <CollapsiblePresence
      open={root.active(item.value)}
      reducedMotion={root.reducedMotion()}
      contentClass={join("pb-4", props.class)}
      contentProps={contentProps}
      contentStyle={props.style}
    >
      {props.children}
    </CollapsiblePresence>
  );
}
