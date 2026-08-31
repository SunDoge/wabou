import { renderComponent } from "@wabou/test/component";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";
import {
  type AppSettings,
  SettingsPage,
} from "../../apps/pi-agent/ui/settings";
import { createAgentWorkspace } from "../../apps/pi-agent/ui/workspace";

function renderSettings(canDeleteProject = true) {
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
      canDeleteProject={canDeleteProject}
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
  return { screen, defaults, agent, deleted: () => deleted };
}

test("Pi Agent settings separate project overrides from global network configuration", () => {
  const { screen, defaults, agent, deleted } = renderSettings();

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
  expect(
    screen.queryByRole("textbox", { name: "Default proxy URL" }),
  ).toBeNull();

  screen.getByRole("tab", { name: "Application settings" }).click();
  expect(screen.queryByRole("textbox", { name: "Project name" })).toBeNull();
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

  screen.getByRole("tab", { name: "Project settings" }).click();
  screen.getByRole("button", { name: "Delete project" }).click();
  const dialog = screen.getByRole("alertdialog", { name: "Delete project" });
  expect(dialog.text).toContain("Delete Build project?");
  dialog.getByRole("button", { name: "Delete Build project?" }).click();
  expect(deleted()).toBe(1);
});

test("Pi Agent keeps the last project available", () => {
  const { screen, deleted } = renderSettings(false);
  const deleteProject = screen.getByRole("button", {
    name: "Delete project",
  });

  expect(deleteProject.disabled).toBe(true);
  expect(() => deleteProject.click()).toThrow("cannot click disabled");
  expect(deleted()).toBe(0);
  expect(
    screen.queryByRole("alertdialog", { name: "Delete project" }),
  ).toBeNull();
});
