import { mergeClasses } from "@wabou/core/style";
import sparkles from "lucide-static/icons/sparkles.svg?raw";
import { type JSX, omit } from "solid-js";
import { Icon, Pulse, Text } from "../primitives";
import {
  Collapsible,
  CollapsibleContent,
  type CollapsibleContentProps,
  type CollapsibleProps,
  CollapsibleTrigger,
  type CollapsibleTriggerProps,
} from "./disclosure";

export interface ReasoningProps extends CollapsibleProps {}

/** Quiet disclosure surface for model reasoning or intermediate thought. */
export function Reasoning(props: ReasoningProps): JSX.Element {
  const rest = omit(props, "class");
  return (
    <Collapsible
      {...rest}
      class={mergeClasses(
        "w-full min-w-0 overflow-hidden rounded-lg border border-subtle bg-surface-muted",
        props.class,
      )}
    />
  );
}

export interface ReasoningTriggerProps
  extends Omit<CollapsibleTriggerProps, "children" | "class"> {
  label?: string;
  streaming?: boolean;
  class?: string;
}

export function ReasoningTrigger(props: ReasoningTriggerProps): JSX.Element {
  const label = () =>
    props.label ?? (props.streaming ? "Thinking" : "Reasoning");
  const rest = omit(props, "label", "streaming", "class", "aria-label");
  return (
    <CollapsibleTrigger
      {...rest}
      aria-label={props["aria-label"] ?? label()}
      class={mergeClasses("min-h-9 px-3 py-1.5 text-left", props.class)}
    >
      <Icon source={sparkles} size={13} class="flex-none text-muted" />
      <Text class="min-w-0 flex-1 text-sm font-medium text-secondary">
        {label()}
      </Text>
      {props.streaming && (
        <Pulse
          aria-hidden="true"
          class="w-1.5 h-1.5 flex-none rounded-full bg-accent"
          from={0.3}
          to={1}
          duration={0.8}
        />
      )}
    </CollapsibleTrigger>
  );
}

export function ReasoningContent(props: CollapsibleContentProps): JSX.Element {
  const rest = omit(props, "class");
  return (
    <CollapsibleContent
      {...rest}
      class={mergeClasses(
        "min-w-0 border-t border-subtle px-3 py-2",
        props.class,
      )}
    />
  );
}
