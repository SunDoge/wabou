import { createLatestAsyncResource } from "@wabou/ui";
import { createEffect, createSignal, onCleanup } from "solid-js";
import {
  createDeferredWriter,
  type DeferredWriterScheduler,
} from "./deferred-writer";

export interface PersistedRecord<T extends object> {
  value(): T;
  loadError(): unknown;
  saveError(): unknown;
  update(patch: Partial<T>): void;
  reload(): Promise<T | undefined>;
  retrySave(): void;
  flush(): void;
}

export interface PersistedValue<T> {
  value(): T;
  loadError(): unknown;
  saveError(): unknown;
  set(next: T): void;
  update(update: (current: T) => T): void;
  reload(): Promise<T | undefined>;
  retrySave(): void;
  flush(): void;
}

export interface PersistedValueOptions<T> {
  initial: T;
  load(): Promise<T>;
  save(value: T): void | PromiseLike<void>;
  onLoadError(error: unknown): void;
  onSaveError(error: unknown): void;
  saveDelayMs?: number;
  scheduler?: DeferredWriterScheduler;
  equals?: (left: T, right: T) => boolean;
}

export function createPersistedValue<T>(
  options: PersistedValueOptions<T>,
): PersistedValue<T> {
  let current = options.initial;
  const [saveError, setSaveError] = createSignal<unknown>();
  const resource = createLatestAsyncResource({
    source: () => true,
    initialValue: options.initial,
    load: options.load,
    onCommit: (value) => {
      current = value;
    },
  });
  const writer = createDeferredWriter<T>({
    write: options.save,
    onError: (error) => {
      setSaveError(() => error);
      options.onSaveError(error);
    },
    delayMs: options.saveDelayMs,
    scheduler: options.scheduler,
    equals: options.equals,
  });
  createEffect(
    () => resource.error(),
    (error) => {
      if (error) options.onLoadError(error);
    },
  );
  const value = () => {
    resource.value();
    return current;
  };
  const set = (valueToSave: T) => {
    current = valueToSave;
    setSaveError(undefined);
    resource.mutate(valueToSave);
    writer.schedule(valueToSave);
  };
  const update = (updater: (current: T) => T) => set(updater(value()));
  const reload = () => resource.refresh();
  const retrySave = () => {
    setSaveError(undefined);
    writer.schedule(value());
    writer.flush();
  };
  const flush = () => writer.flush();

  onCleanup(flush);
  return {
    value,
    loadError: resource.error,
    saveError,
    set,
    update,
    reload,
    retrySave,
    flush,
  };
}

export function createPersistedRecord<T extends object>(
  options: PersistedValueOptions<T>,
): PersistedRecord<T> {
  const state = createPersistedValue(options);
  return {
    value: state.value,
    loadError: state.loadError,
    saveError: state.saveError,
    update: (patch) => state.update((current) => ({ ...current, ...patch })),
    reload: state.reload,
    retrySave: state.retrySave,
    flush: state.flush,
  };
}
