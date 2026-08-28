import { createContext, type JSX, omit, useContext } from "solid-js";
import { match } from "ts-pattern";
import { Text, type TextProps, View, type ViewProps } from "../primitives";
import { Button, type ButtonProps } from "./button";
import { mergeClasses } from "@wabou/core/style";

export type AttachmentState =
  | "idle"
  | "uploading"
  | "processing"
  | "error"
  | "done";
export type AttachmentSize = "default" | "sm" | "xs";
export type AttachmentOrientation = "horizontal" | "vertical";

interface AttachmentContextValue {
  state(): AttachmentState;
  size(): AttachmentSize;
  orientation(): AttachmentOrientation;
}

const AttachmentContext = createContext<AttachmentContextValue>({
  state: () => "done",
  size: () => "default",
  orientation: () => "horizontal",
});

export function attachmentClass(options: {
  state?: AttachmentState;
  size?: AttachmentSize;
  orientation?: AttachmentOrientation;
  class?: string;
}): string {
  const state = options.state ?? "done";
  const size = options.size ?? "default";
  const orientation = options.orientation ?? "horizontal";
  return mergeClasses(
    "max-w-full min-w-0 flex-none flex border bg-surface text-primary",
    match(orientation)
      .with("horizontal", () => "min-w-40 flex-row flex-wrap items-center")
      .with("vertical", () => "w-28 flex-col items-stretch")
      .exhaustive(),
    match(size)
      .with("default", () => "gap-2 rounded-xl p-2 text-sm")
      .with("sm", () => "gap-2 rounded-lg p-1.5 text-xs")
      .with("xs", () => "gap-1.5 rounded-md p-1 text-xs")
      .exhaustive(),
    match(state)
      .with("idle", () => "border-strong")
      .with("uploading", () => "border-focus")
      .with("processing", () => "border-accent")
      .with("error", () => "border-danger bg-danger-surface")
      .with("done", () => "border-subtle")
      .exhaustive(),
    options.class,
  );
}

export interface AttachmentProps extends Omit<ViewProps, "class"> {
  state?: AttachmentState;
  size?: AttachmentSize;
  orientation?: AttachmentOrientation;
  class?: string;
}

/** File/task summary anatomy adapted from shadcn without DOM data selectors. */
export function Attachment(props: AttachmentProps): JSX.Element {
  const forwarded = omit(
    props,
    "state",
    "size",
    "orientation",
    "class",
    "children",
  );
  const context: AttachmentContextValue = {
    state: () => props.state ?? "done",
    size: () => props.size ?? "default",
    orientation: () => props.orientation ?? "horizontal",
  };
  return (
    <AttachmentContext value={context}>
      <View
        {...forwarded}
        class={attachmentClass({
          state: context.state(),
          size: context.size(),
          orientation: context.orientation(),
          class: props.class,
        })}
      >
        {props.children}
      </View>
    </AttachmentContext>
  );
}

export function attachmentMediaClass(
  variant: "icon" | "image",
  context: Pick<AttachmentContextValue, "state" | "size" | "orientation">,
  className?: string,
): string {
  const size = context.size();
  const orientation = context.orientation();
  const state = context.state();
  return mergeClasses(
    "aspect-square flex-none overflow-hidden flex items-center justify-center rounded-lg",
    orientation === "vertical"
      ? "w-full"
      : match(size)
          .with("default", () => "w-10")
          .with("sm", () => "w-8")
          .with("xs", () => "w-7")
          .exhaustive(),
    state === "error"
      ? "bg-danger-surface text-danger-primary"
      : "bg-control text-primary",
    variant === "image" && state !== "done" && "opacity-60",
    className,
  );
}

export function AttachmentMedia(props: {
  children?: JSX.Element;
  variant?: "icon" | "image";
  class?: string;
}): JSX.Element {
  const context = useContext(AttachmentContext);
  return (
    <View
      class={attachmentMediaClass(
        props.variant ?? "icon",
        context,
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export function AttachmentContent(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={mergeClasses(
        "max-w-full min-w-0 flex-1 flex flex-col gap-0.5",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export function AttachmentTitle(props: TextProps): JSX.Element {
  return (
    <Text
      {...props}
      maxLines={props.maxLines ?? 1}
      class={mergeClasses(
        "max-w-full min-w-0 font-medium text-primary",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}

export function AttachmentDescription(props: TextProps): JSX.Element {
  const context = useContext(AttachmentContext);
  return (
    <Text
      {...props}
      maxLines={props.maxLines ?? 1}
      class={mergeClasses(
        "max-w-full min-w-0 text-xs",
        context.state() === "error" ? "text-danger-primary" : "text-muted",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}

export function AttachmentActions(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={mergeClasses("flex-none flex items-center gap-1", props.class)}
    >
      {props.children}
    </View>
  );
}

export function AttachmentAction(props: ButtonProps): JSX.Element {
  return (
    <Button
      {...props}
      variant={props.variant ?? "ghost"}
      size={props.size ?? "sm"}
    />
  );
}

export function attachmentGroupClass(className?: string): string {
  return mergeClasses(
    "w-full min-w-0 overflow-x-auto overflow-y-hidden py-1 flex flex-row items-start gap-3",
    className,
  );
}

export function AttachmentGroup(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={attachmentGroupClass(props.class)}
      scrollbar={props.scrollbar ?? { visibility: "hidden" }}
    >
      {props.children}
    </View>
  );
}
