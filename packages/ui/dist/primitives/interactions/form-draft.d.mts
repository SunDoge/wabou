import { Accessor } from "solid-js";
//#region src/primitives/interactions/form-draft.d.ts
export type FormDraftFieldUpdater<Value> = Value | ((previous: Value) => Value);
/** Validation key used for errors that do not belong to one field. */
export declare const FORM_ERROR: unique symbol;
export type FormDraftErrors<T> = Partial<Record<keyof T | typeof FORM_ERROR, string>>;
export interface FormDraft<T extends Record<PropertyKey, unknown>> {
  value: Accessor<Readonly<T>>;
  dirty: Accessor<boolean>;
  /** Validation errors derived from the current immutable draft. */
  errors: Accessor<Readonly<FormDraftErrors<T>>>;
  valid: Accessor<boolean>;
  formError: Accessor<string | undefined>;
  fieldError<Key extends keyof T>(key: Key): string | undefined;
  field<Key extends keyof T>(key: Key): T[Key];
  control<Key extends keyof T>(key: Key): readonly [Accessor<T[Key]>, (value: FormDraftFieldUpdater<T[Key]>) => void];
  set<Key extends keyof T>(key: Key, value: FormDraftFieldUpdater<T[Key]>): void;
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
export declare function createFormDraft<T extends Record<PropertyKey, unknown>>(initial: T, options?: FormDraftOptions<T>): FormDraft<T>;
//#endregion
//# sourceMappingURL=form-draft.d.mts.map