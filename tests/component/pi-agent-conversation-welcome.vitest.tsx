import { renderComponent } from "@wabou/test/component";
import { expect, test, vi } from "vitest";
import { ConversationWelcome } from "../../apps/pi-agent/ui/conversation-welcome";

test("Pi Agent welcome offers understandable starter tasks", () => {
  const choosePrompt = vi.fn();
  const screen = renderComponent(() => (
    <ConversationWelcome
      workspace="/tmp/pi-workspace"
      choosePrompt={choosePrompt}
    />
  ));

  expect(
    screen.getByRole("heading", { name: "Build pi-workspace with Pi" }),
  ).toBeDefined();
  expect(screen.roots[0]?.text).toContain(
    "Pi starts automatically when you send the first message.",
  );
  screen.getByRole("button", { name: "Review current changes" }).click();
  expect(choosePrompt).toHaveBeenCalledWith(
    "Review the current working tree, identify the highest-risk issue, and suggest the next concrete improvement.",
  );
  expect(
    screen.getByRole("button", { name: "Run project checks" }),
  ).toBeDefined();
  expect(screen.getByRole("button", { name: "Plan a feature" })).toBeDefined();
});
