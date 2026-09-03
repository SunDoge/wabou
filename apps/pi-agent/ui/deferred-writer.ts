export interface DeferredWriterScheduler {
  set(callback: () => void, delayMs: number): unknown;
  clear(handle: unknown): void;
}

export interface DeferredWriter<T> {
  /** Mark a value as already durable without scheduling a write. */
  prime(value: T): void;
  schedule(value: T): void;
  flush(): void;
  cancel(): void;
}

const systemScheduler: DeferredWriterScheduler = {
  set: (callback, delayMs) => setTimeout(callback, delayMs),
  clear: (handle) => clearTimeout(handle as ReturnType<typeof setTimeout>),
};

export function createDeferredWriter<T>(options: {
  write: (value: T) => void | PromiseLike<void>;
  onError: (error: unknown) => void;
  delayMs?: number;
  scheduler?: DeferredWriterScheduler;
  equals?: (left: T, right: T) => boolean;
}): DeferredWriter<T> {
  const scheduler = options.scheduler ?? systemScheduler;
  let timer: unknown;
  let pending: { value: T } | undefined;
  let accepted: { value: T } | undefined;

  const clearAcceptedIfCurrent = (value: T) => {
    if (
      accepted &&
      (!options.equals || options.equals(accepted.value, value))
    ) {
      accepted = undefined;
    }
  };

  const writePending = () => {
    timer = undefined;
    const next = pending;
    pending = undefined;
    if (!next) return;
    try {
      Promise.resolve(options.write(next.value)).catch((error) => {
        clearAcceptedIfCurrent(next.value);
        options.onError(error);
      });
    } catch (error) {
      clearAcceptedIfCurrent(next.value);
      options.onError(error);
    }
  };

  return {
    prime(value) {
      if (timer !== undefined) scheduler.clear(timer);
      timer = undefined;
      pending = undefined;
      accepted = { value };
    },
    schedule(value) {
      if (options.equals && accepted && options.equals(accepted.value, value)) {
        return;
      }
      accepted = { value };
      pending = { value };
      if (timer !== undefined) scheduler.clear(timer);
      timer = scheduler.set(writePending, options.delayMs ?? 150);
    },
    flush() {
      if (timer !== undefined) scheduler.clear(timer);
      writePending();
    },
    cancel() {
      if (timer !== undefined) scheduler.clear(timer);
      timer = undefined;
      pending = undefined;
      accepted = undefined;
    },
  };
}
