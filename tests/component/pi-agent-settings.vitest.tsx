import { renderComponent } from "@wabou/test/component";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";
import {
  type AppSettings,
  SettingsPage,
} from "../../apps/pi-agent/ui/settings";
import { createAgentWorkspace } from "../../apps/pi-agent/ui/workspace";

function renderSettings(
  canDeleteProject = true,
  remove: () => void | Promise<void> = () => {},
  persistence: {
    loadError?: unknown;
    saveError?: unknown;
    reload?: () => void;
    retrySave?: () => void;
  } = {},
) {
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
      appLoadError={persistence.loadError}
      appSaveError={persistence.saveError}
      updateApp={(patch) =>
        setDefaults((current) => ({ ...current, ...patch }))
      }
      reloadApp={persistence.reload}
      retryAppSave={persistence.retrySave}
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
      deleteProject={async () => {
        await remove();
        deleted += 1;
      }}
      setAutoCompaction={() => {}}
      setSteeringMode={() => {}}
      setFollowUpMode={() => {}}
    />
  ));
  return { screen, defaults, agent, deleted: () => deleted };
}

async function settleAsyncAction() {
  await new Promise<void>((resolve) => setTimeout(resolve, 0));
}

test("Pi Agent settings separate project overrides from global network configuration", async () => {
  const { screen, defaults, agent, deleted } = renderSettings();

  screen.getByRole("label", { name: "Project name" }).click();
  const projectName = screen.getByRole("textbox", { name: "Project name" });
  expect(projectName.focused).toBe(true);
  projectName.input("Build project");
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

  const language = screen.getByRole("radiogroup", { name: "Language" });
  expect(language.className).toContain("flex-row");
  screen.getByRole("radio", { name: "中文" }).click();
  expect(defaults().locale).toBe("zh");
  expect(screen.getByRole("heading", { name: "设置" })).toBeDefined();
  screen.getByRole("radio", { name: "English" }).click();
  expect(defaults().locale).toBe("en");

  screen.getByRole("tab", { name: "Project settings" }).click();
  screen.getByRole("button", { name: "Delete project" }).click();
  const dialog = screen.getByRole("alertdialog", { name: "Delete project" });
  dialog.getByRole("button", { name: "Delete Build project?" }).click();
  await settleAsyncAction();
  expect(deleted()).toBe(1);
  expect(
    screen.queryByRole("alertdialog", { name: "Delete project" }),
  ).toBeNull();
});

test("Pi Agent keeps project deletion failures visible and retryable", async () => {
  let attempt = 0;
  const { screen, deleted } = renderSettings(true, async () => {
    attempt += 1;
    if (attempt === 1) throw new Error("host refused deletion");
  });

  screen.getByRole("button", { name: "Delete project" }).click();
  screen.getByRole("button", { name: "Delete Project 1?" }).click();
  await settleAsyncAction();
  expect(
    screen.getByRole("alert", { name: "Could not delete the project" }).text,
  ).toContain("host refused deletion");
  expect(
    screen.getByRole("alertdialog", { name: "Delete project" }),
  ).toBeDefined();
  screen.getByRole("button", { name: "Delete Project 1?" }).click();
  await settleAsyncAction();
  expect(deleted()).toBe(1);
  expect(attempt).toBe(2);
  expect(
    screen.queryByRole("alertdialog", { name: "Delete project" }),
  ).toBeNull();
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

test("Pi Agent settings expose recoverable persistence failures", () => {
  let reloads = 0;
  let retries = 0;
  const { screen } = renderSettings(true, () => {}, {
    loadError: new Error("settings database unavailable"),
    saveError: new Error("settings database is read-only"),
    reload: () => {
      reloads += 1;
    },
    retrySave: () => {
      retries += 1;
    },
  });

  screen.getByRole("tab", { name: "Application settings" }).click();
  const loadFailure = screen.getByRole("alert", {
    name: "Could not load application settings",
  });
  const saveFailure = screen.getByRole("alert", {
    name: "Could not save application settings",
  });
  expect(loadFailure.text).toContain("settings database unavailable");
  expect(saveFailure.text).toContain("settings database is read-only");

  loadFailure.getByRole("button", { name: "Try again" }).click();
  saveFailure.getByRole("button", { name: "Try again" }).click();
  expect(reloads).toBe(1);
  expect(retries).toBe(1);
});
