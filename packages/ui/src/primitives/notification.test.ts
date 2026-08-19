import { expect, test } from "bun:test";
import {
  createElement,
  dispatchEvent,
  EVENT_CODE,
  getMountRoot,
  mount,
} from "@wabou/core/renderer";
import { createRoot, flush, type JSX } from "solid-js";
import {
  createNotifications,
  type NotificationDismissReason,
  NotificationRegion,
} from "./notification";
import { View } from "./view";

test("notification queue enforces its limit and reports dismissal reasons", () => {
  createRoot((dispose) => {
    const dismissed: Array<[string, NotificationDismissReason]> = [];
    const notifications = createNotifications({ defaultDuration: 0, limit: 2 });
    const show = (label: string) =>
      notifications.show({
        "aria-label": label,
        content: () => createElement("view") as unknown as JSX.Element,
        onDismiss: (reason) => dismissed.push([label, reason]),
      });

    const first = show("First");
    flush();
    const second = show("Second");
    flush();
    const third = show("Third");
    flush();
    expect(notifications.items().map((item) => item.id)).toEqual([
      second,
      third,
    ]);
    expect(dismissed).toEqual([["First", "overflow"]]);
    expect(notifications.dismiss(first)).toBe(false);
    expect(notifications.dismiss(second, "dismiss")).toBe(true);
    flush();
    notifications.clear();
    flush();
    expect(notifications.items()).toEqual([]);
    expect(dismissed).toEqual([
      ["First", "overflow"],
      ["Second", "dismiss"],
      ["Third", "programmatic"],
    ]);
    dispose();
  });
});

test("notification timeout pauses and resumes", async () => {
  await new Promise<void>((resolve, reject) => {
    createRoot((dispose) => {
      const notifications = createNotifications({ defaultDuration: 15 });
      const id = notifications.show({
        "aria-label": "Saved",
        content: () => createElement("view") as unknown as JSX.Element,
      });
      flush();
      notifications.pause(id);
      notifications.pause(id);
      setTimeout(() => {
        try {
          expect(notifications.items()).toHaveLength(1);
          notifications.resume(id);
          setTimeout(() => {
            try {
              expect(notifications.items()).toHaveLength(1);
              notifications.resume(id);
              setTimeout(() => {
                try {
                  expect(notifications.items()).toHaveLength(0);
                  dispose();
                  resolve();
                } catch (error) {
                  reject(error);
                }
              }, 30);
            } catch (error) {
              reject(error);
            }
          }, 30);
        } catch (error) {
          reject(error);
        }
      }, 30);
    });
  });
});

test("NotificationRegion mounts non-blocking items on the floating plane", () => {
  const disposeMount = mount(() => null);
  const root = getMountRoot();
  const state = createRoot((dispose) => {
    const notifications = createNotifications({ defaultDuration: 0 });
    notifications.show({
      "aria-label": "Dismiss me",
      content: ({ dismiss }) => View({ onClick: dismiss }),
    });
    flush();
    NotificationRegion({ notifications });
    return { dispose, notifications };
  });
  const floatingPlane = root.lastChild;
  const region = floatingPlane?.firstChild;
  const item = region?.firstChild;
  const dismissTarget = item?.firstChild;
  expect(floatingPlane).not.toBeNull();
  expect(item).not.toBeNull();

  dispatchEvent(dismissTarget!.id, EVENT_CODE.click, "");
  flush();
  expect(state.notifications.items()).toEqual([]);
  expect(region?.firstChild).toBeNull();

  state.dispose();
  disposeMount();
});
