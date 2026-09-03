import { mergeClasses } from "@wabou/core/style";
import { type JSX, omit } from "solid-js";
import {
  PasswordInput as PrimitivePasswordInput,
  type PasswordInputProps as PrimitivePasswordInputProps,
  TextArea as PrimitiveTextArea,
  type TextAreaProps as PrimitiveTextAreaProps,
  TextInput as PrimitiveTextInput,
  type TextInputProps as PrimitiveTextInputProps,
} from "../primitives";
import { componentsControlContentSize, componentsControlSize } from "./theme";

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
      class={mergeClasses(
        // Match GPUI Components' single-line editor frame: a 20px line box
        // inside the 32px control with explicit vertical inset. The native
        // widget owns editing; this component owns its visual geometry.
        "w-full flex items-center py-2 text-primary",
        (props.chrome ?? "default") === "default"
          ? componentsControlSize("default")
          : componentsControlContentSize("default"),
        (props.chrome ?? "default") === "default" &&
          mergeClasses(
            "border border-subtle shadow-xs",
            props.surfaceClass ?? "bg-input",
          ),
        props.disabled && "opacity-50",
        props.class,
      )}
    />
  );
}

export interface PasswordInputProps extends PrimitivePasswordInputProps {
  class?: string;
}

/** A native secret input whose value never crosses into JavaScript. */
export function PasswordInput(props: PasswordInputProps): JSX.Element {
  return (
    <PrimitivePasswordInput
      {...props}
      class={mergeClasses(
        "w-full flex items-center py-2 border shadow-xs",
        componentsControlSize("default"),
        "border-subtle bg-input text-primary",
        props.disabled && "opacity-50",
        props.class,
      )}
    />
  );
}

export interface TextAreaProps extends PrimitiveTextAreaProps {
  class?: string;
  /** Background utility owned by this textarea. Defaults to `bg-input`. */
  surfaceClass?: string;
  /** Use `none` when an enclosing composition owns the visual surface. */
  chrome?: "default" | "none";
}

export function TextArea(props: TextAreaProps): JSX.Element {
  const forwarded = omit(props, "chrome", "surfaceClass");
  return (
    <PrimitiveTextArea
      {...forwarded}
      data-wabou-owns={
        (props.chrome ?? "default") === "default"
          ? "surface native-editor"
          : "native-editor"
      }
      class={mergeClasses(
        "h-24 w-full px-2.5 py-2 text-sm leading-normal text-primary",
        (props.chrome ?? "default") === "default" &&
          mergeClasses(
            "rounded-md border border-subtle shadow-xs",
            props.surfaceClass ?? "bg-input",
          ),
        props.disabled && "opacity-50",
        props.class,
      )}
    />
  );
}
