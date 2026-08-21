import { test } from "@wabou/test";

test("capture navigation viewport at HiDPI", async ({ page }) => {
  await page.getByRole("button", { name: "Navigation menu" }).click();
  await page.getByRole("heading", { name: "Navigation menu" }).waitFor();
  await page.getByRole("menuitem", { name: "Products" }).click();
  await page.getByRole("link", { name: "Wabou Runtime" }).waitFor();
});
