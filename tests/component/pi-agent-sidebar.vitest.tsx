import { renderComponent } from "@wabou/test/component";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";
import { Sidebar } from "../../apps/pi-agent/ui/sidebar";
import {
  type AgentWorkspace,
  createAgentWorkspace,
} from "../../apps/pi-agent/ui/workspace";

test("returns to an existing agent after creating a new one", () => {
  const [agents, setAgents] = createSignal<readonly AgentWorkspace[]>([
    createAgentWorkspace(1),
  ]);
  const [activeId, setActiveId] = createSignal("agent-1");

  const screen = renderComponent(() => (
    <Sidebar
      agents={agents()}
      activeId={activeId()}
      select={setActiveId}
      add={() => {
        const agent = createAgentWorkspace(2);
        setAgents((current) => [...current, agent]);
        setActiveId(agent.id);
      }}
      openSettings={() => {}}
    />
  ));

  screen.getByRole("button", { name: "New agent" }).click();
  expect(activeId()).toBe("agent-2");
  expect(screen.getByRole("button", { name: "Agent 2" }).selected).toBe(true);

  screen.getByRole("button", { name: "Agent 1" }).click();
  expect(activeId()).toBe("agent-1");
  expect(screen.getByRole("button", { name: "Agent 1" }).selected).toBe(true);
});
