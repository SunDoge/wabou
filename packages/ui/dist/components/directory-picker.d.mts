import { InputProps } from "./input.mjs";
import { PickDirectoryOptions } from "@wabou/core";
import { JSX } from "solid-js";
//#region src/components/directory-picker.d.ts
export interface DirectoryPickerProps extends Omit<InputProps, "class" | "onInput" | "value"> {
  value: string;
  onValueChange: (value: string) => void;
  /** Options forwarded to the native picker. `directory` defaults to `value`. */
  dialogOptions?: PickDirectoryOptions;
  browseLabel?: string;
  pendingLabel?: string;
  browseAriaLabel?: string;
  class?: string;
  inputClass?: string;
  buttonClass?: string;
  onBrowseError?: (error: unknown) => void;
  /** Called only after the native picker commits a directory selection. */
  onBrowseSelect?: (value: string) => void;
}
/** A controlled path input paired with the operating system directory picker. */
export declare function DirectoryPicker(props: DirectoryPickerProps): JSX.Element;
//#endregion
//# sourceMappingURL=directory-picker.d.mts.map