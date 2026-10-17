import { PasswordInputProps as PasswordInputProps$1, TextAreaProps as TextAreaProps$1, TextInputProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/input.d.ts
export interface InputProps extends TextInputProps {
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
export declare function Input(props: InputProps): JSX.Element;
export interface PasswordInputProps extends PasswordInputProps$1 {
  class?: string;
}
/** A native secret input whose value never crosses into JavaScript. */
export declare function PasswordInput(props: PasswordInputProps): JSX.Element;
export interface TextAreaProps extends TextAreaProps$1 {
  class?: string;
  /** Background utility owned by this textarea. Defaults to `bg-input`. */
  surfaceClass?: string;
  /** Use `none` when an enclosing composition owns the visual surface. */
  chrome?: "default" | "none";
}
export declare function TextArea(props: TextAreaProps): JSX.Element;
//#endregion
//# sourceMappingURL=input.d.mts.map