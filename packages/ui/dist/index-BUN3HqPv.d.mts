import { PathBuilder, VectorPath, VectorPath as VectorPath$1, VectorPathPaint } from "@wabou/core";
import { Affine2D, Affine2D as Affine2D$1, Shadow, WabouStyle, WabouStyle as WabouStyle$1, rotate2d as rotate2d$1, translate2d as translate2d$1 } from "@wabou/core/style";
import { InterpolateOptions } from "motion-dom";
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
/**
 * Backend-neutral repeating timeline executed by a retained native widget.
 *
 * JS owns animation intent and lifecycle; the native backend samples this
 * descriptor locally without receiving one protocol mutation per frame.
 */
interface NativeLoopAnimation {
  readonly kind: "loop";
  /** Duration of one iteration in seconds. */
  readonly duration: number;
  readonly speed: number;
  readonly paused: boolean;
  readonly reducedMotion: boolean;
}
interface NativeLoopAnimationOptions {
  duration?: MaybeAccessor<number>;
  speed?: MaybeAccessor<number>;
  paused?: MaybeAccessor<boolean>;
  reducedMotion?: MaybeAccessor<boolean>;
}
/**
 * Compile reactive Solid animation policy into a stable native timeline DTO.
 * This accessor changes only when authored policy changes, never per frame.
 */
declare function createNativeLoopAnimation(options?: NativeLoopAnimationOptions): Accessor<NativeLoopAnimation>;
interface KeyframeAnimationOptions<V extends AnimationValue> extends AnimationOptions<V> {
  /** Reactive policy which pauses interpolation and publishes the final keyframe. */
  reducedMotion?: MaybeAccessor<boolean>;
  /** Value exposed while motion is reduced. Defaults to the final keyframe. */
  reducedValue?: V;
}
/**
 * Lifecycle-owned finite or repeating keyframe animation.
 *
 * This is the general primitive behind loops, pulses and component-specific
 * effects. It owns cleanup and reduced-motion behavior so components don't
 * need to coordinate raw Motion controls themselves.
 */
declare function createKeyframeAnimation<V extends AnimationValue>(keyframes: readonly [V, V, ...V[]], options?: KeyframeAnimationOptions<V>): ReactiveAnimation<V>;
interface MotionInterpolationOptions<V extends AnimationValue> extends Pick<InterpolateOptions<V>, "clamp" | "ease"> {}
/** Map one reactive progress value to numeric, color, or complex keyframes. */
declare function createInterpolation(source: Accessor<number>, input: readonly number[], output: readonly number[], options?: MotionInterpolationOptions<number>): Accessor<number>;
declare function createInterpolation(source: Accessor<number>, input: readonly number[], output: readonly string[], options?: MotionInterpolationOptions<string>): Accessor<string>;
interface TransitionOptions extends Omit<AnimationOptions<number>, "autoplay" | "onUpdate" | "onComplete"> {
  /** Skip interpolation while the user's/application's reduced-motion policy is active. */
  reducedMotion?: MaybeAccessor<boolean>;
  /** One-time starting value. Defaults to the current target. */
  initial?: MaybeAccessor<number>;
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
}
interface ViewProps extends PrimitiveProps {}
interface TextProps extends PrimitiveProps {
  /** Maximum rendered lines. Overflow on the final line is replaced by an ellipsis. */
  maxLines?: number;
}
interface RichTextProps extends TextProps {}
/** A styled text-only descendant of RichText. Layout-box styles are invalid. */
interface RichTextSpanProps extends PrimitiveProps {}
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
interface ImageResourceHandle {
  lo: number;
  hi: number;
}
interface ImageResourceReadyEvent {
  resource: ImageResourceHandle;
  width: number;
  height: number;
}
interface ImageResourceErrorEvent {
  resource?: ImageResourceHandle;
  error: string;
}
interface ImageProps extends Omit<PrimitiveProps, "children"> {
  /** Borrowed host-owned immutable image. This component never releases it. */
  resource?: ImageResourceHandle;
  onResourceReady?: (event: ImageResourceReadyEvent) => void;
  onResourceError?: (event: ImageResourceErrorEvent) => void;
}
interface TextInputProps extends Omit<PrimitiveProps, "children"> {
  value?: string;
  placeholder?: string;
  disabled?: boolean;
  readOnly?: boolean;
  /**
   * For multiline editors, submit on plain Enter while keeping Shift+Enter as
   * a native newline edit.
   */
  submitOnEnter?: boolean;
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
interface EditorProps extends Omit<PrimitiveProps, "children"> {
  value?: string;
  /** Optional language identifier consumed by the native editor highlighter. */
  language?: string;
  disabled?: boolean;
  readOnly?: boolean;
  /** Submit on plain Enter while keeping Shift+Enter as a native newline edit. */
  submitOnEnter?: boolean;
  "aria-label": string;
  onInput?: (event: {
    currentTarget: {
      value: string;
    };
  }) => void;
}
type NativeWidgetConfig = object | readonly unknown[];
/**
 * Public boundary for an application-defined retained native widget.
 *
 * `tag` selects the Rust factory, `config` is its complete immutable authored
 * snapshot, and ordinary Wabou event props carry typed native events back to
 * Solid. Stateful native ownership remains keyed by this node's `NodeKey`.
 */
interface NativeWidgetProps<Config extends NativeWidgetConfig> extends Omit<PrimitiveProps, "children" | "widgetConfig"> {
  tag: string;
  config?: Config;
}
/** A layout container. Text content should be placed in a {@link Text}. */
declare function View(props: ViewProps): JSX.Element;
/**
 * Stable retained region projected through its own GPUI Entity.
 *
 * Use this around independently changing route content, scroll viewports,
 * overlays, native-widget regions, animation surfaces, or diagnostic HUDs.
 * It does not create application state and has the same layout semantics as a
 * View; it only limits native invalidation and materialization.
 */
declare function ProjectionBoundary(props: ViewProps): JSX.Element;
/**
 * A measured text run that wraps within its available width by default.
 *
 * Static and reactive child text nodes are concatenated by the native host and
 * participate in the parent layout as one item. Use `maxLines={1}` or
 * `whitespace-nowrap` when the text must remain on one line.
 */
declare function Text(props: TextProps): JSX.Element;
/**
 * One Parley paragraph assembled from explicitly styled text descendants.
 *
 * Unlike adjacent Text components, spans share wrapping, whitespace,
 * selection, and copy semantics because the native host lays them out once.
 */
declare function RichText(props: RichTextProps): JSX.Element;
/** A text-style boundary inside RichText; it never creates a layout box. */
declare function RichTextSpan(props: RichTextSpanProps): JSX.Element;
/** A static SVG asset rendered through the native usvg/Vello pipeline. */
declare function Svg(props: SvgProps): JSX.Element;
/** A native Vello vector path in local logical-pixel coordinates. */
declare function Path(props: PathProps): JSX.Element;
/** A theme-colored SVG icon with stable native sizing and semantics. */
declare function Icon(props: IconProps): JSX.Element;
/** A replaced image node rendered by the native host. */
declare function Image(props: ImageProps): JSX.Element;
/** A native single-line text editor with selection and scrolling. */
declare function TextInput(props: TextInputProps): JSX.Element;
/** A native multiline text editor with wrapping, selection, and scrolling. */
declare function TextArea(props: TextAreaProps): JSX.Element;
/** Native password editor whose value remains in a Rust SecretStore. */
declare function PasswordInput(props: PasswordInputProps): JSX.Element;
/** General-purpose editor whose document and input lifecycle are owned by GPUI. */
declare function Editor(props: EditorProps): JSX.Element;
/** Mount an explicitly registered Rust/GPUI widget without web-element semantics. */
declare function NativeWidget<Config extends NativeWidgetConfig = object>(props: NativeWidgetProps<Config>): JSX.Element;
//#endregion
//#region src/primitives/button.d.ts
declare const ACCENTS: {
  readonly neutral: "#475569";
  readonly sky: "#0284c7";
  readonly amber: "#d97706";
};
interface ButtonProps extends Pick<WabouElementProps, "aria-busy" | "aria-checked" | "aria-controls" | "aria-current" | "aria-expanded" | "aria-haspopup" | "aria-label" | "aria-pressed" | "aria-selected" | "aria-valuetext" | "role" | "focusOrder" | "onBlur" | "onContextMenu" | "onDblClick" | "onFocus" | "onFocusIn" | "onFocusOut" | "onKeyUp" | "onPointerCancel" | "onPointerDown" | "onPointerEnter" | "onPointerLeave" | "onPointerMove" | "onPointerOut" | "onPointerOver" | "onPointerUp" | "onWheel"> {
  class?: string | ((state: ButtonState) => string);
  classList?: WabouClassList | ((state: ButtonState) => WabouClassList);
  style?: WabouStyle$1 | ((state: ButtonState) => WabouStyle$1);
  children?: JSX.Element;
  /** Render an internal visual layer from normalized interaction state. */
  renderContent?: (state: ButtonState) => JSX.Element;
  tone?: keyof typeof ACCENTS;
  variant?: "solid" | "ghost";
  /** Keep interaction behavior but do not inject the default visual geometry. */
  unstyled?: boolean;
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
  code?: string;
  /** Physical Shift, Control, Alt, and Meta modifier bits. */
  mods?: number;
  /** Whether the physical modifiers form this platform's primary chord. */
  primary?: boolean;
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
  /** Props applied to the retained content node inside the animated viewport. */
  contentProps?: Omit<ViewProps, "children" | "class" | "style">;
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
//#region src/primitives/image-resource.d.ts
interface ImageResourceDescriptor {
  handle: ImageResourceHandle;
  width: number;
  height: number;
}
type ImageResourceRequest = {
  kind: "file";
  path: string;
} | {
  kind: "network";
  url: string;
};
interface OwnedImageResource {
  resource: Accessor<ImageResourceDescriptor | undefined>;
  loading: Accessor<boolean>;
  error: Accessor<unknown>;
}
/** Explicitly create a new resource from a host file. No identity deduplication occurs. */
declare function createFileImageResource(path: string): Promise<ImageResourceDescriptor>;
/** Explicitly create a new resource from an HTTP(S) response. */
declare function createNetworkImageResource(url: string): Promise<ImageResourceDescriptor>;
/** Deterministically release a resource. Images only borrow their handle. */
declare function releaseImageResource(handle: ImageResourceHandle): Promise<boolean>;
/**
 * Create a resource owned by the current Solid owner. Source replacement and
 * owner cleanup clear the borrowed handle before releasing the native resource.
 */
declare function createOwnedImageResource(request: Accessor<ImageResourceRequest | undefined>): OwnedImageResource;
//#endregion
//#region src/primitives/interactions/form-draft.d.ts
type FormDraftFieldUpdater<Value> = Value | ((previous: Value) => Value);
/** Validation key used for errors that do not belong to one field. */
declare const FORM_ERROR: unique symbol;
type FormDraftErrors<T> = Partial<Record<keyof T | typeof FORM_ERROR, string>>;
interface FormDraft<T extends Record<PropertyKey, unknown>> {
  value: Accessor<Readonly<T>>;
  dirty: Accessor<boolean>;
  /** Validation errors derived from the current immutable draft. */
  errors: Accessor<Readonly<FormDraftErrors<T>>>;
  valid: Accessor<boolean>;
  formError: Accessor<string | undefined>;
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
interface ModalMotionOptions {
  duration?: number;
  ease?: Easing;
  /** Enter-only timing override. Falls back to `duration`. */
  enterDuration?: number;
  /** Exit-only timing override. Falls back to `duration`. */
  exitDuration?: number;
  /** Enter-only easing override. Falls back to `ease`. */
  enterEase?: Easing;
  /** Exit-only easing override. Falls back to `ease`. */
  exitEase?: Easing;
  /** Initial content scale around its center. Defaults to 1. */
  fromScale?: number;
  /** Initial horizontal offset in logical pixels. Defaults to 0. */
  fromX?: number;
  /** Initial vertical offset in logical pixels. Defaults to 0. */
  fromY?: number;
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
  /** Keep the backdrop visible while the content exits. Edge panels disable this. */
  backdropFade?: boolean;
  contentClass?: string;
  contentStyle?: WabouStyle$1;
  /** Fade the content with the backdrop. Edge panels disable this and slide as solid surfaces. */
  contentFade?: boolean;
  /** Composes component-specific movement with the modal presence transform. */
  contentTransform?: (base: Affine2D, presenceProgress: number) => Affine2D;
  contentShadows?: readonly Shadow[] | null;
  contentRef?: (node: Handle) => void;
  closeOnBackdrop?: boolean;
  closeOnEscape?: boolean;
  /** Overrides the host's default of focusing the first focusable descendant. */
  initialFocus?: () => Handle | undefined;
  restoreFocus?: boolean;
  /** Headless Modal is static by default; styled dialogs opt into motion. */
  motion?: false | ModalMotionOptions;
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
  /** Width and presentation of the native GPUI-base stack container. */
  stackClass?: string;
  itemClass?: string;
  itemStyle?: WabouStyle$1;
  /** Headless regions are static unless motion is explicitly requested. */
  motion?: false | NotificationMotionOptions;
}
interface NotificationMotionOptions {
  duration?: number;
  ease?: Easing;
  /** Initial horizontal offset in logical pixels. */
  fromX?: number;
  /** Initial vertical offset in logical pixels. */
  fromY?: number;
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
  onPointerDown: (event: {
    button?: number;
    stopPropagation(): void;
  }) => void;
  onPointerCancel: () => void;
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
  /** Removes the positioned content subtree from native hit testing. */
  contentInteractionBlocked?: boolean;
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
  /** Set to false to keep presence semantics while disabling visual motion. */
  motion?: false | PopoverMotionOptions;
  /** Open on primary pointer-down; useful for native-feeling selects and menus. */
  openOnPointerDown?: boolean;
}
interface PopoverMotionOptions {
  duration?: number;
  ease?: Easing;
  /** Initial scale around the panel center. Defaults to 0.98. */
  fromScale?: number;
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
//#region src/primitives/retained-items.d.ts
interface RetainedItem<T, Key> {
  readonly key: Key;
  /** Latest source value for this key, including while it exits. */
  readonly value: Accessor<T>;
  /** False as soon as the key leaves the logical source. */
  readonly present: Accessor<boolean>;
}
interface RetainedItems<T, Key> {
  /** Active entries plus entries waiting for their visual exit to finish. */
  readonly entries: Accessor<readonly RetainedItem<T, Key>[]>;
  /** Remove an absent key after its exit completes. */
  release(key: Key): boolean;
}
/**
 * Keep keyed values mounted after logical removal until `release` is called.
 *
 * Entries are stable by key, expose the latest source value, and report
 * logical presence independently from visual retention. This is the common
 * lifecycle needed by exit animations without delaying state or semantics.
 */
declare function createRetainedItems<T, Key>(source: Accessor<readonly T[]>, key: (item: T) => Key): RetainedItems<T, Key>;
//#endregion
//#region src/primitives/scroll-area.d.ts
interface ScrollAreaProps extends Omit<ViewProps, "children" | "class" | "style" | "ref" | "scrollbar" | "onScroll"> {
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
//#endregion
//#region src/primitives/transition-presence.d.ts
interface TransitionPresenceOptions {
  /**
   * Reactive visual readiness. Mounting still follows `open`, but entry waits
   * for this value. This is useful for positioned overlays that must complete
   * native measurement before becoming visible.
   */
  ready?: Accessor<boolean>;
  /** Start visual progress independently from logical presence. */
  initialProgress?: number;
  duration?: number;
  ease?: Easing;
  reducedMotion?: boolean | Accessor<boolean>;
}
interface TransitionPresence {
  phase: Accessor<PresencePhase>;
  mounted: Accessor<boolean>;
  /** Normalized visual progress: 0 when hidden and 1 when fully present. */
  progress: Accessor<number>;
  transition: ReactiveTransition;
}
/**
 * Couples logical presence to an interruptible visual transition.
 *
 * Closing disables the logical surface immediately while keeping its visual
 * subtree mounted until progress reaches zero. Reopening during exit simply
 * retargets the current transition instead of remounting the subtree.
 */
declare function createTransitionPresence(open: Accessor<boolean>, options?: TransitionPresenceOptions): TransitionPresence;
declare namespace index_d_exports {
  export { ActiveResult, AddTabOptions, Affine2D$1 as Affine2D, AnimationFrameCallback, Button, ButtonEvent, ButtonKeyEvent, ButtonPrimitive, ButtonProps, ButtonState, Center, CollapsiblePresence, CollapsiblePresenceProps, Column, ComputeFloatingPositionOptions, ComputeHostFloatingPositionOptions, ComputePositionReturn$1 as ComputePositionReturn, ContainerMatch, ContainerSizeQuery, CreateButtonOptions, DismissEvent, DismissKeyEvent, Editor, EditorProps, FORM_ERROR, FocusResult, FocusTarget, FocusWithinResult, FormDraft, FormDraftErrors, FormDraftFieldUpdater, FormDraftOptions, HoverResult, Icon, IconProps, Image, ImageProps, ImageResourceDescriptor, ImageResourceErrorEvent, ImageResourceHandle, ImageResourceReadyEvent, ImageResourceRequest, KeyedSelection, KeyedSelectionOptions, LayoutProps, LayoutRect$1 as LayoutRect, Link, LinkProps, MeasuredSize, MeasuredSizeOptions, Middleware$1 as Middleware, Modal, ModalControls, ModalEvent, ModalKeyEvent, ModalMotionOptions, ModalOpenChangeReason, ModalProps, ModalTriggerProps, NativeWidget, NativeWidgetConfig, NativeWidgetProps, NotificationControls, NotificationDismissReason, NotificationInput, NotificationItem, NotificationMotionOptions, NotificationPlacement, NotificationPriority, NotificationRegion, NotificationRegionProps, Notifications, NotificationsOptions, OverlayDismissReason, OverlayLayer, OverlayLayerOptions, OverlayPlane, OverlayPlaneProvider, OverlayPlaneProviderProps, OwnedImageResource, PasswordInput, PasswordInputProps, Path, PathBuilder, PathProps, Placement$1 as Placement, Popover, PopoverMotionOptions, PopoverProps, PopoverTriggerProps, PositionPlatform, Presence, PresencePhase, PressOptions, PressResult, PrimitiveProps, ProjectionBoundary, Pulse, PulseProps, RetainedItem, RetainedItems, RichText, RichTextProps, RichTextSpan, RichTextSpanProps, Ripple, RippleProps, Row, ScrollArea, ScrollAreaProps, ScrollResetOptions, ScrollResetTarget, ShortcutDefinition, ShortcutEvent, ShortcutHandler, ShortcutMap, ShortcutsResult, Spin, SpinProps, Strategy$1 as Strategy, Svg, SvgProps, TabKey, TabKeyEvent, TabsOptions, TabsResult, Text, TextArea, TextAreaProps, TextInput, TextInputProps, TextProps, TransitionPresence, TransitionPresenceOptions, VectorPath$1 as VectorPath, VectorPathPaint, View, ViewProps, WabouClassList, WabouStyle$1 as WabouStyle, arrow, autoPlacement, computeFloatingPosition, computeHostFloatingPosition, createActive, createAnimationFrame, createButton, createContainerMatch, createFileImageResource, createFocus, createFocusWithin, createFormDraft, createHover, createKeyedSelection, createMeasuredSize, createNetworkImageResource, createNotifications, createOverlayLayer, createOwnedImageResource, createPresence, createPress, createRetainedItems, createScrollReset, createShortcuts, createTabs, createTransitionPresence, flip, offset, releaseImageResource, rotate2d$1 as rotate2d, shift, size, translate2d$1 as translate2d, useOverlayPlane };
}
//#endregion
export { shift as $, Text as $n, useReducedMotion as $r, FormDraftFieldUpdater as $t, createPress as A, Icon as An, RepeatType as Ar, ModalControls as At, ComputePositionReturn$1 as B, PasswordInput as Bn, createInterpolation as Br, MeasuredSizeOptions as Bt, RetainedItem as C, CreateButtonOptions as Cn, LoopOptions as Cr, Pulse as Ct, PressOptions as D, Affine2D$1 as Dn, PulseOptions as Dr, Spin as Dt, ActiveResult as E, createButton as En, NativeLoopAnimationOptions as Er, RippleProps as Et, PopoverMotionOptions as F, ImageResourceHandle as Fn, SweepGeometry as Fr, ModalProps as Ft, PositionPlatform as G, PrimitiveProps as Gn, createRotation as Gr, LayoutProps as Gt, Middleware$1 as H, Path as Hn, createLoop as Hr, createMeasuredSize as Ht, PopoverProps as I, ImageResourceReadyEvent as In, SweepOptions as Ir, ModalTriggerProps as It, autoPlacement as J, RichTextProps as Jn, normalizeSweepGeometry as Jr, KeyedSelectionOptions as Jt, Strategy$1 as K, ProjectionBoundary as Kn, createSweep as Kr, Row as Kt, PopoverTriggerProps as L, NativeWidget as Ln, TransitionOptions as Lr, ContainerMatch as Lt, PresencePhase as M, Image as Mn, RotationOptions as Mr, ModalKeyEvent as Mt, createPresence as N, ImageProps as Nn, SweepAnimation as Nr, ModalMotionOptions as Nt, PressResult as O, Editor as On, ReactiveAnimation as Or, SpinProps as Ot, Popover as P, ImageResourceErrorEvent as Pn, SweepAxis as Pr, ModalOpenChangeReason as Pt, offset as Q, SvgProps as Qn, useMotionConfig as Qr, FormDraftErrors as Qt, ComputeFloatingPositionOptions as R, NativeWidgetConfig as Rn, animate as Rr, ContainerSizeQuery as Rt, ScrollAreaProps as S, ButtonState as Sn, KeyframeAnimationOptions as Sr, createNotifications as St, createRetainedItems as T, LinkProps as Tn, NativeLoopAnimation as Tr, Ripple as Tt, Placement$1 as U, PathBuilder as Un, createNativeLoopAnimation as Ur, Center as Ut, LayoutRect$1 as V, PasswordInputProps as Vn, createKeyframeAnimation as Vr, createContainerMatch as Vt, PointAnchor as W, PathProps as Wn, createPulse as Wr, Column as Wt, computeHostFloatingPosition as X, RichTextSpanProps as Xn, MotionConfigProvider as Xr, FORM_ERROR as Xt, computeFloatingPosition as Y, RichTextSpan as Yn, MotionConfig as Yr, createKeyedSelection as Yt, flip as Z, Svg as Zn, MotionConfigProviderProps as Zr, FormDraft as Zt, createShortcuts as _, Button as _n, AnimationState as _r, NotificationPriority as _t, AddTabOptions as a, createFileImageResource as an, VectorPath$1 as ar, OverlayLayerOptions as at, createScrollReset as b, ButtonPrimitive as bn, Easing as br, Notifications as bt, TabKeyEvent as c, releaseImageResource as cn, ViewProps as cr, OverlayPlaneProviderProps as ct, createTabs as d, FocusResult as dn, rotate2d$1 as dr, NotificationControls as dt, FormDraftOptions as en, TextArea as er, size as et, ShortcutDefinition as f, FocusWithinResult as fn, translate2d$1 as fr, NotificationDismissReason as ft, ShortcutsResult as g, CollapsiblePresenceProps as gn, AnimationOptions as gr, NotificationPlacement as gt, ShortcutMap as h, CollapsiblePresence as hn, AnimationControls as hr, NotificationMotionOptions as ht, createTransitionPresence as i, OwnedImageResource as in, TextProps as ir, OverlayLayer as it, Presence as j, IconProps as jn, RotationAnimation as jr, ModalEvent as jt, createActive as k, EditorProps as kn, ReactiveTransition as kr, Modal as kt, TabsOptions as l, HoverResult as ln, WabouClassList as lr, createOverlayLayer as lt, ShortcutHandler as m, createFocusWithin as mn, createAnimationFrame as mr, NotificationItem as mt, TransitionPresence as n, ImageResourceDescriptor as nn, TextInput as nr, DismissKeyEvent as nt, FocusTarget as o, createNetworkImageResource as on, VectorPathPaint as or, OverlayPlane as ot, ShortcutEvent as p, createFocus as pn, AnimationFrameCallback as pr, NotificationInput as pt, arrow as q, RichText as qn, createTransition as qr, KeyedSelection as qt, TransitionPresenceOptions as r, ImageResourceRequest as rn, TextInputProps as rr, OverlayDismissReason as rt, TabKey as s, createOwnedImageResource as sn, View as sr, OverlayPlaneProvider as st, index_d_exports as t, createFormDraft as tn, TextAreaProps as tr, DismissEvent as tt, TabsResult as u, createHover as un, WabouStyle$1 as ur, useOverlayPlane as ut, ScrollResetOptions as v, ButtonEvent as vn, AnimationType as vr, NotificationRegion as vt, RetainedItems as w, Link as wn, MotionInterpolationOptions as wr, PulseProps as wt, ScrollArea as x, ButtonProps as xn, EasingFunction as xr, NotificationsOptions as xt, ScrollResetTarget as y, ButtonKeyEvent as yn, AnimationValue as yr, NotificationRegionProps as yt, ComputeHostFloatingPositionOptions as z, NativeWidgetProps as zn, animateKeyframes as zr, MeasuredSize as zt };
//# sourceMappingURL=index-BUN3HqPv.d.mts.map