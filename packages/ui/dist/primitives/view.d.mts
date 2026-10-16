import { PathBuilder, VectorPath, VectorPath as VectorPath$1, VectorPathPaint } from "@wabou/core";
import { Affine2D, Affine2D as Affine2D$1, Shadow, WabouStyle, WabouStyle as WabouStyle$1, rotate2d as rotate2d$1, translate2d as translate2d$1 } from "@wabou/core/style";
import { JSX } from "solid-js";
import { Handle, NativeScrollbarStyle, WabouElementProps } from "@wabou/core/renderer";
//#region src/primitives/view.d.ts
export type WabouClassList = Record<string, boolean | undefined>;
export interface PrimitiveProps extends Omit<WabouElementProps, "children" | "ref" | "style"> {
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
  /** Maximum rendered lines. */
  maxLines?: number;
  /** How content beyond `maxLines` ends. Defaults to `ellipsis`. */
  lineOverflow?: "ellipsis" | "clip";
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
export interface TextAreaProps extends TextInputProps {}
export interface PasswordInputProps extends Omit<PrimitiveProps, "children"> {
  /** Rust SecretStore slot. This is an identifier, never the secret value. */
  secret: string;
  placeholder?: string;
  disabled?: boolean;
  "aria-label"?: string;
  /** Reports only whether the native secret slot is empty. */
  onSecretStateChange?: (event: {
    hasValue: boolean;
  }) => void;
  onKeyDown?: (event: {
    key: string;
    preventDefault(): void;
  }) => void;
}
export interface EditorProps extends Omit<PrimitiveProps, "children"> {
  value?: string;
  /** Optional language identifier consumed by the DOM-free CodeMirror service. */
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
export type NativeWidgetConfig = object | readonly unknown[];
/**
 * Public boundary for an application-defined retained native widget.
 *
 * `tag` selects the Rust factory, `config` is its complete immutable authored
 * snapshot, and ordinary Wabou event props carry typed native events back to
 * Solid. Stateful native ownership remains keyed by this node's `NodeKey`.
 */
export interface NativeWidgetProps<Config extends NativeWidgetConfig> extends Omit<PrimitiveProps, "children" | "widgetConfig"> {
  tag: string;
  config?: Config;
}
/** A layout container. Text content should be placed in a {@link Text}. */
export declare function View(props: ViewProps): JSX.Element;
/**
 * Stable retained region with an independent native invalidation boundary.
 *
 * Use this around independently changing route content, scroll viewports,
 * overlays, native-widget regions, animation surfaces, or diagnostic HUDs.
 * It does not create application state and has the same layout semantics as a
 * View; it only limits native invalidation and materialization.
 */
export declare function ProjectionBoundary(props: ViewProps): JSX.Element;
/**
 * A measured text run that wraps within its available width by default.
 *
 * Static and reactive child text nodes are concatenated by the native host and
 * participate in the parent layout as one item. Use `maxLines={1}` or
 * `whitespace-nowrap` when the text must remain on one line.
 */
export declare function Text(props: TextProps): JSX.Element;
/**
 * One Parley paragraph assembled from explicitly styled text descendants.
 *
 * Unlike adjacent Text components, spans share wrapping, whitespace,
 * selection, and copy semantics because the native host lays them out once.
 */
export declare function RichText(props: RichTextProps): JSX.Element;
/** A text-style boundary inside RichText; it never creates a layout box. */
export declare function RichTextSpan(props: RichTextSpanProps): JSX.Element;
/** A static SVG asset rendered through the native usvg/Vello pipeline. */
export declare function Svg(props: SvgProps): JSX.Element;
/** A native Vello vector path in local logical-pixel coordinates. */
export declare function Path(props: PathProps): JSX.Element;
/** A theme-colored SVG icon with stable native sizing and semantics. */
export declare function Icon(props: IconProps): JSX.Element;
/** A replaced image node rendered by the native host. */
export declare function Image(props: ImageProps): JSX.Element;
/** A native single-line text editor with selection and scrolling. */
export declare function TextInput(props: TextInputProps): JSX.Element;
/** A native multiline text editor with wrapping, selection, and scrolling. */
export declare function TextArea(props: TextAreaProps): JSX.Element;
/** Native password editor whose value remains in a Rust SecretStore. */
export declare function PasswordInput(props: PasswordInputProps): JSX.Element;
/**
 * General-purpose native editor viewport backed by a DOM-free CodeMirror
 * document. CodeMirror owns transactions, selection and history; Rust owns
 * painting, pointer hit testing, scrolling and the platform IME bridge.
 */
export declare function Editor(props: EditorProps): JSX.Element;
/** Mount an explicitly registered Rust widget without web-element semantics. */
export declare function NativeWidget<Config extends NativeWidgetConfig = object>(props: NativeWidgetProps<Config>): JSX.Element;
//#endregion
export { type Affine2D$1 as Affine2D, PathBuilder, type VectorPath$1 as VectorPath, type VectorPathPaint, type WabouStyle$1 as WabouStyle, rotate2d$1 as rotate2d, translate2d$1 as translate2d };
//# sourceMappingURL=view.d.mts.map