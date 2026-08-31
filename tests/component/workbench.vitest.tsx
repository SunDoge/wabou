import { renderComponent } from "@wabou/test/component";
import {
  Text,
  WorkbenchInspector,
  WorkbenchInspectorContent,
  WorkbenchInspectorHeader,
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
