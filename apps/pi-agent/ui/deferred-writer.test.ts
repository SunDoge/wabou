import { describe, expect, test } from "bun:test";
import {
  createDeferredWriter,
  type DeferredWriterScheduler,
} from "./deferred-writer";

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
    get size() {
      return tasks.size;
    },
  };
}

describe("deferred writer", () => {
  test("coalesces rapid updates into the latest value", () => {
    const clock = manualScheduler();
    const written: string[] = [];
    const writer = createDeferredWriter({
      scheduler: clock.scheduler,
      write: (value: string) => {
        written.push(value);
      },
      onError: () => {},
    });

    writer.schedule("first");
    writer.schedule("latest");
    expect(clock.size).toBe(1);
    clock.run();
    expect(written).toEqual(["latest"]);
  });

  test("flushes a pending value during application cleanup", () => {
    const clock = manualScheduler();
    const written: string[] = [];
    const writer = createDeferredWriter({
      scheduler: clock.scheduler,
      write: (value: string) => {
        written.push(value);
      },
      onError: () => {},
    });

    writer.schedule("last edit before reload");
    writer.flush();
    expect(clock.size).toBe(0);
    expect(written).toEqual(["last edit before reload"]);
    writer.flush();
    expect(written).toEqual(["last edit before reload"]);
  });

  test("reports synchronous and asynchronous write failures", async () => {
    const errors: unknown[] = [];
    const syncWriter = createDeferredWriter({
      write: () => {
        throw new Error("sync failure");
      },
      onError: (error) => errors.push(error),
    });
    syncWriter.schedule("value");
    syncWriter.flush();

    const asyncWriter = createDeferredWriter({
      write: async () => {
        throw new Error("async failure");
      },
      onError: (error) => errors.push(error),
    });
    asyncWriter.schedule("value");
    asyncWriter.flush();
    await Promise.resolve();
    await Promise.resolve();

    expect(errors.map(String)).toEqual([
      "Error: sync failure",
      "Error: async failure",
    ]);
  });

  test("primes durable state and coalesces equal snapshots", () => {
    const saved: string[] = [];
    const clock = manualScheduler();
    const writer = createDeferredWriter({
      write: (value: string) => {
        saved.push(value);
      },
      onError: () => {},
      scheduler: clock.scheduler,
      equals: Object.is,
    });

    writer.prime("restored");
    writer.schedule("restored");
    clock.run();
    expect(saved).toEqual([]);

    writer.schedule("edited");
    writer.schedule("edited");
    clock.run();
    expect(saved).toEqual(["edited"]);

    writer.schedule("edited");
    clock.run();
    expect(saved).toEqual(["edited"]);
  });

  test("allows an equal snapshot to retry after a failed write", async () => {
    const errors: unknown[] = [];
    const clock = manualScheduler();
    let attempts = 0;
    const writer = createDeferredWriter({
      write: async () => {
        attempts += 1;
        if (attempts === 1) throw new Error("disk unavailable");
      },
      onError: (error) => errors.push(error),
      scheduler: clock.scheduler,
      equals: Object.is,
    });

    writer.schedule("snapshot");
    clock.run();
    await Promise.resolve();
    await Promise.resolve();
    expect(errors).toHaveLength(1);

    writer.schedule("snapshot");
    clock.run();
    await Promise.resolve();
    expect(attempts).toBe(2);
  });
});
