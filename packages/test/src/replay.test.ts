import { expect, test } from "bun:test";
import type { WindowKey } from "@wabou/core";
import type { Locator, TestAction, TestContext, TestPage } from "./index";
import { replayActions } from "./replay";

test("replay preserves the logical window for every action", async () => {
  const observed: unknown[][] = [];
  const record = (...values: unknown[]): void => {
    observed.push(values);
  };
  const key = (lo: number, hi = 1): WindowKey => ({ lo, hi }) as WindowKey;
  const keyLabel = (value: WindowKey): string => `${value.lo}v${value.hi}`;
  const locator = (
    windowId: WindowKey,
    role: string,
    name: string,
    index?: number,
  ): Locator => ({
    windowId,
    role: role as Locator["role"],
    name,
    index,
    click: async (wait) =>
      record(keyLabel(windowId), role, name, index, "click", wait),
    dragBy: async (x, y, wait) =>
      record(keyLabel(windowId), role, name, "drag", x, y, wait),
    press: async (key, modifiers, wait) =>
      record(keyLabel(windowId), role, name, "key", key, modifiers, wait),
    type: async (text, wait) =>
      record(keyLabel(windowId), role, name, "text", text, wait),
    paste: async (text, wait) =>
      record(keyLabel(windowId), role, name, "paste", text, wait),
    ime: async (text, wait) =>
      record(keyLabel(windowId), role, name, "ime", text, wait),
    wheel: async (y, x, wait) =>
      record(keyLabel(windowId), role, name, "wheel", x, y, wait),
    waitFor: async (wait) =>
      record(keyLabel(windowId), role, name, "wait", wait),
    snapshot: async () => {
      record(keyLabel(windowId), role, name, "snapshot");
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
        current: null,
        expanded: null,
        focused: false,
      };
    },
  });
  const page = {
    effects: {
      respond: (operation: string, result: unknown) =>
        record("effect", operation, result),
    },
    forWindow(windowId: WindowKey): TestPage {
      return {
        ...page,
        getByRole: (role, options) =>
          locator(windowId, role, options.name, options.index),
      };
    },
    getByRole: (role, options) =>
      locator(key(1), role, options.name, options.index),
    waitForIdle: async () => {},
  } satisfies TestPage;
  const window: TestContext["window"] = {
    current: key(1),
    nativeClose: async (windowId, platform) =>
      record(keyLabel(windowId), "nativeClose", platform),
    show: async (windowId) => record(keyLabel(windowId), "show"),
    resize: async (windowId, width, height) =>
      record(keyLabel(windowId), "resize", width, height),
    fileDrop: async (windowId, phase, paths) =>
      record(keyLabel(windowId), "fileDrop", phase, paths),
    state: () => null,
  };
  const actions: TestAction[] = [
    {
      action: "respondToEffect",
      operation: "dialogPickDirectory",
      result: ["/tmp/downloads"],
    },
    {
      action: "resizeWindow",
      windowId: key(8),
      width: 900,
      height: 600,
    },
    {
      action: "fileDrop",
      windowId: key(9),
      phase: "dropped",
      paths: ["/tmp/example.torrent"],
    },
    {
      action: "clickByRole",
      windowId: key(2, 3),
      role: "button",
      label: "Save",
      index: 1,
      wait: { timeout: 2_000, interval: 20, stableFor: 0 },
    },
    {
      action: "waitForByRole",
      windowId: key(5),
      role: "status",
      label: "Ready",
      wait: { timeout: 3_000, interval: 30, stableFor: 0 },
    },
    {
      action: "inputByRole",
      windowId: key(3),
      role: "textbox",
      label: "Name",
      input: { type: "key", key: "a", modifiers: 3 },
    },
    {
      action: "inputByRole",
      windowId: key(4),
      role: "listbox",
      label: "Rows",
      input: { type: "wheel", deltaX: 5, deltaY: 9 },
    },
    {
      action: "assertByRole",
      windowId: key(6),
      role: "button",
      label: "Apply",
      assertion: { type: "disabled", expected: false },
      wait: { timeout: 4_000, interval: 40, stableFor: 400 },
    },
    {
      action: "assertWindowState",
      windowId: key(7),
      expected: { presence: "visible", surfaceGeneration: 2 },
      wait: { timeout: 5_000, interval: 50, stableFor: 0 },
    },
  ];

  await replayActions(
    actions,
    page,
    window,
    async (target, action) => {
      await target.snapshot();
      record("assertWait", action.wait);
    },
    async (_, action) =>
      record(keyLabel(action.windowId), "assertWindow", action.expected),
  );

  expect(observed).toEqual([
    ["effect", "dialogPickDirectory", ["/tmp/downloads"]],
    ["8v1", "resize", 900, 600],
    ["9v1", "fileDrop", "dropped", ["/tmp/example.torrent"]],
    [
      "2v3",
      "button",
      "Save",
      1,
      "click",
      { timeout: 2_000, interval: 20, stableFor: 0 },
    ],
    [
      "5v1",
      "status",
      "Ready",
      "wait",
      { timeout: 3_000, interval: 30, stableFor: 0 },
    ],
    [
      "3v1",
      "textbox",
      "Name",
      "key",
      "a",
      { shift: true, control: true, alt: false, meta: false },
      undefined,
    ],
    ["4v1", "listbox", "Rows", "wheel", 5, 9, undefined],
    ["6v1", "button", "Apply", "snapshot"],
    ["assertWait", { timeout: 4_000, interval: 40, stableFor: 400 }],
    ["7v1", "assertWindow", { presence: "visible", surfaceGeneration: 2 }],
  ]);
});
