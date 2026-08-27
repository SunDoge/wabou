import type { Dialog } from "@wabou/core";
import { renderComponent } from "@wabou/test/component";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";
import { WorkspaceSetup } from "../../apps/pi-agent/ui/workspace-setup";

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
  expect(JSON.stringify(screen.snapshot())).toContain("Choose a project");
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
