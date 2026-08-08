import {
  createElement,
  type Handle,
  type NativeScrollbarStyle,
  spread,
} from "@wabou/solid-renderer";
import type { Affine2D, Shadow, WabouStyle } from "@wabou/style";
import type { JSX } from "solid-js";

export type { Affine2D, WabouStyle } from "@wabou/style";
export { translate2d } from "@wabou/style";
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

export interface ImageProps extends Omit<PrimitiveProps, "children"> {
  src?: string;
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
  tag: "view" | "text" | "img" | "textarea" | "password-input",
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

/** A replaced image node rendered by the native host. */
export function Image(props: ImageProps): JSX.Element {
  return primitive("img", props);
}

/** A native multiline text editor with wrapping, selection, and scrolling. */
export function TextArea(props: TextAreaProps): JSX.Element {
  return primitive("textarea", props);
}

/** Native password editor whose value remains in a Rust SecretStore. */
export function PasswordInput(props: PasswordInputProps): JSX.Element {
  return primitive("password-input", props);
}
