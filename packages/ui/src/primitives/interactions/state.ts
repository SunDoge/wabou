import { type Accessor, createSignal } from "solid-js";

export interface ControllableStateOptions<T> {
  value: () => T | undefined;
  defaultValue: T;
  disabled?: () => boolean;
  onChange?: (value: T) => void;
}

export interface ControllableState<T> {
  value: Accessor<T>;
  set(value: T): boolean;
}

export function createControllableState<T>(
  options: ControllableStateOptions<T>,
): ControllableState<T> {
  // Solid 2 treats a function passed to createSignal as a computation. Keep
  // the generic value in an object so both ordinary and function-valued state
  // remain writable without relying on setter/update-function ambiguity.
  // Controllable state is intentionally writable from host events and public
  // state-machine commands, which can run outside the component owner's
  // reactive scope.
  const [local, setLocal] = createSignal(
    { value: options.defaultValue },
    { ownedWrite: true },
  );
  const value = () => options.value() ?? local().value;
  return {
    value,
    set(next) {
      if (options.disabled?.() || Object.is(value(), next)) return false;
      if (options.value() === undefined) setLocal({ value: next });
      options.onChange?.(next);
      return true;
    },
  };
}
