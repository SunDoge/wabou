import { renderComponent } from "@wabou/test/component";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";
import type { PiSession } from "../../apps/pi-agent/ui/api";
import {
  projectRowSelected,
  Sidebar,
} from "../../apps/pi-agent/ui/sidebar";
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
  let createdSession = false;
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
      newSession={() => (createdSession = true)}
      canCreateSession
      openSettings={() => {}}
    />
  ));

  const search = screen.getByRole("textbox", {
    name: "Search agents and sessions",
  });
  screen.getByRole("button", { name: "New thread" }).click();
  expect(createdSession).toBe(true);
  search.input("release notes");

  expect(screen.queryByRole("button", { name: "Project 1" })).toBeNull();
  expect(screen.getByRole("button", { name: "Project 2" })).toBeDefined();
  screen.getByRole("button", { name: "Write release notes" }).click();
  expect(selected).toEqual(["agent-2", "session-docs"]);

  search.input("missing workspace");
  expect(screen.getByRole("status").text).toBe(
    "No matching agents or sessions.",
  );
});

test("shows sessions only under the active agent and keeps agent selection live", () => {
  const agents = [createAgentWorkspace(1), createAgentWorkspace(2)];
  agents[0].cwd = "/work/api";
  agents[1].cwd = "/work/docs";
  agents[1].state.sessionId = "session-docs";
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
  const selected: string[] = [];
  const screen = renderComponent(() => (
    <Sidebar
      agents={agents}
      sessions={sessions}
      activeId="agent-2"
      select={(id) => selected.push(id)}
      selectSession={() => {}}
      add={() => {}}
      newSession={() => {}}
      canCreateSession
      openSettings={() => {}}
    />
  ));

  expect(screen.queryByRole("button", { name: "Fix API" })).toBeNull();
  expect(
    screen.getByRole("button", { name: "Write release notes" }),
  ).toBeDefined();
  expect(screen.getByRole("button", { name: "Project 2" }).selected).toBe(
    false,
  );
  expect(
    screen.getByRole("button", { name: "Write release notes" }).selected,
  ).toBe(true);
  screen.getByRole("button", { name: "Project 1" }).click();
  expect(selected).toEqual(["agent-1"]);
});

test("selects the project row only when it has no selected retained session", () => {
  const agent = createAgentWorkspace(1);
  const session: PiSession = {
    agentId: agent.id,
    sessionId: "retained-session",
    sessionFile: "/tmp/retained-session.jsonl",
    cwd: "/work/api",
    updatedAt: 1,
  };

  expect(projectRowSelected(agent, agent.id, [session])).toBe(true);
  agent.state.sessionId = session.sessionId;
  expect(projectRowSelected(agent, agent.id, [session])).toBe(false);
  expect(projectRowSelected(agent, "another-agent", [session])).toBe(false);
});

test("updates an existing project row when the active agent changes", () => {
  const agents = [createAgentWorkspace(1), createAgentWorkspace(2)];
  const [activeId, setActiveId] = createSignal(agents[0].id);
  const screen = renderComponent(() => (
    <Sidebar
      agents={agents}
      sessions={[]}
      activeId={activeId()}
      select={setActiveId}
      selectSession={() => {}}
      add={() => {}}
      newSession={() => {}}
      canCreateSession
      openSettings={() => {}}
    />
  ));

  const first = screen.getByRole("button", { name: "Project 1" });
  const second = screen.getByRole("button", { name: "Project 2" });
  expect(first.selected).toBe(true);
  expect(second.selected).toBe(false);

  second.click();
  screen.flush();
  expect(first.selected).toBe(false);
  expect(second.selected).toBe(true);
});

test("uses the session id when the persisted session name is blank", () => {
  const agent = createAgentWorkspace(1);
  agent.state.sessionId = "01a047c5-full-session-id";
  const screen = renderComponent(() => (
    <Sidebar
      agents={[agent]}
      sessions={[
        {
          agentId: agent.id,
          sessionId: "01a047c5-full-session-id",
          sessionFile: "/tmp/blank-name.jsonl",
          name: "   ",
          cwd: "/work/api",
          updatedAt: 1,
        },
      ]}
      activeId={agent.id}
      select={() => {}}
      selectSession={() => {}}
      add={() => {}}
      newSession={() => {}}
      canCreateSession
      openSettings={() => {}}
    />
  ));

  expect(screen.getByRole("button", { name: "01a047c5" }).text).toContain(
    "01a047c5",
  );
});

test("updates the selected session when a keyed agent receives new state", () => {
  const first = createAgentWorkspace(1);
  first.state.sessionId = "session-one";
  const sessions: PiSession[] = [
    {
      agentId: first.id,
      sessionId: "session-one",
      sessionFile: "/tmp/session-one.jsonl",
      name: "Session one",
      cwd: "/work/api",
      updatedAt: 1,
    },
    {
      agentId: first.id,
      sessionId: "session-two",
      sessionFile: "/tmp/session-two.jsonl",
      name: "Session two",
      cwd: "/work/api",
      updatedAt: 2,
    },
  ];
  const [agents, setAgents] = createSignal([first]);
  const [visibleSessions, setVisibleSessions] = createSignal(sessions);
  const screen = renderComponent(() => (
    <Sidebar
      agents={agents()}
      sessions={visibleSessions()}
      activeId={first.id}
      select={() => {}}
      selectSession={() => {}}
      add={() => {}}
      newSession={() => {}}
      canCreateSession
      openSettings={() => {}}
    />
  ));

  expect(screen.getByRole("button", { name: "Session one" }).selected).toBe(
    true,
  );
  setAgents((current) => [
    {
      ...current[0],
      state: { ...current[0].state, sessionId: "session-two" },
    },
  ]);
  setVisibleSessions([sessions[1], sessions[0]]);
  screen.flush();

  expect(screen.getByRole("button", { name: "Session one" }).selected).toBe(
    false,
  );
  expect(screen.getByRole("button", { name: "Session two" }).selected).toBe(
    true,
  );
});
