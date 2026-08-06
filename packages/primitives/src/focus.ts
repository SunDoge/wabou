import { createSignal } from "solid-js";

export interface FocusResult {
  focused: () => boolean;
  bindings: {
    onFocus: () => void;
    onBlur: () => void;
  };
}

/** Reactive focus state and event bindings for a single target. */
export function createFocus(): FocusResult {
  const [focused, setFocused] = createSignal(false);
  return {
    focused,
    bindings: {
      onFocus: () => setFocused(true),
      onBlur: () => setFocused(false),
    },
  };
}

export interface FocusWithinResult {
  focusWithin: () => boolean;
  bindings: {
    onFocusIn: () => void;
    onFocusOut: () => void;
  };
}

/** Reactive equivalent of `:focus-within`, using bubbling focus events. */
export function createFocusWithin(): FocusWithinResult {
  const [focusWithin, setFocusWithin] = createSignal(false);
  return {
    focusWithin,
    bindings: {
      onFocusIn: () => setFocusWithin(true),
      onFocusOut: () => setFocusWithin(false),
    },
  };
}
