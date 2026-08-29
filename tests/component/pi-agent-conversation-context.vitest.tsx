import { renderComponent } from "@wabou/test/component";
import { expect, test } from "vitest";
import { ConversationContext } from "../../apps/pi-agent/ui/conversation-context";
import { initialAgentState } from "../../apps/pi-agent/ui/agent-state";

test("exposes project, branch, and session as one conversation context", () => {
  const screen = renderComponent(() => (
    <ConversationContext
      project="Wabou"
      branch="feat/pi-agent-ui"
      session="Polish the conversation"
      state={initialAgentState}
    />
  ));

  expect(
    screen.getByRole("group", {
      name: "Wabou, feat/pi-agent-ui, Polish the conversation",
    }).text,
  ).toContain("Wabou/feat/pi-agent-ui/Polish the conversation");
});

test("omits an unavailable branch without leaving a duplicate separator", () => {
  const screen = renderComponent(() => (
    <ConversationContext
      project="Wabou"
      session="New session"
      state={initialAgentState}
    />
  ));

  expect(screen.getByRole("group", { name: "Wabou, New session" }).text).toBe(
    "Wabou/New session",
  );
});
