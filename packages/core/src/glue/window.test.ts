import { expect, test } from "bun:test";

test("createWindow returns a handle that targets the created window", async () => {
  const calls: unknown[][] = [];
  let nextRequest = 42;
  Object.assign(globalThis, {
  __wabou_effect_abi: 2,
    __wabou_window_id_lo: 3,
    __wabou_window_id_hi: 1,
    __wabou_effect_submit: (
      capability: number,
      method: number,
      json: string,
    ) => {
      calls.push([capability, method, JSON.parse(json)]);
      const request = nextRequest++;
      if (capability === 2 && method === 1) {
        queueMicrotask(() =>
          __wabou_effect_complete(request, capability, method, 0, '{"lo":42,"hi":1}'),
        );
      }
      return request;
    },
  });
  const { createWindow, currentWindow } = await import("./window");

  const child = await createWindow({
    title: "Inspector",
    width: 640,
    height: 480,
    decorations: false,
    transparent: true,
  });
  expect(child.id).toMatchObject({ lo: 42, hi: 1 });
  child.setTitle("Details");
  child.minimize();
  child.setMaximized(true);
  child.startDragging();
  child.close();
  currentWindow().close();

  expect(calls).toEqual([
    [
      2,
      1,
      {
        title: "Inspector",
        width: 640,
        height: 480,
        decorations: false,
        transparent: true,
      },
    ],
    [2, 4, { windowId: { lo: 42, hi: 1 }, title: "Details" }],
    [2, 5, { windowId: { lo: 42, hi: 1 } }],
    [2, 3, { windowId: { lo: 42, hi: 1 }, value: true }],
    [2, 6, { windowId: { lo: 42, hi: 1 } }],
    [2, 2, { windowId: { lo: 42, hi: 1 } }],
    [2, 2, { windowId: { lo: 3, hi: 1 } }],
  ]);
});
