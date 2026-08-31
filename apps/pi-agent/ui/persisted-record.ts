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

export function createPersistedRecord<T extends object>(options: {
  initial: T;
  load(): Promise<T>;
  save(value: T): void | PromiseLike<void>;
  onLoadError(error: unknown): void;
  onSaveError(error: unknown): void;
  saveDelayMs?: number;
  scheduler?: DeferredWriterScheduler;
}): PersistedRecord<T> {
  const [saveError, setSaveError] = createSignal<unknown>();
  const resource = createLatestAsyncResource({
    source: () => true,
    initialValue: options.initial,
    load: options.load,
  });
  const writer = createDeferredWriter<T>({
    write: options.save,
    onError: (error) => {
      setSaveError(() => error);
      options.onSaveError(error);
    },
    delayMs: options.saveDelayMs,
    scheduler: options.scheduler,
  });
  createEffect(
    () => resource.error(),
    (error) => {
      if (error) options.onLoadError(error);
    },
  );

  const value = () => resource.value() ?? options.initial;
  const update = (patch: Partial<T>) => {
    const next = { ...value(), ...patch };
    setSaveError(undefined);
    resource.mutate(next);
    writer.schedule(next);
  };
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
    update,
    reload,
    retrySave,
    flush,
  };
}
