import { mergeClasses } from "@wabou/core/style";
import { createComponent, type JSX, omit } from "solid-js";
import { match } from "ts-pattern";
import { Text, type TextProps, View, type ViewProps } from "../primitives";
import {
  type ButtonGroupButtonSize,
  type ButtonGroupButtonVariant,
  ButtonGroupContext,
  type ButtonGroupOrientation,
  buttonGroupItemCorners,
  createButtonGroupContext,
  useButtonGroupItem,
} from "./button-group-context";
import { componentsControlContentSize } from "./theme";

export interface ButtonGroupProps extends Omit<ViewProps, "class"> {
  orientation?: ButtonGroupOrientation;
  size?: ButtonGroupButtonSize;
  variant?: ButtonGroupButtonVariant;
  disabled?: boolean;
  class?: string;
}

/** A single bordered control surface composed from ordinary Wabou buttons. */
export function ButtonGroup(props: ButtonGroupProps): JSX.Element {
  const orientation = () => props.orientation ?? "horizontal";
  const size = () => props.size ?? "default";
  const variant = () => props.variant ?? "default";
  const disabled = () => props.disabled ?? false;
  const layout = () =>
    match(orientation())
      .with("horizontal", () => "flex-row items-stretch")
      .with("vertical", () => "flex-col items-stretch")
      .exhaustive();
  const context = createButtonGroupContext(orientation, {
    size,
    variant,
    disabled,
  });
  const forwarded = omit(
    props,
    "orientation",
    "size",
    "variant",
    "disabled",
    "class",
    "children",
  );
  return createComponent(ButtonGroupContext, {
    value: context,
    get children() {
      return (
        <View
          {...forwarded}
          role="group"
          aria-label={props["aria-label"]}
          aria-orientation={orientation()}
          aria-disabled={disabled()}
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
  const size = () => groupItem?.size() ?? "default";
  return (
    <Text
      {...props}
      class={mergeClasses(
        "flex-none flex items-center whitespace-nowrap font-medium text-secondary bg-control",
        componentsControlContentSize(size()),
        groupItem?.disabled() && "opacity-50",
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
