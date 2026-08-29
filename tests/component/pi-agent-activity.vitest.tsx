import { renderComponent } from "@wabou/test/component";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";
import {
  AgentActivityStatus,
  AgentSidebarStatus,
} from "../../apps/pi-agent/ui/agent-activity";
import {
  type AgentViewState,
  initialAgentState,
} from "../../apps/pi-agent/ui/agent-state";

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

test("stays absent while an agent is idle with no queued work", () => {
  const screen = renderComponent(() => (
    <AgentActivityStatus
      state={{ ...initialAgentState, connection: "ready" }}
    />
  ));

  expect(screen.queryByRole("status")).toBeNull();
  expect(screen.roots[0]?.text).toBe("");
});

test("uses shape and an accessible label for exceptional sidebar states", () => {
  const running = renderComponent(() => (
    <AgentSidebarStatus
      state={{
        ...initialAgentState,
        connection: "running",
        activity: { kind: "compacting" },
      }}
    />
  ));
  expect(
    running.getByRole("status", { name: "Compacting context…" }),
  ).toBeDefined();
  running.dispose();

  const failed = renderComponent(() => (
    <AgentSidebarStatus
      state={{ ...initialAgentState, connection: "failed" }}
    />
  ));
  expect(
    failed.getByRole("status", { name: "Pi stopped with an error" }),
  ).toBeDefined();
});

test("keeps ready agents visually quiet", () => {
  const screen = renderComponent(() => (
    <AgentSidebarStatus state={{ ...initialAgentState, connection: "ready" }} />
  ));
  expect(screen.roots).toHaveLength(0);
});

test("tracks sidebar state changes without remounting", () => {
  const [state, setState] = createSignal<AgentViewState>({
    ...initialAgentState,
    connection: "ready",
  });
  const screen = renderComponent(() => <AgentSidebarStatus state={state()} />);

  expect(screen.queryByRole("status")).toBeNull();

  setState({
    ...initialAgentState,
    connection: "running",
    activity: { kind: "responding" },
  });
  screen.flush();
  expect(screen.getByRole("status", { name: "Responding…" })).toBeDefined();

  setState({ ...initialAgentState, connection: "failed" });
  screen.flush();
  expect(
    screen.getByRole("status", { name: "Pi stopped with an error" }),
  ).toBeDefined();
});
