import { expect, test } from "bun:test";

let nextRequest = 1;
const writes: Array<[number, string]> = [];

Object.assign(globalThis, {
  __wabou_clipboard_read: () => nextRequest++,
  __wabou_clipboard_write: (text: string) => {
    const requestId = nextRequest++;
    writes.push([requestId, text]);
    return requestId;
  },
});

const { clipboard, useClipboard } = await import("./clipboard");

test("clipboard resolves concurrent reads by request ID", async () => {
  const first = clipboard.readText();
  const second = clipboard.readText();
  __wabou_clipboard_complete(2, "second", true);
  __wabou_clipboard_complete(1, "first", true);

  expect(await first).toBe("first");
  expect(await second).toBe("second");
});

test("clipboard confirms writes and reports native failures", async () => {
  const written = clipboard.writeText("hello");
  expect(writes.at(-1)).toEqual([3, "hello"]);
  __wabou_clipboard_complete(3, null, true);
  await written;

  const failed = clipboard.writeText("denied");
  __wabou_clipboard_complete(4, null, false);
  await expect(failed).rejects.toThrow("Native clipboard operation failed");
});

test("useClipboard returns the stable window clipboard capability", () => {
  expect(useClipboard()).toBe(clipboard);
});
