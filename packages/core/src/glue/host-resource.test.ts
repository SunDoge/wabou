import { expect, test } from "bun:test";
import { createRoot, flush } from "solid-js";
import { dispatchHostMessage } from "./host-messages";
import {
  createRevisionedHostResource,
  RevisionedHostWaitError,
} from "./host-resource";

interface Value {
  revision: number;
  count: number;
}

interface Patch {
  baseRevision: number;
  revision: number;
  delta: number;
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((accept) => {
    resolve = accept;
  });
  return { promise, resolve };
}

const json = (value: unknown) => JSON.stringify(value);

test("host pushes win races with an older initial load", async () => {
  let resolveLoad!: (value: Value) => void;
  const load = new Promise<Value>((resolve) => {
    resolveLoad = resolve;
  });
  let dispose!: () => void;
  const resource = createRoot((rootDispose) => {
    dispose = rootDispose;
    return createRevisionedHostResource<Value, Patch>({
      initial: { revision: 0, count: 0 },
      load: () => load,
      snapshotTopic: "resource.race.snapshot",
    });
  });

  dispatchHostMessage(
    "resource.race.snapshot",
    json({ revision: 1, count: 2 }),
  );
  flush();
  await Promise.resolve();
  resolveLoad({ revision: 1, count: 1 });
  await resource.refresh();

  expect(resource.value()).toEqual({ revision: 1, count: 2 });
  expect(resource.loading()).toBe(false);
  dispose();
});

test("accepts an initial equal revision but rejects later equal duplicates", async () => {
  const accepted: Value[] = [];
  let failLoad = false;
  let dispose!: () => void;
  const resource = createRoot((rootDispose) => {
    dispose = rootDispose;
    return createRevisionedHostResource<Value>({
      initial: { revision: 0, count: 0 },
      autoLoad: false,
      load: async () => {
        if (failLoad) throw new Error("offline");
        return { revision: 0, count: 1 };
      },
      snapshotTopic: "resource.equal.snapshot",
      onValue: (value) => accepted.push(value),
    });
  });

  expect(await resource.refresh()).toEqual({ revision: 0, count: 1 });
  failLoad = true;
  await expect(resource.refresh()).rejects.toThrow("offline");
  expect(String(resource.error())).toContain("offline");
  dispatchHostMessage(
    "resource.equal.snapshot",
    json({ revision: 0, count: 999 }),
  );
  flush();
  expect(resource.value()).toEqual({ revision: 0, count: 1 });
  expect(accepted).toEqual([{ revision: 0, count: 1 }]);
  expect(resource.error()).toBeUndefined();
  dispose();
});

test("patch gaps coalesce a full refresh and valid patches apply in order", async () => {
  let loads = 0;
  let releaseRefresh!: (value: Value) => void;
  let dispose!: () => void;
  const resource = createRoot((rootDispose) => {
    dispose = rootDispose;
    return createRevisionedHostResource<Value, Patch>({
      initial: { revision: 3, count: 3 },
      autoLoad: false,
      load: () => {
        loads++;
        return new Promise((resolve) => {
          releaseRefresh = resolve;
        });
      },
      snapshotTopic: "resource.patch.snapshot",
      patchTopic: "resource.patch.delta",
      applyPatch: (current, patch) => ({
        revision: patch.revision,
        count: current.count + patch.delta,
      }),
    });
  });

  dispatchHostMessage(
    "resource.patch.delta",
    json({ baseRevision: 3, revision: 4, delta: 2 }),
  );
  flush();
  expect(resource.value()).toEqual({ revision: 4, count: 5 });

  dispatchHostMessage(
    "resource.patch.delta",
    json({ baseRevision: 2, revision: 5, delta: 10 }),
  );
  dispatchHostMessage(
    "resource.patch.delta",
    json({ baseRevision: 2, revision: 5, delta: 10 }),
  );
  flush();
  expect(loads).toBe(1);
  expect(resource.loading()).toBe(true);
  releaseRefresh({ revision: 5, count: 8 });
  await resource.refresh();
  flush();
  expect(resource.value()).toEqual({ revision: 5, count: 8 });
  expect(resource.loading()).toBe(false);
  dispose();
});

test("disposing removes host subscriptions and cancels a queued initial load", async () => {
  let loads = 0;
  const resource = createRoot(() => {
    return createRevisionedHostResource<Value>({
      initial: { revision: 0, count: 0 },
      load: async () => {
        loads++;
        return { revision: 2, count: 2 };
      },
      snapshotTopic: "resource.dispose.snapshot",
    });
  });

  resource.dispose();
  dispatchHostMessage(
    "resource.dispose.snapshot",
    json({ revision: 1, count: 1 }),
  );
  flush();
  await Promise.resolve();
  await Promise.resolve();
  expect(loads).toBe(0);
  expect(resource.value()).toEqual({ revision: 0, count: 0 });
});

test("automatic loading never writes during the creating owned scope", async () => {
  let loads = 0;
  const resource = createRoot(() =>
    createRevisionedHostResource<Value>({
      initial: { revision: 0, count: 0 },
      load: async () => {
        loads++;
        return { revision: 1, count: 1 };
      },
      snapshotTopic: "resource.deferred-initial-load",
    }),
  );

  expect(loads).toBe(0);
  expect(resource.loading()).toBe(false);
  await Promise.resolve();
  expect(loads).toBe(1);
  await Promise.resolve();
  await Promise.resolve();
  expect(resource.value()).toEqual({ revision: 1, count: 1 });
  resource.dispose();
});

test("waitFor resolves current or future values and rejects aborts and disposal", async () => {
  let dispose!: () => void;
  const resource = createRoot((rootDispose) => {
    dispose = rootDispose;
    return createRevisionedHostResource<Value>({
      initial: { revision: 0, count: 0 },
      autoLoad: false,
      load: async () => ({ revision: 0, count: 0 }),
      snapshotTopic: "resource.wait.snapshot",
    });
  });

  expect(await resource.waitFor((value) => value.count === 0)).toEqual({
    revision: 0,
    count: 0,
  });
  const future = resource.waitFor((value) => value.count === 2);
  dispatchHostMessage(
    "resource.wait.snapshot",
    json({ revision: 1, count: 2 }),
  );
  flush();
  expect(await future).toEqual({ revision: 1, count: 2 });

  const controller = new AbortController();
  const aborted = resource.waitFor(() => false, {
    signal: controller.signal,
  });
  controller.abort();
  await expect(aborted).rejects.toThrow("aborted");

  const pending = resource.waitFor(() => false);
  dispose();
  await expect(pending).rejects.toThrow("disposed");
});

test("waitFor rejects timeouts and predicate errors", async () => {
  const resource = createRevisionedHostResource<Value>({
    initial: { revision: 0, count: 0 },
    autoLoad: false,
    load: async () => ({ revision: 0, count: 0 }),
    snapshotTopic: "resource.wait.errors",
  });
  const timeout = resource.waitFor(() => false, { timeout: 1 });
  await expect(timeout).rejects.toBeInstanceOf(RevisionedHostWaitError);
  await timeout.catch((error: RevisionedHostWaitError) =>
    expect(error.reason).toBe("timeout"),
  );
  await expect(
    resource.waitFor(() => {
      throw new Error("bad predicate");
    }),
  ).rejects.toThrow("bad predicate");
  resource.dispose();
});

test("waitFor can close a missed-push race with one full refresh", async () => {
  let loads = 0;
  const resource = createRevisionedHostResource<Value>({
    initial: { revision: 0, count: 0 },
    autoLoad: false,
    load: async () => {
      loads++;
      return { revision: 1, count: 2 };
    },
    snapshotTopic: "resource.wait.refresh",
  });

  expect(
    await resource.waitFor((value) => value.count === 2, {
      timeout: 1,
      refreshOnTimeout: true,
    }),
  ).toEqual({ revision: 1, count: 2 });
  expect(loads).toBe(1);
  resource.dispose();
});

test("waitFor preserves its timeout when the fallback refresh does not converge", async () => {
  const resource = createRevisionedHostResource<Value>({
    initial: { revision: 0, count: 0 },
    autoLoad: false,
    load: async () => ({ revision: 1, count: 1 }),
    snapshotTopic: "resource.wait.refresh-timeout",
  });

  const pending = resource.waitFor((value) => value.count === 2, {
    timeout: 1,
    refreshOnTimeout: true,
  });
  await expect(pending).rejects.toBeInstanceOf(RevisionedHostWaitError);
  await pending.catch((error: RevisionedHostWaitError) =>
    expect(error.reason).toBe("timeout"),
  );
  resource.dispose();
});

test("waitFor does not mistake a pre-existing stale refresh for its consistency read", async () => {
  const stale = deferred<Value>();
  let loads = 0;
  const resource = createRevisionedHostResource<Value>({
    initial: { revision: 0, count: 0 },
    autoLoad: false,
    load: async () => {
      loads++;
      return loads === 1 ? stale.promise : { revision: 1, count: 2 };
    },
    snapshotTopic: "resource.wait.stale-refresh",
  });

  const oldRefresh = resource.refresh();
  const converged = resource.waitFor((value) => value.count === 2, {
    timeout: 1,
    refreshOnTimeout: true,
  });
  await new Promise((resolve) => setTimeout(resolve, 5));
  stale.resolve({ revision: 0, count: 0 });

  expect(await oldRefresh).toEqual({ revision: 0, count: 0 });
  expect(await converged).toEqual({ revision: 1, count: 2 });
  expect(loads).toBe(2);
  resource.dispose();
});
