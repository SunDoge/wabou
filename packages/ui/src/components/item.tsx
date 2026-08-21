import { type JSX, omit } from "solid-js";
import { match } from "ts-pattern";
import { Text, type TextProps, View, type ViewProps } from "../primitives";
import { join } from "./class-names";

export type ItemVariant = "default" | "outline" | "muted";
export type ItemSize = "default" | "sm";
export type ItemMediaVariant = "default" | "icon" | "image";

export function itemClass(
  variant: ItemVariant = "default",
  size: ItemSize = "default",
  className?: string,
): string {
  const colors = match(variant)
    .with("default", () => "border-transparent bg-transparent")
    .with("outline", () => "border-subtle bg-transparent")
    .with("muted", () => "border-transparent bg-control")
    .exhaustive();
  const spacing = match(size)
    .with("default", () => "gap-4 p-4")
    .with("sm", () => "gap-2 px-4 py-3")
    .exhaustive();
  return join(
    "w-full min-w-0 flex flex-row flex-wrap items-center rounded-md border text-sm",
    colors,
    spacing,
    className,
  );
}

export interface ItemProps extends Omit<ViewProps, "class"> {
  variant?: ItemVariant;
  size?: ItemSize;
  class?: string;
}

/** A composable list row based on shadcn's Item anatomy. */
export function Item(props: ItemProps): JSX.Element {
  const rest = omit(props, "variant", "size", "class", "children");
  return (
    <View
      {...rest}
      role={props.role ?? "none"}
      class={itemClass(props.variant, props.size, props.class)}
    >
      {props.children}
    </View>
  );
}

export function ItemGroup(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      role={props.role ?? "group"}
      class={join("w-full min-w-0 flex flex-col", props.class)}
    >
      {props.children}
    </View>
  );
}

export function ItemSeparator(props: { class?: string }): JSX.Element {
  return (
    <View
      aria-hidden="true"
      class={join("w-full h-px flex-none bg-subtle", props.class)}
    />
  );
}

export function itemMediaClass(
  variant: ItemMediaVariant = "default",
  className?: string,
): string {
  return join(
    "flex-none flex items-center justify-center gap-2",
    match(variant)
      .with("default", () => "bg-transparent")
      .with("icon", () => "w-8 h-8 rounded-sm border border-subtle bg-control")
      .with("image", () => "w-10 h-10 overflow-hidden rounded-sm")
      .exhaustive(),
    className,
  );
}

export function ItemMedia(props: {
  children?: JSX.Element;
  variant?: ItemMediaVariant;
  class?: string;
}): JSX.Element {
  return (
    <View class={itemMediaClass(props.variant, props.class)}>
      {props.children}
    </View>
  );
}

export function ItemContent(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={join("flex-1 min-w-0 flex flex-col gap-1", props.class)}
    >
      {props.children}
    </View>
  );
}

export function ItemTitle(props: TextProps): JSX.Element {
  return (
    <Text
      {...props}
      class={join("min-w-0 text-sm font-medium text-primary", props.class)}
    >
      {props.children}
    </Text>
  );
}

export function ItemDescription(props: TextProps): JSX.Element {
  return (
    <Text
      {...props}
      maxLines={props.maxLines ?? 2}
      class={join(
        "w-full min-w-0 whitespace-normal text-sm text-muted",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}

export function ItemActions(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={join("flex-none flex items-center gap-2", props.class)}
    >
      {props.children}
    </View>
  );
}

export function ItemHeader(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={join(
        "w-full min-w-0 flex items-center justify-between gap-2",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export const ItemFooter = ItemHeader;
