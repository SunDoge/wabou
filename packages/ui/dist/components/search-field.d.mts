import { InputProps } from "./input.mjs";
import { JSX } from "solid-js";
import { Handle } from "@wabou/core/renderer";
//#region src/components/search-field.d.ts
export interface SearchFieldProps extends Omit<InputProps, "class" | "onInput" | "ref" | "value"> {
  /** `undefined` selects uncontrolled mode. */
  value?: string;
  defaultValue?: string;
  onValueChange?(value: string): void;
  /** Called when Enter is pressed with the current query. */
  onSearch?(value: string): void;
  onClear?(): void;
  clearLabel?: string;
  /** Quiet fields stay transparent until focus; useful inside navigation chrome. */
  variant?: "default" | "quiet";
  class?: string;
  /** Background utility for the complete search field. Defaults to `bg-input`. */
  surfaceClass?: string;
  inputClass?: string;
  inputRef?: (input: Handle) => void;
}
/** A native search input with consistent clear, Escape, and submit behavior. */
export declare function SearchField(props: SearchFieldProps): JSX.Element;
//#endregion
//# sourceMappingURL=search-field.d.mts.map