import { expect, test } from "@wabou/test";

test("opens the Skills catalog for capture", async ({ page }) => {
  await page.getByRole("button", { name: "Skills" }).click();
  await expect(page.getByRole("heading", { name: "Skills" })).toBeInViewport();
});
