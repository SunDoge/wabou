import { test } from "@wabou/test";

test("capture complete field anatomy at HiDPI", async ({ page }) => {
  await page.getByRole("button", { name: "Field & input group" }).click();
  await page.getByRole("heading", { name: "Field & input group" }).waitFor();
  await page.getByRole("alert", { name: "Use lowercase letters, numbers and hyphens only." }).waitFor();
});
