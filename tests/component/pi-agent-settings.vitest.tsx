import { renderComponent } from "@wabou/test/component";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";
import {
  type AppSettings,
  SettingsPage,
} from "../../apps/pi-agent/ui/settings";
import { createAgentWorkspace } from "../../apps/pi-agent/ui/workspace";

test("Pi Agent settings separate project overrides from global network configuration", () => {
  const [defaults, setDefaults] = createSignal<AppSettings>({
    locale: "en",
    proxy: "",
    noProxy: "127.0.0.1,localhost",
    provider: "",
    model: "",
    subagentsEnabled: true,
  });
  const [agent, setAgent] = createSignal(createAgentWorkspace(1));
  let deleted = 0;
  const screen = renderComponent(() => (
    <SettingsPage
      app={defaults()}
      updateApp={(patch) =>
        setDefaults((current) => ({ ...current, ...patch }))
      }
      project={agent()}
      state={{
        ...agent().state,
        connection: "ready",
        autoCompactionEnabled: true,
        steeringMode: "one-at-a-time",
        followUpMode: "one-at-a-time",
      }}
      updateProject={(patch) =>
        setAgent((current) => ({ ...current, ...patch }))
      }
      close={() => {}}
      deleteProject={() => deleted++}
      setAutoCompaction={() => {}}
      setSteeringMode={() => {}}
      setFollowUpMode={() => {}}
    />
  ));

  screen.getByRole("textbox", { name: "Project name" }).input("Build project");
  expect(agent().name).toBe("Build project");
  screen.getByRole("textbox", { name: "Workspace" }).input("/tmp/project");
  screen.getByRole("textbox", { name: "Provider" }).input("openai");
  screen.getByRole("textbox", { name: "Model" }).input("gpt-5");
  expect(agent()).toMatchObject({
    cwd: "/tmp/project",
    provider: "openai",
    model: "gpt-5",
  });
  screen
    .getByRole("textbox", { name: "Default proxy URL" })
    .input("http://127.0.0.1:7890");
  expect(defaults().proxy).toBe("http://127.0.0.1:7890");
  expect(agent()).not.toHaveProperty("proxy");
  const subagents = screen.getByRole("switch", { name: "Enable subagents" });
  expect(subagents.checked).toBe(true);
  subagents.click();
  expect(defaults().subagentsEnabled).toBe(false);

  screen.getByRole("button", { name: "中文" }).click();
  expect(defaults().locale).toBe("zh");
  expect(screen.getByRole("heading", { name: "设置" })).toBeDefined();
  screen.getByRole("button", { name: "English" }).click();
  expect(defaults().locale).toBe("en");

  screen.getByRole("button", { name: "Delete project" }).click();
  const dialog = screen.getByRole("alertdialog", { name: "Delete project" });
  expect(dialog.text).toContain("Delete Build project?");
  dialog.getByRole("button", { name: "Delete Build project?" }).click();
  expect(deleted).toBe(1);
});
