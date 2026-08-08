import { expect, test } from "bun:test";
import { createComponent, createRoot } from "solid-js";
import { type Clipboard, useClipboard } from "./clipboard";
import { PlatformProvider } from "./platform-context";
import { type WindowState, useWindow } from "./window-metrics";

test("PlatformProvider injects window-scoped services into useXxx hooks", () => {
  const fakeClipboard = {
    readText: async () => "injected",
    writeText: async () => {},
  };
  const metrics = () => ({
    windowId: 99,
    logicalWidth: 320,
    logicalHeight: 200,
    physicalWidth: 640,
    physicalHeight: 400,
    scaleFactor: 2,
    maximized: false,
    focused: true,
  });
  const fakeWindow: WindowState = {
    id: 99,
    close: () => {},
    setMaximized: () => {},
    setTitle: () => {},
    metrics,
    width: () => metrics().logicalWidth,
    height: () => metrics().logicalHeight,
    scaleFactor: () => metrics().scaleFactor,
    maximized: () => metrics().maximized,
    focused: () => metrics().focused,
  };
  let receivedClipboard: Clipboard | undefined;
  let receivedWindow: WindowState | undefined;

  createRoot((dispose) => {
    createComponent(PlatformProvider, {
      value: { clipboard: fakeClipboard, window: fakeWindow },
      get children() {
        receivedClipboard = useClipboard();
        receivedWindow = useWindow();
        return null;
      },
    });
    dispose();
  });

  expect(receivedClipboard).toBe(fakeClipboard);
  expect(receivedWindow).toBe(fakeWindow);
});
