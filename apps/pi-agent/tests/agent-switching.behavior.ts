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
  const first = page.getByRole("button", { name: "Agent 1" });
  await expect(first).toBeSelected();

  await page.getByRole("button", { name: "Add project" }).click();
  const second = page.getByRole("button", { name: "Agent 2" });
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

test("updates the agent through settings without losing its conversation", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Settings" }).click();
  await page.getByRole("heading", { name: "Settings" }).waitFor();
  const name = page.getByRole("textbox", { name: "Agent name" });
  await expect(name).toBeInViewport();
  await name.click();
  await name.press("a", { control: true });
  await name.type("Workspace Agent");
  await expect(name).toHaveValue("Workspace Agent");
  await page.getByRole("button", { name: "Back to agents" }).click();
  await expect(
    page.getByRole("button", { name: "Workspace Agent" }),
  ).toBeSelected();
  await expect(
    page.getByRole("label", {
      name: "Fake Pi completed: Explain the fixture",
    }),
  ).toHaveCount(1);
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
  await new Promise((resolve) => setTimeout(resolve, 300));
  await expect(terminal).toBeFocused();

  await page.getByRole("button", { name: "Close terminal panel" }).click();
  await expect(toggle).toBeUnpressed();
  await expect(
    page.getByRole("region", { name: "Terminal panel" }),
  ).toBeAbsent();
});
