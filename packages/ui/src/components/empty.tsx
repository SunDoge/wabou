import { type JSX, omit } from "solid-js";
import { match } from "ts-pattern";
import {
  Column,
  Text,
  type TextProps,
  View,
  type ViewProps,
} from "../primitives";
import { mergeClasses } from "@wabou/core/style";

export type EmptyVariant = "surface" | "plain";
export type EmptyMediaVariant = "default" | "icon";

export interface EmptyProps extends Omit<ViewProps, "class"> {
  /** `plain` embeds inside an existing surface without nesting another card. */
  variant?: EmptyVariant;
  class?: string;
}

export function emptyClass(
  variant: EmptyVariant = "surface",
  className?: string,
): string {
  return mergeClasses(
    "w-full min-w-0 flex-1 p-8 items-center justify-center gap-6 text-center",
    variant === "surface"
      ? "min-h-64 rounded-lg border border-subtle bg-surface shadow-xs"
      : "min-h-0 bg-transparent",
    className,
  );
}

/** A composable empty-state region based on shadcn's Empty anatomy. */
export function Empty(props: EmptyProps): JSX.Element {
  const rest = omit(props, "variant", "class", "children");
  return (
    <Column {...rest} class={emptyClass(props.variant, props.class)}>
      {props.children}
    </Column>
  );
}

export function EmptyHeader(props: ViewProps): JSX.Element {
  return (
    <Column
      {...props}
      class={mergeClasses(
        "w-full max-w-sm min-w-0 items-center gap-2",
        props.class,
      )}
    >
      {props.children}
    </Column>
  );
}

export interface EmptyMediaProps extends Omit<ViewProps, "class"> {
  variant?: EmptyMediaVariant;
  class?: string;
}

export function emptyMediaClass(
  variant: EmptyMediaVariant = "default",
  className?: string,
): string {
  return mergeClasses(
    "mb-2 flex-none flex items-center justify-center",
    match(variant)
      .with("default", () => "bg-transparent")
      .with("icon", () =>
        mergeClasses("w-10 h-10 rounded-lg", "bg-control text-primary"),
      )
      .exhaustive(),
    className,
  );
}

export function EmptyMedia(props: EmptyMediaProps): JSX.Element {
  const rest = omit(props, "variant", "class", "children");
  return (
    <View {...rest} class={emptyMediaClass(props.variant, props.class)}>
      {props.children}
    </View>
  );
}

export function EmptyTitle(props: TextProps): JSX.Element {
  return (
    <Text
      {...props}
      role={props.role ?? "heading"}
      class={mergeClasses("text-lg font-medium text-primary", props.class)}
    >
      {props.children}
    </Text>
  );
}

export function EmptyDescription(props: TextProps): JSX.Element {
  return (
    <Text
      {...props}
      class={mergeClasses(
        "w-full min-w-0 whitespace-normal text-center text-sm text-muted",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}

export function EmptyContent(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={mergeClasses(
        "w-full max-w-sm min-w-0 flex flex-col items-center gap-4 text-sm",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}
