import { test } from "@wabou/test";

test("capture the application-defined Rust widget", async ({ page }) => {
  await page.getByRole("button", { name: "Native window" }).click();
  await page.getByRole("heading", { name: "Native window" }).waitFor();
});
