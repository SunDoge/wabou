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
  const [local, setLocal] = createSignal<T>(() => options.defaultValue);
  const value = () => options.value() ?? local();
  return {
    value,
    set(next) {
      if (options.disabled?.() || Object.is(value(), next)) return false;
      if (options.value() === undefined) setLocal(() => next);
      options.onChange?.(next);
      return true;
    },
  };
}
