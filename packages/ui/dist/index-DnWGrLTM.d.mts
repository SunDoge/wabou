import { PathBuilder, VectorPath, VectorPath as VectorPath$1, VectorPathPaint } from "@wabou/core";
import { Affine2D, Affine2D as Affine2D$1, Shadow, WabouStyle, WabouStyle as WabouStyle$1, rotate2d as rotate2d$1, translate2d as translate2d$1 } from "@wabou/core/style";
import { Accessor, JSX } from "solid-js";
import { Handle, Host, LayoutRect, LayoutTarget, NativeScrollbarStyle, WabouElementProps } from "@wabou/core/renderer";
import { ComputePositionReturn, ComputePositionReturn as ComputePositionReturn$1, Middleware, Middleware as Middleware$1, Placement, Placement as Placement$1, Strategy, Strategy as Strategy$1, arrow, autoPlacement, flip, offset, shift, size } from "@floating-ui/core";
//#region src/animation/config.d.ts
interface MotionConfig {
  reducedMotion: Accessor<boolean>;
}
interface MotionConfigProviderProps {
  /** Disable non-essential interpolation while preserving final UI state. */
  reducedMotion?: boolean;
  children?: JSX.Element;
}
/** Application-level motion policy inherited by all styled Wabou components. */
declare function MotionConfigProvider(props: MotionConfigProviderProps): JSX.Element;
declare function useMotionConfig(): MotionConfig;
declare function useReducedMotion(): Accessor<boolean>;
//#endregion
//#region src/animation/index.d.ts
type AnimationValue = number | string;
type AnimationType = "tween" | "spring" | false;
type RepeatType = "loop" | "reverse" | "mirror";
type EasingFunction = (progress: number) => number;
type Easing = "linear" | "easeIn" | "easeOut" | "easeInOut" | "circIn" | "circOut" | "circInOut" | "backIn" | "backOut" | "backInOut" | "anticipate" | EasingFunction | readonly [number, number, number, number];
/** Wabou animation options. All time values are expressed in seconds. */
interface AnimationOptions<V extends AnimationValue = number> {
  type?: AnimationType;
  duration?: number;
  visualDuration?: number;
  delay?: number;
  ease?: Easing | Easing[];
  times?: number[];
  repeat?: number;
  repeatType?: RepeatType;
  repeatDelay?: number;
  autoplay?: boolean;
  stiffness?: number;
  damping?: number;
  mass?: number;
  bounce?: number;
  velocity?: number;
  restSpeed?: number;
  restDelta?: number;
  onUpdate?: (value: V) => void;
  onPlay?: () => void;
  onComplete?: () => void;
  onRepeat?: () => void;
  onStop?: () => void;
}
type AnimationState = "idle" | "running" | "paused" | "finished";
/** Backend-independent playback handle returned by Wabou animations. */
interface AnimationControls extends PromiseLike<void> {
  time: number;
  speed: number;
  readonly duration: number;
  readonly state: AnimationState;
  readonly finished: Promise<void>;
  play(): void;
  pause(): void;
  stop(): void;
  cancel(): void;
  complete(): void;
}
/** Animate between two scalar values. */
declare function animate<V extends AnimationValue>(from: V, to: V, options?: AnimationOptions<V>): AnimationControls;
/** Animate through two or more scalar keyframes. */
declare function animateKeyframes<V extends AnimationValue>(keyframes: readonly [V, V, ...V[]], options?: AnimationOptions<V>): AnimationControls;
interface ReactiveAnimation<T> {
  value: Accessor<T>;
  controls: AnimationControls;
}
type MaybeAccessor<T> = T | Accessor<T>;
interface TransitionOptions extends Omit<AnimationOptions<number>, "autoplay" | "onUpdate" | "onComplete"> {
  /** Skip interpolation while the user's/application's reduced-motion policy is active. */
  reducedMotion?: MaybeAccessor<boolean>;
  onUpdate?: (value: number) => void;
  onComplete?: (value: number) => void;
}
interface ReactiveTransition {
  value: Accessor<number>;
  state: Accessor<AnimationState>;
  /** Cancel the current run and synchronously move to a value. */
  jump(value: number): void;
  stop(): void;
}
/**
 * Lifecycle-owned scalar transition that retargets from its current value.
 *
 * Unlike a one-shot animation, changing `target` while a run is active does
 * not restart from the previous keyframe. This makes it suitable for rapidly
 * toggled disclosure, hover and selection state.
 */
declare function createTransition(target: Accessor<number>, options?: TransitionOptions): ReactiveTransition;
interface RepeatingOptions extends Omit<AnimationOptions<number>, "onUpdate"> {
  /** Reactive policy which pauses the loop and publishes `reducedValue`. */
  reducedMotion?: MaybeAccessor<boolean>;
  /** Stable value exposed while motion is reduced. */
  reducedValue?: number;
  onUpdate?: (value: number) => void;
}
interface LoopOptions extends RepeatingOptions {
  from?: number;
  to?: number;
}
/**
 * Lifecycle-owned repeating scalar animation for Solid components.
 *
 * The controls stop automatically with the current Solid owner.
 */
declare function createLoop(options?: LoopOptions): ReactiveAnimation<number>;
interface RotationOptions extends Omit<LoopOptions, "from" | "to"> {
  /** Initial angle in radians. Defaults to zero. */
  from?: number;
  /** Final angle in radians. Defaults to one full turn. */
  to?: number;
}
interface RotationAnimation extends ReactiveAnimation<number> {
  angle: Accessor<number>;
  transform: Accessor<Affine2D>;
}
type SweepAxis = "horizontal" | "vertical";
interface SweepGeometry {
  extent: number;
  itemRatio: number;
}
declare function normalizeSweepGeometry(extent: number, itemRatio: number): SweepGeometry;
interface SweepOptions extends LoopOptions {
  /** Current container width or height in logical pixels. */
  extent: MaybeAccessor<number>;
  /** Moving item's size as a fraction of the container. Defaults to 0.4. */
  itemRatio?: MaybeAccessor<number>;
  axis?: SweepAxis;
}
interface SweepAnimation extends ReactiveAnimation<number> {
  offset: Accessor<number>;
  transform: Accessor<Affine2D>;
}
/**
 * Move an item completely across one measured axis using only a runtime
 * transform. Both repeat boundaries remain outside the container, avoiding a
 * visible reset and avoiding per-frame layout invalidation.
 */
declare function createSweep(options: SweepOptions): SweepAnimation;
/** Repeating center-pivoted rotation backed by Motion value animation. */
declare function createRotation(options?: RotationOptions): RotationAnimation;
interface PulseOptions extends RepeatingOptions {
  from?: number;
  to?: number;
}
/** Repeating from→to→from value animation with automatic cleanup. */
declare function createPulse(options?: PulseOptions): ReactiveAnimation<number>;
//#endregion
//#region src/primitives/animation-frame.d.ts
type AnimationFrameCallback = (timestamp: number) => unknown;
/**
 * Drive explicit paint state from the native host's animation clock.
 * Return `false` to stop scheduling frames before the owner is disposed.
 */
declare function createAnimationFrame(callback: AnimationFrameCallback): () => void;
//#endregion
//#region src/primitives/view.d.ts
type WabouClassList = Record<string, boolean | undefined>;
interface TextSelectionChangeEvent {
  type: "textselectionchange";
  text: string | null;
  kind: "simple" | "word" | "line" | null;
}
interface PrimitiveProps extends Omit<WabouElementProps, "children" | "ref" | "style"> {
  /** Explicit reactive classes; use this for primitive interaction state. */
  classList?: WabouClassList;
  style?: WabouStyle;
  /** Explicit runtime state, composed after the static CSS transform. */
  transform?: Affine2D | null;
  /** Ordered Vello blurred-rounded-rectangle shadow layers. */
  shadows?: readonly Shadow[] | null;
  /** Native overlay scrollbar appearance and visibility policy. */
  scrollbar?: NativeScrollbarStyle | null;
  children?: JSX.Element;
  /** Native host node, useful for imperative primitives and measurement. */
  ref?: (node: Handle) => void;
  /** Fires once when a native text selection gesture commits or changes asynchronously. */
  onTextSelectionChange?: (event: TextSelectionChangeEvent) => void;
}
interface ViewProps extends PrimitiveProps {}
interface TextProps extends PrimitiveProps {
  /** Maximum rendered lines. Overflow on the final line is replaced by an ellipsis. */
  maxLines?: number;
}
interface SvgProps extends Omit<PrimitiveProps, "children"> {
  /** Trusted inline SVG source parsed and cached by the native host. */
  source: string;
}
interface PathProps extends Omit<PrimitiveProps, "children"> {
  /** Immutable geometry and paint snapshot built with PathBuilder. */
  source: VectorPath;
}
interface IconProps extends Omit<SvgProps, "source"> {
  source: string;
  /**
   * Icon size in logical px or a CSS length (`px`, `rem`, `em`, `%`).
   * When omitted or blank, defaults to `1em` (16px root font size).
   */
  size?: number | string;
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
interface TextInputProps extends Omit<PrimitiveProps, "children"> {
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
interface TextAreaProps extends TextInputProps {}
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
/** A native Vello vector path in local logical-pixel coordinates. */
declare function Path(props: PathProps): JSX.Element;
/** A theme-colored SVG icon with stable native sizing and semantics. */
declare function Icon(props: IconProps): JSX.Element;
/** A replaced image node rendered by the native host. */
declare function Image(props: ImageProps): JSX.Element;
/** An explicit network-backed image with bounded decoding and host caching. */
declare function NetworkImage(props: NetworkImageProps): JSX.Element;
/** A native single-line text editor with selection and scrolling. */
declare function TextInput(props: TextInputProps): JSX.Element;
/** A native multiline text editor with wrapping, selection, and scrolling. */
declare function TextArea(props: TextAreaProps): JSX.Element;
/** Native password editor whose value remains in a Rust SecretStore. */
declare function PasswordInput(props: PasswordInputProps): JSX.Element;
/** Experimental native editor for config and script-sized documents. */
declare function CodeEditor(props: CodeEditorProps): JSX.Element;
//#endregion
//#region src/primitives/button.d.ts
declare const ACCENTS: {
  readonly neutral: "#475569";
  readonly sky: "#0284c7";
  readonly amber: "#d97706";
};
interface ButtonProps extends Pick<WabouElementProps, "aria-checked" | "aria-controls" | "aria-current" | "aria-expanded" | "aria-haspopup" | "aria-label" | "aria-pressed" | "aria-selected" | "role" | "focusOrder" | "onBlur" | "onContextMenu" | "onDblClick" | "onFocus" | "onFocusIn" | "onFocusOut" | "onKeyUp" | "onPointerCancel" | "onPointerDown" | "onPointerEnter" | "onPointerLeave" | "onPointerMove" | "onPointerOut" | "onPointerOver" | "onPointerUp" | "onWheel"> {
  class?: string | ((state: ButtonState) => string);
  classList?: WabouClassList | ((state: ButtonState) => WabouClassList);
  style?: WabouStyle$1 | ((state: ButtonState) => WabouStyle$1);
  children?: JSX.Element;
  tone?: keyof typeof ACCENTS;
  variant?: "solid" | "ghost";
  /** Keep interaction behavior but do not inject the default visual geometry. */
  unstyled?: boolean;
  /** Allow selecting the label text. Button labels are non-selectable by default. */
  selectable?: boolean;
  selected?: boolean;
  disabled?: boolean;
  ref?: (node: Handle) => void;
  onKeyDown?: (event: ButtonKeyEvent) => void;
  onClick?: (event: ButtonEvent) => void;
}
interface LinkProps extends ButtonProps {
  /** URL passed explicitly to the native shell when the link is activated. */
  url: string;
  role?: never;
}
interface ButtonEvent {
  readonly defaultPrevented?: boolean;
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
  /** Keyboard-visible focus, separate from focus retained after a click. */
  focusVisible: boolean;
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
    onFocus: (event?: {
      payload?: {
        focusVisible?: boolean;
      };
    }) => void;
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
/**
 * An explicit external-link interaction.
 *
 * Wabou does not assign browser behavior to an `a` tag or `href` attribute;
 * the JS primitive owns activation while Rust only executes `openUrl`.
 */
declare function Link(props: LinkProps): JSX.Element;
//#endregion
//#region src/primitives/collapsible-presence.d.ts
interface CollapsiblePresenceProps {
  open: boolean;
  children?: JSX.Element;
  class?: string;
  contentClass?: string;
  style?: WabouStyle$1;
  contentStyle?: WabouStyle$1;
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
//#region src/primitives/focus.d.ts
interface FocusResult {
  focused: () => boolean;
  focusVisible: () => boolean;
  /** Record that the next/current focus came from direct pointer input. */
  pointerModality: () => void;
  /** Record that the next focus movement came from keyboard input. */
  keyboardModality: () => void;
  bindings: {
    onFocus: (event?: FocusEvent) => void;
    onBlur: () => void;
  };
}
interface FocusEvent {
  /** Native input-modality hint. Styling remains owned by the JS primitive. */
  payload?: {
    focusVisible?: boolean;
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
//#region src/primitives/hover.d.ts
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
//#region src/primitives/interactions/form-draft.d.ts
type FormDraftFieldUpdater<Value> = Value | ((previous: Value) => Value);
type FormDraftErrors<T> = Partial<Record<keyof T, string>>;
interface FormDraft<T extends Record<PropertyKey, unknown>> {
  value: Accessor<Readonly<T>>;
  dirty: Accessor<boolean>;
  /** Validation errors derived from the current immutable draft. */
  errors: Accessor<Readonly<FormDraftErrors<T>>>;
  valid: Accessor<boolean>;
  fieldError<Key extends keyof T>(key: Key): string | undefined;
  field<Key extends keyof T>(key: Key): T[Key];
  control<Key extends keyof T>(key: Key): readonly [Accessor<T[Key]>, (value: FormDraftFieldUpdater<T[Key]>) => void];
  set<Key extends keyof T>(key: Key, value: FormDraftFieldUpdater<T[Key]>): void;
  patch(value: Partial<T>): void;
  /** Restore the last baseline. */
  reset(): void;
  /** Replace both the baseline and current value. */
  resetTo(value: T): void;
  /** Make the current value the new baseline. */
  commit(): void;
}
interface FormDraftOptions<T> {
  equals?: (left: Readonly<T>, right: Readonly<T>) => boolean;
  validate?: (value: Readonly<T>) => FormDraftErrors<T>;
}
/**
 * A small immutable draft for form fields with explicit reset and commit
 * semantics. Transient request/error state belongs outside this model.
 */
declare function createFormDraft<T extends Record<PropertyKey, unknown>>(initial: T, options?: FormDraftOptions<T>): FormDraft<T>;
//#endregion
//#region src/primitives/interactions/selection.d.ts
type SelectionMode = "single" | "multiple";
interface KeyedSelectionOptions<T, Key> {
  items: Accessor<readonly T[]>;
  key: (item: T) => Key;
  mode: SelectionMode;
  initialKeys?: Iterable<Key>;
}
interface KeyedSelection<T, Key> {
  keys: Accessor<ReadonlySet<Key>>;
  items: Accessor<readonly T[]>;
  item: Accessor<T | undefined>;
  isSelected(key: Key): boolean;
  select(key: Key): void;
  deselect(key: Key): void;
  toggle(key: Key): void;
  set(keys: Iterable<Key>): void;
  clear(): void;
}
/**
 * Selection state owned by stable keys while values remain host-owned.
 * Selected items always resolve to the latest objects from `items`; keys that
 * disappear from the source are removed instead of becoming ghost selections.
 */
declare function createKeyedSelection<T, Key>(options: KeyedSelectionOptions<T, Key>): KeyedSelection<T, Key>;
//#endregion
//#region src/primitives/layout.d.ts
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
//#region src/primitives/measure.d.ts
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
/** Inclusive logical-pixel constraints evaluated against a host node's content box. */
interface ContainerSizeQuery {
  minWidth?: number;
  maxWidth?: number;
  minHeight?: number;
  maxHeight?: number;
}
interface ContainerMatch extends MeasuredSize {
  matches: Accessor<boolean>;
}
/** Observe the completed native content-box size of a host node. */
declare function createMeasuredSize(options?: MeasuredSizeOptions): MeasuredSize;
/**
 * Match constraints against a component's completed native content-box size.
 * The result remains false until the first measurement, avoiding a compact
 * layout flash during boot.
 */
declare function createContainerMatch(query: ContainerSizeQuery, options?: MeasuredSizeOptions): ContainerMatch;
//#endregion
//#region src/primitives/modal.d.ts
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
  /** Use `alertdialog` for an interruption that requires an explicit choice. */
  contentRole?: "dialog" | "alertdialog";
  backdropClass?: string;
  backdropStyle?: WabouStyle$1;
  contentClass?: string;
  contentStyle?: WabouStyle$1;
  contentShadows?: readonly Shadow[] | null;
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
//#region src/primitives/motion.d.ts
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
interface RippleProps extends ViewProps, PlaybackProps {
  /** Scale at the beginning of each ripple. Defaults to 0.35. */
  fromScale?: number;
}
/** A center-originating ring that expands while fading out, then repeats. */
declare function Ripple(props: RippleProps): JSX.Element;
//#endregion
//#region src/primitives/notification.d.ts
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
  style?: WabouStyle$1;
  itemClass?: string;
  itemStyle?: WabouStyle$1;
}
/** Render a non-blocking stack on the native floating overlay plane. */
declare function NotificationRegion(props: NotificationRegionProps): JSX.Element;
//#endregion
//#region src/primitives/overlay-layer.d.ts
type OverlayPlane = "floating" | "modal";
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
  /** Must match the native Portal plane so dismissal follows paint order. */
  plane?: () => OverlayPlane;
  onDismiss: (reason: OverlayDismissReason) => void;
  closeOnEscape?: () => boolean;
  closeOnOutside?: () => boolean;
  restoreFocus?: () => boolean;
  returnFocus?: () => {
    focus(): void;
  } | undefined;
}
interface OverlayLayer {
  plane(): OverlayPlane;
  /** Stable native sibling order for the current open lifetime. */
  zIndex(): number;
  isTopmost(): boolean;
  onEscape(event: DismissKeyEvent): void;
  onOutside(event: DismissEvent): void;
}
interface OverlayPlaneProviderProps {
  plane: OverlayPlane;
  children?: JSX.Element;
}
/** Make nested portals inherit the current native stacking plane. */
declare function OverlayPlaneProvider(props: OverlayPlaneProviderProps): JSX.Element;
declare function useOverlayPlane(): OverlayPlane;
declare function createOverlayLayer(options: OverlayLayerOptions): OverlayLayer;
//#endregion
//#region src/primitives/positioner.d.ts
type LayoutRect$1 = LayoutRect;
interface PointAnchor {
  x: number;
  y: number;
}
interface PositionPlatform<T> {
  getRect(target: T): LayoutRect$1 | Promise<LayoutRect$1>;
  getClippingRect(target: T): LayoutRect$1 | Promise<LayoutRect$1>;
  isRTL?(target: T): boolean | Promise<boolean>;
}
interface ComputeFloatingPositionOptions<T> {
  platform: PositionPlatform<T>;
  placement?: Placement;
  strategy?: Strategy;
  middleware?: Array<Middleware | null | undefined | false>;
}
/**
 * Position two Wabou layout targets with Floating UI's renderer-independent
 * geometry engine. Measurement remains host-owned and is supplied explicitly;
 * no DOM-compatible Handle methods are required.
 */
declare function computeFloatingPosition<T>(reference: T, floating: T, options: ComputeFloatingPositionOptions<T>): Promise<ComputePositionReturn>;
type ComputeHostFloatingPositionOptions = Omit<ComputeFloatingPositionOptions<LayoutTarget>, "platform">;
/** Position two native handles from a single coherent Host layout snapshot. */
declare function computeHostFloatingPosition(reference: LayoutTarget, floating: LayoutTarget, host: {
  readonly layout: Pick<Host["layout"], "snapshot">;
}, options?: ComputeHostFloatingPositionOptions): Promise<ComputePositionReturn>;
//#endregion
//#region src/primitives/popover.d.ts
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
  "aria-haspopup"?: "dialog" | "listbox" | "menu" | "tree" | "grid";
  "aria-expanded": boolean;
}
interface PopoverBaseProps {
  trigger: (props: PopoverTriggerProps) => JSX.Element;
  children?: JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean, reason?: OverlayDismissReason | "trigger") => void;
  placement?: Placement$1;
  offset?: number;
  /** Positions from a viewport point instead of the trigger's layout box. */
  anchorPoint?: () => PointAnchor | undefined;
  contentClass?: string;
  contentStyle?: WabouStyle$1;
  contentShadows?: readonly Shadow[] | null;
  /** Keeps composed hover/focus surfaces open while the pointer is inside. */
  onContentPointerEnter?: ViewProps["onPointerEnter"];
  onContentPointerLeave?: ViewProps["onPointerLeave"];
  onContentFocusIn?: ViewProps["onFocusIn"];
  onContentFocusOut?: ViewProps["onFocusOut"];
  closeOnEscape?: boolean;
  restoreFocus?: boolean;
  /**
   * `passthrough` dismisses from global pointer capture while allowing the
   * underlying target to receive the same gesture. Modal-like surfaces keep
   * the default full-viewport backdrop.
   */
  outsidePointerStrategy?: "backdrop" | "passthrough";
  /** Defaults to the nearest overlay plane, or `floating` at app content. */
  plane?: OverlayPlane;
}
type PopoverProps = PopoverBaseProps & ({
  /** Required until the native semantic tree resolves aria-labelledby. */
  "aria-label": string;
  contentRole?: "dialog";
  popupRole?: never;
} | {
  /** Flatten the positioned shell when its child owns popup semantics. */
  contentRole: "presentation";
  popupRole: "listbox" | "menu" | "tree" | "grid" | "tooltip";
  "aria-label"?: never;
});
/** A root-layer floating panel positioned from native layout snapshots. */
declare function Popover(props: PopoverProps): JSX.Element;
//#endregion
//#region src/primitives/presence.d.ts
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
//#region src/primitives/press.d.ts
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
//#region src/primitives/scroll-area.d.ts
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
  style?: WabouStyle$1;
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
 * The viewport also locks its cross axis. Otherwise focus reveal can move a
 * nominally vertical viewport sideways when a descendant is slightly wider,
 * making split-pane edges appear clipped. It deliberately does not grow:
 * implicit `flex-1` makes a
 * nested scroll area expand with an ancestor's intrinsic content instead of
 * establishing its own scroll range.
 */
declare function ScrollArea(props: ScrollAreaProps): JSX.Element;
//#endregion
//#region src/primitives/scroll-reset.d.ts
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
//#region src/primitives/shortcuts.d.ts
interface ShortcutEvent {
  key: string;
  /** Physical Shift, Control, Alt, and Meta modifier bits. */
  mods: number;
  /** Whether the physical modifiers form this platform's Primary chord. */
  primary: boolean;
  repeat?: boolean;
  preventDefault(): void;
}
type ShortcutHandler = (event: ShortcutEvent) => unknown;
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
    onKeyDown: (event: ShortcutEvent) => unknown;
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
//#region src/primitives/tabs.d.ts
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
declare namespace index_d_exports {
  export { ActiveResult, AddTabOptions, Affine2D$1 as Affine2D, AnimationFrameCallback, Button, ButtonEvent, ButtonKeyEvent, ButtonPrimitive, ButtonProps, ButtonState, Center, CodeEditor, CodeEditorProps, CollapsiblePresence, CollapsiblePresenceProps, Column, ComputeFloatingPositionOptions, ComputeHostFloatingPositionOptions, ComputePositionReturn$1 as ComputePositionReturn, ContainerMatch, ContainerSizeQuery, CreateButtonOptions, DismissEvent, DismissKeyEvent, FocusResult, FocusTarget, FocusWithinResult, FormDraft, FormDraftErrors, FormDraftFieldUpdater, FormDraftOptions, HoverResult, Icon, IconProps, Image, ImageProps, ImageSource, KeyedSelection, KeyedSelectionOptions, LayoutProps, LayoutRect$1 as LayoutRect, Link, LinkProps, MeasuredSize, MeasuredSizeOptions, Middleware$1 as Middleware, Modal, ModalControls, ModalEvent, ModalKeyEvent, ModalOpenChangeReason, ModalProps, ModalTriggerProps, NetworkImage, NetworkImageProps, NetworkImageSource, NotificationControls, NotificationDismissReason, NotificationInput, NotificationItem, NotificationPlacement, NotificationPriority, NotificationRegion, NotificationRegionProps, Notifications, NotificationsOptions, OverlayDismissReason, OverlayLayer, OverlayLayerOptions, OverlayPlane, OverlayPlaneProvider, OverlayPlaneProviderProps, PasswordInput, PasswordInputProps, Path, PathBuilder, PathProps, Placement$1 as Placement, Popover, PopoverProps, PopoverTriggerProps, PositionPlatform, Presence, PresencePhase, PressOptions, PressResult, PrimitiveProps, Pulse, PulseProps, Ripple, RippleProps, Row, ScrollArea, ScrollAreaProps, ScrollResetOptions, ScrollResetTarget, ShortcutDefinition, ShortcutEvent, ShortcutHandler, ShortcutMap, ShortcutsResult, Spin, SpinProps, Strategy$1 as Strategy, Svg, SvgProps, TabKey, TabKeyEvent, TabsOptions, TabsResult, Text, TextArea, TextAreaProps, TextInput, TextInputProps, TextProps, VectorPath$1 as VectorPath, VectorPathPaint, View, ViewProps, WabouClassList, WabouStyle$1 as WabouStyle, arrow, autoPlacement, computeFloatingPosition, computeHostFloatingPosition, createActive, createAnimationFrame, createButton, createContainerMatch, createFocus, createFocusWithin, createFormDraft, createHover, createKeyedSelection, createMeasuredSize, createNotifications, createOverlayLayer, createPresence, createPress, createScrollReset, createShortcuts, createTabs, flip, offset, rotate2d$1 as rotate2d, shift, size, translate2d$1 as translate2d, useOverlayPlane };
}
//#endregion
export { OverlayPlane as $, PulseOptions as $n, CollapsiblePresenceProps as $t, PopoverTriggerProps as A, TextArea as An, MeasuredSize as At, arrow as B, WabouStyle$1 as Bn, createKeyedSelection as Bt, createActive as C, Path as Cn, ModalEvent as Ct, createPresence as D, Svg as Dn, ModalTriggerProps as Dt, PresencePhase as E, PrimitiveProps as En, ModalProps as Et, Middleware$1 as F, VectorPath$1 as Fn, Column as Ft, offset as G, AnimationControls as Gn, createFormDraft as Gt, computeFloatingPosition as H, translate2d$1 as Hn, FormDraftErrors as Ht, Placement$1 as I, VectorPathPaint as In, LayoutProps as It, DismissEvent as J, AnimationType as Jn, FocusResult as Jt, shift as K, AnimationOptions as Kn, HoverResult as Kt, PointAnchor as L, View as Ln, Row as Lt, ComputeHostFloatingPositionOptions as M, TextInput as Mn, createContainerMatch as Mt, ComputePositionReturn$1 as N, TextInputProps as Nn, createMeasuredSize as Nt, Popover as O, SvgProps as On, ContainerMatch as Ot, LayoutRect$1 as P, TextProps as Pn, Center as Pt, OverlayLayerOptions as Q, LoopOptions as Qn, CollapsiblePresence as Qt, PositionPlatform as R, ViewProps as Rn, KeyedSelection as Rt, PressResult as S, PasswordInputProps as Sn, useReducedMotion as Sr, ModalControls as St, Presence as T, PathProps as Tn, ModalOpenChangeReason as Tt, computeHostFloatingPosition as U, AnimationFrameCallback as Un, FormDraftFieldUpdater as Ut, autoPlacement as V, rotate2d$1 as Vn, FormDraft as Vt, flip as W, createAnimationFrame as Wn, FormDraftOptions as Wt, OverlayDismissReason as X, Easing as Xn, createFocus as Xt, DismissKeyEvent as Y, AnimationValue as Yn, FocusWithinResult as Yt, OverlayLayer as Z, EasingFunction as Zn, createFocusWithin as Zt, createScrollReset as _, ImageSource as _n, normalizeSweepGeometry as _r, Ripple as _t, TabKeyEvent as a, ButtonState as an, SweepAnimation as ar, NotificationDismissReason as at, ActiveResult as b, NetworkImageSource as bn, MotionConfigProviderProps as br, SpinProps as bt, createTabs as c, LinkProps as cn, SweepOptions as cr, NotificationPlacement as ct, ShortcutHandler as d, CodeEditor as dn, animateKeyframes as dr, NotificationRegionProps as dt, Button as en, ReactiveAnimation as er, OverlayPlaneProvider as et, ShortcutMap as f, CodeEditorProps as fn, createLoop as fr, Notifications as ft, ScrollResetTarget as g, ImageProps as gn, createTransition as gr, PulseProps as gt, ScrollResetOptions as h, Image as hn, createSweep as hr, Pulse as ht, TabKey as i, ButtonProps as in, RotationOptions as ir, NotificationControls as it, ComputeFloatingPositionOptions as j, TextAreaProps as jn, MeasuredSizeOptions as jt, PopoverProps as k, Text as kn, ContainerSizeQuery as kt, ShortcutDefinition as l, createButton as ln, TransitionOptions as lr, NotificationPriority as lt, createShortcuts as m, IconProps as mn, createRotation as mr, createNotifications as mt, AddTabOptions as n, ButtonKeyEvent as nn, RepeatType as nr, createOverlayLayer as nt, TabsOptions as o, CreateButtonOptions as on, SweepAxis as or, NotificationInput as ot, ShortcutsResult as p, Icon as pn, createPulse as pr, NotificationsOptions as pt, size as q, AnimationState as qn, createHover as qt, FocusTarget as r, ButtonPrimitive as rn, RotationAnimation as rr, useOverlayPlane as rt, TabsResult as s, Link as sn, SweepGeometry as sr, NotificationItem as st, index_d_exports as t, ButtonEvent as tn, ReactiveTransition as tr, OverlayPlaneProviderProps as tt, ShortcutEvent as u, Affine2D$1 as un, animate as ur, NotificationRegion as ut, ScrollArea as v, NetworkImage as vn, MotionConfig as vr, RippleProps as vt, createPress as w, PathBuilder as wn, ModalKeyEvent as wt, PressOptions as x, PasswordInput as xn, useMotionConfig as xr, Modal as xt, ScrollAreaProps as y, NetworkImageProps as yn, MotionConfigProvider as yr, Spin as yt, Strategy$1 as z, WabouClassList as zn, KeyedSelectionOptions as zt };
//# sourceMappingURL=index-DnWGrLTM.d.mts.map