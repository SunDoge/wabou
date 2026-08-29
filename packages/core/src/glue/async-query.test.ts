import { expect, test } from "bun:test";
import { createRoot, createSignal, flush } from "solid-js";
import { createAsyncQuery } from "./async-query";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((accept) => {
    resolve = accept;
  });
  return { promise, resolve };
}

test("Solid async query keeps the latest settled result", async () => {
  const first = deferred<string>();
  const second = deferred<string>();
  let setKey!: (key: string) => void;
  let query!: ReturnType<typeof createAsyncQuery<string, string>>;

  const dispose = createRoot((dispose) => {
    const pair = createSignal("first");
    setKey = pair[1];
    query = createAsyncQuery({
      source: pair[0],
      load: (key) => (key === "first" ? first.promise : second.promise),
    });
    // Observe the lazy memo just as JSX would.
    expect(query.latest()).toBeUndefined();
    return dispose;
  });

  setKey("second");
  flush();

  first.resolve("stale");
  await first.promise;
  await Promise.resolve();
  flush();
  expect(query.latest()).toBeUndefined();

  second.resolve("latest");
  await second.promise;
  await Promise.resolve();
  flush();
  expect(query.latest()).toBe("latest");
  dispose();
});

test("refresh delegates to the async memo and aborts replaced work", async () => {
  const requests: Array<{
    signal: AbortSignal;
    request: ReturnType<typeof deferred<number>>;
  }> = [];
  let query!: ReturnType<typeof createAsyncQuery<string, number>>;
  const dispose = createRoot((dispose) => {
    query = createAsyncQuery({
      source: () => "settings",
      load: (_key, { signal }) => {
        const request = deferred<number>();
        requests.push({ signal, request });
        return request.promise;
      },
    });
    query.latest();
    return dispose;
  });

  expect(requests).toHaveLength(1);
  const refreshed = query.refresh();
  flush();
  expect(requests).toHaveLength(2);
  expect(requests[0]?.signal.aborted).toBe(true);
  requests[1]?.request.resolve(2);
  expect(await refreshed).toBe(2);
  dispose();
  expect(requests[1]?.signal.aborted).toBe(true);
});
