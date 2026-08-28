export interface DeferredWriterScheduler {
  set(callback: () => void, delayMs: number): unknown;
  clear(handle: unknown): void;
}

export interface DeferredWriter<T> {
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
}): DeferredWriter<T> {
  const scheduler = options.scheduler ?? systemScheduler;
  let timer: unknown;
  let pending: { value: T } | undefined;

  const writePending = () => {
    timer = undefined;
    const next = pending;
    pending = undefined;
    if (!next) return;
    try {
      Promise.resolve(options.write(next.value)).catch(options.onError);
    } catch (error) {
      options.onError(error);
    }
  };

  return {
    schedule(value) {
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
    },
  };
}
