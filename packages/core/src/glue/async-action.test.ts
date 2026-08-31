import { expect, test } from "bun:test";
import { createRoot, flush } from "solid-js";
import {
  type AsyncActionResult,
  AsyncActionConflictError,
  createAsyncAction,
  createKeyedAsyncAction,
} from "./async-action";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (cause: unknown) => void;
  const promise = new Promise<T>((accept, fail) => {
    resolve = accept;
    reject = fail;
  });
  return { promise, resolve, reject };
}

test("concurrent calls join one async action", async () => {
  const request = deferred<number>();
  let calls = 0;
  const action = createAsyncAction(async (value: number) => {
    calls++;
    return (await request.promise) + value;
  });

  const first = action.run(2);
  const second = action.run(2);
  expect(first).toBe(second);
  flush();
  expect(action.pending()).toBe(true);
  expect(action.pendingArgs()).toEqual([2]);
  request.resolve(3);
  expect(await first).toEqual({ ok: true, value: 5 });
  flush();
  expect(calls).toBe(1);
  expect(action.pending()).toBe(false);
  expect(action.pendingArgs()).toBeUndefined();
  expect(action.error()).toBeUndefined();
});

test("starts immediately while preserving a re-entrant single flight", async () => {
  let nested: Promise<AsyncActionResult<number>> | undefined;
  let calls = 0;
  let action!: ReturnType<typeof createAsyncAction<[number], number>>;
  action = createAsyncAction((value: number) => {
    calls += 1;
    nested = action.run(value);
    return value * 2;
  });

  const outer = action.run(3);
  expect(calls).toBe(1);
  expect(nested).toBe(outer);
  expect(await outer).toEqual({ ok: true, value: 6 });
});

test("concurrent calls do not silently discard different arguments", async () => {
  const request = deferred<number>();
  const seen: number[] = [];
  const action = createAsyncAction(async (value: number) => {
    seen.push(value);
    return await request.promise;
  });

  const first = action.run(2);
  const conflicting = await action.run(100);
  expect(conflicting.ok).toBe(false);
  if (!conflicting.ok)
    expect(conflicting.error).toBeInstanceOf(AsyncActionConflictError);
  expect(action.pending()).toBe(true);
  expect(action.error()).toBeUndefined();
  request.resolve(3);
  expect(await first).toEqual({ ok: true, value: 3 });
  expect(seen).toEqual([2]);
});

test("failures become explicit results and reset on the next run", async () => {
  let fail = true;
  const action = createAsyncAction(() => {
    if (fail) throw new Error("failed");
    return "ready";
  });

  const failed = await action.run();
  expect(failed.ok).toBe(false);
  expect(String(action.error())).toContain("failed");
  fail = false;
  expect(await action.run()).toEqual({ ok: true, value: "ready" });
  expect(action.error()).toBeUndefined();
});

test("settling after owner disposal does not publish state", async () => {
  const request = deferred<void>();
  let dispose!: () => void;
  let action!: ReturnType<typeof createAsyncAction<[], void>>;
  createRoot((rootDispose) => {
    dispose = rootDispose;
    action = createAsyncAction(() => request.promise);
  });

  const result = action.run();
  dispose();
  flush();
  expect(action.pending()).toBe(false);
  request.reject(new Error("late"));
  expect((await result).ok).toBe(false);
  expect(action.error()).toBeUndefined();
  expect((await action.run()).ok).toBe(false);
});

test("keyed actions join equal keys while unrelated keys run concurrently", async () => {
  const requests = new Map<string, ReturnType<typeof deferred<number>>>();
  const calls: string[] = [];
  const action = createKeyedAsyncAction(
    (key: string, _offset: number) => key,
    async (key: string, offset: number) => {
      calls.push(key);
      const request = deferred<number>();
      requests.set(key, request);
      return (await request.promise) + offset;
    },
  );

  const first = action.run("a", 1);
  const duplicate = action.run("a", 100);
  const other = action.run("b", 2);
  expect(first).toBe(duplicate);
  await Promise.resolve();
  flush();
  expect(calls).toEqual(["a", "b"]);
  expect(action.pending("a")).toBe(true);
  expect(action.pending("b")).toBe(true);

  requests.get("a")?.resolve(10);
  expect(await first).toEqual({ ok: true, value: 11 });
  flush();
  expect(action.pending("a")).toBe(false);
  expect(action.pending("b")).toBe(true);
  requests.get("b")?.resolve(20);
  expect(await other).toEqual({ ok: true, value: 22 });
});

test("keyed action errors and disposal are isolated per key", async () => {
  let dispose!: () => void;
  let action!: ReturnType<
    typeof createKeyedAsyncAction<string, [string], void>
  >;
  createRoot((rootDispose) => {
    dispose = rootDispose;
    action = createKeyedAsyncAction(
      (key: string) => key,
      async (key: string) => {
        if (key === "bad") throw new Error("bad key");
      },
    );
  });

  expect((await action.run("bad")).ok).toBe(false);
  expect(String(action.error("bad"))).toContain("bad key");
  expect((await action.run("good")).ok).toBe(true);
  expect(action.error("good")).toBeUndefined();
  action.reset("bad");
  flush();
  expect(action.error("bad")).toBeUndefined();
  dispose();
  expect((await action.run("later")).ok).toBe(false);
});
