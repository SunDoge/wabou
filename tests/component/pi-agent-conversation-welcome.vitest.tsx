import { renderComponent } from "@wabou/test/component";
import { expect, test, vi } from "vitest";
import { ConversationWelcome } from "../../apps/pi-agent/ui/conversation-welcome";

test("Pi Agent welcome offers understandable starter tasks", () => {
  const choosePrompt = vi.fn();
  const screen = renderComponent(() => (
    <ConversationWelcome choosePrompt={choosePrompt} />
  ));

  screen.getByRole("button", { name: "Understand this project" }).click();
  expect(choosePrompt).toHaveBeenCalledWith(
    "Explain this repository's structure and suggest a good first task.",
  );
  expect(screen.getByRole("button", { name: "Fix a problem" })).toBeDefined();
  expect(screen.getByRole("button", { name: "Plan a feature" })).toBeDefined();
});
