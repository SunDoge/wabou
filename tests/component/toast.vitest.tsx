import { renderComponent } from "@wabou/test/component";
import { Button, createToasts, Toaster, View } from "@wabou/ui";
import { expect, test } from "vitest";

test("animates and retains a polite toast by default", async () => {
  const screen = renderComponent(
    () => {
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
    },
    { clock: "fake" },
  );

  screen.getByRole("button", { name: "Save" }).click();
  const toast = screen.getByRole("status", { name: "Saved" });
  expect(toast.text).toContain("Changes are on disk.");
  expect(toast.children[0]?.className).toContain("rounded-lg");
  expect(toast.children[0]?.className).toContain("border-strong");
  expect(toast.children[0]?.className).toContain("py-3.5");
  expect(toast.attribute("__wabou_native_transition")).toBeNull();
  expect(toast.transform).toEqual([1, 0, 0, 1, 0, 12]);
  expect(toast.style("opacity")).toBe("0");
  await screen.advanceTime(180);
  expect(toast.transform).toEqual([1, 0, 0, 1, 0, 0]);
  expect(toast.style("opacity")).toBe("1");
  screen.getByRole("button", { name: "Dismiss Saved" }).click();
  expect(screen.queryByRole("status", { name: "Saved" })).not.toBeNull();
  await screen.advanceTime(180);
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
        <Toaster toasts={toasts} motion={false} />
      </View>
    );
  });

  screen.getByRole("button", { name: "Fail" }).click();
  screen.getByRole("alert", { name: "Download failed" });
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
        <Toaster toasts={toasts} motion={false} />
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

test("an explicit motion contract uses the shared JS timeline", async () => {
  const screen = renderComponent(
    () => {
      const toasts = createToasts({ defaultDuration: 0 });
      return (
        <View>
          <Button onClick={() => toasts.success("Animated save")}>Save</Button>
          <Toaster toasts={toasts} motion={{ fromY: 12 }} />
        </View>
      );
    },
    { clock: "fake" },
  );

  screen.getByRole("button", { name: "Save" }).click();
  const toast = screen.getByRole("status", { name: "Animated save" });
  expect(toast.transform).toEqual([1, 0, 0, 1, 0, 12]);
  await screen.advanceTime(180);
  screen.getByRole("button", { name: "Dismiss Animated save" }).click();
  expect(
    screen.queryByRole("status", { name: "Animated save" }),
  ).not.toBeNull();
  await screen.advanceTime(180);
  expect(screen.queryByRole("status", { name: "Animated save" })).toBeNull();
});
