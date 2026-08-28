import { expect, test } from "@wabou/test";

test("starts a deterministic Pi agent and renders its streamed response", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Start agent" }).click();

  const composer = page.getByRole("textbox", {
    name: "Ask this agent to work in its repository…",
  });
  await composer.waitFor();
  await composer.type("Explain the fixture");
  await page.getByRole("button", { name: "Send" }).click();

  await expect(
    page.getByRole("label", {
      name: "Fake Pi completed: Explain the fixture",
    }),
  ).toHaveCount(1, { timeout: 5_000 });
  await expect(
    page.getByRole("button", { name: "read: README.md" }),
  ).toHaveCount(1);

  await page.getByRole("button", { name: "Commands" }).click();
  const subagents = page.getByRole("option", { name: "/subagents" });
  await expect(subagents).toBeInViewport();
  await subagents.click();
  await expect(composer).toHaveValue("/subagents ");
  await composer.press("a", { control: true });
  await composer.press("Backspace");
  await expect(composer).toHaveValue("");
});

test("keeps the active workspace usable at its minimum window size", async ({
  page,
  window,
}) => {
  await window.resize(window.current, 1180, 680);

  const composer = page.getByRole("textbox", {
    name: "Ask this agent to work in its repository…",
  });
  const send = page.getByRole("button", { name: "Send" });
  const newThread = page.getByRole("button", { name: "New thread" });
  const model = page.getByRole("combobox", { name: "Choose model" });
  for (const control of [composer, send, newThread, model]) {
    await expect(control).toBeInViewport();
  }
  await expect(newThread).toNotOverlap(composer);

  const terminalToggle = page.getByRole("button", { name: "Toggle terminal" });
  await terminalToggle.click();
  await expect(
    page.getByRole("region", { name: "Terminal panel" }),
  ).toBeInViewport();
  await expect(composer).toBeInViewport();
  await page.getByRole("button", { name: "Close terminal panel" }).click();
});

test("returns to an existing agent after creating a new one", async ({
  page,
}) => {
  const first = page.getByRole("button", { name: "Project 1" });
  await expect(first).toBeSelected();

  await page.getByRole("button", { name: "Add project" }).click();
  const second = page.getByRole("button", { name: "Project 2" });
  await expect(second).toBeSelected();

  await first.click();
  await expect(first).toBeSelected();
  await expect(second).toBeDeselected();
  await expect(
    page.getByRole("label", {
      name: "Fake Pi completed: Explain the fixture",
    }),
  ).toHaveCount(1);
});

test("changes model through the native combobox overlay", async ({ page }) => {
  const model = page.getByRole("combobox", { name: "Choose model" });
  await model.click();
  await page.getByRole("option", { name: "Alternative model" }).click();
  await expect(model).toHaveValue("Alternative model");
  await expect(
    page.getByRole("label", { name: "Alternative model · medium thinking" }),
  ).toHaveCount(1);
});

test("updates project and app settings without losing its conversation", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Settings" }).click();
  await page.getByRole("heading", { name: "Settings" }).waitFor();
  const name = page.getByRole("textbox", { name: "Project name" });
  await expect(name).toBeInViewport();
  await name.click();
  await name.press("a", { control: true });
  await name.type("Workspace Agent");
  await expect(name).toHaveValue("Workspace Agent");
  const provider = page.getByRole("textbox", { name: "Provider" });
  await provider.type("openai");
  await expect(provider).toHaveValue("openai");
  const configuredModel = page.getByRole("textbox", { name: "Model" });
  await configuredModel.type("gpt-5");
  await expect(configuredModel).toHaveValue("gpt-5");
  await configuredModel.wheel(1_200);
  const proxy = page.getByRole("textbox", { name: "Default proxy URL" });
  await proxy.type("http://127.0.0.1:7890");
  await expect(proxy).toHaveValue("http://127.0.0.1:7890");
  const subagents = page.getByRole("switch", { name: "Enable subagents" });
  await expect(subagents).toBeChecked();
  await subagents.click();
  await expect(subagents).toBeUnchecked();
  await page.getByRole("button", { name: "Back to projects" }).click();
  await expect(
    page.getByRole("button", { name: "Workspace Agent" }),
  ).toBeSelected();
  await expect(
    page.getByRole("label", {
      name: "Fake Pi completed: Explain the fixture",
    }),
  ).toHaveCount(1);
});

test("creates a fresh session and restores the previous transcript", async ({
  page,
}) => {
  const previousSession = page.getByRole("button", {
    name: "Deterministic test 1",
  });
  await expect(previousSession).toBeSelected();

  await page.getByRole("button", { name: "New thread" }).click();
  const freshSession = page.getByRole("button", {
    name: "Deterministic test 2",
  });
  await expect(freshSession).toBeSelected({ timeout: 5_000 });
  await expect(previousSession).toBeDeselected();
  await expect(
    page.getByRole("label", {
      name: "Fake Pi completed: Explain the fixture",
    }),
  ).toBeAbsent();

  await previousSession.click();
  await expect(previousSession).toBeSelected({ timeout: 5_000 });
  await expect(freshSession).toBeDeselected();
  await expect(
    page.getByRole("label", {
      name: "Fake Pi completed: Explain the fixture",
    }),
  ).toHaveCount(1, { timeout: 5_000 });
});

test("aborts a running response and returns the session to ready", async ({
  page,
}) => {
  const composer = page.getByRole("textbox", {
    name: "Ask this agent to work in its repository…",
  });
  await composer.type("Wait for abort");
  await page.getByRole("button", { name: "Send" }).click();

  const stop = page.getByRole("button", { name: "Stop" });
  await expect(stop).toBeEnabled({ timeout: 5_000 });
  await stop.click();
  await expect(stop).toBeAbsent({ timeout: 5_000 });
  await expect(
    page.getByRole("combobox", { name: "Choose model" }),
  ).toBeEnabled();
  await expect(
    page.getByRole("combobox", { name: "Choose model" }),
  ).toHaveValue("gpt-5");
});

test("opens and closes an embedded native terminal panel", async ({ page }) => {
  const toggle = page.getByRole("button", { name: "Toggle terminal" });
  await expect(toggle).toBeEnabled();
  await toggle.click();
  await expect(toggle).toBePressed();
  await expect(
    page.getByRole("region", { name: "Terminal panel" }),
  ).toHaveCount(1);
  await expect(
    page.getByRole("tablist", { name: "Terminal sessions" }),
  ).toHaveCount(1);

  const terminal = page.getByRole("textbox", { name: "Terminal 1" });
  await terminal.type("printf wabou-terminal-ready");
  await terminal.press("Enter");
  await expect(terminal).toBeFocused();

  await page.getByRole("button", { name: "Project 2" }).click();
  await expect(
    page.getByRole("region", { name: "Terminal panel" }),
  ).toBeAbsent();

  await page.getByRole("button", { name: "Workspace Agent" }).click();
  await expect(
    page.getByRole("region", { name: "Terminal panel" }),
  ).toBeAbsent();
  await expect(
    page.getByRole("button", { name: "Toggle terminal" }),
  ).toBeUnpressed();
});

test("keeps retained layout stable across repeated agent switches", async ({
  page,
}) => {
  const first = page.getByRole("button", { name: "Workspace Agent" });
  const second = page.getByRole("button", { name: "Project 2" });
  const composer = page.getByRole("textbox", {
    name: "Ask this agent to work in its repository…",
  });
  const initialComposer = await composer.snapshot();
  const initialFirst = await first.snapshot();

  for (let iteration = 0; iteration < 8; iteration += 1) {
    await second.click();
    await expect(second).toBeSelected();
    await first.click();
    await expect(first).toBeSelected();
    await expect(composer).toHaveBounds(initialComposer.bounds, {
      tolerance: 0.5,
    });
    await expect(first).toHaveBounds(initialFirst.bounds, { tolerance: 0.5 });
  }

  await expect(
    page.getByRole("label", {
      name: "Fake Pi completed: Explain the fixture",
    }),
  ).toHaveCount(1);
});

test("recovers after the Pi process exits unexpectedly", async ({ page }) => {
  const composer = page.getByRole("textbox", {
    name: "Ask this agent to work in its repository…",
  });
  await composer.press("a", { control: true });
  await composer.type("Exit fixture");
  await page.getByRole("button", { name: "Send" }).click();

  const restart = page.getByRole("button", { name: "Start agent" });
  await expect(restart).toBeEnabled({ timeout: 5_000 });
  await restart.click();
  await expect(composer).toBeInViewport({ timeout: 5_000 });
  await expect(
    page.getByRole("combobox", { name: "Choose model" }),
  ).toBeEnabled();
  await page.getByRole("button", { name: "Commands" }).click();
  await expect(page.getByRole("option", { name: "/fixture" })).toBeInViewport();
  await expect(page.getByRole("option", { name: "/subagents" })).toBeAbsent();
  await page.getByRole("button", { name: "Commands" }).click();
});

test(
  "prepares a default workspace after deleting the last project",
  async ({ page }) => {
    const deleteCurrentProject = async (name: string) => {
      await page.getByRole("button", { name: "Settings" }).click();
      await page.getByRole("heading", { name: "Settings" }).waitFor();
      await page.getByRole("textbox", { name: "Project name" }).wheel(5_000);
      await page.getByRole("button", { name: "Delete project" }).click();
      await page.getByRole("button", { name: `Delete ${name}?` }).click();
    };

    await deleteCurrentProject("Workspace Agent");
    await expect(
      page.getByRole("button", { name: "Project 2" }),
    ).toBeSelected();
    await deleteCurrentProject("Project 2");

    await expect(
      page.getByRole("button", { name: "Project 3" }),
    ).toBeSelected();
    await expect(page.getByRole("button", { name: "Start agent" })).toBeEnabled(
      { timeout: 5_000 },
    );
  },
  { timeout: 15_000 },
);
