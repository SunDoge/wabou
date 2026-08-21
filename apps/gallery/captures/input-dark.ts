import { test } from "@wabou/test";

test("capture input anatomy in the dark theme", async ({ page }) => {
  await page.getByRole("button", { name: "Input" }).click();
  await page.getByRole("heading", { name: "Input" }).waitFor();
  await page.getByRole("textbox", { name: "Search components" }).waitFor();
});

