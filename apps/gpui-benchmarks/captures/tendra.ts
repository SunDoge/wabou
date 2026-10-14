import { test } from "@wabou/test";

test("capture Tendra", async ({ page }) => {
  await page.getByRole("button", { name: "Open Tendra" }).click();
  await page.getByRole("group", { name: "Tendra bid pipeline" }).waitFor();
});
