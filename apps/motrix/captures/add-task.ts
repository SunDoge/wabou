import { test } from "@wabou/test";

test("open the add-task dialog for capture", async ({ page }) => {
  await page.getByRole("button", { name: "New task" }).click();
  await page.getByRole("dialog", { name: "Add download task" }).waitFor();
});
