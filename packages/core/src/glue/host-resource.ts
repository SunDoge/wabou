import { type Accessor, createSignal, getOwner, onCleanup } from "solid-js";
import {
  type HostJsonSubscriptionOptions,
  subscribeJson,
} from "./host-messages";

export interface RevisionedHostValue {
  revision: number;
}

export interface RevisionedHostPatch {
  baseRevision: number;
}

export interface RevisionedHostResourceOptions<
  T extends RevisionedHostValue,
  P extends RevisionedHostPatch,
> {
  initial: T;
  load: () => Promise<T>;
  snapshotTopic: string;
  patchTopic?: string;
  applyPatch?: (current: T, patch: P) => T | undefined;
  decodeSnapshot?: HostJsonSubscriptionOptions<T>["decode"];
  decodePatch?: HostJsonSubscriptionOptions<P>["decode"];
  onValue?: (value: T, source: "load" | "snapshot" | "patch") => void;
  onError?: (error: unknown, source: "load" | "snapshot" | "patch") => void;
  autoLoad?: boolean;
}

export interface RevisionedHostResource<T extends RevisionedHostValue> {
  value: Accessor<T>;
  loading: Accessor<boolean>;
  error: Accessor<unknown | undefined>;
  /**
   * Load an authoritative snapshot from the host.
   *
   * Transport/load failures are stored in `error()` and also reject this
   * promise so command paths cannot accidentally continue with stale state.
   * `undefined` only means the result lost a revision race or the resource was
   * already disposed.
   */
  refresh(): Promise<T | undefined>;
  waitFor(
    predicate: (value: T) => boolean,
    options?: RevisionedHostWaitOptions,
  ): Promise<T>;
  dispose(): void;
}

export interface RevisionedHostWaitOptions {
  timeout?: number;
  signal?: AbortSignal;
  /**
   * If no pushed value matches before `timeout`, perform one coalesced full
   * refresh and test the refreshed value before reporting the timeout.
   *
   * This is useful after host commands: pushes keep the common path cheap,
   * while the refresh closes over dropped/coalesced notification races.
   */
  refreshOnTimeout?: boolean;
}

export type RevisionedHostWaitErrorReason = "timeout" | "aborted" | "disposed";

export class RevisionedHostWaitError extends Error {
  readonly reason: RevisionedHostWaitErrorReason;

  constructor(reason: RevisionedHostWaitErrorReason, message: string) {
    super(message);
    this.name = "RevisionedHostWaitError";
    this.reason = reason;
  }
}

interface ValueWaiter<T> {
  predicate: (value: T) => boolean;
  resolve: (value: T) => void;
  reject: (error: Error) => void;
  timer?: ReturnType<typeof setTimeout>;
  signal?: AbortSignal;
  onAbort?: () => void;
}

/**
 * Keep a Solid value synchronized with a host-owned revisioned snapshot.
 *
 * A revision identifies the exact snapshot contents. After the first host
 * value, producers must increase it whenever those contents can change;
 * another payload with the same revision is treated as a duplicate.
 *
 * The initial RPC closes the subscription race by ignoring results older than
 * an already received host push. A patch whose base revision no longer
 * matches automatically falls back to one coalesced full refresh.
 */
export function createRevisionedHostResource<
  T extends RevisionedHostValue,
  P extends RevisionedHostPatch = RevisionedHostPatch,
>(options: RevisionedHostResourceOptions<T, P>): RevisionedHostResource<T> {
  const [value, setValue] = createSignal<T>(
    // biome-ignore lint/complexity/noBannedTypes: Solid's overload excludes callable generic values with the built-in Function type.
    options.initial as Exclude<T, Function>,
    { equals: false },
  );
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<unknown>();
  let disposed = false;
  let refreshPromise: Promise<T | undefined> | undefined;
  let hostGeneration = 0;
  // `initial` is local fallback state, so the first host value may legally
  // carry the same revision. Once any host value has been accepted, however,
  // a revision identifies that exact value and equal revisions are duplicates.
  let hasAcceptedHostValue = false;
  const waiters = new Set<ValueWaiter<T>>();

  const removeWaiter = (waiter: ValueWaiter<T>) => {
    waiters.delete(waiter);
    if (waiter.timer !== undefined) clearTimeout(waiter.timer);
    if (waiter.signal && waiter.onAbort)
      waiter.signal.removeEventListener("abort", waiter.onAbort);
  };
  const rejectWaiter = (waiter: ValueWaiter<T>, error: Error) => {
    removeWaiter(waiter);
    waiter.reject(error);
  };
  const notifyWaiters = (next: T) => {
    for (const waiter of [...waiters]) {
      let matches: boolean;
      try {
        matches = waiter.predicate(next);
      } catch (cause) {
        rejectWaiter(
          waiter,
          cause instanceof Error ? cause : new Error(String(cause)),
        );
        continue;
      }
      if (matches) {
        removeWaiter(waiter);
        waiter.resolve(next);
      }
    }
  };

  const reportError = (
    next: unknown,
    source: "load" | "snapshot" | "patch",
  ) => {
    if (disposed) return;
    setError(next);
    options.onError?.(next, source);
  };
  const accept = (
    next: T,
    source: "load" | "snapshot" | "patch",
  ): T | undefined => {
    if (disposed || next.revision < value().revision) return undefined;
    if (hasAcceptedHostValue && next.revision === value().revision) {
      // A valid duplicate still proves the transport recovered, but it must
      // not republish the value or repeat onValue side effects.
      setError(undefined);
      return value();
    }
    hasAcceptedHostValue = true;
    setError(undefined);
    // biome-ignore lint/complexity/noBannedTypes: Solid's Setter excludes callable generic values with the built-in Function type.
    setValue(next as Exclude<T, Function>);
    options.onValue?.(next, source);
    notifyWaiters(next);
    return next;
  };

  const waitForPush = (
    predicate: (value: T) => boolean,
    waitOptions: RevisionedHostWaitOptions = {},
  ): Promise<T> => {
    if (disposed)
      return Promise.reject(
        new RevisionedHostWaitError(
          "disposed",
          "revisioned host resource is disposed",
        ),
      );
    let matches: boolean;
    try {
      matches = predicate(value());
    } catch (cause) {
      return Promise.reject(
        cause instanceof Error ? cause : new Error(String(cause)),
      );
    }
    if (matches) return Promise.resolve(value());
    if (waitOptions.signal?.aborted)
      return Promise.reject(
        new RevisionedHostWaitError(
          "aborted",
          "revisioned host resource wait aborted",
        ),
      );

    return new Promise<T>((resolve, reject) => {
      const waiter: ValueWaiter<T> = {
        predicate,
        resolve,
        reject,
        signal: waitOptions.signal,
      };
      if (waitOptions.timeout !== undefined) {
        waiter.timer = setTimeout(
          () =>
            rejectWaiter(
              waiter,
              new RevisionedHostWaitError(
                "timeout",
                `revisioned host resource wait timed out after ${waitOptions.timeout}ms`,
              ),
            ),
          Math.max(0, waitOptions.timeout),
        );
      }
      if (waitOptions.signal) {
        waiter.onAbort = () =>
          rejectWaiter(
            waiter,
            new RevisionedHostWaitError(
              "aborted",
              "revisioned host resource wait aborted",
            ),
          );
        waitOptions.signal.addEventListener("abort", waiter.onAbort, {
          once: true,
        });
      }
      waiters.add(waiter);
    });
  };

  const refresh = (): Promise<T | undefined> => {
    if (disposed) return Promise.resolve(undefined);
    if (refreshPromise) return refreshPromise;
    const generationAtStart = hostGeneration;
    setLoading(true);
    refreshPromise = options
      .load()
      .then((next) => {
        if (
          hostGeneration !== generationAtStart &&
          next.revision <= value().revision
        )
          return undefined;
        return accept(next, "load");
      })
      .catch((cause: unknown) => {
        reportError(cause, "load");
        throw cause;
      })
      .finally(() => {
        refreshPromise = undefined;
        if (!disposed) setLoading(false);
      });
    return refreshPromise;
  };

  const waitFor = async (
    predicate: (value: T) => boolean,
    waitOptions: RevisionedHostWaitOptions = {},
  ): Promise<T> => {
    try {
      return await waitForPush(predicate, waitOptions);
    } catch (cause) {
      if (
        !(cause instanceof RevisionedHostWaitError) ||
        cause.reason !== "timeout" ||
        !waitOptions.refreshOnTimeout
      )
        throw cause;
      if (waitOptions.signal?.aborted)
        throw new RevisionedHostWaitError(
          "aborted",
          "revisioned host resource wait aborted",
        );
      // A refresh already in flight at the timeout boundary may have started
      // before the command whose result we are waiting for. Consume it first,
      // but do not let that stale request substitute for the post-timeout
      // consistency read.
      const existingRefresh = refreshPromise;
      if (existingRefresh) {
        try {
          await existingRefresh;
        } catch {
          // The fresh read below owns the error reported to this waiter.
        }
        if (predicate(value())) return value();
        if (waitOptions.signal?.aborted)
          throw new RevisionedHostWaitError(
            "aborted",
            "revisioned host resource wait aborted",
          );
      }
      const refreshed = await refresh();
      const current = refreshed ?? value();
      if (predicate(current)) return current;
      throw cause;
    }
  };

  const unsubscribers = [
    subscribeJson<T>(
      options.snapshotTopic,
      (next) => {
        hostGeneration++;
        accept(next, "snapshot");
      },
      {
        decode: options.decodeSnapshot,
        onError: (cause) => reportError(cause, "snapshot"),
      },
    ),
  ];
  const applyPatch = options.applyPatch;
  if (options.patchTopic && applyPatch) {
    unsubscribers.push(
      subscribeJson<P>(
        options.patchTopic,
        (patch) => {
          hostGeneration++;
          if (patch.baseRevision !== value().revision) {
            void refresh().catch(() => undefined);
            return;
          }
          try {
            const next = applyPatch(value(), patch);
            if (next) accept(next, "patch");
            else void refresh().catch(() => undefined);
          } catch (cause) {
            reportError(cause, "patch");
            void refresh().catch(() => undefined);
          }
        },
        {
          decode: options.decodePatch,
          onError: (cause) => reportError(cause, "patch"),
        },
      ),
    );
  }

  const dispose = () => {
    if (disposed) return;
    disposed = true;
    for (const unsubscribe of unsubscribers) unsubscribe();
    for (const waiter of [...waiters])
      rejectWaiter(
        waiter,
        new RevisionedHostWaitError(
          "disposed",
          "revisioned host resource is disposed",
        ),
      );
  };
  if (getOwner()) onCleanup(dispose);
  // Solid 2 treats a synchronous signal write during component construction
  // as an owned-scope violation. Subscription setup stays synchronous so no
  // host push can be missed, while the one-shot initial read starts after the
  // current component/computation has returned.
  if (options.autoLoad !== false)
    queueMicrotask(() => void refresh().catch(() => undefined));

  return { value, loading, error, refresh, waitFor, dispose };
}
