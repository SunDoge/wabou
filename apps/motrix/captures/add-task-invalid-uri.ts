import { test } from "@wabou/test";

test("show unsupported URI validation for capture", async ({ page }) => {
  await page.getByRole("button", { name: "New task" }).click();
  await page
    .getByRole("textbox", { name: "Download URLs" })
    .type("file:///tmp/private");
  await page
    .getByRole("alert", { name: "Download URI validation error" })
    .waitFor();
});
