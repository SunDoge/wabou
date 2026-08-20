import { expect, test } from "@wabou/test";

test(
  "show a failed task in the compact download list",
  async ({ page }) => {
    await page.getByRole("button", { name: "Downloads" }).click();
    await page.getByRole("button", { name: "New task" }).click();
    await page
      .getByRole("textbox", { name: "Download URLs" })
      .type("http://127.0.0.1:9/compact-row.bin");
    await page
      .getByRole("textbox", { name: "Output filename" })
      .type("compact-row.bin");
    await page.getByRole("button", { name: "Create task" }).click();
    await page
      .getByRole("alert", { name: "Download failed: compact-row.bin" })
      .waitFor({ timeout: 5_000 });
    await page.getByRole("button", { name: "Dismiss" }).click();
    await expect(
      page.getByRole("button", { name: "Retry compact-row.bin" }),
    ).toBeInViewport();
    await expect(
      page.getByRole("button", { name: "Remove compact-row.bin" }),
    ).toBeInViewport();
    const identity = page.getByRole("group", {
      name: "Task identity: compact-row.bin",
    });
    const status = page.getByRole("group", {
      name: "Task status: compact-row.bin",
    });
    await expect(identity).toNotOverlap(status, { tolerance: 1 });
  },
  { timeout: 10_000 },
);
