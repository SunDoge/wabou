import { expect, test } from "@wabou/test";

test("captures application and project settings", async ({ page }) => {
  await page.getByRole("button", { name: "Start agent" }).click();
  await page.getByRole("button", { name: "Settings" }).click();
  await expect(page.getByRole("heading", { name: "Settings" })).toHaveCount(1);
  await expect(
    page.getByRole("tab", { name: "Project settings" }),
  ).toBeSelected();
});
