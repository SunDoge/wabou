import type { JSX } from "solid-js";
import { Select, type SelectOption, type SelectProps } from "./select";

export type NativeSelectOption = SelectOption;

export interface NativeSelectProps
  extends Omit<SelectProps, "motion" | "contentClass" | "contentShadows"> {}

/**
 * Compact Wabou-native select for ordinary forms.
 *
 * Unlike the composable Select skin, this deliberately fixes immediate motion
 * and elevation so callers only own options, value, and form sizing.
 */
export function NativeSelect(props: NativeSelectProps): JSX.Element {
  return (
    <Select
      {...props}
      motion={false}
      contentClass="rounded-md"
      contentShadows={null}
    />
  );
}
