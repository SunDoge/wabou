import { describe, expect, test } from "bun:test";
import { createRoot, flush } from "solid-js";
import type { DeferredWriterScheduler } from "./deferred-writer";
import {
  createPersistedRecord,
  createPersistedValue,
} from "./persisted-record";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((accept) => {
    resolve = accept;
  });
  return { promise, resolve };
}

function manualScheduler() {
  let nextHandle = 0;
  const tasks = new Map<number, () => void>();
  const scheduler: DeferredWriterScheduler = {
    set(callback) {
      const handle = ++nextHandle;
      tasks.set(handle, callback);
      return handle;
    },
    clear(handle) {
      tasks.delete(handle as number);
    },
  };
  return {
    scheduler,
    run() {
      for (const [handle, task] of [...tasks]) {
        tasks.delete(handle);
        task();
      }
    },
  };
}

describe("persisted record", () => {
  test("keeps and saves a local edit made before hydration completes", async () => {
    const initialLoad = deferred<{ proxy: string; model: string }>();
    const saved: Array<{ proxy: string; model: string }> = [];
    const clock = manualScheduler();
    let record!: PersistedRecordForTest;
    let dispose!: () => void;

    createRoot((rootDispose) => {
      dispose = rootDispose;
      record = createPersistedRecord({
        initial: { proxy: "", model: "" },
        load: () => initialLoad.promise,
        save: (value) => {
          saved.push(value);
        },
        onLoadError: () => {},
        onSaveError: () => {},
        scheduler: clock.scheduler,
      });
      flush();
    });

    expect(record.loading()).toBe(true);

    record.update({ proxy: "http://127.0.0.1:7890" });
    initialLoad.resolve({ proxy: "http://persisted", model: "gpt-5" });
    await initialLoad.promise;
    await Promise.resolve();
    expect(record.loading()).toBe(false);
    expect(record.value()).toEqual({
      proxy: "http://127.0.0.1:7890",
      model: "",
    });

    clock.run();
    expect(saved).toEqual([{ proxy: "http://127.0.0.1:7890", model: "" }]);
    dispose();
  });

  test("flushes the last edit when its owner is disposed", () => {
    const saved: Array<{ theme: string }> = [];
    createRoot((dispose) => {
      const record = createPersistedRecord({
        initial: { theme: "light" },
        load: async () => ({ theme: "light" }),
        save: (value) => {
          saved.push(value);
        },
        onLoadError: () => {},
        onSaveError: () => {},
      });
      flush();
      record.update({ theme: "dark" });
      dispose();
    });
    expect(saved).toEqual([{ theme: "dark" }]);
  });

  test("exposes a failed save and retries the current value", async () => {
    const clock = manualScheduler();
    const saved: Array<{ proxy: string }> = [];
    const reported: unknown[] = [];
    let attempts = 0;
    let record!: ReturnType<typeof createPersistedRecord<{ proxy: string }>>;

    createRoot(() => {
      record = createPersistedRecord({
        initial: { proxy: "" },
        load: async () => ({ proxy: "" }),
        save: async (value) => {
          attempts += 1;
          if (attempts === 1) throw new Error("disk is read-only");
          saved.push(value);
        },
        onLoadError: () => {},
        onSaveError: (error) => reported.push(error),
        scheduler: clock.scheduler,
      });
      flush();
    });

    record.update({ proxy: "http://127.0.0.1:7890" });
    clock.run();
    await Promise.resolve();
    await Promise.resolve();
    expect(String(record.saveError())).toContain("disk is read-only");
    expect(reported).toHaveLength(1);

    record.retrySave();
    await Promise.resolve();
    expect(record.saveError()).toBeUndefined();
    expect(attempts).toBe(2);
    expect(saved).toEqual([{ proxy: "http://127.0.0.1:7890" }]);
  });
});

describe("persisted value", () => {
  test("persists non-record values with functional updates", () => {
    const clock = manualScheduler();
    const saved: string[][] = [];
    let state!: ReturnType<typeof createPersistedValue<readonly string[]>>;

    createRoot(() => {
      state = createPersistedValue({
        initial: ["agent-1"],
        load: async () => ["agent-1"],
        save: (value) => {
          saved.push([...value]);
        },
        onLoadError: () => {},
        onSaveError: () => {},
        scheduler: clock.scheduler,
      });
      flush();
    });

    state.update((current) => [...current, "agent-2"]);
    expect(state.value()).toEqual(["agent-1", "agent-2"]);
    clock.run();
    expect(saved).toEqual([["agent-1", "agent-2"]]);
  });
});

type PersistedRecordForTest = ReturnType<
  typeof createPersistedRecord<{ proxy: string; model: string }>
>;
