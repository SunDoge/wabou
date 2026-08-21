import { test } from "@wabou/test";

test("capture drawer", async ({ page }) => {
  await page.getByRole("button", { name: "Theme: Dark", index: 0 }).click();
  await page.getByRole("button", { name: "Drawer" }).click();
  await page.getByRole("heading", { name: "Drawer" }).waitFor();
  await page.getByRole("button", { name: "Open drawer" }).click();
  await page.getByRole("dialog", { name: "Create task" }).waitFor();
});
