import { expect, test } from "bun:test";

test("createWindow returns a handle that targets the created window", async () => {
  const calls: unknown[][] = [];
  let nextRequest = 42;
  Object.assign(globalThis, {
    __wabou_effect_abi: 1,
    __wabou_window_id: 3,
    __wabou_effect_submit: (capability: number, method: number, json: string) => {
      calls.push([capability, method, JSON.parse(json)]);
      return nextRequest++;
    },
  });
  const { createWindow, currentWindow } = await import("./window");

  const child = createWindow({
    title: "Inspector",
    width: 640,
    height: 480,
    transparent: true,
  });
  expect(child.id).toBe(42);
  child.setTitle("Details");
  child.setMaximized(true);
  child.close();
  currentWindow().close();

  expect(calls).toEqual([
    [2, 1, { title: "Inspector", width: 640, height: 480, transparent: true }],
    [2, 4, { windowId: 42, title: "Details" }],
    [2, 3, { windowId: 42, value: true }],
    [2, 2, { windowId: 42 }],
    [2, 2, { windowId: 3 }],
  ]);
});
