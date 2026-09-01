import { renderComponent } from "@wabou/test/component";
import {
  Button,
  createToasts,
  MotionConfigProvider,
  Toaster,
  View,
} from "@wabou/ui";
import { expect, test } from "vitest";

const transitionOf = (toast: { attribute(name: string): string | null }) =>
  JSON.parse(toast.attribute("__wabou_native_transition") ?? "null");

test("shows and dismisses a polite toast", () => {
  const screen = renderComponent(() => {
    const toasts = createToasts({ defaultDuration: 0 });
    return (
      <View>
        <Button
          onClick={() =>
            toasts.success("Saved", { description: "Changes are on disk." })
          }
        >
          Save
        </Button>
        <Toaster toasts={toasts} />
      </View>
    );
  });

  screen.getByRole("button", { name: "Save" }).click();
  const toast = screen.getByRole("status", { name: "Saved" });
  expect(toast.text).toContain("Changes are on disk.");
  expect(toast.children[0]?.className).toContain("rounded-lg");
  expect(toast.children[0]?.className).toContain("border-subtle");
  expect(toast.children[0]?.className).toContain("py-3.5");
  expect(toast.transform).toEqual([1, 0, 0, 1, 0, 0]);
  const entering = transitionOf(toast);
  expect(entering).toMatchObject({
    duration: 0.18,
    easing: "easeOut",
    fromTransform: [1, 0, 0, 1, 0, 12],
    toTransform: [1, 0, 0, 1, 0, 0],
    fromOpacity: 0,
    toOpacity: 1,
  });
  toast.emit("transitionend", { generation: entering.generation });
  screen.getByRole("button", { name: "Dismiss Saved" }).click();
  expect(screen.queryByRole("status", { name: "Saved" })).not.toBeNull();
  expect(toast.interactionBlocked).toBe(true);
  expect(toast.attribute("aria-hidden")).toBe("true");
  const exiting = transitionOf(toast);
  expect(exiting).toMatchObject({
    fromTransform: [1, 0, 0, 1, 0, 0],
    toTransform: [1, 0, 0, 1, 0, 12],
    fromOpacity: 1,
    toOpacity: 0,
  });
  toast.emit("transitionend", { generation: entering.generation });
  expect(screen.queryByRole("status", { name: "Saved" })).not.toBeNull();
  toast.emit("transitionend", { generation: exiting.generation });
  expect(screen.queryByRole("status", { name: "Saved" })).toBeNull();
});

test("destructive toasts are assertive and actions dismiss by default", () => {
  let retried = 0;
  const screen = renderComponent(() => {
    const toasts = createToasts({ defaultDuration: 0 });
    return (
      <View>
        <Button
          onClick={() =>
            toasts.error("Download failed", {
              action: { label: "Retry", onAction: () => retried++ },
            })
          }
        >
          Fail
        </Button>
        <Toaster toasts={toasts} />
      </View>
    );
  });

  screen.getByRole("button", { name: "Fail" }).click();
  const toast = screen.getByRole("alert", { name: "Download failed" });
  const entering = transitionOf(toast);
  toast.emit("transitionend", { generation: entering.generation });
  screen.getByRole("button", { name: "Retry" }).click();
  expect(retried).toBe(1);
  expect(screen.queryByRole("alert")).not.toBeNull();
  const exiting = transitionOf(toast);
  toast.emit("transitionend", { generation: exiting.generation });
  expect(screen.queryByRole("alert")).toBeNull();
});

test("queue limits still use the primitive overflow policy", () => {
  const dismissed: string[] = [];
  const screen = renderComponent(() => {
    const toasts = createToasts({ defaultDuration: 0, limit: 1 });
    return (
      <View>
        <Button
          onClick={() => {
            toasts.show({
              title: "First",
              onDismiss: (reason) => dismissed.push(reason),
            });
            toasts.show({ title: "Second" });
          }}
        >
          Queue
        </Button>
        <Toaster toasts={toasts} />
      </View>
    );
  });

  screen.getByRole("button", { name: "Queue" }).click();
  // Both commands share one Solid transaction, so the overflowed item never
  // enters the authored tree and therefore has no visual exit to retain.
  expect(screen.queryByRole("status", { name: "First" })).toBeNull();
  expect(screen.getByRole("status", { name: "Second" })).not.toBeNull();
  expect(dismissed).toEqual(["overflow"]);
});

test("reduced motion publishes the final toast transform immediately", () => {
  const screen = renderComponent(() => {
    const toasts = createToasts({ defaultDuration: 0 });
    return (
      <MotionConfigProvider reducedMotion>
        <Button onClick={() => toasts.success("Quietly saved")}>Save</Button>
        <Toaster toasts={toasts} />
      </MotionConfigProvider>
    );
  });

  screen.getByRole("button", { name: "Save" }).click();
  expect(
    screen.getByRole("status", { name: "Quietly saved" }).transform,
  ).toEqual([1, 0, 0, 1, 0, 0]);
  expect(
    screen
      .getByRole("status", { name: "Quietly saved" })
      .attribute("__wabou_native_transition"),
  ).toBeNull();
});
