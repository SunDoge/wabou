import { expect, test } from "@wabou/test";

test(
  "create and inspect a failed task for capture",
  async ({ effects, page }) => {
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
    await page
      .getByRole("group", { name: "Task detail status: Failed" })
      .waitFor();
    await expect(files).toHaveSameBoundsAs(overview, ["y"], { tolerance: 1 });
    await expect(activity).toHaveSameBoundsAs(overview, ["y"], {
      tolerance: 1,
    });
    effects.respond("clipboardWrite", null);
    await page.getByRole("button", { name: "Copy source" }).click();
    await page
      .getByRole("status", { name: "Download action completed" })
      .waitFor();
  },
  { timeout: 10_000 },
);
