import { expect, test } from "@wabou/test";

test(
  "Motrix remains navigable when its recoverable engine cannot start",
  async ({ page }) => {
    await page.getByRole("button", { name: "Dashboard" }).waitFor();
    await page.getByRole("button", { name: "Settings" }).click();
    await page.getByRole("heading", { name: "Settings" }).waitFor();
    await page.getByRole("alert", { name: "Settings load error" }).waitFor();
    await page.getByRole("button", { name: "Retry" }).click();
    await page.getByRole("button", { name: "Downloads" }).click();
    await page.getByRole("heading", { name: "All Downloads" }).waitFor({
      timeout: 500,
    });
    await page.getByRole("button", { name: "Settings" }).click();
    await page.getByRole("group", { name: "Settings categories" }).waitFor({
      timeout: 5_000,
    });
    await page.getByRole("button", { name: "Downloads" }).click();
    await page.getByRole("heading", { name: "All Downloads" }).waitFor();
    await expect(page.getByRole("button", { name: "Pause all" })).toBeEnabled();
    await expect(
      page.getByRole("button", { name: "Resume all" }),
    ).toBeEnabled();
  },
  { timeout: 10_000 },
);
