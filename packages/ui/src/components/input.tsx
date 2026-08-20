import {
  TextInput as PrimitiveTextInput,
  type TextInputProps as PrimitiveTextInputProps,
} from "../primitives";
import type { JSX } from "solid-js";
import { join } from "./class-names";

export interface InputProps extends PrimitiveTextInputProps {
  class?: string;
}

/** A plain-text input. Secrets must use `PasswordInput`. */
export function Input(props: InputProps): JSX.Element {
  return (
    <PrimitiveTextInput
      {...props}
      class={join(
        "h-8 w-full px-3 rounded-md border text-sm shadow-xs",
        "border-subtle bg-input text-primary",
        props.disabled && "opacity-50",
        props.class,
      )}
    />
  );
}
