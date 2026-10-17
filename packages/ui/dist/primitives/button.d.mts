import { WabouClassList, WabouStyle } from "./view.mjs";
import { Accessor, JSX } from "solid-js";
import { Handle, WabouElementProps } from "@wabou/core/renderer";
//#region src/primitives/button.d.ts
declare const ACCENTS: {
  readonly neutral: "#475569";
  readonly sky: "#0284c7";
  readonly amber: "#d97706";
};
export interface ButtonProps extends Pick<WabouElementProps, "aria-busy" | "aria-checked" | "aria-controls" | "aria-current" | "aria-expanded" | "aria-haspopup" | "aria-label" | "aria-pressed" | "aria-selected" | "aria-valuetext" | "id" | "role" | "focusOrder" | "onBlur" | "onContextMenu" | "onDblClick" | "onFocus" | "onFocusIn" | "onFocusOut" | "onKeyUp" | "onPointerCancel" | "onPointerDown" | "onPointerEnter" | "onPointerLeave" | "onPointerMove" | "onPointerOut" | "onPointerOver" | "onPointerUp" | "onWheel"> {
  class?: string | ((state: ButtonState) => string);
  classList?: WabouClassList | ((state: ButtonState) => WabouClassList);
  style?: WabouStyle | ((state: ButtonState) => WabouStyle);
  children?: JSX.Element;
  /** Render an internal visual layer from normalized interaction state. */
  /**
   * Build the button's persistent visual subtree once. Read the supplied
   * accessor inside reactive props to style it from interaction state.
   *
   * Passing an accessor is intentional: rebuilding this subtree on every
   * pointer transition can destroy the native gesture target between down
   * and up.
   */
  renderContent?: (state: Accessor<ButtonState>) => JSX.Element;
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
  code?: string;
  /** Physical Shift, Control, Alt, and Meta modifier bits. */
  mods?: number;
  /** Whether the physical modifiers form this platform's primary chord. */
  primary?: boolean;
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
export declare function createButton(options?: CreateButtonOptions): ButtonPrimitive;
/**
 * A native button with consistent hover, pressed, focus and disabled feedback.
 *
 * Interaction styling is deliberately implemented with reactive inline styles:
 * applications do not need CSS pseudo-class support to get a responsive button.
 */
export declare function Button(props: ButtonProps): JSX.Element;
/**
 * An explicit external-link interaction.
 *
 * Wabou does not assign browser behavior to an `a` tag or `href` attribute;
 * the JS primitive owns activation while Rust only executes `openUrl`.
 */
export declare function Link(props: LinkProps): JSX.Element;
//#endregion
export {};
//# sourceMappingURL=button.d.mts.map