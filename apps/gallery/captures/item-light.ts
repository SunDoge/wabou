import { test } from "@wabou/test";

test("capture item anatomy in the light theme", async ({ page }) => {
  await page.getByRole("button", { name: "Theme: Dark", index: 0 }).click();
  await page.getByRole("button", { name: "Item" }).click();
  await page.getByRole("heading", { name: "Item" }).waitFor();
  await page.getByRole("button", { name: "Open" }).waitFor();
});
