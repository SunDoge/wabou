import { test } from "@wabou/test";

test("capture OceanView", async ({ page }) => {
  await page.getByRole("button", { name: "Open OceanView" }).click();
  await page.getByRole("group", { name: "OceanView fleet command" }).waitFor();
});
