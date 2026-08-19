import { expect, test } from "bun:test";

let nextRequest = 1;
const calls: Array<[number, number, unknown]> = [];

Object.assign(globalThis, {
  __wabou_effect_abi: 2,
  __wabou_effect_submit: (capability: number, method: number, json: string) => {
    const requestId = nextRequest++;
    calls.push([capability, method, JSON.parse(json)]);
    return requestId;
  },
});

const { notification } = await import("./notification");

test("system notification completion and failure follow the effect bridge", async () => {
  const shown = notification.show({ title: "Ready", body: "Export completed" });
  expect(calls.at(-1)).toEqual([
    6,
    1,
    { title: "Ready", body: "Export completed", silent: false },
  ]);
  __wabou_effect_complete(1, 6, 1, 0, "null");
  await shown;

  const failed = notification.show({ title: "Denied" });
  __wabou_effect_complete(
    2,
    6,
    1,
    2,
    JSON.stringify({ code: "platformFailure", message: "permission denied" }),
  );
  await expect(failed).rejects.toThrow("permission denied");
});
