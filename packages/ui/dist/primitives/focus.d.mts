//#region src/primitives/focus.d.ts
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
  payload?: {
    focusVisible?: boolean;
  };
}
/** Reactive focus state and event bindings for a single target. */
export declare function createFocus(): FocusResult;
export interface FocusWithinResult {
  focusWithin: () => boolean;
  bindings: {
    onFocusIn: () => void;
    onFocusOut: () => void;
  };
}
/** Reactive equivalent of `:focus-within`, using bubbling focus events. */
export declare function createFocusWithin(): FocusWithinResult;
//#endregion
//# sourceMappingURL=focus.d.mts.map