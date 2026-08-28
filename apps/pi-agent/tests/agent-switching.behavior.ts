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
