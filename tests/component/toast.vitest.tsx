import { renderComponent } from "@wabou/test/component";
import { Button, createToasts, Toaster, View } from "@wabou/ui";
import { expect, test } from "vitest";

const transitionOf = (toast: { attribute(name: string): string | null }) =>
  JSON.parse(toast.attribute("__wabou_native_transition") ?? "null");

test("shows and synchronously dismisses a polite toast by default", () => {
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
  expect(toast.attribute("__wabou_native_transition")).toBeNull();
  screen.getByRole("button", { name: "Dismiss Saved" }).click();
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

test("an explicit motion contract retains a toast until GPUI completes exit", () => {
  const screen = renderComponent(() => {
    const toasts = createToasts({ defaultDuration: 0 });
    return (
      <View>
        <Button onClick={() => toasts.success("Animated save")}>Save</Button>
        <Toaster toasts={toasts} motion={{ fromY: 12 }} />
      </View>
    );
  });

  screen.getByRole("button", { name: "Save" }).click();
  const toast = screen.getByRole("status", { name: "Animated save" });
  const entering = transitionOf(toast);
  toast.emit("transitionend", { generation: entering.generation });
  screen.getByRole("button", { name: "Dismiss Animated save" }).click();
  expect(
    screen.queryByRole("status", { name: "Animated save" }),
  ).not.toBeNull();
  const exiting = transitionOf(toast);
  toast.emit("transitionend", { generation: exiting.generation });
  expect(screen.queryByRole("status", { name: "Animated save" })).toBeNull();
});
