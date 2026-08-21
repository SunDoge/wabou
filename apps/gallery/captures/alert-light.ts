import { test } from "@wabou/test";

test("capture compound alerts", async ({ page }) => {
  await page.getByRole("button", { name: "Alert" }).click();
  await page.getByRole("heading", { name: "Alert" }).waitFor();
  await page.getByRole("alert", { name: "Framework update" }).waitFor();
  await page.getByRole("alert", { name: "Build failed" }).waitFor();
});
