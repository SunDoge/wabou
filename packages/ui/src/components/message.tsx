import { mergeClasses } from "@wabou/core/style";
import { createContext, type JSX, omit, Show, useContext } from "solid-js";
import { match } from "ts-pattern";
import {
  createFocusWithin,
  createHover,
  Text,
  type TextProps,
  translate2d,
  View,
  type ViewProps,
} from "../primitives";

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
  interacting(): boolean;
}

const MessageContext = createContext<MessageContextValue>({
  align: () => "start",
  interacting: () => false,
});

interface BubbleContextValue extends MessageContextValue {
  variant(): BubbleVariant;
}

const BubbleContext = createContext<BubbleContextValue>({
  align: () => "start",
  interacting: () => false,
  variant: () => "default",
});

export function MessageGroup(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      role={props.role ?? "group"}
      class={mergeClasses("w-full min-w-0 flex flex-col gap-3", props.class)}
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
  return mergeClasses(
    "relative w-full min-w-0 flex gap-2 text-sm",
    align === "end" ? "flex-row-reverse" : "flex-row",
    className,
  );
}

export function Message(props: MessageProps): JSX.Element {
  const hover = createHover();
  const focus = createFocusWithin();
  const forwarded = omit(
    props,
    "align",
    "class",
    "children",
    "onPointerEnter",
    "onPointerLeave",
    "onFocusIn",
    "onFocusOut",
  );
  const context: MessageContextValue = {
    align: () => props.align ?? "start",
    interacting: () => hover.hovered() || focus.focusWithin(),
  };
  return (
    <MessageContext value={context}>
      <View
        {...forwarded}
        role={props.role ?? "group"}
        class={messageClass(context.align(), props.class)}
        onPointerEnter={(event) => {
          hover.bindings.onPointerEnter();
          props.onPointerEnter?.(event);
        }}
        onPointerLeave={(event) => {
          hover.bindings.onPointerLeave();
          props.onPointerLeave?.(event);
        }}
        onFocusIn={(event) => {
          focus.bindings.onFocusIn();
          props.onFocusIn?.(event);
        }}
        onFocusOut={(event) => {
          focus.bindings.onFocusOut();
          props.onFocusOut?.(event);
        }}
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
      class={mergeClasses(
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
      class={mergeClasses(
        "flex-1 min-w-0 flex flex-col gap-2",
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
      class={mergeClasses(
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

export function messageActionsClass(
  align: MessageAlign = "start",
  className?: string,
): string {
  return mergeClasses(
    "h-7 px-1 flex flex-row items-center gap-1",
    align === "end" ? "self-end justify-end" : "self-start justify-start",
    className,
  );
}

export interface MessageActionsProps extends Omit<ViewProps, "class"> {
  /** Override the containing message direction for a local action rail. */
  align?: MessageAlign;
  /** Keep actions visible, or reveal them while their message is hovered/focused. */
  visibility?: "always" | "interaction";
  class?: string;
}

/** Compact, consistently aligned actions belonging to one message. */
export function MessageActions(props: MessageActionsProps): JSX.Element {
  const context = useContext(MessageContext);
  const forwarded = omit(props, "align", "visibility", "class", "children");
  const interactionClass = () =>
    props.visibility === "interaction" && !context.interacting()
      ? "opacity-0 pointer-events-none"
      : "opacity-100";
  return (
    <View
      {...forwarded}
      role={props.role ?? "toolbar"}
      class={messageActionsClass(
        props.align ?? context.align(),
        mergeClasses(interactionClass(), props.class),
      )}
    >
      {props.children}
    </View>
  );
}

export function BubbleGroup(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={mergeClasses("w-full min-w-0 grid gap-2", props.class)}
    >
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
  return mergeClasses(
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
    interacting: message.interacting,
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
  return mergeClasses(
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
      class={mergeClasses(
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
      class={mergeClasses(
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
      class={mergeClasses("w-4 h-4 flex-none", props.class)}
    >
      {props.children}
    </View>
  );
}

export function MarkerContent(props: TextProps): JSX.Element {
  return (
    <Text
      {...props}
      class={mergeClasses(
        "min-w-0 whitespace-nowrap text-sm text-muted",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}
