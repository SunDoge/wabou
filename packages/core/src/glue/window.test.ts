import { expect, test } from "bun:test";
import { createWindow, currentWindow } from "./window";

test("createWindow returns a handle that targets the created window", () => {
  const calls: unknown[][] = [];
  Object.assign(globalThis, {
    __wabou_window_id: 3,
    __wabou_window_create: (json: string) => {
      calls.push(["create", JSON.parse(json)]);
      return 42;
    },
    __wabou_window_close: (id: number) => calls.push(["close", id]),
    __wabou_window_set_maximized: (id: number, value: boolean) =>
      calls.push(["maximize", id, value]),
    __wabou_window_set_title: (id: number, title: string) =>
      calls.push(["title", id, title]),
  });

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
    [
      "create",
      { title: "Inspector", width: 640, height: 480, transparent: true },
    ],
    ["title", 42, "Details"],
    ["maximize", 42, true],
    ["close", 42],
    ["close", 3],
  ]);
});
