import { type JSX, omit } from "solid-js";
import {
  Button as HeadlessButton,
  Text,
  View,
  type ViewProps,
} from "../primitives";
import { Button, type ButtonProps } from "./button";
import { join } from "./class-names";

export function Breadcrumb(props: {
  children?: JSX.Element;
  class?: string;
  "aria-label"?: string;
}): JSX.Element {
  return (
    <View
      role="group"
      aria-label={props["aria-label"] ?? "Breadcrumb"}
      class={join("min-w-0", props.class)}
    >
      {props.children}
    </View>
  );
}

export function BreadcrumbList(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={join(
        "min-w-0 flex flex-wrap items-center gap-1.5 text-sm text-muted",
        props.class,
      )}
    />
  );
}

export function BreadcrumbItem(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={join("min-w-0 flex items-center gap-1.5", props.class)}
    />
  );
}

export interface BreadcrumbLinkProps
  extends Omit<ButtonProps, "class" | "role" | "variant" | "size"> {
  class?: string;
}

export function BreadcrumbLink(props: BreadcrumbLinkProps): JSX.Element {
  return (
    <HeadlessButton
      {...props}
      unstyled
      role="link"
      class={(state) =>
        join(
          "min-w-0 rounded-sm text-sm text-secondary",
          state.hovered && "text-primary",
          state.focusVisible && "border border-focus",
          props.class,
        )
      }
    />
  );
}

export function BreadcrumbPage(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element {
  return (
    <Text
      role="link"
      aria-current="page"
      class={join("min-w-0 text-sm font-medium text-primary", props.class)}
    >
      {props.children}
    </Text>
  );
}

export function BreadcrumbSeparator(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element {
  return (
    <Text aria-hidden class={join("flex-none text-xs text-muted", props.class)}>
      {props.children ?? "/"}
    </Text>
  );
}

export function BreadcrumbEllipsis(props: { class?: string }): JSX.Element {
  return (
    <Text aria-hidden class={join("flex-none text-sm text-muted", props.class)}>
      ...
    </Text>
  );
}

export function Pagination(props: {
  children?: JSX.Element;
  class?: string;
  "aria-label"?: string;
}): JSX.Element {
  return (
    <View
      role="group"
      aria-label={props["aria-label"] ?? "Pagination"}
      class={join("flex items-center", props.class)}
    >
      {props.children}
    </View>
  );
}

export function PaginationContent(props: ViewProps): JSX.Element {
  return (
    <View {...props} class={join("flex items-center gap-1", props.class)} />
  );
}

export function PaginationItem(props: ViewProps): JSX.Element {
  return <View {...props} class={join("flex-none", props.class)} />;
}

export interface PaginationLinkProps
  extends Omit<ButtonProps, "variant" | "size"> {
  active?: boolean;
}

export function PaginationLink(props: PaginationLinkProps): JSX.Element {
  const forwarded = omit(props, "active");
  return (
    <Button
      {...forwarded}
      role="link"
      size="icon"
      variant={props.active ? "outline" : "ghost"}
      selected={props.active}
      aria-current={props.active ? "page" : undefined}
    />
  );
}

export function PaginationPrevious(
  props: Omit<ButtonProps, "variant" | "size">,
): JSX.Element {
  return (
    <Button {...props} variant="ghost" size="sm">
      {props.children ?? "Previous"}
    </Button>
  );
}

export function PaginationNext(
  props: Omit<ButtonProps, "variant" | "size">,
): JSX.Element {
  return (
    <Button {...props} variant="ghost" size="sm">
      {props.children ?? "Next"}
    </Button>
  );
}
