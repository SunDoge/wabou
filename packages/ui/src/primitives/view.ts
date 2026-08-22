import type { VectorPath } from "@wabou/core";
import {
  createElement,
  type Handle,
  mergeProps,
  type NativeScrollbarStyle,
  spread,
  TEXT_BEHAVIOR,
  type WabouElementProps,
} from "@wabou/core/renderer";
import type { Affine2D, Shadow, WabouStyle } from "@wabou/core/style";

export type { VectorPath, VectorPathPaint } from "@wabou/core";
export { PathBuilder } from "@wabou/core";

import { type JSX, omit, untrack } from "solid-js";

export type { Affine2D, WabouStyle } from "@wabou/core/style";
export { rotate2d, translate2d } from "@wabou/core/style";
export type WabouClassList = Record<string, boolean | undefined>;

export interface TextSelectionChangeEvent {
  type: "textselectionchange";
  text: string | null;
  kind: "simple" | "word" | "line" | null;
}

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
  /** Fires once when a native text selection gesture commits or changes asynchronously. */
  onTextSelectionChange?: (event: TextSelectionChangeEvent) => void;
}

export interface ViewProps extends PrimitiveProps {}

export interface TextProps extends PrimitiveProps {
  /** Maximum rendered lines. Overflow on the final line is replaced by an ellipsis. */
  maxLines?: number;
}

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

export interface NetworkImageSource {
  kind: "network";
  url: string;
  format: "raster";
  /** Decoded pixels are shared by URL for the lifetime of this native runtime. */
  cache: "memory";
}

export interface FileImageSource {
  kind: "file";
  /** Absolute or application-resolved path owned by the native host. */
  path: string;
}

export type ImageSource = FileImageSource | NetworkImageSource;

export interface ImageProps extends Omit<PrimitiveProps, "children"> {
  /** Low-level native source. Prefer a source-specific component. */
  source?: ImageSource;
}

export interface NetworkImageProps extends Omit<ImageProps, "source"> {
  /** This component performs a host network request for the URL. */
  url: string;
  format: "raster";
  cache: "memory";
  /** Fired when the current URL is decoded and ready for native painting. */
  onResourceReady?: (event: { url: string }) => void;
  /** Fired when the current URL fails to download or decode. */
  onResourceError?: (event: { url: string; error: string }) => void;
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

export interface CodeEditorProps extends Omit<PrimitiveProps, "children"> {
  value?: string;
  /** The initial experimental adapter supports JSON highlighting. */
  language?: "json";
  disabled?: boolean;
  readOnly?: boolean;
  "aria-label": string;
  onInput?: (event: { currentTarget: { value: string } }) => void;
}

type InternalPrimitiveTag =
  | "view"
  | "text"
  | "svg"
  | "img"
  | "button"
  | "input"
  | "textarea"
  | "password-input"
  | "code-editor"
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
    | "svg"
    | "img"
    | "input"
    | "textarea"
    | "password-input"
    | "code-editor"
    | "vector-path",
  props: PrimitiveProps,
) {
  return createInternalPrimitive(tag, props);
}

function editorPrimitive(
  tag: "input" | "textarea" | "password-input" | "code-editor",
  props: TextInputProps | PasswordInputProps | CodeEditorProps,
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
      (maxLines == null || maxLines === 1 ? TEXT_BEHAVIOR.SingleLine : 0),
    maxLines: maxLines ?? 0,
  };
}

/**
 * A single measured text run.
 *
 * Static and reactive child text nodes are concatenated by the native host and
 * participate in the parent layout as one item.
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
        return {
          display: "inline-flex",
          "align-items": "center",
          "justify-content": "center",
          "align-self": "center",
          width: iconSize,
          height: iconSize,
          "flex-shrink": 0,
          "line-height": "1",
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
  const rest = omit(props, "source");
  const node = createElement("img");
  spread(node, { role: "img" }, false);
  spread(node, rest, false);
  spread(
    node,
    {
      get src(): string | undefined {
        const source = props.source;
        return source?.kind === "file" ? source.path : undefined;
      },
      get source(): NetworkImageSource | undefined {
        const source = props.source;
        return source?.kind === "network" ? source : undefined;
      },
    },
    false,
  );
  return node as unknown as JSX.Element;
}

/** An explicit network-backed image with bounded decoding and host caching. */
export function NetworkImage(props: NetworkImageProps): JSX.Element {
  const rest = omit(props, "url", "format", "cache");
  return Image(
    mergeProps(rest, {
      get source(): NetworkImageSource {
        return {
          kind: "network",
          url: props.url,
          format: props.format,
          cache: props.cache,
        };
      },
    }) as ImageProps,
  );
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

/** Experimental native editor for config and script-sized documents. */
export function CodeEditor(props: CodeEditorProps): JSX.Element {
  return editorPrimitive("code-editor", props);
}
