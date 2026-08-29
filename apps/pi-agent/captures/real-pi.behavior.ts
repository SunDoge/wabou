import { expect, test } from "@wabou/test";

test(
  "captures a completed real Pi conversation",
  async ({ page }) => {
    await page.getByRole("button", { name: "Start agent" }).click();
    const composer = page.getByRole("textbox", {
      name: "Ask this agent to work in its repository…",
    });
    await composer.waitFor();
    await composer.type(
      "Inspect this workspace briefly and explain in one short paragraph what Wabou is.",
    );
    await page.getByRole("button", { name: "Send" }).click();
    const stop = page.getByRole("button", { name: "Stop" });
    await stop.waitFor({ timeout: 30_000 });
    await expect(stop).toBeAbsent({ timeout: 50_000 });
    await page
      .getByRole("button", { name: "Collapse sessions for Project 1" })
      .waitFor({ timeout: 5_000 });
    await page.getByRole("button", { name: "New thread" }).waitFor();
  },
  { timeout: 60_000 },
);
