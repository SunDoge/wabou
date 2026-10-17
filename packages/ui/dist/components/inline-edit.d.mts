import { JSX } from "solid-js";
//#region src/components/inline-edit.d.ts
export interface InlineEditProps {
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  onCommit?: (value: string) => void;
  onCancel?: () => void;
  disabled?: boolean;
  placeholder?: string;
  /** Accessible name for the editor and its trigger. */
  "aria-label": string;
  class?: string;
  displayClass?: string;
  inputClass?: string;
}
/** Compact rename interaction with explicit Enter, Escape, and blur behavior. */
export declare function InlineEdit(props: InlineEditProps): JSX.Element;
//#endregion
//# sourceMappingURL=inline-edit.d.mts.map