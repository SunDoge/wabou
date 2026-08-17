import { expect, test } from "bun:test";
import type { Locator, TestAction, TestContext, TestPage } from "./index";
import { replayActions } from "./replay";

test("replay preserves the logical window for every action", async () => {
  const observed: unknown[][] = [];
  const record = (...values: unknown[]): void => {
    observed.push(values);
  };
  const locator = (windowId: number, role: string, name: string): Locator => ({
    windowId,
    click: async () => record(windowId, role, name, "click"),
    dragBy: async (x, y) => record(windowId, role, name, "drag", x, y),
    press: async (key, modifiers) =>
      record(windowId, role, name, "key", key, modifiers),
    type: async (text) => record(windowId, role, name, "text", text),
    paste: async (text) => record(windowId, role, name, "paste", text),
    ime: async (text) => record(windowId, role, name, "ime", text),
    wheel: async (y, x) => record(windowId, role, name, "wheel", x, y),
    waitFor: async () => record(windowId, role, name, "probe"),
    snapshot: async () => {
      throw new Error("not used");
    },
  });
  const page = {
    forWindow(windowId: number): TestPage {
      return {
        ...page,
        getByRole: (role, options) => locator(windowId, role, options.name),
      };
    },
    getByRole: (role, options) => locator(1, role, options.name),
    waitForIdle: async () => {},
  } satisfies TestPage;
  const window: TestContext["window"] = {
    nativeClose: async (windowId, platform) =>
      record(windowId, "nativeClose", platform),
    show: async (windowId) => record(windowId, "show"),
    state: () => null,
  };
  const actions: TestAction[] = [
    { action: "clickByRole", windowId: 2, role: "button", label: "Save" },
    {
      action: "inputByRole",
      windowId: 3,
      role: "textbox",
      label: "Name",
      input: { type: "key", key: "a", modifiers: 3 },
    },
    {
      action: "inputByRole",
      windowId: 4,
      role: "listbox",
      label: "Rows",
      input: { type: "wheel", deltaX: 5, deltaY: 9 },
    },
  ];

  await replayActions(actions, page, window);

  expect(observed).toEqual([
    [2, "button", "Save", "click"],
    [
      3,
      "textbox",
      "Name",
      "key",
      "a",
      { shift: true, control: true, alt: false, meta: false },
    ],
    [4, "listbox", "Rows", "wheel", 5, 9],
  ]);
});
