import type { Dialog } from "@wabou/core";
import { renderComponent } from "@wabou/test/component";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";
import {
  WorkspaceSetup,
  WorkspaceSetupBoundary,
} from "../../apps/pi-agent/ui/workspace-setup";

const dialogWith = (pickDirectory: Dialog["pickDirectory"]): Dialog => ({
  open: async () => null,
  save: async () => null,
  pickDirectory,
  message: async () => "ok",
});

test("Pi Agent setup selects a workspace before starting", async () => {
  const start = vi.fn(async () => {});
  const App = () => {
    const [path, setPath] = createSignal("");
    return (
      <WorkspaceSetup
        path={path()}
        updatePath={setPath}
        start={start}
        openSettings={() => {}}
      />
    );
  };
  const screen = renderComponent(App, {
    platform: {
      dialog: dialogWith(async () => "/work/wabou"),
    },
  });
  const startButton = screen.getByRole("button", { name: "Start agent" });
  expect(JSON.stringify(screen.snapshot())).toContain(
    "Start your first coding agent",
  );
  expect(JSON.stringify(screen.snapshot())).not.toContain("Describe a task");
  expect(screen.getByRole("button", { name: "Review settings" })).toBeDefined();
  expect(startButton.disabled).toBe(true);

  screen.getByRole("button", { name: "Choose a repository" }).click();
  await screen.waitFor(() => expect(startButton.disabled).toBe(false));
  expect(screen.getByRole("textbox", { name: "Workspace" }).value).toBe(
    "/work/wabou",
  );

  startButton.click();
  await screen.waitFor(() => expect(start).toHaveBeenCalledOnce());
});

test("Pi Agent setup exposes recent runtime diagnostics", () => {
  const screen = renderComponent(() => (
    <WorkspaceSetup
      path="/work/wabou"
      runtimeLogs={["bun install failed", "proxy refused connection"]}
      updatePath={() => {}}
      start={async () => {}}
      openSettings={() => {}}
    />
  ));

  const output = screen.getByRole("group", { name: "Runtime output" });
  expect(output.text).toContain("bun install failed");
  expect(output.text).toContain("proxy refused connection");
});

test("Pi Agent setup keeps the directory form hidden while preparing its default workspace", () => {
  const screen = renderComponent(() => (
    <WorkspaceSetupBoundary
      pending
      path=""
      updatePath={() => {}}
      start={async () => {}}
      openSettings={() => {}}
    />
  ));

  expect(
    screen.getByRole("status", { name: "Preparing your workspace…" }),
  ).toBeDefined();
  expect(screen.queryByRole("button", { name: "Start agent" })).toBeNull();
});

test("Pi Agent setup reports startup progress and failures with shared status components", async () => {
  let finishStart: (() => void) | undefined;
  const start = new Promise<void>((resolve) => {
    finishStart = resolve;
  });
  const screen = renderComponent(() => (
    <WorkspaceSetup
      path="/work/wabou"
      error="Runtime failed to start"
      updatePath={() => {}}
      start={() => start}
      openSettings={() => {}}
    />
  ));

  expect(
    screen.getByRole("alert", { name: "Runtime failed to start" }).text,
  ).toContain("Runtime failed to start");
  screen.getByRole("button", { name: "Start agent" }).click();
  await screen.waitFor(() =>
    expect(
      screen.getByRole("button", {
        name: "Start agent",
        disabled: true,
      }).text,
    ).toContain("Starting"),
  );
  expect(screen.getByRole("status", { name: "Starting…" })).toBeDefined();

  finishStart?.();
  await screen.waitFor(() =>
    expect(screen.getByRole("button", { name: "Start agent" }).disabled).toBe(
      false,
    ),
  );
});
