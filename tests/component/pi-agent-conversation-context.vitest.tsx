import { renderComponent } from "@wabou/test/component";
import { expect, test } from "vitest";
import { initialAgentState } from "../../apps/pi-agent/ui/agent-state";
import { ConversationContext } from "../../apps/pi-agent/ui/conversation-context";

test("exposes project, branch, and session as one conversation context", () => {
  const screen = renderComponent(() => (
    <ConversationContext
      project="Wabou"
      branch="feat/pi-agent-ui"
      session="Polish the conversation"
      state={initialAgentState}
    />
  ));

  const breadcrumb = screen.getByRole("group", {
    name: "Wabou, feat/pi-agent-ui, Polish the conversation",
  });
  expect(
    breadcrumb
      .getByRole("link", { name: "Polish the conversation" })
      .attribute("aria-current"),
  ).toBe("page");
});

test("omits an unavailable branch while preserving the current session", () => {
  const screen = renderComponent(() => (
    <ConversationContext
      project="Wabou"
      session="New session"
      state={initialAgentState}
    />
  ));

  const breadcrumb = screen.getByRole("group", { name: "Wabou, New session" });
  expect(breadcrumb.text).not.toContain("undefined");
  expect(breadcrumb.getByRole("link", { name: "New session" })).toBeTruthy();
});
