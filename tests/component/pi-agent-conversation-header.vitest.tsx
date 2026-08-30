import { renderComponent } from "@wabou/test/component";
import { expect, test, vi } from "vitest";
import { initialAgentState } from "../../apps/pi-agent/ui/agent-state";
import { ConversationHeader } from "../../apps/pi-agent/ui/conversation-header";
import { compactBranchLabel } from "../../apps/pi-agent/ui/conversation-context";

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
  screen.getByRole("button", { name: "Workspace files" }).click();
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
