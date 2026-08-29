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
  const toolActivity = page.getByRole("button", {
    name: "Worked, 1 tool call",
  });
  await expect(toolActivity).toHaveCount(1);
  await toolActivity.click();
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

test("forks from a retained user message and restores it to the composer", async ({
  page,
}) => {
  const forkMessage = page.getByRole("button", {
    name: "Fork from this message",
  });
  await expect(forkMessage).toBeEnabled({ timeout: 5_000 });
  await forkMessage.click();

  const dialog = page.getByRole("alertdialog", {
    name: "Fork from this message?",
  });
  await expect(dialog).toBeInViewport();
  await dialog.getByRole("button", { name: "Fork" }).click();
  await expect(dialog).toBeAbsent();

  const composer = page.getByRole("textbox", {
    name: "Ask this agent to work in its repository…",
  });
  await expect(composer).toHaveValue("Explain the fixture", { timeout: 5_000 });
  await composer.press("a", { control: true });
  await composer.press("Backspace");
  await expect(composer).toHaveValue("");
});

test("keeps conversation turn navigation synchronized with native scrolling", async ({
  page,
  window,
}) => {
  await window.resize(window.current, 1_180, 460);
  const composer = page.getByRole("textbox", {
    name: "Ask this agent to work in its repository…",
  });
  await composer.type("Verify the navigation rail");
  await page.getByRole("button", { name: "Send" }).click();
  await expect(
    page.getByRole("label", {
      name: "Fake Pi completed: Verify the navigation rail",
    }),
  ).toHaveCount(1, { timeout: 5_000 });

  const first = page.getByRole("button", {
    name: "Jump to turn 1: Explain the fixture",
  });
  const second = page.getByRole("button", {
    name: "Jump to turn 2: Verify the navigation rail",
  });
  await expect(second).toBeCurrent("step");

  await page
    .getByRole("region", { name: "Assistant response", index: 0 })
    .wheel(-5_000);
  await expect(first).toBeCurrent("step");

  await second.click();
  await expect(second).toBeCurrent("step");
  await window.resize(window.current, 1_200, 800);
});

test("round-trips a Pi extension UI request through the native dialog", async ({
  page,
}) => {
  const composer = page.getByRole("textbox", {
    name: "Ask this agent to work in its repository…",
  });
  await composer.type("Exercise extension UI");
  await page.getByRole("button", { name: "Send" }).click();

  const dialog = page.getByRole("dialog", { name: "Choose fixture mode" });
  await expect(dialog).toBeInViewport({ timeout: 5_000 });
  await expect(
    dialog.getByRole("label", {
      name: "The deterministic extension is waiting for a native UI response.",
    }),
  ).toHaveCount(1);
  await dialog.getByRole("option", { name: "Careful" }).click();

  await expect(dialog).toBeAbsent({ timeout: 5_000 });
  await expect(
    page.getByRole("label", { name: "Extension UI selected: Careful" }),
  ).toHaveCount(1, { timeout: 5_000 });
  await expect(page.getByRole("button", { name: "Stop" })).toBeAbsent();
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
  const firstSession = page.getByRole("button", {
    name: "Deterministic test 1",
  });
  await expect(first).toBeDeselected();
  await expect(firstSession).toBeSelected();

  await page.getByRole("button", { name: "Add project" }).click();
  const second = page.getByRole("button", { name: "Project 2" });
  await expect(second).toBeSelected();

  await first.click();
  await expect(first).toBeDeselected();
  await expect(firstSession).toBeSelected();
  await expect(second).toBeDeselected();
  await expect(
    page.getByRole("label", {
      name: "Fake Pi completed: Verify the navigation rail",
    }),
  ).toHaveCount(1);
});

test("changes model through the native combobox overlay", async ({ page }) => {
  const model = page.getByRole("combobox", { name: "Choose model" });
  const thinking = page.getByRole("combobox", { name: "Thinking level" });
  await model.click();
  await page.getByRole("option", { name: "Alternative model" }).click();
  await expect(model).toHaveValue("Alternative model");
  await expect(thinking).toHaveValue("medium");
});

test("updates project and app settings without losing its conversation", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Settings" }).click();
  await page.getByRole("heading", { name: "Settings" }).waitFor();
  const name = page.getByRole("textbox", { name: "Project name" });
  await expect(name).toBeInViewport();
  const autoCompaction = page.getByRole("switch", {
    name: "Automatic context compaction",
  });
  await expect(autoCompaction).toBeChecked();
  await autoCompaction.click();
  await expect(autoCompaction).toBeUnchecked({ timeout: 5_000 });
  const steering = page.getByRole("combobox", { name: "Steering messages" });
  await steering.click();
  await page.getByRole("option", { name: "All queued messages" }).click();
  await expect(steering).toHaveValue("All queued messages", { timeout: 5_000 });
  const followUp = page.getByRole("combobox", { name: "Follow-up messages" });
  await followUp.click();
  await page.getByRole("option", { name: "All queued messages" }).click();
  await expect(followUp).toHaveValue("All queued messages", {
    timeout: 5_000,
  });
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
  await proxy.wheel(-600);
  const subagents = page.getByRole("switch", { name: "Enable subagents" });
  await expect(subagents).toBeChecked();
  await subagents.click();
  await expect(subagents).toBeUnchecked();

  await subagents.wheel(-300);
  await page.getByRole("button", { name: "中文" }).click();
  await page.getByRole("heading", { name: "设置" }).waitFor();
  await page.getByRole("button", { name: "返回项目" }).click();
  await page.getByRole("button", { name: "设置" }).click();
  await page.getByRole("heading", { name: "设置" }).waitFor();
  await page.getByRole("textbox", { name: "项目名称" }).wheel(800);
  await page.getByRole("button", { name: "English" }).click();
  await page.getByRole("heading", { name: "Settings" }).waitFor();

  await page.getByRole("button", { name: "Back to projects" }).click();
  await expect(
    page.getByRole("button", { name: "Workspace Agent" }),
  ).toBeDeselected();
  await expect(
    page.getByRole("button", { name: "Deterministic test 1" }),
  ).toBeSelected();
  await expect(
    page.getByRole("label", {
      name: "Fake Pi completed: Verify the navigation rail",
    }),
  ).toHaveCount(1);
});

test("creates a fresh session and restores the previous transcript", async ({
  page,
}) => {
  const previousSession = page.getByRole("button", {
    name: "Deterministic test 1",
  });
  await expect(previousSession).toBeSelected({ timeout: 5_000 });

  await page.getByRole("button", { name: "New thread" }).click();
  const freshSession = page.getByRole("button", {
    name: "Deterministic test 2",
  });
  await expect(freshSession).toBeSelected({ timeout: 5_000 });
  await expect(previousSession).toBeDeselected();
  await expect(
    page.getByRole("label", {
      name: "Fake Pi completed: Verify the navigation rail",
    }),
  ).toBeAbsent();

  await previousSession.click();
  await expect(previousSession).toBeSelected({ timeout: 5_000 });
  await expect(freshSession).toBeDeselected();
  await expect(
    page.getByRole("label", {
      name: "Fake Pi completed: Verify the navigation rail",
    }),
  ).toHaveCount(1, { timeout: 5_000 });
  await expect(
    page.getByRole("button", { name: "Search conversation" }),
  ).toHaveCount(1);
});

test("renames the current session and refreshes its sidebar entry", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Rename session" }).click();
  const name = page.getByRole("textbox", { name: "Session name" });
  await name.press("a", { control: true });
  await name.type("Reviewed fixture session");
  await page.getByRole("button", { name: "Save" }).click();

  await expect(
    page.getByRole("button", { name: "Reviewed fixture session" }),
  ).toHaveCount(1, { timeout: 5_000 });
  await expect(
    page.getByRole("button", { name: "Deterministic test 1" }),
  ).toBeAbsent();
});

test("clones and compacts a session without losing its transcript", async ({
  page,
}) => {
  const actions = page.getByRole("button", { name: "Session actions" });
  await actions.click();
  await page.getByRole("menuitem", { name: "Clone current branch" }).click();

  const clone = page.getByRole("button", { name: "Deterministic test 3" });
  await expect(clone).toBeSelected({ timeout: 5_000 });
  await expect(
    page.getByRole("label", {
      name: "Fake Pi completed: Verify the navigation rail",
    }),
  ).toHaveCount(1, { timeout: 5_000 });

  await actions.click();
  await page.getByRole("menuitem", { name: "Compact context" }).click();
  await expect(
    page.getByRole("label", {
      name: "Fake Pi completed: Verify the navigation rail",
    }),
  ).toHaveCount(1, { timeout: 5_000 });
  await expect(
    page.getByRole("button", { name: "Search conversation" }),
  ).toHaveCount(1);
});

test("exports the active session through the native save dialog", async ({
  page,
  effects,
  files,
}) => {
  const exportPath = files.writeText("exports/session.html", "");
  effects.respond("dialogSave", [exportPath]);
  await page.getByRole("button", { name: "Session actions" }).click();
  await page.getByRole("menuitem", { name: "Export as HTML" }).click();

  await expect(
    page.getByRole("label", { name: "Session exported" }),
  ).toHaveCount(1, { timeout: 5_000 });
  await page.getByRole("button", { name: "Dismiss Session exported" }).click();
});

test("aborts a running response and returns the session to ready", async ({
  page,
}) => {
  const composer = page.getByRole("textbox", {
    name: "Ask this agent to work in its repository…",
  });
  await composer.type("Wait for abort");
  await expect(composer).toHaveValue("Wait for abort");
  await page.getByRole("button", { name: "Send" }).click();
  await expect(composer).toHaveValue("", { timeout: 5_000 });

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
  await expect(
    page.getByRole("label", {
      name: "Fake Pi completed: Verify the navigation rail",
    }),
  ).toHaveCount(1);
});

test("searches the retained transcript and closes without changing sessions", async ({
  page,
}) => {
  const toggle = page.getByRole("button", { name: "Search conversation" });
  await expect(toggle).toBeEnabled({ timeout: 5_000 });
  await expect(toggle).toNotOverlap(
    page.getByRole("combobox", { name: "Choose model" }),
  );
  await expect(toggle).toNotOverlap(
    page.getByRole("combobox", { name: "Thinking level" }),
  );
  await toggle.click();
  await expect(toggle).toBePressed();
  await expect(
    page.getByRole("group", { name: "Search conversation" }),
  ).toHaveCount(1);
  const search = page.getByRole("textbox", { name: "Search conversation" });
  await expect(search).toHaveCount(1);
  await search.type("Verify the navigation rail");

  const previous = page.getByRole("button", { name: "Previous match" });
  const next = page.getByRole("button", { name: "Next match" });
  await expect(previous).toBeEnabled();
  await expect(next).toBeEnabled();
  await next.click();
  await previous.click();
  await page.getByRole("button", { name: "Close search" }).click();

  await expect(search).toBeAbsent();
  await expect(
    page.getByRole("button", { name: "Deterministic test 3" }),
  ).toBeSelected();
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

test("creates and closes terminal tabs without remounting the survivor", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Toggle terminal" }).click();
  const first = page.getByRole("textbox", { name: "Terminal 1" });
  await expect(first).toHaveCount(1);

  await page.getByRole("button", { name: "New terminal" }).click();
  const second = page.getByRole("textbox", { name: "Terminal 2" });
  await expect(second).toHaveCount(1);
  await second.type("printf wabou-second-terminal");
  await second.press("Enter");
  await expect(second).toBeFocused();

  await page.getByRole("button", { name: "Close terminal 2" }).click();
  await expect(second).toBeAbsent();
  await expect(first).toHaveCount(1);
  await expect(first).toBeFocused();

  await page.getByRole("button", { name: "Close terminal panel" }).click();
  await expect(
    page.getByRole("region", { name: "Terminal panel" }),
  ).toBeAbsent();
});

test("keeps retained layout stable across repeated agent switches", async ({
  page,
}) => {
  const first = page.getByRole("button", { name: "Workspace Agent" });
  const firstSession = page.getByRole("button", {
    name: "Deterministic test 3",
  });
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
    await expect(first).toBeDeselected();
    await expect(firstSession).toBeSelected();
    await expect(composer).toHaveBounds(initialComposer.bounds, {
      tolerance: 0.5,
    });
    await expect(first).toHaveBounds(initialFirst.bounds, { tolerance: 0.5 });
  }

  await expect(
    page.getByRole("label", {
      name: "Fake Pi completed: Verify the navigation rail",
    }),
  ).toHaveCount(1);
});

test("attaches a native-picked image to the Pi prompt", async ({
  page,
  effects,
  files,
}) => {
  const imagePath = files.writeText("attachments/fixture.png", "fixture");
  effects.respond("dialogOpen", [imagePath]);
  await page.getByRole("button", { name: "Attach images" }).click();

  const attachments = page.getByRole("group", { name: "Attached images" });
  await expect(attachments).toHaveCount(1);
  await expect(
    attachments.getByRole("button", { name: "Remove fixture.png" }),
  ).toHaveCount(1);

  const composer = page.getByRole("textbox", {
    name: "Ask this agent to work in its repository…",
  });
  await composer.type("Inspect attached image");
  await page.getByRole("button", { name: "Send" }).click();

  await expect(
    page.getByRole("button", { name: "Remove fixture.png" }),
  ).toBeAbsent({ timeout: 5_000 });
  await expect(attachments).toHaveCount(1);
  await expect(
    page.getByRole("label", {
      name: "Fixture received 1 image attachment",
    }),
  ).toHaveCount(1, { timeout: 5_000 });
});

test("cancels a blocking Pi extension request and resumes the agent", async ({
  page,
}) => {
  const composer = page.getByRole("textbox", {
    name: "Ask this agent to work in its repository…",
  });
  await composer.type("Exercise extension UI");
  await page.getByRole("button", { name: "Send" }).click();

  const dialog = page.getByRole("dialog", { name: "Choose fixture mode" });
  await expect(dialog).toBeInViewport({ timeout: 5_000 });
  await dialog.getByRole("button", { name: "Cancel" }).click();

  await expect(dialog).toBeAbsent();
  await expect(
    page.getByRole("label", { name: "Extension UI selected: cancelled" }),
  ).toHaveCount(1, { timeout: 5_000 });
  await expect(page.getByRole("button", { name: "Stop" })).toBeAbsent();
});

test(
  "browses an isolated workspace and attaches a file as context",
  async ({ page, files }) => {
    const fixturePath = files.writeText(
      "project-two/README.md",
      "# Project two\n\nA deterministic workspace fixture.\n",
    );
    const workspace = fixturePath.replace(/[\\/][^\\/]+$/, "");

    await page.getByRole("button", { name: "Project 2" }).click();
    await page.getByRole("button", { name: "Settings" }).click();
    const workspaceInput = page.getByRole("textbox", { name: "Workspace" });
    await workspaceInput.click();
    await workspaceInput.press("a", { control: true });
    await workspaceInput.type(workspace);
    await expect(workspaceInput).toHaveValue(workspace);
    await page.getByRole("button", { name: "Back to projects" }).click();

    await page.getByRole("button", { name: "Start agent" }).click();
    await page
      .getByRole("textbox", {
        name: "Ask this agent to work in its repository…",
      })
      .waitFor({ timeout: 5_000 });

    await page.getByRole("button", { name: "Workspace files" }).click();
    const panel = page.getByRole("region", { name: "Workspace files" });
    await expect(panel).toBeInViewport();
    const readme = panel.getByRole("button", { name: "README.md" });
    await expect(readme).toHaveCount(1, { timeout: 5_000 });
    await readme.click();
    const addToContext = panel.getByRole("button", {
      name: "Add to context",
    });
    await expect(addToContext).toHaveCount(1, { timeout: 5_000 });
    await addToContext.click();
    await expect(
      page.getByRole("group", { name: "Context files" }),
    ).toHaveCount(1);
    await expect(
      page.getByRole("button", { name: "Remove README.md" }),
    ).toHaveCount(1);
    await panel.getByRole("button", { name: "Close workspace files" }).click();
    await expect(panel).toBeAbsent();

    await page.getByRole("button", { name: "Workspace Agent" }).click();
    await expect(
      page.getByRole("button", { name: "Workspace Agent" }),
    ).toBeDeselected();
    await expect(
      page.getByRole("button", { name: "Deterministic test 3" }),
    ).toBeSelected();
  },
  { timeout: 10_000 },
);

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
  "keeps one usable project instead of implicitly replacing the last one",
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
      page.getByRole("button", { name: "Deterministic test 1" }),
    ).toBeSelected();
    await page.getByRole("button", { name: "Settings" }).click();
    await page.getByRole("heading", { name: "Settings" }).waitFor();
    await page.getByRole("textbox", { name: "Project name" }).wheel(5_000);
    await expect(
      page.getByRole("button", { name: "Delete project" }),
    ).toBeDisabled();
    await page.getByRole("button", { name: "Back to projects" }).click();
  },
  { timeout: 15_000 },
);
