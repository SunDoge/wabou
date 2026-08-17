import { ButtonProps as ButtonProps$1, CodeEditorProps, ModalControls, ModalProps, TextAreaProps as TextAreaProps$1, ViewProps, WabouStyle } from "@wabou/primitives";
import { JSX, ParentProps } from "solid-js";
import { CalendarDate } from "@internationalized/date";
import { JSX as JSX$1 } from "@solidjs/web";
//#region src/display.d.ts
declare function Skeleton(props: {
  class?: string;
}): JSX.Element;
declare function Spinner(props: {
  label?: string;
  class?: string;
}): JSX.Element;
declare function Kbd(props: {
  class?: string;
  children?: JSX.Element;
}): JSX.Element;
declare function KbdGroup(props: {
  class?: string;
  children?: JSX.Element;
}): JSX.Element;
//#endregion
//#region src/avatar.d.ts
type AvatarSize = "sm" | "default" | "lg";
interface AvatarProps {
  src?: string;
  alt?: string;
  fallback: string;
  size?: AvatarSize;
  class?: string;
}
declare function Avatar(props: AvatarProps): import("@wabou/solid-renderer").JSX.Element;
declare function AvatarGroup(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function AvatarGroupCount(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
//#endregion
//#region src/config-editor.d.ts
interface ConfigEditorProps extends CodeEditorProps {
  class?: string;
}
/**
 * Experimental native configuration editor. Its Wabou-owned props deliberately
 * hide the editor-core implementation so the backend can evolve independently.
 */
declare function ConfigEditor(props: ConfigEditorProps): JSX.Element;
//#endregion
//#region src/date-picker.d.ts
interface CalendarProps {
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
interface CalendarLabels {
  previousMonth: string;
  nextMonth: string;
  today: string;
  selectToday: string;
}
/** A Wabou-native calendar using @internationalized/date for date arithmetic. */
declare function Calendar(props: CalendarProps): JSX.Element;
interface DatePickerProps extends Omit<CalendarProps, "aria-label"> {
  "aria-label": string;
  placeholder?: string;
  class?: string;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  onValueChange?: (value: CalendarDate) => void;
}
/** A shadcn-inspired date picker composed from Wabou Popover and Calendar. */
declare function DatePicker(props: DatePickerProps): JSX.Element;
//#endregion
//#region src/dialog.d.ts
interface DialogProps extends Omit<ModalProps, "contentClass"> {
  contentClass?: string;
}
declare function Dialog(props: DialogProps): JSX.Element;
declare function DialogHeader(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function DialogFooter(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function DialogTitle(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function DialogDescription(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
//#endregion
//#region src/disclosure.d.ts
interface CollapsibleProps {
  children?: JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  disabled?: boolean;
  reducedMotion?: boolean;
  onOpenChange?: (open: boolean) => void;
  class?: string;
}
declare function Collapsible(props: CollapsibleProps): import("@wabou/solid-renderer").JSX.Element;
declare function CollapsibleTrigger(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function CollapsibleContent(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
type AccordionType = "single" | "multiple";
type AccordionValue = string | readonly string[];
declare function nextAccordionValue(current: AccordionValue, type: AccordionType, item: string, collapsible?: boolean): AccordionValue;
interface AccordionProps {
  children?: JSX.Element;
  type?: AccordionType;
  value?: AccordionValue;
  defaultValue?: AccordionValue;
  collapsible?: boolean;
  disabled?: boolean;
  reducedMotion?: boolean;
  onValueChange?: (value: AccordionValue) => void;
  class?: string;
}
declare function Accordion(props: AccordionProps): import("@wabou/solid-renderer").JSX.Element;
declare function AccordionItem(props: {
  value: string;
  disabled?: boolean;
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function AccordionTrigger(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function AccordionContent(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
//#endregion
//#region src/forms.d.ts
type FieldOrientation = "vertical" | "horizontal";
declare function Field(props: {
  children?: JSX.Element;
  orientation?: FieldOrientation;
  invalid?: boolean;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function FieldGroup(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function FieldLabel(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function FieldContent(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function FieldDescription(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function FieldError(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function InputGroup(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function InputGroupInput(props: InputProps): import("@wabou/solid-renderer").JSX.Element;
declare function InputGroupText(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function InputGroupButton(props: ButtonProps): import("@wabou/solid-renderer").JSX.Element;
declare function InputGroupTextArea(props: TextAreaProps$1 & {
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
//#endregion
//#region src/layout.d.ts
declare function Empty(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function EmptyHeader(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function EmptyMedia(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function EmptyTitle(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function EmptyDescription(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function EmptyContent(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function ButtonGroup(props: {
  children?: JSX.Element;
  orientation?: "horizontal" | "vertical";
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
declare function ButtonGroupText(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/solid-renderer").JSX.Element;
//#endregion
//#region src/select.d.ts
interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}
interface SelectProps {
  options: readonly SelectOption[];
  value?: string;
  defaultValue?: string;
  open?: boolean;
  defaultOpen?: boolean;
  disabled?: boolean;
  placeholder?: string;
  "aria-label": string;
  class?: string;
  contentClass?: string;
  onValueChange?: (value: string) => void;
  onOpenChange?: (open: boolean) => void;
}
/** Shadcn-inspired single Select backed by Wabou-native interaction state. */
declare function Select(props: SelectProps): JSX.Element;
//#endregion
//#region src/slider.d.ts
interface SliderProps {
  value?: number;
  defaultValue?: number;
  min?: number;
  max?: number;
  step?: number;
  disabled?: boolean;
  label: string;
  valueText?: (value: number) => string;
  onValueChange?: (value: number) => void;
  class?: string;
}
declare function Slider(props: SliderProps): JSX.Element;
//#endregion
//#region src/title-bar.d.ts
declare const titleBarClass = "border-b border-subtle";
interface TitleBarProps extends ViewProps {
  class?: string;
  children?: JSX.Element;
}
declare const titleBarLayoutStyle: {
  readonly display: "flex";
  readonly "flex-direction": "row";
  readonly "align-items": "center";
  readonly "flex-shrink": 0;
  readonly height: "40px";
};
declare const titleBarDragRegionLayoutStyle: {
  readonly display: "flex";
  readonly "flex-direction": "row";
  readonly "align-items": "center";
  readonly "flex-grow": 1;
  readonly "flex-shrink": 1;
  readonly "flex-basis": "0%";
  readonly height: "100%";
};
/** Layout shell for an application-owned title bar. */
declare function TitleBar(props: TitleBarProps): JSX.Element;
interface TitleBarDragRegionProps extends Omit<ViewProps, "onPointerDown" | "onDblClick"> {
  class?: string;
  style?: WabouStyle;
  children?: JSX.Element;
}
/** Explicit non-interactive region that moves the native window. */
declare function TitleBarDragRegion(props: TitleBarDragRegionProps): JSX.Element;
//#endregion
//#region src/selection.d.ts
interface CheckboxProps {
  checked?: boolean;
  defaultChecked?: boolean;
  indeterminate?: boolean;
  disabled?: boolean;
  label?: string;
  "aria-label"?: string;
  class?: string;
  onCheckedChange?: (checked: boolean) => void;
}
declare function Checkbox(props: CheckboxProps): JSX.Element;
interface RadioGroupProps {
  value?: string;
  defaultValue?: string;
  disabled?: boolean;
  "aria-label"?: string;
  class?: string;
  children?: JSX.Element;
  onValueChange?: (value: string) => void;
}
declare function RadioGroup(props: RadioGroupProps): JSX.Element;
interface RadioGroupItemProps {
  value: string;
  label?: string;
  disabled?: boolean;
  class?: string;
}
declare function RadioGroupItem(props: RadioGroupItemProps): JSX.Element;
interface ToggleProps {
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
declare function Toggle(props: ToggleProps): JSX.Element;
interface ToggleGroupProps {
  type: "single";
  value?: string;
  defaultValue?: string;
  disabled?: boolean;
  "aria-label"?: string;
  class?: string;
  children?: JSX.Element;
  onValueChange?: (value: string) => void;
}
/** Shadcn-style single-value toggle group with native roving focus. */
declare function ToggleGroup(props: ToggleGroupProps): JSX.Element;
interface ToggleGroupItemProps {
  value: string;
  disabled?: boolean;
  class?: string;
  children?: JSX.Element;
}
declare function ToggleGroupItem(props: ToggleGroupItemProps): JSX.Element;
//#endregion
//#region src/tabs.d.ts
interface TabsProps {
  value?: string;
  defaultValue?: string;
  orientation?: "horizontal" | "vertical";
  class?: string;
  children?: JSX.Element;
  onValueChange?: (value: string) => void;
}
declare function Tabs(props: TabsProps): JSX.Element;
declare function TabsList(props: {
  variant?: "default" | "line";
  "aria-label"?: string;
  class?: string;
  children?: JSX.Element;
}): JSX.Element;
interface TabsTriggerProps {
  value: string;
  disabled?: boolean;
  class?: string;
  children?: JSX.Element;
}
declare function TabsTrigger(props: TabsTriggerProps): JSX.Element;
declare function TabsContent(props: {
  value: string;
  class?: string;
  children?: JSX.Element;
}): JSX.Element;
//#endregion
//#region src/theme.d.ts
type ComponentsTheme = "light" | "dark";
type ComponentsProviderProps = ParentProps<{
  theme?: ComponentsTheme;
}>;
declare function ComponentsProvider(props: ComponentsProviderProps): JSX.Element;
declare function useComponentsTheme(): () => ComponentsTheme;
//#endregion
//#region src/index.d.ts
type ButtonVariant = "default" | "secondary" | "outline" | "ghost" | "destructive";
type ButtonSize = "sm" | "default" | "lg" | "icon";
interface ButtonProps extends Omit<ButtonProps$1, "variant" | "tone"> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  class?: string;
  style?: ButtonProps$1["style"];
}
declare function Button(props: ButtonProps): JSX.Element;
interface BadgeProps {
  children?: JSX.Element;
  variant?: "default" | "secondary" | "outline" | "success" | "destructive";
  class?: string;
}
declare function Badge(props: BadgeProps): JSX.Element;
interface FpsProps {
  /** Controlled FPS value. When omitted, the component measures host frames. */
  value?: number;
  /** Text displayed after the value. Set to an empty string for value only. */
  label?: string;
  /** FPS at or above this value uses the success treatment. */
  goodAt?: number;
  /** FPS below this value uses the destructive treatment. */
  warningBelow?: number;
  class?: string;
}
/** Live host frame-rate indicator with sensible performance thresholds. */
declare function Fps(props: FpsProps): JSX.Element;
declare function Card(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
declare function CardHeader(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
declare function CardTitle(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
declare function CardDescription(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
declare function CardContent(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
declare function CardFooter(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
declare function Separator(props: {
  orientation?: "horizontal" | "vertical";
  class?: string;
}): JSX.Element;
declare function Alert(props: {
  title: string;
  children?: JSX.Element;
  variant?: "default" | "destructive";
  class?: string;
}): JSX.Element;
interface InputProps {
  type?: "text" | "password";
  value?: string;
  placeholder?: string;
  disabled?: boolean;
  class?: string;
  ref?: JSX$1.InputHTMLAttributes<HTMLInputElement>["ref"];
  onInput?: (event: {
    currentTarget: {
      value: string;
    };
  }) => void;
  onKeyDown?: JSX$1.InputHTMLAttributes<HTMLInputElement>["onKeyDown"];
}
declare function Input(props: InputProps): JSX.Element;
interface TextAreaProps extends TextAreaProps$1 {
  class?: string;
}
declare function TextArea(props: TextAreaProps): JSX.Element;
interface SwitchProps {
  checked?: boolean;
  defaultChecked?: boolean;
  disabled?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  label?: string;
}
declare function Switch(props: SwitchProps): JSX.Element;
declare function Progress(props: {
  value?: number;
  class?: string;
}): JSX.Element;
//#endregion
export { Accordion, AccordionContent, AccordionItem, AccordionProps, AccordionTrigger, AccordionType, Alert, Avatar, AvatarGroup, AvatarGroupCount, AvatarProps, AvatarSize, Badge, BadgeProps, Button, ButtonGroup, ButtonGroupText, ButtonProps, ButtonSize, ButtonVariant, Calendar, CalendarDate, CalendarLabels, CalendarProps, Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle, Checkbox, type CheckboxProps, Collapsible, CollapsibleContent, CollapsibleProps, CollapsibleTrigger, ComponentsProvider, type ComponentsProviderProps, type ComponentsTheme, ConfigEditor, ConfigEditorProps, DatePicker, DatePickerProps, Dialog, type ModalControls as DialogControls, DialogDescription, DialogFooter, DialogHeader, DialogProps, DialogTitle, Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle, Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldOrientation, Fps, FpsProps, Input, InputGroup, InputGroupButton, InputGroupInput, InputGroupText, InputGroupTextArea, InputProps, Kbd, KbdGroup, Progress, RadioGroup, RadioGroupItem, type RadioGroupItemProps, type RadioGroupProps, Select, SelectOption, SelectProps, Separator, Skeleton, Slider, SliderProps, Spinner, Switch, SwitchProps, Tabs, TabsContent, TabsList, type TabsProps, TabsTrigger, type TabsTriggerProps, TextArea, TextAreaProps, TitleBar, TitleBarDragRegion, TitleBarDragRegionProps, TitleBarProps, Toggle, ToggleGroup, ToggleGroupItem, type ToggleGroupItemProps, type ToggleGroupProps, type ToggleProps, nextAccordionValue, titleBarClass, titleBarDragRegionLayoutStyle, titleBarLayoutStyle, useComponentsTheme };
//# sourceMappingURL=index.d.mts.map