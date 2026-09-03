import { mergeClasses } from "@wabou/core/style";
import type { JSX } from "solid-js";
import { Text, View, type ViewProps } from "../primitives";

export interface PropertyListProps extends Omit<ViewProps, "class"> {
  class?: string;
}

export function PropertyList(props: PropertyListProps): JSX.Element {
  return (
    <View
      {...props}
      role="table"
      class={mergeClasses(
        "w-full min-w-0 flex flex-col overflow-hidden rounded-xl border border-subtle bg-surface",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export interface PropertyRowProps {
  name: string;
  value: string;
  class?: string;
  nameClass?: string;
  valueClass?: string;
}

/** Compact two-column data intended for inspectors and settings summaries. */
export function PropertyRow(props: PropertyRowProps): JSX.Element {
  return (
    <View
      role="row"
      aria-label={props.name}
      class={mergeClasses(
        "min-h-10 w-full min-w-0 px-3 py-2 flex flex-row items-center gap-3 border-b border-subtle",
        props.class,
      )}
    >
      <Text
        role="cell"
        class={mergeClasses(
          "w-48 flex-none text-xs font-mono text-accent whitespace-nowrap",
          props.nameClass,
        )}
      >
        {props.name}
      </Text>
      <Text
        role="cell"
        class={mergeClasses(
          "min-w-0 flex-1 text-xs text-muted whitespace-normal",
          props.valueClass,
        )}
      >
        {props.value}
      </Text>
    </View>
  );
}
