import type { JSX } from "solid-js";
import { Text, type TextProps, View, type ViewProps } from "../primitives";
import { mergeClasses } from "@wabou/core/style";

export interface StatusBarProps extends Omit<ViewProps, "class"> {
  class?: string;
}

/** Persistent low-emphasis application state at the bottom of a desktop window. */
export function StatusBar(props: StatusBarProps): JSX.Element {
  return (
    <View
      {...props}
      role="status"
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
  return (
    <Text
      {...props}
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
