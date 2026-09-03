import { renderComponent } from "@wabou/test/component";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";
import type { PiSession } from "../../apps/pi-agent/ui/api";
import { Sidebar } from "../../apps/pi-agent/ui/sidebar";
import {
  activeSidebarValue,
  groupSessionsByRecency,
  nextSessionClockDelay,
  sessionRecency,
  sessionTimeLabel,
  sortSessionsByRecency,
} from "../../apps/pi-agent/ui/sidebar-presentation";
import { createAgentWorkspace } from "../../apps/pi-agent/ui/workspace";

test("describes session recency without depending on wall-clock time", () => {
  expect(sessionRecency(10_000, 10_059)).toEqual({ kind: "now" });
  expect(sessionRecency(10_000, 10_300)).toEqual({
    kind: "minutes",
    value: 5,
  });
  expect(sessionRecency(10_000, 17_200)).toEqual({
    kind: "hours",
    value: 2,
  });
  expect(sessionTimeLabel(10_000, 10_300, "en")).toBe("5m");
  expect(sessionTimeLabel(10_000, 10_300, "zh")).toBe("5 分钟");
});

test("schedules only the next visible session-time transition", () => {
  expect(nextSessionClockDelay([], 100)).toBeUndefined();
  expect(nextSessionClockDelay([{ updatedAt: 100 }], 159)).toBe(1);
  expect(nextSessionClockDelay([{ updatedAt: 100 }], 160)).toBe(60);
  expect(nextSessionClockDelay([{ updatedAt: 100 }], 3_699)).toBe(1);
  expect(
    nextSessionClockDelay([{ updatedAt: 200 }, { updatedAt: 3_650 }], 3_699),
  ).toBe(11);
});

test("refreshes relative session labels at their next visible boundary", () => {
  vi.useFakeTimers();
  vi.setSystemTime(new Date(100_000));
  const agent = createAgentWorkspace(1);
  const screen = renderComponent(() => (
    <Sidebar
      agents={[agent]}
      sessions={[
        {
          agentId: agent.id,
          sessionId: "recent",
          sessionFile: "/tmp/recent.jsonl",
          name: "Recent work",
          cwd: "/work/api",
          updatedAt: 100,
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

  expect(screen.getByRole("button", { name: "Recent work" }).text).toContain(
    "Now",
  );
  vi.advanceTimersByTime(60_000);
  screen.flush();
  expect(screen.getByRole("button", { name: "Recent work" }).text).toContain(
    "1m",
  );
  screen.dispose();
  vi.useRealTimers();
});

test("keeps the new-thread control mounted across connection changes", () => {
  const [canCreateSession, setCanCreateSession] = createSignal(false);
  const agent = createAgentWorkspace(1);
  const screen = renderComponent(() => (
    <Sidebar
      agents={[agent]}
      sessions={[]}
      activeId={agent.id}
      select={() => {}}
      selectSession={() => {}}
      add={() => {}}
      newSession={() => {}}
      canCreateSession={canCreateSession()}
      openSettings={() => {}}
    />
  ));

  const before = screen.getByRole("button", { name: "New thread" });
  expect(before.disabled).toBe(true);
  const identity = before.identity;

  setCanCreateSession(true);
  screen.flush();

  const after = screen.getByRole("button", { name: "New thread" });
  expect(after.disabled).toBe(false);
  expect(after.identity).toEqual(identity);
});

test("sorts retained sessions newest-first with a deterministic tie break", () => {
  const sessions = [
    { sessionId: "b", updatedAt: 20 },
    { sessionId: "c", updatedAt: 30 },
    { sessionId: "a", updatedAt: 20 },
  ] as PiSession[];

  expect(sortSessionsByRecency(sessions).map((item) => item.sessionId)).toEqual(
    ["c", "a", "b"],
  );
  expect(sessions.map((item) => item.sessionId)).toEqual(["b", "c", "a"]);
});

test("groups retained sessions into stable scan ranges", () => {
  const now = new Date(2026, 7, 30, 12, 0, 0).getTime() / 1_000;
  const sessions = [
    { sessionId: "older", updatedAt: now - 10 * 86_400 },
    { sessionId: "today", updatedAt: now - 60 },
    { sessionId: "week", updatedAt: now - 3 * 86_400 },
  ] as PiSession[];

  expect(
    groupSessionsByRecency(sessions, now).map((group) => ({
      key: group.key,
      sessions: group.sessions.map((session) => session.sessionId),
    })),
  ).toEqual([
    { key: "today", sessions: ["today"] },
    { key: "previous-seven-days", sessions: ["week"] },
    { key: "older", sessions: ["older"] },
  ]);
});

test("does not expose managed workspace ids as project subtitles", () => {
  const agent = createAgentWorkspace(1);
  agent.cwd = "/data/pi-agent/workspaces/agent-1";
  const screen = renderComponent(() => (
    <Sidebar
      agents={[agent]}
      sessions={[]}
      activeId={agent.id}
      select={() => {}}
      selectSession={() => {}}
      add={() => {}}
      newSession={() => {}}
      canCreateSession
      openSettings={() => {}}
    />
  ));

  expect(screen.getByRole("button", { name: "Project 1" }).text).toBe(
    "Project 1",
  );
});

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

test("shows retained sessions for every project and keeps navigation live", () => {
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
  const selectedSessions: [string, string][] = [];
  const screen = renderComponent(() => (
    <Sidebar
      agents={agents}
      sessions={sessions}
      activeId="agent-2"
      select={(id) => selected.push(id)}
      selectSession={(agentId, sessionId) =>
        selectedSessions.push([agentId, sessionId])
      }
      add={() => {}}
      newSession={() => {}}
      canCreateSession
      nowSeconds={61}
      openSettings={() => {}}
    />
  ));

  const inactiveSession = screen.getByRole("button", { name: "Fix API" });
  expect(inactiveSession.selected).toBe(false);
  expect(
    screen.getByRole("button", { name: "Write release notes" }),
  ).toBeDefined();
  expect(screen.getByRole("button", { name: "Project 2" }).selected).toBe(
    false,
  );
  expect(
    screen.getByRole("button", { name: "Write release notes" }).selected,
  ).toBe(true);
  expect(
    screen.getByRole("button", { name: "Write release notes" }).text,
  ).toContain("Now");
  expect(
    screen
      .getAllByRole("button")
      .map((button) => button.name)
      .filter((name) => name === "Fix API" || name === "Write release notes"),
  ).toEqual(["Fix API", "Write release notes"]);
  inactiveSession.click();
  expect(selectedSessions).toEqual([["agent-1", "session-api"]]);
  screen.getByRole("button", { name: "Project 1" }).click();
  expect(selected).toEqual(["agent-1"]);
});

test("keeps the new-thread action when the first retained session appears", () => {
  const agent = createAgentWorkspace(1);
  const [sessions, setSessions] = createSignal<readonly PiSession[]>([]);
  const screen = renderComponent(() => (
    <Sidebar
      agents={[agent]}
      sessions={sessions()}
      activeId={agent.id}
      select={() => {}}
      selectSession={() => {}}
      add={() => {}}
      newSession={() => {}}
      canCreateSession
      openSettings={() => {}}
    />
  ));

  expect(screen.getByRole("button", { name: "New thread" })).toBeDefined();
  expect(
    screen.queryByRole("textbox", { name: "Search agents and sessions" }),
  ).toBeNull();

  setSessions([
    {
      agentId: agent.id,
      sessionId: "first-session",
      sessionFile: "/tmp/first-session.jsonl",
      name: "First session",
      cwd: "/work/api",
      updatedAt: 1,
    },
  ]);
  screen.flush();

  expect(screen.getByRole("button", { name: "New thread" })).toBeDefined();
  expect(
    screen.getByRole("textbox", { name: "Search agents and sessions" }),
  ).toBeDefined();
  expect(screen.getByRole("button", { name: "First session" })).toBeDefined();
});

test("collapses project sessions without hiding search results or changing selection", () => {
  const agent = createAgentWorkspace(1);
  agent.cwd = "/work/api";
  const sessions: PiSession[] = [
    {
      agentId: agent.id,
      sessionId: "session-one",
      sessionFile: "/tmp/session-one.jsonl",
      name: "Fix API retries",
      cwd: agent.cwd,
      updatedAt: 100,
    },
    {
      agentId: agent.id,
      sessionId: "session-two",
      sessionFile: "/tmp/session-two.jsonl",
      name: "Document transport",
      cwd: agent.cwd,
      updatedAt: 90,
    },
  ];
  const screen = renderComponent(() => (
    <Sidebar
      agents={[agent]}
      sessions={sessions}
      activeId={agent.id}
      select={() => {}}
      selectSession={() => {}}
      add={() => {}}
      newSession={() => {}}
      canCreateSession
      nowSeconds={120}
      openSettings={() => {}}
    />
  ));

  screen
    .getByRole("button", { name: "Collapse sessions for Project 1" })
    .click();
  screen.flush();
  expect(screen.queryByRole("button", { name: "Fix API retries" })).toBeNull();
  expect(screen.getByRole("button", { name: "Project 1" }).selected).toBe(true);

  const search = screen.getByRole("textbox", {
    name: "Search agents and sessions",
  });
  search.input("transport");
  expect(
    screen.getByRole("button", { name: "Document transport" }),
  ).toBeDefined();

  search.input("");
  expect(
    screen.queryByRole("button", { name: "Document transport" }),
  ).toBeNull();
  screen.getByRole("button", { name: "Expand sessions for Project 1" }).click();
  expect(screen.getByRole("button", { name: "Fix API retries" })).toBeDefined();
});

test("resolves one active sidebar destination from project and session state", () => {
  const agent = createAgentWorkspace(1);
  const session: PiSession = {
    agentId: agent.id,
    sessionId: "retained-session",
    sessionFile: "/tmp/retained-session.jsonl",
    cwd: "/work/api",
    updatedAt: 1,
  };

  expect(activeSidebarValue([agent], agent.id, [session])).toBe(
    `project:${agent.id}`,
  );
  agent.state.sessionId = session.sessionId;
  expect(activeSidebarValue([agent], agent.id, [session])).toBe(
    `session:${agent.id}:${session.sessionId}`,
  );
  expect(activeSidebarValue([agent], "another-agent", [session])).toBe(
    undefined,
  );
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
      nowSeconds={301}
      openSettings={() => {}}
    />
  ));

  const session = screen.getByRole("button", { name: "01a047c5" });
  expect(session.text).toContain("01a047c5");
  expect(
    session.children.find((child) => child.text === "5m")?.className,
  ).toContain("text-secondary");
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
  screen
    .getByRole("button", { name: "Collapse sessions for Project 1" })
    .click();
  expect(screen.queryByRole("button", { name: "Session one" })).toBeNull();
  setAgents((current) => [
    {
      ...current[0],
      state: { ...current[0].state, sessionId: "session-two" },
    },
  ]);
  setVisibleSessions([sessions[1], sessions[0]]);
  screen.flush();

  expect(screen.getByRole("button", { name: "Session one" })).toBeDefined();
  expect(screen.getByRole("button", { name: "Session one" }).selected).toBe(
    false,
  );
  expect(screen.getByRole("button", { name: "Session two" }).selected).toBe(
    true,
  );
});

test("exposes Skills as a stable application destination", () => {
  const agent = createAgentWorkspace(1);
  const openSkills = vi.fn();
  const openSettings = vi.fn();
  const screen = renderComponent(() => (
    <Sidebar
      agents={[agent]}
      sessions={[]}
      activeId={agent.id}
      activePage="skills"
      select={() => {}}
      selectSession={() => {}}
      add={() => {}}
      newSession={() => {}}
      canCreateSession
      openSkills={openSkills}
      openSettings={openSettings}
    />
  ));

  const skills = screen.getByRole("button", { name: "Skills" });
  expect(skills.selected).toBe(true);
  skills.click();
  expect(openSkills).toHaveBeenCalledTimes(1);
  screen.getByRole("button", { name: "Settings" }).click();
  expect(openSettings).toHaveBeenCalledTimes(1);
});
