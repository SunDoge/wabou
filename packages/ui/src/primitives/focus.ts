import { createSignal } from "solid-js";

// Input modality belongs to the application, not to one control: Tab is
// received by the old target before native focus moves to the next target.
let keyboardModality = false;

export interface FocusResult {
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

export interface FocusEvent {
  /** Native input-modality hint. Styling remains owned by the JS primitive. */
  payload?: { focusVisible?: boolean };
}

/** Reactive focus state and event bindings for a single target. */
export function createFocus(): FocusResult {
  const [focused, setFocused] = createSignal(false);
  const [focusVisible, setFocusVisible] = createSignal(false);
  return {
    focused,
    focusVisible,
    pointerModality: () => {
      keyboardModality = false;
      // Native focus is assigned before pointerdown is dispatched. Clear a
      // keyboard ring immediately when that pointerdown reaches JavaScript.
      if (focused()) setFocusVisible(false);
    },
    keyboardModality: () => {
      keyboardModality = true;
      if (focused()) setFocusVisible(true);
    },
    bindings: {
      onFocus: (event) => {
        setFocused(true);
        setFocusVisible(event?.payload?.focusVisible ?? keyboardModality);
      },
      onBlur: () => {
        setFocused(false);
        setFocusVisible(false);
      },
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
