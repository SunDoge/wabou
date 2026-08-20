import { type Accessor, createSignal, getOwner, onCleanup } from "solid-js";

export type AsyncActionResult<T> =
  | { ok: true; value: T }
  | { ok: false; error: unknown };

/** A concurrent call tried to replace the arguments of an in-flight action. */
export class AsyncActionConflictError extends Error {
  constructor() {
    super(
      "async action is already running with different arguments; use a keyed action for independent operations",
    );
    this.name = "AsyncActionConflictError";
  }
}

export interface AsyncAction<Args extends unknown[], T> {
  pending: Accessor<boolean>;
  error: Accessor<unknown | undefined>;
  run(...args: Args): Promise<AsyncActionResult<T>>;
  reset(): void;
}

export interface KeyedAsyncAction<Key, Args extends unknown[], T> {
  pendingKeys: Accessor<ReadonlySet<Key>>;
  pending(key: Key): boolean;
  error(key: Key): unknown | undefined;
  run(...args: Args): Promise<AsyncActionResult<T>>;
  reset(key: Key): void;
  resetAll(): void;
}

/**
 * Run an imperative async operation as a single flight with explicit state.
 * Repeated calls with the same argument identities join the pending operation.
 * A call with different arguments returns [`AsyncActionConflictError`] rather
 * than silently discarding those arguments. Use `createKeyedAsyncAction` when
 * independently keyed operations should run concurrently.
 */
export function createAsyncAction<Args extends unknown[], T>(
  action: (...args: Args) => PromiseLike<T> | T,
): AsyncAction<Args, T> {
  const [pending, setPending] = createSignal(false);
  const [error, setError] = createSignal<unknown>();
  let disposed = false;
  let inFlight: Promise<AsyncActionResult<T>> | undefined;
  let inFlightArgs: Args | undefined;

  const run = (...args: Args): Promise<AsyncActionResult<T>> => {
    if (disposed)
      return Promise.resolve({
        ok: false,
        error: new Error("cannot run a disposed async action"),
      });
    if (inFlight) {
      if (sameArguments(inFlightArgs, args)) return inFlight;
      return Promise.resolve({
        ok: false,
        error: new AsyncActionConflictError(),
      });
    }
    setPending(true);
    setError(undefined);
    inFlightArgs = args;
    inFlight = Promise.resolve()
      .then(() => action(...args))
      .then(
        (value): AsyncActionResult<T> => ({ ok: true, value }),
        (cause): AsyncActionResult<T> => {
          if (!disposed) setError(cause);
          return { ok: false, error: cause };
        },
      )
      .finally(() => {
        inFlight = undefined;
        inFlightArgs = undefined;
        if (!disposed) setPending(false);
      });
    return inFlight;
  };

  const reset = () => {
    if (!disposed) setError(undefined);
  };
  if (getOwner())
    onCleanup(() => {
      disposed = true;
      setPending(false);
      setError(undefined);
    });

  return { pending, error, run, reset };
}

function sameArguments<Args extends unknown[]>(
  previous: Args | undefined,
  next: Args,
): boolean {
  return (
    previous !== undefined &&
    previous.length === next.length &&
    previous.every((value, index) => Object.is(value, next[index]))
  );
}

/**
 * Run one async single-flight per stable key. Calls for the same key join the
 * existing operation, while unrelated keys remain independently concurrent.
 */
export function createKeyedAsyncAction<Key, Args extends unknown[], T>(
  keyOf: (...args: Args) => Key,
  action: (...args: Args) => PromiseLike<T> | T,
): KeyedAsyncAction<Key, Args, T> {
  const [pendingKeys, setPendingKeys] = createSignal<ReadonlySet<Key>>(
    new Set(),
  );
  const [errors, setErrors] = createSignal<ReadonlyMap<Key, unknown>>(
    new Map(),
  );
  const inFlight = new Map<Key, Promise<AsyncActionResult<T>>>();
  let disposed = false;

  const run = (...args: Args): Promise<AsyncActionResult<T>> => {
    if (disposed)
      return Promise.resolve({
        ok: false,
        error: new Error("cannot run a disposed keyed async action"),
      });
    let key: Key;
    try {
      key = keyOf(...args);
    } catch (error) {
      return Promise.resolve({ ok: false, error });
    }
    const existing = inFlight.get(key);
    if (existing) return existing;

    setPendingKeys((current) => new Set([...current, key]));
    setErrors((current) => {
      if (!current.has(key)) return current;
      const next = new Map(current);
      next.delete(key);
      return next;
    });
    const request = Promise.resolve()
      .then(() => action(...args))
      .then(
        (value): AsyncActionResult<T> => ({ ok: true, value }),
        (cause): AsyncActionResult<T> => {
          if (!disposed)
            setErrors((current) => new Map(current).set(key, cause));
          return { ok: false, error: cause };
        },
      )
      .finally(() => {
        inFlight.delete(key);
        if (!disposed)
          setPendingKeys((current) => {
            if (!current.has(key)) return current;
            const next = new Set(current);
            next.delete(key);
            return next;
          });
      });
    inFlight.set(key, request);
    return request;
  };

  const reset = (key: Key) => {
    if (disposed) return;
    setErrors((current) => {
      if (!current.has(key)) return current;
      const next = new Map(current);
      next.delete(key);
      return next;
    });
  };
  const resetAll = () => {
    if (!disposed) setErrors(new Map());
  };
  if (getOwner())
    onCleanup(() => {
      disposed = true;
      inFlight.clear();
      setPendingKeys(new Set<Key>());
      setErrors(new Map<Key, unknown>());
    });

  return {
    pendingKeys,
    pending: (key) => pendingKeys().has(key),
    error: (key) => errors().get(key),
    run,
    reset,
    resetAll,
  };
}
