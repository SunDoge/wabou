import { type Accessor, createSignal } from "solid-js";

export interface PressOptions {
  disabled?: Accessor<boolean> | boolean;
  onPress?: (event: unknown) => void;
}

export interface PressResult {
  pressed: () => boolean;
  bindings: {
    onPointerDown: () => void;
    onPointerUp: () => void;
    onPointerCancel: () => void;
    onPointerLeave: () => void;
    onClick: (event: unknown) => void;
  };
}

/** Reactive pointer-press state with disabled-aware activation. */
export function createPress(options: PressOptions = {}): PressResult {
  const [pressed, setPressed] = createSignal(false);
  const disabled = () =>
    typeof options.disabled === "function"
      ? options.disabled()
      : !!options.disabled;
  const release = () => setPressed(false);

  return {
    pressed,
    bindings: {
      onPointerDown: () => {
        if (!disabled()) setPressed(true);
      },
      onPointerUp: release,
      onPointerCancel: release,
      onPointerLeave: release,
      onClick: (event) => {
        if (!disabled()) options.onPress?.(event);
      },
    },
  };
}

export interface ActiveResult {
  active: () => boolean;
  bindings: PressResult["bindings"];
}

/** CSS `:active`-like state without an activation callback. */
export function createActive(
  disabled?: Accessor<boolean> | boolean,
): ActiveResult {
  const { pressed, bindings } = createPress({ disabled });
  return { active: pressed, bindings };
}
