import { test } from "@wabou/test";

test("apply a dashboard speed profile for capture", async ({ page }) => {
  await page.getByRole("combobox", { name: "Speed profile" }).click();
  await page.getByRole("option", { name: "Balanced" }).click();
  await page
    .getByRole("status", { name: "Speed profile Balanced applied." })
    .waitFor();
});
