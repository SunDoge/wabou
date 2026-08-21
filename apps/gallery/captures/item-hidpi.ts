import { test } from "@wabou/test";

test("capture item anatomy at HiDPI", async ({ page }) => {
  await page.getByRole("button", { name: "Item" }).click();
  await page.getByRole("heading", { name: "Item" }).waitFor();
  await page.getByRole("button", { name: "Open" }).waitFor();
});
