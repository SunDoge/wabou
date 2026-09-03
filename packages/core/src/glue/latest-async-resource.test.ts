import { expect, test } from "bun:test";
import { createRoot, createSignal, flush } from "solid-js";
import { createLatestAsyncResource } from "./latest-async-resource";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (cause: unknown) => void;
  const promise = new Promise<T>((accept, fail) => {
    resolve = accept;
    reject = fail;
  });
  return { promise, resolve, reject };
}

test("publishes a synchronous bootstrap load in the initiating flush", () => {
  let resource!: ReturnType<
    typeof createLatestAsyncResource<string, string>
  >;
  const committed: string[] = [];
  createRoot((dispose) => {
    resource = createLatestAsyncResource({
      source: () => "bootstrap",
      load: () => "ready",
      onCommit: (value) => committed.push(value),
    });
    flush();
    expect(resource.value()).toBe("ready");
    expect(resource.status()).toBe("ready");
    expect(resource.loading()).toBe(false);
    expect(committed).toEqual(["ready"]);
    dispose();
  });
});

test("only the latest key can update the resource", async () => {
  const first = deferred<string>();
  const second = deferred<string>();
  let resource!: ReturnType<typeof createLatestAsyncResource<string, string>>;
  let dispose!: () => void;

  createRoot((rootDispose) => {
    dispose = rootDispose;
    const [key, setKey] = createSignal<string>();
    resource = createLatestAsyncResource({
      source: key,
      load: (value) => (value === "first" ? first.promise : second.promise),
    });
    setKey("first");
    flush();
    setKey("second");
    flush();
  });

  first.resolve("stale");
  await first.promise;
  expect(resource.value()).toBeUndefined();
  expect(resource.loading()).toBe(true);

  second.resolve("latest");
  await second.promise;
  await Promise.resolve();
  expect(resource.value()).toBe("latest");
  expect(resource.loading()).toBe(false);
  dispose();
});

test("commits only the winning request before notifying dependents", async () => {
  const first = deferred<string>();
  const second = deferred<string>();
  const committed: string[] = [];
  let setKey!: (value: string) => void;
  let resource!: ReturnType<typeof createLatestAsyncResource<string, string>>;

  createRoot(() => {
    const pair = createSignal("first");
    setKey = pair[1];
    resource = createLatestAsyncResource({
      source: pair[0],
      load: (key) => (key === "first" ? first.promise : second.promise),
      onCommit: (value) => committed.push(value),
    });
    flush();
    setKey("second");
    flush();
  });

  first.resolve("stale");
  await first.promise;
  second.resolve("latest");
  await second.promise;
  await Promise.resolve();

  expect(resource.value()).toBe("latest");
  expect(committed).toEqual(["latest"]);
});

test("refresh replaces an in-flight request and exposes errors", async () => {
  const requests: ReturnType<typeof deferred<string>>[] = [];
  let resource!: ReturnType<typeof createLatestAsyncResource<string, string>>;
  let dispose!: () => void;

  createRoot((rootDispose) => {
    dispose = rootDispose;
    resource = createLatestAsyncResource({
      source: () => "task",
      load: () => {
        const request = deferred<string>();
        requests.push(request);
        return request.promise;
      },
    });
    flush();
  });

  expect(requests).toHaveLength(1);
  const refresh = resource.refresh();
  expect(requests).toHaveLength(2);
  requests[0]?.resolve("stale");
  requests[1]?.reject(new Error("failed"));
  expect(await refresh).toBeUndefined();
  expect(resource.status()).toBe("error");
  expect(resource.value()).toBeUndefined();
  expect(String(resource.error())).toContain("failed");
  expect(resource.loading()).toBe(false);
  dispose();
});

test("clearing the source aborts pending work", () => {
  let signal!: AbortSignal;
  let setKey!: (value: string | undefined) => void;
  let resource!: ReturnType<typeof createLatestAsyncResource<string, string>>;

  createRoot((dispose) => {
    const pair = createSignal<string>();
    setKey = pair[1];
    resource = createLatestAsyncResource({
      source: pair[0],
      load: (_key, context) => {
        signal = context.signal;
        return new Promise(() => undefined);
      },
    });
    setKey("task");
    flush();
    expect(signal.aborted).toBe(false);
    setKey(undefined);
    flush();
    expect(signal.aborted).toBe(true);
    expect(resource.loading()).toBe(false);
    dispose();
  });
});

test("an undefined source disables loading and re-enabling reloads the key", async () => {
  let setEnabled!: (enabled: boolean) => void;
  let resource!: ReturnType<typeof createLatestAsyncResource<string, number>>;
  let dispose!: () => void;
  let loads = 0;
  createRoot((rootDispose) => {
    dispose = rootDispose;
    const [enabled, writeEnabled] = createSignal(false);
    setEnabled = writeEnabled;
    resource = createLatestAsyncResource({
      source: () => (enabled() ? "details" : undefined),
      load: async () => ++loads,
    });
    flush();
    expect(loads).toBe(0);
    setEnabled(true);
    flush();
  });
  await Promise.resolve();
  flush();
  await Promise.resolve();
  expect(resource.value()).toBe(1);

  setEnabled(false);
  flush();
  expect(resource.value()).toBeUndefined();
  expect(resource.status()).toBe("idle");
  setEnabled(true);
  flush();
  await Promise.resolve();
  flush();
  await Promise.resolve();
  expect(resource.value()).toBe(2);
  dispose();
});

test("initial values remain distinct from readiness and can be mutated", async () => {
  const request = deferred<string>();
  let resource!: ReturnType<typeof createLatestAsyncResource<boolean, string>>;
  createRoot((dispose) => {
    resource = createLatestAsyncResource({
      source: () => true,
      initialValue: "fallback",
      load: () => request.promise,
    });
    expect(resource.value()).toBe("fallback");
    expect(resource.status()).toBe("idle");
    flush();
    expect(resource.status()).toBe("pending");
    resource.mutate("local");
    flush();
    expect(resource.value()).toBe("local");
    expect(resource.status()).toBe("ready");
    expect(resource.loading()).toBe(false);
    request.resolve("stale");
    dispose();
  });
  await request.promise;
  await Promise.resolve();
  expect(resource.value()).toBe("local");
});

test("source invalidation does not reload an unchanged key", async () => {
  let invalidate!: (value: number) => void;
  let loads = 0;
  let dispose!: () => void;
  createRoot((rootDispose) => {
    dispose = rootDispose;
    const [revision, setRevision] = createSignal(0);
    invalidate = setRevision;
    createLatestAsyncResource({
      source: () => {
        revision();
        return "fixture";
      },
      load: async () => {
        loads++;
        return "loaded";
      },
    });
    flush();
  });
  await Promise.resolve();
  flush();
  expect(loads).toBe(1);

  invalidate(1);
  flush();
  await Promise.resolve();
  expect(loads).toBe(1);
  dispose();
});
