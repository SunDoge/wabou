import type { JSX } from "solid-js";
import { mergeClasses } from "@wabou/core/style";
import { View, type ViewProps } from "./view";

export interface LayoutProps extends ViewProps {
  class?: string;
  children?: JSX.Element;
}

/** Horizontal flex container. No wrapper node is added beyond the host View. */
export function Row(props: LayoutProps): JSX.Element {
  return <View {...props} class={mergeClasses("flex flex-row", props.class)} />;
}

/** Vertical flex container. No wrapper node is added beyond the host View. */
export function Column(props: LayoutProps): JSX.Element {
  return <View {...props} class={mergeClasses("flex flex-col", props.class)} />;
}

/** Flex container that centers children on both axes. */
export function Center(props: LayoutProps): JSX.Element {
  return (
    <View
      {...props}
      class={mergeClasses("flex items-center justify-center", props.class)}
    />
  );
}
