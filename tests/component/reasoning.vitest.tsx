import { renderComponent } from "@wabou/test/component";
import { Reasoning, ReasoningContent, ReasoningTrigger, Text } from "@wabou/ui";
import { expect, test } from "vitest";

test("reasoning anatomy exposes streaming state without owning model data", () => {
  const screen = renderComponent(() => (
    <Reasoning reducedMotion role="group" aria-label="Model reasoning">
      <ReasoningTrigger streaming />
      <ReasoningContent role="region" aria-label="Reasoning details">
        <Text>Inspecting the retained tree.</Text>
      </ReasoningContent>
    </Reasoning>
  ));

  const trigger = screen.getByRole("button", { name: "Thinking" });
  expect(trigger.expanded).toBe(false);
  expect(
    screen.queryByRole("region", { name: "Reasoning details" }),
  ).toBeNull();

  trigger.click();
  expect(trigger.expanded).toBe(true);
  expect(
    screen.getByRole("region", { name: "Reasoning details" }).text,
  ).toContain("Inspecting the retained tree.");
});
