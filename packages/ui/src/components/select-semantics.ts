import { mergeClasses } from "@wabou/core/style";
import type { ButtonState } from "../primitives";
import {
  componentsDisabledControlClass,
  componentsDisabledItemClass,
} from "./theme";

export type PickerTriggerVariant = "default" | "ghost";

export function pickerTriggerClass(
  variant: PickerTriggerVariant,
  state: ButtonState,
): string {
  if (state.disabled)
    return mergeClasses(
      variant === "default" ? "shadow-none" : "bg-transparent shadow-none",
      componentsDisabledControlClass(true),
    );
  const focus = state.focused ? "border-focus" : "border-transparent";
  if (variant === "default") {
    return mergeClasses(
      "bg-input shadow-xs",
      state.focused ? "border-focus" : "border-subtle",
    );
  }
  return mergeClasses(
    state.pressed
      ? "bg-control-pressed"
      : state.hovered
        ? "bg-control-hover"
        : "bg-transparent",
    "shadow-none",
    focus,
  );
}

/** Keep semantic ID references live for the same lifetime as the popup node. */
export function selectControlsId(
  listboxId: string,
  open: boolean,
): string | undefined {
  return open ? listboxId : undefined;
}

/** Shared visual contract for selectable and explicitly unavailable options. */
export function pickerOptionClass(
  disabled: boolean,
  highlighted: boolean,
): string {
  if (disabled) return componentsDisabledItemClass(true);
  if (highlighted) return "bg-control-hover text-primary cursor-pointer";
  return "bg-transparent text-secondary cursor-pointer";
}
