import { $n as AnimationFrameCallback, $t as FormDraftFieldUpdater, A as createPresence, An as NetworkImageSource, Ar as createTransition, At as ModalControls, Bn as TextArea, C as ActiveResult, Cn as Icon, Cr as animateKeyframes, Ct as Pulse, D as createPress, Dn as ImageSource, Dr as createPulse, Dt as Spin, E as createActive, En as ImageProps, Er as createLoop, Et as RippleProps, F as PopoverMotionOptions, Fn as PathProps, Fr as useMotionConfig, Ft as ModalProps, Gn as VectorPath, Gt as LayoutProps, Hn as TextInput, Ht as createMeasuredSize, I as PopoverProps$1, Ir as useReducedMotion, Jn as ViewProps, Jt as KeyedSelectionOptions, Kn as VectorPathPaint, Kt as Row, Ln as Svg, Lt as ContainerMatch, M as RetainedItems, Mn as PasswordInputProps$1, Mr as MotionConfig, N as createRetainedItems, Nn as Path, Nr as MotionConfigProvider, Nt as ModalMotionOptions, On as NetworkImage, Or as createRotation, P as Popover$1, Pn as PathBuilder, Pr as MotionConfigProviderProps, Pt as ModalOpenChangeReason, Qt as FormDraftErrors, Rn as SvgProps, Rt as ContainerSizeQuery, S as ScrollAreaProps, Sn as CodeEditorProps, Sr as animate, St as createNotifications, T as PressResult, Tn as Image, Tr as createKeyframeAnimation, Tt as Ripple, U as Placement, Un as TextInputProps, Ut as Center, Vn as TextAreaProps, Vt as createContainerMatch, W as PointAnchor, Wn as TextProps, Wt as Column, Xn as WabouStyle, Xt as FORM_ERROR, Yn as WabouClassList, Yt as createKeyedSelection, Zt as FormDraft, _ as createShortcuts, _n as Link, _r as SweepAnimation, an as FocusWithinResult, ar as AnimationValue, b as createScrollReset, br as SweepOptions, bt as Notifications, cn as CollapsiblePresence, cr as KeyframeAnimationOptions, d as createTabs, dr as PulseOptions, en as FormDraftOptions, er as createAnimationFrame, fr as ReactiveAnimation, ft as NotificationDismissReason, gr as RotationOptions, gt as NotificationPlacement, hn as ButtonState, hr as RotationAnimation, ht as NotificationMotionOptions, i as createTransitionPresence, in as FocusResult, ir as AnimationType, it as OverlayLayer, j as RetainedItem, jn as PasswordInput$1, jr as normalizeSweepGeometry, kn as NetworkImageProps, kr as createSweep, kt as Modal, lr as LoopOptions, lt as createOverlayLayer, mn as ButtonProps$1, mr as RepeatType, n as TransitionPresence, nr as AnimationOptions, on as createFocus, or as Easing, pr as ReactiveTransition, qn as View, qt as KeyedSelection, r as TransitionPresenceOptions, rn as createHover, rr as AnimationState, sn as createFocusWithin, sr as EasingFunction, st as OverlayPlaneProvider, t as index_d_exports, tn as createFormDraft, tr as AnimationControls, u as TabsResult, un as Button$1, ur as MotionInterpolationOptions, vn as LinkProps, vr as SweepAxis, vt as NotificationRegion, wn as IconProps, wr as createInterpolation, x as ScrollArea, xn as CodeEditor, xr as TransitionOptions, yn as createButton, yr as SweepGeometry, yt as NotificationRegionProps, zn as Text } from "./index-DDOVEQ1Q.mjs";
import { PickDirectoryOptions } from "@wabou/core";
import { Shadow, WabouStyle as WabouStyle$1 } from "@wabou/core/style";
import { Accessor, JSX, ParentProps, Setter } from "solid-js";
import { Handle, WabouPointerEvent } from "@wabou/core/renderer";
import { CalendarDate } from "@internationalized/date";
import { ColumnDef, Row as Row$1, RowSelectionState, SortingState, Table as Table$1 } from "@tanstack/table-core";
import { RouterHistory, createMemoryHistory } from "@tanstack/history";
import { AnyRoute, AnyRouter, BaseRootRoute, BaseRoute, RouterConstructorOptions, RouterCore, TrailingSlashOption, notFound, redirect } from "@tanstack/router-core";
export * from "@wabou/core";
export * from "@wabou/core/i18n";
//#region src/components/alert.d.ts
type AlertVariant = "default" | "destructive";
declare function alertColors(variant: AlertVariant): {
  container: string;
  title: string;
  description: string;
};
interface AlertProps extends Omit<ViewProps, "role"> {
  variant?: AlertVariant;
  /** Optional leading graphic with caller-owned size and color. */
  icon?: JSX.Element;
  /** Convenience form; compound usage can render AlertTitle directly. */
  title?: string;
}
/** A native status callout with shadcn-compatible compound composition. */
declare function Alert(props: AlertProps): JSX.Element;
declare function AlertTitle(props: TextProps): JSX.Element;
declare function AlertDescription(props: TextProps): JSX.Element;
//#endregion
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
//#region src/components/aspect-ratio.d.ts
interface AspectRatioProps extends Omit<ViewProps, "style"> {
  /** Width divided by height. Defaults to a square. */
  ratio?: number;
  style?: WabouStyle$1;
}
declare function aspectRatioStyle(ratio: number | undefined, style?: WabouStyle$1): WabouStyle$1;
/** A native Taffy aspect-ratio constraint with explicit overflow ownership. */
declare function AspectRatio(props: AspectRatioProps): JSX.Element;
//#endregion
//#region src/components/attachment.d.ts
type AttachmentState = "idle" | "uploading" | "processing" | "error" | "done";
type AttachmentSize = "default" | "sm" | "xs";
type AttachmentOrientation = "horizontal" | "vertical";
interface AttachmentContextValue {
  state(): AttachmentState;
  size(): AttachmentSize;
  orientation(): AttachmentOrientation;
}
declare function attachmentClass(options: {
  state?: AttachmentState;
  size?: AttachmentSize;
  orientation?: AttachmentOrientation;
  class?: string;
}): string;
interface AttachmentProps extends Omit<ViewProps, "class"> {
  state?: AttachmentState;
  size?: AttachmentSize;
  orientation?: AttachmentOrientation;
  class?: string;
}
/** File/task summary anatomy adapted from shadcn without DOM data selectors. */
declare function Attachment(props: AttachmentProps): JSX.Element;
declare function attachmentMediaClass(variant: "icon" | "image", context: Pick<AttachmentContextValue, "state" | "size" | "orientation">, className?: string): string;
declare function AttachmentMedia(props: {
  children?: JSX.Element;
  variant?: "icon" | "image";
  class?: string;
}): JSX.Element;
declare function AttachmentContent(props: ViewProps): JSX.Element;
declare function AttachmentTitle(props: TextProps): JSX.Element;
declare function AttachmentDescription(props: TextProps): JSX.Element;
declare function AttachmentActions(props: ViewProps): JSX.Element;
declare function AttachmentAction(props: ButtonProps): JSX.Element;
declare function AttachmentGroup(props: ViewProps): JSX.Element;
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
//#region src/components/badge.d.ts
type BadgeVariant = "default" | "secondary" | "outline" | "ghost" | "link" | "success" | "destructive";
interface BadgeProps extends Omit<TextProps, "class"> {
  variant?: BadgeVariant;
  /** Typography weight selected without competing utility declarations. */
  weight?: "normal" | "medium";
  class?: string;
}
declare function badgeClass(variant?: BadgeVariant, weight?: NonNullable<BadgeProps["weight"]>, className?: string): string;
/** Compact status text with shadcn-compatible visual variants. */
declare function Badge(props: BadgeProps): JSX.Element;
//#endregion
//#region src/components/button-group-context.d.ts
type ButtonGroupOrientation = "horizontal" | "vertical";
//#endregion
//#region src/components/button-group.d.ts
interface ButtonGroupProps extends Omit<ViewProps, "class"> {
  orientation?: ButtonGroupOrientation;
  class?: string;
}
/** A single bordered control surface composed from ordinary Wabou buttons. */
declare function ButtonGroup(props: ButtonGroupProps): JSX.Element;
declare function ButtonGroupText(props: TextProps): JSX.Element;
interface ButtonGroupSeparatorProps {
  orientation?: "horizontal" | "vertical";
  class?: string;
}
declare function ButtonGroupSeparator(props: ButtonGroupSeparatorProps): JSX.Element;
//#endregion
//#region src/components/card.d.ts
interface CardProps extends Omit<ViewProps, "class"> {
  class?: string;
}
declare function Card(props: CardProps): JSX.Element;
declare function CardHeader(props: ViewProps): JSX.Element;
declare function CardTitle(props: TextProps): JSX.Element;
declare function CardDescription(props: TextProps): JSX.Element;
/** Top-end action slot owned by the relative CardHeader surface. */
declare function CardAction(props: ViewProps): JSX.Element;
declare function CardContent(props: ViewProps): JSX.Element;
declare function CardFooter(props: ViewProps): JSX.Element;
//#endregion
//#region src/components/carousel.d.ts
type CarouselOrientation = "horizontal" | "vertical";
interface CarouselApi {
  selectedIndex(): number;
  itemCount(): number;
  canScrollPrevious(): boolean;
  canScrollNext(): boolean;
  scrollPrevious(): void;
  scrollNext(): void;
  scrollTo(index: number): void;
}
declare function normalizeCarouselIndex(index: number, count: number, loop: boolean): number;
interface CarouselProps {
  "aria-label": string;
  index?: number;
  defaultIndex?: number;
  onIndexChange?(index: number): void;
  orientation?: CarouselOrientation;
  loop?: boolean;
  setApi?(api: CarouselApi): void;
  class?: string;
  children?: JSX.Element;
}
/** A native snapping carousel with captured pointer dragging and keyboard navigation. */
declare function Carousel(props: CarouselProps): JSX.Element;
interface CarouselContentProps extends Omit<ViewProps, "transform"> {
  trackClass?: string;
  dragThreshold?: number;
}
declare function CarouselContent(props: CarouselContentProps): JSX.Element;
interface CarouselItemProps extends ViewProps {
  "aria-label"?: string;
}
declare function CarouselItem(props: CarouselItemProps): JSX.Element;
declare function CarouselPrevious(props: Omit<ButtonProps, "children"> & {
  children?: JSX.Element;
}): JSX.Element;
declare function CarouselNext(props: Omit<ButtonProps, "children"> & {
  children?: JSX.Element;
}): JSX.Element;
//#endregion
//#region src/components/popover.d.ts
type PopoverProps = PopoverProps$1;
/** Shared visual-motion contract for components backed by a popup surface. */
interface PopupMotionProps {
  /** Override the default popup transition, or disable it without changing presence semantics. */
  motion?: false | PopoverMotionOptions;
}
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
interface ComboboxProps extends PopupMotionProps {
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
interface DropdownMenuProps extends PopupMotionProps {
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
  placement?: Placement;
  /** Skip returning focus when ownership moves directly to a sibling menu. */
  restoreFocus?: boolean;
  outsidePointerStrategy?: "backdrop" | "passthrough";
  /** Observe or override keys before the menu's vertical navigation runs. */
  onContentKeyDown?: (event: DropdownMenuKeyEvent) => void;
  /** Optional viewport point used by context-menu style triggers. */
  anchorPoint?: () => PointAnchor | undefined;
}
interface DropdownMenuKeyEvent {
  key: string;
  readonly defaultPrevented?: boolean;
  preventDefault(): void;
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
interface ContextMenuProps extends PopupMotionProps {
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
interface DatePickerProps extends Omit<CalendarProps, "aria-label">, PopupMotionProps {
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
  /** Background utility owned by this input. Defaults to `bg-input`. */
  surfaceClass?: string;
  /**
   * Selects which component owns the input surface.
   *
   * Use `none` when composing an input inside an `InputGroup`; the group then
   * owns its background, border, radius, and shadow without conflicting style
   * declarations on the native editor.
   */
  chrome?: "default" | "none";
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
interface SkeletonProps {
  class?: string;
  /** Disable the shimmer while preserving the stable loading placeholder. */
  animated?: boolean;
}
declare function Skeleton(props: SkeletonProps): JSX.Element;
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
//#region src/components/drawer.d.ts
type DrawerDirection = "top" | "right" | "bottom" | "left";
type DrawerOpenChangeReason = ModalOpenChangeReason | "drag";
declare function drawerDragOffset(direction: DrawerDirection, rawDelta: number): number;
declare function drawerShouldDismiss(offset: number, size: number, threshold: number): boolean;
interface DrawerProps extends Omit<ModalProps, "defaultOpen" | "onOpenChange" | "contentClass" | "contentTransform" | "motion"> {
  defaultOpen?: boolean;
  onOpenChange?(open: boolean, reason: DrawerOpenChangeReason): void;
  direction?: DrawerDirection;
  dismissible?: boolean;
  dismissThreshold?: number;
  contentClass?: string;
}
/** A focus-isolated edge drawer with a captured native drag-to-dismiss gesture. */
declare function Drawer(props: DrawerProps): JSX.Element;
declare function DrawerHandle(props: ViewProps): JSX.Element;
declare function DrawerHeader(props: ViewProps): JSX.Element;
declare function DrawerFooter(props: ViewProps): JSX.Element;
declare function DrawerTitle(props: TextProps): JSX.Element;
declare function DrawerDescription(props: TextProps): JSX.Element;
declare function DrawerClose(props: ButtonProps): JSX.Element;
//#endregion
//#region src/components/empty.d.ts
type EmptyVariant = "surface" | "plain";
type EmptyMediaVariant = "default" | "icon";
interface EmptyProps extends Omit<ViewProps, "class"> {
  /** `plain` embeds inside an existing surface without nesting another card. */
  variant?: EmptyVariant;
  class?: string;
}
declare function emptyClass(variant?: EmptyVariant, className?: string): string;
/** A composable empty-state region based on shadcn's Empty anatomy. */
declare function Empty(props: EmptyProps): JSX.Element;
declare function EmptyHeader(props: ViewProps): JSX.Element;
interface EmptyMediaProps extends Omit<ViewProps, "class"> {
  variant?: EmptyMediaVariant;
  class?: string;
}
declare function emptyMediaClass(variant?: EmptyMediaVariant, className?: string): string;
declare function EmptyMedia(props: EmptyMediaProps): JSX.Element;
declare function EmptyTitle(props: TextProps): JSX.Element;
declare function EmptyDescription(props: TextProps): JSX.Element;
declare function EmptyContent(props: ViewProps): JSX.Element;
//#endregion
//#region src/components/label.d.ts
interface LabelProps extends Omit<TextProps, "class" | "role"> {
  class?: string;
  disabled?: boolean;
  /**
   * Explicit native control target. Unlike HTML `for`, this cannot silently
   * point at a missing string id and remains safe across retained-tree reuse.
   */
  control?: Handle | (() => Handle | undefined);
}
/** Text label that forwards pointer activation to an explicit native control. */
declare function Label(props: LabelProps): JSX.Element;
//#endregion
//#region src/components/forms.d.ts
type FieldOrientation = "vertical" | "horizontal";
declare function fieldClass(orientation?: FieldOrientation, invalid?: boolean, className?: string): string;
declare function Field(props: {
  children?: JSX.Element;
  orientation?: FieldOrientation;
  invalid?: boolean;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function FieldSet(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function FieldLegend(props: {
  children?: JSX.Element;
  variant?: "legend" | "label";
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function FieldGroup(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
interface FieldLabelProps extends LabelProps {}
declare function FieldLabel(props: FieldLabelProps): import("@wabou/core").JSX.Element;
declare function FieldTitle(props: {
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
interface FieldErrorLike {
  message?: string;
}
declare function uniqueFieldErrors(errors: ReadonlyArray<FieldErrorLike | undefined> | undefined): string[];
declare function fieldErrorLabel(explicit: string | undefined, children: JSX.Element, messages: readonly string[]): string | undefined;
declare function FieldError(props: {
  children?: JSX.Element;
  errors?: ReadonlyArray<FieldErrorLike | undefined>;
  "aria-label"?: string;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function FieldSeparator(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
type InputGroupOrientation = "horizontal" | "vertical";
type InputGroupAddonAlign = "inline-start" | "inline-end" | "block-start" | "block-end";
declare function inputGroupClass(orientation: InputGroupOrientation, focused: boolean, invalid: boolean): string;
interface InputGroupProps extends Omit<ViewProps, "children"> {
  children?: JSX.Element;
  orientation?: InputGroupOrientation;
  invalid?: boolean;
  disabled?: boolean;
  /** Background utility owned by the compound control. Defaults to `bg-input`. */
  surfaceClass?: string;
}
declare function InputGroup(props: InputGroupProps): import("@wabou/core").JSX.Element;
declare function InputGroupInput(props: InputProps): import("@wabou/core").JSX.Element;
interface InputGroupAddonProps extends ViewProps {
  align?: InputGroupAddonAlign;
  focusControl?: boolean;
}
declare function inputGroupAddonClass(align: InputGroupAddonAlign): string;
declare function InputGroupAddon(props: InputGroupAddonProps): import("@wabou/core").JSX.Element;
declare function InputGroupText(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core").JSX.Element;
declare function InputGroupButton(props: ButtonProps): import("@wabou/core").JSX.Element;
declare function InputGroupTextArea(props: TextAreaProps & {
  class?: string;
}): import("@wabou/core").JSX.Element;
//#endregion
//#region src/components/delayed-open.d.ts
interface DelayedOpenController {
  scheduleOpen(): void;
  scheduleClose(): void;
  openNow(): void;
  closeNow(): void;
  cancel(): void;
  dispose(): void;
}
interface DelayedOpenOptions {
  openDelay: () => number;
  closeDelay: () => number;
  setOpen(open: boolean): void;
}
/** Owns cancellable open/close timers independently from a rendered surface. */
declare function createDelayedOpenController(options: DelayedOpenOptions): DelayedOpenController;
//#endregion
//#region src/components/hover-card.d.ts
interface HoverCardTriggerProps {
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
  "aria-haspopup": "dialog";
  "aria-expanded": boolean;
}
interface HoverCardProps extends PopupMotionProps {
  "aria-label": string;
  trigger(props: HoverCardTriggerProps): JSX.Element;
  children?: JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?(open: boolean): void;
  openDelay?: number;
  closeDelay?: number;
  placement?: Placement;
  offset?: number;
  contentClass?: string;
  contentShadows?: readonly Shadow[] | null;
  disabled?: boolean;
}
/** A preview surface that tolerates pointer travel between trigger and card. */
declare function HoverCard(props: HoverCardProps): JSX.Element;
//#endregion
//#region src/components/input-otp.d.ts
declare function normalizeOtpValue(value: string, maxLength: number, allowed?: RegExp): string;
interface InputOTPProps {
  value?: string;
  defaultValue?: string;
  maxLength: number;
  allowed?: RegExp;
  disabled?: boolean;
  readOnly?: boolean;
  "aria-label": string;
  class?: string;
  inputClass?: string;
  children?: JSX.Element;
  onValueChange?: (value: string) => void;
  onComplete?: (value: string) => void;
  ref?: TextInputProps["ref"];
}
declare function InputOTP(props: InputOTPProps): JSX.Element;
declare function InputOTPGroup(props: ViewProps): JSX.Element;
interface InputOTPSlotProps extends Omit<ViewProps, "children"> {
  index: number;
}
declare function InputOTPSlot(props: InputOTPSlotProps): JSX.Element;
declare function InputOTPSeparator(props: ViewProps): JSX.Element;
//#endregion
//#region src/components/item.d.ts
type ItemVariant = "default" | "outline" | "muted";
type ItemSize = "default" | "sm";
type ItemMediaVariant = "default" | "icon" | "image";
declare function itemClass(variant?: ItemVariant, size?: ItemSize, className?: string): string;
interface ItemProps extends Omit<ViewProps, "class"> {
  variant?: ItemVariant;
  size?: ItemSize;
  class?: string;
}
/** A composable list row based on shadcn's Item anatomy. */
declare function Item(props: ItemProps): JSX.Element;
declare function ItemGroup(props: ViewProps): JSX.Element;
declare function ItemSeparator(props: {
  class?: string;
}): JSX.Element;
declare function itemMediaClass(variant?: ItemMediaVariant, className?: string): string;
declare function ItemMedia(props: {
  children?: JSX.Element;
  variant?: ItemMediaVariant;
  class?: string;
}): JSX.Element;
declare function ItemContent(props: ViewProps): JSX.Element;
declare function ItemTitle(props: TextProps): JSX.Element;
declare function ItemDescription(props: TextProps): JSX.Element;
declare function ItemActions(props: ViewProps): JSX.Element;
declare function ItemHeader(props: ViewProps): JSX.Element;
declare const ItemFooter: typeof ItemHeader;
//#endregion
//#region src/components/layout.d.ts
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
//#region src/components/menubar.d.ts
interface MenubarProps {
  "aria-label": string;
  value?: string | null;
  defaultValue?: string | null;
  onValueChange?: (value: string | null) => void;
  loop?: boolean;
  class?: string;
  children?: JSX.Element;
}
/** Persistent application menus with one tab stop and sibling menu switching. */
declare function Menubar(props: MenubarProps): JSX.Element;
interface MenubarMenuProps extends PopupMotionProps {
  value: string;
  label: string;
  items: readonly DropdownMenuItem[];
  disabled?: boolean;
  onAction?: (id: string) => void;
  children?: JSX.Element;
}
declare function MenubarMenu(props: MenubarMenuProps): JSX.Element;
//#endregion
//#region src/components/message.d.ts
type MessageAlign = "start" | "end";
type BubbleVariant = "default" | "secondary" | "muted" | "tinted" | "outline" | "ghost" | "destructive";
declare function MessageGroup(props: ViewProps): JSX.Element;
interface MessageProps extends Omit<ViewProps, "class"> {
  align?: MessageAlign;
  class?: string;
}
declare function messageClass(align?: MessageAlign, className?: string): string;
declare function Message(props: MessageProps): JSX.Element;
declare function MessageAvatar(props: ViewProps): JSX.Element;
declare function MessageContent(props: ViewProps): JSX.Element;
declare function MessageHeader(props: TextProps): JSX.Element;
declare const MessageFooter: typeof MessageHeader;
declare function BubbleGroup(props: ViewProps): JSX.Element;
interface BubbleProps extends Omit<ViewProps, "class"> {
  variant?: BubbleVariant;
  align?: MessageAlign;
  class?: string;
}
declare function bubbleClass(variant?: BubbleVariant, align?: MessageAlign, className?: string): string;
declare function Bubble(props: BubbleProps): JSX.Element;
declare function bubbleContentClass(variant: BubbleVariant, className?: string): string;
declare function BubbleContent(props: ViewProps): JSX.Element;
declare function BubbleReactions(props: {
  children?: JSX.Element;
  side?: "top" | "bottom";
  align?: MessageAlign;
  class?: string;
}): JSX.Element;
type MarkerVariant = "default" | "separator" | "border";
declare function Marker(props: {
  children?: JSX.Element;
  variant?: MarkerVariant;
  class?: string;
}): JSX.Element;
declare function MarkerIcon(props: ViewProps): JSX.Element;
declare function MarkerContent(props: TextProps): JSX.Element;
//#endregion
//#region src/components/message-scroller.d.ts
type MessageScrollDirection = "start" | "end";
declare function messageScrollRange(contentHeight: number, viewportHeight: number): number;
declare function isMessageScrollNearEnd(scrollY: number, contentHeight: number, viewportHeight: number, threshold?: number): boolean;
interface MessageScrollerControls {
  followingEnd(): boolean;
  canScrollStart(): boolean;
  canScrollEnd(): boolean;
  scrollTo(direction: MessageScrollDirection): void;
}
declare function useMessageScroller(): MessageScrollerControls;
interface MessageScrollerProps extends ViewProps {
  /** Start attached to the end of the conversation. Defaults to true. */
  followEnd?: boolean;
  /** Distance in logical pixels that still counts as being at the end. */
  endThreshold?: number;
}
declare function MessageScroller(props: MessageScrollerProps): JSX.Element;
interface MessageScrollerViewportProps extends ViewProps {}
declare function MessageScrollerViewport(props: MessageScrollerViewportProps): JSX.Element;
declare function MessageScrollerContent(props: ViewProps): JSX.Element;
declare function MessageScrollerItem(props: ViewProps): JSX.Element;
interface MessageScrollerButtonProps extends Omit<ButtonProps, "onClick"> {
  direction?: MessageScrollDirection;
  onClick?: ButtonProps["onClick"];
}
declare function MessageScrollerButton(props: MessageScrollerButtonProps): JSX.Element;
//#endregion
//#region src/components/pagination-state.d.ts
type PaginationRangeItem = number | "ellipsis-start" | "ellipsis-end";
declare function normalizePageCount(count: number): number;
declare function clampPage(page: number, count: number): number;
/**
 * Produces a stable, 1-indexed page range with explicit start/end ellipses.
 * A single hidden page is shown directly instead of being replaced by an
 * ellipsis, which keeps every item actionable and avoids misleading gaps.
 */
declare function createPaginationRange(options: {
  count: number;
  page: number;
  siblingCount?: number;
  boundaryCount?: number;
}): PaginationRangeItem[];
//#endregion
//#region src/components/navigation.d.ts
interface BreadcrumbProps extends Omit<ViewProps, "class" | "role"> {
  class?: string;
}
declare function Breadcrumb(props: BreadcrumbProps): JSX.Element;
declare function BreadcrumbList(props: ViewProps): JSX.Element;
declare function BreadcrumbItem(props: ViewProps): JSX.Element;
interface BreadcrumbLinkProps extends Omit<ButtonProps, "class" | "role" | "variant" | "size"> {
  class?: string;
}
declare function BreadcrumbLink(props: BreadcrumbLinkProps): JSX.Element;
interface BreadcrumbPageProps extends Omit<TextProps, "class" | "role"> {
  class?: string;
}
declare function BreadcrumbPage(props: BreadcrumbPageProps): JSX.Element;
interface BreadcrumbSeparatorProps extends Omit<ViewProps, "class" | "role"> {
  class?: string;
}
declare function BreadcrumbSeparator(props: BreadcrumbSeparatorProps): JSX.Element;
interface BreadcrumbEllipsisProps extends Omit<ViewProps, "class" | "role" | "children"> {
  class?: string;
}
declare function BreadcrumbEllipsis(props: BreadcrumbEllipsisProps): JSX.Element;
interface PaginationProps {
  children?: JSX.Element;
  class?: string;
  "aria-label"?: string;
  /** Enables managed pagination. Omit it to retain the composition-only API. */
  count?: number;
  page?: number;
  defaultPage?: number;
  disabled?: boolean;
  onPageChange?: (page: number) => void;
}
declare function Pagination(props: PaginationProps): JSX.Element;
declare function PaginationContent(props: ViewProps): JSX.Element;
declare function PaginationItem(props: ViewProps): JSX.Element;
interface PaginationLinkProps extends Omit<ButtonProps, "variant" | "size"> {
  active?: boolean;
  /** Selects this page when used inside a managed Pagination. */
  page?: number;
}
declare function PaginationLink(props: PaginationLinkProps): JSX.Element;
declare function PaginationEllipsis(props: {
  class?: string;
}): JSX.Element;
declare function PaginationItems(props: {
  siblingCount?: number;
  boundaryCount?: number;
  renderItem?: (page: number) => JSX.Element;
  renderEllipsis?: (side: "start" | "end") => JSX.Element;
}): JSX.Element;
declare function PaginationPrevious(props: Omit<ButtonProps, "variant" | "size">): JSX.Element;
declare function PaginationNext(props: Omit<ButtonProps, "variant" | "size">): JSX.Element;
//#endregion
//#region src/components/navigation-menu.d.ts
interface NavigationMenuProps {
  "aria-label": string;
  value?: string | null;
  defaultValue?: string | null;
  onValueChange?: (value: string | null) => void;
  loop?: boolean;
  class?: string;
  viewportClass?: string;
  children?: JSX.Element;
}
declare function NavigationMenu(props: NavigationMenuProps): JSX.Element;
declare function NavigationMenuList(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
interface NavigationMenuItemProps {
  value: string;
  disabled?: boolean;
  class?: string;
  children?: JSX.Element;
}
declare function NavigationMenuItem(props: NavigationMenuItemProps): JSX.Element;
declare function navigationMenuTriggerClass(open: boolean, className?: string): string;
interface NavigationMenuTriggerProps extends Omit<ButtonProps, "variant"> {}
declare function NavigationMenuTrigger(props: NavigationMenuTriggerProps): JSX.Element;
declare function NavigationMenuContent(props: ViewProps): JSX.Element;
interface NavigationMenuLinkProps extends ButtonProps {
  active?: boolean;
  closeOnSelect?: boolean;
}
declare function NavigationMenuLink(props: NavigationMenuLinkProps): JSX.Element;
declare function NavigationMenuIndicator(props: ViewProps): JSX.Element;
/** The shared Popover content already is the native viewport. */
declare function NavigationMenuViewport(): JSX.Element;
//#endregion
//#region src/components/number-field.d.ts
interface NumberFieldProps extends Omit<InputProps, "class" | "onInput" | "placeholder" | "value"> {
  /** `undefined` selects uncontrolled mode; `null` represents an empty field. */
  value?: number | null;
  defaultValue?: number | null;
  min?: number;
  max?: number;
  step?: number;
  largeStep?: number;
  locale?: string;
  formatOptions?: Intl.NumberFormatOptions;
  placeholder?: string;
  changeOnWheel?: boolean;
  onValueChange?(value: number | null): void;
  class?: string;
  inputClass?: string;
  incrementLabel?: string;
  decrementLabel?: string;
  "aria-label": string;
}
/** Locale-aware numeric input with explicit native stepping semantics. */
declare function NumberField(props: NumberFieldProps): JSX.Element;
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
//#region src/components/progress.d.ts
interface ProgressValueDetails {
  value: number;
  min: number;
  max: number;
  percent: number;
}
interface ProgressRootProps extends Omit<ViewProps, "children" | "class" | "role"> {
  value?: number;
  minValue?: number;
  maxValue?: number;
  indeterminate?: boolean;
  /** Direct accessible label; visual labels do not rely on DOM id wiring. */
  label?: string;
  getValueLabel?: (details: ProgressValueDetails) => string;
  children?: JSX.Element;
  class?: string;
}
declare function normalizeProgressValue(value: number | undefined, minValue: number | undefined, maxValue: number | undefined): ProgressValueDetails;
/** Semantic progress state with explicit, composable visual parts. */
declare function ProgressRoot(props: ProgressRootProps): JSX.Element;
declare function ProgressTrack(props: ViewProps): JSX.Element;
declare function ProgressFill(props: ViewProps): JSX.Element;
declare function ProgressLabel(props: TextProps): JSX.Element;
declare function ProgressValueLabel(props: TextProps): JSX.Element;
interface ProgressProps extends Omit<ProgressRootProps, "children" | "class"> {
  /** Classes applied to the visual track, preserving the original shorthand. */
  class?: string;
}
/** Compact progress bar; use ProgressRoot and parts for custom composition. */
declare function Progress(props: ProgressProps): JSX.Element;
//#endregion
//#region src/components/rating-state.d.ts
declare function normalizeRatingMax(max: number | undefined): number;
declare function clampRatingValue(value: number | undefined, max: number): number;
declare function ratingLabel(value: number): string;
//#endregion
//#region src/components/rating.d.ts
interface RatingProps {
  value?: number;
  defaultValue?: number;
  max?: number;
  disabled?: boolean;
  readOnly?: boolean;
  allowClear?: boolean;
  label: string;
  class?: string;
  size?: number;
  onValueChange?: (value: number) => void;
}
declare function Rating(props: RatingProps): JSX.Element;
//#endregion
//#region src/components/resizable.d.ts
type ResizableDirection = "horizontal" | "vertical";
interface ResizablePanelDefinition {
  id: string;
  defaultSize: number;
  minSize?: number;
  maxSize?: number;
}
type ResizablePanelSizes = Readonly<Record<string, number>>;
declare function validateResizableSizes(panels: readonly ResizablePanelDefinition[], sizes?: ResizablePanelSizes): ResizablePanelSizes;
interface ResizablePanelState {
  readonly panels: readonly ResizablePanelDefinition[];
  sizes(): ResizablePanelSizes;
  size(id: string): number;
  pairRange(before: string, after: string): {
    min: number;
    max: number;
  };
  resizePair(before: string, after: string, beforeSize: number): boolean;
  resetPair(before: string, after: string): boolean;
}
declare function createResizablePanelState(options: {
  panels: readonly ResizablePanelDefinition[];
  value?: () => ResizablePanelSizes | undefined;
  defaultValue?: ResizablePanelSizes;
  onValueChange?: (sizes: ResizablePanelSizes) => void;
}): ResizablePanelState;
interface ResizablePanelGroupProps {
  panels: readonly ResizablePanelDefinition[];
  children?: JSX.Element;
  direction?: ResizableDirection;
  value?: ResizablePanelSizes;
  defaultValue?: ResizablePanelSizes;
  onValueChange?: (sizes: ResizablePanelSizes) => void;
  "aria-label": string;
  class?: string;
}
declare function ResizablePanelGroup(props: ResizablePanelGroupProps): JSX.Element;
declare function ResizablePanel(props: {
  id: string;
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
interface ResizableHandleProps {
  before: string;
  after: string;
  "aria-label": string;
  keyboardStep?: number;
  class?: string;
}
declare function ResizableHandle(props: ResizableHandleProps): JSX.Element;
//#endregion
//#region src/components/search-field.d.ts
interface SearchFieldProps extends Omit<InputProps, "class" | "onInput" | "ref" | "value"> {
  /** `undefined` selects uncontrolled mode. */
  value?: string;
  defaultValue?: string;
  onValueChange?(value: string): void;
  /** Called when Enter is pressed with the current query. */
  onSearch?(value: string): void;
  onClear?(): void;
  clearLabel?: string;
  class?: string;
  /** Background utility for the complete search field. Defaults to `bg-input`. */
  surfaceClass?: string;
  inputClass?: string;
  inputRef?: (input: Handle) => void;
}
/** A native search input with consistent clear, Escape, and submit behavior. */
declare function SearchField(props: SearchFieldProps): JSX.Element;
//#endregion
//#region src/components/select.d.ts
interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}
interface SelectProps extends PopupMotionProps {
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
interface ToggleGroupBaseProps {
  disabled?: boolean;
  "aria-label"?: string;
  variant?: "default" | "outline";
  size?: "sm" | "default" | "lg";
  spacing?: 0 | 1 | 2;
  loop?: boolean;
  class?: string;
  children?: JSX.Element;
}
type ToggleGroupProps = ToggleGroupBaseProps & ({
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
declare function ToggleGroup(props: ToggleGroupProps): JSX.Element;
interface ToggleGroupItemProps {
  value: string;
  disabled?: boolean;
  variant?: "default" | "outline" | "accent";
  size?: "sm" | "default" | "lg";
  class?: string;
  children?: JSX.Element;
}
declare function ToggleGroupItem(props: ToggleGroupItemProps): JSX.Element;
//#endregion
//#region src/components/separator.d.ts
interface SeparatorProps extends Omit<ViewProps, "aria-hidden" | "aria-orientation" | "class" | "role"> {
  /** Direction of the dividing line. */
  orientation?: "horizontal" | "vertical";
  /**
   * Decorative separators stay out of the semantic tree. Set to false when
   * the divider represents a meaningful boundary between regions or controls.
   */
  decorative?: boolean;
  class?: string;
}
/** A visual divider with an opt-in semantic separator contract. */
declare function Separator(props: SeparatorProps): JSX.Element;
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
//#region src/components/sidebar.d.ts
interface SidebarSearchGroup<Item> {
  label: string;
  items: readonly Item[];
}
/**
 * Filter grouped sidebar data without taking ownership of routing or identity.
 * Group labels participate in matching so a query can reveal a whole section.
 */
declare function filterSidebarGroups<Item>(groups: readonly SidebarSearchGroup<Item>[], query: string, searchableText: (item: Item) => string): SidebarSearchGroup<Item>[];
interface SidebarProps extends Omit<ViewProps, "class"> {
  class?: string;
}
/** Structural application sidebar. State, routing and width remain explicit. */
declare function Sidebar(props: SidebarProps): JSX.Element;
declare function SidebarHeader(props: ViewProps): JSX.Element;
declare function SidebarSearch(props: SearchFieldProps): JSX.Element;
interface SidebarContentProps extends ScrollAreaProps {
  contentClass?: string;
}
/** The only scrolling region in a standard sidebar; header/footer stay fixed. */
declare function SidebarContent(props: SidebarContentProps): JSX.Element;
declare function SidebarGroup(props: ViewProps): JSX.Element;
declare function SidebarGroupLabel(props: TextProps): JSX.Element;
interface SidebarMenuButtonProps extends Omit<ButtonProps$1, "class" | "unstyled"> {
  class?: string;
}
/** Consistent navigation row; applications still own activation and routing. */
declare function SidebarMenuButton(props: SidebarMenuButtonProps): JSX.Element;
declare function SidebarEmpty(props: {
  title?: string;
  description?: string;
  class?: string;
}): JSX.Element;
declare function SidebarFooter(props: ViewProps): JSX.Element;
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
//#region src/components/table.d.ts
interface TableProps extends Omit<ViewProps, "class"> {
  class?: string;
  /** Minimum intrinsic width of the rows before horizontal scrolling begins. */
  contentClass?: string;
}
/**
 * A horizontally scrollable table surface.
 *
 * Wabou has no implicit HTML table layout. Columns align because every row
 * uses the same flex-cell anatomy; applications can override individual cell
 * widths with the usual flex and width utilities.
 */
declare function Table(props: TableProps): JSX.Element;
declare function TableHeader(props: ViewProps): JSX.Element;
declare function TableBody(props: ViewProps): JSX.Element;
declare function TableFooter(props: ViewProps): JSX.Element;
interface TableRowProps extends Omit<ViewProps, "class"> {
  class?: string;
  selected?: boolean;
}
declare function TableRow(props: TableRowProps): JSX.Element;
interface TableHeadProps extends Omit<TextProps, "class"> {
  class?: string;
}
declare function TableHead(props: TableHeadProps): JSX.Element;
interface TableCellProps extends Omit<ViewProps, "class"> {
  class?: string;
}
declare function TableCell(props: TableCellProps): JSX.Element;
declare function TableCaption(props: TextProps): JSX.Element;
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
  motion?: false | NotificationMotionOptions;
}
/** Render a non-blocking stack of styled toasts on the floating plane. */
declare function Toaster(props: ToasterProps): JSX.Element;
//#endregion
//#region src/components/toolbar.d.ts
type ToolbarOrientation = "horizontal" | "vertical";
interface ToolbarProps {
  "aria-label": string;
  /** Semantic role used by composite controls built on the toolbar primitive. */
  role?: "toolbar" | "menubar";
  orientation?: ToolbarOrientation;
  loop?: boolean;
  class?: string;
  children?: JSX.Element;
}
/** A compact command surface with one native tab stop and arrow navigation. */
declare function Toolbar(props: ToolbarProps): JSX.Element;
interface ToolbarButtonProps extends Omit<ButtonProps, "focusOrder"> {}
declare function ToolbarButton(props: ToolbarButtonProps): JSX.Element;
interface ToolbarToggleProps extends Omit<ToolbarButtonProps, "aria-pressed" | "onClick" | "variant"> {
  pressed?: boolean;
  defaultPressed?: boolean;
  onPressedChange?(pressed: boolean): void;
}
declare function ToolbarToggle(props: ToolbarToggleProps): JSX.Element;
declare function ToolbarGroup(props: {
  "aria-label"?: string;
  class?: string;
  children?: JSX.Element;
}): JSX.Element;
declare function ToolbarSeparator(props: {
  class?: string;
}): JSX.Element;
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
interface TooltipProps extends PopupMotionProps {
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
//#region src/components/tree-view.d.ts
interface TreeNode {
  id: string;
  label: string;
  disabled?: boolean;
  children?: readonly TreeNode[];
}
interface VisibleTreeNode {
  node: TreeNode;
  parentId: string | null;
  level: number;
  position: number;
  setSize: number;
}
interface TreeModel {
  get(id: string): TreeNode | undefined;
  parent(id: string): string | null | undefined;
  firstChild(id: string): string | undefined;
  isBranch(id: string): boolean;
  visible(expandedIds: readonly string[]): readonly VisibleTreeNode[];
}
/** Validates a nested tree once and provides deterministic visible traversal. */
declare function createTreeModel(nodes: readonly TreeNode[]): TreeModel;
interface TreeItemRenderState {
  expanded: boolean;
  selected: boolean;
  level: number;
}
interface TreeViewProps {
  items: readonly TreeNode[];
  "aria-label": string;
  expandedIds?: readonly string[];
  defaultExpandedIds?: readonly string[];
  onExpandedChange?(ids: readonly string[]): void;
  /** `undefined` selects uncontrolled mode; `null` is a controlled empty selection. */
  selectedId?: string | null;
  defaultSelectedId?: string | null;
  onSelectedChange?(id: string | null): void;
  renderItem?(node: TreeNode, state: TreeItemRenderState): JSX.Element;
  class?: string;
  itemClass?: string;
}
/** A single-select tree with explicit data, expansion, and native focus routing. */
declare function TreeView(props: TreeViewProps): JSX.Element;
//#endregion
//#region src/components/index.d.ts
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
//#endregion
//#region src/integrations/standard-schema.d.ts
interface StandardSchemaIssue {
  readonly message: string;
  readonly path?: ReadonlyArray<PropertyKey | {
    readonly key: unknown;
  }>;
}
type StandardSchemaResult<Output> = {
  readonly value: Output;
  readonly issues?: undefined;
} | {
  readonly issues: readonly StandardSchemaIssue[];
};
/** Structural subset of Standard Schema V1 used by synchronous form drafts. */
interface StandardSchema<Input, Output = Input> {
  readonly "~standard": {
    readonly version: 1;
    readonly vendor: string;
    readonly validate: (value: unknown) => StandardSchemaResult<Output> | Promise<StandardSchemaResult<Output>>;
    readonly types?: {
      readonly input: Input;
      readonly output: Output;
    };
  };
}
/**
 * Adapt any synchronous Standard Schema V1 implementation (including Valibot,
 * Zod, and ArkType) to `createFormDraft` without coupling Wabou to its API.
 */
declare function createStandardSchemaValidator<T extends Record<PropertyKey, unknown>>(schema: StandardSchema<T, unknown>): (value: Readonly<T>) => FormDraftErrors<T>;
//#endregion
//#region src/integrations/tanstack-table.d.ts
interface TanStackDataTableOptions<TData> {
  /** Static data or a reactive accessor. */
  data: readonly TData[] | Accessor<readonly TData[]>;
  columns: readonly ColumnDef<TData, unknown>[];
  getRowId?: (row: TData, index: number, parent?: Row$1<TData>) => string;
  enableRowSelection?: boolean | ((row: Row$1<TData>) => boolean);
  initialSorting?: SortingState;
  initialGlobalFilter?: string;
  initialRowSelection?: RowSelectionState;
}
/** Column definition re-exported so ordinary consumers only import `@wabou/ui`. */
type TanStackDataTableColumn<TData, TValue = unknown> = ColumnDef<TData, TValue>;
interface TanStackDataTable<TData> {
  /** The framework-agnostic TanStack instance for advanced capabilities. */
  readonly table: Table$1<TData>;
  /** Reactive rows after filtering and sorting. */
  readonly rows: Accessor<readonly Row$1<TData>[]>;
  readonly sorting: Accessor<SortingState>;
  readonly setSorting: Setter<SortingState>;
  readonly globalFilter: Accessor<string>;
  readonly setGlobalFilter: Setter<string>;
  readonly rowSelection: Accessor<RowSelectionState>;
  readonly setRowSelection: Setter<RowSelectionState>;
  readonly selectedCount: Accessor<number>;
}
/**
 * Solid's reactive ownership around TanStack Table's DOM-independent core.
 *
 * Wabou deliberately owns no duplicate sorting, filtering, or selection state
 * machine here. Applications retain the native renderer and component layer,
 * while TanStack owns the mature data model.
 */
declare function createTanStackDataTable<TData>(options: TanStackDataTableOptions<TData>): TanStackDataTable<TData>;
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
export { Accordion, AccordionContent, AccordionItem, AccordionProps, AccordionTrigger, AccordionType, type ActiveResult, AdaptiveSplitPane, AdaptiveSplitPaneDetail, AdaptiveSplitPaneMain, Alert, AlertDescription, AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogProps, AlertDialogTitle, AlertProps, AlertTitle, AlertVariant, AnimationControls, type AnimationFrameCallback, AnimationOptions, AnimationState, AnimationType, AnimationValue, AspectRatio, AspectRatioProps, Attachment, AttachmentAction, AttachmentActions, AttachmentContent, AttachmentDescription, AttachmentGroup, AttachmentMedia, AttachmentOrientation, AttachmentProps, AttachmentSize, AttachmentState, AttachmentTitle, Avatar, AvatarGroup, AvatarGroupCount, AvatarProps, AvatarSize, Badge, BadgeProps, BadgeVariant, BaseRootRoute, BaseRoute, Breadcrumb, BreadcrumbEllipsis, BreadcrumbEllipsisProps, BreadcrumbItem, BreadcrumbLink, BreadcrumbLinkProps, BreadcrumbList, BreadcrumbPage, BreadcrumbPageProps, BreadcrumbProps, BreadcrumbSeparator, BreadcrumbSeparatorProps, Bubble, BubbleContent, BubbleGroup, BubbleProps, BubbleReactions, BubbleVariant, Button, ButtonGroup, ButtonGroupProps, ButtonGroupSeparator, ButtonGroupSeparatorProps, ButtonGroupText, ButtonProps, ButtonSize, ButtonVariant, Calendar, CalendarDate, CalendarLabels, CalendarProps, Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardProps, CardTitle, Carousel, CarouselApi, CarouselContent, CarouselContentProps, CarouselItem, CarouselItemProps, CarouselNext, CarouselOrientation, CarouselPrevious, CarouselProps, Center, Checkbox, type CheckboxProps, CodeEditor, type CodeEditorProps, Collapsible, CollapsibleContent, CollapsiblePresence, CollapsibleProps, CollapsibleTrigger, Column, Combobox, ComboboxOption, ComboboxProps, Command, CommandItem, CommandProps, type ComponentsElevation, ComponentsProvider, type ComponentsProviderProps, type ComponentsTheme, ConfigEditor, ConfigEditorProps, type ContainerMatch, type ContainerSizeQuery, ContextMenu, ContextMenuProps, ContextMenuTriggerProps, CreateToastsOptions, DatePicker, DatePickerProps, type DelayedOpenController, type DelayedOpenOptions, Dialog, type ModalControls as DialogControls, DialogDescription, DialogDescription as SheetDescription, DialogFooter, DialogFooter as SheetFooter, DialogHeader, DialogHeader as SheetHeader, DialogProps, DialogScrollBody, DialogScrollBody as SheetScrollBody, DialogTitle, DialogTitle as SheetTitle, DirectoryPicker, DirectoryPickerProps, Drawer, DrawerClose, DrawerDescription, DrawerDirection, DrawerFooter, DrawerHandle, DrawerHeader, DrawerOpenChangeReason, DrawerProps, DrawerTitle, DropdownMenu, DropdownMenuItem, DropdownMenuKeyEvent, DropdownMenuProps, Easing, EasingFunction, Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaProps, EmptyMediaVariant, EmptyProps, EmptyTitle, EmptyVariant, FORM_ERROR, Field, FieldContent, FieldDescription, FieldError, FieldErrorLike, FieldGroup, FieldLabel, FieldLabelProps, FieldLegend, FieldOrientation, FieldSeparator, FieldSet, FieldTitle, type FocusResult, type FocusWithinResult, type FormDraft, type FormDraftErrors, type FormDraftFieldUpdater, type FormDraftOptions, Fps, FpsProps, HoverCard, HoverCardProps, HoverCardTriggerProps, Icon, type IconProps, Image, type ImageProps, type ImageSource, Input, InputGroup, InputGroupAddon, InputGroupAddonAlign, InputGroupAddonProps, InputGroupButton, InputGroupInput, InputGroupOrientation, InputGroupProps, InputGroupText, InputGroupTextArea, InputOTP, InputOTPGroup, InputOTPProps, InputOTPSeparator, InputOTPSlot, InputOTPSlotProps, InputProps, Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader, ItemMedia, ItemMediaVariant, ItemProps, ItemSeparator, ItemSize, ItemTitle, ItemVariant, Kbd, KbdGroup, type KeyedSelection, type KeyedSelectionOptions, KeyframeAnimationOptions, Label, LabelProps, type LayoutProps, LoopOptions, Marker, MarkerContent, MarkerIcon, MarkerVariant, Menubar, MenubarMenu, MenubarMenuProps, MenubarProps, Message, MessageAlign, MessageAvatar, MessageContent, MessageFooter, MessageGroup, MessageHeader, MessageProps, MessageScrollDirection, MessageScroller, MessageScrollerButton, MessageScrollerButtonProps, MessageScrollerContent, MessageScrollerControls, MessageScrollerItem, MessageScrollerProps, MessageScrollerViewport, MessageScrollerViewportProps, Modal, type ModalMotionOptions, type ModalProps, type MotionConfig, MotionConfigProvider, type MotionConfigProviderProps, MotionInterpolationOptions, NavigationMenu, NavigationMenuContent, NavigationMenuIndicator, NavigationMenuItem, NavigationMenuItemProps, NavigationMenuLink, NavigationMenuLinkProps, NavigationMenuList, NavigationMenuProps, NavigationMenuTrigger, NavigationMenuTriggerProps, NavigationMenuViewport, NetworkImage, type NetworkImageProps, type NetworkImageSource, type NotificationMotionOptions, NotificationRegion, type NotificationRegionProps, type Notifications, NumberField, NumberFieldProps, type OverlayLayer, OverlayPlaneProvider, PageHeader, PageHeaderProps, PageViewport, PageViewportProps, Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationItems, PaginationLink, PaginationLinkProps, PaginationNext, PaginationPrevious, PaginationProps, type PaginationRangeItem, PasswordInput, PasswordInputProps, Path, PathBuilder, type PathProps, Popover, PopoverDescription, PopoverFooter, PopoverHeader, PopoverProps, PopoverTitle, PopupMotionProps, type PressResult, Button$1 as PrimitiveButton, type ButtonProps$1 as PrimitiveButtonProps, type ButtonState as PrimitiveButtonState, Link as PrimitiveLink, type LinkProps as PrimitiveLinkProps, PasswordInput$1 as PrimitivePasswordInput, type PasswordInputProps$1 as PrimitivePasswordInputProps, Popover$1 as PrimitivePopover, type PopoverProps$1 as PrimitivePopoverProps, TextArea as PrimitiveTextArea, type TextAreaProps as PrimitiveTextAreaProps, TextInput as PrimitiveTextInput, type TextInputProps as PrimitiveTextInputProps, Progress, ProgressFill, ProgressLabel, ProgressProps, ProgressRoot, ProgressRootProps, ProgressTrack, ProgressValueDetails, ProgressValueLabel, Pulse, PulseOptions, RadioGroup, RadioGroupItem, type RadioGroupItemProps, type RadioGroupProps, Rating, RatingProps, ReactiveAnimation, ReactiveTransition, RepeatType, ResizableDirection, ResizableHandle, ResizableHandleProps, ResizablePanel, ResizablePanelDefinition, ResizablePanelGroup, ResizablePanelGroupProps, ResizablePanelSizes, ResizablePanelState, ResponsiveGrid, ResponsiveGridColumnCount, ResponsiveGridProps, ResponsiveGridRemainder, ResponsiveGridState, type RetainedItem, type RetainedItems, Ripple, type RippleProps, RotationAnimation, RotationOptions, RouteActiveOptions, RouterProvider, RouterProviderProps, Row, ScrollArea, type ScrollAreaProps, SearchField, SearchFieldProps, Select, SelectOption, SelectProps, Separator, SeparatorProps, Sheet, SheetProps, SheetSide, Sidebar, SidebarContent, SidebarContentProps, SidebarEmpty, SidebarFooter, SidebarGroup, SidebarGroupLabel, SidebarHeader, SidebarMenuButton, SidebarMenuButtonProps, SidebarProps, SidebarSearch, SidebarSearchGroup, Skeleton, type SkeletonProps, Slider, SliderProps, Spin, Spinner, SplitPane, SplitPaneAside, SplitPaneMain, StandardSchema, StandardSchemaIssue, StandardSchemaResult, Svg, type SvgProps, SweepAnimation, SweepAxis, SweepGeometry, SweepOptions, Switch, SwitchProps, Table, TableBody, TableCaption, TableCell, TableCellProps, TableFooter, TableHead, TableHeadProps, TableHeader, TableProps, TableRow, TableRowProps, Tabs, TabsContent, TabsList, type TabsProps, type TabsResult, TabsTrigger, type TabsTriggerProps, TanStackDataTable, TanStackDataTableColumn, TanStackDataTableOptions, Text, TextArea$1 as TextArea, TextAreaProps$1 as TextAreaProps, type TextProps, TitleBar, TitleBarDragRegion, TitleBarDragRegionProps, TitleBarProps, ToastAction, ToastInput, ToastVariant, Toaster, ToasterProps, Toasts, Toggle, ToggleGroup, ToggleGroupItem, type ToggleGroupItemProps, type ToggleGroupProps, type ToggleProps, Toolbar, ToolbarButton, ToolbarButtonProps, ToolbarGroup, ToolbarOrientation, ToolbarProps, ToolbarSeparator, ToolbarToggle, ToolbarToggleProps, Tooltip, TooltipProps, TooltipTriggerProps, TransitionOptions, type TransitionPresence, type TransitionPresenceOptions, TreeItemRenderState, TreeModel, TreeNode, TreeView, TreeViewProps, type VectorPath, type VectorPathPaint, View, type ViewProps, VisibleTreeNode, type WabouClassList, WabouDataRouter, type WabouStyle, WindowFrame, WindowFrameProps, alertColors, animate, animateKeyframes, aspectRatioStyle, attachmentClass, attachmentMediaClass, badgeClass, bubbleClass, bubbleContentClass, clampPage, clampRatingValue, componentsElevation, createActive, createAnimationFrame, createButton, createContainerMatch, createDataRouter, createDelayedOpenController, createDelayedOpenController as createTooltipDelayController, createFocus, createFocusWithin, createFormDraft, createHover, createInterpolation, createKeyedSelection, createKeyframeAnimation, createLoop, createMeasuredSize, createMemoryHistory, createNotifications, createOverlayLayer, createPaginationRange, createPresence, createPress, createPulse, createResizablePanelState, createRetainedItems, createRotation, createScrollReset, createShortcuts, createStandardSchemaValidator, createSweep, createTabs, createTanStackDataTable, createToasts, createTransition, createTransitionPresence, createTreeModel, drawerDragOffset, drawerShouldDismiss, emptyClass, emptyMediaClass, fieldClass, fieldErrorLabel, filterCommandItems, filterSidebarGroups, inputGroupAddonClass, inputGroupClass, isMessageScrollNearEnd, itemClass, itemMediaClass, messageClass, messageScrollRange, moveMenuHighlight, navigationMenuTriggerClass, nextAccordionValue, normalizeCarouselIndex, normalizeOtpValue, normalizePageCount, normalizeProgressValue, normalizeRatingMax, normalizeSweepGeometry, notFound, pageHeaderClass, pageViewportClass, pageViewportContentClass, index_d_exports as primitives, ratingLabel, reconcileCommandHighlight, redirect, responsiveGridColumnCount, responsiveGridRemainderCount, titleBarClass, titleBarDragRegionLayoutStyle, titleBarLayoutStyle, uniqueFieldErrors, useComponentsTheme, useLoaderData, useLocation, useMessageScroller, useMotionConfig, useNavigate, useParams, useReducedMotion, useResponsiveGrid, useRouteActive, useRouter, useRouterState, validateResizableSizes, windowFrameBackdropClassList, windowFrameClientClassList, windowFrameShadows };
//# sourceMappingURL=index.d.mts.map