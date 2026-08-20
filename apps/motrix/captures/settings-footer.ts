import { test } from "@wabou/test";

test("scroll Settings footer into view for capture", async ({ page }) => {
  await page.getByRole("button", { name: "Settings" }).click();
  const general = page.getByRole("tab", { name: "Configure General" });
  await general.waitFor();
  await general.wheel(600);
  await page.getByRole("button", { name: "Save settings" }).waitFor();
});
