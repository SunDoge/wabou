// Install the same timers/frame glue as an application entry.
import "@wabou/core";
import { expect, test } from "@wabou/test";

type Key = { lo: number; hi: number };
const native = (globalThis as any).__wabou_capabilities.probe;
async function call<T>(method: string, request: unknown): Promise<T> {
  const result = JSON.parse(await native[method](JSON.stringify(request)));
  if (!result.ok) throw new Error(result.error.message);
  return result.value;
}
async function result<T>(task: Key): Promise<T> {
  try {
    for (;;) {
      const value = await call<any>("poll", task);
      if (value.status === "ready") return value.value;
      if (value.status === "failed") throw new Error(value.error);
      // Each asynchronous capability invocation returns to the host executor.
    }
  } finally {
    await call("release", task);
  }
}

test("shared runtime supplies typed state, files, tasks and TCP to SDK modules", async () => {
  const entry = native.increment;
  expect(await call("increment", 2)).toBe(2);
  expect(await call("increment", 3)).toBe(5);
  expect(native.increment === entry).toBe(true);
  await result(await call<Key>("sleep", 0));

  const file = await result<Key>(await call<Key>("openFile", "document.txt"));
  expect(await result(await call<Key>("readFile", [file, 1]))).toEqual([65]);
  expect(await result(await call<Key>("readFile", [file, 1]))).toEqual([66]);
  await call("closeFile", file);

  const connection = await result<Key>(
    await call<Key>("connect", (globalThis as any).__probe_address),
  );
  const read = await call<Key>("readSocket", [connection, 1]);
  expect(await result(await call<Key>("writeSocket", [connection, [67]]))).toBe(1);
  expect(await result(read)).toEqual([67]);
  const pending = await call<Key>("readSocket", [connection, 1]);
  await call("closeSocket", connection);
  const cancelled = await call<any>("poll", pending);
  // Cancellation is cooperative; query until the host has observed close.
  let value = cancelled;
  while (value.status === "pending") value = await call<any>("poll", pending);
  expect(value.status).toBe("failed");
  expect(value.error).toBe("connection closed");
  await call("release", pending);
});
