import { renderComponent } from "@wabou/test/component";
import {
  Text,
  WorkbenchInspector,
  WorkbenchInspectorContent,
  WorkbenchInspectorHeader,
  WorkbenchInspectorState,
} from "@wabou/ui";
import { expect, test } from "vitest";

test("workbench inspectors expose one bounded region contract", () => {
  const screen = renderComponent(() => (
    <WorkbenchInspector role="region" aria-label="Workspace files">
      <WorkbenchInspectorHeader>
        <Text>Workspace files</Text>
      </WorkbenchInspectorHeader>
      <WorkbenchInspectorContent>
        <Text>README.md</Text>
      </WorkbenchInspectorContent>
    </WorkbenchInspector>
  ));

  const inspector = screen.getByRole("region", { name: "Workspace files" });
  expect(inspector.className).toContain("overflow-hidden");
  expect(inspector.children[0]?.className).toContain("flex-none");
  expect(inspector.children[1]?.className).toContain("min-h-0");
  expect(inspector.text).toContain("README.md");
});

test("workbench inspector states expose one mutually exclusive status owner", () => {
  const loading = renderComponent(() => (
    <WorkbenchInspectorState state="loading" title="Loading changes…" />
  ));
  expect(
    loading.getAllByRole("status", { name: "Loading changes…" }),
  ).toHaveLength(1);
  loading.dispose();

  const failed = renderComponent(() => (
    <WorkbenchInspectorState
      state="error"
      title="Could not load changes"
      description="permission denied"
    />
  ));
  const alert = failed.getByRole("alert", {
    name: "Could not load changes",
  });
  expect(alert.text).toContain("permission denied");
  expect(failed.queryByRole("status")).toBeNull();
});

test("workbench inspector state owns its standard recovery action", () => {
  let retries = 0;
  const screen = renderComponent(() => (
    <WorkbenchInspectorState
      state="error"
      title="Could not load changes"
      action={{ label: "Try again", onAction: () => retries++ }}
    />
  ));

  const retry = screen.getByRole("button", { name: "Try again" });
  expect(retry.snapshot().attributes).toMatchObject({
    "aria-label": "Try again",
  });
  retry.click();
  expect(retries).toBe(1);
});
