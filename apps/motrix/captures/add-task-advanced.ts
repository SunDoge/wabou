import { test } from "@wabou/test";

test("expand add-task advanced options for capture", async ({ page }) => {
  await page.getByRole("button", { name: "New task" }).click();
  await page.getByRole("button", { name: "Advanced HTTP options" }).click();
  await page
    .getByRole("textbox", { name: "HTTP request headers" })
    .waitFor();
});
