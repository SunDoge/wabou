import { createContext, type JSX, omit, Show, useContext } from "solid-js";
import { match } from "ts-pattern";
import {
  Text,
  type TextProps,
  translate2d,
  View,
  type ViewProps,
} from "../primitives";
import { join } from "./class-names";

export type MessageAlign = "start" | "end";
export type BubbleVariant =
  | "default"
  | "secondary"
  | "muted"
  | "tinted"
  | "outline"
  | "ghost"
  | "destructive";

interface MessageContextValue {
  align(): MessageAlign;
}

const MessageContext = createContext<MessageContextValue>({
  align: () => "start",
});

interface BubbleContextValue extends MessageContextValue {
  variant(): BubbleVariant;
}

const BubbleContext = createContext<BubbleContextValue>({
  align: () => "start",
  variant: () => "default",
});

export function MessageGroup(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      role={props.role ?? "group"}
      class={join("w-full min-w-0 flex flex-col gap-3", props.class)}
    >
      {props.children}
    </View>
  );
}

export interface MessageProps extends Omit<ViewProps, "class"> {
  align?: MessageAlign;
  class?: string;
}

export function messageClass(
  align: MessageAlign = "start",
  className?: string,
): string {
  return join(
    "relative w-full min-w-0 flex gap-2 text-sm",
    align === "end" ? "flex-row-reverse" : "flex-row",
    className,
  );
}

export function Message(props: MessageProps): JSX.Element {
  const forwarded = omit(props, "align", "class", "children");
  const context: MessageContextValue = {
    align: () => props.align ?? "start",
  };
  return (
    <MessageContext value={context}>
      <View
        {...forwarded}
        role={props.role ?? "group"}
        class={messageClass(context.align(), props.class)}
      >
        {props.children}
      </View>
    </MessageContext>
  );
}

export function MessageAvatar(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={join(
        "w-8 h-8 flex-none self-end overflow-hidden rounded-full bg-control flex items-center justify-center",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export function MessageContent(props: ViewProps): JSX.Element {
  const context = useContext(MessageContext);
  return (
    <View
      {...props}
      class={join(
        "w-full min-w-0 flex flex-col gap-2",
        context.align() === "end" ? "items-end" : "items-start",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export function MessageHeader(props: TextProps): JSX.Element {
  const context = useContext(MessageContext);
  return (
    <Text
      {...props}
      class={join(
        "max-w-full min-w-0 px-3 text-xs font-medium text-muted",
        context.align() === "end" && "text-right",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}

export const MessageFooter = MessageHeader;

export function BubbleGroup(props: ViewProps): JSX.Element {
  return (
    <View {...props} class={join("min-w-0 flex flex-col gap-2", props.class)}>
      {props.children}
    </View>
  );
}

export interface BubbleProps extends Omit<ViewProps, "class"> {
  variant?: BubbleVariant;
  align?: MessageAlign;
  class?: string;
}

export function bubbleClass(
  variant: BubbleVariant = "default",
  align: MessageAlign = "start",
  className?: string,
): string {
  return join(
    "relative min-w-0 flex flex-col gap-1",
    variant === "ghost" ? "max-w-full" : "max-w-4/5",
    align === "end" ? "self-end items-end" : "self-start items-start",
    className,
  );
}

export function Bubble(props: BubbleProps): JSX.Element {
  const message = useContext(MessageContext);
  const forwarded = omit(props, "variant", "align", "class", "children");
  const context: BubbleContextValue = {
    variant: () => props.variant ?? "default",
    align: () => props.align ?? message.align(),
  };
  return (
    <BubbleContext value={context}>
      <View
        {...forwarded}
        class={bubbleClass(context.variant(), context.align(), props.class)}
      >
        {props.children}
      </View>
    </BubbleContext>
  );
}

export function bubbleContentClass(
  variant: BubbleVariant,
  className?: string,
): string {
  const colors = match(variant)
    .with("default", () => "border-transparent bg-accent text-on-accent")
    .with("secondary", () => "border-transparent bg-control text-primary")
    .with("muted", () => "border-transparent bg-control text-secondary")
    .with("tinted", () => "border-transparent bg-selected text-primary")
    .with("outline", () => "border-subtle bg-surface text-primary")
    .with("ghost", () => "border-transparent bg-transparent text-primary")
    .with(
      "destructive",
      () => "border-danger bg-danger-surface text-danger-primary",
    )
    .exhaustive();
  return join(
    "max-w-full min-w-0 overflow-hidden rounded-xl border",
    variant === "ghost" ? "p-0" : "px-3 py-2",
    colors,
    className,
  );
}

export function BubbleContent(props: ViewProps): JSX.Element {
  const context = useContext(BubbleContext);
  return (
    <View {...props} class={bubbleContentClass(context.variant(), props.class)}>
      {props.children}
    </View>
  );
}

export function BubbleReactions(props: {
  children?: JSX.Element;
  side?: "top" | "bottom";
  align?: MessageAlign;
  class?: string;
}): JSX.Element {
  const bubble = useContext(BubbleContext);
  const side = () => props.side ?? "bottom";
  const align = () => props.align ?? bubble.align();
  return (
    <View
      class={join(
        "relative z-10 flex-none flex items-center justify-center gap-1 rounded-full bg-control px-1.5 py-0.5 text-sm",
        align() === "end" ? "self-end" : "self-start",
        props.class,
      )}
      transform={translate2d(0, side() === "top" ? 4 : -4)}
    >
      {props.children}
    </View>
  );
}

export type MarkerVariant = "default" | "separator" | "border";

export function Marker(props: {
  children?: JSX.Element;
  variant?: MarkerVariant;
  class?: string;
}): JSX.Element {
  const variant = () => props.variant ?? "default";
  return (
    <View
      class={join(
        "w-full min-w-0 min-h-4 flex items-center gap-2 text-sm text-muted",
        variant() === "border" && "border-b border-subtle pb-2",
        props.class,
      )}
    >
      <Show when={variant() === "separator"}>
        <View aria-hidden="true" class="flex-1 min-w-0 h-px bg-subtle" />
      </Show>
      {props.children}
      <Show when={variant() === "separator"}>
        <View aria-hidden="true" class="flex-1 min-w-0 h-px bg-subtle" />
      </Show>
    </View>
  );
}

export function MarkerIcon(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      aria-hidden="true"
      class={join("w-4 h-4 flex-none", props.class)}
    >
      {props.children}
    </View>
  );
}

export function MarkerContent(props: TextProps): JSX.Element {
  return (
    <Text
      {...props}
      class={join("min-w-0 whitespace-nowrap text-sm text-muted", props.class)}
    >
      {props.children}
    </Text>
  );
}
