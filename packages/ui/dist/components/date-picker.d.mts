import { PopupMotionProps } from "./popover.mjs";
import { Shadow } from "@wabou/core/style";
import { JSX } from "solid-js";
import { CalendarDate } from "@internationalized/date";
//#region src/components/date-picker.d.ts
export interface CalendarProps {
  value?: CalendarDate;
  defaultValue?: CalendarDate;
  minValue?: CalendarDate;
  maxValue?: CalendarDate;
  disabled?: boolean;
  isDateUnavailable?: (date: CalendarDate) => boolean;
  locale?: string;
  labels?: Partial<CalendarLabels>;
  "aria-label"?: string;
  onValueChange?: (value: CalendarDate) => void;
}
export interface CalendarLabels {
  previousMonth: string;
  nextMonth: string;
  today: string;
  selectToday: string;
}
/** A Wabou-native calendar using @internationalized/date for date arithmetic. */
export declare function Calendar(props: CalendarProps): JSX.Element;
export interface DatePickerProps extends Omit<CalendarProps, "aria-label">, PopupMotionProps {
  "aria-label": string;
  placeholder?: string;
  class?: string;
  contentShadows?: readonly Shadow[] | null;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  onValueChange?: (value: CalendarDate) => void;
}
/** A shadcn-inspired date picker composed from Wabou Popover and Calendar. */
export declare function DatePicker(props: DatePickerProps): JSX.Element;
//#endregion
export { CalendarDate };
//# sourceMappingURL=date-picker.d.mts.map