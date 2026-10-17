import { InputProps } from "./input.mjs";
import { NumberFormatOptions } from "@wabou/core";
import { JSX } from "solid-js";
//#region src/components/number-field.d.ts
export interface NumberFieldProps extends Omit<InputProps, "class" | "onInput" | "placeholder" | "value"> {
  /** `undefined` selects uncontrolled mode; `null` represents an empty field. */
  value?: number | null;
  defaultValue?: number | null;
  min?: number;
  max?: number;
  step?: number;
  largeStep?: number;
  locale?: string;
  formatOptions?: NumberFormatOptions;
  placeholder?: string;
  changeOnWheel?: boolean;
  onValueChange?(value: number | null): void;
  class?: string;
  inputClass?: string;
  incrementLabel?: string;
  decrementLabel?: string;
  "aria-label": string;
}
/** Locale-aware numeric input with explicit native stepping semantics. */
export declare function NumberField(props: NumberFieldProps): JSX.Element;
//#endregion
//# sourceMappingURL=number-field.d.mts.map