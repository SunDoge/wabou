import { expect, test } from "bun:test";
import { createComponent, createRoot } from "solid-js";
import { type Clipboard, useClipboard } from "./clipboard";
import { type Dialog, useDialog } from "./dialog";
import { type Notification, useNotification } from "./notification";
import { PlatformProvider } from "./platform-context";
import { windowKeyFromJSON } from "./window";
import { useWindow, type WindowState } from "./window-metrics";

const resolve = (value: unknown): unknown =>
  typeof value === "function" ? resolve(value()) : value;

test("PlatformProvider injects window-scoped services into useXxx hooks", () => {
  const windowKey = windowKeyFromJSON({ lo: 99, hi: 1 });
  const fakeClipboard = {
    readText: async () => "injected",
    writeText: async () => {},
  };
  const metrics = () => ({
    windowId: windowKey,
    logicalWidth: 320,
    logicalHeight: 200,
    physicalWidth: 640,
    physicalHeight: 400,
    scaleFactor: 2,
    maximized: false,
    focused: true,
    colorScheme: "light" as const,
  });
  const fakeWindow: WindowState = {
    id: windowKey,
    close: () => {},
    minimize: () => {},
    setMaximized: () => {},
    setTitle: () => {},
    startDragging: () => {},
    metrics,
    width: () => metrics().logicalWidth,
    height: () => metrics().logicalHeight,
    scaleFactor: () => metrics().scaleFactor,
    maximized: () => metrics().maximized,
    focused: () => metrics().focused,
    colorScheme: () => metrics().colorScheme,
  };
  const fakeDialog: Dialog = {
    open: async () => null,
    save: async () => null,
    pickDirectory: async () => null,
    message: async () => "ok",
  };
  const fakeNotification: Notification = { show: async () => {} };
  let receivedClipboard: Clipboard | undefined;
  let receivedDialog: Dialog | undefined;
  let receivedNotification: Notification | undefined;
  let receivedWindow: WindowState | undefined;

  createRoot((dispose) => {
    resolve(
      createComponent(PlatformProvider, {
        value: {
          clipboard: fakeClipboard,
          dialog: fakeDialog,
          notification: fakeNotification,
          window: fakeWindow,
        },
        get children() {
          receivedClipboard = useClipboard();
          receivedDialog = useDialog();
          receivedNotification = useNotification();
          receivedWindow = useWindow();
          return null;
        },
      }),
    );
    dispose();
  });

  expect(receivedClipboard).toBe(fakeClipboard);
  expect(receivedDialog).toBe(fakeDialog);
  expect(receivedNotification).toBe(fakeNotification);
  expect(receivedWindow).toBe(fakeWindow);
});

test("nested partial providers inherit services they do not override", () => {
  const windowKey = windowKeyFromJSON({ lo: 42, hi: 1 });
  const parentClipboard: Clipboard = {
    readText: async () => "parent",
    writeText: async () => {},
  };
  const childClipboard: Clipboard = {
    readText: async () => "child",
    writeText: async () => {},
  };
  const metrics = () => ({
    windowId: windowKey,
    logicalWidth: 1,
    logicalHeight: 1,
    physicalWidth: 1,
    physicalHeight: 1,
    scaleFactor: 1,
    maximized: false,
    focused: false,
    colorScheme: "light" as const,
  });
  const parentWindow: WindowState = {
    id: windowKey,
    close: () => {},
    minimize: () => {},
    setMaximized: () => {},
    setTitle: () => {},
    startDragging: () => {},
    metrics,
    width: () => 1,
    height: () => 1,
    scaleFactor: () => 1,
    maximized: () => false,
    focused: () => false,
    colorScheme: () => "light",
  };
  let receivedClipboard: Clipboard | undefined;
  let receivedWindow: WindowState | undefined;

  createRoot((dispose) => {
    resolve(
      createComponent(PlatformProvider, {
        value: { clipboard: parentClipboard, window: parentWindow },
        get children() {
          return createComponent(PlatformProvider, {
            value: { clipboard: childClipboard },
            get children() {
              receivedClipboard = useClipboard();
              receivedWindow = useWindow();
              return null;
            },
          });
        },
      }),
    );
    dispose();
  });

  expect(receivedClipboard).toBe(childClipboard);
  expect(receivedWindow).toBe(parentWindow);
});
