import {
  type Accessor,
  createMemo,
  latest,
  onCleanup,
  refresh,
  resolve,
  type SourceAccessor,
} from "solid-js";

export interface AsyncQueryOptions<K, T> {
  source: Accessor<K | undefined>;
  load: (key: K, context: { signal: AbortSignal }) => Promise<T>;
  initialValue?: T;
}

export interface AsyncQuery<T> {
  /** Read the current result, suspending through Solid's nearest Loading boundary. */
  value: SourceAccessor<T | undefined>;
  /** Read the last settled result while a replacement is loading. */
  latest: Accessor<T | undefined>;
  /** Re-run the query for its current key and await the settled result. */
  refresh(): Promise<T | undefined>;
}

/**
 * Create a latest-wins query using Solid 2's native async graph.
 *
 * Promise ownership, stale-result suppression, pending propagation, and error
 * propagation belong to Solid. Wabou only adds AbortSignal lifecycle and an
 * explicit refresh operation.
 */
export function createAsyncQuery<K, T>(
  options: AsyncQueryOptions<K, T>,
): AsyncQuery<T> {
  let controller: AbortController | undefined;
  const value = createMemo<T | undefined>(() => {
    const key = options.source();
    controller?.abort();
    controller = undefined;
    if (key === undefined) return options.initialValue;
    controller = new AbortController();
    return options.load(key, { signal: controller.signal });
  });
  const latestValue = createMemo<T | undefined>(() => latest(value), {
    loadingValue: options.initialValue,
  });

  onCleanup(() => controller?.abort());

  return {
    value,
    latest: latestValue,
    async refresh() {
      refresh(value);
      return resolve(value);
    },
  };
}
