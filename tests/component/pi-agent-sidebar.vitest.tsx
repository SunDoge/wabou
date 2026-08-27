import { renderComponent } from "@wabou/test/component";
import { expect, test } from "vitest";
import type { PiSession } from "../../apps/pi-agent/ui/api";
import { Sidebar } from "../../apps/pi-agent/ui/sidebar";
import { createAgentWorkspace } from "../../apps/pi-agent/ui/workspace";

test("Pi Agent sidebar searches agents and sessions without flattening the hierarchy", () => {
  const agents = [createAgentWorkspace(1), createAgentWorkspace(2)];
  agents[0].cwd = "/work/api";
  agents[1].cwd = "/work/docs";
  const sessions: PiSession[] = [
    {
      agentId: "agent-1",
      sessionId: "session-api",
      sessionFile: "/tmp/session-api.jsonl",
      name: "Fix API",
      cwd: "/work/api",
      updatedAt: 1,
    },
    {
      agentId: "agent-2",
      sessionId: "session-docs",
      sessionFile: "/tmp/session-docs.jsonl",
      name: "Write release notes",
      cwd: "/work/docs",
      updatedAt: 2,
    },
  ];
  let selected: [string, string] | undefined;
  const screen = renderComponent(() => (
    <Sidebar
      agents={agents}
      sessions={sessions}
      activeId="agent-1"
      select={() => {}}
      selectSession={(agentId, sessionId) => {
        selected = [agentId, sessionId];
      }}
      add={() => {}}
      openSettings={() => {}}
    />
  ));

  const search = screen.getByRole("textbox", {
    name: "Search agents and sessions",
  });
  search.input("release notes");

  expect(screen.queryByRole("button", { name: "Agent 1" })).toBeNull();
  expect(screen.getByRole("button", { name: "Agent 2" })).toBeDefined();
  screen.getByRole("button", { name: "Write release notes" }).click();
  expect(selected).toEqual(["agent-2", "session-docs"]);

  search.input("missing workspace");
  expect(screen.getByRole("status").text).toBe(
    "No matching agents or sessions.",
  );
});
