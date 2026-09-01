import { mergeClasses } from "@wabou/core/style";
import { createComponent, type JSX } from "solid-js";
import { match } from "ts-pattern";
import { Text, type TextProps, View, type ViewProps } from "../primitives";
import {
  ButtonGroupContext,
  type ButtonGroupOrientation,
  buttonGroupItemCorners,
  createButtonGroupContext,
  useButtonGroupItem,
} from "./button-group-context";

export interface ButtonGroupProps extends Omit<ViewProps, "class"> {
  orientation?: ButtonGroupOrientation;
  class?: string;
}

/** A single bordered control surface composed from ordinary Wabou buttons. */
export function ButtonGroup(props: ButtonGroupProps): JSX.Element {
  const orientation = () => props.orientation ?? "horizontal";
  const layout = () =>
    match(orientation())
      .with("horizontal", () => "flex-row items-stretch")
      .with("vertical", () => "flex-col items-stretch")
      .exhaustive();
  const context = createButtonGroupContext(orientation);
  return createComponent(ButtonGroupContext, {
    value: context,
    get children() {
      return (
        <View
          {...props}
          role="group"
          aria-label={props["aria-label"]}
          class={mergeClasses(
            "min-w-0 flex gap-0 rounded-lg border border-strong bg-surface shadow-xs",
            layout(),
            props.class,
          )}
        >
          {props.children}
        </View>
      );
    },
  });
}

export function ButtonGroupText(props: TextProps): JSX.Element {
  const groupItem = useButtonGroupItem();
  return (
    <Text
      {...props}
      class={mergeClasses(
        "min-h-8 px-3 flex-none flex items-center whitespace-nowrap text-sm font-medium text-secondary bg-control",
        groupItem && buttonGroupItemCorners(groupItem),
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}

export interface ButtonGroupSeparatorProps {
  orientation?: "horizontal" | "vertical";
  class?: string;
}

export function ButtonGroupSeparator(
  props: ButtonGroupSeparatorProps,
): JSX.Element {
  return (
    <View
      role="separator"
      aria-hidden="true"
      class={mergeClasses(
        "flex-none self-stretch bg-strong",
        match(props.orientation ?? "vertical")
          .with("vertical", () => "w-px min-h-full")
          .with("horizontal", () => "h-px min-w-full")
          .exhaustive(),
        props.class,
      )}
    />
  );
}
