import {
  TextInput as PrimitiveTextInput,
  type TextInputProps as PrimitiveTextInputProps,
} from "../primitives";
import { type JSX, omit } from "solid-js";
import { join } from "./class-names";

export interface InputProps extends PrimitiveTextInputProps {
  class?: string;
  /** Background utility owned by this input. Defaults to `bg-input`. */
  surfaceClass?: string;
  /**
   * Selects which component owns the input surface.
   *
   * Use `none` when composing an input inside an `InputGroup`; the group then
   * owns its background, border, radius, and shadow without conflicting style
   * declarations on the native editor.
   */
  chrome?: "default" | "none";
}

/** A plain-text input. Secrets must use `PasswordInput`. */
export function Input(props: InputProps): JSX.Element {
  const forwarded = omit(props, "chrome", "surfaceClass");
  return (
    <PrimitiveTextInput
      {...forwarded}
      data-wabou-owns={
        (props.chrome ?? "default") === "default"
          ? "surface native-editor"
          : "native-editor"
      }
      class={join(
        "h-8 w-full px-3 text-sm text-primary",
        (props.chrome ?? "default") === "default" &&
          join(
            "rounded-md border border-subtle shadow-xs",
            props.surfaceClass ?? "bg-input",
          ),
        props.disabled && "opacity-50",
        props.class,
      )}
    />
  );
}
