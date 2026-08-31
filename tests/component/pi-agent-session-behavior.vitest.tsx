import { renderComponent } from "@wabou/test/component";
import { expect, test } from "vitest";
import { initialAgentState } from "../../apps/pi-agent/ui/agent-state";
import { SessionBehaviorSettings } from "../../apps/pi-agent/ui/session-behavior-settings";

test("changes Pi session behavior through explicit controls", () => {
  const changes: string[] = [];
  const screen = renderComponent(() => (
    <SessionBehaviorSettings
      state={{
        ...initialAgentState,
        connection: "ready",
        autoCompactionEnabled: true,
        steeringMode: "one-at-a-time",
        followUpMode: "one-at-a-time",
      }}
      setAutoCompaction={(enabled) => changes.push(`compact:${enabled}`)}
      setSteeringMode={(mode) => changes.push(`steer:${mode}`)}
      setFollowUpMode={(mode) => changes.push(`follow:${mode}`)}
    />
  ));

  screen.getByRole("switch", { name: "Automatic context compaction" }).click();
  expect(changes).toContain("compact:false");
  screen.getByRole("label", { name: "Steering messages" }).click();
  const steering = screen.getByRole("combobox", { name: "Steering messages" });
  expect(steering.focused).toBe(true);
  steering.click();
  screen.getByRole("option", { name: "All queued messages" }).click();
  screen.getByRole("combobox", { name: "Follow-up messages" }).click();
  screen.getByRole("option", { name: "All queued messages" }).click();
  expect(changes).toContain("steer:all");
  expect(changes).toContain("follow:all");
});
