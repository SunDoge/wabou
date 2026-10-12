import { $t as FocusResult, A as createPress, An as Path, Ar as createLoop, At as createMeasuredSize, B as PointAnchor, Bn as SvgProps, Br as useMotionConfig, Bt as FormDraftErrors, C as RetainedItem, Cn as ImageResourceHandle, Cr as SweepGeometry, Ct as ModalProps, Dn as NativeWidgetProps, Dr as animateKeyframes, E as ActiveResult, En as NativeWidgetConfig, Er as animate, Et as ContainerSizeQuery, F as PopoverMotionOptions, Fn as RichText, Fr as createTransition, Ft as KeyedSelection, Gn as TextInputProps, Gt as ImageResourceRequest, Ht as FormDraftOptions, I as PopoverProps$1, In as RichTextProps, Ir as normalizeSweepGeometry, It as KeyedSelectionOptions, Jn as VectorPathPaint, Jt as createNetworkImageResource, K as OverlayLayer, Kn as TextProps, Kt as OwnedImageResource, Ln as RichTextSpan, Lr as MotionConfig, Lt as createKeyedSelection, Mn as PathProps, Mr as createPulse, Mt as Column, N as createPresence, Nr as createRotation, Nt as LayoutProps, O as PressResult, Or as createInterpolation, Pn as ProjectionBoundary, Pr as createSweep, Pt as Row, Qn as WabouStyle, Qt as createHover, Rn as RichTextSpanProps, Rr as MotionConfigProvider, Rt as FORM_ERROR, S as ScrollAreaProps, Sn as ImageResourceErrorEvent, Sr as SweepAxis, St as ModalOpenChangeReason, T as createRetainedItems, Tn as NativeWidget, Tr as TransitionOptions, Tt as ContainerMatch, Un as TextAreaProps$1, Ut as createFormDraft, Vn as Text, Vr as useReducedMotion, Vt as FormDraftFieldUpdater, Wt as ImageResourceDescriptor, Xn as ViewProps, Xt as releaseImageResource, Y as OverlayPlaneProvider, Yn as View, Yt as createOwnedImageResource, Z as createOverlayLayer, Zn as WabouClassList, _ as createShortcuts, _n as EditorProps, _r as ReactiveTransition, _t as Modal, ar as AnimationState, b as createScrollReset, bn as Image, br as RotationOptions, cr as Easing, ct as Notifications, d as createTabs, dr as LoopOptions, dt as Pulse, en as FocusWithinResult, et as NotificationDismissReason, fr as MotionInterpolationOptions, gn as Editor, gr as ReactiveAnimation, hr as PulseOptions, ht as Spin, i as createTransitionPresence, ir as AnimationOptions, it as NotificationPlacement, jn as PathBuilder, jr as createNativeLoopAnimation, jt as Center, k as createActive, kn as PasswordInputProps$1, kr as createKeyframeAnimation, kt as createContainerMatch, ln as ButtonProps$1, lr as EasingFunction, mn as createButton, mr as NativeLoopAnimationOptions, mt as RippleProps, n as TransitionPresence, nn as createFocusWithin, nr as createAnimationFrame, or as AnimationType, ot as NotificationRegion, pr as NativeLoopAnimation, pt as Ripple, qn as VectorPath, qt as createFileImageResource, r as TransitionPresenceOptions, rn as CollapsiblePresence, rr as AnimationControls, rt as NotificationMotionOptions, sn as ButtonKeyEvent, sr as AnimationValue, st as NotificationRegionProps, t as index_d_exports, tn as createFocus, tr as AnimationFrameCallback, u as TabsResult, un as ButtonState, ur as KeyframeAnimationOptions, ut as createNotifications, vn as Icon, vr as RepeatType, vt as ModalControls, w as RetainedItems, wn as ImageResourceReadyEvent, wr as SweepOptions, x as ScrollArea, xn as ImageProps, xr as SweepAnimation, xt as ModalMotionOptions, yn as IconProps, yr as RotationAnimation, z as Placement, zn as Svg, zr as MotionConfigProviderProps, zt as FormDraft } from "./index-BKDcOGe8.mjs";
import { FileDropPosition, Kv, KvKey, PickDirectoryOptions } from "@wabou/core";
import { Shadow, WabouStyle as WabouStyle$1 } from "@wabou/core/style";
import { Accessor, JSX as JSX$1, ParentProps, Setter } from "solid-js";
import { Handle, LayoutRect, WabouPointerEvent } from "@wabou/core/renderer";
import { CalendarDate } from "@internationalized/date";
import { QrCodeGenerateData } from "uqr";
import { ColumnDef, Row as Row$1, RowSelectionState, SortingState, Table as Table$1 } from "@tanstack/table-core";
import { RouterHistory, createMemoryHistory } from "@tanstack/history";
import { AnyRoute, AnyRouter, BaseRootRoute, BaseRoute, RouterConstructorOptions, RouterCore, TrailingSlashOption, notFound, redirect } from "@tanstack/router-core";
export * from "@wabou/core";
export * from "@wabou/core/i18n";
//#region src/components/tone.d.ts
/** Semantic color shared by compact component surfaces and indicators. */
export type ComponentTone = "neutral" | "accent" | "success" | "danger";
export type ComponentTonePresentation = "text" | "surface" | "solid" | "indicator";
/**
 * Maps semantic intent to the built-in Wabou theme contract.
 *
 * Keeping this recipe in one place prevents badges, icon tiles and status
 * indicators from drifting into slightly different colors for the same state.
 */
export declare function componentToneClass(tone: ComponentTone, presentation: ComponentTonePresentation): string;
//#endregion
//#region src/components/activity-status.d.ts
export type ActivityStatusTone = Exclude<ComponentTone, "neutral"> | "muted";
export interface ActivityStatusIndicatorProps {
  animated?: boolean;
  tone?: ActivityStatusTone;
  class?: string;
}
/** A consistently sized status dot. Animation is explicit so idle state never
 * keeps the native frame clock alive accidentally. */
export declare function ActivityStatusIndicator(props: ActivityStatusIndicatorProps): JSX$1.Element;
export interface ActivityStatusProps extends Omit<ViewProps, "children" | "class"> {
  label: string;
  animated?: boolean;
  tone?: ActivityStatusTone;
  class?: string;
  textClass?: string;
}
/** Shrink-safe inline progress or presence status for agent and desktop UI. */
export declare function ActivityStatus(props: ActivityStatusProps): JSX$1.Element;
//#endregion
//#region src/components/alert.d.ts
export type AlertVariant = "default" | "info" | "success" | "warning" | "error" | "destructive";
export type AlertSize = "sm" | "default" | "lg";
export declare function alertColors(variant: AlertVariant): {
  container: string;
  title: string;
  description: string;
};
export interface AlertProps extends Omit<ViewProps, "role"> {
  variant?: AlertVariant;
  size?: AlertSize;
  /** Optional leading graphic. Pass false to suppress the semantic default. */
  icon?: JSX$1.Element | false;
  /** Edge-to-edge status strip without container chrome. */
  banner?: boolean;
  /** Show a close control and delegate visibility to the owner. */
  onClose?: () => void;
  /** Convenience form; compound usage can render AlertTitle directly. */
  title?: string;
}
/** A native status callout with shadcn-compatible compound composition. */
export declare function Alert(props: AlertProps): JSX$1.Element;
export declare function AlertTitle(props: TextProps): JSX$1.Element;
export declare function AlertDescription(props: TextProps): JSX$1.Element;
/** Recovery and acknowledgement controls belonging to an alert. */
export declare function AlertActions(props: ViewProps): JSX$1.Element;
//#endregion
//#region src/components/button.d.ts
export type ButtonVariant = "default" | "secondary" | "outline" | "ghost" | "destructive";
export type ButtonSize = "sm" | "default" | "lg" | "icon";
export interface ButtonProps extends Omit<ButtonProps$1, "variant" | "tone" | "unstyled"> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  class?: string;
  style?: ButtonProps$1["style"];
  /** Disable activation and replace the leading content with a native spinner. */
  loading?: boolean;
  /** Visible label used while loading. Defaults to the ordinary children. */
  loadingLabel?: string;
}
export declare function Button(props: ButtonProps): JSX$1.Element;
//#endregion
//#region src/components/dialog.d.ts
export interface DialogProps extends Omit<ModalProps, "contentClass"> {
  contentClass?: string;
}
export declare function Dialog(props: DialogProps): JSX$1.Element;
export declare function DialogHeader(props: {
  children?: JSX$1.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function DialogFooter(props: {
  children?: JSX$1.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
/**
 * The shrinking, independently scrollable region between a dialog's fixed
 * header and footer. The dialog surface must have a bounded or maximum height.
 */
export interface DialogScrollBodyProps extends Omit<ScrollAreaProps, "class" | "contentClass"> {
  class?: string;
  contentClass?: string;
}
export declare function DialogScrollBody(props: DialogScrollBodyProps): import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function DialogTitle(props: {
  children?: JSX$1.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function DialogDescription(props: {
  children?: JSX$1.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
//#endregion
//#region src/components/alert-dialog.d.ts
export interface AlertDialogProps extends Omit<DialogProps, "children" | "contentRole"> {
  children?: JSX$1.Element | ((controls: ModalControls) => JSX$1.Element);
}
/**
 * A blocking confirmation dialog. Backdrop dismissal is disabled by default so
 * every close is an intentional cancel, confirmation, or Escape action.
 */
export declare function AlertDialog(props: AlertDialogProps): JSX$1.Element;
export declare function AlertDialogAction(props: ButtonProps): JSX$1.Element;
export declare function AlertDialogCancel(props: ButtonProps): JSX$1.Element;
export declare const AlertDialogHeader: typeof DialogHeader;
export declare const AlertDialogFooter: typeof DialogFooter;
export declare const AlertDialogTitle: typeof DialogTitle;
export declare const AlertDialogDescription: typeof DialogDescription;
//#endregion
//#region src/components/aspect-ratio.d.ts
export interface AspectRatioProps extends Omit<ViewProps, "style"> {
  /** Width divided by height. Defaults to a square. */
  ratio?: number;
  style?: WabouStyle$1;
}
export declare function aspectRatioStyle(ratio: number | undefined, style?: WabouStyle$1): WabouStyle$1;
/** A native Taffy aspect-ratio constraint with explicit overflow ownership. */
export declare function AspectRatio(props: AspectRatioProps): JSX$1.Element;
//#endregion
//#region src/components/attachment.d.ts
export type AttachmentState = "idle" | "uploading" | "processing" | "error" | "done";
export type AttachmentSize = "default" | "sm" | "xs";
export type AttachmentOrientation = "horizontal" | "vertical";
interface AttachmentContextValue {
  state(): AttachmentState;
  size(): AttachmentSize;
  orientation(): AttachmentOrientation;
}
export declare function attachmentClass(options: {
  state?: AttachmentState;
  size?: AttachmentSize;
  orientation?: AttachmentOrientation;
  class?: string;
}): string;
export interface AttachmentProps extends Omit<ViewProps, "class"> {
  state?: AttachmentState;
  size?: AttachmentSize;
  orientation?: AttachmentOrientation;
  class?: string;
}
/** File/task summary anatomy adapted from shadcn without DOM data selectors. */
export declare function Attachment(props: AttachmentProps): JSX$1.Element;
export declare function attachmentMediaClass(variant: "icon" | "image", context: Pick<AttachmentContextValue, "state" | "size" | "orientation">, className?: string): string;
export declare function AttachmentMedia(props: {
  children?: JSX$1.Element;
  variant?: "icon" | "image";
  class?: string;
}): JSX$1.Element;
export declare function AttachmentContent(props: ViewProps): JSX$1.Element;
export declare function AttachmentTitle(props: TextProps): JSX$1.Element;
export declare function AttachmentDescription(props: TextProps): JSX$1.Element;
export declare function AttachmentActions(props: ViewProps): JSX$1.Element;
export declare function AttachmentAction(props: ButtonProps): JSX$1.Element;
export declare function attachmentGroupClass(className?: string): string;
export declare function AttachmentGroup(props: ViewProps): JSX$1.Element;
//#endregion
//#region src/components/avatar.d.ts
export type AvatarSize = "sm" | "default" | "lg";
export interface AvatarProps extends Omit<ViewProps, "children"> {
  image?: ImageResourceHandle;
  /** Full accessible name and source for generated initials. */
  name?: string;
  /** Accessible label override when it should differ from `name`. */
  alt?: string;
  /** Explicit visual fallback. Defaults to initials derived from `name`. */
  fallback?: string;
  size?: AvatarSize;
}
/** Derive at most two stable initials without depending on Intl. */
export declare function avatarInitials(name: string): string;
export declare function Avatar(props: AvatarProps): JSX$1.Element;
export type AvatarGroupProps = ViewProps;
export declare function AvatarGroup(props: AvatarGroupProps): JSX$1.Element;
export type AvatarGroupCountProps = ViewProps;
export declare function AvatarGroupCount(props: AvatarGroupCountProps): JSX$1.Element;
//#endregion
//#region src/components/badge.d.ts
export type BadgeVariant = "default" | "secondary" | "outline" | "ghost" | "link" | "info" | "success" | "destructive";
export type BadgeSize = "sm" | "default" | "lg";
export interface BadgeProps extends Omit<TextProps, "class"> {
  variant?: BadgeVariant;
  size?: BadgeSize;
  /** Typography weight selected without competing utility declarations. */
  weight?: "normal" | "medium";
  class?: string;
}
export declare function badgeClass(variant?: BadgeVariant, weight?: NonNullable<BadgeProps["weight"]>, className?: string, size?: BadgeSize): string;
/** Compact status text with shadcn-compatible visual variants. */
export declare function Badge(props: BadgeProps): JSX$1.Element;
//#endregion
//#region src/components/button-group-context.d.ts
type ButtonGroupOrientation = "horizontal" | "vertical";
type ButtonGroupButtonSize = "sm" | "default" | "lg" | "icon";
type ButtonGroupButtonVariant = "default" | "secondary" | "outline" | "ghost" | "destructive";
//#endregion
//#region src/components/button-group.d.ts
export interface ButtonGroupProps extends Omit<ViewProps, "class"> {
  orientation?: ButtonGroupOrientation;
  size?: ButtonGroupButtonSize;
  variant?: ButtonGroupButtonVariant;
  disabled?: boolean;
  class?: string;
}
/** A single bordered control surface composed from ordinary Wabou buttons. */
export declare function ButtonGroup(props: ButtonGroupProps): JSX$1.Element;
export declare function ButtonGroupText(props: TextProps): JSX$1.Element;
export interface ButtonGroupSeparatorProps {
  orientation?: "horizontal" | "vertical";
  class?: string;
}
export declare function ButtonGroupSeparator(props: ButtonGroupSeparatorProps): JSX$1.Element;
//#endregion
//#region src/components/card.d.ts
export type CardVariant = "raised" | "filled" | "outline" | "plain";
export type CardSize = "sm" | "default" | "lg";
export interface CardProps extends Omit<ViewProps, "class"> {
  variant?: CardVariant;
  size?: CardSize;
  class?: string;
}
export declare function Card(props: CardProps): JSX$1.Element;
export declare function CardHeader(props: ViewProps): JSX$1.Element;
export declare function CardTitle(props: TextProps): JSX$1.Element;
export declare function CardDescription(props: TextProps): JSX$1.Element;
/** Top-end action slot owned by the relative CardHeader surface. */
export declare function CardAction(props: ViewProps): JSX$1.Element;
export declare function CardContent(props: ViewProps): JSX$1.Element;
export declare function CardFooter(props: ViewProps): JSX$1.Element;
//#endregion
//#region src/components/carousel.d.ts
export type CarouselOrientation = "horizontal" | "vertical";
export interface CarouselApi {
  selectedIndex(): number;
  itemCount(): number;
  canScrollPrevious(): boolean;
  canScrollNext(): boolean;
  scrollPrevious(): void;
  scrollNext(): void;
  scrollTo(index: number): void;
}
export declare function normalizeCarouselIndex(index: number, count: number, loop: boolean): number;
export interface CarouselProps {
  "aria-label": string;
  index?: number;
  defaultIndex?: number;
  onIndexChange?(index: number): void;
  orientation?: CarouselOrientation;
  loop?: boolean;
  setApi?(api: CarouselApi): void;
  class?: string;
  children?: JSX$1.Element;
}
/** A native snapping carousel with captured pointer dragging and keyboard navigation. */
export declare function Carousel(props: CarouselProps): JSX$1.Element;
export interface CarouselContentProps extends Omit<ViewProps, "transform"> {
  trackClass?: string;
  dragThreshold?: number;
}
export declare function CarouselContent(props: CarouselContentProps): JSX$1.Element;
export interface CarouselItemProps extends ViewProps {
  "aria-label"?: string;
}
export declare function CarouselItem(props: CarouselItemProps): JSX$1.Element;
export declare function CarouselPrevious(props: Omit<ButtonProps, "children"> & {
  children?: JSX$1.Element;
}): JSX$1.Element;
export declare function CarouselNext(props: Omit<ButtonProps, "children"> & {
  children?: JSX$1.Element;
}): JSX$1.Element;
//#endregion
//#region src/components/chart.d.ts
export interface ChartSeriesConfig {
  label: string;
  colorClass: string;
}
export type ChartConfig = Readonly<Record<string, ChartSeriesConfig>>;
export interface ChartContainerProps {
  config: ChartConfig;
  label: string;
  class?: string;
  style?: ViewProps["style"];
  children?: JSX$1.Element;
}
export declare function ChartContainer(props: ChartContainerProps): JSX$1.Element;
export declare function useChartConfig(): ChartConfig;
export declare function ChartLegend(props: {
  class?: string;
}): JSX$1.Element;
export declare function ChartEmpty(props: {
  message?: string;
  class?: string;
}): JSX$1.Element;
//#endregion
//#region src/components/code-block.d.ts
export interface CodeBlockProps extends Omit<ViewProps, "children"> {
  code: string;
  language?: string;
  copyable?: boolean;
  copyLabel?: string;
  copiedLabel?: string;
}
export declare function CodeBlock(props: CodeBlockProps): JSX$1.Element;
//#endregion
//#region src/components/menu-state.d.ts
interface MenuStateItem {
  id: string;
  label: string;
  disabled?: boolean;
}
type MenuMove = "first" | "last" | "next" | "previous";
/** Resolve one keyboard move without coupling menu state to rendering. */
export declare function moveMenuHighlight(items: readonly MenuStateItem[], current: string | undefined, move: MenuMove): string | undefined;
//#endregion
//#region src/components/command-state.d.ts
interface CommandStateItem extends MenuStateItem {
  keywords?: readonly string[];
}
export declare function filterCommandItems<T extends CommandStateItem>(items: readonly T[], query: string): T[];
export declare function reconcileCommandHighlight<T extends CommandStateItem>(items: readonly T[], highlighted: string | undefined): string | undefined;
//#endregion
//#region src/components/command.d.ts
export interface CommandItem extends CommandStateItem {
  description?: string;
  /** Human-readable platform shortcut, such as `Ctrl K` or `⌘ K`. */
  shortcut?: string;
  onSelect?: () => void;
}
export interface CommandProps {
  items: readonly CommandItem[];
  "aria-label": string;
  query?: string;
  defaultQuery?: string;
  placeholder?: string;
  emptyText?: string;
  loading?: boolean;
  loadingText?: string;
  error?: unknown;
  errorText?: string;
  retryLabel?: string;
  class?: string;
  listClass?: string;
  onQueryChange?: (query: string) => void;
  onAction?: (id: string) => void;
  onRetry?: () => void;
  onDismiss?: () => void;
  inputRef?: (node: Handle) => void;
}
export interface CommandListProps {
  items: readonly CommandItem[];
  "aria-label": string;
  highlighted?: string;
  emptyText?: string;
  class?: string;
  itemClass?: string;
  loading?: boolean;
  loadingText?: string;
  error?: unknown;
  errorText?: string;
  retryLabel?: string;
  onRetry?: () => void;
  onHighlightChange?: (id: string) => void;
  onAction?: (id: string) => void;
  renderLeading?: (item: CommandItem) => JSX$1.Element;
}
/** Reusable command-result surface for search fields and inline completions. */
export declare function CommandList(props: CommandListProps): JSX$1.Element;
export interface CommandListNavigationOptions {
  onAction?: (id: string) => void;
  onDismiss?: () => void;
}
/**
 * Shared command-list navigation for any focus owner, including native editors.
 * The caller forwards key events while the list remains a passive popup.
 */
export declare function createCommandListNavigation(items: Accessor<readonly CommandStateItem[]>, options?: CommandListNavigationOptions): {
  highlighted: import("solid-js").SourceAccessor<string | undefined>;
  setHighlighted: import("solid-js").Setter<string | undefined>;
  select: (id: string | undefined) => boolean;
  move: (direction: "first" | "last" | "next" | "previous") => boolean;
  handleKeyDown: (event: {
    key: string;
    preventDefault(): void;
  }) => boolean;
};
/** Searchable command list whose filtering and keyboard behavior are host-independent. */
export declare function Command(props: CommandProps): JSX$1.Element;
//#endregion
//#region src/components/popover.d.ts
export type PopoverProps = PopoverProps$1;
/** Shared visual-motion contract for components backed by a popup surface. */
export interface PopupMotionProps {
  /** Override the default popup transition, or disable it without changing presence semantics. */
  motion?: false | PopoverMotionOptions;
}
/** A ready-to-use floating surface backed by Wabou's collision-aware overlay. */
export declare function Popover(props: PopoverProps): JSX$1.Element;
export declare function PopoverHeader(props: {
  children?: JSX$1.Element;
  class?: string;
}): JSX$1.Element;
export declare function PopoverTitle(props: {
  children?: JSX$1.Element;
  class?: string;
}): JSX$1.Element;
export declare function PopoverDescription(props: {
  children?: JSX$1.Element;
  class?: string;
}): JSX$1.Element;
export declare function PopoverFooter(props: {
  children?: JSX$1.Element;
  class?: string;
}): JSX$1.Element;
//#endregion
//#region src/components/select-semantics.d.ts
type PickerTriggerVariant = "default" | "ghost";
//#endregion
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
export declare function Combobox(props: ComboboxProps): JSX$1.Element;
//#endregion
//#region src/components/content-state.d.ts
export type ContentStateKind = "empty" | "loading" | "error";
export interface ContentStateProps extends Omit<ViewProps, "children" | "role"> {
  state: ContentStateKind;
  title: string;
  description?: string;
  /** Standard compact recovery or next-step action. */
  action?: {
    label: string;
    onAction(): void;
  };
  /** Lazily render media inside the state's reactive owner. */
  renderMedia?: () => JSX$1.Element;
  /** Lazily render custom actions inside the state's reactive owner. */
  renderAction?: () => JSX$1.Element;
}
/** Mutually exclusive loading, empty, or error state for a bounded region. */
export declare function ContentState(props: ContentStateProps): JSX$1.Element;
export interface ResourceBoundaryProps extends Omit<ViewProps, "children" | "role"> {
  /** True while the resource is performing its initial load or refreshing. */
  loading: boolean;
  /** A rejected resource value. `undefined` and `null` mean no error. */
  error?: unknown;
  /** Whether a usable resource value currently exists, including an empty collection. */
  hasContent: boolean;
  loadingTitle: string;
  errorTitle: string;
  emptyTitle: string;
  loadingDescription?: string;
  emptyDescription?: string;
  retryLabel?: string;
  onRetry?: () => void;
  renderContent(): JSX$1.Element;
  renderEmptyMedia?: () => JSX$1.Element;
  renderErrorMedia?: () => JSX$1.Element;
}
/**
 * Mutually exclusive async resource boundary.
 *
 * Existing content remains mounted during a background refresh. This avoids
 * replacing a useful inspector or list with a loading spinner every time its
 * resource is refreshed.
 */
export declare function ResourceBoundary(props: ResourceBoundaryProps): JSX$1.Element;
//#endregion
//#region src/components/dropdown-menu.d.ts
export interface DropdownMenuItem extends MenuStateItem {
  /** Optional decorative Lucide/static SVG shown in the shared leading slot. */
  icon?: string;
  /** Checked state. Defining this reserves the shared leading status slot. */
  checked?: boolean;
  description?: string;
  /** Human-readable platform shortcut, such as `Ctrl K` or `⌘ K`. */
  shortcut?: string;
  destructive?: boolean;
  separatorBefore?: boolean;
  onSelect?: () => void;
}
export interface DropdownMenuTriggerProps {
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
}
export interface DropdownMenuProps extends PopupMotionProps {
  trigger(props: DropdownMenuTriggerProps): JSX$1.Element;
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
export interface DropdownMenuKeyEvent {
  key: string;
  readonly defaultPrevented?: boolean;
  preventDefault(): void;
}
/** A compact action menu with native focus, typeahead, and overlay routing. */
export declare function DropdownMenu(props: DropdownMenuProps): JSX$1.Element;
//#endregion
//#region src/components/context-menu.d.ts
export interface ContextMenuTriggerProps {
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
export interface ContextMenuProps extends PopupMotionProps {
  trigger(props: ContextMenuTriggerProps): JSX$1.Element;
  items: readonly DropdownMenuItem[];
  "aria-label": string;
  onAction?: (id: string) => void;
  contentClass?: string;
  contentShadows?: readonly Shadow[] | null;
}
/** An action menu anchored to the native secondary-click coordinate. */
export declare function ContextMenu(props: ContextMenuProps): JSX$1.Element;
//#endregion
//#region src/components/copy-button.d.ts
export interface CopyButtonProps extends Omit<ButtonProps, "onClick"> {
  value: string;
  idleLabel?: string;
  copiedLabel?: string;
  copiedChildren?: JSX$1.Element;
  onCopied?: () => void;
  onCopyError?: (error: unknown) => void;
}
export declare function CopyButton(props: CopyButtonProps): JSX$1.Element;
//#endregion
//#region src/integrations/standard-schema.d.ts
export interface StandardSchemaIssue {
  readonly message: string;
  readonly path?: ReadonlyArray<PropertyKey | {
    readonly key: unknown;
  }>;
}
export type StandardSchemaResult<Output> = {
  readonly value: Output;
  readonly issues?: undefined;
} | {
  readonly issues: readonly StandardSchemaIssue[];
};
/** Structural subset of Standard Schema V1 used by synchronous form drafts. */
export interface StandardSchema<Input, Output = Input> {
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
export declare function createStandardSchemaValidator<T extends Record<PropertyKey, unknown>>(schema: StandardSchema<T, unknown>): (value: Readonly<T>) => FormDraftErrors<T>;
//#endregion
//#region src/integrations/tanstack-table.d.ts
declare const dataTableFeatures: {
  coreReactivityFeature: import("@tanstack/table-core/reactivity").TableReactivityBindings;
  columnFilteringFeature: import("@tanstack/table-core").TableFeature;
  columnVisibilityFeature: import("@tanstack/table-core").TableFeature;
  globalFilteringFeature: import("@tanstack/table-core").TableFeature;
  rowSelectionFeature: import("@tanstack/table-core").TableFeature;
  rowSortingFeature: import("@tanstack/table-core").TableFeature;
  coreRowModel: (table: Table$1<any, any>) => () => import("@tanstack/table-core").RowModel<any, any>;
  filteredRowModel: (table: Table$1<any, any>) => () => import("@tanstack/table-core").RowModel<any, any>;
  sortedRowModel: (table: Table$1<any, any>) => () => import("@tanstack/table-core").RowModel<any, any>;
  filterFns: {
    arrIncludes: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    arrIncludesAll: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    arrHas: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    arrIncludesSome: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    between: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    betweenInclusive: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    empty: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    endsWith: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    equals: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    equalsString: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    equalsStringSensitive: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    inDateRange: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    inNumberRange: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    includesString: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    includesStringSensitive: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    notEmpty: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    startsWith: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    weakEquals: import("@tanstack/table-core").CreatedFilterFn<any, any>;
  };
  sortFns: {
    alphanumeric: import("@tanstack/table-core").CreatedSortFn<any, any>;
    alphanumericCaseSensitive: import("@tanstack/table-core").CreatedSortFn<any, any>;
    basic: import("@tanstack/table-core").CreatedSortFn<any, any>;
    datetime: import("@tanstack/table-core").CreatedSortFn<any, any>;
    text: import("@tanstack/table-core").CreatedSortFn<any, any>;
    textCaseSensitive: import("@tanstack/table-core").CreatedSortFn<any, any>;
  };
};
type DataTableFeatures = typeof dataTableFeatures;
export type TanStackDataTableRow<TData extends object> = Row$1<DataTableFeatures, TData>;
export interface TanStackDataTableOptions<TData extends object> {
  /** Static data or a reactive accessor. */
  data: readonly TData[] | Accessor<readonly TData[]>;
  columns: readonly ColumnDef<DataTableFeatures, TData, unknown>[];
  getRowId?: (row: TData, index: number, parent?: TanStackDataTableRow<TData>) => string;
  enableRowSelection?: boolean | ((row: TanStackDataTableRow<TData>) => boolean);
  initialSorting?: SortingState;
  initialGlobalFilter?: string;
  initialRowSelection?: RowSelectionState;
}
/** Column definition re-exported so ordinary consumers only import `@wabou/ui`. */
export type TanStackDataTableColumn<TData extends object, TValue = unknown> = ColumnDef<DataTableFeatures, TData, TValue>;
export interface TanStackDataTable<TData extends object> {
  /** TanStack v9 instance with filtering, sorting, visibility, and selection. */
  readonly table: Table$1<DataTableFeatures, TData>;
  /** Reactive rows after filtering and sorting. */
  readonly rows: Accessor<readonly TanStackDataTableRow<TData>[]>;
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
export declare function createTanStackDataTable<TData extends object>(options: TanStackDataTableOptions<TData>): TanStackDataTable<TData>;
//#endregion
//#region src/components/data-table.d.ts
export interface DataTableProps<TData extends object> {
  model: TanStackDataTable<TData>;
  "aria-label": string;
  emptyMessage?: string;
  selectable?: boolean;
  renderCell?: (options: {
    value: unknown;
    columnId: string;
    row: TanStackDataTableRow<TData>;
  }) => JSX$1.Element;
}
export type TableSortDirection = "asc" | "desc" | false | undefined;
export declare function TableSortIndicator(props: {
  direction: TableSortDirection | (() => TableSortDirection);
}): JSX$1.Element;
/** Shadcn-style table anatomy backed by the framework-agnostic TanStack core. */
export declare function DataTable<TData extends object>(props: DataTableProps<TData>): JSX$1.Element;
//#endregion
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
export declare function Calendar(props: CalendarProps): JSX$1.Element;
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
export declare function DatePicker(props: DatePickerProps): JSX$1.Element;
//#endregion
//#region src/components/development-diagnostics.d.ts
interface DevServerDiagnostic {
  message: string;
  stack?: string;
  id?: string;
  frame?: string;
  plugin?: string;
  loc?: {
    file?: string;
    line: number;
    column: number;
  };
}
//#endregion
//#region src/components/dev-server-error.d.ts
/** Native equivalent of Vite's browser error overlay. */
export declare function DevServerErrorOverlay(): JSX$1.Element;
//#endregion
//#region src/components/diff-viewer.d.ts
export type DiffFileStatus = "added" | "modified" | "deleted" | "renamed";
export interface DiffFile {
  path: string;
  oldPath?: string;
  status: DiffFileStatus;
  additions: number;
  deletions: number;
  /** Unified patch for this file. Kept out of the visible tree while collapsed. */
  patch: string;
}
export interface DiffViewerLabels {
  filesChanged: (count: number) => string;
  additions: (count: number) => string;
  deletions: (count: number) => string;
  empty: string;
  technicalDetails: string;
}
export interface DiffViewerProps extends Omit<ViewProps, "children"> {
  files: readonly DiffFile[];
  labels?: Partial<DiffViewerLabels>;
  /** Files opened initially. Details remain opt-in by default. */
  defaultExpanded?: readonly string[];
}
/**
 * A progressive-disclosure code change viewer.
 *
 * The summary and file metadata are ordinary Wabou components. Unified patch
 * text is mounted only after disclosure and uses Wabou's native editor viewport
 * for selection, copying, and large documents.
 */
export declare function DiffViewer(props: DiffViewerProps): import("@wabou/core/jsx-runtime").JSX.Element;
//#endregion
//#region src/components/direction.d.ts
export type Direction = "ltr" | "rtl";
export interface DirectionProviderProps {
  dir: Direction;
  children?: JSX$1.Element;
}
/** Own logical direction in JavaScript instead of relying on web inheritance. */
export declare function DirectionProvider(props: DirectionProviderProps): JSX$1.Element;
export declare function useDirection(): Direction;
export interface DirectionalRowProps extends Omit<ViewProps, "class"> {
  class?: string;
}
export declare function DirectionalRow(props: DirectionalRowProps): JSX$1.Element;
export interface DirectionalTextProps extends Omit<TextProps, "class"> {
  class?: string;
}
export declare function DirectionalText(props: DirectionalTextProps): JSX$1.Element;
//#endregion
//#region src/components/input.d.ts
export interface InputProps extends TextInputProps {
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
export declare function Input(props: InputProps): JSX$1.Element;
export interface PasswordInputProps extends PasswordInputProps$1 {
  class?: string;
}
/** A native secret input whose value never crosses into JavaScript. */
export declare function PasswordInput(props: PasswordInputProps): JSX$1.Element;
export interface TextAreaProps extends TextAreaProps$1 {
  class?: string;
  /** Background utility owned by this textarea. Defaults to `bg-input`. */
  surfaceClass?: string;
  /** Use `none` when an enclosing composition owns the visual surface. */
  chrome?: "default" | "none";
}
export declare function TextArea(props: TextAreaProps): JSX$1.Element;
//#endregion
//#region src/components/directory-picker.d.ts
export interface DirectoryPickerProps extends Omit<InputProps, "class" | "onInput" | "value"> {
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
  /** Called only after the native picker commits a directory selection. */
  onBrowseSelect?: (value: string) => void;
}
/** A controlled path input paired with the operating system directory picker. */
export declare function DirectoryPicker(props: DirectoryPickerProps): JSX$1.Element;
//#endregion
//#region src/components/disclosure.d.ts
export interface CollapsibleProps extends Omit<ViewProps, "children" | "class"> {
  children?: JSX$1.Element;
  open?: boolean;
  defaultOpen?: boolean;
  disabled?: boolean;
  reducedMotion?: boolean;
  onOpenChange?: (open: boolean) => void;
  class?: string;
}
export declare function Collapsible(props: CollapsibleProps): import("@wabou/core/jsx-runtime").JSX$1.Element;
export interface CollapsibleTriggerProps extends Omit<ButtonProps$1, "children" | "class" | "unstyled"> {
  children?: JSX$1.Element;
  /** Render the built-in trailing chevron. Disable when children provide one. */
  indicator?: boolean;
  class?: string;
}
export declare function CollapsibleTrigger(props: CollapsibleTriggerProps): import("@wabou/core/jsx-runtime").JSX$1.Element;
export interface CollapsibleContentProps extends ViewProps {
  duration?: number;
  ease?: Easing;
  animateInitial?: boolean;
}
export declare function CollapsibleContent(props: CollapsibleContentProps): import("@wabou/core/jsx-runtime").JSX$1.Element;
export type AccordionType = "single" | "multiple";
type AccordionValue = string | readonly string[];
export declare function nextAccordionValue(current: AccordionValue, type: AccordionType, item: string, collapsible?: boolean): AccordionValue;
export interface AccordionProps extends Omit<ViewProps, "children" | "class"> {
  children?: JSX$1.Element;
  type?: AccordionType;
  value?: AccordionValue;
  defaultValue?: AccordionValue;
  collapsible?: boolean;
  disabled?: boolean;
  reducedMotion?: boolean;
  onValueChange?: (value: AccordionValue) => void;
  class?: string;
}
export declare function Accordion(props: AccordionProps): import("@wabou/core/jsx-runtime").JSX$1.Element;
export interface AccordionItemProps extends Omit<ViewProps, "children" | "class"> {
  value: string;
  disabled?: boolean;
  children?: JSX$1.Element;
  class?: string;
}
export declare function AccordionItem(props: AccordionItemProps): import("@wabou/core/jsx-runtime").JSX$1.Element;
export interface AccordionTriggerProps extends Omit<ButtonProps$1, "children" | "class" | "unstyled"> {
  children?: JSX$1.Element;
  class?: string;
}
export declare function AccordionTrigger(props: AccordionTriggerProps): import("@wabou/core/jsx-runtime").JSX$1.Element;
export type AccordionContentProps = ViewProps;
export declare function AccordionContent(props: AccordionContentProps): import("@wabou/core/jsx-runtime").JSX$1.Element;
//#endregion
//#region src/components/display.d.ts
interface SkeletonProps {
  class?: string;
  /** Disable the shimmer while preserving the stable loading placeholder. */
  animated?: boolean;
}
export declare function Skeleton(props: SkeletonProps): JSX$1.Element;
export declare function Spinner(props: {
  label?: string;
  class?: string;
  /** Hide spinner semantics when a parent status already announces progress. */
  decorative?: boolean;
  /** Duration of one revolution in seconds. */
  duration?: number;
  /** Playback-rate multiplier. */
  speed?: number;
  paused?: boolean;
}): JSX$1.Element;
type KbdProps = TextProps;
export declare function Kbd(props: KbdProps): JSX$1.Element;
type KbdGroupProps = ViewProps;
export declare function KbdGroup(props: KbdGroupProps): JSX$1.Element;
//#endregion
//#region src/components/drawer.d.ts
export type DrawerDirection = "top" | "right" | "bottom" | "left";
export type DrawerOpenChangeReason = ModalOpenChangeReason | "drag";
export declare function drawerDragOffset(direction: DrawerDirection, rawDelta: number): number;
export declare function drawerShouldDismiss(offset: number, size: number, threshold: number): boolean;
export interface DrawerProps extends Omit<ModalProps, "defaultOpen" | "onOpenChange" | "contentClass" | "contentTransform" | "motion"> {
  defaultOpen?: boolean;
  onOpenChange?(open: boolean, reason: DrawerOpenChangeReason): void;
  direction?: DrawerDirection;
  dismissible?: boolean;
  dismissThreshold?: number;
  contentClass?: string;
}
/** A focus-isolated edge drawer with a captured native drag-to-dismiss gesture. */
export declare function Drawer(props: DrawerProps): JSX$1.Element;
export declare function DrawerHandle(props: ViewProps): JSX$1.Element;
export declare function DrawerHeader(props: ViewProps): JSX$1.Element;
export declare function DrawerFooter(props: ViewProps): JSX$1.Element;
export declare function DrawerTitle(props: TextProps): JSX$1.Element;
export declare function DrawerDescription(props: TextProps): JSX$1.Element;
export declare function DrawerClose(props: ButtonProps): JSX$1.Element;
//#endregion
//#region src/components/drop-zone.d.ts
export interface DropZoneProps extends Omit<ViewProps, "class" | "children"> {
  /** Called with paths accepted by this zone after a native drop. */
  onDrop: (paths: readonly string[]) => void;
  /** Return true for paths this zone accepts. All paths are accepted by default. */
  accept?: (path: string) => boolean;
  /** Reports paths rejected by `accept` without hiding a partially valid drop. */
  onRejected?: (paths: readonly string[]) => void;
  disabled?: boolean;
  label?: string;
  activeLabel?: string;
  description?: string;
  class?: string;
}
/** Inclusive hit test in the logical coordinate space shared with native DnD. */
export declare function pointInLayoutRect(point: FileDropPosition, rect: LayoutRect): boolean;
/**
 * A component-local target for native filesystem drops.
 *
 * Native backends publish window-relative coordinates, so the zone measures
 * its completed native layout before claiming an event. Events without a
 * position are deliberately ignored instead of being delivered ambiguously to
 * every mounted zone.
 */
export declare function DropZone(props: DropZoneProps): JSX$1.Element;
//#endregion
//#region src/components/empty.d.ts
export type EmptyVariant = "surface" | "plain";
export type EmptyMediaVariant = "default" | "icon";
export interface EmptyProps extends Omit<ViewProps, "class"> {
  /** `plain` embeds inside an existing surface without nesting another card. */
  variant?: EmptyVariant;
  class?: string;
}
export declare function emptyClass(variant?: EmptyVariant, className?: string): string;
/** A composable empty-state region based on shadcn's Empty anatomy. */
export declare function Empty(props: EmptyProps): JSX$1.Element;
export declare function EmptyHeader(props: ViewProps): JSX$1.Element;
export interface EmptyMediaProps extends Omit<ViewProps, "class"> {
  variant?: EmptyMediaVariant;
  class?: string;
}
export declare function emptyMediaClass(variant?: EmptyMediaVariant, className?: string): string;
export declare function EmptyMedia(props: EmptyMediaProps): JSX$1.Element;
export declare function EmptyTitle(props: TextProps): JSX$1.Element;
export declare function EmptyDescription(props: TextProps): JSX$1.Element;
export declare function EmptyContent(props: ViewProps): JSX$1.Element;
//#endregion
//#region src/components/label.d.ts
export interface LabelProps extends Omit<TextProps, "class" | "role"> {
  class?: string;
  disabled?: boolean;
  /**
   * Explicit native control target. Unlike HTML `for`, this cannot silently
   * point at a missing string id and remains safe across retained-tree reuse.
   */
  control?: Handle | (() => Handle | undefined);
}
/** Text label that forwards pointer activation to an explicit native control. */
export declare function Label(props: LabelProps): JSX$1.Element;
//#endregion
//#region src/components/forms.d.ts
export type FieldOrientation = "vertical" | "horizontal";
export declare function fieldClass(orientation?: FieldOrientation, invalid?: boolean, className?: string): string;
export declare function Field(props: {
  children?: JSX$1.Element;
  orientation?: FieldOrientation;
  invalid?: boolean;
  required?: boolean;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function FieldSet(props: {
  children?: JSX$1.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function FieldLegend(props: {
  children?: JSX$1.Element;
  variant?: "legend" | "label";
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function FieldGroup(props: {
  children?: JSX$1.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
export interface FieldLabelProps extends LabelProps {}
export declare function FieldLabel(props: FieldLabelProps): import("@wabou/core/jsx-runtime").JSX$1.Element;
export interface LabeledFieldProps {
  label: JSX$1.Element;
  description?: JSX$1.Element;
  invalid?: boolean;
  required?: boolean;
  disabled?: boolean;
  errors?: ReadonlyArray<FieldErrorLike | undefined>;
  class?: string;
  controlRef?: (node: Handle) => void;
  /** Render the native control and attach the supplied ref to its focus owner. */
  renderControl: (ref: (node: Handle) => void) => JSX$1.Element;
}
/**
 * A complete native field whose visible label always focuses its control.
 * This avoids repeating ad-hoc Handle plumbing in every settings surface.
 */
export declare function LabeledField(props: LabeledFieldProps): import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function FieldTitle(props: {
  children?: JSX$1.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function FieldContent(props: {
  children?: JSX$1.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function FieldDescription(props: {
  children?: JSX$1.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
export interface FieldErrorLike {
  message?: string;
}
export declare function uniqueFieldErrors(errors: ReadonlyArray<FieldErrorLike | undefined> | undefined): string[];
export declare function fieldErrorLabel(explicit: string | undefined, children: JSX$1.Element, messages: readonly string[]): string | undefined;
export declare function FieldError(props: {
  children?: JSX$1.Element;
  errors?: ReadonlyArray<FieldErrorLike | undefined>;
  "aria-label"?: string;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function FieldSeparator(props: {
  children?: JSX$1.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
export type InputGroupOrientation = "horizontal" | "vertical";
export type InputGroupVariant = "default" | "quiet";
export type InputGroupAddonAlign = "inline-start" | "inline-end" | "block-start" | "block-end";
export declare function inputGroupClass(orientation: InputGroupOrientation, focused: boolean, invalid: boolean, variant?: InputGroupVariant): string;
export interface InputGroupProps extends Omit<ViewProps, "children"> {
  children?: JSX$1.Element;
  orientation?: InputGroupOrientation;
  variant?: InputGroupVariant;
  invalid?: boolean;
  disabled?: boolean;
  /** Background utility owned by the compound control. Defaults to `bg-input`. */
  surfaceClass?: string;
}
export declare function InputGroup(props: InputGroupProps): import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function InputGroupInput(props: InputProps): import("@wabou/core/jsx-runtime").JSX$1.Element;
export interface InputGroupAddonProps extends ViewProps {
  align?: InputGroupAddonAlign;
  focusControl?: boolean;
}
export declare function inputGroupAddonClass(align: InputGroupAddonAlign): string;
export declare function InputGroupAddon(props: InputGroupAddonProps): import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function InputGroupText(props: {
  children?: JSX$1.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function InputGroupButton(props: ButtonProps): import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function InputGroupTextArea(props: TextAreaProps$1 & {
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
//#endregion
//#region src/components/group-box.d.ts
export type GroupBoxVariant = "normal" | "fill" | "outline";
export interface GroupBoxProps extends Omit<ViewProps, "children" | "class"> {
  title?: JSX$1.Element;
  description?: JSX$1.Element;
  children?: JSX$1.Element;
  variant?: GroupBoxVariant;
  class?: string;
  headerClass?: string;
  contentClass?: string;
}
export declare function groupBoxContentClass(variant?: GroupBoxVariant, className?: string): string;
/** A lightweight titled surface for related controls and settings rows. */
export declare function GroupBox(props: GroupBoxProps): JSX$1.Element;
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
export declare function createDelayedOpenController(options: DelayedOpenOptions): DelayedOpenController;
//#endregion
//#region src/components/hover-card.d.ts
export interface HoverCardTriggerProps {
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
export interface HoverCardProps extends PopupMotionProps {
  "aria-label": string;
  trigger(props: HoverCardTriggerProps): JSX$1.Element;
  children?: JSX$1.Element;
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
export declare function HoverCard(props: HoverCardProps): JSX$1.Element;
//#endregion
//#region src/components/icon-frame.d.ts
export type IconFrameSize = "sm" | "default" | "lg" | "xl";
export type IconFrameVariant = "plain" | "soft" | "solid";
export interface IconFrameProps extends Omit<ViewProps, "children">, Pick<IconProps, "source" | "fill" | "label"> {
  /** Size of the square visual container. */
  size?: IconFrameSize;
  /** Explicit icon size. Defaults are tuned for each container size. */
  iconSize?: number | string;
  /** Color intent, independent from the frame's visual emphasis. */
  tone?: ComponentTone;
  variant?: IconFrameVariant;
  iconClass?: string;
}
/**
 * A square icon surface whose geometry is correct by construction.
 *
 * Use this for standalone icon tiles. Buttons and menu items already own their
 * icon alignment and should continue to use `Icon` directly.
 */
export declare function IconFrame(props: IconFrameProps): import("@wabou/core/jsx-runtime").JSX.Element;
//#endregion
//#region src/components/image-list.d.ts
export interface ImageListProps<T> {
  /** Reactive backing collection. GPUI materializes only visible native rows. */
  items: () => readonly T[];
  getItemKey: (item: T, index: number) => string | number;
  getResource?: (item: T, index: number) => ImageResourceHandle;
  getLabel: (item: T, index: number) => string;
  getDescription?: (item: T, index: number) => string | undefined;
  itemHeight?: number;
  thumbnailWidth?: number;
  thumbnailHeight?: number;
  viewportHeight?: number;
  selectedKey?: string | number;
  onSelectionChange?: (item: T, index: number) => void;
  onResourceReady?: (item: T, index: number, event: ImageResourceReadyEvent) => void;
  onResourceError?: (item: T, index: number, event: ImageResourceErrorEvent) => void;
  accessibilityLabel?: string;
  class?: string;
  renderTrailing?: (item: T, index: number) => JSX$1.Element;
  /** Optional generated thumbnail; bypasses Image resource loading for this row. */
  renderThumbnail?: (item: T, index: number) => JSX$1.Element;
}
/**
 * A virtualized, selectable image list for page strips, albums and file pickers.
 * Resource creation remains owned by the caller; this component only borrows handles.
 */
export declare function ImageList<T>(props: ImageListProps<T>): JSX$1.Element;
//#endregion
//#region src/components/image-viewport.d.ts
export interface ImageViewportSize {
  width: number;
  height: number;
}
export interface ImageViewportPoint {
  x: number;
  y: number;
}
export interface ImageViewportRect extends ImageViewportPoint {
  width: number;
  height: number;
}
export interface ImageViewportTransform {
  readonly viewport: ImageViewportSize;
  readonly image: ImageViewportSize;
  readonly frame: ImageViewportRect;
  readonly scale: number;
  imageToViewport(point: ImageViewportPoint): ImageViewportPoint;
  viewportToImage(point: ImageViewportPoint): ImageViewportPoint;
}
/** Deterministic contain + zoom + pan transform shared by paint and annotations. */
export declare function imageViewportTransform(options: {
  viewport: ImageViewportSize;
  image: ImageViewportSize;
  zoom?: number;
  pan?: ImageViewportPoint;
}): ImageViewportTransform;
export interface ImageViewportProps extends Omit<ViewProps, "children" | "ref" | "onWheel" | "onPointerDown" | "onPointerMove" | "onPointerUp" | "onPointerCancel"> {
  resource?: ImageResourceHandle;
  /** Intrinsic image size in image pixels. */
  imageSize?: ImageViewportSize;
  zoom?: number;
  pan?: ImageViewportPoint;
  pannable?: boolean;
  onPanChange?: (pan: ImageViewportPoint) => void;
  /** Optional replacement for the native Image, useful for generated media. */
  media?: JSX$1.Element;
  children?: JSX$1.Element;
  imageLabel?: string;
  onResourceReady?: (event: ImageResourceReadyEvent) => void;
  onResourceError?: (event: ImageResourceErrorEvent) => void;
}
/** A clipped image-space viewport with one explicit, reusable coordinate model. */
export declare function ImageViewport(props: ImageViewportProps): JSX$1.Element;
export interface AnnotationRegion extends ImageViewportRect {
  id: string;
  label?: string;
}
export declare function clampAnnotationRegion(region: AnnotationRegion, image: ImageViewportSize, minimumSize?: number): AnnotationRegion;
export interface AnnotationLayerProps extends Omit<ViewProps, "children" | "onPointerDown" | "onPointerMove" | "onPointerUp" | "onPointerCancel"> {
  regions: readonly AnnotationRegion[];
  selectedId?: string | null;
  minimumSize?: number;
  createRegionId?: () => string;
  interactionMode?: "edit" | "passthrough";
  onRegionsChange?: (regions: readonly AnnotationRegion[]) => void;
  onSelectedIdChange?: (id: string | null) => void;
}
/** Editable image-space regions composed above an ImageViewport. */
export declare function AnnotationLayer(props: AnnotationLayerProps): JSX$1.Element;
export interface ImageOverlayItem extends ImageViewportRect {
  id: string;
}
export interface ImageOverlayLayerProps<T extends ImageOverlayItem> extends Omit<ViewProps, "children"> {
  items: readonly T[];
  children: (item: T, index: () => number) => JSX$1.Element;
}
/** Read-only image-space overlays sharing the viewport's exact zoom and pan transform. */
export declare function ImageOverlayLayer<T extends ImageOverlayItem>(props: ImageOverlayLayerProps<T>): JSX$1.Element;
//#endregion
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
export declare function InlineEdit(props: InlineEditProps): JSX$1.Element;
//#endregion
//#region src/components/input-otp.d.ts
export declare function normalizeOtpValue(value: string, maxLength: number, allowed?: RegExp): string;
export interface InputOTPProps {
  value?: string;
  defaultValue?: string;
  maxLength: number;
  allowed?: RegExp;
  disabled?: boolean;
  readOnly?: boolean;
  "aria-label": string;
  class?: string;
  inputClass?: string;
  children?: JSX$1.Element;
  onValueChange?: (value: string) => void;
  onComplete?: (value: string) => void;
  ref?: TextInputProps["ref"];
}
export declare function InputOTP(props: InputOTPProps): JSX$1.Element;
export declare function InputOTPGroup(props: ViewProps): JSX$1.Element;
export interface InputOTPSlotProps extends Omit<ViewProps, "children"> {
  index: number;
}
export declare function InputOTPSlot(props: InputOTPSlotProps): JSX$1.Element;
export declare function InputOTPSeparator(props: ViewProps): JSX$1.Element;
//#endregion
//#region src/components/item.d.ts
export type ItemVariant = "default" | "outline" | "muted";
export type ItemSize = "default" | "sm";
export type ItemMediaVariant = "default" | "icon" | "image";
export declare function itemClass(variant?: ItemVariant, size?: ItemSize, className?: string): string;
export interface ItemProps extends Omit<ViewProps, "class"> {
  variant?: ItemVariant;
  size?: ItemSize;
  /** Persistent list selection, independent from transient pointer hover. */
  selected?: boolean;
  /** Disable the complete row subtree, including trailing actions. */
  disabled?: boolean;
  class?: string;
}
/** A composable list row based on shadcn's Item anatomy. */
export declare function Item(props: ItemProps): JSX$1.Element;
export declare function ItemGroup(props: ViewProps): JSX$1.Element;
export declare function ItemSeparator(props: {
  class?: string;
}): JSX$1.Element;
export declare function itemMediaClass(variant?: ItemMediaVariant, className?: string): string;
export declare function ItemMedia(props: {
  children?: JSX$1.Element;
  variant?: ItemMediaVariant;
  class?: string;
}): JSX$1.Element;
export declare function ItemContent(props: ViewProps): JSX$1.Element;
export declare function ItemTitle(props: TextProps): JSX$1.Element;
export declare function ItemDescription(props: TextProps): JSX$1.Element;
export declare function ItemActions(props: ViewProps): JSX$1.Element;
export declare function ItemHeader(props: ViewProps): JSX$1.Element;
export declare const ItemFooter: typeof ItemHeader;
//#endregion
//#region src/components/layout.d.ts
export type ResponsiveGridColumnCount = 1 | 2 | 3 | 4;
export interface ResponsiveGridState {
  columns: Accessor<ResponsiveGridColumnCount>;
  width: Accessor<number>;
  height: Accessor<number>;
}
/** Read the completed native size and active column count of the nearest grid. */
export declare function useResponsiveGrid(): ResponsiveGridState;
export declare function responsiveGridColumnCount(options: {
  width: number;
  minColumnWidth: number;
  gap?: number;
  maxColumns?: ResponsiveGridColumnCount;
  initialColumns?: ResponsiveGridColumnCount;
  itemCount?: number;
  balanceLastRow?: boolean;
}): ResponsiveGridColumnCount;
export declare function responsiveGridRemainderCount(itemCount: number, columns: ResponsiveGridColumnCount): number;
export interface ResponsiveGridProps extends Omit<ViewProps, "children" | "class" | "ref"> {
  children?: JSX$1.Element;
  /** Minimum usable content width for one item, in logical pixels. */
  minColumnWidth: number;
  /** Native row/column gap in logical pixels; also used to select the column count. */
  gap?: number;
  maxColumns?: ResponsiveGridColumnCount;
  /** Safe column count used until the native container has been measured. */
  initialColumns?: ResponsiveGridColumnCount;
  /** Number of rendered cells, used by optional last-row balancing. */
  itemCount?: number;
  /** Reduce the column count when it would leave one orphaned final cell. */
  balanceLastRow?: boolean;
  class?: string;
  ref?: ViewProps["ref"];
}
/**
 * A grid that responds to its own native content box instead of the window.
 *
 * This is important inside sidebars, split panes and dialogs: window media
 * queries do not know how much width the component actually receives.
 */
export declare function ResponsiveGrid(props: ResponsiveGridProps): JSX$1.Element;
/** Fill the unused cells in the final row using the grid's measured columns. */
export declare function ResponsiveGridRemainder(props: {
  itemCount: number;
  class?: string;
}): JSX$1.Element;
/**
 * A horizontal primary/aside boundary with explicit flex shrink semantics.
 * Use `SplitPaneMain` for the elastic region and `SplitPaneAside` for a
 * class-sized fixed rail. Both regions clip at their own boundary, so content
 * cannot paint across the divider or a rounded parent clip.
 */
export declare function SplitPane(props: {
  children?: JSX$1.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function SplitPaneMain(props: {
  children?: JSX$1.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function SplitPaneAside(props: {
  children?: JSX$1.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
/**
 * Master/detail layout whose detail region can move from an inline rail to a
 * modal surface without changing the application's selection model.
 */
export declare function AdaptiveSplitPane(props: {
  children?: JSX$1.Element;
  /** Controlled compact mode. Omit it to measure this pane natively. */
  compact?: boolean;
  /** Inclusive native content width that activates compact mode. */
  compactAt?: number;
  onCompactChange?: (compact: boolean) => void;
  "aria-label"?: string;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function AdaptiveSplitPaneMain(props: {
  children?: JSX$1.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function AdaptiveSplitPaneDetail(props: {
  children?: JSX$1.Element;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  "aria-label": string;
  class?: string;
  modalClass?: string;
}): import("@wabou/core/jsx-runtime").JSX$1.Element;
//#endregion
//#region src/components/listbox.d.ts
export interface ListboxOption {
  value: string;
  label: string;
  /** Accessible identity when the visible label is not unique or sufficiently descriptive. */
  accessibilityLabel?: string;
  description?: string;
  disabled?: boolean;
}
export interface ListboxProps {
  options: readonly ListboxOption[];
  value?: string;
  defaultValue?: string;
  "aria-label": string;
  emptyText?: string;
  class?: string;
  listClass?: string;
  itemClass?: string;
  maxVisibleItems?: number;
  /** Fixed row height used for both layout and viewport calculation. */
  itemHeight?: number;
  /** Explicit scroll viewport height for inspector and split-pane layouts. */
  viewportHeight?: number;
  /** Fill the available flex height instead of deriving a fixed viewport. */
  fill?: boolean;
  /** Receives the focusable listbox handle for dialog and popover composition. */
  ref?: (node: Handle) => void;
  renderLeading?: (option: ListboxOption) => JSX$1.Element;
  renderTrailing?: (option: ListboxOption) => JSX$1.Element;
  onValueChange?: (value: string) => void;
  /** Invoked after pointer or keyboard activation of an enabled option. */
  onAction?: (value: string) => void;
  onDismiss?: () => void;
}
/**
 * Focusable inline single-selection list.
 *
 * Unlike Select, Listbox owns no popup or trigger. It is suitable for dialogs,
 * inspectors and other surfaces where the choices are already visible.
 */
export declare function Listbox(props: ListboxProps): JSX$1.Element;
//#endregion
//#region src/components/markdown.d.ts
export type MarkdownVariant = "document" | "conversation" | "prompt";
export interface MarkdownProps {
  source: string;
  /** Repair an incomplete Markdown tail while text is still arriving. */
  streaming?: boolean;
  /** Document typography by default; conversation and prompt stay message-sized. */
  variant?: MarkdownVariant;
  class?: string;
  "aria-label"?: string;
}
/** Parses GFM in JavaScript and renders native Wabou components, without HTML or a DOM. */
export declare function Markdown(props: MarkdownProps): JSX$1.Element;
//#endregion
//#region src/components/menubar.d.ts
export interface MenubarProps {
  "aria-label": string;
  value?: string | null;
  defaultValue?: string | null;
  onValueChange?: (value: string | null) => void;
  loop?: boolean;
  class?: string;
  children?: JSX$1.Element;
}
/** Persistent application menus with one tab stop and sibling menu switching. */
export declare function Menubar(props: MenubarProps): JSX$1.Element;
export interface MenubarMenuProps extends PopupMotionProps {
  value: string;
  label: string;
  items: readonly DropdownMenuItem[];
  disabled?: boolean;
  onAction?: (id: string) => void;
  children?: JSX$1.Element;
}
export declare function MenubarMenu(props: MenubarMenuProps): JSX$1.Element;
//#endregion
//#region src/components/message.d.ts
export type MessageAlign = "start" | "end";
export type BubbleVariant = "default" | "secondary" | "muted" | "tinted" | "outline" | "ghost" | "destructive";
export declare function MessageGroup(props: ViewProps): JSX$1.Element;
export interface MessageProps extends Omit<ViewProps, "class"> {
  align?: MessageAlign;
  class?: string;
}
export declare function messageClass(align?: MessageAlign, className?: string): string;
export declare function Message(props: MessageProps): JSX$1.Element;
export declare function MessageAvatar(props: ViewProps): JSX$1.Element;
export declare function MessageContent(props: ViewProps): JSX$1.Element;
export declare function MessageHeader(props: TextProps): JSX$1.Element;
export declare const MessageFooter: typeof MessageHeader;
export declare function messageActionsClass(align?: MessageAlign, className?: string): string;
export interface MessageActionsProps extends Omit<ViewProps, "class"> {
  /** Override the containing message direction for a local action rail. */
  align?: MessageAlign;
  /** Keep actions visible, or reveal them while their message is hovered/focused. */
  visibility?: "always" | "interaction";
  class?: string;
}
/** Compact, consistently aligned actions belonging to one message. */
export declare function MessageActions(props: MessageActionsProps): JSX$1.Element;
export declare function BubbleGroup(props: ViewProps): JSX$1.Element;
export interface BubbleProps extends Omit<ViewProps, "class"> {
  variant?: BubbleVariant;
  align?: MessageAlign;
  class?: string;
}
export declare function bubbleClass(variant?: BubbleVariant, align?: MessageAlign, className?: string): string;
export declare function Bubble(props: BubbleProps): JSX$1.Element;
export declare function bubbleContentClass(variant: BubbleVariant, className?: string): string;
export declare function BubbleContent(props: ViewProps): JSX$1.Element;
export declare function BubbleReactions(props: {
  children?: JSX$1.Element;
  side?: "top" | "bottom";
  align?: MessageAlign;
  class?: string;
}): JSX$1.Element;
export type MarkerVariant = "default" | "separator" | "border";
export declare function Marker(props: {
  children?: JSX$1.Element;
  variant?: MarkerVariant;
  class?: string;
}): JSX$1.Element;
export declare function MarkerIcon(props: ViewProps): JSX$1.Element;
export declare function MarkerContent(props: TextProps): JSX$1.Element;
//#endregion
//#region src/components/message-scroller.d.ts
export type MessageScrollDirection = "start" | "end";
export interface MessageScrollIntoViewOptions {
  margin?: number;
  align?: "nearest" | "start";
}
export declare function messageScrollRange(contentHeight: number, viewportHeight: number): number;
export declare function isMessageScrollNearEnd(scrollY: number, contentHeight: number, viewportHeight: number, threshold?: number): boolean;
/** Smallest vertical delta that reveals a target without disturbing visible content. */
export declare function messageScrollRevealDelta(viewport: LayoutRect, target: LayoutRect, margin?: number): number;
/** Vertical delta that places a target at the viewport's reading start. */
export declare function messageScrollStartDelta(viewport: LayoutRect, target: LayoutRect, margin?: number): number;
export interface MessageScrollerControls {
  followingEnd(): boolean;
  canScrollStart(): boolean;
  canScrollEnd(): boolean;
  activeAnchor(): string | undefined;
  scrollTo(direction: MessageScrollDirection): void;
  scrollIntoView(target: Handle, options?: MessageScrollIntoViewOptions): void;
  scrollToAnchor(anchor: string, options?: MessageScrollIntoViewOptions): void;
}
export declare function useMessageScroller(): MessageScrollerControls;
export interface MessageScrollerProps extends ViewProps {
  /** Start attached to the end of the conversation. Defaults to true. */
  followEnd?: boolean;
  /** Distance in logical pixels that still counts as being at the end. */
  endThreshold?: number;
}
export interface MessageAnchorRect {
  id: string;
  rect: LayoutRect;
}
/** Pick the last conversation anchor that has crossed the reading line. */
export declare function activeMessageAnchor(viewport: LayoutRect, anchors: readonly MessageAnchorRect[], offset?: number): string | undefined;
export declare function MessageScroller(props: MessageScrollerProps): JSX$1.Element;
export interface MessageScrollerViewportProps extends ViewProps {}
export declare function MessageScrollerViewport(props: MessageScrollerViewportProps): JSX$1.Element;
export declare function MessageScrollerContent(props: ViewProps): JSX$1.Element;
export interface MessageScrollerItemProps extends ViewProps {
  /** Stable semantic id used by conversation navigation. */
  anchor?: string;
}
export declare function MessageScrollerItem(props: MessageScrollerItemProps): JSX$1.Element;
export interface MessageScrollerButtonProps extends Omit<ButtonProps, "onClick"> {
  direction?: MessageScrollDirection;
  onClick?: ButtonProps["onClick"];
}
export declare function MessageScrollerButton(props: MessageScrollerButtonProps): JSX$1.Element;
export interface MessageScrollerNavigatorItem {
  id: string;
  label: string;
}
export interface MessageScrollerNavigatorProps {
  items: readonly MessageScrollerNavigatorItem[];
  "aria-label": string;
  itemAriaLabel(item: MessageScrollerNavigatorItem, index: number): string;
  minItems?: number;
  class?: string;
  railClass?: string;
}
/** Compact anchor rail for navigating long retained conversations. */
export declare function MessageScrollerNavigator(props: MessageScrollerNavigatorProps): JSX$1.Element;
//#endregion
//#region src/components/pagination-state.d.ts
type PaginationRangeItem = number | "ellipsis-start" | "ellipsis-end";
export declare function normalizePageCount(count: number): number;
export declare function clampPage(page: number, count: number): number;
/**
 * Produces a stable, 1-indexed page range with explicit start/end ellipses.
 * A single hidden page is shown directly instead of being replaced by an
 * ellipsis, which keeps every item actionable and avoids misleading gaps.
 */
export declare function createPaginationRange(options: {
  count: number;
  page: number;
  siblingCount?: number;
  boundaryCount?: number;
}): PaginationRangeItem[];
//#endregion
//#region src/components/navigation.d.ts
export interface BreadcrumbProps extends Omit<ViewProps, "class" | "role"> {
  class?: string;
}
export declare function Breadcrumb(props: BreadcrumbProps): JSX$1.Element;
export declare function BreadcrumbList(props: ViewProps): JSX$1.Element;
export declare function BreadcrumbItem(props: ViewProps): JSX$1.Element;
export interface BreadcrumbLinkProps extends Omit<ButtonProps, "class" | "role" | "variant" | "size"> {
  class?: string;
}
export declare function BreadcrumbLink(props: BreadcrumbLinkProps): JSX$1.Element;
export interface BreadcrumbPageProps extends Omit<TextProps, "class" | "role"> {
  class?: string;
}
export declare function BreadcrumbPage(props: BreadcrumbPageProps): JSX$1.Element;
export interface BreadcrumbSeparatorProps extends Omit<ViewProps, "class" | "role"> {
  class?: string;
}
export declare function BreadcrumbSeparator(props: BreadcrumbSeparatorProps): JSX$1.Element;
export interface BreadcrumbEllipsisProps extends Omit<ViewProps, "class" | "role" | "children"> {
  class?: string;
}
export declare function BreadcrumbEllipsis(props: BreadcrumbEllipsisProps): JSX$1.Element;
export interface PaginationProps {
  children?: JSX$1.Element;
  class?: string;
  "aria-label"?: string;
  count: number;
  page?: number;
  defaultPage?: number;
  disabled?: boolean;
  onPageChange?: (page: number) => void;
}
export declare function Pagination(props: PaginationProps): JSX$1.Element;
export declare function PaginationContent(props: ViewProps): JSX$1.Element;
export declare function PaginationItem(props: ViewProps): JSX$1.Element;
export interface PaginationLinkProps extends Omit<ButtonProps, "variant" | "size"> {
  active?: boolean;
  /** Selects this page when used inside a managed Pagination. */
  page?: number;
}
export declare function PaginationLink(props: PaginationLinkProps): JSX$1.Element;
export declare function PaginationEllipsis(props: {
  class?: string;
}): JSX$1.Element;
export declare function PaginationItems(props: {
  siblingCount?: number;
  boundaryCount?: number;
  renderItem?: (page: number) => JSX$1.Element;
  renderEllipsis?: (side: "start" | "end") => JSX$1.Element;
}): JSX$1.Element;
export declare function PaginationPrevious(props: Omit<ButtonProps, "variant" | "size">): JSX$1.Element;
export declare function PaginationNext(props: Omit<ButtonProps, "variant" | "size">): JSX$1.Element;
//#endregion
//#region src/components/navigation-menu.d.ts
export interface NavigationMenuProps {
  "aria-label": string;
  value?: string | null;
  defaultValue?: string | null;
  onValueChange?: (value: string | null) => void;
  loop?: boolean;
  class?: string;
  viewportClass?: string;
  children?: JSX$1.Element;
}
export declare function NavigationMenu(props: NavigationMenuProps): JSX$1.Element;
export declare function NavigationMenuList(props: {
  children?: JSX$1.Element;
  class?: string;
}): JSX$1.Element;
export interface NavigationMenuItemProps {
  value: string;
  disabled?: boolean;
  class?: string;
  children?: JSX$1.Element;
}
export declare function NavigationMenuItem(props: NavigationMenuItemProps): JSX$1.Element;
export declare function navigationMenuTriggerClass(open: boolean, className?: string): string;
export interface NavigationMenuTriggerProps extends Omit<ButtonProps, "variant"> {}
export declare function NavigationMenuTrigger(props: NavigationMenuTriggerProps): JSX$1.Element;
export declare function NavigationMenuContent(props: ViewProps): JSX$1.Element;
export interface NavigationMenuLinkProps extends ButtonProps {
  active?: boolean;
  closeOnSelect?: boolean;
}
export declare function NavigationMenuLink(props: NavigationMenuLinkProps): JSX$1.Element;
export declare function NavigationMenuIndicator(props: ViewProps): JSX$1.Element;
/** The shared Popover content already is the native viewport. */
export declare function NavigationMenuViewport(): JSX$1.Element;
//#endregion
//#region src/components/number-field.d.ts
export interface NumberFieldProps extends Omit<InputProps, "class" | "onInput" | "placeholder" | "value"> {
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
export declare function NumberField(props: NumberFieldProps): JSX$1.Element;
//#endregion
//#region src/components/page.d.ts
export declare const pageViewportClass: (className?: string) => string;
export declare const pageViewportContentClass: (className?: string) => string;
export interface PageViewportProps extends Omit<ScrollAreaProps, "class" | "contentClass" | "ref"> {
  children?: JSX$1.Element;
  /** Classes applied to the bounded scrolling viewport. */
  class?: string;
  /** Classes applied to the intrinsic-height page content wrapper. */
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
export declare function PageViewport(props: PageViewportProps): JSX$1.Element;
export declare const pageHeaderClass: (className?: string, stacked?: boolean) => string;
export declare const pageHeaderTitleClass: () => string;
export declare const pageHeaderDescriptionClass: (stacked?: boolean) => string;
export interface PageHeaderProps {
  title: string;
  description?: string;
  /** Optional content rendered immediately after the title. */
  titleAdornment?: JSX$1.Element;
  /** Page-level controls rendered at the trailing edge. */
  actions?: JSX$1.Element;
  /** Stack actions below the title for narrow application windows. */
  stacked?: boolean;
  class?: string;
}
/** Consistent page title, supporting text and trailing application actions. */
export declare function PageHeader(props: PageHeaderProps): JSX$1.Element;
//#endregion
//#region src/components/onboarding.d.ts
export interface OnboardingProps extends Omit<PageViewportProps, "class" | "contentClass"> {
  class?: string;
  contentClass?: string;
}
/**
 * Full-height first-run boundary with native scrolling and a readable measure.
 * Content stays centered when it fits and remains reachable when it grows.
 */
export declare function Onboarding(props: OnboardingProps): JSX$1.Element;
export declare function OnboardingHeader(props: ViewProps): JSX$1.Element;
export declare function OnboardingHeading(props: ViewProps): JSX$1.Element;
export declare function OnboardingTitle(props: TextProps): JSX$1.Element;
export declare function OnboardingDescription(props: TextProps): JSX$1.Element;
export declare function OnboardingFooter(props: ViewProps): JSX$1.Element;
//#endregion
//#region src/components/progress.d.ts
export interface ProgressValueDetails {
  value: number;
  min: number;
  max: number;
  percent: number;
}
export interface ProgressRootProps extends Omit<ViewProps, "children" | "class" | "role"> {
  value?: number;
  minValue?: number;
  maxValue?: number;
  indeterminate?: boolean;
  /** Direct accessible label; visual labels do not rely on DOM id wiring. */
  label?: string;
  getValueLabel?: (details: ProgressValueDetails) => string;
  children?: JSX$1.Element;
  class?: string;
}
export type ProgressSize = "xs" | "sm" | "default" | "lg";
export declare function normalizeProgressValue(value: number | undefined, minValue: number | undefined, maxValue: number | undefined): ProgressValueDetails;
/** Semantic progress state with explicit, composable visual parts. */
export declare function ProgressRoot(props: ProgressRootProps): JSX$1.Element;
export interface ProgressTrackProps extends ViewProps {
  size?: ProgressSize;
}
export declare function ProgressTrack(props: ProgressTrackProps): JSX$1.Element;
export declare function ProgressFill(props: ViewProps): JSX$1.Element;
export declare function ProgressLabel(props: TextProps): JSX$1.Element;
export declare function ProgressValueLabel(props: TextProps): JSX$1.Element;
export interface ProgressProps extends Omit<ProgressRootProps, "children" | "class"> {
  /** Classes applied to the visual track, preserving the original shorthand. */
  class?: string;
  size?: ProgressSize;
}
/** Compact progress bar; use ProgressRoot and parts for custom composition. */
export declare function Progress(props: ProgressProps): JSX$1.Element;
export interface ProgressCircleProps extends Omit<ProgressRootProps, "children" | "class"> {
  class?: string;
  size?: ProgressSize;
}
/** Compact circular progress indicator using the same semantic range contract. */
export declare function ProgressCircle(props: ProgressCircleProps): JSX$1.Element;
//#endregion
//#region src/components/prompt-composer.d.ts
export interface PromptComposerProps extends Omit<ViewProps, "class"> {
  class?: string;
  disabled?: boolean;
  invalid?: boolean;
  /** Background utility owned by the compound surface. */
  surfaceClass?: string;
}
export interface PromptComposerRowProps extends Omit<ViewProps, "class"> {
  class?: string;
  /** Allow controls to form additional rows when the embedding surface opts in. */
  wrap?: boolean;
}
export declare function promptComposerEditorHeightClass(value: string): string;
export interface PromptComposerEditorProps extends TextAreaProps {
  value?: string;
}
/** Native multiline editor with density and chrome owned by PromptComposer. */
export declare function PromptComposerEditor(props: PromptComposerEditorProps): JSX$1.Element;
/** Stable circular primary action for a PromptComposer toolbar. */
export declare function PromptComposerAction(props: ButtonProps): JSX$1.Element;
export declare function promptComposerClass(focused: boolean, invalid: boolean, disabled: boolean, className?: string): string;
/** Shared compound surface for prompts, attachments, controls and status. */
export declare function PromptComposer(props: PromptComposerProps): JSX$1.Element;
/** Compact metadata row above the authored prompt. */
export declare function PromptComposerStatus(props: ViewProps): JSX$1.Element;
/** Responsive action row below the authored prompt. */
export declare function PromptComposerToolbar(props: PromptComposerRowProps): JSX$1.Element;
/** Shrink-safe group for the composer tools preceding its primary action. */
export declare function PromptComposerTools(props: PromptComposerRowProps): JSX$1.Element;
//#endregion
//#region src/components/prompt-suggestion.d.ts
export interface PromptSuggestionsProps extends Omit<ResponsiveGridProps, "gap" | "initialColumns" | "maxColumns" | "minColumnWidth" | "balanceLastRow"> {
  itemCount?: number;
  minColumnWidth?: number;
  maxColumns?: 1 | 2 | 3;
  gap?: number;
}
/** Container-responsive starter actions for empty conversations and assistants. */
export declare function PromptSuggestions(props: PromptSuggestionsProps): JSX$1.Element;
export interface PromptSuggestionProps extends Omit<ButtonProps, "children" | "class"> {
  title: string;
  description?: string;
  icon?: string;
  class?: string;
}
/** One explicit prompt choice; the application remains responsible for its payload. */
export declare function PromptSuggestion(props: PromptSuggestionProps): JSX$1.Element;
//#endregion
//#region src/components/property-list.d.ts
export interface PropertyListProps extends Omit<ViewProps, "class"> {
  class?: string;
}
export declare function PropertyList(props: PropertyListProps): JSX$1.Element;
export interface PropertyRowProps {
  name: string;
  value: string;
  class?: string;
  nameClass?: string;
  valueClass?: string;
}
/** Compact two-column data intended for inspectors and settings summaries. */
export declare function PropertyRow(props: PropertyRowProps): JSX$1.Element;
//#endregion
//#region src/components/qr-code.d.ts
export type QrCodeErrorCorrection = "L" | "M" | "Q" | "H";
export interface QrCodeMatrix {
  readonly size: number;
  readonly data: readonly (readonly boolean[])[];
}
export interface QRCodeProps extends Omit<ViewProps, "children" | "style"> {
  value: QrCodeGenerateData;
  /** Logical-pixel width and height. The renderer keeps the code square. */
  size?: number;
  /** Error recovery level. Defaults to medium for application UI. */
  errorCorrection?: QrCodeErrorCorrection;
  /** Number of empty modules around the encoded matrix. */
  quietZone?: number;
  /** Packed RGBA (`0xRRGGBBAA`) used for dark modules. */
  foreground?: number;
  /** Background paint accepted by Wabou Style IR. */
  background?: WabouStyle$1["background-color"];
  style?: WabouStyle$1;
}
/** Encode with uqr while keeping its renderer-independent matrix contract. */
export declare function encodeQrCode(value: QrCodeGenerateData, errorCorrection?: QrCodeErrorCorrection): QrCodeMatrix;
/**
 * Convert consecutive dark modules into one retained native path. Horizontal
 * runs keep the bridge traffic and native scene node count independent of the
 * number of QR modules.
 */
export declare function qrCodePath(matrix: QrCodeMatrix, renderedSize: number, quiet: number, foreground?: number): import("@wabou/core").VectorPath;
/** A QR encoder from the JS ecosystem rendered as one native vector path. */
export declare function QRCode(props: QRCodeProps): JSX$1.Element;
//#endregion
//#region src/components/rating-state.d.ts
export declare function normalizeRatingMax(max: number | undefined): number;
export declare function clampRatingValue(value: number | undefined, max: number): number;
export declare function ratingLabel(value: number): string;
//#endregion
//#region src/components/rating.d.ts
export interface RatingProps {
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
export declare function Rating(props: RatingProps): JSX$1.Element;
//#endregion
//#region src/components/reasoning.d.ts
export interface ReasoningProps extends CollapsibleProps {}
/** Quiet disclosure surface for model reasoning or intermediate thought. */
export declare function Reasoning(props: ReasoningProps): JSX$1.Element;
export interface ReasoningTriggerProps extends Omit<CollapsibleTriggerProps, "children" | "class"> {
  label?: string;
  streaming?: boolean;
  class?: string;
}
export declare function ReasoningTrigger(props: ReasoningTriggerProps): JSX$1.Element;
export declare function ReasoningContent(props: CollapsibleContentProps): JSX$1.Element;
//#endregion
//#region src/components/resizable.d.ts
export type ResizableDirection = "horizontal" | "vertical";
export interface ResizablePanelDefinition {
  id: string;
  defaultSize: number;
  minSize?: number;
  maxSize?: number;
}
export type ResizablePanelSizes = Readonly<Record<string, number>>;
export declare function validateResizableSizes(panels: readonly ResizablePanelDefinition[], sizes?: ResizablePanelSizes): ResizablePanelSizes;
export interface ResizablePanelState {
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
export declare function createResizablePanelState(options: {
  panels: readonly ResizablePanelDefinition[];
  value?: () => ResizablePanelSizes | undefined;
  defaultValue?: ResizablePanelSizes;
  onValueChange?: (sizes: ResizablePanelSizes) => void;
}): ResizablePanelState;
export interface ResizablePanelGroupProps {
  panels: readonly ResizablePanelDefinition[];
  children?: JSX$1.Element;
  direction?: ResizableDirection;
  value?: ResizablePanelSizes;
  defaultValue?: ResizablePanelSizes;
  onValueChange?: (sizes: ResizablePanelSizes) => void;
  "aria-label": string;
  class?: string;
}
export declare function ResizablePanelGroup(props: ResizablePanelGroupProps): JSX$1.Element;
export declare function ResizablePanel(props: {
  id: string;
  children?: JSX$1.Element;
  class?: string;
}): JSX$1.Element;
export interface ResizableHandleProps {
  before: string;
  after: string;
  "aria-label": string;
  keyboardStep?: number;
  class?: string;
}
export declare function ResizableHandle(props: ResizableHandleProps): JSX$1.Element;
//#endregion
//#region src/components/search-field.d.ts
export interface SearchFieldProps extends Omit<InputProps, "class" | "onInput" | "ref" | "value"> {
  /** `undefined` selects uncontrolled mode. */
  value?: string;
  defaultValue?: string;
  onValueChange?(value: string): void;
  /** Called when Enter is pressed with the current query. */
  onSearch?(value: string): void;
  onClear?(): void;
  clearLabel?: string;
  /** Quiet fields stay transparent until focus; useful inside navigation chrome. */
  variant?: "default" | "quiet";
  class?: string;
  /** Background utility for the complete search field. Defaults to `bg-input`. */
  surfaceClass?: string;
  inputClass?: string;
  inputRef?: (input: Handle) => void;
}
/** A native search input with consistent clear, Escape, and submit behavior. */
export declare function SearchField(props: SearchFieldProps): JSX$1.Element;
//#endregion
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
export declare function Select(props: SelectProps): JSX$1.Element;
//#endregion
//#region src/components/selection.d.ts
type SelectionControlSize = "sm" | "default" | "lg";
interface CheckboxProps {
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
export declare function Checkbox(props: CheckboxProps): JSX$1.Element;
interface RadioGroupProps {
  value?: string;
  defaultValue?: string;
  disabled?: boolean;
  orientation?: "horizontal" | "vertical";
  appearance?: "radio" | "segment";
  size?: SelectionControlSize;
  loop?: boolean;
  "aria-label"?: string;
  class?: string;
  children?: JSX$1.Element;
  onValueChange?: (value: string) => void;
}
export declare function RadioGroup(props: RadioGroupProps): JSX$1.Element;
interface RadioGroupItemProps {
  value: string;
  label?: string;
  "aria-label"?: string;
  disabled?: boolean;
  size?: SelectionControlSize;
  class?: string;
}
export declare function RadioGroupItem(props: RadioGroupItemProps): JSX$1.Element;
interface ToggleProps {
  pressed?: boolean;
  defaultPressed?: boolean;
  disabled?: boolean;
  variant?: "default" | "outline";
  size?: "sm" | "default" | "lg";
  "aria-label"?: string;
  class?: string;
  children?: JSX$1.Element;
  onPressedChange?: (pressed: boolean) => void;
}
export declare function Toggle(props: ToggleProps): JSX$1.Element;
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
  children?: JSX$1.Element;
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
export declare function ToggleGroup(props: ToggleGroupProps): JSX$1.Element;
interface ToggleGroupItemProps {
  value: string;
  "aria-label"?: string;
  disabled?: boolean;
  variant?: "default" | "outline" | "accent";
  size?: "sm" | "default" | "lg";
  class?: string;
  children?: JSX$1.Element;
}
export declare function ToggleGroupItem(props: ToggleGroupItemProps): JSX$1.Element;
//#endregion
//#region src/components/separator.d.ts
export interface SeparatorProps extends Omit<ViewProps, "aria-hidden" | "aria-orientation" | "class" | "role"> {
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
export declare function Separator(props: SeparatorProps): JSX$1.Element;
export interface LabeledSeparatorProps extends Omit<ViewProps, "class"> {
  class?: string;
}
/**
 * A horizontal divider whose compact center content names the boundary.
 *
 * Keep the content short. Interactive content is supported so disclosures can
 * explain what happened between two regions without turning into a full row.
 */
export declare function LabeledSeparator(props: LabeledSeparatorProps): JSX$1.Element;
//#endregion
//#region src/components/settings.d.ts
export type SettingsItemOrientation = "horizontal" | "vertical";
export interface SettingsItemProps extends Omit<ViewProps, "children" | "class"> {
  title: string;
  description?: JSX$1.Element;
  children?: JSX$1.Element;
  orientation?: SettingsItemOrientation;
  disabled?: boolean;
  class?: string;
  labelClass?: string;
  controlClass?: string;
}
/**
 * One settings row with stable explanatory and control regions.
 * The row blocks its complete subtree when disabled; controls should also
 * receive `disabled` when they need to expose that state independently.
 */
export declare function SettingsItem(props: SettingsItemProps): JSX$1.Element;
export interface SettingsSectionProps {
  title: string;
  description?: string;
  children?: JSX$1.Element;
  /** Stack the explanation above the controls at a constrained viewport. */
  stacked?: boolean;
  class?: string;
  contentClass?: string;
}
/**
 * A settings-page section with one explanatory label and one control surface.
 * The page owns the responsive breakpoint and passes `stacked`; the component
 * owns the repeated alignment, spacing, and surface contract.
 */
export declare function SettingsSection(props: SettingsSectionProps): JSX$1.Element;
export interface SettingsGroupProps {
  title: string;
  description?: string;
  children?: JSX$1.Element;
  class?: string;
}
/** A titled field group inside a SettingsSection control surface. */
export declare function SettingsGroup(props: SettingsGroupProps): JSX$1.Element;
//#endregion
//#region src/components/shader-layer.d.ts
export interface ShaderLayerConfig {
  /** WGSL defining `wabou_effect`, or a complete module when `module` is true. */
  source: string;
  /** Accept a complete single-pass module exposing `vs_main` and `fs_main`. */
  module?: boolean;
  /** Up to 256 scalar values exposed through `wabou.values` or binding zero. */
  values?: readonly number[];
  /** Initial animation time in seconds. */
  time?: number;
  /** Native animation clock multiplier. */
  speed?: number;
  /** Whether the native frame clock advances time. */
  animated?: boolean;
  /** Temporarily freeze the native frame clock. */
  paused?: boolean;
}
export interface ShaderLayerProps extends Omit<NativeWidgetProps<ShaderLayerConfig>, "tag" | "config" | "children">, ShaderLayerConfig {
  class?: string;
}
/** A layout-aware custom WGSL surface rendered and animated by the Vello host. */
export declare function ShaderLayer(props: ShaderLayerProps): JSX$1.Element;
//#endregion
//#region src/components/sheet.d.ts
export type SheetSide = "top" | "right" | "bottom" | "left";
export interface SheetProps extends Omit<ModalProps, "contentClass"> {
  side?: SheetSide;
  contentClass?: string;
}
/** A modal edge panel that shares native focus isolation with Dialog. */
export declare function Sheet(props: SheetProps): JSX$1.Element;
//#endregion
//#region src/components/shortcut-recorder.d.ts
export interface RecordedShortcut {
  chord: string;
  parts: readonly string[];
}
export declare function shortcutFromKeyEvent(event: Pick<ButtonKeyEvent, "key" | "mods" | "primary">): RecordedShortcut | undefined;
export interface ShortcutRecorderProps {
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  label?: string;
  placeholder?: string;
  disabled?: boolean;
  class?: string;
}
export declare function ShortcutRecorder(props: ShortcutRecorderProps): JSX$1.Element;
//#endregion
//#region src/components/theme.d.ts
type ComponentsTheme = "light" | "dark";
type ComponentsElevation = "raised" | "floating" | "modal";
type ComponentsControlSize = "sm" | "default" | "lg" | "icon";
/**
 * Geometry contract for Wabou's default desktop theme.
 *
 * Components consume these recipes instead of independently choosing height,
 * padding, radius, and icon rhythm. The values intentionally favor desktop
 * density over touch-first sizing.
 */
export declare const componentsThemeContract: Readonly<{
  controlHeight: Readonly<{
    sm: 28;
    default: 32;
    lg: 40;
    icon: 32;
  }>;
  controlPaddingX: Readonly<{
    sm: 8;
    default: 10;
    lg: 12;
    icon: 0;
  }>;
  iconSize: Readonly<{
    sm: 14;
    default: 16;
    lg: 18;
  }>;
  typography: Readonly<{
    xs: Readonly<{
      size: 12;
      lineHeight: 16;
    }>;
    sm: Readonly<{
      size: 14;
      lineHeight: 20;
    }>;
    md: Readonly<{
      size: 16;
      lineHeight: 24;
    }>;
    lg: Readonly<{
      size: 18;
      lineHeight: 28;
    }>;
    xl: Readonly<{
      size: 20;
      lineHeight: 28;
    }>;
  }>;
  controlRadius: 6;
  containerRadius: 8;
  containerPadding: 20;
  sectionGap: 16;
  /** Disabled controls remain readable while losing active-surface emphasis. */
  disabledOpacity: 0.6;
}>;
export declare function componentsControlSize(size: ComponentsControlSize): string;
/**
 * Native elevation recipes adapted from gpui-component. Wabou and GPUI both
 * pass standard deviation directly to their renderer, so these values should
 * not use CSS's doubled blur radius. Floating surfaces also carry a subtle
 * foreground-colored ring: black in light mode, white in dark mode.
 */
export declare function componentsElevation(theme: ComponentsTheme, elevation: ComponentsElevation): Shadow[];
type ComponentsProviderProps = ParentProps<{
  theme?: ComponentsTheme;
}>;
export declare function ComponentsProvider(props: ComponentsProviderProps): JSX$1.Element;
export declare function useComponentsTheme(): () => ComponentsTheme;
//#endregion
//#region src/components/sidebar.d.ts
export interface SidebarSearchGroup<Item> {
  label: string;
  items: readonly Item[];
}
/**
 * Filter grouped sidebar data without taking ownership of routing or identity.
 * Group labels participate in matching so a query can reveal a whole section.
 */
export declare function filterSidebarGroups<Item>(groups: readonly SidebarSearchGroup<Item>[], query: string, searchableText: (item: Item) => string): SidebarSearchGroup<Item>[];
export interface SidebarProps extends Omit<ViewProps, "class"> {
  class?: string;
  /** Native shadow recipe for sidebars that float inside a window frame. */
  elevation?: ComponentsElevation;
}
/** Structural application sidebar. State, routing and width remain explicit. */
export declare function Sidebar(props: SidebarProps): JSX$1.Element;
export declare function SidebarHeader(props: ViewProps): JSX$1.Element;
export declare function SidebarSearch(props: SearchFieldProps): JSX$1.Element;
export interface SidebarContentProps extends ScrollAreaProps {
  contentClass?: string;
}
/** The only scrolling region in a standard sidebar; header/footer stay fixed. */
export declare function SidebarContent(props: SidebarContentProps): JSX$1.Element;
export declare function SidebarGroup(props: ViewProps): JSX$1.Element;
export declare function SidebarGroupLabel(props: TextProps): JSX$1.Element;
export interface SidebarMenuProps extends Omit<ViewProps, "class"> {
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  class?: string;
}
/**
 * Single-selection navigation scope for sidebar destinations.
 * Buttons without a value remain actions and never become selected items.
 */
export declare function SidebarMenu(props: SidebarMenuProps): JSX$1.Element;
export interface SidebarMenuButtonProps extends Omit<ButtonProps$1, "class" | "unstyled"> {
  /** Value controlled by the nearest SidebarMenu. Omit for action buttons. */
  value?: string;
  class?: string;
}
/** Consistent navigation row; applications still own activation and routing. */
export declare function SidebarMenuButton(props: SidebarMenuButtonProps): JSX$1.Element;
/** Fixed icon slot that keeps navigation labels on one shared baseline. */
export declare function SidebarMenuIcon(props: ViewProps): JSX$1.Element;
/** Truncating label slot for rows that also contain icons or suffix actions. */
export declare function SidebarMenuLabel(props: TextProps): JSX$1.Element;
/** End-aligned metadata or action slot that never compresses the row label. */
export declare function SidebarMenuSuffix(props: ViewProps): JSX$1.Element;
export declare function SidebarEmpty(props: {
  title?: string;
  description?: string;
  class?: string;
}): JSX$1.Element;
export declare function SidebarFooter(props: ViewProps): JSX$1.Element;
//#endregion
//#region src/components/slider.d.ts
export interface SliderProps {
  value?: number;
  defaultValue?: number;
  min?: number;
  max?: number;
  step?: number;
  disabled?: boolean;
  /** Direction of the track and pointer interaction. */
  orientation?: "horizontal" | "vertical";
  /** Fill from the thumb toward the maximum end without changing values. */
  reversed?: boolean;
  label: string;
  valueText?: (value: number) => string;
  onValueChange?: (value: number) => void;
  class?: string;
}
export declare function Slider(props: SliderProps): JSX$1.Element;
//#endregion
//#region src/components/split-button.d.ts
export interface SplitButtonProps extends Omit<ButtonProps, "children" | "aria-label"> {
  label: string;
  menuLabel?: string;
  items: readonly DropdownMenuItem[];
  onAction?: (id: string) => void;
  onClick?: ButtonProps["onClick"];
  class?: string;
}
/**
 * Desktop split action: the label always performs the primary command while
 * the adjacent arrow owns alternative commands and their keyboard behavior.
 */
export declare function SplitButton(props: SplitButtonProps): JSX$1.Element;
//#endregion
//#region src/components/stat-card.d.ts
export interface StatCardProps extends Omit<ViewProps, "children"> {
  label: string;
  value: string;
  description?: string;
  trend?: string;
  indicatorClass?: string;
}
export declare function StatCard(props: StatCardProps): JSX$1.Element;
//#endregion
//#region src/components/status-bar.d.ts
export interface StatusBarProps extends Omit<ViewProps, "class"> {
  class?: string;
}
/** Persistent low-emphasis application state at the bottom of a desktop window. */
export declare function StatusBar(props: StatusBarProps): JSX$1.Element;
export interface StatusBarItemProps extends Omit<TextProps, "class"> {
  class?: string;
  grow?: boolean;
}
export declare function StatusBarItem(props: StatusBarItemProps): JSX$1.Element;
export declare function StatusBarSeparator(props: {
  class?: string;
}): JSX$1.Element;
export interface StatusBarGroupProps extends Omit<ViewProps, "class"> {
  class?: string;
  grow?: boolean;
  shrink?: boolean;
}
/** A shrink-safe status bar group for icons, indicators and related text. */
export declare function StatusBarGroup(props: StatusBarGroupProps): JSX$1.Element;
export type StatusBarIndicatorTone = Exclude<ComponentTone, "neutral"> | "muted";
export interface StatusBarIndicatorProps {
  tone?: StatusBarIndicatorTone;
  class?: string;
}
/** Compact, decorative state indicator with a theme-aware semantic tone. */
export declare function StatusBarIndicator(props: StatusBarIndicatorProps): JSX$1.Element;
//#endregion
//#region src/components/stepper.d.ts
export interface StepperStep {
  id: string;
  label: string;
  description?: string;
}
export interface StepperProps extends Omit<ViewProps, "children"> {
  steps: readonly StepperStep[];
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
}
export declare function Stepper(props: StepperProps): JSX$1.Element;
//#endregion
//#region src/components/switch.d.ts
interface SwitchProps {
  checked?: boolean;
  defaultChecked?: boolean;
  disabled?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  label?: string;
  /** Place the label before or after the control in reading order. */
  labelPlacement?: "start" | "end";
  "aria-label"?: string;
  class?: string;
  size?: "sm" | "default";
}
export declare function Switch(props: SwitchProps): JSX$1.Element;
//#endregion
//#region src/components/table.d.ts
export interface TableProps extends Omit<ViewProps, "class"> {
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
export declare function Table(props: TableProps): JSX$1.Element;
export declare function TableHeader(props: ViewProps): JSX$1.Element;
export declare function TableBody(props: ViewProps): JSX$1.Element;
export declare function TableFooter(props: ViewProps): JSX$1.Element;
export interface TableRowProps extends Omit<ViewProps, "class"> {
  class?: string;
  selected?: boolean;
}
export declare function TableRow(props: TableRowProps): JSX$1.Element;
export interface TableHeadProps extends Omit<TextProps, "class"> {
  class?: string;
}
export declare function TableHead(props: TableHeadProps): JSX$1.Element;
export interface TableCellProps extends Omit<ViewProps, "class"> {
  class?: string;
}
export declare function TableCell(props: TableCellProps): JSX$1.Element;
export declare function TableCaption(props: TextProps): JSX$1.Element;
//#endregion
//#region src/components/tabs.d.ts
interface TabsProps {
  value?: string;
  defaultValue?: string;
  orientation?: "horizontal" | "vertical";
  class?: string;
  children?: JSX$1.Element;
  onValueChange?: (value: string) => void;
}
export declare function Tabs(props: TabsProps): JSX$1.Element;
export declare function TabsList(props: {
  variant?: "default" | "line";
  /** Keep tab semantics and roving focus while leaving layout and paint to the caller. */
  unstyled?: boolean;
  "aria-label"?: string;
  class?: string;
  children?: JSX$1.Element;
}): JSX$1.Element;
interface TabsTriggerProps {
  value: string;
  disabled?: boolean;
  /** Keep tab behavior and semantics without applying the component skin. */
  unstyled?: boolean;
  "aria-label"?: string;
  class?: string | ((state: ButtonState) => string);
  children?: JSX$1.Element;
}
interface TabsItemState {
  selected: boolean;
}
interface TabsItemProps {
  value: string;
  disabled?: boolean;
  closeLabel?: string;
  onClose?: () => void;
  class?: string | ((state: TabsItemState) => string);
  triggerClass?: string | ((state: ButtonState) => string);
  children?: JSX$1.Element;
}
/**
 * Bounded, optionally closeable tab chrome. The tab trigger and close action
 * remain sibling hit targets so closing a tab never selects it first.
 */
export declare function TabsItem(props: TabsItemProps): JSX$1.Element;
export declare function TabsTrigger(props: TabsTriggerProps): JSX$1.Element;
export declare function TabsContent(props: {
  value: string;
  /** Keep stateful/native content mounted while hiding inactive panels. */
  keepMounted?: boolean;
  class?: string;
  children?: JSX$1.Element;
}): JSX$1.Element;
//#endregion
//#region src/components/timeline.d.ts
export interface TimelineItem {
  id: string;
  title: string;
  description?: string;
  time?: string;
  status?: "complete" | "current" | "pending";
}
export interface TimelineProps extends Omit<ViewProps, "children"> {
  items: readonly TimelineItem[];
}
export declare function Timeline(props: TimelineProps): JSX$1.Element;
//#endregion
//#region src/components/title-bar.d.ts
export interface WindowFrameProps extends ViewProps {
  class?: string;
  children?: JSX$1.Element;
  /** Clip the application surface to desktop-style corners when restored. */
  rounded?: boolean;
  /** Let the native window material remain visible beneath a translucent client surface. */
  material?: "solid" | "translucent";
}
export declare function windowFrameMaterialStyle(theme: "light" | "dark", material?: "solid" | "translucent"): WabouStyle;
export declare function windowFrameBackdropClassList(maximized: boolean, rounded?: boolean): WabouClassList;
export declare function windowFrameClientClassList(maximized: boolean, rounded?: boolean, classList?: WabouClassList): WabouClassList;
/** Two restrained client-decoration layers sized to fit the 12px backdrop. */
export declare function windowFrameShadows(theme: "light" | "dark"): Shadow[];
/**
 * Root frame for an application-owned title bar and window chrome.
 *
 * Rounded outer corners require the native window to preserve alpha and the
 * Rust host to clear with a transparent base color. Maximized windows are
 * intentionally square so their content reaches every display edge.
 */
export declare function WindowFrame(props: WindowFrameProps): JSX$1.Element;
export declare const titleBarClass = "border-b border-subtle";
export interface TitleBarProps extends ViewProps {
  class?: string;
  children?: JSX$1.Element;
}
export declare const titleBarLayoutStyle: {
  readonly display: "flex";
  readonly "flex-direction": "row";
  readonly "align-items": "center";
  readonly "flex-shrink": 0;
  readonly height: "40px";
};
export declare const titleBarDragRegionLayoutStyle: {
  readonly display: "flex";
  readonly "flex-direction": "row";
  readonly "align-items": "center";
  readonly "flex-grow": 1;
  readonly "flex-shrink": 1;
  readonly "flex-basis": "0%";
  readonly height: "100%";
};
/** Layout shell for an application-owned title bar. */
export declare function TitleBar(props: TitleBarProps): JSX$1.Element;
export interface TitleBarDragRegionProps extends Omit<ViewProps, "onPointerDown" | "onDblClick"> {
  class?: string;
  style?: WabouStyle;
  children?: JSX$1.Element;
}
/** Explicit non-interactive region that moves the native window. */
export declare function TitleBarDragRegion(props: TitleBarDragRegionProps): JSX$1.Element;
//#endregion
//#region src/components/toast.d.ts
export type ToastVariant = "default" | "success" | "warning" | "destructive";
export interface ToastAction {
  label: string;
  onAction(): void;
  /** Defaults to true. */
  dismiss?: boolean;
}
export interface ToastInput {
  title: string;
  description?: string;
  variant?: ToastVariant;
  action?: ToastAction;
  duration?: number;
  onDismiss?: (reason: NotificationDismissReason) => void;
}
export interface Toasts {
  /** The unstyled queue remains available for advanced composition. */
  readonly notifications: Notifications;
  show(input: ToastInput): number;
  success(title: string, input?: Omit<ToastInput, "title" | "variant">): number;
  warning(title: string, input?: Omit<ToastInput, "title" | "variant">): number;
  error(title: string, input?: Omit<ToastInput, "title" | "variant">): number;
  dismiss(id: number): boolean;
  clear(): void;
}
export interface CreateToastsOptions {
  defaultDuration?: number;
  limit?: number;
}
/** Create an owner-scoped, styled toast queue over NotificationRegion. */
export declare function createToasts(options?: CreateToastsOptions): Toasts;
export interface ToasterProps {
  toasts: Toasts;
  placement?: NotificationPlacement;
  class?: string;
  itemClass?: string;
  motion?: false | NotificationMotionOptions;
}
/** Render a non-blocking stack of styled toasts on the floating plane. */
export declare function Toaster(props: ToasterProps): JSX$1.Element;
//#endregion
//#region src/components/tool.d.ts
export type ToolStatus = "pending" | "running" | "success" | "failed";
export interface ToolProps extends CollapsibleProps {}
/** Composable disclosure root for an AI or automation tool invocation. */
export declare function Tool(props: ToolProps): JSX$1.Element;
export interface ToolHeaderProps extends Omit<CollapsibleTriggerProps, "children" | "class"> {
  title: string;
  summary?: string;
  status?: ToolStatus;
  icon?: string;
  class?: string;
}
export declare function toolHeaderLabel(title: string, summary?: string, status?: ToolStatus): string;
/** Stable title, summary, status and disclosure geometry for one tool call. */
export declare function ToolHeader(props: ToolHeaderProps): JSX$1.Element;
export declare function ToolContent(props: CollapsibleContentProps): JSX$1.Element;
export interface ToolCodeSectionProps extends Omit<ViewProps, "children" | "class"> {
  code: string;
  label: string;
  language?: string;
  copyable?: boolean;
  class?: string;
  codeClass?: string;
  codeProps?: Omit<CodeBlockProps, "code" | "language" | "copyable">;
}
/** Labelled code payload used for tool parameters, results and errors. */
export declare function ToolCodeSection(props: ToolCodeSectionProps): JSX$1.Element;
export interface ToolInputProps extends Omit<ToolCodeSectionProps, "label"> {
  label?: string;
}
export declare function ToolInput(props: ToolInputProps): JSX$1.Element;
export interface ToolOutputProps extends Omit<ToolCodeSectionProps, "label"> {
  label?: string;
  error?: boolean;
}
export declare function ToolOutput(props: ToolOutputProps): JSX$1.Element;
//#endregion
//#region src/components/toolbar.d.ts
export type ToolbarOrientation = "horizontal" | "vertical";
export interface ToolbarProps {
  "aria-label": string;
  /** Semantic role used by composite controls built on the toolbar primitive. */
  role?: "toolbar" | "menubar";
  orientation?: ToolbarOrientation;
  loop?: boolean;
  class?: string;
  children?: JSX$1.Element;
}
/** A compact command surface with one native tab stop and arrow navigation. */
export declare function Toolbar(props: ToolbarProps): JSX$1.Element;
export interface ToolbarButtonProps extends Omit<ButtonProps, "focusOrder"> {}
export declare function ToolbarButton(props: ToolbarButtonProps): JSX$1.Element;
export interface ToolbarToggleProps extends Omit<ToolbarButtonProps, "aria-pressed" | "onClick" | "variant"> {
  pressed?: boolean;
  defaultPressed?: boolean;
  onPressedChange?(pressed: boolean): void;
}
export declare function ToolbarToggle(props: ToolbarToggleProps): JSX$1.Element;
export declare function ToolbarGroup(props: {
  "aria-label"?: string;
  class?: string;
  children?: JSX$1.Element;
}): JSX$1.Element;
export declare function ToolbarSeparator(props: {
  class?: string;
}): JSX$1.Element;
//#endregion
//#region src/components/tooltip.d.ts
export interface TooltipTriggerProps {
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
export interface TooltipProps extends PopupMotionProps {
  trigger(props: TooltipTriggerProps): JSX$1.Element;
  children?: JSX$1.Element;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?(open: boolean): void;
  openDelay?: number;
  closeDelay?: number;
  placement?: Placement;
  offset?: number;
  contentClass?: string;
  disabled?: boolean;
  /** Optional keyboard shortcut presented as a native keycap. */
  shortcut?: string;
}
export interface TooltipContentProps {
  children?: JSX$1.Element;
  shortcut?: string;
  id?: string;
  class?: string;
}
/** Visual content shared by managed and explicitly composed tooltips. */
export declare function TooltipContent(props: TooltipContentProps): JSX$1.Element;
/** A delayed, non-interactive label for pointer and keyboard focus targets. */
export declare function Tooltip(props: TooltipProps): JSX$1.Element;
//#endregion
//#region src/components/tree-view.d.ts
export interface TreeNode {
  id: string;
  label: string;
  disabled?: boolean;
  /** The node is expandable even when its children have not been loaded yet. */
  hasChildren?: boolean;
  children?: readonly TreeNode[];
}
export interface VisibleTreeNode {
  node: TreeNode;
  parentId: string | null;
  level: number;
  position: number;
  setSize: number;
}
export interface TreeModel {
  get(id: string): TreeNode | undefined;
  parent(id: string): string | null | undefined;
  firstChild(id: string): string | undefined;
  isBranch(id: string): boolean;
  visible(expandedIds: readonly string[]): readonly VisibleTreeNode[];
}
/** Validates a nested tree once and provides deterministic visible traversal. */
export declare function createTreeModel(nodes: readonly TreeNode[]): TreeModel;
export interface TreeItemRenderState {
  expanded: boolean;
  selected: boolean;
  level: number;
}
export interface TreeViewProps {
  items: readonly TreeNode[];
  "aria-label": string;
  expandedIds?: readonly string[];
  defaultExpandedIds?: readonly string[];
  onExpandedChange?(ids: readonly string[]): void;
  /** `undefined` selects uncontrolled mode; `null` is a controlled empty selection. */
  selectedId?: string | null;
  defaultSelectedId?: string | null;
  onSelectedChange?(id: string | null): void;
  renderItem?(node: TreeNode, state: TreeItemRenderState): JSX$1.Element;
  /** Window a large fixed-height tree while retaining native tree semantics. */
  virtual?: {
    itemHeight: number;
    overscan?: number;
    viewportHeight?: number;
  };
  class?: string;
  itemClass?: string;
}
/** A single-select tree with explicit data, expansion, and native focus routing. */
export declare function TreeView(props: TreeViewProps): JSX$1.Element;
//#endregion
//#region src/components/typography.d.ts
type TypographyTextProps = Omit<TextProps, "class"> & {
  class?: string;
};
export declare const TypographyH1: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare const TypographyH2: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare const TypographyH3: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare const TypographyH4: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare const TypographyP: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare const TypographyLead: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare const TypographyLarge: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare const TypographySmall: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare const TypographyMuted: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare const TypographyInlineCode: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX$1.Element;
export declare function TypographyBlockquote(props: TypographyTextProps): JSX$1.Element;
export declare function TypographyList(props: ViewProps): JSX$1.Element;
export declare function TypographyListItem(props: TypographyTextProps): JSX$1.Element;
//#endregion
//#region src/components/workbench-style.d.ts
export declare const workbenchClass: (className?: string) => string;
export declare const workbenchSidebarClass: (className?: string) => string;
export declare const workbenchMainClass: (className?: string) => string;
export declare const workbenchHeaderClass: (className?: string) => string;
export declare const workbenchContentClass: (className?: string) => string;
/** Shared readable column for workbench transcripts, editors and composers. */
export declare const workbenchContentColumnClass: (className?: string) => string;
export declare const workbenchFooterClass: (className?: string) => string;
/** Bounded auxiliary pane beside primary workbench content. */
export declare const workbenchInspectorClass: (className?: string) => string;
/** Fixed inspector chrome aligned with the inspector body. */
export declare const workbenchInspectorHeaderClass: (className?: string) => string;
/** Shrink-safe body for inspector content and nested scroll areas. */
export declare const workbenchInspectorContentClass: (className?: string) => string;
//#endregion
//#region src/components/workbench.d.ts
/** Full-window desktop application boundary with explicit shrink semantics. */
export declare function Workbench(props: ViewProps): JSX$1.Element;
/** Fixed-width navigation rail paired with a {@link WorkbenchMain}. */
export declare function WorkbenchSidebar(props: SidebarProps): JSX$1.Element;
/** The resizable application column beside the navigation rail. */
export declare function WorkbenchMain(props: ViewProps): JSX$1.Element;
/** Shared 48px chrome row for both sidebar and content headers. */
export declare function WorkbenchHeader(props: ViewProps): JSX$1.Element;
/** Bounded application content. Add a ScrollArea inside when scrolling is needed. */
export declare function WorkbenchContent(props: ViewProps): JSX$1.Element;
/** A centered 896px desktop content column that still shrinks with its pane. */
export declare function WorkbenchContentColumn(props: ViewProps): JSX$1.Element;
/** Fixed chrome below the workbench content, such as a composer or status bar. */
export declare function WorkbenchFooter(props: ViewProps): JSX$1.Element;
/** Fixed-width auxiliary pane for file previews, diffs and contextual tools. */
export declare function WorkbenchInspector(props: ViewProps): JSX$1.Element;
/** Inspector title row with a stable height and bounded children. */
export declare function WorkbenchInspectorHeader(props: ViewProps): JSX$1.Element;
export interface WorkbenchInspectorTitlebarProps extends Omit<ViewProps, "children"> {
  title: string;
  description?: string;
  closeLabel?: string;
  onClose?: () => void;
  /** Lazily render an optional action before the standard close button. */
  renderAction?: () => JSX$1.Element;
}
/**
 * Consistent inspector chrome with a bounded title, optional description and
 * correctly sized trailing actions.
 */
export declare function WorkbenchInspectorTitlebar(props: WorkbenchInspectorTitlebarProps): JSX$1.Element;
/** Flexible, clipped inspector body. Add a ScrollArea inside when needed. */
export declare function WorkbenchInspectorContent(props: ViewProps): JSX$1.Element;
export interface WorkbenchInspectorStateProps extends ContentStateProps {}
/** Mutually exclusive centered state for a bounded inspector body. */
export declare function WorkbenchInspectorState(props: WorkbenchInspectorStateProps): JSX$1.Element;
//#endregion
//#region src/components/index.d.ts
interface FpsBaseProps {
  /** Text displayed after the value. Set to an empty string for value only. */
  label?: string;
  /** FPS at or above this value uses the success treatment. */
  goodAt?: number;
  /** FPS below this value uses the destructive treatment. */
  warningBelow?: number;
  class?: string;
}
export type FpsProps = FpsBaseProps & ({
  /** Explicitly drive the native animation clock to measure live FPS. */
  live: true;
  value?: never;
} | {
  /** Render an externally measured FPS value without scheduling frames. */
  value: number;
  live?: false;
});
/** Frame-rate indicator. Live measurement is intentionally opt-in because it
 * keeps the platform frame clock active. */
export declare function Fps(props: FpsProps): JSX$1.Element;
//#endregion
//#region src/router/data.d.ts
export type WabouDataRouter<TRouteTree extends AnyRoute = AnyRoute> = RouterCore<TRouteTree, TrailingSlashOption, boolean, RouterHistory>;
export interface RouterPersistenceOptions {
  /** Application-scoped KV handle. The host must opt in with `HostBuilder::kv()`. */
  kv: Kv;
  /** Explicit hierarchical identity for this router. */
  key: KvKey;
  /** Ignore locations written by another schema version. Defaults to 1. */
  version?: number;
}
export type WabouRouterConstructorOptions<TRouteTree extends AnyRoute, TTrailingSlash extends TrailingSlashOption, TStructuralSharing extends boolean, THistory extends RouterHistory, TDehydrated extends Record<string, any>> = RouterConstructorOptions<TRouteTree, TTrailingSlash, TStructuralSharing, THistory, TDehydrated> & {
  /** Optionally restore and save the last committed native location. */
  persistence?: RouterPersistenceOptions;
};
/** Create a TanStack Router Core instance adapted to Solid 2 and native history. */
export declare function createDataRouter<TRouteTree extends AnyRoute, TTrailingSlash extends TrailingSlashOption = "never", TStructuralSharing extends boolean = false, THistory extends RouterHistory = RouterHistory, TDehydrated extends Record<string, any> = Record<string, any>>(options: WabouRouterConstructorOptions<TRouteTree, TTrailingSlash, TStructuralSharing, THistory, TDehydrated>): RouterCore<TRouteTree, TTrailingSlash, TStructuralSharing, THistory, TDehydrated>;
export interface RouterProviderProps {
  router: AnyRouter;
  fallback?: JSX$1.Element;
}
/** Own router lifecycle and render its current native component branch. */
export declare function RouterProvider(props: RouterProviderProps): JSX$1.Element;
export declare function useRouter(): AnyRouter;
export declare function useRouterState<T>(selector: (router: AnyRouter) => T): Accessor<T>;
export declare function useNavigate(): AnyRouter["navigate"];
export declare function useLocation(): Accessor<AnyRouter["state"]["location"]>;
export interface RouteActiveOptions {
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
export declare function useRouteActive(to: string, options?: RouteActiveOptions): Accessor<boolean>;
export declare function useParams<T extends Record<string, string> = Record<string, string>>(): Accessor<T>;
export declare function useLoaderData<T = unknown>(): Accessor<T | undefined>;
//#endregion
export { type ActiveResult, AnimationControls, type AnimationFrameCallback, AnimationOptions, AnimationState, AnimationType, AnimationValue, BaseRootRoute, BaseRoute, CalendarDate, Center, type CheckboxProps, CollapsiblePresence, Column, type ComponentsControlSize, type ComponentsElevation, type ComponentsProviderProps, type ComponentsTheme, type ContainerMatch, type ContainerSizeQuery, type DelayedOpenController, type DelayedOpenOptions, type DevServerDiagnostic, type ModalControls as DialogControls, DialogDescription as SheetDescription, DialogFooter as SheetFooter, DialogHeader as SheetHeader, DialogScrollBody as SheetScrollBody, DialogTitle as SheetTitle, Easing, EasingFunction, Editor, type EditorProps, FORM_ERROR, type FocusResult, type FocusWithinResult, type FormDraft, type FormDraftErrors, type FormDraftFieldUpdater, type FormDraftOptions, Icon, type IconProps, Image, type ImageProps, type ImageResourceDescriptor, type ImageResourceErrorEvent, type ImageResourceHandle, type ImageResourceReadyEvent, type ImageResourceRequest, type KeyedSelection, type KeyedSelectionOptions, KeyframeAnimationOptions, type LayoutProps, LoopOptions, Modal, type ModalMotionOptions, type ModalProps, type MotionConfig, MotionConfigProvider, type MotionConfigProviderProps, MotionInterpolationOptions, NativeLoopAnimation, NativeLoopAnimationOptions, NativeWidget, type NativeWidgetConfig, type NativeWidgetProps, type NotificationMotionOptions, NotificationRegion, type NotificationRegionProps, type Notifications, type OverlayLayer, OverlayPlaneProvider, type OwnedImageResource, type PaginationRangeItem, Path, PathBuilder, type PathProps, type PressResult, ProjectionBoundary, Pulse, PulseOptions, type RadioGroupItemProps, type RadioGroupProps, ReactiveAnimation, ReactiveTransition, RepeatType, type RetainedItem, type RetainedItems, RichText, type RichTextProps, RichTextSpan, type RichTextSpanProps, Ripple, type RippleProps, RotationAnimation, RotationOptions, Row, ScrollArea, type ScrollAreaProps, type SkeletonProps, Spin, Svg, type SvgProps, SweepAnimation, SweepAxis, SweepGeometry, SweepOptions, type SwitchProps, type TabsItemProps, type TabsItemState, type TabsProps, type TabsResult, type TabsTriggerProps, Text, type TextProps, type ToggleGroupItemProps, type ToggleGroupProps, type ToggleProps, TransitionOptions, type TransitionPresence, type TransitionPresenceOptions, type VectorPath, type VectorPathPaint, View, type ViewProps, type WabouClassList, type WabouStyle, animate, animateKeyframes, createActive, createAnimationFrame, createButton, createContainerMatch, createDelayedOpenController as createTooltipDelayController, createFileImageResource, createFocus, createFocusWithin, createFormDraft, createHover, createInterpolation, createKeyedSelection, createKeyframeAnimation, createLoop, createMeasuredSize, createMemoryHistory, createNativeLoopAnimation, createNetworkImageResource, createNotifications, createOverlayLayer, createOwnedImageResource, createPresence, createPress, createPulse, createRetainedItems, createRotation, createScrollReset, createShortcuts, createSweep, createTabs, createTransition, createTransitionPresence, normalizeSweepGeometry, notFound, index_d_exports as primitives, redirect, releaseImageResource, useMotionConfig, useReducedMotion };
//# sourceMappingURL=index.d.mts.map