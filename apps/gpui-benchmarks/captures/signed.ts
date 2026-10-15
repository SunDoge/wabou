import { test } from "@wabou/test";

test("capture Signed", async ({ page }) => {
  await page.getByRole("button", { name: "Open Signed" }).click();
  await page.getByRole("group", { name: "Signed PDF workspace" }).waitFor();
});
