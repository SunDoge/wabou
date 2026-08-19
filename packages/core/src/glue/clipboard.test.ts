import { expect, test } from "bun:test";
import { EFFECT_ABI_VERSION } from "../generated/effect-abi";

let nextRequest = 1;
const calls: Array<[number, number, unknown]> = [];

Object.assign(globalThis, {
  __wabou_effect_abi: EFFECT_ABI_VERSION,
  __wabou_effect_submit: (capability: number, method: number, json: string) => {
    const requestId = nextRequest++;
    calls.push([capability, method, JSON.parse(json)]);
    return requestId;
  },
});

const { clipboard, useClipboard } = await import("./clipboard");

test("clipboard resolves concurrent reads by effect ID", async () => {
  const first = clipboard.readText();
  const second = clipboard.readText();
  __wabou_effect_complete(2, 1, 1, 0, JSON.stringify("second"));
  __wabou_effect_complete(1, 1, 1, 0, JSON.stringify("first"));

  expect(await first).toBe("first");
  expect(await second).toBe("second");
});

test("clipboard confirms writes and reports native failures", async () => {
  const written = clipboard.writeText("hello");
  expect(calls.at(-1)).toEqual([1, 2, { text: "hello" }]);
  __wabou_effect_complete(3, 1, 2, 0, "null");
  await written;

  const failed = clipboard.writeText("denied");
  __wabou_effect_complete(
    4,
    1,
    2,
    2,
    JSON.stringify({ code: "platformFailure", message: "denied" }),
  );
  await expect(failed).rejects.toThrow("denied");
});

test("useClipboard returns the stable window clipboard capability", () => {
  expect(useClipboard()).toBe(clipboard);
});
