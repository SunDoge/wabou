import { renderComponent } from "@wabou/test/component";
import {
  Button,
  createToasts,
  MotionConfigProvider,
  Toaster,
  View,
} from "@wabou/ui";
import { expect, test } from "vitest";

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
  expect(toast.transform).toEqual([1, 0, 0, 1, 0, 12]);
  screen.getByRole("button", { name: "Dismiss Saved" }).click();
  expect(screen.queryByRole("status", { name: "Saved" })).toBeNull();
  expect(toast.interactionBlocked).toBe(true);
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
  expect(screen.getByRole("alert", { name: "Download failed" })).not.toBeNull();
  screen.getByRole("button", { name: "Retry" }).click();
  expect(retried).toBe(1);
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
});
