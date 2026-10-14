import { test } from "@wabou/test";

test("capture Skills catalog at two times scale", async ({ page }) => {
  await page.getByRole("button", { name: "Open Skills" }).click();
  await page.getByRole("group", { name: "Skills catalog" }).waitFor();
  await page.getByRole("listitem", { name: "frontend-design" }).waitFor();
});
