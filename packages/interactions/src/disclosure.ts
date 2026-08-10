import { match } from "ts-pattern";
import { unchanged, type UpdateResult } from "./machine";
import { createControllableState } from "./state";

export type DisclosureEvent =
  | { type: "OPEN" }
  | { type: "CLOSE" }
  | { type: "TOGGLE" }
  | { type: "DISABLED"; disabled: boolean };
export interface DisclosureState {
  open: boolean;
  disabled: boolean;
}

export function updateDisclosure(
  state: DisclosureState,
  event: DisclosureEvent,
): UpdateResult<DisclosureState> {
  return match(event)
    .with({ type: "DISABLED" }, ({ disabled }) => ({
      state: { ...state, disabled, open: disabled ? false : state.open },
      commands: [],
    }))
    .with({ type: "OPEN" }, () =>
      state.disabled || state.open
        ? unchanged(state)
        : { state: { ...state, open: true }, commands: [] },
    )
    .with({ type: "CLOSE" }, () =>
      !state.open
        ? unchanged(state)
        : { state: { ...state, open: false }, commands: [] },
    )
    .with({ type: "TOGGLE" }, () =>
      state.disabled
        ? unchanged(state)
        : { state: { ...state, open: !state.open }, commands: [] },
    )
    .exhaustive();
}

export interface DisclosureOptions {
  open?: () => boolean | undefined;
  defaultOpen?: boolean;
  disabled?: () => boolean;
  onOpenChange?: (open: boolean) => void;
}
export function createDisclosure(options: DisclosureOptions = {}) {
  const controlled = createControllableState({
    value: options.open ?? (() => undefined),
    defaultValue: options.defaultOpen ?? false,
    disabled: options.disabled,
    onChange: options.onOpenChange,
  });
  const set = (type: "OPEN" | "CLOSE" | "TOGGLE") => {
    const result = updateDisclosure(
      { open: controlled.value(), disabled: options.disabled?.() ?? false },
      { type },
    );
    return controlled.set(result.state.open);
  };
  return {
    open: controlled.value,
    disabled: () => options.disabled?.() ?? false,
    openDisclosure: () => set("OPEN"),
    close: () => set("CLOSE"),
    toggle: () => set("TOGGLE"),
  };
}
