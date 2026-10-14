import { test } from "@wabou/test";

test("capture Omanta", async ({ page }) => {
  await page.getByRole("button", { name: "Open Omanta" }).click();
  await page.getByRole("group", { name: "Omanta file browser" }).waitFor();
});
