import type { Handle, WabouElementProps } from "@wabou/core/renderer";
import { useHost } from "@wabou/core/renderer";
import { type Accessor, type JSX, untrack } from "solid-js";
import { createFocus } from "./focus";
import { createHover } from "./hover";
import { createPress } from "./press";
import type { WabouClassList, WabouStyle } from "./view";

const ACCENTS = {
  neutral: "#475569",
  sky: "#0284c7",
  amber: "#d97706",
} as const;

export interface ButtonProps
  extends Pick<
    WabouElementProps,
    | "aria-checked"
    | "aria-controls"
    | "aria-current"
    | "aria-expanded"
    | "aria-haspopup"
    | "aria-label"
    | "aria-pressed"
    | "aria-selected"
    | "role"
    | "focusOrder"
    | "onBlur"
    | "onContextMenu"
    | "onDblClick"
    | "onFocus"
    | "onFocusIn"
    | "onFocusOut"
    | "onKeyUp"
    | "onPointerCancel"
    | "onPointerDown"
    | "onPointerEnter"
    | "onPointerLeave"
    | "onPointerMove"
    | "onPointerOut"
    | "onPointerOver"
    | "onPointerUp"
    | "onWheel"
  > {
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
  ref?: (node: Handle) => void;
  onKeyDown?: (event: ButtonKeyEvent) => void;
  onClick?: (event: ButtonEvent) => void;
}

export interface LinkProps extends ButtonProps {
  /** URL passed explicitly to the native shell when the link is activated. */
  url: string;
  role?: never;
}

export interface ButtonEvent {
  readonly defaultPrevented?: boolean;
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
  /** Keyboard-visible focus, separate from focus retained after a click. */
  focusVisible: boolean;
  selected: boolean;
  disabled: boolean;
}

export function resolveButtonFocusOrder(
  disabled: boolean,
  focusOrder: number | undefined,
): number {
  return disabled ? -1 : (focusOrder ?? 0);
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
    onFocus: (event?: { payload?: { focusVisible?: boolean } }) => void;
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
      focusVisible: focus.focusVisible(),
      selected: selected(),
      disabled: disabled(),
    }),
    bindings: {
      onPointerEnter: hover.bindings.onPointerEnter,
      onPointerLeave: () => {
        hover.bindings.onPointerLeave();
        press.bindings.onPointerLeave();
      },
      onPointerDown: () => {
        focus.pointerModality();
        press.bindings.onPointerDown();
      },
      onPointerUp: press.bindings.onPointerUp,
      onPointerCancel: press.bindings.onPointerCancel,
      onFocus: focus.bindings.onFocus,
      onBlur: focus.bindings.onBlur,
      onClick: press.bindings.onClick as (event: ButtonEvent) => void,
      onKeyDown: (event) => {
        focus.keyboardModality();
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
  const forwardedRef = untrack(() => props.ref);
  const refProps = forwardedRef ? { ref: forwardedRef } : {};
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
  const structuralStyle = (): WabouStyle => ({
    display: "flex",
    "align-items": "center",
    "flex-shrink": 0,
    "white-space": "nowrap",
    "user-select": props.selectable ? "text" : "none",
    cursor: disabled() ? "not-allowed" : "pointer",
    "outline-width": state().focusVisible ? "2px" : "0px",
    "outline-offset": "2px",
    "outline-color": "#38bdf8",
    "outline-style": "solid",
  });
  const defaultStyle = (): WabouStyle => {
    // `unstyled` removes the opinionated skin, not the structural button
    // layout. Native Wabou elements intentionally have no browser UA
    // stylesheet, so a bare button would otherwise place its text at the
    // start of the content box instead of centering it vertically.
    if (props.unstyled) return structuralStyle();
    return {
      ...structuralStyle(),
      "justify-content": "center",
      "min-height": "32px",
      padding: "6px 12px",
      "border-radius": "6px",
      // Keep focus styling paint-only so focus cannot move the label.
      "border-width": "1px",
      "border-color": state().focusVisible ? "#7dd3fc" : "#64748b",
      "background-color": background(),
      color: "#f8fafc",
      opacity: disabled() ? 0.45 : 1,
    };
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
      {...refProps}
      disabled={disabled()}
      aria-disabled={disabled()}
      focusOrder={resolveButtonFocusOrder(disabled(), props.focusOrder)}
      role={props.role ?? "button"}
      aria-haspopup={props["aria-haspopup"]}
      aria-expanded={props["aria-expanded"]}
      aria-controls={props["aria-controls"]}
      aria-label={props["aria-label"]}
      aria-checked={props["aria-checked"]}
      aria-current={props["aria-current"]}
      aria-selected={props["aria-selected"]}
      aria-pressed={props["aria-pressed"]}
      class={
        typeof props.class === "function" ? props.class(state()) : props.class
      }
      classList={
        typeof props.classList === "function"
          ? props.classList(state())
          : props.classList
      }
      style={{
        ...defaultStyle(),
        ...customStyle(),
      }}
      onPointerEnter={(event) => {
        primitive.bindings.onPointerEnter();
        props.onPointerEnter?.(event);
      }}
      onPointerLeave={(event) => {
        primitive.bindings.onPointerLeave();
        props.onPointerLeave?.(event);
      }}
      onPointerDown={(event) => {
        primitive.bindings.onPointerDown();
        props.onPointerDown?.(event);
      }}
      onPointerUp={(event) => {
        primitive.bindings.onPointerUp();
        props.onPointerUp?.(event);
      }}
      onPointerCancel={(event) => {
        primitive.bindings.onPointerCancel();
        props.onPointerCancel?.(event);
      }}
      onPointerMove={props.onPointerMove}
      onPointerOver={props.onPointerOver}
      onPointerOut={props.onPointerOut}
      onFocus={(event) => {
        primitive.bindings.onFocus();
        props.onFocus?.(event);
      }}
      onBlur={(event) => {
        primitive.bindings.onBlur();
        props.onBlur?.(event);
      }}
      onFocusIn={props.onFocusIn}
      onFocusOut={props.onFocusOut}
      onClick={primitive.bindings.onClick}
      onContextMenu={props.onContextMenu}
      onDblClick={props.onDblClick}
      onKeyDown={primitive.bindings.onKeyDown}
      onKeyUp={props.onKeyUp}
      onWheel={props.onWheel}
    >
      {props.children}
    </button>
  );
}

/**
 * An explicit external-link interaction.
 *
 * Wabou does not assign browser behavior to an `a` tag or `href` attribute;
 * the JS primitive owns activation while Rust only executes `openUrl`.
 */
export function Link(props: LinkProps): JSX.Element {
  const host = useHost();
  return (
    <Button
      {...props}
      role="link"
      onClick={(event) => {
        props.onClick?.(event);
        if (!event.defaultPrevented) host.system.openUrl(props.url);
      }}
    />
  );
}
