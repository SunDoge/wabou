import {
  createElement,
  type Handle,
  type NativeScrollbarStyle,
  spread,
} from "@wabou/solid-renderer";
import type { Affine2D, Shadow, WabouStyle } from "@wabou/style";
import { omit, type JSX } from "solid-js";

export type { Affine2D, WabouStyle } from "@wabou/style";
export { rotate2d, translate2d } from "@wabou/style";
export type WabouClassList = Record<string, boolean | undefined>;

export interface TextSelectionChangeEvent {
  type: "textselectionchange";
  text: string | null;
  kind: "simple" | "word" | "line" | null;
}

interface PrimitiveProps {
  class?: string;
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
  [name: string]: unknown;
}

export interface ViewProps extends PrimitiveProps {}

export interface TextProps extends PrimitiveProps {}

export interface SvgProps extends Omit<PrimitiveProps, "children"> {
  /** Trusted inline SVG source parsed and cached by the native host. */
  source: string;
}

export interface IconProps extends Omit<SvgProps, "source"> {
  source: string;
  size?: number;
  /** Override Lucide's root `fill="none"`, for example with currentColor. */
  fill?: "none" | "currentColor";
  /** Accessible name. Omit for a decorative icon. */
  label?: string;
}

export interface NetworkImageSource {
  kind: "network";
  url: string;
  format: "raster";
  /** Decoded pixels are shared by URL for the lifetime of this native runtime. */
  cache: "memory";
}

export type ImageSource = NetworkImageSource;

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

export interface TextAreaProps extends Omit<PrimitiveProps, "children"> {
  value?: string;
  placeholder?: string;
  disabled?: boolean;
  readOnly?: boolean;
  onInput?: (event: { currentTarget: { value: string } }) => void;
}

export interface PasswordInputProps extends Omit<PrimitiveProps, "children"> {
  /** Rust SecretStore slot. This is an identifier, never the secret value. */
  secret: string;
  placeholder?: string;
  disabled?: boolean;
  "aria-label"?: string;
  onKeyDown?: (event: { key: string; preventDefault(): void }) => void;
}

function primitive(
  tag: "view" | "text" | "svg" | "img" | "textarea" | "password-input",
  props: PrimitiveProps,
) {
  const node = createElement(tag);
  spread(node, props, false);
  return node as unknown as JSX.Element;
}

/** A layout container. Text content should be placed in a {@link Text}. */
export function View(props: ViewProps): JSX.Element {
  return primitive("view", props);
}

/**
 * A single measured text run.
 *
 * Static and reactive child text nodes are concatenated by the native host and
 * participate in the parent layout as one item.
 */
export function Text(props: TextProps): JSX.Element {
  return primitive("text", props);
}

/** A static SVG asset rendered through the native usvg/Vello pipeline. */
export function Svg(props: SvgProps): JSX.Element {
  return primitive("svg", props);
}

/** A theme-colored SVG icon with stable native sizing and semantics. */
export function Icon(props: IconProps): JSX.Element {
  const rest = omit(props, "source", "size", "fill", "label");
  const node = createElement("svg");
  spread(node, rest, false);
  spread(
    node,
    {
      get source() {
        return props.fill && props.fill !== "none"
          ? props.source.replace('fill="none"', `fill="${props.fill}"`)
          : props.source;
      },
      get width() {
        return String(props.size ?? 24);
      },
      get height() {
        return String(props.size ?? 24);
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
  return primitive("img", props);
}

/** An explicit network-backed image with bounded decoding and host caching. */
export function NetworkImage(props: NetworkImageProps): JSX.Element {
  const rest = omit(props, "url", "format", "cache");
  const node = createElement("img");
  spread(node, rest, false);
  spread(
    node,
    {
      get source(): NetworkImageSource {
        return {
          kind: "network",
          url: props.url,
          format: props.format,
          cache: props.cache,
        };
      },
    },
    false,
  );
  return node as unknown as JSX.Element;
}

/** A native multiline text editor with wrapping, selection, and scrolling. */
export function TextArea(props: TextAreaProps): JSX.Element {
  return primitive("textarea", props);
}

/** Native password editor whose value remains in a Rust SecretStore. */
export function PasswordInput(props: PasswordInputProps): JSX.Element {
  return primitive("password-input", props);
}
