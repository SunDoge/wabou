import { type Accessor, createMemo, createSignal } from "solid-js";

export type FormDraftFieldUpdater<Value> = Value | ((previous: Value) => Value);
export type FormDraftErrors<T> = Partial<Record<keyof T, string>>;

export interface FormDraft<T extends Record<PropertyKey, unknown>> {
  value: Accessor<Readonly<T>>;
  dirty: Accessor<boolean>;
  /** Validation errors derived from the current immutable draft. */
  errors: Accessor<Readonly<FormDraftErrors<T>>>;
  valid: Accessor<boolean>;
  fieldError<Key extends keyof T>(key: Key): string | undefined;
  field<Key extends keyof T>(key: Key): T[Key];
  control<Key extends keyof T>(
    key: Key,
  ): readonly [
    Accessor<T[Key]>,
    (value: FormDraftFieldUpdater<T[Key]>) => void,
  ];
  set<Key extends keyof T>(
    key: Key,
    value: FormDraftFieldUpdater<T[Key]>,
  ): void;
  patch(value: Partial<T>): void;
  /** Restore the last baseline. */
  reset(): void;
  /** Replace both the baseline and current value. */
  resetTo(value: T): void;
  /** Make the current value the new baseline. */
  commit(): void;
}

export interface FormDraftOptions<T> {
  equals?: (left: Readonly<T>, right: Readonly<T>) => boolean;
  validate?: (value: Readonly<T>) => FormDraftErrors<T>;
}

/**
 * A small immutable draft for form fields with explicit reset and commit
 * semantics. Transient request/error state belongs outside this model.
 */
export function createFormDraft<T extends Record<PropertyKey, unknown>>(
  initial: T,
  options: FormDraftOptions<T> = {},
): FormDraft<T> {
  const [baselineBox, setBaselineBox] = createSignal({
    value: { ...initial } as T,
  });
  const [box, setBox] = createSignal({ value: { ...initial } as T });
  const value = () => box().value;
  const equals = options.equals ?? shallowEqual;
  const dirty = createMemo(() => !equals(value(), baselineBox().value));
  const errors = createMemo<Readonly<FormDraftErrors<T>>>(
    () => options.validate?.(value()) ?? ({} as FormDraftErrors<T>),
  );
  const valid = createMemo(() => Reflect.ownKeys(errors()).length === 0);
  const replace = (next: T) => setBox({ value: next });

  const resetTo = (next: T) => {
    setBaselineBox({ value: { ...next } as T });
    replace({ ...next } as T);
  };
  const setField = <Key extends keyof T>(
    key: Key,
    updater: FormDraftFieldUpdater<T[Key]>,
  ) => {
    setBox((currentBox) => {
      const current = currentBox.value;
      const previous = current[key];
      const next =
        typeof updater === "function"
          ? (updater as (value: T[Key]) => T[Key])(previous)
          : updater;
      return Object.is(previous, next)
        ? currentBox
        : { value: { ...current, [key]: next } };
    });
  };

  return {
    value,
    dirty,
    errors,
    valid,
    fieldError: (key) => errors()[key],
    field: (key) => value()[key],
    control: (key) => [() => value()[key], (next) => setField(key, next)],
    set: setField,
    patch: (patch) => {
      setBox((currentBox) => {
        const current = currentBox.value;
        for (const key of Reflect.ownKeys(patch) as (keyof T)[]) {
          if (!Object.is(current[key], patch[key])) {
            return { value: { ...current, ...patch } };
          }
        }
        return currentBox;
      });
    },
    reset: () => replace({ ...baselineBox().value } as T),
    resetTo,
    commit: () => {
      setBox((currentBox) => {
        setBaselineBox({ value: { ...currentBox.value } as T });
        return currentBox;
      });
    },
  };
}

function shallowEqual<T extends Record<PropertyKey, unknown>>(
  left: Readonly<T>,
  right: Readonly<T>,
): boolean {
  const keys = Reflect.ownKeys(left);
  if (keys.length !== Reflect.ownKeys(right).length) return false;
  return keys.every((key) => Object.is(left[key], right[key]));
}
