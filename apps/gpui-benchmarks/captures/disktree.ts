import { test } from "@wabou/test";

test("capture DiskTree", async ({ page }) => {
  await page.getByRole("button", { name: "Open DiskTree" }).click();
  await page.getByRole("group", { name: "DiskTree explorer" }).waitFor();
});
