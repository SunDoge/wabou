import { PopupMotionProps } from "./popover.mjs";
import { PickerTriggerVariant } from "./select-semantics.mjs";
import { Shadow } from "@wabou/core/style";
import { JSX } from "solid-js";
import { Handle } from "@wabou/core/renderer";
//#region src/components/select.d.ts
export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}
export interface SelectProps extends PopupMotionProps {
  options: readonly SelectOption[];
  value?: string;
  defaultValue?: string;
  open?: boolean;
  defaultOpen?: boolean;
  disabled?: boolean;
  /** Receives the native trigger handle for explicit label/focus composition. */
  ref?: (node: Handle) => void;
  placeholder?: string;
  "aria-label": string;
  class?: string;
  triggerVariant?: PickerTriggerVariant;
  contentClass?: string;
  contentShadows?: readonly Shadow[] | null;
  onValueChange?: (value: string) => void;
  onOpenChange?: (open: boolean) => void;
}
/** Shadcn-inspired single Select backed by Wabou-native interaction state. */
export declare function Select(props: SelectProps): JSX.Element;
//#endregion
//# sourceMappingURL=select.d.mts.map