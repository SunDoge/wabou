import {
  type Accessor,
  createEffect,
  createSignal,
  flush,
  getOwner,
  onCleanup,
} from "solid-js";

export interface LatestAsyncResourceOptions<K, T> {
  source: Accessor<K | undefined>;
  load: (key: K, context: { signal: AbortSignal }) => Promise<T>;
  initialValue?: T;
  retainPrevious?: boolean;
  autoLoad?: boolean;
  /** Runs synchronously before a latest load or local mutation is published. */
  onCommit?: (value: T) => void;
}

export type LatestAsyncResourceStatus = "idle" | "pending" | "ready" | "error";

export interface LatestAsyncResource<T> {
  value: Accessor<T | undefined>;
  loading: Accessor<boolean>;
  error: Accessor<unknown | undefined>;
  status: Accessor<LatestAsyncResourceStatus>;
  /**
   * Start a latest-wins load for the current source.
   *
   * Load failures are represented by `status() === "error"` and `error()`;
   * they do not reject this promise. `undefined` means the load failed, was
   * superseded/aborted, or there is no active source.
   */
  refresh(): Promise<T | undefined>;
  mutate(value: T): void;
  dispose(): void;
}

/**
 * Load the latest reactive key while exposing ordinary, non-suspending state.
 * Older requests are aborted when possible and can never overwrite newer data.
 */
export function createLatestAsyncResource<K, T>(
  options: LatestAsyncResourceOptions<K, T>,
): LatestAsyncResource<T> {
  const initialBox = Object.hasOwn(options, "initialValue")
    ? { value: options.initialValue as T }
    : undefined;
  const [valueBox, setValueBox] = createSignal<{ value: T } | undefined>(
    initialBox,
  );
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<unknown>();
  const [status, setStatus] = createSignal<LatestAsyncResourceStatus>("idle");
  let currentKey: K | undefined;
  let generation = 0;
  let controller: AbortController | undefined;
  let disposed = false;

  const refresh = async (): Promise<T | undefined> => {
    const key = currentKey;
    if (disposed || key === undefined) return undefined;

    const request = ++generation;
    controller?.abort();
    controller = new AbortController();
    const signal = controller.signal;
    setLoading(true);
    setError(undefined);
    setStatus("pending");
    try {
      const next = await options.load(key, { signal });
      if (disposed || request !== generation) return undefined;
      flush(() => {
        options.onCommit?.(next);
        setValueBox({ value: next });
        setStatus("ready");
      });
      return next;
    } catch (cause) {
      if (disposed || request !== generation || signal.aborted)
        return undefined;
      flush(() => {
        setError(cause);
        setStatus("error");
      });
      return undefined;
    } finally {
      if (!disposed && request === generation) {
        controller = undefined;
        flush(() => setLoading(false));
      }
    }
  };

  createEffect(options.source, (key) => {
    if (Object.is(key, currentKey)) return;
    generation++;
    controller?.abort();
    controller = undefined;
    currentKey = key;
    setError(undefined);
    setLoading(false);
    setStatus("idle");
    if (!options.retainPrevious) setValueBox(initialBox);
    if (key !== undefined && options.autoLoad !== false) void refresh();
  });

  const mutate = (next: T) => {
    if (disposed) return;
    generation++;
    controller?.abort();
    controller = undefined;
    options.onCommit?.(next);
    setValueBox({ value: next });
    setError(undefined);
    setLoading(false);
    setStatus("ready");
  };

  const dispose = () => {
    if (disposed) return;
    disposed = true;
    generation++;
    controller?.abort();
    controller = undefined;
  };
  if (getOwner()) onCleanup(dispose);

  return {
    value: () => valueBox()?.value,
    loading,
    error,
    status,
    refresh,
    mutate,
    dispose,
  };
}
