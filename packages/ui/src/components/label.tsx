import type { Handle } from "@wabou/core/renderer";
import { type JSX, omit } from "solid-js";
import { Text, type TextProps } from "../primitives";
import { mergeClasses } from "@wabou/core/style";
import { componentsDisabledInteractiveClass } from "./theme";

export interface LabelProps extends Omit<TextProps, "class" | "role"> {
  class?: string;
  disabled?: boolean;
  /**
   * Explicit native control target. Unlike HTML `for`, this cannot silently
   * point at a missing string id and remains safe across retained-tree reuse.
   */
  control?: Handle | (() => Handle | undefined);
}

function resolveControl(control: LabelProps["control"]): Handle | undefined {
  return typeof control === "function" ? control() : control;
}

/** Text label that forwards pointer activation to an explicit native control. */
export function Label(props: LabelProps): JSX.Element {
  const rest = omit(
    props,
    "class",
    "children",
    "disabled",
    "control",
    "onClick",
  );
  return (
    <Text
      {...rest}
      role="label"
      aria-disabled={props.disabled}
      class={mergeClasses(
        "min-w-0 text-sm font-medium text-primary",
        props.disabled ? "" : "cursor-pointer",
        props.class,
        componentsDisabledInteractiveClass(props.disabled ?? false),
      )}
      onClick={(event) => {
        props.onClick?.(event);
        if (!props.disabled && !event.defaultPrevented) {
          resolveControl(props.control)?.focus();
        }
      }}
    >
      {props.children}
    </Text>
  );
}
