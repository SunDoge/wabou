import { type JSX, omit } from "solid-js";
import { match } from "ts-pattern";
import { View, type ViewProps } from "../primitives";
import { mergeClasses } from "@wabou/core/style";

export interface SeparatorProps
  extends Omit<
    ViewProps,
    "aria-hidden" | "aria-orientation" | "class" | "role"
  > {
  /** Direction of the dividing line. */
  orientation?: "horizontal" | "vertical";
  /**
   * Decorative separators stay out of the semantic tree. Set to false when
   * the divider represents a meaningful boundary between regions or controls.
   */
  decorative?: boolean;
  class?: string;
}

/** A visual divider with an opt-in semantic separator contract. */
export function Separator(props: SeparatorProps): JSX.Element {
  const orientation = () => props.orientation ?? "horizontal";
  const decorative = () => props.decorative ?? true;
  const dimensions = () =>
    match(orientation())
      .with("horizontal", () => "h-px w-full")
      .with("vertical", () => "w-px h-full")
      .exhaustive();
  const rest = omit(props, "class", "decorative", "orientation");

  return (
    <View
      {...rest}
      role={decorative() ? "presentation" : "separator"}
      aria-hidden={decorative() ? "true" : undefined}
      aria-orientation={decorative() ? undefined : orientation()}
      class={mergeClasses("flex-none bg-subtle", dimensions(), props.class)}
    />
  );
}
