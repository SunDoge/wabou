import { renderComponent } from "@wabou/test/component";
import {
  Tool,
  ToolContent,
  ToolHeader,
  ToolInput,
  ToolOutput,
  toolHeaderLabel,
} from "@wabou/ui";
import { expect, test } from "vitest";

test("tool header exposes a concise accessible status", () => {
  expect(toolHeaderLabel("read", "src/runtime.rs", "success")).toBe(
    "read: src/runtime.rs: Completed",
  );
});

test("tool anatomy owns disclosure, status and payload semantics", () => {
  const screen = renderComponent(() => (
    <Tool reducedMotion role="group" aria-label="Read source tool call">
      <ToolHeader title="read" summary="src/runtime.rs" status="success" />
      <ToolContent role="region" aria-label="Tool details">
        <ToolInput code={'{"path":"src/runtime.rs"}'} language="json" />
        <ToolOutput code="Loaded 120 lines" language="text" />
      </ToolContent>
    </Tool>
  ));

  expect(
    screen.getByRole("group", { name: "Read source tool call" }),
  ).toBeTruthy();
  const trigger = screen.getByRole("button", {
    name: "read: src/runtime.rs: Completed",
  });
  expect(trigger.expanded).toBe(false);
  expect(screen.queryByRole("region", { name: "Tool details" })).toBeNull();

  trigger.click();
  expect(trigger.expanded).toBe(true);
  expect(screen.getByRole("region", { name: "Tool details" }).text).toContain(
    "Loaded 120 lines",
  );
});
