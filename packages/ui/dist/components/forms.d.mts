import { TextAreaProps, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { ButtonProps } from "./button.mjs";
import { InputProps } from "./input.mjs";
import { LabelProps } from "./label.mjs";
import { JSX } from "solid-js";
import { Handle } from "@wabou/core/renderer";
//#region src/components/forms.d.ts
export type FieldOrientation = "vertical" | "horizontal";
export declare function fieldClass(orientation?: FieldOrientation, invalid?: boolean, className?: string): string;
export declare function Field(props: {
  children?: JSX.Element;
  orientation?: FieldOrientation;
  invalid?: boolean;
  required?: boolean;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
export declare function FieldSet(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
export declare function FieldLegend(props: {
  children?: JSX.Element;
  variant?: "legend" | "label";
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
export declare function FieldGroup(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
export interface FieldLabelProps extends LabelProps {}
export declare function FieldLabel(props: FieldLabelProps): import("@wabou/core/jsx-runtime").JSX.Element;
export interface LabeledFieldProps {
  label: JSX.Element;
  description?: JSX.Element;
  invalid?: boolean;
  required?: boolean;
  disabled?: boolean;
  errors?: ReadonlyArray<FieldErrorLike | undefined>;
  class?: string;
  controlRef?: (node: Handle) => void;
  /** Render the native control and attach the supplied ref to its focus owner. */
  renderControl: (ref: (node: Handle) => void) => JSX.Element;
}
/**
 * A complete native field whose visible label always focuses its control.
 * This avoids repeating ad-hoc Handle plumbing in every settings surface.
 */
export declare function LabeledField(props: LabeledFieldProps): import("@wabou/core/jsx-runtime").JSX.Element;
export declare function FieldTitle(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
export declare function FieldContent(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
export declare function FieldDescription(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
export interface FieldErrorLike {
  message?: string;
}
export declare function uniqueFieldErrors(errors: ReadonlyArray<FieldErrorLike | undefined> | undefined): string[];
export declare function fieldErrorLabel(explicit: string | undefined, children: JSX.Element, messages: readonly string[]): string | undefined;
export declare function FieldError(props: {
  children?: JSX.Element;
  errors?: ReadonlyArray<FieldErrorLike | undefined>;
  "aria-label"?: string;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
export declare function FieldSeparator(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
export type InputGroupOrientation = "horizontal" | "vertical";
export type InputGroupVariant = "default" | "quiet";
export type InputGroupAddonAlign = "inline-start" | "inline-end" | "block-start" | "block-end";
export declare function inputGroupClass(orientation: InputGroupOrientation, focused: boolean, invalid: boolean, variant?: InputGroupVariant): string;
export interface InputGroupProps extends Omit<ViewProps, "children"> {
  children?: JSX.Element;
  orientation?: InputGroupOrientation;
  variant?: InputGroupVariant;
  invalid?: boolean;
  disabled?: boolean;
  /** Background utility owned by the compound control. Defaults to `bg-input`. */
  surfaceClass?: string;
}
export declare function InputGroup(props: InputGroupProps): import("@wabou/core/jsx-runtime").JSX.Element;
export declare function InputGroupInput(props: InputProps): import("@wabou/core/jsx-runtime").JSX.Element;
export interface InputGroupAddonProps extends ViewProps {
  align?: InputGroupAddonAlign;
  focusControl?: boolean;
}
export declare function inputGroupAddonClass(align: InputGroupAddonAlign): string;
export declare function InputGroupAddon(props: InputGroupAddonProps): import("@wabou/core/jsx-runtime").JSX.Element;
export declare function InputGroupText(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
export declare function InputGroupButton(props: ButtonProps): import("@wabou/core/jsx-runtime").JSX.Element;
export declare function InputGroupTextArea(props: TextAreaProps & {
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
//#endregion
//# sourceMappingURL=forms.d.mts.map