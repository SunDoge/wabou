import { mergeClasses } from "@wabou/core/style";
import { type JSX, omit } from "solid-js";
import { match } from "ts-pattern";
import { View, type ViewProps } from "../primitives";

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

export interface LabeledSeparatorProps extends Omit<ViewProps, "class"> {
  class?: string;
}

/**
 * A horizontal divider whose compact center content names the boundary.
 *
 * Keep the content short. Interactive content is supported so disclosures can
 * explain what happened between two regions without turning into a full row.
 */
export function LabeledSeparator(props: LabeledSeparatorProps): JSX.Element {
  const rest = omit(props, "class", "children");
  return (
    <View
      {...rest}
      class={mergeClasses(
        "w-full min-w-0 h-7 flex flex-row items-center gap-3",
        props.class,
      )}
    >
      <Separator class="min-w-0 flex-1" />
      <View class="min-w-0 max-w-4/5 flex-none flex items-center justify-center">
        {props.children}
      </View>
      <Separator class="min-w-0 flex-1" />
    </View>
  );
}
