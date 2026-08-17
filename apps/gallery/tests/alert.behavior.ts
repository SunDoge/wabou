import { test } from "@wabou/test";

test("alerts expose explicit native live-region semantics", async ({ page }) => {
  await page.getByRole("button", { name: "Alert" }).click();
  await page.getByRole("alert", { name: "Heads up" }).waitFor();
  await page.getByRole("alert", { name: "Build failed" }).waitFor();
  await page
    .getByRole("label", { name: "A newer framework build is available." })
    .waitFor();
  await page
    .getByRole("label", { name: "The native bundle could not be linked." })
    .waitFor();
});
