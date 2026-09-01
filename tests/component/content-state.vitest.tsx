import { renderComponent } from "@wabou/test/component";
import { ContentState } from "@wabou/ui";
import { expect, test } from "vitest";

test("content state exposes exactly one state owner", () => {
  const loading = renderComponent(() => (
    <ContentState state="loading" title="Loading workspace…" />
  ));
  expect(
    loading.getAllByRole("status", { name: "Loading workspace…" }),
  ).toHaveLength(1);
  loading.dispose();

  const failed = renderComponent(() => (
    <ContentState
      state="error"
      title="Could not load workspace"
      description="Permission denied while reading a deliberately long path"
    />
  ));
  const alert = failed.getByRole("alert", {
    name: "Could not load workspace",
  });
  expect(
    failed.getByRole("heading", { name: "Could not load workspace" }),
  ).toBeTruthy();
  expect(alert.text).toContain("Permission denied");
  expect(alert.className).toContain("min-w-0");
  expect(failed.queryByRole("status")).toBeNull();
});

test("content state owns its compact recovery action", () => {
  let retries = 0;
  const screen = renderComponent(() => (
    <ContentState
      state="error"
      title="Could not load workspace"
      action={{ label: "Try again", onAction: () => retries++ }}
    />
  ));

  screen.getByRole("button", { name: "Try again" }).click();
  expect(retries).toBe(1);
});
