import { $ as OverlayPlaneProvider, $n as createTransition, $t as CodeEditor, At as Center, Bn as LoopOptions, Bt as CollapsiblePresence, C as createActive, Cn as VectorPath, D as createPresence, Dn as WabouClassList, En as ViewProps, Fn as AnimationState, Ft as createHover, Gn as RotationAnimation, Hn as ReactiveAnimation, Ht as Button$1, In as AnimationType, It as FocusResult, Jn as animate, Kn as RotationOptions, Kt as ButtonProps$1, Ln as AnimationValue, Lt as FocusWithinResult, Mn as createAnimationFrame, Mt as LayoutProps, Nn as AnimationControls, Nt as Row, O as Popover, On as WabouStyle, Pn as AnimationOptions, Qn as createRotation, Rn as Easing, Rt as createFocus, S as PressResult, Sn as TextProps, Tn as View, Tt as ModalProps, Un as ReactiveTransition, Vn as PulseOptions, Wn as RepeatType, X as OverlayLayer, Xn as createLoop, Xt as LinkProps, Yn as animateKeyframes, Yt as Link, Zn as createPulse, Zt as createButton, _ as createScrollReset, _n as Text, _t as RippleProps, an as ImageSource, b as ActiveResult, bn as TextInput, bt as Modal, c as createTabs, cn as NetworkImageSource, dn as Path, dt as Notifications, en as CodeEditorProps, fn as PathBuilder, gn as SvgProps, gt as Ripple, hn as Svg, in as ImageProps, jn as AnimationFrameCallback, jt as Column, k as PopoverProps, kt as createMeasuredSize, ln as PasswordInput$1, lt as NotificationRegion, m as createShortcuts, mt as Pulse, nn as IconProps, on as NetworkImage, pn as PathProps, pt as createNotifications, qn as TransitionOptions, qt as ButtonState, rn as Image, s as TabsResult, sn as NetworkImageProps, t as index_d_exports, tn as Icon, tt as createOverlayLayer, un as PasswordInputProps$1, ut as NotificationRegionProps, v as ScrollArea, vn as TextArea, vt as Spin, w as createPress, wn as VectorPathPaint, xn as TextInputProps, xt as ModalControls, y as ScrollAreaProps, yn as TextAreaProps, zn as EasingFunction, zt as createFocusWithin } from "./index-DGA9DZCQ.mjs";
import { Shadow } from "@wabou/core/style";
import { Accessor, JSX, ParentProps } from "solid-js";
import { CalendarDate } from "@internationalized/date";
import { RouterHistory, createMemoryHistory } from "@tanstack/history";
import { AnyRoute, AnyRouter, BaseRootRoute, BaseRoute, RouterConstructorOptions, RouterCore, TrailingSlashOption, notFound, redirect } from "@tanstack/router-core";
export * from "@wabou/core";
export * from "@wabou/core/i18n";
//#region src/components/avatar.d.ts
type AvatarSize = "sm" | "default" | "lg";
interface AvatarProps {
  src?: string;
  alt?: string;
  fallback: string;
  size?: AvatarSize;
  class?: string;
}
declare function Avatar(props: AvatarProps): import("@wabou/core").JSX.Element;
declare function AvatarGroup(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function AvatarGroupCount(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
//#endregion
//#region src/components/config-editor.d.ts
interface ConfigEditorProps extends CodeEditorProps {
  class?: string;
}
/**
 * Experimental native configuration editor. Its Wabou-owned props deliberately
 * hide the editor-core implementation so the backend can evolve independently.
 */
declare function ConfigEditor(props: ConfigEditorProps): JSX.Element;
//#endregion
//#region src/components/date-picker.d.ts
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
  contentShadows?: readonly Shadow[] | null;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  onValueChange?: (value: CalendarDate) => void;
}
/** A shadcn-inspired date picker composed from Wabou Popover and Calendar. */
declare function DatePicker(props: DatePickerProps): JSX.Element;
//#endregion
//#region src/components/dialog.d.ts
interface DialogProps extends Omit<ModalProps, "contentClass"> {
  contentClass?: string;
}
declare function Dialog(props: DialogProps): JSX.Element;
declare function DialogHeader(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function DialogFooter(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function DialogTitle(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function DialogDescription(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
//#endregion
//#region src/components/disclosure.d.ts
interface CollapsibleProps {
  children?: JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  disabled?: boolean;
  reducedMotion?: boolean;
  onOpenChange?: (open: boolean) => void;
  class?: string;
}
declare function Collapsible(props: CollapsibleProps): import("@wabou/core").JSX.Element;
declare function CollapsibleTrigger(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function CollapsibleContent(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
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
declare function Accordion(props: AccordionProps): import("@wabou/core").JSX.Element;
declare function AccordionItem(props: {
  value: string;
  disabled?: boolean;
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function AccordionTrigger(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function AccordionContent(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
//#endregion
//#region src/components/display.d.ts
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
//#region src/components/forms.d.ts
type FieldOrientation = "vertical" | "horizontal";
declare function Field(props: {
  children?: JSX.Element;
  orientation?: FieldOrientation;
  invalid?: boolean;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function FieldGroup(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function FieldLabel(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function FieldContent(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function FieldDescription(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function FieldError(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function InputGroup(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function InputGroupInput(props: InputProps): import("@wabou/core").JSX.Element;
declare function InputGroupText(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function InputGroupButton(props: ButtonProps): import("@wabou/core").JSX.Element;
declare function InputGroupTextArea(props: TextAreaProps & {
  class?: string;
}): import("@wabou/core").JSX.Element;
//#endregion
//#region src/components/layout.d.ts
declare function Empty(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function EmptyHeader(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function EmptyMedia(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function EmptyTitle(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function EmptyDescription(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function EmptyContent(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function ButtonGroup(props: {
  children?: JSX.Element;
  orientation?: "horizontal" | "vertical";
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function ButtonGroupText(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
/**
 * A horizontal primary/aside boundary with explicit flex shrink semantics.
 * Use `SplitPaneMain` for the elastic region and `SplitPaneAside` for a
 * class-sized fixed rail. Both regions clip at their own boundary, so content
 * cannot paint across the divider or a rounded parent clip.
 */
declare function SplitPane(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function SplitPaneMain(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function SplitPaneAside(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
//#endregion
//#region src/components/select.d.ts
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
  contentShadows?: readonly Shadow[] | null;
  onValueChange?: (value: string) => void;
  onOpenChange?: (open: boolean) => void;
}
/** Shadcn-inspired single Select backed by Wabou-native interaction state. */
declare function Select(props: SelectProps): JSX.Element;
//#endregion
//#region src/components/selection.d.ts
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
//#region src/components/slider.d.ts
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
//#region src/components/tabs.d.ts
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
//#region src/components/theme.d.ts
type ComponentsTheme = "light" | "dark";
type ComponentsElevation = "raised" | "floating" | "modal";
/**
 * Native elevation recipes adapted from gpui-component. Wabou and GPUI both
 * pass standard deviation directly to their renderer, so these values should
 * not use CSS's doubled blur radius. Floating surfaces also carry a subtle
 * foreground-colored ring: black in light mode, white in dark mode.
 */
declare function componentsElevation(theme: ComponentsTheme, elevation: ComponentsElevation): Shadow[];
type ComponentsProviderProps = ParentProps<{
  theme?: ComponentsTheme;
}>;
declare function ComponentsProvider(props: ComponentsProviderProps): JSX.Element;
declare function useComponentsTheme(): () => ComponentsTheme;
//#endregion
//#region src/components/title-bar.d.ts
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
//#region src/components/index.d.ts
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
  shadows?: readonly import("@wabou/core/style").Shadow[] | null;
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
interface InputProps extends TextInputProps {
  class?: string;
}
/** A plain-text input. Secrets must use {@link PasswordInput}. */
declare function Input(props: InputProps): JSX.Element;
interface PasswordInputProps extends PasswordInputProps$1 {
  class?: string;
}
/** A native secret input whose value never crosses into JavaScript. */
declare function PasswordInput(props: PasswordInputProps): JSX.Element;
interface TextAreaProps$1 extends TextAreaProps {
  class?: string;
}
declare function TextArea$1(props: TextAreaProps$1): JSX.Element;
interface SwitchProps {
  checked?: boolean;
  defaultChecked?: boolean;
  disabled?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  label?: string;
  "aria-label"?: string;
}
declare function Switch(props: SwitchProps): JSX.Element;
declare function Progress(props: {
  value?: number;
  label?: string;
  class?: string;
}): JSX.Element;
//#endregion
//#region src/router/data.d.ts
type WabouDataRouter<TRouteTree extends AnyRoute = AnyRoute> = RouterCore<TRouteTree, TrailingSlashOption, boolean, RouterHistory>;
/** Create a TanStack Router Core instance adapted to Solid 2 and native history. */
declare function createDataRouter<TRouteTree extends AnyRoute, TTrailingSlash extends TrailingSlashOption = "never", TStructuralSharing extends boolean = false, THistory extends RouterHistory = RouterHistory, TDehydrated extends Record<string, any> = Record<string, any>>(options: RouterConstructorOptions<TRouteTree, TTrailingSlash, TStructuralSharing, THistory, TDehydrated>): RouterCore<TRouteTree, TTrailingSlash, TStructuralSharing, THistory, TDehydrated>;
interface RouterProviderProps {
  router: AnyRouter;
  fallback?: JSX.Element;
}
/** Own router lifecycle and render its current native component branch. */
declare function RouterProvider(props: RouterProviderProps): JSX.Element;
declare function useRouter(): AnyRouter;
declare function useRouterState<T>(selector: (router: AnyRouter) => T): Accessor<T>;
declare function useNavigate(): AnyRouter["navigate"];
declare function useLocation(): Accessor<AnyRouter["state"]["location"]>;
declare function useParams<T extends Record<string, string> = Record<string, string>>(): Accessor<T>;
declare function useLoaderData<T = unknown>(): Accessor<T | undefined>;
//#endregion
export { Accordion, AccordionContent, AccordionItem, AccordionProps, AccordionTrigger, AccordionType, type ActiveResult, Alert, AnimationControls, type AnimationFrameCallback, AnimationOptions, AnimationState, AnimationType, AnimationValue, Avatar, AvatarGroup, AvatarGroupCount, AvatarProps, AvatarSize, Badge, BadgeProps, BaseRootRoute, BaseRoute, Button, ButtonGroup, ButtonGroupText, ButtonProps, ButtonSize, ButtonVariant, Calendar, CalendarDate, CalendarLabels, CalendarProps, Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle, Center, Checkbox, type CheckboxProps, CodeEditor, type CodeEditorProps, Collapsible, CollapsibleContent, CollapsiblePresence, CollapsibleProps, CollapsibleTrigger, Column, type ComponentsElevation, ComponentsProvider, type ComponentsProviderProps, type ComponentsTheme, ConfigEditor, ConfigEditorProps, DatePicker, DatePickerProps, Dialog, type ModalControls as DialogControls, DialogDescription, DialogFooter, DialogHeader, DialogProps, DialogTitle, Easing, EasingFunction, Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle, Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldOrientation, type FocusResult, type FocusWithinResult, Fps, FpsProps, Icon, type IconProps, Image, type ImageProps, type ImageSource, Input, InputGroup, InputGroupButton, InputGroupInput, InputGroupText, InputGroupTextArea, InputProps, Kbd, KbdGroup, type LayoutProps, LoopOptions, Modal, type ModalProps, NetworkImage, type NetworkImageProps, type NetworkImageSource, NotificationRegion, type NotificationRegionProps, type Notifications, type OverlayLayer, OverlayPlaneProvider, PasswordInput, PasswordInputProps, Path, PathBuilder, type PathProps, Popover, type PopoverProps, type PressResult, Button$1 as PrimitiveButton, type ButtonProps$1 as PrimitiveButtonProps, type ButtonState as PrimitiveButtonState, Link as PrimitiveLink, type LinkProps as PrimitiveLinkProps, PasswordInput$1 as PrimitivePasswordInput, type PasswordInputProps$1 as PrimitivePasswordInputProps, TextArea as PrimitiveTextArea, type TextAreaProps as PrimitiveTextAreaProps, TextInput as PrimitiveTextInput, type TextInputProps as PrimitiveTextInputProps, Progress, Pulse, PulseOptions, RadioGroup, RadioGroupItem, type RadioGroupItemProps, type RadioGroupProps, ReactiveAnimation, ReactiveTransition, RepeatType, Ripple, type RippleProps, RotationAnimation, RotationOptions, RouterProvider, RouterProviderProps, Row, ScrollArea, type ScrollAreaProps, Select, SelectOption, SelectProps, Separator, Skeleton, Slider, SliderProps, Spin, Spinner, SplitPane, SplitPaneAside, SplitPaneMain, Svg, type SvgProps, Switch, SwitchProps, Tabs, TabsContent, TabsList, type TabsProps, type TabsResult, TabsTrigger, type TabsTriggerProps, Text, TextArea$1 as TextArea, TextAreaProps$1 as TextAreaProps, type TextProps, TitleBar, TitleBarDragRegion, TitleBarDragRegionProps, TitleBarProps, Toggle, ToggleGroup, ToggleGroupItem, type ToggleGroupItemProps, type ToggleGroupProps, type ToggleProps, TransitionOptions, type VectorPath, type VectorPathPaint, View, type ViewProps, type WabouClassList, WabouDataRouter, type WabouStyle, animate, animateKeyframes, componentsElevation, createActive, createAnimationFrame, createButton, createDataRouter, createFocus, createFocusWithin, createHover, createLoop, createMeasuredSize, createMemoryHistory, createNotifications, createOverlayLayer, createPresence, createPress, createPulse, createRotation, createScrollReset, createShortcuts, createTabs, createTransition, nextAccordionValue, notFound, index_d_exports as primitives, redirect, titleBarClass, titleBarDragRegionLayoutStyle, titleBarLayoutStyle, useComponentsTheme, useLoaderData, useLocation, useNavigate, useParams, useRouter, useRouterState };
//# sourceMappingURL=index.d.mts.map