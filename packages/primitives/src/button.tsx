import type { Handle } from "@wabou/solid-renderer";
import type { JSX as WebJSX } from "@solidjs/web";
import type { Accessor, JSX } from "solid-js";
import { createFocus } from "./focus";
import { createHover } from "./hover";
import { createPress } from "./press";
import type { WabouClassList, WabouStyle } from "./view";

const ACCENTS = {
  neutral: "#475569",
  sky: "#0284c7",
  amber: "#d97706",
} as const;

export interface ButtonProps {
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
  role?: WebJSX.ButtonHTMLAttributes<HTMLButtonElement>["role"];
  ref?: (node: Handle) => void;
  "aria-haspopup"?:
    | boolean
    | "false"
    | "true"
    | "menu"
    | "listbox"
    | "tree"
    | "grid"
    | "dialog";
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

export interface ButtonEvent {
  stopPropagation(): void;
  preventDefault(): void;
}

export interface ButtonKeyEvent extends ButtonEvent {
  key: string;
  repeat?: boolean;
  readonly defaultPrevented?: boolean;
}

export interface ButtonState {
  hovered: boolean;
  pressed: boolean;
  focused: boolean;
  selected: boolean;
  disabled: boolean;
}

export interface CreateButtonOptions {
  disabled?: Accessor<boolean> | boolean;
  selected?: Accessor<boolean> | boolean;
  onPress?: (event: ButtonEvent) => void;
  onKeyDown?: (event: ButtonKeyEvent) => void;
}

export interface ButtonPrimitive {
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
export function createButton(
  options: CreateButtonOptions = {},
): ButtonPrimitive {
  const hover = createHover();
  const focus = createFocus();
  const disabled = () =>
    typeof options.disabled === "function"
      ? options.disabled()
      : (options.disabled ?? false);
  const selected = () =>
    typeof options.selected === "function"
      ? options.selected()
      : (options.selected ?? false);
  const press = createPress({
    disabled,
    onPress: (event) => options.onPress?.(event as ButtonEvent),
  });
  return {
    state: () => ({
      hovered: hover.hovered(),
      pressed: press.pressed(),
      focused: focus.focused(),
      selected: selected(),
      disabled: disabled(),
    }),
    bindings: {
      onPointerEnter: hover.bindings.onPointerEnter,
      onPointerLeave: () => {
        hover.bindings.onPointerLeave();
        press.bindings.onPointerLeave();
      },
      onPointerDown: press.bindings.onPointerDown,
      onPointerUp: press.bindings.onPointerUp,
      onPointerCancel: press.bindings.onPointerCancel,
      onFocus: focus.bindings.onFocus,
      onBlur: focus.bindings.onBlur,
      onClick: press.bindings.onClick as (event: ButtonEvent) => void,
      onKeyDown: (event) => {
        options.onKeyDown?.(event);
        if (event.defaultPrevented || event.repeat) return;
        if (event.key !== "Enter" && event.key !== " ") return;
        event.preventDefault();
        if (!disabled()) options.onPress?.(event);
      },
    },
  };
}

/**
 * A native button with consistent hover, pressed, focus and disabled feedback.
 *
 * Interaction styling is deliberately implemented with reactive inline styles:
 * applications do not need CSS pseudo-class support to get a responsive button.
 */
export function Button(props: ButtonProps): JSX.Element {
  const disabled = () => props.disabled ?? false;
  const primitive = createButton({
    disabled,
    selected: () => props.selected ?? false,
    onPress: (event) => props.onClick?.(event),
    onKeyDown: (event) => props.onKeyDown?.(event),
  });
  const variant = () => props.variant ?? "solid";
  const accent = () => ACCENTS[props.tone ?? "neutral"];
  const state = primitive.state;
  const customStyle = () =>
    typeof props.style === "function" ? props.style(state()) : props.style;
  const defaultStyle = (): WabouStyle =>
    props.unstyled
      ? {
          // `unstyled` removes the opinionated skin, not the structural button
          // layout. Native Wabou elements intentionally have no browser UA
          // stylesheet, so a bare button would otherwise place its text at the
          // start of the content box instead of centering it vertically.
          display: "flex",
          "align-items": "center",
          "flex-shrink": 0,
          "white-space": "nowrap",
          "user-select": props.selectable ? "text" : "none",
          cursor: disabled() ? "not-allowed" : "pointer",
          "outline-width": state().focused ? "2px" : "0px",
          "outline-offset": "2px",
          "outline-color": "#38bdf8",
          "outline-style": "solid",
        }
      : {
          display: "flex",
          "align-items": "center",
          "justify-content": "center",
          "flex-shrink": 0,
          "white-space": "nowrap",
          "user-select": props.selectable ? "text" : "none",
          cursor: disabled() ? "not-allowed" : "pointer",
          "outline-width": state().focused ? "2px" : "0px",
          "outline-offset": "2px",
          "outline-color": "#38bdf8",
          "outline-style": "solid",
          "min-height": "32px",
          padding: "6px 12px",
          "border-radius": "6px",
          // Keep focus styling paint-only so focus cannot move the label.
          "border-width": "1px",
          "border-color": state().focused ? "#7dd3fc" : "#64748b",
          "background-color": background(),
          color: "#f8fafc",
          opacity: disabled() ? 0.45 : 1,
        };

  const background = () => {
    if (variant() === "ghost" && !props.selected) {
      if (state().pressed) return "#1e293b";
      return state().hovered ? "#334155" : "transparent";
    }
    if (state().pressed) return "#1e293b";
    if (state().hovered && !props.selected) return "#334155";
    return accent();
  };

  return (
    // biome-ignore lint/a11y/useAriaPropsSupportedByRole: headless controls replace the default button role at runtime.
    <button
      type="button"
      disabled={disabled()}
      title={props.title}
      role={props.role}
      ref={props.ref as never}
      aria-haspopup={props["aria-haspopup"] as never}
      aria-expanded={props["aria-expanded"] as never}
      aria-controls={props["aria-controls"]}
      aria-label={props["aria-label"]}
      aria-checked={props["aria-checked"] as never}
      aria-selected={props["aria-selected"] as never}
      aria-pressed={props["aria-pressed"] as never}
      class={
        typeof props.class === "function" ? props.class(state()) : props.class
      }
      classList={
        typeof props.classList === "function"
          ? props.classList(state())
          : props.classList
      }
      style={
        {
          ...defaultStyle(),
          ...customStyle(),
        } as unknown as JSX.CSSProperties
      }
      {...primitive.bindings}
    >
      {props.children}
    </button>
  );
}
