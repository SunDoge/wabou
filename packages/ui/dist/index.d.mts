import { $n as Text, $r as useReducedMotion, $t as FormDraftFieldUpdater, A as createPress, An as Icon, Ar as RepeatType, At as ModalControls, Br as createInterpolation, C as RetainedItem, Cr as LoopOptions, Ct as Pulse, Dr as PulseOptions, Dt as Spin, E as ActiveResult, En as createButton, Er as NativeLoopAnimationOptions, Et as RippleProps, F as PopoverMotionOptions, Fn as ImageResourceHandle, Fr as SweepGeometry, Ft as ModalProps, Gr as createRotation, Gt as LayoutProps, Hn as Path, Hr as createLoop, Ht as createMeasuredSize, I as PopoverProps$1, In as ImageResourceReadyEvent, Ir as SweepOptions, Jn as RichTextProps, Jr as normalizeSweepGeometry, Jt as KeyedSelectionOptions, Kn as ProjectionBoundary, Kr as createSweep, Kt as Row, Ln as NativeWidget, Lr as TransitionOptions, Lt as ContainerMatch, Mn as Image, Mr as RotationOptions, N as createPresence, Nn as ImageProps, Nr as SweepAnimation, Nt as ModalMotionOptions, O as PressResult, On as Editor, Or as ReactiveAnimation, Pn as ImageResourceErrorEvent, Pr as SweepAxis, Pt as ModalOpenChangeReason, Qn as SvgProps, Qr as useMotionConfig, Qt as FormDraftErrors, Rn as NativeWidgetConfig, Rr as animate, Rt as ContainerSizeQuery, S as ScrollAreaProps, Sn as ButtonState, Sr as KeyframeAnimationOptions, St as createNotifications, T as createRetainedItems, Tr as NativeLoopAnimation, Tt as Ripple, U as Placement, Un as PathBuilder, Ur as createNativeLoopAnimation, Ut as Center, Vn as PasswordInputProps$1, Vr as createKeyframeAnimation, Vt as createContainerMatch, W as PointAnchor, Wn as PathProps, Wr as createPulse, Wt as Column, Xn as RichTextSpanProps, Xr as MotionConfigProvider, Xt as FORM_ERROR, Yn as RichTextSpan, Yr as MotionConfig, Yt as createKeyedSelection, Zn as Svg, Zr as MotionConfigProviderProps, Zt as FormDraft, _ as createShortcuts, _r as AnimationState, an as createFileImageResource, ar as VectorPath, b as createScrollReset, br as Easing, bt as Notifications, cn as releaseImageResource, cr as ViewProps, d as createTabs, dn as FocusResult, en as FormDraftOptions, fn as FocusWithinResult, ft as NotificationDismissReason, gr as AnimationOptions, gt as NotificationPlacement, hn as CollapsiblePresence, hr as AnimationControls, ht as NotificationMotionOptions, i as createTransitionPresence, in as OwnedImageResource, ir as TextProps, it as OverlayLayer, jn as IconProps, jr as RotationAnimation, k as createActive, kn as EditorProps, kr as ReactiveTransition, kt as Modal, lr as WabouClassList, lt as createOverlayLayer, mn as createFocusWithin, mr as createAnimationFrame, n as TransitionPresence, nn as ImageResourceDescriptor, on as createNetworkImageResource, or as VectorPathPaint, pn as createFocus, pr as AnimationFrameCallback, qn as RichText, qr as createTransition, qt as KeyedSelection, r as TransitionPresenceOptions, rn as ImageResourceRequest, rr as TextInputProps, sn as createOwnedImageResource, sr as View, st as OverlayPlaneProvider, t as index_d_exports, tn as createFormDraft, tr as TextAreaProps$1, u as TabsResult, un as createHover, ur as WabouStyle, vr as AnimationType, vt as NotificationRegion, w as RetainedItems, wr as MotionInterpolationOptions, x as ScrollArea, xn as ButtonProps$1, xr as EasingFunction, yn as ButtonKeyEvent, yr as AnimationValue, yt as NotificationRegionProps, zn as NativeWidgetProps, zr as animateKeyframes } from "./index-gI4KJtKK.mjs";
import { FileDropPosition, Kv, KvKey, PickDirectoryOptions } from "@wabou/core";
import { Shadow, WabouStyle as WabouStyle$1 } from "@wabou/core/style";
import { Accessor, JSX, ParentProps, Setter } from "solid-js";
import { Handle, LayoutRect, WabouPointerEvent } from "@wabou/core/renderer";
import { CalendarDate } from "@internationalized/date";
import { QrCodeGenerateData } from "uqr";
import { ColumnDef, Row as Row$1, RowSelectionState, SortingState, Table as Table$1 } from "@tanstack/table-core";
import { RouterHistory, createMemoryHistory } from "@tanstack/history";
import { AnyRoute, AnyRouter, BaseRootRoute, BaseRoute, RouterConstructorOptions, RouterCore, TrailingSlashOption, notFound, redirect } from "@tanstack/router-core";
export * from "@wabou/core";
export * from "@wabou/core/i18n";
//#region src/components/activity-status.d.ts
type ActivityStatusTone = "accent" | "danger" | "muted" | "success";
interface ActivityStatusIndicatorProps {
  animated?: boolean;
  tone?: ActivityStatusTone;
  class?: string;
}
/** A consistently sized status dot. Animation is explicit so idle state never
 * keeps the native frame clock alive accidentally. */
declare function ActivityStatusIndicator(props: ActivityStatusIndicatorProps): JSX.Element;
interface ActivityStatusProps extends Omit<ViewProps, "children" | "class"> {
  label: string;
  animated?: boolean;
  tone?: ActivityStatusTone;
  class?: string;
  textClass?: string;
}
/** Shrink-safe inline progress or presence status for agent and desktop UI. */
declare function ActivityStatus(props: ActivityStatusProps): JSX.Element;
//#endregion
//#region src/components/alert.d.ts
type AlertVariant = "default" | "info" | "success" | "warning" | "error" | "destructive";
type AlertSize = "sm" | "default" | "lg";
declare function alertColors(variant: AlertVariant): {
  container: string;
  title: string;
  description: string;
};
interface AlertProps extends Omit<ViewProps, "role"> {
  variant?: AlertVariant;
  size?: AlertSize;
  /** Optional leading graphic. Pass false to suppress the semantic default. */
  icon?: JSX.Element | false;
  /** Edge-to-edge status strip without container chrome. */
  banner?: boolean;
  /** Show a close control and delegate visibility to the owner. */
  onClose?: () => void;
  /** Convenience form; compound usage can render AlertTitle directly. */
  title?: string;
}
/** A native status callout with shadcn-compatible compound composition. */
declare function Alert(props: AlertProps): JSX.Element;
declare function AlertTitle(props: TextProps): JSX.Element;
declare function AlertDescription(props: TextProps): JSX.Element;
/** Recovery and acknowledgement controls belonging to an alert. */
declare function AlertActions(props: ViewProps): JSX.Element;
//#endregion
//#region src/components/button.d.ts
type ButtonVariant = "default" | "secondary" | "outline" | "ghost" | "destructive";
type ButtonSize = "sm" | "default" | "lg" | "icon";
interface ButtonProps extends Omit<ButtonProps$1, "variant" | "tone"> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  class?: string;
  style?: ButtonProps$1["style"];
  /** Disable activation and replace the leading content with a native spinner. */
  loading?: boolean;
  /** Visible label used while loading. Defaults to the ordinary children. */
  loadingLabel?: string;
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
}): import("@wabou/core/jsx-runtime").JSX.Element;
declare function DialogFooter(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
/**
 * The shrinking, independently scrollable region between a dialog's fixed
 * header and footer. The dialog surface must have a bounded or maximum height.
 */
interface DialogScrollBodyProps extends Omit<ScrollAreaProps, "class" | "contentClass"> {
  class?: string;
  contentClass?: string;
}
declare function DialogScrollBody(props: DialogScrollBodyProps): import("@wabou/core/jsx-runtime").JSX.Element;
declare function DialogTitle(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
declare function DialogDescription(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
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
declare function attachmentGroupClass(className?: string): string;
declare function AttachmentGroup(props: ViewProps): JSX.Element;
//#endregion
//#region src/components/avatar.d.ts
type AvatarSize = "sm" | "default" | "lg";
interface AvatarProps extends Omit<ViewProps, "children"> {
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
declare function avatarInitials(name: string): string;
declare function Avatar(props: AvatarProps): JSX.Element;
type AvatarGroupProps = ViewProps;
declare function AvatarGroup(props: AvatarGroupProps): JSX.Element;
type AvatarGroupCountProps = ViewProps;
declare function AvatarGroupCount(props: AvatarGroupCountProps): JSX.Element;
//#endregion
//#region src/components/badge.d.ts
type BadgeVariant = "default" | "secondary" | "outline" | "ghost" | "link" | "success" | "destructive";
type BadgeSize = "sm" | "default" | "lg";
interface BadgeProps extends Omit<TextProps, "class"> {
  variant?: BadgeVariant;
  size?: BadgeSize;
  /** Typography weight selected without competing utility declarations. */
  weight?: "normal" | "medium";
  class?: string;
}
declare function badgeClass(variant?: BadgeVariant, weight?: NonNullable<BadgeProps["weight"]>, className?: string, size?: BadgeSize): string;
/** Compact status text with shadcn-compatible visual variants. */
declare function Badge(props: BadgeProps): JSX.Element;
//#endregion
//#region src/components/button-group-context.d.ts
type ButtonGroupOrientation = "horizontal" | "vertical";
type ButtonGroupButtonSize = "sm" | "default" | "lg" | "icon";
type ButtonGroupButtonVariant = "default" | "secondary" | "outline" | "ghost" | "destructive";
//#endregion
//#region src/components/button-group.d.ts
interface ButtonGroupProps extends Omit<ViewProps, "class"> {
  orientation?: ButtonGroupOrientation;
  size?: ButtonGroupButtonSize;
  variant?: ButtonGroupButtonVariant;
  disabled?: boolean;
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
type CardVariant = "raised" | "filled" | "outline" | "plain";
type CardSize = "sm" | "default" | "lg";
interface CardProps extends Omit<ViewProps, "class"> {
  variant?: CardVariant;
  size?: CardSize;
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
//#region src/components/chart.d.ts
interface ChartSeriesConfig {
  label: string;
  colorClass: string;
}
type ChartConfig = Readonly<Record<string, ChartSeriesConfig>>;
interface ChartContainerProps {
  config: ChartConfig;
  label: string;
  class?: string;
  style?: ViewProps["style"];
  children?: JSX.Element;
}
declare function ChartContainer(props: ChartContainerProps): JSX.Element;
declare function useChartConfig(): ChartConfig;
declare function ChartLegend(props: {
  class?: string;
}): JSX.Element;
declare function ChartEmpty(props: {
  message?: string;
  class?: string;
}): JSX.Element;
//#endregion
//#region src/components/code-block.d.ts
interface CodeBlockProps extends Omit<ViewProps, "children"> {
  code: string;
  language?: string;
  copyable?: boolean;
  copyLabel?: string;
  copiedLabel?: string;
}
declare function CodeBlock(props: CodeBlockProps): JSX.Element;
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
  /** Human-readable platform shortcut, such as `Ctrl K` or `⌘ K`. */
  shortcut?: string;
  onSelect?: () => void;
}
interface CommandProps {
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
interface CommandListProps {
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
  renderLeading?: (item: CommandItem) => JSX.Element;
}
/** Reusable command-result surface for search fields and inline completions. */
declare function CommandList(props: CommandListProps): JSX.Element;
interface CommandListNavigationOptions {
  onAction?: (id: string) => void;
  onDismiss?: () => void;
}
/**
 * Shared command-list navigation for any focus owner, including native editors.
 * The caller forwards key events while the list remains a passive popup.
 */
declare function createCommandListNavigation(items: Accessor<readonly CommandStateItem[]>, options?: CommandListNavigationOptions): {
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
declare function Command(props: CommandProps): JSX.Element;
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
//#region src/components/select-semantics.d.ts
type PickerTriggerVariant = "default" | "ghost";
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
  triggerVariant?: PickerTriggerVariant;
  contentClass?: string;
  contentShadows?: readonly Shadow[] | null;
  onValueChange?: (value: string) => void;
  onOpenChange?: (open: boolean) => void;
}
/** A searchable single-value picker built from Popover and Command. */
declare function Combobox(props: ComboboxProps): JSX.Element;
//#endregion
//#region src/components/content-state.d.ts
type ContentStateKind = "empty" | "loading" | "error";
interface ContentStateProps extends Omit<ViewProps, "children" | "role"> {
  state: ContentStateKind;
  title: string;
  description?: string;
  /** Standard compact recovery or next-step action. */
  action?: {
    label: string;
    onAction(): void;
  };
  /** Lazily render media inside the state's reactive owner. */
  renderMedia?: () => JSX.Element;
  /** Lazily render custom actions inside the state's reactive owner. */
  renderAction?: () => JSX.Element;
}
/** Mutually exclusive loading, empty, or error state for a bounded region. */
declare function ContentState(props: ContentStateProps): JSX.Element;
interface ResourceBoundaryProps extends Omit<ViewProps, "children" | "role"> {
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
  renderContent(): JSX.Element;
  renderEmptyMedia?: () => JSX.Element;
  renderErrorMedia?: () => JSX.Element;
}
/**
 * Mutually exclusive async resource boundary.
 *
 * Existing content remains mounted during a background refresh. This avoids
 * replacing a useful inspector or list with a loading spinner every time its
 * resource is refreshed.
 */
declare function ResourceBoundary(props: ResourceBoundaryProps): JSX.Element;
//#endregion
//#region src/components/dropdown-menu.d.ts
interface DropdownMenuItem extends MenuStateItem {
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
interface DropdownMenuTriggerProps {
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
interface DropdownMenuProps extends PopupMotionProps {
  trigger(props: DropdownMenuTriggerProps): JSX.Element;
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
//#region src/components/copy-button.d.ts
interface CopyButtonProps extends Omit<ButtonProps, "onClick"> {
  value: string;
  idleLabel?: string;
  copiedLabel?: string;
  copiedChildren?: JSX.Element;
  onCopied?: () => void;
  onCopyError?: (error: unknown) => void;
}
declare function CopyButton(props: CopyButtonProps): JSX.Element;
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
//#region src/components/data-table.d.ts
interface DataTableProps<TData> {
  model: TanStackDataTable<TData>;
  "aria-label": string;
  emptyMessage?: string;
  selectable?: boolean;
  renderCell?: (options: {
    value: unknown;
    columnId: string;
    row: Row$1<TData>;
  }) => JSX.Element;
}
/** Shadcn-style table anatomy backed by the framework-agnostic TanStack core. */
declare function DataTable<TData>(props: DataTableProps<TData>): JSX.Element;
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
//#region src/components/dev-server-error.d.ts
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
/** Native equivalent of Vite's browser error overlay. */
declare function DevServerErrorOverlay(): JSX.Element;
//#endregion
//#region src/components/diff-viewer.d.ts
type DiffFileStatus = "added" | "modified" | "deleted" | "renamed";
interface DiffFile {
  path: string;
  oldPath?: string;
  status: DiffFileStatus;
  additions: number;
  deletions: number;
  /** Unified patch for this file. Kept out of the visible tree while collapsed. */
  patch: string;
}
interface DiffViewerLabels {
  filesChanged: (count: number) => string;
  additions: (count: number) => string;
  deletions: (count: number) => string;
  empty: string;
  technicalDetails: string;
}
interface DiffViewerProps extends Omit<ViewProps, "children"> {
  files: readonly DiffFile[];
  labels?: Partial<DiffViewerLabels>;
  /** Files opened initially. Details remain opt-in by default. */
  defaultExpanded?: readonly string[];
}
/**
 * A progressive-disclosure code change viewer.
 *
 * The summary and file metadata are ordinary Wabou components. Unified patch
 * text is mounted only after disclosure and uses the DOM-free CodeMirror
 * document/native editor viewport for selection, copying, and large documents.
 */
declare function DiffViewer(props: DiffViewerProps): import("@wabou/core/jsx-runtime").JSX.Element;
//#endregion
//#region src/components/direction.d.ts
type Direction = "ltr" | "rtl";
interface DirectionProviderProps {
  dir: Direction;
  children?: JSX.Element;
}
/** Own logical direction in JavaScript instead of relying on web inheritance. */
declare function DirectionProvider(props: DirectionProviderProps): JSX.Element;
declare function useDirection(): Direction;
interface DirectionalRowProps extends Omit<ViewProps, "class"> {
  class?: string;
}
declare function DirectionalRow(props: DirectionalRowProps): JSX.Element;
interface DirectionalTextProps extends Omit<TextProps, "class"> {
  class?: string;
}
declare function DirectionalText(props: DirectionalTextProps): JSX.Element;
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
interface PasswordInputProps extends PasswordInputProps$1 {
  class?: string;
}
/** A native secret input whose value never crosses into JavaScript. */
declare function PasswordInput(props: PasswordInputProps): JSX.Element;
interface TextAreaProps extends TextAreaProps$1 {
  class?: string;
  /** Background utility owned by this textarea. Defaults to `bg-input`. */
  surfaceClass?: string;
  /** Use `none` when an enclosing composition owns the visual surface. */
  chrome?: "default" | "none";
}
declare function TextArea(props: TextAreaProps): JSX.Element;
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
interface CollapsibleProps extends Omit<ViewProps, "children" | "class"> {
  children?: JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  disabled?: boolean;
  reducedMotion?: boolean;
  onOpenChange?: (open: boolean) => void;
  class?: string;
}
declare function Collapsible(props: CollapsibleProps): import("@wabou/core/jsx-runtime").JSX.Element;
interface CollapsibleTriggerProps extends Omit<ButtonProps$1, "children" | "class" | "unstyled"> {
  children?: JSX.Element;
  /** Render the built-in trailing chevron. Disable when children provide one. */
  indicator?: boolean;
  class?: string;
}
declare function CollapsibleTrigger(props: CollapsibleTriggerProps): import("@wabou/core/jsx-runtime").JSX.Element;
interface CollapsibleContentProps extends ViewProps {
  duration?: number;
  ease?: Easing;
  animateInitial?: boolean;
}
declare function CollapsibleContent(props: CollapsibleContentProps): import("@wabou/core/jsx-runtime").JSX.Element;
type AccordionType = "single" | "multiple";
type AccordionValue = string | readonly string[];
declare function nextAccordionValue(current: AccordionValue, type: AccordionType, item: string, collapsible?: boolean): AccordionValue;
interface AccordionProps extends Omit<ViewProps, "children" | "class"> {
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
declare function Accordion(props: AccordionProps): import("@wabou/core/jsx-runtime").JSX.Element;
interface AccordionItemProps extends Omit<ViewProps, "children" | "class"> {
  value: string;
  disabled?: boolean;
  children?: JSX.Element;
  class?: string;
}
declare function AccordionItem(props: AccordionItemProps): import("@wabou/core/jsx-runtime").JSX.Element;
interface AccordionTriggerProps extends Omit<ButtonProps$1, "children" | "class" | "unstyled"> {
  children?: JSX.Element;
  class?: string;
}
declare function AccordionTrigger(props: AccordionTriggerProps): import("@wabou/core/jsx-runtime").JSX.Element;
type AccordionContentProps = ViewProps;
declare function AccordionContent(props: AccordionContentProps): import("@wabou/core/jsx-runtime").JSX.Element;
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
  /** Hide spinner semantics when a parent status already announces progress. */
  decorative?: boolean;
  /** Duration of one revolution in seconds. */
  duration?: number;
  /** Playback-rate multiplier. */
  speed?: number;
  paused?: boolean;
}): JSX.Element;
type KbdProps = TextProps;
declare function Kbd(props: KbdProps): JSX.Element;
type KbdGroupProps = ViewProps;
declare function KbdGroup(props: KbdGroupProps): JSX.Element;
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
//#region src/components/drop-zone.d.ts
interface DropZoneProps extends Omit<ViewProps, "class" | "children"> {
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
declare function pointInLayoutRect(point: FileDropPosition, rect: LayoutRect): boolean;
/**
 * A component-local target for native filesystem drops.
 *
 * Native backends publish window-relative coordinates, so the zone measures
 * its completed native layout before claiming an event. Events without a
 * position are deliberately ignored instead of being delivered ambiguously to
 * every mounted zone.
 */
declare function DropZone(props: DropZoneProps): JSX.Element;
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
  required?: boolean;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
declare function FieldSet(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
declare function FieldLegend(props: {
  children?: JSX.Element;
  variant?: "legend" | "label";
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
declare function FieldGroup(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
interface FieldLabelProps extends LabelProps {}
declare function FieldLabel(props: FieldLabelProps): import("@wabou/core/jsx-runtime").JSX.Element;
interface LabeledFieldProps {
  label: JSX.Element;
  description?: JSX.Element;
  invalid?: boolean;
  required?: boolean;
  disabled?: boolean;
  errors?: ReadonlyArray<FieldErrorLike | undefined>;
  class?: string;
  controlRef?: (node: Handle) => void;
  /** Render the native control and attach the supplied ref to its focus owner. */
  renderControl: (ref: (node: Handle) => void) => JSX.Element;
}
/**
 * A complete native field whose visible label always focuses its control.
 * This avoids repeating ad-hoc Handle plumbing in every settings surface.
 */
declare function LabeledField(props: LabeledFieldProps): import("@wabou/core/jsx-runtime").JSX.Element;
declare function FieldTitle(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
declare function FieldContent(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
declare function FieldDescription(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
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
}): import("@wabou/core/jsx-runtime").JSX.Element;
declare function FieldSeparator(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
type InputGroupOrientation = "horizontal" | "vertical";
type InputGroupVariant = "default" | "quiet";
type InputGroupAddonAlign = "inline-start" | "inline-end" | "block-start" | "block-end";
declare function inputGroupClass(orientation: InputGroupOrientation, focused: boolean, invalid: boolean, variant?: InputGroupVariant): string;
interface InputGroupProps extends Omit<ViewProps, "children"> {
  children?: JSX.Element;
  orientation?: InputGroupOrientation;
  variant?: InputGroupVariant;
  invalid?: boolean;
  disabled?: boolean;
  /** Background utility owned by the compound control. Defaults to `bg-input`. */
  surfaceClass?: string;
}
declare function InputGroup(props: InputGroupProps): import("@wabou/core/jsx-runtime").JSX.Element;
declare function InputGroupInput(props: InputProps): import("@wabou/core/jsx-runtime").JSX.Element;
interface InputGroupAddonProps extends ViewProps {
  align?: InputGroupAddonAlign;
  focusControl?: boolean;
}
declare function inputGroupAddonClass(align: InputGroupAddonAlign): string;
declare function InputGroupAddon(props: InputGroupAddonProps): import("@wabou/core/jsx-runtime").JSX.Element;
declare function InputGroupText(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
declare function InputGroupButton(props: ButtonProps): import("@wabou/core/jsx-runtime").JSX.Element;
declare function InputGroupTextArea(props: TextAreaProps$1 & {
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
//#endregion
//#region src/components/group-box.d.ts
type GroupBoxVariant = "normal" | "fill" | "outline";
interface GroupBoxProps extends Omit<ViewProps, "children" | "class"> {
  title?: JSX.Element;
  description?: JSX.Element;
  children?: JSX.Element;
  variant?: GroupBoxVariant;
  class?: string;
  headerClass?: string;
  contentClass?: string;
}
declare function groupBoxContentClass(variant?: GroupBoxVariant, className?: string): string;
/** A lightweight titled surface for related controls and settings rows. */
declare function GroupBox(props: GroupBoxProps): JSX.Element;
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
//#region src/components/icon-frame.d.ts
type IconFrameSize = "sm" | "default" | "lg";
type IconFrameVariant = "plain" | "muted" | "selected" | "solid";
interface IconFrameProps extends Omit<ViewProps, "children">, Pick<IconProps, "source" | "fill" | "label"> {
  /** Size of the square visual container. */
  size?: IconFrameSize;
  /** Explicit icon size. Defaults are tuned for each container size. */
  iconSize?: number | string;
  variant?: IconFrameVariant;
  iconClass?: string;
}
/**
 * A square icon surface whose geometry is correct by construction.
 *
 * Use this for standalone icon tiles. Buttons and menu items already own their
 * icon alignment and should continue to use `Icon` directly.
 */
declare function IconFrame(props: IconFrameProps): import("@wabou/core/jsx-runtime").JSX.Element;
//#endregion
//#region src/components/image-list.d.ts
interface ImageListProps<T> {
  /** Reactive backing collection. Only visible rows mount their Image nodes. */
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
  renderTrailing?: (item: T, index: number) => JSX.Element;
  /** Optional generated thumbnail; bypasses Image resource loading for this row. */
  renderThumbnail?: (item: T, index: number) => JSX.Element;
}
/**
 * A virtualized, selectable image list for page strips, albums and file pickers.
 * Resource creation remains owned by the caller; this component only borrows handles.
 */
declare function ImageList<T>(props: ImageListProps<T>): JSX.Element;
//#endregion
//#region src/components/image-viewport.d.ts
interface ImageViewportSize {
  width: number;
  height: number;
}
interface ImageViewportPoint {
  x: number;
  y: number;
}
interface ImageViewportRect extends ImageViewportPoint {
  width: number;
  height: number;
}
interface ImageViewportTransform {
  readonly viewport: ImageViewportSize;
  readonly image: ImageViewportSize;
  readonly frame: ImageViewportRect;
  readonly scale: number;
  imageToViewport(point: ImageViewportPoint): ImageViewportPoint;
  viewportToImage(point: ImageViewportPoint): ImageViewportPoint;
}
/** Deterministic contain + zoom + pan transform shared by paint and annotations. */
declare function imageViewportTransform(options: {
  viewport: ImageViewportSize;
  image: ImageViewportSize;
  zoom?: number;
  pan?: ImageViewportPoint;
}): ImageViewportTransform;
interface ImageViewportProps extends Omit<ViewProps, "children" | "ref" | "onWheel" | "onPointerDown" | "onPointerMove" | "onPointerUp" | "onPointerCancel"> {
  resource?: ImageResourceHandle;
  /** Intrinsic image size in image pixels. */
  imageSize?: ImageViewportSize;
  zoom?: number;
  pan?: ImageViewportPoint;
  pannable?: boolean;
  onPanChange?: (pan: ImageViewportPoint) => void;
  /** Optional replacement for the native Image, useful for generated media. */
  media?: JSX.Element;
  children?: JSX.Element;
  imageLabel?: string;
  onResourceReady?: (event: ImageResourceReadyEvent) => void;
  onResourceError?: (event: ImageResourceErrorEvent) => void;
}
/** A clipped image-space viewport with one explicit, reusable coordinate model. */
declare function ImageViewport(props: ImageViewportProps): JSX.Element;
interface AnnotationRegion extends ImageViewportRect {
  id: string;
  label?: string;
}
declare function clampAnnotationRegion(region: AnnotationRegion, image: ImageViewportSize, minimumSize?: number): AnnotationRegion;
interface AnnotationLayerProps extends Omit<ViewProps, "children" | "onPointerDown" | "onPointerMove" | "onPointerUp" | "onPointerCancel"> {
  regions: readonly AnnotationRegion[];
  selectedId?: string | null;
  minimumSize?: number;
  createRegionId?: () => string;
  interactionMode?: "edit" | "passthrough";
  onRegionsChange?: (regions: readonly AnnotationRegion[]) => void;
  onSelectedIdChange?: (id: string | null) => void;
}
/** Editable image-space regions composed above an ImageViewport. */
declare function AnnotationLayer(props: AnnotationLayerProps): JSX.Element;
interface ImageOverlayItem extends ImageViewportRect {
  id: string;
}
interface ImageOverlayLayerProps<T extends ImageOverlayItem> extends Omit<ViewProps, "children"> {
  items: readonly T[];
  children: (item: T, index: () => number) => JSX.Element;
}
/** Read-only image-space overlays sharing the viewport's exact zoom and pan transform. */
declare function ImageOverlayLayer<T extends ImageOverlayItem>(props: ImageOverlayLayerProps<T>): JSX.Element;
//#endregion
//#region src/components/inline-edit.d.ts
interface InlineEditProps {
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
declare function InlineEdit(props: InlineEditProps): JSX.Element;
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
  /** Persistent list selection, independent from transient pointer hover. */
  selected?: boolean;
  /** Disable the complete row subtree, including trailing actions. */
  disabled?: boolean;
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
  itemCount?: number;
  balanceLastRow?: boolean;
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
}): import("@wabou/core/jsx-runtime").JSX.Element;
declare function SplitPaneMain(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
declare function SplitPaneAside(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
/**
 * Master/detail layout whose detail region can move from an inline rail to a
 * modal surface without changing the application's selection model.
 */
declare function AdaptiveSplitPane(props: {
  children?: JSX.Element;
  /** Controlled compact mode. Omit it to measure this pane natively. */
  compact?: boolean;
  /** Inclusive native content width that activates compact mode. */
  compactAt?: number;
  onCompactChange?: (compact: boolean) => void;
  "aria-label"?: string;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
declare function AdaptiveSplitPaneMain(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
declare function AdaptiveSplitPaneDetail(props: {
  children?: JSX.Element;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  "aria-label": string;
  class?: string;
  modalClass?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
//#endregion
//#region src/components/listbox.d.ts
interface ListboxOption {
  value: string;
  label: string;
  /** Accessible identity when the visible label is not unique or sufficiently descriptive. */
  accessibilityLabel?: string;
  description?: string;
  disabled?: boolean;
}
interface ListboxProps {
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
  renderLeading?: (option: ListboxOption) => JSX.Element;
  renderTrailing?: (option: ListboxOption) => JSX.Element;
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
declare function Listbox(props: ListboxProps): JSX.Element;
//#endregion
//#region src/components/markdown.d.ts
type MarkdownVariant = "document" | "conversation" | "prompt";
interface MarkdownProps {
  source: string;
  /** Repair an incomplete Markdown tail while text is still arriving. */
  streaming?: boolean;
  /** Document typography by default; conversation and prompt stay message-sized. */
  variant?: MarkdownVariant;
  class?: string;
  "aria-label"?: string;
}
/** Parses GFM in JavaScript and renders native Wabou components, without HTML or a DOM. */
declare function Markdown(props: MarkdownProps): JSX.Element;
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
declare function messageActionsClass(align?: MessageAlign, className?: string): string;
interface MessageActionsProps extends Omit<ViewProps, "class"> {
  /** Override the containing message direction for a local action rail. */
  align?: MessageAlign;
  /** Keep actions visible, or reveal them while their message is hovered/focused. */
  visibility?: "always" | "interaction";
  class?: string;
}
/** Compact, consistently aligned actions belonging to one message. */
declare function MessageActions(props: MessageActionsProps): JSX.Element;
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
interface MessageScrollIntoViewOptions {
  margin?: number;
  align?: "nearest" | "start";
}
declare function messageScrollRange(contentHeight: number, viewportHeight: number): number;
declare function isMessageScrollNearEnd(scrollY: number, contentHeight: number, viewportHeight: number, threshold?: number): boolean;
/** Smallest vertical delta that reveals a target without disturbing visible content. */
declare function messageScrollRevealDelta(viewport: LayoutRect, target: LayoutRect, margin?: number): number;
/** Vertical delta that places a target at the viewport's reading start. */
declare function messageScrollStartDelta(viewport: LayoutRect, target: LayoutRect, margin?: number): number;
interface MessageScrollerControls {
  followingEnd(): boolean;
  canScrollStart(): boolean;
  canScrollEnd(): boolean;
  activeAnchor(): string | undefined;
  scrollTo(direction: MessageScrollDirection): void;
  scrollIntoView(target: Handle, options?: MessageScrollIntoViewOptions): void;
  scrollToAnchor(anchor: string, options?: MessageScrollIntoViewOptions): void;
}
declare function useMessageScroller(): MessageScrollerControls;
interface MessageScrollerProps extends ViewProps {
  /** Start attached to the end of the conversation. Defaults to true. */
  followEnd?: boolean;
  /** Distance in logical pixels that still counts as being at the end. */
  endThreshold?: number;
}
interface MessageAnchorRect {
  id: string;
  rect: LayoutRect;
}
/** Pick the last conversation anchor that has crossed the reading line. */
declare function activeMessageAnchor(viewport: LayoutRect, anchors: readonly MessageAnchorRect[], offset?: number): string | undefined;
declare function MessageScroller(props: MessageScrollerProps): JSX.Element;
interface MessageScrollerViewportProps extends ViewProps {}
declare function MessageScrollerViewport(props: MessageScrollerViewportProps): JSX.Element;
declare function MessageScrollerContent(props: ViewProps): JSX.Element;
interface MessageScrollerItemProps extends ViewProps {
  /** Stable semantic id used by conversation navigation. */
  anchor?: string;
}
declare function MessageScrollerItem(props: MessageScrollerItemProps): JSX.Element;
interface MessageScrollerButtonProps extends Omit<ButtonProps, "onClick"> {
  direction?: MessageScrollDirection;
  onClick?: ButtonProps["onClick"];
}
declare function MessageScrollerButton(props: MessageScrollerButtonProps): JSX.Element;
interface MessageScrollerNavigatorItem {
  id: string;
  label: string;
}
interface MessageScrollerNavigatorProps {
  items: readonly MessageScrollerNavigatorItem[];
  "aria-label": string;
  itemAriaLabel(item: MessageScrollerNavigatorItem, index: number): string;
  minItems?: number;
  class?: string;
  railClass?: string;
}
/** Compact anchor rail for navigating long retained conversations. */
declare function MessageScrollerNavigator(props: MessageScrollerNavigatorProps): JSX.Element;
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
  count: number;
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
declare function PageViewport(props: PageViewportProps): JSX.Element;
declare const pageHeaderClass: (className?: string, stacked?: boolean) => string;
declare const pageHeaderTitleClass: () => string;
declare const pageHeaderDescriptionClass: (stacked?: boolean) => string;
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
//#region src/components/onboarding.d.ts
interface OnboardingProps extends Omit<PageViewportProps, "class" | "contentClass"> {
  class?: string;
  contentClass?: string;
}
/**
 * Full-height first-run boundary with native scrolling and a readable measure.
 * Content stays centered when it fits and remains reachable when it grows.
 */
declare function Onboarding(props: OnboardingProps): JSX.Element;
declare function OnboardingHeader(props: ViewProps): JSX.Element;
declare function OnboardingHeading(props: ViewProps): JSX.Element;
declare function OnboardingTitle(props: TextProps): JSX.Element;
declare function OnboardingDescription(props: TextProps): JSX.Element;
declare function OnboardingFooter(props: ViewProps): JSX.Element;
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
type ProgressSize = "xs" | "sm" | "default" | "lg";
declare function normalizeProgressValue(value: number | undefined, minValue: number | undefined, maxValue: number | undefined): ProgressValueDetails;
/** Semantic progress state with explicit, composable visual parts. */
declare function ProgressRoot(props: ProgressRootProps): JSX.Element;
interface ProgressTrackProps extends ViewProps {
  size?: ProgressSize;
}
declare function ProgressTrack(props: ProgressTrackProps): JSX.Element;
declare function ProgressFill(props: ViewProps): JSX.Element;
declare function ProgressLabel(props: TextProps): JSX.Element;
declare function ProgressValueLabel(props: TextProps): JSX.Element;
interface ProgressProps extends Omit<ProgressRootProps, "children" | "class"> {
  /** Classes applied to the visual track, preserving the original shorthand. */
  class?: string;
  size?: ProgressSize;
}
/** Compact progress bar; use ProgressRoot and parts for custom composition. */
declare function Progress(props: ProgressProps): JSX.Element;
interface ProgressCircleProps extends Omit<ProgressRootProps, "children" | "class"> {
  class?: string;
  size?: ProgressSize;
}
/** Compact circular progress indicator using the same semantic range contract. */
declare function ProgressCircle(props: ProgressCircleProps): JSX.Element;
//#endregion
//#region src/components/prompt-composer.d.ts
interface PromptComposerProps extends Omit<ViewProps, "class"> {
  class?: string;
  disabled?: boolean;
  invalid?: boolean;
  /** Background utility owned by the compound surface. */
  surfaceClass?: string;
}
interface PromptComposerRowProps extends Omit<ViewProps, "class"> {
  class?: string;
  /** Allow controls to form additional rows when the embedding surface opts in. */
  wrap?: boolean;
}
declare function promptComposerEditorHeightClass(value: string): string;
interface PromptComposerEditorProps extends TextAreaProps {
  value?: string;
}
/** Native multiline editor with density and chrome owned by PromptComposer. */
declare function PromptComposerEditor(props: PromptComposerEditorProps): JSX.Element;
/** Stable circular primary action for a PromptComposer toolbar. */
declare function PromptComposerAction(props: ButtonProps): JSX.Element;
declare function promptComposerClass(focused: boolean, invalid: boolean, disabled: boolean, className?: string): string;
/** Shared compound surface for prompts, attachments, controls and status. */
declare function PromptComposer(props: PromptComposerProps): JSX.Element;
/** Compact metadata row above the authored prompt. */
declare function PromptComposerStatus(props: ViewProps): JSX.Element;
/** Responsive action row below the authored prompt. */
declare function PromptComposerToolbar(props: PromptComposerRowProps): JSX.Element;
/** Shrink-safe group for the composer tools preceding its primary action. */
declare function PromptComposerTools(props: PromptComposerRowProps): JSX.Element;
//#endregion
//#region src/components/prompt-suggestion.d.ts
interface PromptSuggestionsProps extends Omit<ResponsiveGridProps, "gap" | "initialColumns" | "maxColumns" | "minColumnWidth" | "balanceLastRow"> {
  itemCount?: number;
  minColumnWidth?: number;
  maxColumns?: 1 | 2 | 3;
  gap?: number;
}
/** Container-responsive starter actions for empty conversations and assistants. */
declare function PromptSuggestions(props: PromptSuggestionsProps): JSX.Element;
interface PromptSuggestionProps extends Omit<ButtonProps, "children" | "class"> {
  title: string;
  description?: string;
  icon?: string;
  class?: string;
}
/** One explicit prompt choice; the application remains responsible for its payload. */
declare function PromptSuggestion(props: PromptSuggestionProps): JSX.Element;
//#endregion
//#region src/components/property-list.d.ts
interface PropertyListProps extends Omit<ViewProps, "class"> {
  class?: string;
}
declare function PropertyList(props: PropertyListProps): JSX.Element;
interface PropertyRowProps {
  name: string;
  value: string;
  class?: string;
  nameClass?: string;
  valueClass?: string;
}
/** Compact two-column data intended for inspectors and settings summaries. */
declare function PropertyRow(props: PropertyRowProps): JSX.Element;
//#endregion
//#region src/components/qr-code.d.ts
type QrCodeErrorCorrection = "L" | "M" | "Q" | "H";
interface QrCodeMatrix {
  readonly size: number;
  readonly data: readonly (readonly boolean[])[];
}
interface QRCodeProps extends Omit<ViewProps, "children" | "style"> {
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
declare function encodeQrCode(value: QrCodeGenerateData, errorCorrection?: QrCodeErrorCorrection): QrCodeMatrix;
/**
 * Convert consecutive dark modules into one retained native path. Horizontal
 * runs keep the bridge traffic and native scene node count independent of the
 * number of QR modules.
 */
declare function qrCodePath(matrix: QrCodeMatrix, renderedSize: number, quiet: number, foreground?: number): import("@wabou/core").VectorPath;
/** A QR encoder from the JS ecosystem rendered as one native vector path. */
declare function QRCode(props: QRCodeProps): JSX.Element;
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
//#region src/components/reasoning.d.ts
interface ReasoningProps extends CollapsibleProps {}
/** Quiet disclosure surface for model reasoning or intermediate thought. */
declare function Reasoning(props: ReasoningProps): JSX.Element;
interface ReasoningTriggerProps extends Omit<CollapsibleTriggerProps, "children" | "class"> {
  label?: string;
  streaming?: boolean;
  class?: string;
}
declare function ReasoningTrigger(props: ReasoningTriggerProps): JSX.Element;
declare function ReasoningContent(props: CollapsibleContentProps): JSX.Element;
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
  /** Quiet fields stay transparent until focus; useful inside navigation chrome. */
  variant?: "default" | "quiet";
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
declare function Select(props: SelectProps): JSX.Element;
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
declare function Checkbox(props: CheckboxProps): JSX.Element;
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
  children?: JSX.Element;
  onValueChange?: (value: string) => void;
}
declare function RadioGroup(props: RadioGroupProps): JSX.Element;
interface RadioGroupItemProps {
  value: string;
  label?: string;
  "aria-label"?: string;
  disabled?: boolean;
  size?: SelectionControlSize;
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
  /** Join items into one clipped control surface owned by the group. */
  segmented?: boolean;
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
  "aria-label"?: string;
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
interface LabeledSeparatorProps extends Omit<ViewProps, "class"> {
  class?: string;
}
/**
 * A horizontal divider whose compact center content names the boundary.
 *
 * Keep the content short. Interactive content is supported so disclosures can
 * explain what happened between two regions without turning into a full row.
 */
declare function LabeledSeparator(props: LabeledSeparatorProps): JSX.Element;
//#endregion
//#region src/components/settings.d.ts
type SettingsItemOrientation = "horizontal" | "vertical";
interface SettingsItemProps extends Omit<ViewProps, "children" | "class"> {
  title: string;
  description?: JSX.Element;
  children?: JSX.Element;
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
declare function SettingsItem(props: SettingsItemProps): JSX.Element;
interface SettingsSectionProps {
  title: string;
  description?: string;
  children?: JSX.Element;
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
declare function SettingsSection(props: SettingsSectionProps): JSX.Element;
interface SettingsGroupProps {
  title: string;
  description?: string;
  children?: JSX.Element;
  class?: string;
}
/** A titled field group inside a SettingsSection control surface. */
declare function SettingsGroup(props: SettingsGroupProps): JSX.Element;
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
//#region src/components/shortcut-recorder.d.ts
interface RecordedShortcut {
  chord: string;
  parts: readonly string[];
}
declare function shortcutFromKeyEvent(event: Pick<ButtonKeyEvent, "key" | "mods" | "primary">): RecordedShortcut | undefined;
interface ShortcutRecorderProps {
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  label?: string;
  placeholder?: string;
  disabled?: boolean;
  class?: string;
}
declare function ShortcutRecorder(props: ShortcutRecorderProps): JSX.Element;
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
declare const componentsThemeContract: Readonly<{
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
}>;
declare function componentsControlSize(size: ComponentsControlSize): string;
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
  /** Native shadow recipe for sidebars that float inside a window frame. */
  elevation?: ComponentsElevation;
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
interface SidebarMenuProps extends Omit<ViewProps, "class"> {
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  class?: string;
}
/**
 * Single-selection navigation scope for sidebar destinations.
 * Buttons without a value remain actions and never become selected items.
 */
declare function SidebarMenu(props: SidebarMenuProps): JSX.Element;
interface SidebarMenuButtonProps extends Omit<ButtonProps$1, "class" | "unstyled"> {
  /** Value controlled by the nearest SidebarMenu. Omit for action buttons. */
  value?: string;
  class?: string;
}
/** Consistent navigation row; applications still own activation and routing. */
declare function SidebarMenuButton(props: SidebarMenuButtonProps): JSX.Element;
/** Fixed icon slot that keeps navigation labels on one shared baseline. */
declare function SidebarMenuIcon(props: ViewProps): JSX.Element;
/** Truncating label slot for rows that also contain icons or suffix actions. */
declare function SidebarMenuLabel(props: TextProps): JSX.Element;
/** End-aligned metadata or action slot that never compresses the row label. */
declare function SidebarMenuSuffix(props: ViewProps): JSX.Element;
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
  /** Direction of the track and pointer interaction. */
  orientation?: "horizontal" | "vertical";
  /** Fill from the thumb toward the maximum end without changing values. */
  reversed?: boolean;
  label: string;
  valueText?: (value: number) => string;
  onValueChange?: (value: number) => void;
  class?: string;
}
declare function Slider(props: SliderProps): JSX.Element;
//#endregion
//#region src/components/split-button.d.ts
interface SplitButtonProps extends Omit<ButtonProps, "children" | "aria-label"> {
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
declare function SplitButton(props: SplitButtonProps): JSX.Element;
//#endregion
//#region src/components/stat-card.d.ts
interface StatCardProps extends Omit<ViewProps, "children"> {
  label: string;
  value: string;
  description?: string;
  trend?: string;
  indicatorClass?: string;
}
declare function StatCard(props: StatCardProps): JSX.Element;
//#endregion
//#region src/components/status-bar.d.ts
interface StatusBarProps extends Omit<ViewProps, "class"> {
  class?: string;
}
/** Persistent low-emphasis application state at the bottom of a desktop window. */
declare function StatusBar(props: StatusBarProps): JSX.Element;
interface StatusBarItemProps extends Omit<TextProps, "class"> {
  class?: string;
  grow?: boolean;
}
declare function StatusBarItem(props: StatusBarItemProps): JSX.Element;
declare function StatusBarSeparator(props: {
  class?: string;
}): JSX.Element;
interface StatusBarGroupProps extends Omit<ViewProps, "class"> {
  class?: string;
  grow?: boolean;
  shrink?: boolean;
}
/** A shrink-safe status bar group for icons, indicators and related text. */
declare function StatusBarGroup(props: StatusBarGroupProps): JSX.Element;
type StatusBarIndicatorTone = "accent" | "danger" | "muted" | "success";
interface StatusBarIndicatorProps {
  tone?: StatusBarIndicatorTone;
  class?: string;
}
/** Compact, decorative state indicator with a theme-aware semantic tone. */
declare function StatusBarIndicator(props: StatusBarIndicatorProps): JSX.Element;
//#endregion
//#region src/components/stepper.d.ts
interface StepperStep {
  id: string;
  label: string;
  description?: string;
}
interface StepperProps extends Omit<ViewProps, "children"> {
  steps: readonly StepperStep[];
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
}
declare function Stepper(props: StepperProps): JSX.Element;
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
declare function Switch(props: SwitchProps): JSX.Element;
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
  children?: JSX.Element;
}
/**
 * Bounded, optionally closeable tab chrome. The tab trigger and close action
 * remain sibling hit targets so closing a tab never selects it first.
 */
declare function TabsItem(props: TabsItemProps): JSX.Element;
declare function TabsTrigger(props: TabsTriggerProps): JSX.Element;
declare function TabsContent(props: {
  value: string;
  /** Keep stateful/native content mounted while hiding inactive panels. */
  keepMounted?: boolean;
  class?: string;
  children?: JSX.Element;
}): JSX.Element;
//#endregion
//#region src/components/timeline.d.ts
interface TimelineItem {
  id: string;
  title: string;
  description?: string;
  time?: string;
  status?: "complete" | "current" | "pending";
}
interface TimelineProps extends Omit<ViewProps, "children"> {
  items: readonly TimelineItem[];
}
declare function Timeline(props: TimelineProps): JSX.Element;
//#endregion
//#region src/components/title-bar.d.ts
interface WindowFrameProps extends ViewProps {
  class?: string;
  children?: JSX.Element;
  /** Clip the application surface to desktop-style corners when restored. */
  rounded?: boolean;
  /** Let the native window material remain visible beneath a translucent client surface. */
  material?: "solid" | "translucent";
}
declare function windowFrameMaterialStyle(theme: "light" | "dark", material?: "solid" | "translucent"): WabouStyle;
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
//#region src/components/tool.d.ts
type ToolStatus = "pending" | "running" | "success" | "failed";
interface ToolProps extends CollapsibleProps {}
/** Composable disclosure root for an AI or automation tool invocation. */
declare function Tool(props: ToolProps): JSX.Element;
interface ToolHeaderProps extends Omit<CollapsibleTriggerProps, "children" | "class"> {
  title: string;
  summary?: string;
  status?: ToolStatus;
  icon?: string;
  class?: string;
}
declare function toolHeaderLabel(title: string, summary?: string, status?: ToolStatus): string;
/** Stable title, summary, status and disclosure geometry for one tool call. */
declare function ToolHeader(props: ToolHeaderProps): JSX.Element;
declare function ToolContent(props: CollapsibleContentProps): JSX.Element;
interface ToolCodeSectionProps extends Omit<ViewProps, "children" | "class"> {
  code: string;
  label: string;
  language?: string;
  copyable?: boolean;
  class?: string;
  codeClass?: string;
  codeProps?: Omit<CodeBlockProps, "code" | "language" | "copyable">;
}
/** Labelled code payload used for tool parameters, results and errors. */
declare function ToolCodeSection(props: ToolCodeSectionProps): JSX.Element;
interface ToolInputProps extends Omit<ToolCodeSectionProps, "label"> {
  label?: string;
}
declare function ToolInput(props: ToolInputProps): JSX.Element;
interface ToolOutputProps extends Omit<ToolCodeSectionProps, "label"> {
  label?: string;
  error?: boolean;
}
declare function ToolOutput(props: ToolOutputProps): JSX.Element;
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
  /** Optional keyboard shortcut presented as a native keycap. */
  shortcut?: string;
}
interface TooltipContentProps {
  children?: JSX.Element;
  shortcut?: string;
  id?: string;
  class?: string;
}
/** Visual content shared by managed and explicitly composed tooltips. */
declare function TooltipContent(props: TooltipContentProps): JSX.Element;
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
//#region src/components/typography.d.ts
type TypographyTextProps = Omit<TextProps, "class"> & {
  class?: string;
};
declare const TypographyH1: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
declare const TypographyH2: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
declare const TypographyH3: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
declare const TypographyH4: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
declare const TypographyP: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
declare const TypographyLead: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
declare const TypographyLarge: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
declare const TypographySmall: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
declare const TypographyMuted: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
declare const TypographyInlineCode: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
declare function TypographyBlockquote(props: TypographyTextProps): JSX.Element;
declare function TypographyList(props: ViewProps): JSX.Element;
declare function TypographyListItem(props: TypographyTextProps): JSX.Element;
//#endregion
//#region src/components/workbench-style.d.ts
declare const workbenchClass: (className?: string) => string;
declare const workbenchSidebarClass: (className?: string) => string;
declare const workbenchMainClass: (className?: string) => string;
declare const workbenchHeaderClass: (className?: string) => string;
declare const workbenchContentClass: (className?: string) => string;
/** Shared readable column for workbench transcripts, editors and composers. */
declare const workbenchContentColumnClass: (className?: string) => string;
declare const workbenchFooterClass: (className?: string) => string;
/** Bounded auxiliary pane beside primary workbench content. */
declare const workbenchInspectorClass: (className?: string) => string;
/** Fixed inspector chrome aligned with the inspector body. */
declare const workbenchInspectorHeaderClass: (className?: string) => string;
/** Shrink-safe body for inspector content and nested scroll areas. */
declare const workbenchInspectorContentClass: (className?: string) => string;
//#endregion
//#region src/components/workbench.d.ts
/** Full-window desktop application boundary with explicit shrink semantics. */
declare function Workbench(props: ViewProps): JSX.Element;
/** Fixed-width navigation rail paired with a {@link WorkbenchMain}. */
declare function WorkbenchSidebar(props: SidebarProps): JSX.Element;
/** The resizable application column beside the navigation rail. */
declare function WorkbenchMain(props: ViewProps): JSX.Element;
/** Shared 48px chrome row for both sidebar and content headers. */
declare function WorkbenchHeader(props: ViewProps): JSX.Element;
/** Bounded application content. Add a ScrollArea inside when scrolling is needed. */
declare function WorkbenchContent(props: ViewProps): JSX.Element;
/** A centered 896px desktop content column that still shrinks with its pane. */
declare function WorkbenchContentColumn(props: ViewProps): JSX.Element;
/** Fixed chrome below the workbench content, such as a composer or status bar. */
declare function WorkbenchFooter(props: ViewProps): JSX.Element;
/** Fixed-width auxiliary pane for file previews, diffs and contextual tools. */
declare function WorkbenchInspector(props: ViewProps): JSX.Element;
/** Inspector title row with a stable height and bounded children. */
declare function WorkbenchInspectorHeader(props: ViewProps): JSX.Element;
interface WorkbenchInspectorTitlebarProps extends Omit<ViewProps, "children"> {
  title: string;
  description?: string;
  closeLabel?: string;
  onClose?: () => void;
  /** Lazily render an optional action before the standard close button. */
  renderAction?: () => JSX.Element;
}
/**
 * Consistent inspector chrome with a bounded title, optional description and
 * correctly sized trailing actions.
 */
declare function WorkbenchInspectorTitlebar(props: WorkbenchInspectorTitlebarProps): JSX.Element;
/** Flexible, clipped inspector body. Add a ScrollArea inside when needed. */
declare function WorkbenchInspectorContent(props: ViewProps): JSX.Element;
interface WorkbenchInspectorStateProps extends ContentStateProps {}
/** Mutually exclusive centered state for a bounded inspector body. */
declare function WorkbenchInspectorState(props: WorkbenchInspectorStateProps): JSX.Element;
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
type FpsProps = FpsBaseProps & ({
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
declare function Fps(props: FpsProps): JSX.Element;
//#endregion
//#region src/router/data.d.ts
type WabouDataRouter<TRouteTree extends AnyRoute = AnyRoute> = RouterCore<TRouteTree, TrailingSlashOption, boolean, RouterHistory>;
interface RouterPersistenceOptions {
  /** Application-scoped KV handle. The host must opt in with `HostBuilder::kv()`. */
  kv: Kv;
  /** Explicit hierarchical identity for this router. */
  key: KvKey;
  /** Ignore locations written by another schema version. Defaults to 1. */
  version?: number;
}
type WabouRouterConstructorOptions<TRouteTree extends AnyRoute, TTrailingSlash extends TrailingSlashOption, TStructuralSharing extends boolean, THistory extends RouterHistory, TDehydrated extends Record<string, any>> = RouterConstructorOptions<TRouteTree, TTrailingSlash, TStructuralSharing, THistory, TDehydrated> & {
  /** Optionally restore and save the last committed native location. */
  persistence?: RouterPersistenceOptions;
};
/** Create a TanStack Router Core instance adapted to Solid 2 and native history. */
declare function createDataRouter<TRouteTree extends AnyRoute, TTrailingSlash extends TrailingSlashOption = "never", TStructuralSharing extends boolean = false, THistory extends RouterHistory = RouterHistory, TDehydrated extends Record<string, any> = Record<string, any>>(options: WabouRouterConstructorOptions<TRouteTree, TTrailingSlash, TStructuralSharing, THistory, TDehydrated>): RouterCore<TRouteTree, TTrailingSlash, TStructuralSharing, THistory, TDehydrated>;
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
export { Accordion, AccordionContent, AccordionContentProps, AccordionItem, AccordionItemProps, AccordionProps, AccordionTrigger, AccordionTriggerProps, AccordionType, type ActiveResult, ActivityStatus, ActivityStatusIndicator, ActivityStatusIndicatorProps, ActivityStatusProps, ActivityStatusTone, AdaptiveSplitPane, AdaptiveSplitPaneDetail, AdaptiveSplitPaneMain, Alert, AlertActions, AlertDescription, AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogProps, AlertDialogTitle, AlertProps, AlertSize, AlertTitle, AlertVariant, AnimationControls, type AnimationFrameCallback, AnimationOptions, AnimationState, AnimationType, AnimationValue, AnnotationLayer, AnnotationLayerProps, AnnotationRegion, AspectRatio, AspectRatioProps, Attachment, AttachmentAction, AttachmentActions, AttachmentContent, AttachmentDescription, AttachmentGroup, AttachmentMedia, AttachmentOrientation, AttachmentProps, AttachmentSize, AttachmentState, AttachmentTitle, Avatar, AvatarGroup, AvatarGroupCount, AvatarGroupCountProps, AvatarGroupProps, AvatarProps, AvatarSize, Badge, BadgeProps, BadgeSize, BadgeVariant, BaseRootRoute, BaseRoute, Breadcrumb, BreadcrumbEllipsis, BreadcrumbEllipsisProps, BreadcrumbItem, BreadcrumbLink, BreadcrumbLinkProps, BreadcrumbList, BreadcrumbPage, BreadcrumbPageProps, BreadcrumbProps, BreadcrumbSeparator, BreadcrumbSeparatorProps, Bubble, BubbleContent, BubbleGroup, BubbleProps, BubbleReactions, BubbleVariant, Button, ButtonGroup, ButtonGroupProps, ButtonGroupSeparator, ButtonGroupSeparatorProps, ButtonGroupText, ButtonProps, ButtonSize, ButtonVariant, Calendar, CalendarDate, CalendarLabels, CalendarProps, Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardProps, CardSize, CardTitle, CardVariant, Carousel, CarouselApi, CarouselContent, CarouselContentProps, CarouselItem, CarouselItemProps, CarouselNext, CarouselOrientation, CarouselPrevious, CarouselProps, Center, ChartConfig, ChartContainer, ChartContainerProps, ChartEmpty, ChartLegend, ChartSeriesConfig, Checkbox, type CheckboxProps, CodeBlock, CodeBlockProps, Collapsible, CollapsibleContent, CollapsibleContentProps, CollapsiblePresence, CollapsibleProps, CollapsibleTrigger, CollapsibleTriggerProps, Column, Combobox, ComboboxOption, ComboboxProps, Command, CommandItem, CommandList, CommandListNavigationOptions, CommandListProps, CommandProps, type ComponentsControlSize, type ComponentsElevation, ComponentsProvider, type ComponentsProviderProps, type ComponentsTheme, type ContainerMatch, type ContainerSizeQuery, ContentState, ContentStateKind, ContentStateProps, ContextMenu, ContextMenuProps, ContextMenuTriggerProps, CopyButton, CopyButtonProps, CreateToastsOptions, DataTable, DataTableProps, DatePicker, DatePickerProps, type DelayedOpenController, type DelayedOpenOptions, DevServerDiagnostic, DevServerErrorOverlay, Dialog, type ModalControls as DialogControls, DialogDescription, DialogDescription as SheetDescription, DialogFooter, DialogFooter as SheetFooter, DialogHeader, DialogHeader as SheetHeader, DialogProps, DialogScrollBody, DialogScrollBody as SheetScrollBody, DialogScrollBodyProps, DialogTitle, DialogTitle as SheetTitle, DiffFile, DiffFileStatus, DiffViewer, DiffViewerLabels, DiffViewerProps, Direction, DirectionProvider, DirectionProviderProps, DirectionalRow, DirectionalRowProps, DirectionalText, DirectionalTextProps, DirectoryPicker, DirectoryPickerProps, Drawer, DrawerClose, DrawerDescription, DrawerDirection, DrawerFooter, DrawerHandle, DrawerHeader, DrawerOpenChangeReason, DrawerProps, DrawerTitle, DropZone, DropZoneProps, DropdownMenu, DropdownMenuItem, DropdownMenuKeyEvent, DropdownMenuProps, DropdownMenuTriggerProps, Easing, EasingFunction, Editor, type EditorProps, Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaProps, EmptyMediaVariant, EmptyProps, EmptyTitle, EmptyVariant, FORM_ERROR, Field, FieldContent, FieldDescription, FieldError, FieldErrorLike, FieldGroup, FieldLabel, FieldLabelProps, FieldLegend, FieldOrientation, FieldSeparator, FieldSet, FieldTitle, type FocusResult, type FocusWithinResult, type FormDraft, type FormDraftErrors, type FormDraftFieldUpdater, type FormDraftOptions, Fps, FpsProps, GroupBox, GroupBoxProps, GroupBoxVariant, HoverCard, HoverCardProps, HoverCardTriggerProps, Icon, IconFrame, IconFrameProps, IconFrameSize, IconFrameVariant, type IconProps, Image, ImageList, ImageListProps, ImageOverlayItem, ImageOverlayLayer, ImageOverlayLayerProps, type ImageProps, type ImageResourceDescriptor, type ImageResourceErrorEvent, type ImageResourceHandle, type ImageResourceReadyEvent, type ImageResourceRequest, ImageViewport, ImageViewportPoint, ImageViewportProps, ImageViewportRect, ImageViewportSize, ImageViewportTransform, InlineEdit, InlineEditProps, Input, InputGroup, InputGroupAddon, InputGroupAddonAlign, InputGroupAddonProps, InputGroupButton, InputGroupInput, InputGroupOrientation, InputGroupProps, InputGroupText, InputGroupTextArea, InputGroupVariant, InputOTP, InputOTPGroup, InputOTPProps, InputOTPSeparator, InputOTPSlot, InputOTPSlotProps, InputProps, Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader, ItemMedia, ItemMediaVariant, ItemProps, ItemSeparator, ItemSize, ItemTitle, ItemVariant, Kbd, KbdGroup, type KeyedSelection, type KeyedSelectionOptions, KeyframeAnimationOptions, Label, LabelProps, LabeledField, LabeledFieldProps, LabeledSeparator, LabeledSeparatorProps, type LayoutProps, Listbox, ListboxOption, ListboxProps, LoopOptions, Markdown, MarkdownProps, MarkdownVariant, Marker, MarkerContent, MarkerIcon, MarkerVariant, Menubar, MenubarMenu, MenubarMenuProps, MenubarProps, Message, MessageActions, MessageActionsProps, MessageAlign, MessageAnchorRect, MessageAvatar, MessageContent, MessageFooter, MessageGroup, MessageHeader, MessageProps, MessageScrollDirection, MessageScrollIntoViewOptions, MessageScroller, MessageScrollerButton, MessageScrollerButtonProps, MessageScrollerContent, MessageScrollerControls, MessageScrollerItem, MessageScrollerItemProps, MessageScrollerNavigator, MessageScrollerNavigatorItem, MessageScrollerNavigatorProps, MessageScrollerProps, MessageScrollerViewport, MessageScrollerViewportProps, Modal, type ModalMotionOptions, type ModalProps, type MotionConfig, MotionConfigProvider, type MotionConfigProviderProps, MotionInterpolationOptions, NativeLoopAnimation, NativeLoopAnimationOptions, NativeWidget, type NativeWidgetConfig, type NativeWidgetProps, NavigationMenu, NavigationMenuContent, NavigationMenuIndicator, NavigationMenuItem, NavigationMenuItemProps, NavigationMenuLink, NavigationMenuLinkProps, NavigationMenuList, NavigationMenuProps, NavigationMenuTrigger, NavigationMenuTriggerProps, NavigationMenuViewport, type NotificationMotionOptions, NotificationRegion, type NotificationRegionProps, type Notifications, NumberField, NumberFieldProps, Onboarding, OnboardingDescription, OnboardingFooter, OnboardingHeader, OnboardingHeading, OnboardingProps, OnboardingTitle, type OverlayLayer, OverlayPlaneProvider, type OwnedImageResource, PageHeader, PageHeaderProps, PageViewport, PageViewportProps, Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationItems, PaginationLink, PaginationLinkProps, PaginationNext, PaginationPrevious, PaginationProps, type PaginationRangeItem, PasswordInput, PasswordInputProps, Path, PathBuilder, type PathProps, Popover, PopoverDescription, PopoverFooter, PopoverHeader, PopoverProps, PopoverTitle, PopupMotionProps, type PressResult, Progress, ProgressCircle, ProgressCircleProps, ProgressFill, ProgressLabel, ProgressProps, ProgressRoot, ProgressRootProps, ProgressSize, ProgressTrack, ProgressTrackProps, ProgressValueDetails, ProgressValueLabel, ProjectionBoundary, PromptComposer, PromptComposerAction, PromptComposerEditor, PromptComposerEditorProps, PromptComposerProps, PromptComposerRowProps, PromptComposerStatus, PromptComposerToolbar, PromptComposerTools, PromptSuggestion, PromptSuggestionProps, PromptSuggestions, PromptSuggestionsProps, PropertyList, PropertyListProps, PropertyRow, PropertyRowProps, Pulse, PulseOptions, QRCode, QRCodeProps, QrCodeErrorCorrection, QrCodeMatrix, RadioGroup, RadioGroupItem, type RadioGroupItemProps, type RadioGroupProps, Rating, RatingProps, ReactiveAnimation, ReactiveTransition, Reasoning, ReasoningContent, ReasoningProps, ReasoningTrigger, ReasoningTriggerProps, RecordedShortcut, RepeatType, ResizableDirection, ResizableHandle, ResizableHandleProps, ResizablePanel, ResizablePanelDefinition, ResizablePanelGroup, ResizablePanelGroupProps, ResizablePanelSizes, ResizablePanelState, ResourceBoundary, ResourceBoundaryProps, ResponsiveGrid, ResponsiveGridColumnCount, ResponsiveGridProps, ResponsiveGridRemainder, ResponsiveGridState, type RetainedItem, type RetainedItems, RichText, type RichTextProps, RichTextSpan, type RichTextSpanProps, Ripple, type RippleProps, RotationAnimation, RotationOptions, RouteActiveOptions, RouterPersistenceOptions, RouterProvider, RouterProviderProps, Row, ScrollArea, type ScrollAreaProps, SearchField, SearchFieldProps, Select, SelectOption, SelectProps, Separator, SeparatorProps, SettingsGroup, SettingsGroupProps, SettingsItem, SettingsItemOrientation, SettingsItemProps, SettingsSection, SettingsSectionProps, Sheet, SheetProps, SheetSide, ShortcutRecorder, ShortcutRecorderProps, Sidebar, SidebarContent, SidebarContentProps, SidebarEmpty, SidebarFooter, SidebarGroup, SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuButtonProps, SidebarMenuIcon, SidebarMenuLabel, SidebarMenuProps, SidebarMenuSuffix, SidebarProps, SidebarSearch, SidebarSearchGroup, Skeleton, type SkeletonProps, Slider, SliderProps, Spin, Spinner, SplitButton, SplitButtonProps, SplitPane, SplitPaneAside, SplitPaneMain, StandardSchema, StandardSchemaIssue, StandardSchemaResult, StatCard, StatCardProps, StatusBar, StatusBarGroup, StatusBarGroupProps, StatusBarIndicator, StatusBarIndicatorProps, StatusBarIndicatorTone, StatusBarItem, StatusBarItemProps, StatusBarProps, StatusBarSeparator, Stepper, StepperProps, StepperStep, Svg, type SvgProps, SweepAnimation, SweepAxis, SweepGeometry, SweepOptions, Switch, type SwitchProps, Table, TableBody, TableCaption, TableCell, TableCellProps, TableFooter, TableHead, TableHeadProps, TableHeader, TableProps, TableRow, TableRowProps, Tabs, TabsContent, TabsItem, type TabsItemProps, type TabsItemState, TabsList, type TabsProps, type TabsResult, TabsTrigger, type TabsTriggerProps, TanStackDataTable, TanStackDataTableColumn, TanStackDataTableOptions, Text, TextArea, TextAreaProps, type TextProps, Timeline, TimelineItem, TimelineProps, TitleBar, TitleBarDragRegion, TitleBarDragRegionProps, TitleBarProps, ToastAction, ToastInput, ToastVariant, Toaster, ToasterProps, Toasts, Toggle, ToggleGroup, ToggleGroupItem, type ToggleGroupItemProps, type ToggleGroupProps, type ToggleProps, Tool, ToolCodeSection, ToolCodeSectionProps, ToolContent, ToolHeader, ToolHeaderProps, ToolInput, ToolInputProps, ToolOutput, ToolOutputProps, ToolProps, ToolStatus, Toolbar, ToolbarButton, ToolbarButtonProps, ToolbarGroup, ToolbarOrientation, ToolbarProps, ToolbarSeparator, ToolbarToggle, ToolbarToggleProps, Tooltip, TooltipContent, TooltipContentProps, TooltipProps, TooltipTriggerProps, TransitionOptions, type TransitionPresence, type TransitionPresenceOptions, TreeItemRenderState, TreeModel, TreeNode, TreeView, TreeViewProps, TypographyBlockquote, TypographyH1, TypographyH2, TypographyH3, TypographyH4, TypographyInlineCode, TypographyLarge, TypographyLead, TypographyList, TypographyListItem, TypographyMuted, TypographyP, TypographySmall, type VectorPath, type VectorPathPaint, View, type ViewProps, VisibleTreeNode, type WabouClassList, WabouDataRouter, WabouRouterConstructorOptions, type WabouStyle, WindowFrame, WindowFrameProps, Workbench, WorkbenchContent, WorkbenchContentColumn, WorkbenchFooter, WorkbenchHeader, WorkbenchInspector, WorkbenchInspectorContent, WorkbenchInspectorHeader, WorkbenchInspectorState, WorkbenchInspectorStateProps, WorkbenchInspectorTitlebar, WorkbenchInspectorTitlebarProps, WorkbenchMain, WorkbenchSidebar, activeMessageAnchor, alertColors, animate, animateKeyframes, aspectRatioStyle, attachmentClass, attachmentGroupClass, attachmentMediaClass, avatarInitials, badgeClass, bubbleClass, bubbleContentClass, clampAnnotationRegion, clampPage, clampRatingValue, componentsControlSize, componentsElevation, componentsThemeContract, createActive, createAnimationFrame, createButton, createCommandListNavigation, createContainerMatch, createDataRouter, createDelayedOpenController, createDelayedOpenController as createTooltipDelayController, createFileImageResource, createFocus, createFocusWithin, createFormDraft, createHover, createInterpolation, createKeyedSelection, createKeyframeAnimation, createLoop, createMeasuredSize, createMemoryHistory, createNativeLoopAnimation, createNetworkImageResource, createNotifications, createOverlayLayer, createOwnedImageResource, createPaginationRange, createPresence, createPress, createPulse, createResizablePanelState, createRetainedItems, createRotation, createScrollReset, createShortcuts, createStandardSchemaValidator, createSweep, createTabs, createTanStackDataTable, createToasts, createTransition, createTransitionPresence, createTreeModel, drawerDragOffset, drawerShouldDismiss, emptyClass, emptyMediaClass, encodeQrCode, fieldClass, fieldErrorLabel, filterCommandItems, filterSidebarGroups, groupBoxContentClass, imageViewportTransform, inputGroupAddonClass, inputGroupClass, isMessageScrollNearEnd, itemClass, itemMediaClass, messageActionsClass, messageClass, messageScrollRange, messageScrollRevealDelta, messageScrollStartDelta, moveMenuHighlight, navigationMenuTriggerClass, nextAccordionValue, normalizeCarouselIndex, normalizeOtpValue, normalizePageCount, normalizeProgressValue, normalizeRatingMax, normalizeSweepGeometry, notFound, pageHeaderClass, pageHeaderDescriptionClass, pageHeaderTitleClass, pageViewportClass, pageViewportContentClass, pointInLayoutRect, index_d_exports as primitives, promptComposerClass, promptComposerEditorHeightClass, qrCodePath, ratingLabel, reconcileCommandHighlight, redirect, releaseImageResource, responsiveGridColumnCount, responsiveGridRemainderCount, shortcutFromKeyEvent, titleBarClass, titleBarDragRegionLayoutStyle, titleBarLayoutStyle, toolHeaderLabel, uniqueFieldErrors, useChartConfig, useComponentsTheme, useDirection, useLoaderData, useLocation, useMessageScroller, useMotionConfig, useNavigate, useParams, useReducedMotion, useResponsiveGrid, useRouteActive, useRouter, useRouterState, validateResizableSizes, windowFrameBackdropClassList, windowFrameClientClassList, windowFrameMaterialStyle, windowFrameShadows, workbenchClass, workbenchContentClass, workbenchContentColumnClass, workbenchFooterClass, workbenchHeaderClass, workbenchInspectorClass, workbenchInspectorContentClass, workbenchInspectorHeaderClass, workbenchMainClass, workbenchSidebarClass };
//# sourceMappingURL=index.d.mts.map