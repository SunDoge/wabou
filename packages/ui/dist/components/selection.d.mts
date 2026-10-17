import { JSX } from "solid-js";
//#region src/components/selection.d.ts
export type SelectionControlSize = "sm" | "default" | "lg";
export interface CheckboxProps {
  checked?: boolean;
  defaultChecked?: boolean;
  indeterminate?: boolean;
  disabled?: boolean;
  size?: SelectionControlSize;
  label?: string;
  "aria-label"?: string;
  class?: string;
  onCheckedChange?: (checked: boolean) => void;
}
export declare function Checkbox(props: CheckboxProps): JSX.Element;
export interface RadioGroupProps {
  value?: string;
  defaultValue?: string;
  disabled?: boolean;
  orientation?: "horizontal" | "vertical";
  appearance?: "radio" | "segment";
  size?: SelectionControlSize;
  loop?: boolean;
  "aria-label"?: string;
  class?: string;
  children?: JSX.Element;
  onValueChange?: (value: string) => void;
}
export declare function RadioGroup(props: RadioGroupProps): JSX.Element;
export interface RadioGroupItemProps {
  value: string;
  label?: string;
  "aria-label"?: string;
  disabled?: boolean;
  size?: SelectionControlSize;
  class?: string;
}
export declare function RadioGroupItem(props: RadioGroupItemProps): JSX.Element;
export interface ToggleProps {
  pressed?: boolean;
  defaultPressed?: boolean;
  disabled?: boolean;
  variant?: "default" | "outline";
  size?: "sm" | "default" | "lg";
  "aria-label"?: string;
  class?: string;
  children?: JSX.Element;
  onPressedChange?: (pressed: boolean) => void;
}
export declare function Toggle(props: ToggleProps): JSX.Element;
interface ToggleGroupBaseProps {
  disabled?: boolean;
  "aria-label"?: string;
  variant?: "default" | "outline";
  size?: "sm" | "default" | "lg";
  spacing?: 0 | 1 | 2;
  /** Join items into one clipped control surface owned by the group. */
  segmented?: boolean;
  loop?: boolean;
  class?: string;
  children?: JSX.Element;
}
export type ToggleGroupProps = ToggleGroupBaseProps & ({
  type?: "single";
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
} | {
  type: "multiple";
  value?: readonly string[];
  defaultValue?: readonly string[];
  onValueChange?: (value: readonly string[]) => void;
});
/** Shadcn-style single-value toggle group with native roving focus. */
export declare function ToggleGroup(props: ToggleGroupProps): JSX.Element;
export interface ToggleGroupItemProps {
  value: string;
  "aria-label"?: string;
  disabled?: boolean;
  variant?: "default" | "outline" | "accent";
  size?: "sm" | "default" | "lg";
  class?: string;
  children?: JSX.Element;
}
export declare function ToggleGroupItem(props: ToggleGroupItemProps): JSX.Element;
//#endregion
export {};
//# sourceMappingURL=selection.d.mts.map