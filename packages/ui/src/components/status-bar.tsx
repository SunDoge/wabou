import { mergeClasses } from "@wabou/core/style";
import { type JSX, omit } from "solid-js";
import { Text, type TextProps, View, type ViewProps } from "../primitives";

export interface StatusBarProps extends Omit<ViewProps, "class"> {
  class?: string;
}

/** Persistent low-emphasis application state at the bottom of a desktop window. */
export function StatusBar(props: StatusBarProps): JSX.Element {
  return (
    <View
      {...props}
      role={props.role ?? "status"}
      class={mergeClasses(
        "h-7 w-full min-w-0 flex-none flex flex-row items-center gap-1 px-2 border-t border-subtle bg-control",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export interface StatusBarItemProps extends Omit<TextProps, "class"> {
  class?: string;
  grow?: boolean;
}

export function StatusBarItem(props: StatusBarItemProps): JSX.Element {
  const forwarded = omit(props, "class", "grow", "children");
  return (
    <Text
      {...forwarded}
      class={mergeClasses(
        "h-full min-w-0 px-1.5 flex items-center text-xs text-muted whitespace-nowrap",
        props.grow ? "flex-1" : "flex-none",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}

export function StatusBarSeparator(props: { class?: string }): JSX.Element {
  return (
    <View
      role="separator"
      aria-orientation="vertical"
      class={mergeClasses("w-px h-4 flex-none bg-subtle", props.class)}
    />
  );
}

export interface StatusBarGroupProps extends Omit<ViewProps, "class"> {
  class?: string;
  grow?: boolean;
  shrink?: boolean;
}

/** A shrink-safe status bar group for icons, indicators and related text. */
export function StatusBarGroup(props: StatusBarGroupProps): JSX.Element {
  const forwarded = omit(props, "class", "grow", "shrink", "children");
  return (
    <View
      {...forwarded}
      class={mergeClasses(
        "h-full min-w-0 px-1.5 flex flex-row items-center gap-1.5 text-xs text-muted",
        props.grow ? "flex-1" : props.shrink ? "shrink" : "flex-none",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export type StatusBarIndicatorTone = "accent" | "danger" | "muted" | "success";

export interface StatusBarIndicatorProps {
  tone?: StatusBarIndicatorTone;
  class?: string;
}

const indicatorToneClass: Record<StatusBarIndicatorTone, string> = {
  accent: "bg-accent",
  danger: "bg-danger-primary",
  muted: "bg-muted",
  success: "bg-success-primary",
};

/** Compact, decorative state indicator with a theme-aware semantic tone. */
export function StatusBarIndicator(
  props: StatusBarIndicatorProps,
): JSX.Element {
  return (
    <View
      aria-hidden="true"
      class={mergeClasses(
        "w-1.5 h-1.5 flex-none rounded-full",
        indicatorToneClass[props.tone ?? "muted"],
        props.class,
      )}
    />
  );
}
