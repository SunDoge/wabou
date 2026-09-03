import { renderComponent } from "@wabou/test/component";
import { expect, test } from "vitest";
import { ConversationWorkspaceStatus } from "../../apps/pi-agent/ui/conversation-workspace-status";

test("presents active workspace metadata through status bar groups", () => {
  const screen = renderComponent(() => (
    <ConversationWorkspaceStatus
      project="Wabou"
      branch="feat/gpui"
      repository
      connection="ready"
      runtimeLog="Runtime connected"
    />
  ));

  const status = screen.getByRole("status", { name: "Workspace status" });
  expect(status.className).toContain("max-w-4xl");
  expect(status.text).toContain("Wabou");
  expect(status.text).toContain("feat/gpui");
  expect(status.text).toContain("Runtime connected");
});

test("promotes failed workspace state to an alert", () => {
  const screen = renderComponent(() => (
    <ConversationWorkspaceStatus
      project="Wabou"
      repository={false}
      connection="failed"
      error="Runtime exited"
    />
  ));

  const alert = screen.getByRole("alert", { name: "Workspace status" });
  expect(alert.text).toContain("Runtime exited");
  expect(alert.text).toContain("error");
});
