import { renderComponent } from "@wabou/test/component";
import { ContentState, ResourceBoundary, Text } from "@wabou/ui";
import { createSignal } from "solid-js";
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

test("resource boundary keeps usable content mounted during refresh", () => {
  const [loading, setLoading] = createSignal(true);
  const [hasContent, setHasContent] = createSignal(false);
  const screen = renderComponent(() => (
    <ResourceBoundary
      loading={loading()}
      hasContent={hasContent()}
      loadingTitle="Loading files…"
      errorTitle="Could not load files"
      emptyTitle="No files"
      renderContent={() => <Text>README.md</Text>}
    />
  ));

  expect(screen.getByRole("status", { name: "Loading files…" })).toBeTruthy();
  setHasContent(true);
  screen.flush();
  expect(screen.queryByRole("status", { name: "Loading files…" })).toBeNull();
  expect(screen.roots[0]?.text).toContain("README.md");

  setLoading(false);
  screen.flush();
  expect(screen.roots[0]?.text).toContain("README.md");
});

test("resource boundary gives errors precedence and owns retry", () => {
  const [error, setError] = createSignal<unknown>();
  let retries = 0;
  const screen = renderComponent(() => (
    <ResourceBoundary
      loading={false}
      error={error()}
      hasContent={true}
      loadingTitle="Loading files…"
      errorTitle="Could not load files"
      emptyTitle="No files"
      retryLabel="Try again"
      onRetry={() => retries++}
      renderContent={() => <Text>README.md</Text>}
    />
  ));

  expect(screen.roots[0]?.text).toContain("README.md");
  setError(new Error("permission denied"));
  screen.flush();
  const alert = screen.getByRole("alert", { name: "Could not load files" });
  expect(alert.text).toContain("permission denied");
  expect(screen.roots[0]?.text).not.toContain("README.md");
  screen.getByRole("button", { name: "Try again" }).click();
  expect(retries).toBe(1);
});
