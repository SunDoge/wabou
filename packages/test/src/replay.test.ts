import { expect, test } from "bun:test";
import type { Locator, TestAction, TestContext, TestPage } from "./index";
import { replayActions } from "./replay";

test("replay preserves the logical window for every action", async () => {
  const observed: unknown[][] = [];
  const record = (...values: unknown[]): void => {
    observed.push(values);
  };
  const locator = (
    windowId: number,
    role: string,
    name: string,
    index?: number,
  ): Locator => ({
    windowId,
    role: role as Locator["role"],
    name,
    index,
    click: async (wait) => record(windowId, role, name, index, "click", wait),
    dragBy: async (x, y, wait) =>
      record(windowId, role, name, "drag", x, y, wait),
    press: async (key, modifiers, wait) =>
      record(windowId, role, name, "key", key, modifiers, wait),
    type: async (text, wait) =>
      record(windowId, role, name, "text", text, wait),
    paste: async (text, wait) =>
      record(windowId, role, name, "paste", text, wait),
    ime: async (text, wait) => record(windowId, role, name, "ime", text, wait),
    wheel: async (y, x, wait) =>
      record(windowId, role, name, "wheel", x, y, wait),
    waitFor: async (wait) => record(windowId, role, name, "wait", wait),
    snapshot: async () => {
      record(windowId, role, name, "snapshot");
      return {
        name,
        value: null,
        numericValue: null,
        minNumericValue: null,
        maxNumericValue: null,
        bounds: { x: 0, y: 0, width: 100, height: 20 },
        disabled: false,
        checked: null,
        pressed: null,
        selected: null,
        expanded: null,
        focused: false,
      };
    },
  });
  const page = {
    forWindow(windowId: number): TestPage {
      return {
        ...page,
        getByRole: (role, options) =>
          locator(windowId, role, options.name, options.index),
      };
    },
    getByRole: (role, options) => locator(1, role, options.name, options.index),
    waitForIdle: async () => {},
  } satisfies TestPage;
  const window: TestContext["window"] = {
    nativeClose: async (windowId, platform) =>
      record(windowId, "nativeClose", platform),
    show: async (windowId) => record(windowId, "show"),
    state: () => null,
  };
  const actions: TestAction[] = [
    {
      action: "clickByRole",
      windowId: 2,
      role: "button",
      label: "Save",
      index: 1,
      wait: { timeout: 2_000, interval: 20 },
    },
    {
      action: "waitForByRole",
      windowId: 5,
      role: "status",
      label: "Ready",
      wait: { timeout: 3_000, interval: 30 },
    },
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
    {
      action: "assertByRole",
      windowId: 6,
      role: "button",
      label: "Apply",
      assertion: { type: "disabled", expected: false },
      wait: { timeout: 4_000, interval: 40 },
    },
    {
      action: "assertWindowState",
      windowId: 7,
      expected: { presence: "visible", surfaceGeneration: 2 },
      wait: { timeout: 5_000, interval: 50 },
    },
  ];

  await replayActions(
    actions,
    page,
    window,
    async (target) => {
      await target.snapshot();
    },
    async (_, action) =>
      record(action.windowId, "assertWindow", action.expected),
  );

  expect(observed).toEqual([
    [2, "button", "Save", 1, "click", { timeout: 2_000, interval: 20 }],
    [5, "status", "Ready", "wait", { timeout: 3_000, interval: 30 }],
    [
      3,
      "textbox",
      "Name",
      "key",
      "a",
      { shift: true, control: true, alt: false, meta: false },
      undefined,
    ],
    [4, "listbox", "Rows", "wheel", 5, 9, undefined],
    [6, "button", "Apply", "snapshot"],
    [7, "assertWindow", { presence: "visible", surfaceGeneration: 2 }],
  ]);
});
