import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import { createContext, type JSX, useContext } from "solid-js";
import { createTransition, useReducedMotion } from "../animation";
import {
  Button,
  CollapsiblePresence,
  Icon,
  rotate2d,
  Text,
  View,
} from "../primitives";
import {
  createControllableState,
  createDisclosure,
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

export interface CollapsibleProps {
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
  return (
    <CollapsibleContext value={context}>
      <View class={join("flex flex-col", props.class)}>{props.children}</View>
    </CollapsibleContext>
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
        <DisclosureIndicator
          open={context.open}
          reducedMotion={context.reducedMotion}
        />
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
    <CollapsiblePresence
      open={context.open()}
      reducedMotion={context.reducedMotion()}
      contentClass={props.class}
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
  const context: AccordionContextValue = {
    active: (item) => isSelected(state.value(), item),
    toggle: (item) => {
      state.set(
        nextAccordionValue(state.value(), type(), item, props.collapsible),
      );
    },
    disabled: () => props.disabled ?? false,
    reducedMotion: () => props.reducedMotion ?? inheritedReducedMotion(),
  };
  return (
    <AccordionContext value={context}>
      <View class={join("flex flex-col", props.class)}>{props.children}</View>
    </AccordionContext>
  );
}
export function AccordionItem(props: {
  value: string;
  disabled?: boolean;
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <AccordionItemContext
      value={{ value: props.value, disabled: () => props.disabled ?? false }}
    >
      <View class={join("flex flex-col border-b border-subtle", props.class)}>
        {props.children}
      </View>
    </AccordionItemContext>
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
        <DisclosureIndicator open={open} reducedMotion={root.reducedMotion} />
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
    <CollapsiblePresence
      open={root.active(item.value)}
      reducedMotion={root.reducedMotion()}
      contentClass={join("pb-4", props.class)}
    >
      {props.children}
    </CollapsiblePresence>
  );
}
