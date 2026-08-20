import { test } from "@wabou/test";

test(
  "create and inspect a failed task for capture",
  async ({ page }) => {
    await page.getByRole("button", { name: "Downloads" }).click();
    await page.getByRole("button", { name: "New task" }).click();
    await page.getByRole("button", { name: "Links" }).click();
    await page
      .getByRole("textbox", { name: "Download URLs" })
      .type("http://127.0.0.1:9/wabou-capture.bin");
    await page
      .getByRole("textbox", { name: "Output filename" })
      .type("wabou-capture.bin");
    await page.getByRole("button", { name: "Create task" }).click();
    await page
      .getByRole("alert", { name: "Download failed: wabou-capture.bin" })
      .waitFor({ timeout: 5_000 });
    await page.getByRole("button", { name: "Dismiss" }).click();
    await page
      .getByRole("button", { name: "Inspect wabou-capture.bin" })
      .click({ timeout: 5_000 });
    const overview = page.getByRole("tab", { name: "Task overview" });
    const files = page.getByRole("tab", { name: "Task files" });
    const activity = page.getByRole("tab", { name: "Task activity" });
    await overview.waitFor();
    const [overviewBox, filesBox, activityBox] = await Promise.all([
      overview.snapshot(),
      files.snapshot(),
      activity.snapshot(),
    ]);
    const tabY = overviewBox.bounds.y;
    if (
      Math.abs(filesBox.bounds.y - tabY) > 1 ||
      Math.abs(activityBox.bounds.y - tabY) > 1
    ) {
      throw new Error("task detail tabs must remain on one row");
    }
  },
  { timeout: 10_000 },
);
