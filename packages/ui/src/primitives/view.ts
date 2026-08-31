import type { VectorPath } from "@wabou/core";
import {
  createElement,
  type Handle,
  mergeProps,
  type NativeScrollbarStyle,
  spread,
  TEXT_BEHAVIOR,
  type WabouElementProps,
  type WabouTextSelectionChangeEvent,
} from "@wabou/core/renderer";
import {
  type Affine2D,
  px,
  type Shadow,
  type WabouStyle,
} from "@wabou/core/style";

export type { VectorPath, VectorPathPaint } from "@wabou/core";
export { PathBuilder } from "@wabou/core";

import { type JSX, omit, untrack } from "solid-js";

export type { Affine2D, WabouStyle } from "@wabou/core/style";
export { rotate2d, translate2d } from "@wabou/core/style";
export type WabouClassList = Record<string, boolean | undefined>;

export type TextSelectionChangeEvent = WabouTextSelectionChangeEvent;

export interface PrimitiveProps
  extends Omit<WabouElementProps, "children" | "ref" | "style"> {
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

export interface ViewProps extends PrimitiveProps {}

export interface TextProps extends PrimitiveProps {
  /** Maximum rendered lines. Overflow on the final line is replaced by an ellipsis. */
  maxLines?: number;
}

export interface RichTextProps extends TextProps {}

/** A styled text-only descendant of RichText. Layout-box styles are invalid. */
export interface RichTextSpanProps extends PrimitiveProps {}

export interface SvgProps extends Omit<PrimitiveProps, "children"> {
  /** Trusted inline SVG source parsed and cached by the native host. */
  source: string;
}

export interface PathProps extends Omit<PrimitiveProps, "children"> {
  /** Immutable geometry and paint snapshot built with PathBuilder. */
  source: VectorPath;
}

export interface IconProps extends Omit<SvgProps, "source"> {
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

const ICON_SIZE_UNITLESS_RE = /^-?\d*\.?\d+$/;

function normalizeIconSize(size: number | string | undefined): number | string {
  if (size == null) return "1em";
  if (typeof size === "number") return size;
  const value = size.trim();
  if (!value) return "1em";
  const parsed = Number.parseFloat(value);
  if (Number.isFinite(parsed) && ICON_SIZE_UNITLESS_RE.test(value)) {
    return parsed;
  }
  return value;
}

function applyIconFill(source: string, fill: string): string {
  return source.replace(/fill=(["'])none\1/, `fill="${fill}"`);
}

export interface ImageResourceHandle {
  lo: number;
  hi: number;
}

export interface ImageResourceReadyEvent {
  resource: ImageResourceHandle;
  width: number;
  height: number;
}

export interface ImageResourceErrorEvent {
  resource?: ImageResourceHandle;
  error: string;
}

export interface ImageProps extends Omit<PrimitiveProps, "children"> {
  /** Borrowed host-owned immutable image. This component never releases it. */
  resource?: ImageResourceHandle;
  onResourceReady?: (event: ImageResourceReadyEvent) => void;
  onResourceError?: (event: ImageResourceErrorEvent) => void;
}

export interface TextInputProps extends Omit<PrimitiveProps, "children"> {
  value?: string;
  placeholder?: string;
  disabled?: boolean;
  readOnly?: boolean;
  onInput?: (event: { currentTarget: { value: string } }) => void;
}

export interface TextAreaProps extends TextInputProps {}

export interface PasswordInputProps extends Omit<PrimitiveProps, "children"> {
  /** Rust SecretStore slot. This is an identifier, never the secret value. */
  secret: string;
  placeholder?: string;
  disabled?: boolean;
  "aria-label"?: string;
  onKeyDown?: (event: { key: string; preventDefault(): void }) => void;
}

export interface EditorProps extends Omit<PrimitiveProps, "children"> {
  value?: string;
  /** Optional language identifier consumed by the native editor highlighter. */
  language?: string;
  disabled?: boolean;
  readOnly?: boolean;
  "aria-label": string;
  onInput?: (event: { currentTarget: { value: string } }) => void;
}

export type NativeWidgetConfig = object | readonly unknown[];

/**
 * Public boundary for an application-defined retained native widget.
 *
 * `tag` selects the Rust factory, `config` is its complete immutable authored
 * snapshot, and ordinary Wabou event props carry typed native events back to
 * Solid. Stateful native ownership remains keyed by this node's `NodeKey`.
 */
export interface NativeWidgetProps<Config extends NativeWidgetConfig>
  extends Omit<PrimitiveProps, "children" | "widgetConfig"> {
  tag: string;
  config?: Config;
}

type InternalPrimitiveTag =
  | "view"
  | "text"
  | "text-span"
  | "svg"
  | "img"
  | "button"
  | "input"
  | "textarea"
  | "password-input"
  | "editor"
  | "vector-path";

/** @internal Host tags are renderer details, not public JSX elements. */
export function createInternalPrimitive(
  tag: InternalPrimitiveTag,
  props: object,
) {
  const node = createElement(tag);
  spread(node, props, false);
  return node as unknown as JSX.Element;
}

function primitive(
  tag:
    | "view"
    | "text"
    | "text-span"
    | "svg"
    | "img"
    | "input"
    | "textarea"
    | "password-input"
    | "editor"
    | "vector-path",
  props: PrimitiveProps,
) {
  return createInternalPrimitive(tag, props);
}

function editorPrimitive(
  tag: "input" | "textarea" | "password-input" | "editor",
  props: TextInputProps | PasswordInputProps | EditorProps,
) {
  // Keyboard policy belongs to the JS primitive. The widget trait only says
  // whether a native implementation can receive focus once JS requests it.
  return primitive(
    tag,
    mergeProps(props, {
      get role() {
        return props.role ?? "textbox";
      },
      get focusOrder() {
        return props.disabled ? -1 : (props.focusOrder ?? 0);
      },
      get "aria-disabled"() {
        return props.disabled ?? false;
      },
    }) as PrimitiveProps,
  );
}

function semanticPrimitive(
  tag: "text" | "svg" | "img",
  role: "label" | "img",
  props: PrimitiveProps,
) {
  const node = createElement(tag);
  // Primitive semantics are authored here in JavaScript. The native runtime
  // projects explicit roles; it does not infer behavior from tag names.
  spread(node, { role }, false);
  spread(node, props, false);
  return node as unknown as JSX.Element;
}

/** A layout container. Text content should be placed in a {@link Text}. */
export function View(props: ViewProps): JSX.Element {
  return primitive("view", props);
}

function resolvedTextBehavior(maxLines: number | undefined) {
  if (maxLines != null && (!Number.isInteger(maxLines) || maxLines < 1)) {
    throw new RangeError("Text maxLines must be a positive integer");
  }
  return {
    flags:
      TEXT_BEHAVIOR.AggregateDirectText |
      (maxLines === 1 ? TEXT_BEHAVIOR.SingleLine : 0),
    maxLines: maxLines ?? 0,
  };
}

/**
 * A measured text run that wraps within its available width by default.
 *
 * Static and reactive child text nodes are concatenated by the native host and
 * participate in the parent layout as one item. Use `maxLines={1}` or
 * `whitespace-nowrap` when the text must remain on one line.
 */
export function Text(props: TextProps): JSX.Element {
  // Validate the initial value before Solid owns the reactive spread. An
  // exception thrown from inside a render effect becomes a StatusError and
  // intentionally halts the owner, which would obscure this public API error.
  resolvedTextBehavior(untrack(() => props.maxLines));
  const node = createElement("text");
  spread(node, omit(props, "maxLines"), false);
  spread(
    node,
    {
      role: props.role ?? "label",
      get textBehavior() {
        return resolvedTextBehavior(props.maxLines);
      },
    },
    false,
  );
  return node as unknown as JSX.Element;
}

/**
 * One Parley paragraph assembled from explicitly styled text descendants.
 *
 * Unlike adjacent Text components, spans share wrapping, whitespace,
 * selection, and copy semantics because the native host lays them out once.
 */
export function RichText(props: RichTextProps): JSX.Element {
  resolvedTextBehavior(untrack(() => props.maxLines));
  const node = createElement("text");
  spread(node, omit(props, "maxLines"), false);
  spread(
    node,
    {
      role: props.role ?? "label",
      get textBehavior() {
        const behavior = resolvedTextBehavior(props.maxLines);
        return {
          ...behavior,
          flags: behavior.flags | TEXT_BEHAVIOR.AggregateStyledText,
        };
      },
    },
    false,
  );
  return node as unknown as JSX.Element;
}

/** A text-style boundary inside RichText; it never creates a layout box. */
export function RichTextSpan(props: RichTextSpanProps): JSX.Element {
  return primitive("text-span", props);
}

/** A static SVG asset rendered through the native usvg/Vello pipeline. */
export function Svg(props: SvgProps): JSX.Element {
  return semanticPrimitive("svg", "img", props);
}

/** A native Vello vector path in local logical-pixel coordinates. */
export function Path(props: PathProps): JSX.Element {
  return primitive("vector-path", props);
}

/** A theme-colored SVG icon with stable native sizing and semantics. */
export function Icon(props: IconProps): JSX.Element {
  const rest = omit(props, "source", "size", "fill", "label", "class");
  const node = createElement("svg");
  spread(node, rest, false);
  spread(
    node,
    {
      get class() {
        return props.class
          ? `self-center shrink-0 ${props.class}`
          : "self-center shrink-0";
      },
      get style(): WabouStyle {
        const iconSize = normalizeIconSize(props.size);
        const layoutSize =
          typeof iconSize === "number" ? px(iconSize) : iconSize;
        return {
          // Wabou has no inline formatting context. An icon is an ordinary
          // explicitly-sized flex item in every native backend.
          display: "flex",
          "align-items": "center",
          "justify-content": "center",
          "align-self": "center",
          width: layoutSize,
          height: layoutSize,
          "flex-shrink": 0,
          "line-height": "1",
          // Icon is visual content, never the interaction owner. Keeping it
          // out of native hit testing makes an enclosing Button receive the
          // complete pointer gesture directly; callers can opt back in with
          // an explicit style when building a genuinely interactive graphic.
          "pointer-events": "none",
          ...(props.style ?? {}),
        };
      },
      get width(): string | undefined {
        const iconSize = normalizeIconSize(props.size);
        return typeof iconSize === "number" ? String(iconSize) : undefined;
      },
      get source() {
        return props.fill && props.fill !== "none"
          ? applyIconFill(props.source, props.fill)
          : props.source;
      },
      get height() {
        const iconSize = normalizeIconSize(props.size);
        return typeof iconSize === "number" ? String(iconSize) : undefined;
      },
      get role() {
        return props.label ? "img" : undefined;
      },
      get "aria-label"() {
        return props.label;
      },
      get "aria-hidden"() {
        return props.label ? undefined : "true";
      },
    },
    false,
  );
  return node as unknown as JSX.Element;
}

/** A replaced image node rendered by the native host. */
export function Image(props: ImageProps): JSX.Element {
  const rest = omit(props, "resource");
  const node = createElement("img");
  spread(node, { role: "img" }, false);
  spread(node, rest, false);
  spread(
    node,
    {
      get resource(): ImageResourceHandle | undefined {
        return props.resource;
      },
    },
    false,
  );
  return node as unknown as JSX.Element;
}

/** A native single-line text editor with selection and scrolling. */
export function TextInput(props: TextInputProps): JSX.Element {
  return editorPrimitive("input", props);
}

/** A native multiline text editor with wrapping, selection, and scrolling. */
export function TextArea(props: TextAreaProps): JSX.Element {
  return editorPrimitive("textarea", props);
}

/** Native password editor whose value remains in a Rust SecretStore. */
export function PasswordInput(props: PasswordInputProps): JSX.Element {
  return editorPrimitive("password-input", props);
}

/** General-purpose editor whose document and input lifecycle are owned by GPUI. */
export function Editor(props: EditorProps): JSX.Element {
  return editorPrimitive("editor", props);
}

/** Mount an explicitly registered Rust/GPUI widget without web-element semantics. */
export function NativeWidget<Config extends NativeWidgetConfig = object>(
  props: NativeWidgetProps<Config>,
): JSX.Element {
  const tag = untrack(() => props.tag.trim());
  if (!tag) throw new TypeError("NativeWidget tag must not be empty");
  const rest = omit(props, "tag", "config");
  const node = createElement(tag);
  spread(node, rest, false);
  spread(
    node,
    {
      get widgetConfig(): Config | undefined {
        return props.config;
      },
    },
    false,
  );
  return node as unknown as JSX.Element;
}
