import { renderComponent } from "@wabou/test/component";
import { expect, test } from "vitest";
import { AgentActivityStatus } from "../../apps/pi-agent/ui/agent-activity";
import { initialAgentState } from "../../apps/pi-agent/ui/agent-state";

test("shows the concrete agent activity and queued work", () => {
  const screen = renderComponent(() => (
    <AgentActivityStatus
      state={{
        ...initialAgentState,
        connection: "running",
        activity: { kind: "retrying", attempt: 2, maxAttempts: 3 },
        queue: { steering: 1, followUp: 2 },
      }}
    />
  ));

  expect(screen.getByRole("status").text).toBe("Retrying 2/3…");
  expect(screen.roots[0]?.text).toContain("3 queued");
});
