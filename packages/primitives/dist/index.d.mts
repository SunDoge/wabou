import { Accessor, JSX } from "solid-js";
import { Handle, Host, LayoutRect as LayoutRect$1, LayoutTarget, NativeScrollbarStyle } from "@wabou/solid-renderer";
import { Affine2D, Affine2D as Affine2D$1, Shadow, WabouStyle, WabouStyle as WabouStyle$1, rotate2d, translate2d } from "@wabou/style";
import { Easing } from "@wabou/animation";
import { ComputePositionReturn, ComputePositionReturn as ComputePositionReturn$1, Middleware, Middleware as Middleware$1, Placement, Placement as Placement$1, Strategy, Strategy as Strategy$1, arrow, autoPlacement, flip, offset, shift, size } from "@floating-ui/core";
import { JSX as JSX$1 } from "@solidjs/web";
//#region src/animation-frame.d.ts
type AnimationFrameCallback = (timestamp: number) => unknown;
/**
 * Drive explicit paint state from the native host's animation clock.
 * Return `false` to stop scheduling frames before the owner is disposed.
 */
declare function createAnimationFrame(callback: AnimationFrameCallback): () => void;
//#endregion
//#region src/view.d.ts
type WabouClassList = Record<string, boolean | undefined>;
interface TextSelectionChangeEvent {
  type: "textselectionchange";
  text: string | null;
  kind: "simple" | "word" | "line" | null;
}
interface PrimitiveProps {
  class?: string;
  /** Explicit reactive classes; use this for primitive interaction state. */
  classList?: WabouClassList;
  style?: WabouStyle$1;
  /** Explicit runtime state, composed after the static CSS transform. */
  transform?: Affine2D$1 | null;
  /** Ordered Vello blurred-rounded-rectangle shadow layers. */
  shadows?: readonly Shadow[] | null;
  /** Native overlay scrollbar appearance and visibility policy. */
  scrollbar?: NativeScrollbarStyle | null;
  children?: JSX.Element;
  /** Native host node, useful for imperative primitives and measurement. */
  ref?: (node: Handle) => void;
  /** Fires once when a native text selection gesture commits or changes asynchronously. */
  onTextSelectionChange?: (event: TextSelectionChangeEvent) => void;
  [name: string]: unknown;
}
interface ViewProps extends PrimitiveProps {}
interface TextProps extends PrimitiveProps {}
interface SvgProps extends Omit<PrimitiveProps, "children"> {
  /** Trusted inline SVG source parsed and cached by the native host. */
  source: string;
}
interface IconProps extends Omit<SvgProps, "source"> {
  source: string;
  size?: number;
  /** Override Lucide's root `fill="none"`, for example with currentColor. */
  fill?: "none" | "currentColor";
  /** Accessible name. Omit for a decorative icon. */
  label?: string;
}
interface NetworkImageSource {
  kind: "network";
  url: string;
  format: "raster";
  /** Decoded pixels are shared by URL for the lifetime of this native runtime. */
  cache: "memory";
}
type ImageSource = NetworkImageSource;
interface ImageProps extends Omit<PrimitiveProps, "children"> {
  /** Low-level native source. Prefer a source-specific component. */
  source?: ImageSource;
}
interface NetworkImageProps extends Omit<ImageProps, "source"> {
  /** This component performs a host network request for the URL. */
  url: string;
  format: "raster";
  cache: "memory";
  /** Fired when the current URL is decoded and ready for native painting. */
  onResourceReady?: (event: {
    url: string;
  }) => void;
  /** Fired when the current URL fails to download or decode. */
  onResourceError?: (event: {
    url: string;
    error: string;
  }) => void;
}
interface TextAreaProps extends Omit<PrimitiveProps, "children"> {
  value?: string;
  placeholder?: string;
  disabled?: boolean;
  readOnly?: boolean;
  onInput?: (event: {
    currentTarget: {
      value: string;
    };
  }) => void;
}
interface PasswordInputProps extends Omit<PrimitiveProps, "children"> {
  /** Rust SecretStore slot. This is an identifier, never the secret value. */
  secret: string;
  placeholder?: string;
  disabled?: boolean;
  "aria-label"?: string;
  onKeyDown?: (event: {
    key: string;
    preventDefault(): void;
  }) => void;
}
interface CodeEditorProps extends Omit<PrimitiveProps, "children"> {
  value?: string;
  /** The initial experimental adapter supports JSON highlighting. */
  language?: "json";
  disabled?: boolean;
  readOnly?: boolean;
  "aria-label": string;
  onInput?: (event: {
    currentTarget: {
      value: string;
    };
  }) => void;
}
/** A layout container. Text content should be placed in a {@link Text}. */
declare function View(props: ViewProps): JSX.Element;
/**
 * A single measured text run.
 *
 * Static and reactive child text nodes are concatenated by the native host and
 * participate in the parent layout as one item.
 */
declare function Text(props: TextProps): JSX.Element;
/** A static SVG asset rendered through the native usvg/Vello pipeline. */
declare function Svg(props: SvgProps): JSX.Element;
/** A theme-colored SVG icon with stable native sizing and semantics. */
declare function Icon(props: IconProps): JSX.Element;
/** A replaced image node rendered by the native host. */
declare function Image(props: ImageProps): JSX.Element;
/** An explicit network-backed image with bounded decoding and host caching. */
declare function NetworkImage(props: NetworkImageProps): JSX.Element;
/** A native multiline text editor with wrapping, selection, and scrolling. */
declare function TextArea(props: TextAreaProps): JSX.Element;
/** Native password editor whose value remains in a Rust SecretStore. */
declare function PasswordInput(props: PasswordInputProps): JSX.Element;
/** Experimental native editor for config and script-sized documents. */
declare function CodeEditor(props: CodeEditorProps): JSX.Element;
//#endregion
//#region src/button.d.ts
declare const ACCENTS: {
  readonly neutral: "#475569";
  readonly sky: "#0284c7";
  readonly amber: "#d97706";
};
interface ButtonProps {
  class?: string | ((state: ButtonState) => string);
  classList?: WabouClassList | ((state: ButtonState) => WabouClassList);
  style?: WabouStyle | ((state: ButtonState) => WabouStyle);
  children?: JSX.Element;
  tone?: keyof typeof ACCENTS;
  variant?: "solid" | "ghost";
  /** Keep interaction behavior but do not inject the default visual geometry. */
  unstyled?: boolean;
  /** Allow selecting the label text. Button labels are non-selectable by default. */
  selectable?: boolean;
  selected?: boolean;
  disabled?: boolean;
  title?: string;
  role?: JSX$1.ButtonHTMLAttributes<HTMLButtonElement>["role"];
  ref?: (node: Handle) => void;
  "aria-haspopup"?: boolean | "false" | "true" | "menu" | "listbox" | "tree" | "grid" | "dialog";
  "aria-expanded"?: boolean;
  "aria-controls"?: string;
  "aria-label"?: string;
  "aria-checked"?: boolean | "mixed";
  "aria-selected"?: boolean;
  "aria-pressed"?: boolean;
  onKeyDown?: (event: ButtonKeyEvent) => void;
  onClick?: (event: ButtonEvent) => void;
  [name: string]: unknown;
}
interface ButtonEvent {
  stopPropagation(): void;
  preventDefault(): void;
}
interface ButtonKeyEvent extends ButtonEvent {
  key: string;
  repeat?: boolean;
  readonly defaultPrevented?: boolean;
}
interface ButtonState {
  hovered: boolean;
  pressed: boolean;
  focused: boolean;
  selected: boolean;
  disabled: boolean;
}
interface CreateButtonOptions {
  disabled?: Accessor<boolean> | boolean;
  selected?: Accessor<boolean> | boolean;
  onPress?: (event: ButtonEvent) => void;
  onKeyDown?: (event: ButtonKeyEvent) => void;
}
interface ButtonPrimitive {
  state: Accessor<ButtonState>;
  bindings: {
    onPointerEnter: () => void;
    onPointerLeave: () => void;
    onPointerDown: () => void;
    onPointerUp: () => void;
    onPointerCancel: () => void;
    onFocus: () => void;
    onBlur: () => void;
    onClick: (event: ButtonEvent) => void;
    onKeyDown: (event: ButtonKeyEvent) => void;
  };
}
/** Headless button state and event normalization. */
declare function createButton(options?: CreateButtonOptions): ButtonPrimitive;
/**
 * A native button with consistent hover, pressed, focus and disabled feedback.
 *
 * Interaction styling is deliberately implemented with reactive inline styles:
 * applications do not need CSS pseudo-class support to get a responsive button.
 */
declare function Button(props: ButtonProps): JSX.Element;
//#endregion
//#region src/focus.d.ts
interface FocusResult {
  focused: () => boolean;
  bindings: {
    onFocus: () => void;
    onBlur: () => void;
  };
}
/** Reactive focus state and event bindings for a single target. */
declare function createFocus(): FocusResult;
interface FocusWithinResult {
  focusWithin: () => boolean;
  bindings: {
    onFocusIn: () => void;
    onFocusOut: () => void;
  };
}
/** Reactive equivalent of `:focus-within`, using bubbling focus events. */
declare function createFocusWithin(): FocusWithinResult;
//#endregion
//#region src/hover.d.ts
interface HoverResult {
  hovered: () => boolean;
  bindings: {
    onPointerEnter: () => void;
    onPointerLeave: () => void;
  };
}
/** Reactive hover state and event bindings for a single target. */
declare function createHover(): HoverResult;
//#endregion
//#region src/layout.d.ts
interface LayoutProps extends ViewProps {
  class?: string;
  children?: JSX.Element;
}
/** Horizontal flex container. No wrapper node is added beyond the host View. */
declare function Row(props: LayoutProps): JSX.Element;
/** Vertical flex container. No wrapper node is added beyond the host View. */
declare function Column(props: LayoutProps): JSX.Element;
/** Flex container that centers children on both axes. */
declare function Center(props: LayoutProps): JSX.Element;
//#endregion
//#region src/collapsible-presence.d.ts
interface CollapsiblePresenceProps {
  open: boolean;
  children?: JSX.Element;
  class?: string;
  contentClass?: string;
  style?: WabouStyle;
  contentStyle?: WabouStyle;
  duration?: number;
  ease?: Easing;
  reducedMotion?: boolean;
  /** Animate an initially-open disclosure from zero height. Defaults to false. */
  animateInitial?: boolean;
}
/**
 * Measured disclosure content with explicit presence and subtree isolation.
 * Height participates in layout while a subtree opacity layer masks glyphs
 * crossing the moving clip edge.
 */
declare function CollapsiblePresence(props: CollapsiblePresenceProps): JSX.Element;
//#endregion
//#region src/measure.d.ts
interface MeasuredSize {
  ref(node: Handle): void;
  width: Accessor<number>;
  height: Accessor<number>;
  measured: Accessor<boolean>;
}
interface MeasuredSizeOptions {
  onChange?: (size: {
    width: number;
    height: number;
  }) => void;
}
/** Observe the completed native content-box size of a host node. */
declare function createMeasuredSize(options?: MeasuredSizeOptions): MeasuredSize;
//#endregion
//#region src/modal.d.ts
interface ModalEvent {
  stopPropagation(): void;
  preventDefault(): void;
}
interface ModalKeyEvent extends ModalEvent {
  key: string;
}
interface ModalTriggerProps {
  ref: (node: Handle) => void;
  onClick: (event: ModalEvent) => void;
  onKeyDown: (event: ModalKeyEvent) => void;
  "aria-haspopup": "dialog";
  "aria-expanded": boolean;
}
type ModalOpenChangeReason = "trigger" | "escape" | "backdrop" | "programmatic";
interface ModalControls {
  close(): void;
}
interface ModalProps {
  children?: JSX.Element | ((controls: ModalControls) => JSX.Element);
  trigger?: (props: ModalTriggerProps) => JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean, reason: ModalOpenChangeReason) => void;
  /** Required until the native semantic tree resolves aria-labelledby. */
  "aria-label": string;
  backdropClass?: string;
  backdropStyle?: WabouStyle;
  contentClass?: string;
  contentStyle?: WabouStyle;
  contentRef?: (node: Handle) => void;
  closeOnBackdrop?: boolean;
  closeOnEscape?: boolean;
  /** Overrides the host's default of focusing the first focusable descendant. */
  initialFocus?: () => Handle | undefined;
  restoreFocus?: boolean;
}
/**
 * A native modal plane with host-enforced focus, hit-test, and accessibility
 * isolation. Visual styling remains explicit so applications can own it.
 */
declare function Modal(props: ModalProps): JSX.Element;
//#endregion
//#region src/overlay-layer.d.ts
interface DismissEvent {
  preventDefault(): void;
  stopPropagation(): void;
}
interface DismissKeyEvent extends DismissEvent {
  key: string;
}
type OverlayDismissReason = "escape" | "outside";
interface OverlayLayerOptions {
  open: () => boolean;
  onDismiss: (reason: OverlayDismissReason) => void;
  closeOnEscape?: () => boolean;
  closeOnOutside?: () => boolean;
  restoreFocus?: () => boolean;
  returnFocus?: () => {
    focus(): void;
  } | undefined;
}
interface OverlayLayer {
  isTopmost(): boolean;
  onEscape(event: DismissKeyEvent): void;
  onOutside(event: DismissEvent): void;
}
declare function createOverlayLayer(options: OverlayLayerOptions): OverlayLayer;
//#endregion
//#region src/motion.d.ts
interface PlaybackProps {
  /** Animation duration in seconds. */
  duration?: number;
  /** Playback-rate multiplier. */
  speed?: number;
  paused?: boolean;
}
interface SpinProps extends Omit<ViewProps, "transform">, PlaybackProps {
  children?: JSX.Element;
}
/** A single native View whose contents rotate around its border-box center. */
declare function Spin(props: SpinProps): JSX.Element;
interface PulseProps extends ViewProps, PlaybackProps {
  from?: number;
  to?: number;
}
/** A single native View with a repeating opacity pulse. */
declare function Pulse(props: PulseProps): JSX.Element;
//#endregion
//#region src/presence.d.ts
type PresencePhase = "unmounted" | "entering" | "present" | "exiting";
interface Presence {
  phase: Accessor<PresencePhase>;
  mounted: Accessor<boolean>;
  finishEnter(): void;
  finishExit(): void;
}
/** Explicit mount lifecycle for content whose exit must finish before removal. */
declare function createPresence(open: Accessor<boolean>): Presence;
//#endregion
//#region src/notification.d.ts
type NotificationPriority = "polite" | "assertive";
type NotificationDismissReason = "dismiss" | "timeout" | "overflow" | "programmatic";
interface NotificationInput {
  /** Accessible announcement independent of the rendered visual content. */
  "aria-label": string;
  content: (controls: NotificationControls) => JSX.Element;
  priority?: NotificationPriority;
  /** Milliseconds before dismissal. Zero disables automatic dismissal. */
  duration?: number;
  onDismiss?: (reason: NotificationDismissReason) => void;
}
interface NotificationItem extends NotificationInput {
  id: number;
}
interface NotificationControls {
  dismiss(): void;
}
interface NotificationsOptions {
  defaultDuration?: number;
  limit?: number;
}
interface Notifications {
  readonly items: Accessor<readonly NotificationItem[]>;
  show(input: NotificationInput): number;
  dismiss(id: number, reason?: NotificationDismissReason): boolean;
  pause(id: number): void;
  resume(id: number): void;
  clear(): void;
}
/** Create an owner-scoped notification queue with explicit JavaScript timers. */
declare function createNotifications(options?: NotificationsOptions): Notifications;
type NotificationPlacement = "top-start" | "top" | "top-end" | "bottom-start" | "bottom" | "bottom-end";
interface NotificationRegionProps {
  notifications: Notifications;
  placement?: NotificationPlacement;
  class?: string;
  style?: WabouStyle;
  itemClass?: string;
  itemStyle?: WabouStyle;
}
/** Render a non-blocking stack on the native floating overlay plane. */
declare function NotificationRegion(props: NotificationRegionProps): JSX.Element;
//#endregion
//#region src/positioner.d.ts
type LayoutRect = LayoutRect$1;
interface PositionPlatform<T> {
  getRect(target: T): LayoutRect | Promise<LayoutRect>;
  getClippingRect(target: T): LayoutRect | Promise<LayoutRect>;
  isRTL?(target: T): boolean | Promise<boolean>;
}
interface ComputeFloatingPositionOptions<T> {
  platform: PositionPlatform<T>;
  placement?: Placement$1;
  strategy?: Strategy$1;
  middleware?: Array<Middleware$1 | null | undefined | false>;
}
/**
 * Position two Wabou layout targets with Floating UI's renderer-independent
 * geometry engine. Measurement remains host-owned and is supplied explicitly;
 * no DOM-compatible Handle methods are required.
 */
declare function computeFloatingPosition<T>(reference: T, floating: T, options: ComputeFloatingPositionOptions<T>): Promise<ComputePositionReturn$1>;
type ComputeHostFloatingPositionOptions = Omit<ComputeFloatingPositionOptions<LayoutTarget>, "platform">;
/** Position two native handles from a single coherent Host layout snapshot. */
declare function computeHostFloatingPosition(reference: LayoutTarget, floating: LayoutTarget, host: {
  readonly layout: Pick<Host["layout"], "snapshot">;
}, options?: ComputeHostFloatingPositionOptions): Promise<ComputePositionReturn$1>;
//#endregion
//#region src/popover.d.ts
interface PopoverTriggerProps {
  ref: (node: Handle) => void;
  onClick: (event: {
    stopPropagation(): void;
  }) => void;
  onKeyDown: (event: {
    key: string;
    preventDefault(): void;
    stopPropagation(): void;
  }) => void;
  "aria-haspopup": "dialog" | "listbox" | "menu" | "tree" | "grid";
  "aria-expanded": boolean;
}
interface PopoverBaseProps {
  trigger: (props: PopoverTriggerProps) => JSX.Element;
  children?: JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean, reason?: OverlayDismissReason | "trigger") => void;
  placement?: Placement;
  offset?: number;
  contentClass?: string;
  contentStyle?: WabouStyle;
  closeOnEscape?: boolean;
  restoreFocus?: boolean;
}
type PopoverProps = PopoverBaseProps & ({
  /** Required until the native semantic tree resolves aria-labelledby. */
  "aria-label": string;
  contentRole?: "dialog";
  popupRole?: never;
} | {
  /** Flatten the positioned shell when its child owns popup semantics. */
  contentRole: "presentation";
  popupRole: "listbox" | "menu" | "tree" | "grid";
  "aria-label"?: never;
});
/** A root-layer floating panel positioned from native layout snapshots. */
declare function Popover(props: PopoverProps): JSX.Element;
//#endregion
//#region src/press.d.ts
interface PressOptions {
  disabled?: Accessor<boolean> | boolean;
  onPress?: (event: unknown) => void;
}
interface PressResult {
  pressed: () => boolean;
  bindings: {
    onPointerDown: () => void;
    onPointerUp: () => void;
    onPointerCancel: () => void;
    onPointerLeave: () => void;
    onClick: (event: unknown) => void;
  };
}
/** Reactive pointer-press state with disabled-aware activation. */
declare function createPress(options?: PressOptions): PressResult;
interface ActiveResult {
  active: () => boolean;
  bindings: PressResult["bindings"];
}
/** CSS `:active`-like state without an activation callback. */
declare function createActive(disabled?: Accessor<boolean> | boolean): ActiveResult;
//#endregion
//#region src/scroll-area.d.ts
interface ScrollAreaProps {
  children?: JSX.Element;
  /**
   * Classes applied to the clipped scrolling viewport.
   *
   * The viewport has no implicit flex growth. Give it an explicit height or
   * use `flex-1 min-h-0` inside a bounded flex container.
   */
  class?: string;
  /** Classes applied to the intrinsic-height content wrapper. */
  contentClass?: string;
  style?: WabouStyle;
  ref?: (node: Handle) => void;
  scrollbar?: NativeScrollbarStyle;
  onScroll?: (event: {
    scrollX?: number;
    scrollY?: number;
  }) => void;
}
/**
 * Vertical native scroll viewport with explicit sizing.
 *
 * The inner wrapper deliberately cannot shrink. This makes its intrinsic
 * height become the viewport's scroll extent instead of allowing a flex
 * parent to compress overflowing sections until no scroll range remains.
 * The viewport itself deliberately does not grow: implicit `flex-1` makes a
 * nested scroll area expand with an ancestor's intrinsic content instead of
 * establishing its own scroll range.
 */
declare function ScrollArea(props: ScrollAreaProps): JSX.Element;
//#endregion
//#region src/scroll-reset.d.ts
interface ScrollResetTarget {
  scrollTo(options: {
    left?: number;
    top?: number;
  }): void;
}
interface ScrollResetOptions<K> {
  /** The explicitly owned viewport; no global/window fallback is used. */
  target: Accessor<ScrollResetTarget | undefined>;
  /** Reset whenever this navigation or content identity changes. */
  key: Accessor<K>;
  left?: number;
  top?: number;
}
/** Reset one explicitly selected native viewport after its key changes. */
declare function createScrollReset<K>(options: ScrollResetOptions<K>): () => void;
//#endregion
//#region src/shortcuts.d.ts
interface ShortcutEvent {
  key: string;
  /** Physical Shift, Control, Alt, and Meta modifier bits. */
  mods: number;
  /** Whether the physical modifiers form this platform's Primary chord. */
  primary: boolean;
  repeat?: boolean;
  preventDefault(): void;
}
type ShortcutHandler = (event: ShortcutEvent) => void;
interface ShortcutDefinition {
  handler: ShortcutHandler;
  /** Repeated keydown events are ignored by default. */
  allowRepeat?: boolean;
  /** Defaults to true so application shortcuts preempt focused widgets. */
  preventDefault?: boolean;
}
type ShortcutMap = Record<string, ShortcutHandler | ShortcutDefinition>;
interface ShortcutsResult {
  handleKeyDown: (event: ShortcutEvent) => boolean;
  bindings: {
    onKeyDown: (event: ShortcutEvent) => void;
  };
}
/**
 * Compile declarative application shortcuts into one keydown binding.
 *
 * Chords use names such as `Primary+T`, `Control+Tab`, and
 * `Control+Shift+Tab`. `Primary` resolves to Command on macOS and Control on
 * other platforms while still requiring an exact modifier match.
 */
declare function createShortcuts(shortcuts: ShortcutMap): ShortcutsResult;
//#endregion
//#region src/tabs.d.ts
type TabKey = string | number;
interface TabsOptions<T, K extends TabKey> {
  initialTabs?: readonly T[];
  key: (tab: T) => K;
  initialActiveKey?: K;
  onActiveChange?: (key: K | undefined) => void;
  orientation?: "horizontal" | "vertical";
}
interface AddTabOptions {
  activate?: boolean;
  index?: number;
}
/** Minimal native capability needed for keyboard focus movement. */
interface FocusTarget {
  focus(): void;
}
interface TabsResult<T, K extends TabKey> {
  tabs: Accessor<readonly T[]>;
  activeKey: Accessor<K | undefined>;
  activeTab: Accessor<T | undefined>;
  select: (key: K) => boolean;
  selectNext: () => boolean;
  selectPrevious: () => boolean;
  selectFirst: () => boolean;
  selectLast: () => boolean;
  add: (tab: T, options?: AddTabOptions) => boolean;
  close: (key: K) => boolean;
  move: (key: K, index: number) => boolean;
  register: (key: K, node: FocusTarget) => void;
  focus: (key: K) => boolean;
  handleKeyDown: (key: K, event: TabKeyEvent) => boolean;
}
interface TabKeyEvent {
  key: string;
  preventDefault?: () => void;
}
/**
 * Stateful tab collection with stable identity and deterministic activation.
 *
 * Closing the active tab selects its right-hand neighbour, or the previous
 * tab when the closed tab was last. Reordering never changes the active key.
 */
declare function createTabs<T, K extends TabKey>(options: TabsOptions<T, K>): TabsResult<T, K>;
//#endregion
export { type ActiveResult, type AddTabOptions, type Affine2D, type AnimationFrameCallback, Button, type ButtonEvent, type ButtonKeyEvent, type ButtonPrimitive, type ButtonProps, type ButtonState, Center, CodeEditor, type CodeEditorProps, CollapsiblePresence, type CollapsiblePresenceProps, Column, type ComputeFloatingPositionOptions, type ComputeHostFloatingPositionOptions, type ComputePositionReturn, type CreateButtonOptions, type DismissEvent, type DismissKeyEvent, type FocusResult, type FocusTarget, type FocusWithinResult, type HoverResult, Icon, type IconProps, Image, type ImageProps, type ImageSource, type LayoutProps, type LayoutRect, type MeasuredSize, type MeasuredSizeOptions, type Middleware, Modal, type ModalControls, type ModalEvent, type ModalKeyEvent, type ModalOpenChangeReason, type ModalProps, type ModalTriggerProps, NetworkImage, type NetworkImageProps, type NetworkImageSource, type NotificationControls, type NotificationDismissReason, type NotificationInput, type NotificationItem, type NotificationPlacement, type NotificationPriority, NotificationRegion, type NotificationRegionProps, type Notifications, type NotificationsOptions, type OverlayDismissReason, type OverlayLayer, type OverlayLayerOptions, PasswordInput, type PasswordInputProps, type Placement, Popover, type PopoverProps, type PopoverTriggerProps, type PositionPlatform, type Presence, type PresencePhase, type PressOptions, type PressResult, type PrimitiveProps, Pulse, type PulseProps, Row, ScrollArea, type ScrollAreaProps, type ScrollResetOptions, type ScrollResetTarget, type ShortcutDefinition, type ShortcutEvent, type ShortcutHandler, type ShortcutMap, type ShortcutsResult, Spin, type SpinProps, type Strategy, Svg, type SvgProps, type TabKey, type TabKeyEvent, type TabsOptions, type TabsResult, Text, TextArea, type TextAreaProps, type TextProps, View, type ViewProps, type WabouClassList, type WabouStyle, arrow, autoPlacement, computeFloatingPosition, computeHostFloatingPosition, createActive, createAnimationFrame, createButton, createFocus, createFocusWithin, createHover, createMeasuredSize, createNotifications, createOverlayLayer, createPresence, createPress, createScrollReset, createShortcuts, createTabs, flip, offset, rotate2d, shift, size, translate2d };
//# sourceMappingURL=index.d.mts.map