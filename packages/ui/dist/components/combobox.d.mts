import { CommandItem } from "./command.mjs";
import { PopupMotionProps } from "./popover.mjs";
import { PickerTriggerVariant } from "./select-semantics.mjs";
import { Shadow } from "@wabou/core/style";
import { JSX } from "solid-js";
//#region src/components/combobox.d.ts
export interface ComboboxOption extends CommandItem {
  value: string;
}
export interface ComboboxProps extends PopupMotionProps {
  options: readonly ComboboxOption[];
  "aria-label": string;
  value?: string;
  defaultValue?: string;
  open?: boolean;
  defaultOpen?: boolean;
  disabled?: boolean;
  placeholder?: string;
  searchPlaceholder?: string;
  emptyText?: string;
  class?: string;
  triggerVariant?: PickerTriggerVariant;
  contentClass?: string;
  contentShadows?: readonly Shadow[] | null;
  onValueChange?: (value: string) => void;
  onOpenChange?: (open: boolean) => void;
}
/** A searchable single-value picker built from Popover and Command. */
export declare function Combobox(props: ComboboxProps): JSX.Element;
//#endregion
//# sourceMappingURL=combobox.d.mts.map