import { renderComponent } from "@wabou/test/component";
import { expect, test, vi } from "vitest";
import { initialAgentState } from "../../apps/pi-agent/ui/agent-state";
import { compactBranchLabel } from "../../apps/pi-agent/ui/conversation-context";
import { ConversationHeader } from "../../apps/pi-agent/ui/conversation-header";

const readyState = {
  ...initialAgentState,
  connection: "ready" as const,
  sessionId: "session-1",
  items: [{ id: "user-1", kind: "user" as const, text: "Inspect it" }],
};

test("Pi Agent header preserves full semantics while compacting long branches", () => {
  const branch = "feature/conversation-header-that-is-deliberately-long";
  expect(compactBranchLabel(branch)).toHaveLength(28);
  expect(compactBranchLabel(branch)).toMatch(/^…/);

  const toggleFiles = vi.fn();
  const goBack = vi.fn();
  const screen = renderComponent(() => (
    <ConversationHeader
      project="Documentation workspace"
      session="Explain the renderer boundary"
      branch={branch}
      state={readyState}
      cwdAvailable
      repository
      terminalOpen={false}
      filesOpen={false}
      changesOpen={false}
      searchOpen={false}
      canGoBack
      canGoForward={false}
      goBack={goBack}
      toggleTerminal={() => {}}
      toggleFiles={toggleFiles}
      toggleChanges={() => {}}
      toggleSearch={() => {}}
      newSession={() => {}}
      compactSession={() => {}}
      cloneSession={() => {}}
      exportSession={() => {}}
      abort={() => {}}
    />
  ));

  expect(
    screen.getByRole("group", {
      name: `Documentation workspace, ${branch}, Explain the renderer boundary`,
    }),
  ).toBeTruthy();
  expect(
    screen.getByRole("toolbar", { name: "Conversation actions" }),
  ).toBeTruthy();
  const navigation = screen.getByRole("toolbar", {
    name: "Session navigation",
  });
  const previous = navigation.getByRole("button", { name: "Previous session" });
  const next = navigation.getByRole("button", {
    name: "Next session",
    disabled: true,
  });
  expect(previous.focusOrder).toBe(0);
  expect(next.focusOrder).toBe(-1);
  previous.click();
  expect(goBack).toHaveBeenCalledOnce();

  const actions = screen.getByRole("toolbar", { name: "Conversation actions" });
  const terminal = actions.getByRole("button", { name: "Toggle terminal" });
  const files = actions.getByRole("button", { name: "Workspace files" });
  const sessionActions = actions.getByRole("button", {
    name: "Session actions",
  });
  expect(terminal.pressed).toBe(false);
  terminal.focus();
  terminal.press("ArrowRight");
  expect(files.focused).toBe(true);
  files.press("End");
  expect(sessionActions.focused).toBe(true);
  files.click();
  expect(toggleFiles).toHaveBeenCalledOnce();
});

test("Pi Agent header swaps session actions for a stop action while running", () => {
  const abort = vi.fn();
  const screen = renderComponent(() => (
    <ConversationHeader
      project="Wabou"
      session="Current task"
      state={{ ...readyState, connection: "running" }}
      cwdAvailable
      repository={false}
      terminalOpen={false}
      filesOpen={false}
      changesOpen={false}
      searchOpen={false}
      toggleTerminal={() => {}}
      toggleFiles={() => {}}
      toggleChanges={() => {}}
      toggleSearch={() => {}}
      newSession={() => {}}
      compactSession={() => {}}
      cloneSession={() => {}}
      exportSession={() => {}}
      abort={abort}
    />
  ));

  expect(screen.queryByRole("button", { name: "New session" })).toBeNull();
  screen.getByRole("button", { name: "Stop" }).click();
  expect(abort).toHaveBeenCalledOnce();
});

test("Pi Agent prevents duplicate stop requests while one is pending", () => {
  const abort = vi.fn();
  const screen = renderComponent(() => (
    <ConversationHeader
      project="Wabou"
      session="Current task"
      state={{ ...readyState, connection: "running" }}
      cwdAvailable
      repository={false}
      terminalOpen={false}
      filesOpen={false}
      changesOpen={false}
      searchOpen={false}
      toggleTerminal={() => {}}
      toggleFiles={() => {}}
      toggleChanges={() => {}}
      toggleSearch={() => {}}
      newSession={() => {}}
      compactSession={() => {}}
      cloneSession={() => {}}
      exportSession={() => {}}
      abort={abort}
      abortPending
    />
  ));

  const stop = screen.getByRole("button", { name: "Stop", disabled: true });
  expect(screen.getByRole("status", { name: "Loading" })).toBeDefined();
  expect(() => stop.click()).toThrow("cannot click disabled component");
  expect(abort).not.toHaveBeenCalled();
});

test("Pi Agent disables new-session entry points while creation is pending", () => {
  const newSession = vi.fn();
  const screen = renderComponent(() => (
    <ConversationHeader
      project="Wabou"
      session="Current task"
      state={readyState}
      cwdAvailable
      repository={false}
      terminalOpen={false}
      filesOpen={false}
      changesOpen={false}
      searchOpen={false}
      toggleTerminal={() => {}}
      toggleFiles={() => {}}
      toggleChanges={() => {}}
      toggleSearch={() => {}}
      newSession={newSession}
      newSessionPending
      compactSession={() => {}}
      cloneSession={() => {}}
      exportSession={() => {}}
      abort={() => {}}
    />
  ));

  const create = screen.getByRole("button", {
    name: "New session",
    disabled: true,
  });
  expect(() => create.click()).toThrow("cannot click disabled");
  expect(newSession).not.toHaveBeenCalled();
});

test("Pi Agent keeps local workspace actions available while the process is stopped", () => {
  const toggleFiles = vi.fn();
  const screen = renderComponent(() => (
    <ConversationHeader
      project="Wabou"
      session="Local workspace"
      state={{ ...readyState, connection: "stopped" }}
      cwdAvailable
      repository
      terminalOpen={false}
      filesOpen={false}
      changesOpen={false}
      searchOpen={false}
      toggleTerminal={() => {}}
      toggleFiles={toggleFiles}
      toggleChanges={() => {}}
      toggleSearch={() => {}}
      newSession={() => {}}
      compactSession={() => {}}
      cloneSession={() => {}}
      exportSession={() => {}}
      abort={() => {}}
    />
  ));

  const files = screen.getByRole("button", { name: "Workspace files" });
  expect(files.disabled).toBe(false);
  files.click();
  expect(toggleFiles).toHaveBeenCalledOnce();
  expect(screen.queryByRole("button", { name: "New session" })).toBeNull();
  expect(screen.queryByRole("button", { name: "Stop" })).toBeNull();
});
