import { test } from "@wabou/test";

test(
  "open populated notification history for capture",
  async ({ page }) => {
    const filename =
      "wabou-notification-capture-with-a-very-long-filename-for-narrow-windows.bin";
    await page.getByRole("button", { name: "New task" }).click();
    await page
      .getByRole("textbox", { name: "Download URLs" })
      .type(`http://127.0.0.1:9/${filename}`);
    await page.getByRole("button", { name: "Create task" }).click();
    await page
      .getByRole("alert", {
        name: `Download failed: ${filename}`,
      })
      .waitFor({ timeout: 5_000 });
    await page.getByRole("button", { name: "Dismiss" }).click();
    await page.getByRole("button", { name: "Notifications" }).click();
    await page
      .getByRole("button", {
        name: `View Download failed: ${filename}`,
      })
      .waitFor();
  },
  { timeout: 10_000 },
);
