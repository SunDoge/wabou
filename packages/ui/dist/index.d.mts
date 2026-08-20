import { $n as PulseOptions, An as TextArea, Bn as WabouStyle, Bt as createKeyedSelection, C as createActive, Cn as Path, D as createPresence, Dn as Svg, Et as ModalProps, Fn as VectorPath, Ft as Column, Gn as AnimationControls, Gt as createFormDraft, Ht as FormDraftErrors, I as Placement, In as VectorPathPaint, It as LayoutProps, Jn as AnimationType, Jt as FocusResult, Kn as AnimationOptions, L as PointAnchor, Ln as View, Lt as Row, Mn as TextInput, Mt as createContainerMatch, Nn as TextInputProps, Nt as createMeasuredSize, O as Popover$1, On as SvgProps, Ot as ContainerMatch, Pn as TextProps, Pt as Center, Qn as LoopOptions, Qt as CollapsiblePresence, Rn as ViewProps, Rt as KeyedSelection, S as PressResult, Sn as PasswordInputProps$1, St as ModalControls, Tn as PathProps, Un as AnimationFrameCallback, Ut as FormDraftFieldUpdater, Vt as FormDraft, Wn as createAnimationFrame, Wt as FormDraftOptions, Xn as Easing, Xt as createFocus, Yn as AnimationValue, Yt as FocusWithinResult, Z as OverlayLayer, Zn as EasingFunction, Zt as createFocusWithin, _ as createScrollReset, _n as ImageSource, _t as Ripple, an as ButtonState, ar as TransitionOptions, at as NotificationDismissReason, b as ActiveResult, bn as NetworkImageSource, c as createTabs, cn as LinkProps, cr as createLoop, ct as NotificationPlacement, dn as CodeEditor, dr as createTransition, dt as NotificationRegionProps, en as Button$1, er as ReactiveAnimation, et as OverlayPlaneProvider, fn as CodeEditorProps, ft as Notifications, gn as ImageProps, hn as Image, ht as Pulse, in as ButtonProps$1, ir as RotationOptions, jn as TextAreaProps, k as PopoverProps$1, kn as Text, kt as ContainerSizeQuery, ln as createButton, lr as createPulse, m as createShortcuts, mn as IconProps, mt as createNotifications, nr as RepeatType, nt as createOverlayLayer, or as animate, pn as Icon, qn as AnimationState, qt as createHover, rr as RotationAnimation, s as TabsResult, sn as Link, sr as animateKeyframes, t as index_d_exports, tr as ReactiveTransition, ur as createRotation, ut as NotificationRegion, v as ScrollArea, vn as NetworkImage, vt as RippleProps, w as createPress, wn as PathBuilder, xn as PasswordInput$1, xt as Modal, y as ScrollAreaProps, yn as NetworkImageProps, yt as Spin, zn as WabouClassList, zt as KeyedSelectionOptions } from "./index-Cz-vb6k4.mjs";
import { PickDirectoryOptions } from "@wabou/core";
import { Shadow } from "@wabou/core/style";
import { Accessor, JSX, ParentProps } from "solid-js";
import { Handle, WabouPointerEvent } from "@wabou/core/renderer";
import { CalendarDate } from "@internationalized/date";
import { RouterHistory, createMemoryHistory } from "@tanstack/history";
import { AnyRoute, AnyRouter, BaseRootRoute, BaseRoute, RouterConstructorOptions, RouterCore, TrailingSlashOption, notFound, redirect } from "@tanstack/router-core";
export * from "@wabou/core";
export * from "@wabou/core/i18n";
//#region src/components/button.d.ts
type ButtonVariant = "default" | "secondary" | "outline" | "ghost" | "destructive";
type ButtonSize = "sm" | "default" | "lg" | "icon";
interface ButtonProps extends Omit<ButtonProps$1, "variant" | "tone"> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  class?: string;
  style?: ButtonProps$1["style"];
}
declare function Button(props: ButtonProps): JSX.Element;
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
/**
 * The shrinking, independently scrollable region between a dialog's fixed
 * header and footer. The dialog surface must have a bounded or maximum height.
 */
declare function DialogScrollBody(props: {
  children?: JSX.Element;
  class?: string;
  contentClass?: string;
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
//#region src/components/alert-dialog.d.ts
interface AlertDialogProps extends Omit<DialogProps, "children" | "contentRole"> {
  children?: JSX.Element | ((controls: ModalControls) => JSX.Element);
}
/**
 * A blocking confirmation dialog. Backdrop dismissal is disabled by default so
 * every close is an intentional cancel, confirmation, or Escape action.
 */
declare function AlertDialog(props: AlertDialogProps): JSX.Element;
declare function AlertDialogAction(props: ButtonProps): JSX.Element;
declare function AlertDialogCancel(props: ButtonProps): JSX.Element;
declare const AlertDialogHeader: typeof DialogHeader;
declare const AlertDialogFooter: typeof DialogFooter;
declare const AlertDialogTitle: typeof DialogTitle;
declare const AlertDialogDescription: typeof DialogDescription;
//#endregion
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
//#region src/components/menu-state.d.ts
interface MenuStateItem {
  id: string;
  label: string;
  disabled?: boolean;
}
type MenuMove = "first" | "last" | "next" | "previous";
/** Resolve one keyboard move without coupling menu state to rendering. */
declare function moveMenuHighlight(items: readonly MenuStateItem[], current: string | undefined, move: MenuMove): string | undefined;
//#endregion
//#region src/components/command-state.d.ts
interface CommandStateItem extends MenuStateItem {
  keywords?: readonly string[];
}
declare function filterCommandItems<T extends CommandStateItem>(items: readonly T[], query: string): T[];
declare function reconcileCommandHighlight<T extends CommandStateItem>(items: readonly T[], highlighted: string | undefined): string | undefined;
//#endregion
//#region src/components/command.d.ts
interface CommandItem extends CommandStateItem {
  description?: string;
  onSelect?: () => void;
}
interface CommandProps {
  items: readonly CommandItem[];
  "aria-label": string;
  query?: string;
  defaultQuery?: string;
  placeholder?: string;
  emptyText?: string;
  class?: string;
  listClass?: string;
  onQueryChange?: (query: string) => void;
  onAction?: (id: string) => void;
  onDismiss?: () => void;
  inputRef?: (node: Handle) => void;
}
/** Searchable command list whose filtering and keyboard behavior are host-independent. */
declare function Command(props: CommandProps): JSX.Element;
//#endregion
//#region src/components/combobox.d.ts
interface ComboboxOption extends CommandItem {
  value: string;
}
interface ComboboxProps {
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
  contentClass?: string;
  contentShadows?: readonly Shadow[] | null;
  onValueChange?: (value: string) => void;
  onOpenChange?: (open: boolean) => void;
}
/** A searchable single-value picker built from Popover and Command. */
declare function Combobox(props: ComboboxProps): JSX.Element;
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
//#region src/components/dropdown-menu.d.ts
interface DropdownMenuItem extends MenuStateItem {
  description?: string;
  destructive?: boolean;
  separatorBefore?: boolean;
  onSelect?: () => void;
}
interface DropdownMenuProps {
  trigger(props: {
    ref(node: Handle): void;
    onClick(event: {
      stopPropagation(): void;
    }): void;
    onKeyDown(event: {
      key: string;
      preventDefault(): void;
      stopPropagation(): void;
    }): void;
    "aria-haspopup": "menu";
    "aria-expanded": boolean;
  }): JSX.Element;
  items: readonly DropdownMenuItem[];
  "aria-label": string;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  onAction?: (id: string) => void;
  contentClass?: string;
  contentShadows?: readonly Shadow[] | null;
  /** Optional viewport point used by context-menu style triggers. */
  anchorPoint?: () => PointAnchor | undefined;
}
/** A compact action menu with native focus, typeahead, and overlay routing. */
declare function DropdownMenu(props: DropdownMenuProps): JSX.Element;
//#endregion
//#region src/components/context-menu.d.ts
interface ContextMenuTriggerProps {
  ref(node: Handle): void;
  onContextMenu(event: WabouPointerEvent): void;
  onKeyDown(event: {
    key: string;
    preventDefault(): void;
    stopPropagation(): void;
  }): void;
  "aria-haspopup": "menu";
  "aria-expanded": boolean;
}
interface ContextMenuProps {
  trigger(props: ContextMenuTriggerProps): JSX.Element;
  items: readonly DropdownMenuItem[];
  "aria-label": string;
  onAction?: (id: string) => void;
  contentClass?: string;
  contentShadows?: readonly Shadow[] | null;
}
/** An action menu anchored to the native secondary-click coordinate. */
declare function ContextMenu(props: ContextMenuProps): JSX.Element;
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
//#region src/components/input.d.ts
interface InputProps extends TextInputProps {
  class?: string;
}
/** A plain-text input. Secrets must use `PasswordInput`. */
declare function Input(props: InputProps): JSX.Element;
//#endregion
//#region src/components/directory-picker.d.ts
interface DirectoryPickerProps extends Omit<InputProps, "class" | "onInput" | "value"> {
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
}
/** A controlled path input paired with the operating system directory picker. */
declare function DirectoryPicker(props: DirectoryPickerProps): JSX.Element;
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
type EmptyVariant = "surface" | "plain";
declare const emptyClass: (variant?: EmptyVariant, className?: string) => string;
declare function Empty(props: {
  children?: JSX.Element;
  class?: string;
  /** `plain` embeds inside an existing Card without creating a nested surface. */
  variant?: EmptyVariant;
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
type ResponsiveGridColumnCount = 1 | 2 | 3 | 4;
interface ResponsiveGridState {
  columns: Accessor<ResponsiveGridColumnCount>;
  width: Accessor<number>;
  height: Accessor<number>;
}
/** Read the completed native size and active column count of the nearest grid. */
declare function useResponsiveGrid(): ResponsiveGridState;
declare function responsiveGridColumnCount(options: {
  width: number;
  minColumnWidth: number;
  gap?: number;
  maxColumns?: ResponsiveGridColumnCount;
  initialColumns?: ResponsiveGridColumnCount;
}): ResponsiveGridColumnCount;
declare function responsiveGridRemainderCount(itemCount: number, columns: ResponsiveGridColumnCount): number;
interface ResponsiveGridProps extends Omit<ViewProps, "children" | "class" | "ref"> {
  children?: JSX.Element;
  /** Minimum usable content width for one item, in logical pixels. */
  minColumnWidth: number;
  /** Native row/column gap in logical pixels; also used to select the column count. */
  gap?: number;
  maxColumns?: ResponsiveGridColumnCount;
  /** Safe column count used until the native container has been measured. */
  initialColumns?: ResponsiveGridColumnCount;
  class?: string;
  ref?: ViewProps["ref"];
}
/**
 * A grid that responds to its own native content box instead of the window.
 *
 * This is important inside sidebars, split panes and dialogs: window media
 * queries do not know how much width the component actually receives.
 */
declare function ResponsiveGrid(props: ResponsiveGridProps): JSX.Element;
/** Fill the unused cells in the final row using the grid's measured columns. */
declare function ResponsiveGridRemainder(props: {
  itemCount: number;
  class?: string;
}): JSX.Element;
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
/**
 * Master/detail layout whose detail region can move from an inline rail to a
 * modal surface without changing the application's selection model.
 */
declare function AdaptiveSplitPane(props: {
  children?: JSX.Element;
  compact: boolean;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function AdaptiveSplitPaneMain(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function AdaptiveSplitPaneDetail(props: {
  children?: JSX.Element;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  "aria-label": string;
  class?: string;
  modalClass?: string;
}): import("@wabou/core").JSX.Element;
//#endregion
//#region src/components/navigation.d.ts
declare function Breadcrumb(props: {
  children?: JSX.Element;
  class?: string;
  "aria-label"?: string;
}): JSX.Element;
declare function BreadcrumbList(props: ViewProps): JSX.Element;
declare function BreadcrumbItem(props: ViewProps): JSX.Element;
interface BreadcrumbLinkProps extends Omit<ButtonProps, "class" | "role" | "variant" | "size"> {
  class?: string;
}
declare function BreadcrumbLink(props: BreadcrumbLinkProps): JSX.Element;
declare function BreadcrumbPage(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
declare function BreadcrumbSeparator(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
declare function BreadcrumbEllipsis(props: {
  class?: string;
}): JSX.Element;
declare function Pagination(props: {
  children?: JSX.Element;
  class?: string;
  "aria-label"?: string;
}): JSX.Element;
declare function PaginationContent(props: ViewProps): JSX.Element;
declare function PaginationItem(props: ViewProps): JSX.Element;
interface PaginationLinkProps extends Omit<ButtonProps, "variant" | "size"> {
  active?: boolean;
}
declare function PaginationLink(props: PaginationLinkProps): JSX.Element;
declare function PaginationPrevious(props: Omit<ButtonProps, "variant" | "size">): JSX.Element;
declare function PaginationNext(props: Omit<ButtonProps, "variant" | "size">): JSX.Element;
//#endregion
//#region src/components/page.d.ts
declare const pageViewportClass: (className?: string) => string;
declare const pageViewportContentClass: (className?: string) => string;
interface PageViewportProps extends Omit<ScrollAreaProps, "class" | "contentClass" | "ref"> {
  children?: JSX.Element;
  /** Classes applied to the bounded scrolling viewport. */
  class?: string;
  /** Classes applied to the full-height page content wrapper. */
  contentClass?: string;
  /** Reset the page to its origin whenever this identity changes. */
  resetKey?: unknown;
  ref?: (node: Handle) => void;
}
/**
 * A full-height application page boundary.
 *
 * This composes native scrolling with an explicitly sized content wrapper and
 * optional identity-based scroll reset. Page implementations can therefore
 * focus on their own layout instead of reconstructing flex/overflow rules.
 */
declare function PageViewport(props: PageViewportProps): JSX.Element;
declare const pageHeaderClass: (className?: string, stacked?: boolean) => string;
interface PageHeaderProps {
  title: string;
  description?: string;
  /** Optional content rendered immediately after the title. */
  titleAdornment?: JSX.Element;
  /** Page-level controls rendered at the trailing edge. */
  actions?: JSX.Element;
  /** Stack actions below the title for narrow application windows. */
  stacked?: boolean;
  class?: string;
}
/** Consistent page title, supporting text and trailing application actions. */
declare function PageHeader(props: PageHeaderProps): JSX.Element;
//#endregion
//#region src/components/popover.d.ts
type PopoverProps = PopoverProps$1;
/** A ready-to-use floating surface backed by Wabou's collision-aware overlay. */
declare function Popover(props: PopoverProps): JSX.Element;
declare function PopoverHeader(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
declare function PopoverTitle(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
declare function PopoverDescription(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
declare function PopoverFooter(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
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
  variant?: "default" | "accent";
  class?: string;
  children?: JSX.Element;
}
declare function ToggleGroupItem(props: ToggleGroupItemProps): JSX.Element;
//#endregion
//#region src/components/sheet.d.ts
type SheetSide = "top" | "right" | "bottom" | "left";
interface SheetProps extends Omit<ModalProps, "contentClass"> {
  side?: SheetSide;
  contentClass?: string;
}
/** A modal edge panel that shares native focus isolation with Dialog. */
declare function Sheet(props: SheetProps): JSX.Element;
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
  /** Keep tab semantics and roving focus while leaving layout and paint to the caller. */
  unstyled?: boolean;
  "aria-label"?: string;
  class?: string;
  children?: JSX.Element;
}): JSX.Element;
interface TabsTriggerProps {
  value: string;
  disabled?: boolean;
  /** Keep tab behavior and semantics without applying the component skin. */
  unstyled?: boolean;
  "aria-label"?: string;
  class?: string | ((state: ButtonState) => string);
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
interface WindowFrameProps extends ViewProps {
  class?: string;
  children?: JSX.Element;
  /** Clip the application surface to desktop-style corners when restored. */
  rounded?: boolean;
}
declare function windowFrameBackdropClassList(maximized: boolean, rounded?: boolean): WabouClassList;
declare function windowFrameClientClassList(maximized: boolean, rounded?: boolean, classList?: WabouClassList): WabouClassList;
/** Two restrained client-decoration layers sized to fit the 12px backdrop. */
declare function windowFrameShadows(theme: "light" | "dark"): Shadow[];
/**
 * Root frame for an application-owned title bar and window chrome.
 *
 * Rounded outer corners require the native window to preserve alpha and the
 * Rust host to clear with a transparent base color. Maximized windows are
 * intentionally square so their content reaches every display edge.
 */
declare function WindowFrame(props: WindowFrameProps): JSX.Element;
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
//#region src/components/toast.d.ts
type ToastVariant = "default" | "success" | "warning" | "destructive";
interface ToastAction {
  label: string;
  onAction(): void;
  /** Defaults to true. */
  dismiss?: boolean;
}
interface ToastInput {
  title: string;
  description?: string;
  variant?: ToastVariant;
  action?: ToastAction;
  duration?: number;
  onDismiss?: (reason: NotificationDismissReason) => void;
}
interface Toasts {
  /** The unstyled queue remains available for advanced composition. */
  readonly notifications: Notifications;
  show(input: ToastInput): number;
  success(title: string, input?: Omit<ToastInput, "title" | "variant">): number;
  warning(title: string, input?: Omit<ToastInput, "title" | "variant">): number;
  error(title: string, input?: Omit<ToastInput, "title" | "variant">): number;
  dismiss(id: number): boolean;
  clear(): void;
}
interface CreateToastsOptions {
  defaultDuration?: number;
  limit?: number;
}
/** Create an owner-scoped, styled toast queue over NotificationRegion. */
declare function createToasts(options?: CreateToastsOptions): Toasts;
interface ToasterProps {
  toasts: Toasts;
  placement?: NotificationPlacement;
  class?: string;
  itemClass?: string;
}
/** Render a non-blocking stack of styled toasts on the floating plane. */
declare function Toaster(props: ToasterProps): JSX.Element;
//#endregion
//#region src/components/tooltip-state.d.ts
interface TooltipDelayController {
  scheduleOpen(): void;
  scheduleClose(): void;
  openNow(): void;
  closeNow(): void;
  dispose(): void;
}
interface TooltipDelayOptions {
  openDelay: () => number;
  closeDelay: () => number;
  setOpen(open: boolean): void;
}
/** Owns tooltip timers independently from rendering and positioning. */
declare function createTooltipDelayController(options: TooltipDelayOptions): TooltipDelayController;
//#endregion
//#region src/components/tooltip.d.ts
interface TooltipTriggerProps {
  ref(node: Handle): void;
  onPointerEnter(): void;
  onPointerLeave(): void;
  onFocus(): void;
  onBlur(): void;
  onKeyDown(event: {
    key: string;
    preventDefault(): void;
    stopPropagation(): void;
  }): void;
}
interface TooltipProps {
  trigger(props: TooltipTriggerProps): JSX.Element;
  children?: JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?(open: boolean): void;
  openDelay?: number;
  closeDelay?: number;
  placement?: Placement;
  offset?: number;
  contentClass?: string;
  disabled?: boolean;
}
/** A delayed, non-interactive label for pointer and keyboard focus targets. */
declare function Tooltip(props: TooltipProps): JSX.Element;
//#endregion
//#region src/components/index.d.ts
interface BadgeProps {
  children?: JSX.Element;
  variant?: "default" | "secondary" | "outline" | "success" | "destructive";
  /** Typography weight selected without competing utility declarations. */
  weight?: "normal" | "medium";
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
  ref?: ViewProps["ref"];
  shadows?: readonly import("@wabou/core/style").Shadow[] | null;
  role?: ViewProps["role"];
  "aria-label"?: string;
  "aria-hidden"?: ViewProps["aria-hidden"];
}): JSX.Element;
declare function CardHeader(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
interface CardTitleProps extends TextProps {}
declare function CardTitle(props: CardTitleProps): JSX.Element;
interface CardDescriptionProps extends TextProps {}
declare function CardDescription(props: CardDescriptionProps): JSX.Element;
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
  class?: string;
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
interface RouteActiveOptions {
  /** Match only this path instead of descendant routes. Defaults to true for `/`. */
  exact?: boolean;
  /** Include the target's search parameters in the match. Defaults to false. */
  includeSearch?: boolean;
  /** Match the pending destination while navigation is loading. */
  pending?: boolean;
}
/**
 * Reactively report whether a native router destination is active.
 *
 * This delegates path, base-path, parameter, and trailing-slash behavior to
 * Router Core instead of duplicating pathname comparisons in navigation UI.
 */
declare function useRouteActive(to: string, options?: RouteActiveOptions): Accessor<boolean>;
declare function useParams<T extends Record<string, string> = Record<string, string>>(): Accessor<T>;
declare function useLoaderData<T = unknown>(): Accessor<T | undefined>;
//#endregion
export { Accordion, AccordionContent, AccordionItem, AccordionProps, AccordionTrigger, AccordionType, type ActiveResult, AdaptiveSplitPane, AdaptiveSplitPaneDetail, AdaptiveSplitPaneMain, Alert, AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogProps, AlertDialogTitle, AnimationControls, type AnimationFrameCallback, AnimationOptions, AnimationState, AnimationType, AnimationValue, Avatar, AvatarGroup, AvatarGroupCount, AvatarProps, AvatarSize, Badge, BadgeProps, BaseRootRoute, BaseRoute, Breadcrumb, BreadcrumbEllipsis, BreadcrumbItem, BreadcrumbLink, BreadcrumbLinkProps, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator, Button, ButtonGroup, ButtonGroupText, ButtonProps, ButtonSize, ButtonVariant, Calendar, CalendarDate, CalendarLabels, CalendarProps, Card, CardContent, CardDescription, CardDescriptionProps, CardFooter, CardHeader, CardTitle, CardTitleProps, Center, Checkbox, type CheckboxProps, CodeEditor, type CodeEditorProps, Collapsible, CollapsibleContent, CollapsiblePresence, CollapsibleProps, CollapsibleTrigger, Column, Combobox, ComboboxOption, ComboboxProps, Command, CommandItem, CommandProps, type ComponentsElevation, ComponentsProvider, type ComponentsProviderProps, type ComponentsTheme, ConfigEditor, ConfigEditorProps, type ContainerMatch, type ContainerSizeQuery, ContextMenu, ContextMenuProps, ContextMenuTriggerProps, CreateToastsOptions, DatePicker, DatePickerProps, Dialog, type ModalControls as DialogControls, DialogDescription, DialogDescription as SheetDescription, DialogFooter, DialogFooter as SheetFooter, DialogHeader, DialogHeader as SheetHeader, DialogProps, DialogScrollBody, DialogScrollBody as SheetScrollBody, DialogTitle, DialogTitle as SheetTitle, DirectoryPicker, DirectoryPickerProps, DropdownMenu, DropdownMenuItem, DropdownMenuProps, Easing, EasingFunction, Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle, EmptyVariant, Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldOrientation, type FocusResult, type FocusWithinResult, type FormDraft, type FormDraftErrors, type FormDraftFieldUpdater, type FormDraftOptions, Fps, FpsProps, Icon, type IconProps, Image, type ImageProps, type ImageSource, Input, InputGroup, InputGroupButton, InputGroupInput, InputGroupText, InputGroupTextArea, InputProps, Kbd, KbdGroup, type KeyedSelection, type KeyedSelectionOptions, type LayoutProps, LoopOptions, Modal, type ModalProps, NetworkImage, type NetworkImageProps, type NetworkImageSource, NotificationRegion, type NotificationRegionProps, type Notifications, type OverlayLayer, OverlayPlaneProvider, PageHeader, PageHeaderProps, PageViewport, PageViewportProps, Pagination, PaginationContent, PaginationItem, PaginationLink, PaginationLinkProps, PaginationNext, PaginationPrevious, PasswordInput, PasswordInputProps, Path, PathBuilder, type PathProps, Popover, PopoverDescription, PopoverFooter, PopoverHeader, PopoverProps, PopoverTitle, type PressResult, Button$1 as PrimitiveButton, type ButtonProps$1 as PrimitiveButtonProps, type ButtonState as PrimitiveButtonState, Link as PrimitiveLink, type LinkProps as PrimitiveLinkProps, PasswordInput$1 as PrimitivePasswordInput, type PasswordInputProps$1 as PrimitivePasswordInputProps, Popover$1 as PrimitivePopover, type PopoverProps$1 as PrimitivePopoverProps, TextArea as PrimitiveTextArea, type TextAreaProps as PrimitiveTextAreaProps, TextInput as PrimitiveTextInput, type TextInputProps as PrimitiveTextInputProps, Progress, Pulse, PulseOptions, RadioGroup, RadioGroupItem, type RadioGroupItemProps, type RadioGroupProps, ReactiveAnimation, ReactiveTransition, RepeatType, ResponsiveGrid, ResponsiveGridColumnCount, ResponsiveGridProps, ResponsiveGridRemainder, ResponsiveGridState, Ripple, type RippleProps, RotationAnimation, RotationOptions, RouteActiveOptions, RouterProvider, RouterProviderProps, Row, ScrollArea, type ScrollAreaProps, Select, SelectOption, SelectProps, Separator, Sheet, SheetProps, SheetSide, Skeleton, Slider, SliderProps, Spin, Spinner, SplitPane, SplitPaneAside, SplitPaneMain, Svg, type SvgProps, Switch, SwitchProps, Tabs, TabsContent, TabsList, type TabsProps, type TabsResult, TabsTrigger, type TabsTriggerProps, Text, TextArea$1 as TextArea, TextAreaProps$1 as TextAreaProps, type TextProps, TitleBar, TitleBarDragRegion, TitleBarDragRegionProps, TitleBarProps, ToastAction, ToastInput, ToastVariant, Toaster, ToasterProps, Toasts, Toggle, ToggleGroup, ToggleGroupItem, type ToggleGroupItemProps, type ToggleGroupProps, type ToggleProps, Tooltip, TooltipProps, TooltipTriggerProps, TransitionOptions, type VectorPath, type VectorPathPaint, View, type ViewProps, type WabouClassList, WabouDataRouter, type WabouStyle, WindowFrame, WindowFrameProps, animate, animateKeyframes, componentsElevation, createActive, createAnimationFrame, createButton, createContainerMatch, createDataRouter, createFocus, createFocusWithin, createFormDraft, createHover, createKeyedSelection, createLoop, createMeasuredSize, createMemoryHistory, createNotifications, createOverlayLayer, createPresence, createPress, createPulse, createRotation, createScrollReset, createShortcuts, createTabs, createToasts, createTooltipDelayController, createTransition, emptyClass, filterCommandItems, moveMenuHighlight, nextAccordionValue, notFound, pageHeaderClass, pageViewportClass, pageViewportContentClass, index_d_exports as primitives, reconcileCommandHighlight, redirect, responsiveGridColumnCount, responsiveGridRemainderCount, titleBarClass, titleBarDragRegionLayoutStyle, titleBarLayoutStyle, useComponentsTheme, useLoaderData, useLocation, useNavigate, useParams, useResponsiveGrid, useRouteActive, useRouter, useRouterState, windowFrameBackdropClassList, windowFrameClientClassList, windowFrameShadows };
//# sourceMappingURL=index.d.mts.map