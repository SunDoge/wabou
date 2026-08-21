import { test } from "@wabou/test";

test("capture toggle groups at HiDPI", async ({ page }) => {
  await page.getByRole("button", { name: "Toggle" }).click();
  await page.getByRole("heading", { name: "Toggle" }).waitFor();
  await page.getByRole("button", { name: "Underline" }).click();
  await page
    .getByRole("status", { name: "Formatting: Bold, Underline" })
    .waitFor();
});
